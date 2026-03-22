#![allow(unexpected_cfgs, deprecated)]

use crate::state::AppState;
use std::collections::HashSet;
use tauri::{Emitter, Manager};

/// Capture a screenshot to a temp PNG file, read the bytes, delete the file immediately.
pub fn capture_screenshot_png() -> Result<Vec<u8>, String> {
    let tmp_path = std::env::temp_dir().join("phantom_watcher.png");
    let tmp_str = tmp_path.to_string_lossy().to_string();

    let output = std::process::Command::new("screencapture")
        .args(["-x", "-C", "-t", "png", &tmp_str])
        .output()
        .map_err(|e| format!("Failed to run screencapture: {e}"))?;

    if !output.status.success() {
        let _ = std::fs::remove_file(&tmp_path);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("screencapture failed: {stderr}"));
    }

    let bytes =
        std::fs::read(&tmp_path).map_err(|e| format!("Failed to read screenshot file: {e}"))?;

    // ALWAYS delete the temp file after reading bytes
    let _ = std::fs::remove_file(&tmp_path);

    if bytes.is_empty() {
        return Err("Screenshot is empty. Check screen recording permission.".to_string());
    }

    Ok(bytes)
}

/// Run macOS Vision framework OCR on raw image bytes.
#[cfg(target_os = "macos")]
pub fn ocr_image_bytes(image_data: &[u8]) -> Result<String, String> {
    use objc::runtime::{Object, BOOL, YES};
    use objc::{class, msg_send, sel, sel_impl};
    use std::ffi::c_void;
    use std::sync::{Arc, Mutex};

    unsafe {
        // 1. Create NSData from bytes
        let ns_data: *mut Object = msg_send![
            class!(NSData),
            dataWithBytes: image_data.as_ptr() as *const c_void
            length: image_data.len()
        ];
        if ns_data.is_null() {
            return Err("Failed to create NSData from image bytes".to_string());
        }

        // 2. Create NSImage from NSData
        let ns_image: *mut Object = msg_send![class!(NSImage), alloc];
        let ns_image: *mut Object = msg_send![ns_image, initWithData: ns_data];
        if ns_image.is_null() {
            return Err("Failed to create NSImage from data".to_string());
        }

        // 3. Get CGImage from NSImage
        let zero_rect = cocoa::foundation::NSRect::new(
            cocoa::foundation::NSPoint::new(0.0, 0.0),
            cocoa::foundation::NSSize::new(0.0, 0.0),
        );
        let null_ptr: *mut Object = std::ptr::null_mut();
        let cg_image: *mut c_void = msg_send![
            ns_image,
            CGImageForProposedRect: &zero_rect as *const cocoa::foundation::NSRect
            context: null_ptr
            hints: null_ptr
        ];
        if cg_image.is_null() {
            return Err("Failed to get CGImage from NSImage".to_string());
        }

        // 4. Create VNRecognizeTextRequest with completion handler
        let results_store: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let error_store: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        let results_clone = results_store.clone();
        let error_clone = error_store.clone();

        let handler = block::ConcreteBlock::new(
            move |request: *mut Object, error: *mut Object| {
                if !error.is_null() {
                    let desc: *mut Object = msg_send![error, localizedDescription];
                    let cstr: *const std::ffi::c_char = msg_send![desc, UTF8String];
                    if !cstr.is_null() {
                        let err_str = std::ffi::CStr::from_ptr(cstr).to_string_lossy().to_string();
                        *error_clone.lock().unwrap() = Some(err_str);
                    }
                    return;
                }

                let observations: *mut Object = msg_send![request, results];
                if observations.is_null() {
                    return;
                }

                let count: usize = msg_send![observations, count];
                let mut texts = results_clone.lock().unwrap();

                for i in 0..count {
                    let observation: *mut Object = msg_send![observations, objectAtIndex: i];
                    // Get top candidate
                    let candidates: *mut Object =
                        msg_send![observation, topCandidates: 1usize];
                    let candidates_count: usize = msg_send![candidates, count];
                    if candidates_count > 0 {
                        let candidate: *mut Object =
                            msg_send![candidates, objectAtIndex: 0usize];
                        let ns_string: *mut Object = msg_send![candidate, string];
                        if !ns_string.is_null() {
                            let cstr: *const std::ffi::c_char =
                                msg_send![ns_string, UTF8String];
                            if !cstr.is_null() {
                                let text =
                                    std::ffi::CStr::from_ptr(cstr).to_string_lossy().to_string();
                                texts.push(text);
                            }
                        }
                    }
                }
            },
        );
        let handler = handler.copy();

        let text_request: *mut Object =
            msg_send![class!(VNRecognizeTextRequest), alloc];
        let text_request: *mut Object =
            msg_send![text_request, initWithCompletionHandler: &*handler];
        if text_request.is_null() {
            return Err("Failed to create VNRecognizeTextRequest".to_string());
        }

        // 5. Set recognition level to accurate (1) and auto-detect language
        let accurate_level: i64 = 1; // VNRequestTextRecognitionLevelAccurate
        let _: () = msg_send![text_request, setRecognitionLevel: accurate_level];
        let _: () =
            msg_send![text_request, setAutomaticallyDetectsLanguage: YES];

        // 6. Create VNImageRequestHandler with CGImage
        let request_handler: *mut Object =
            msg_send![class!(VNImageRequestHandler), alloc];
        let options: *mut Object = msg_send![class!(NSDictionary), dictionary];
        let request_handler: *mut Object = msg_send![
            request_handler,
            initWithCGImage: cg_image
            options: options
        ];
        if request_handler.is_null() {
            return Err("Failed to create VNImageRequestHandler".to_string());
        }

        // 7. Perform the request
        let requests_array: *mut Object =
            msg_send![class!(NSArray), arrayWithObject: text_request];
        let mut error_ptr: *mut Object = std::ptr::null_mut();
        let success: BOOL = msg_send![
            request_handler,
            performRequests: requests_array
            error: &mut error_ptr
        ];

        if success == objc::runtime::NO {
            if !error_ptr.is_null() {
                let desc: *mut Object = msg_send![error_ptr, localizedDescription];
                let cstr: *const std::ffi::c_char = msg_send![desc, UTF8String];
                if !cstr.is_null() {
                    let err_str = std::ffi::CStr::from_ptr(cstr).to_string_lossy().to_string();
                    return Err(format!("Vision OCR failed: {err_str}"));
                }
            }
            return Err("Vision OCR failed with unknown error".to_string());
        }

        // Check for errors from the completion handler
        if let Some(err) = error_store.lock().unwrap().clone() {
            return Err(format!("Vision OCR error: {err}"));
        }

        let texts = results_store.lock().unwrap();
        Ok(texts.join("\n"))
    }
}

#[cfg(not(target_os = "macos"))]
pub fn ocr_image_bytes(_image_data: &[u8]) -> Result<String, String> {
    Err("Vision OCR is only available on macOS".to_string())
}

/// Compute Jaccard word similarity between two strings.
pub fn compute_similarity(a: &str, b: &str) -> f64 {
    let words_a: HashSet<&str> = a.split_whitespace().collect();
    let words_b: HashSet<&str> = b.split_whitespace().collect();

    if words_a.is_empty() && words_b.is_empty() {
        return 1.0;
    }

    let intersection_count: u32 = words_a.intersection(&words_b).count().try_into().unwrap_or(0);
    let union_count: u32 = words_a.union(&words_b).count().try_into().unwrap_or(0);

    if union_count == 0 {
        return 1.0;
    }

    f64::from(intersection_count) / f64::from(union_count)
}

const FLASH_MODEL: &str = "gemini-2.5-flash";
const PRO_MODEL: &str = "gemini-2.5-pro";
const SIMILARITY_THRESHOLD: f64 = 0.85;
const MIN_INTERVAL_MS: u64 = 2000;
const MAX_INTERVAL_MS: u64 = 10000;
const DEFAULT_INTERVAL_MS: u64 = 3000;
const ERROR_BACKOFF_MS: u64 = 2000;

/// Start the background watcher loop.
#[cfg(target_os = "macos")]
pub fn start_watcher(app: tauri::AppHandle) {
    let state = app.state::<AppState>();
    state.set_watcher_active(true);
    state.set_watcher_interval_ms(DEFAULT_INTERVAL_MS);

    let _ = app.emit("watcher-started", ());

    tauri::async_runtime::spawn(async move {
        loop {
            let state = app.state::<AppState>();
            let interval_ms = state.get_watcher_interval_ms();

            tokio::time::sleep(std::time::Duration::from_millis(interval_ms)).await;

            // Check if watcher is still active
            let state = app.state::<AppState>();
            if !state.get_watcher_active() {
                eprintln!("[phantom] watcher: stopped");
                let _ = app.emit("watcher-stopped", ());
                break;
            }

            eprintln!("[phantom] watcher: tick (interval={}ms)", interval_ms);

            // 1. Capture screenshot
            let _ = app.emit("watcher-stage", "capturing");
            let image_bytes = match tokio::task::spawn_blocking(capture_screenshot_png).await {
                Ok(Ok(bytes)) => bytes,
                Ok(Err(e)) => {
                    eprintln!("[phantom] watcher: screenshot error: {e}");
                    let _ = app.emit(
                        "watcher-ocr-tick",
                        serde_json::json!({ "status": "error", "error": e }),
                    );
                    let state = app.state::<AppState>();
                    let current = state.get_watcher_interval_ms();
                    state.set_watcher_interval_ms((current + ERROR_BACKOFF_MS).min(MAX_INTERVAL_MS));
                    continue;
                }
                Err(e) => {
                    eprintln!("[phantom] watcher: task join error: {e}");
                    continue;
                }
            };

            // 2. Run OCR on the image bytes
            let _ = app.emit("watcher-stage", "extracting");
            let image_bytes_clone = image_bytes.clone();
            let ocr_text = match tokio::task::spawn_blocking(move || {
                ocr_image_bytes(&image_bytes_clone)
            })
            .await
            {
                Ok(Ok(text)) => text,
                Ok(Err(e)) => {
                    eprintln!("[phantom] watcher: OCR error: {e}");
                    let _ = app.emit(
                        "watcher-ocr-tick",
                        serde_json::json!({ "status": "error", "error": e }),
                    );
                    let state = app.state::<AppState>();
                    let current = state.get_watcher_interval_ms();
                    state.set_watcher_interval_ms((current + ERROR_BACKOFF_MS).min(MAX_INTERVAL_MS));
                    continue;
                }
                Err(e) => {
                    eprintln!("[phantom] watcher: OCR task join error: {e}");
                    continue;
                }
            };

            eprintln!(
                "[phantom] watcher: OCR got {} chars",
                ocr_text.len()
            );

            // 3. Compare with last OCR text
            let state = app.state::<AppState>();
            let last_text = state.get_last_ocr_text();
            let similarity = compute_similarity(&last_text, &ocr_text);

            eprintln!("[phantom] watcher: similarity={:.2}", similarity);

            if similarity >= SIMILARITY_THRESHOLD {
                // No significant change — increase interval
                let _ = app.emit("watcher-stage", "idle");
                let _ = app.emit(
                    "watcher-ocr-tick",
                    serde_json::json!({
                        "status": "no_change",
                        "similarity": similarity,
                        "text_length": ocr_text.len()
                    }),
                );
                let state = app.state::<AppState>();
                let current = state.get_watcher_interval_ms();
                state.set_watcher_interval_ms((current + 1000).min(MAX_INTERVAL_MS));
                continue;
            }

            // Change detected — store new text and reset interval
            state.set_last_ocr_text(ocr_text.clone());
            state.set_watcher_interval_ms(MIN_INTERVAL_MS);

            let _ = app.emit(
                "watcher-ocr-tick",
                serde_json::json!({
                    "status": "changed",
                    "similarity": similarity,
                    "text_length": ocr_text.len()
                }),
            );

            // 4. Get API key and settings
            let api_key = state.get_api_key();
            if api_key.is_empty() {
                eprintln!("[phantom] watcher: no API key, skipping");
                continue;
            }

            let spoof_ua = state.get_spoof_user_agent();
            let jitter = state.get_network_jitter();
            let proxy = state.get_proxy_url();
            let proxy_ref = if proxy.is_empty() {
                None
            } else {
                Some(proxy.as_str())
            };
            let response_language = state.get_response_language();

            // 5. Send to Flash model to filter/structure
            let _ = app.emit("watcher-stage", "analyzing");
            let flash_prompt = format!(
                "You are a screen content filter. Analyze the following OCR text extracted from a screen capture.\n\
                Filter out UI noise (menu bars, status bars, window chrome, timestamps, icons).\n\
                Extract and structure only the meaningful content (questions, problems, text, code).\n\
                If there is no relevant educational or work content, respond with exactly: NO_RELEVANT_CONTENT\n\
                Otherwise, return the structured content clearly.\n\n\
                OCR Text:\n{ocr_text}"
            );

            let flash_result = crate::gemini::send_text_prompt(
                &api_key,
                FLASH_MODEL,
                &flash_prompt,
                spoof_ua,
                jitter,
                proxy_ref,
            )
            .await;

            let filtered_text = match flash_result {
                Ok((text, usage)) => {
                    let state = app.state::<AppState>();
                    if let Some(db_path) = state.get_usage_db_path() {
                        crate::usage_db::record_usage(&db_path, "watcher", FLASH_MODEL, usage.input_tokens, usage.output_tokens);
                    }
                    text
                }
                Err(e) => {
                    eprintln!("[phantom] watcher: Flash API error: {e}");
                    let _ = app.emit(
                        "watcher-ocr-tick",
                        serde_json::json!({ "status": "api_error", "error": e }),
                    );
                    let state = app.state::<AppState>();
                    let current = state.get_watcher_interval_ms();
                    state.set_watcher_interval_ms((current + ERROR_BACKOFF_MS).min(MAX_INTERVAL_MS));
                    continue;
                }
            };

            eprintln!(
                "[phantom] watcher: Flash returned {} chars",
                filtered_text.len()
            );

            // 6. Check if Flash says no relevant content
            if filtered_text.trim() == "NO_RELEVANT_CONTENT" {
                eprintln!("[phantom] watcher: no relevant content, skipping Pro");
                let _ = app.emit("watcher-stage", "idle");
                let _ = app.emit(
                    "watcher-ocr-tick",
                    serde_json::json!({ "status": "no_relevant_content" }),
                );
                let state = app.state::<AppState>();
                state.set_watcher_interval_ms(DEFAULT_INTERVAL_MS);
                continue;
            }

            // 7. Send to Pro model for full response
            let _ = app.emit("watcher-stage", "generating");
            let lang_instruction = match response_language.as_str() {
                "auto" | "" => String::new(),
                lang => format!("\n\nIMPORTANT: You MUST respond in {lang}."),
            };

            let pro_prompt = format!(
                "Based on the following structured screen content, provide a helpful, concise response. \
                If there are questions visible, answer them. If there is code, explain or help with it. \
                Be direct and actionable.{lang_instruction}\n\n\
                Content:\n{filtered_text}"
            );

            let pro_result = crate::gemini::send_text_prompt(
                &api_key,
                PRO_MODEL,
                &pro_prompt,
                spoof_ua,
                jitter,
                proxy_ref,
            )
            .await;

            match pro_result {
                Ok((response, usage)) => {
                    eprintln!(
                        "[phantom] watcher: Pro returned {} chars",
                        response.len()
                    );
                    let state = app.state::<AppState>();
                    if let Some(db_path) = state.get_usage_db_path() {
                        crate::usage_db::record_usage(&db_path, "watcher", PRO_MODEL, usage.input_tokens, usage.output_tokens);
                    }
                    state.set_last_response(Some(response.clone()));
                    let _ = app.emit(
                        "capture-response",
                        serde_json::json!({
                            "text": response,
                            "source": "watcher",
                            "model": PRO_MODEL
                        }),
                    );
                    // Stop watcher after successful response
                    state.set_watcher_active(false);
                    let _ = app.emit("watcher-stopped", ());
                    eprintln!("[phantom] watcher: stopped after response");
                    break;
                }
                Err(e) => {
                    eprintln!("[phantom] watcher: Pro API error: {e}");
                    let _ = app.emit(
                        "watcher-ocr-tick",
                        serde_json::json!({ "status": "api_error", "error": e }),
                    );
                    let state = app.state::<AppState>();
                    let current = state.get_watcher_interval_ms();
                    state.set_watcher_interval_ms((current + ERROR_BACKOFF_MS).min(MAX_INTERVAL_MS));
                }
            }
        }
    });
}

#[cfg(not(target_os = "macos"))]
pub fn start_watcher(_app: tauri::AppHandle) {}

/// Stop the watcher by setting the active flag to false.
pub fn stop_watcher(app: &tauri::AppHandle) {
    let state = app.state::<AppState>();
    state.set_watcher_active(false);
    let _ = app.emit("watcher-stopped", ());
}
