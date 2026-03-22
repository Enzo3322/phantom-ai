# Continuous OCR Watcher Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a background screen watcher that periodically OCRs the screen via macOS Vision framework, detects content changes, and processes through a Flash→Pro model pipeline, toggled via `Cmd+Shift+O`.

**Architecture:** A new `watcher.rs` module runs a background async loop (same pattern as `dodge.rs`). Each cycle: screenshot → Vision OCR → delete file → diff → Gemini Flash filter → Gemini Pro response. State fields in `AppState` control the loop and track last OCR text. Frontend shows a watcher indicator and receives responses via existing events.

**Tech Stack:** Rust (Tauri 2), macOS Vision framework via objc bindings, Gemini API (existing client), React/TypeScript frontend.

---

### Task 1: Add watcher state fields to AppState

**Files:**
- Modify: `src-tauri/src/state.rs`

- [ ] **Step 1: Add watcher fields to AppState struct**

Add after the `spoof_user_agent` field (line 29):

```rust
// Watcher (continuous OCR)
pub watcher_active: Mutex<bool>,
pub watcher_interval_ms: Mutex<u64>,
pub last_ocr_text: Mutex<String>,
```

- [ ] **Step 2: Add getter/setter methods**

Add after `set_spoof_user_agent` (line 203):

```rust
pub fn get_watcher_active(&self) -> bool {
    *self.watcher_active.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn set_watcher_active(&self, val: bool) {
    *self.watcher_active.lock().unwrap_or_else(|e| e.into_inner()) = val;
}

pub fn get_watcher_interval_ms(&self) -> u64 {
    *self.watcher_interval_ms.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn set_watcher_interval_ms(&self, val: u64) {
    *self.watcher_interval_ms.lock().unwrap_or_else(|e| e.into_inner()) = val;
}

pub fn get_last_ocr_text(&self) -> String {
    self.last_ocr_text.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

pub fn set_last_ocr_text(&self, val: String) {
    *self.last_ocr_text.lock().unwrap_or_else(|e| e.into_inner()) = val;
}
```

- [ ] **Step 3: Add defaults**

In the `Default` impl, add after `spoof_user_agent`:

```rust
watcher_active: Mutex::new(false),
watcher_interval_ms: Mutex::new(3000),
last_ocr_text: Mutex::new(String::new()),
```

- [ ] **Step 4: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles with no errors

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/state.rs
git commit -m "feat: add watcher state fields to AppState"
```

---

### Task 2: Add `send_to_gemini_text` helper to gemini.rs

The existing `send_to_gemini` function works for transcriptions but hardcodes the prompt format. We need a simpler text-only call for the Flash filter stage.

**Files:**
- Modify: `src-tauri/src/gemini.rs`

- [ ] **Step 1: Add `send_text_prompt` function**

Add at the end of `gemini.rs`:

```rust
pub async fn send_text_prompt(
    api_key: &str,
    model: &str,
    prompt: &str,
    spoof_ua: bool,
    jitter: bool,
    proxy_url: Option<&str>,
) -> Result<String, String> {
    if jitter {
        network_stealth::apply_jitter().await;
    }

    let client = if spoof_ua {
        network_stealth::build_stealth_client(proxy_url)?
    } else {
        reqwest::Client::new()
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let request = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part::Text {
                text: prompt.to_string(),
            }],
        }],
    };

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    let status = response.status();
    let raw = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    if !status.is_success() {
        return Err(format!("Gemini API error ({}): {}", status, &raw[..raw.len().min(300)]));
    }

    let body: GeminiResponse = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    body.candidates
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.content.parts.into_iter().next())
        .map(|p| p.text)
        .ok_or_else(|| "Empty response from Gemini".to_string())
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles (function is unused for now, that's ok)

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/gemini.rs
git commit -m "feat: add send_text_prompt helper for text-only Gemini calls"
```

---

### Task 3: Create watcher.rs — OCR via Vision framework + watcher loop

**Files:**
- Create: `src-tauri/src/watcher.rs`
- Modify: `src-tauri/src/lib.rs` (add `mod watcher;`)

- [ ] **Step 1: Create `watcher.rs` with OCR function**

Create `src-tauri/src/watcher.rs`:

```rust
#![allow(unexpected_cfgs)]

use crate::state::AppState;
use tauri::{Emitter, Manager};
use std::collections::HashSet;

#[cfg(target_os = "macos")]
fn capture_screenshot_png() -> Result<Vec<u8>, String> {
    let tmp_path = std::env::temp_dir().join("phantom_watcher.png");
    let tmp_str = tmp_path.to_string_lossy().to_string();

    let output = std::process::Command::new("screencapture")
        .args(["-x", "-C", "-t", "png", &tmp_str])
        .output()
        .map_err(|e| format!("Failed to run screencapture: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("screencapture failed: {stderr}"));
    }

    let bytes = std::fs::read(&tmp_path)
        .map_err(|e| format!("Failed to read screenshot: {e}"))?;

    // Always delete the file
    let _ = std::fs::remove_file(&tmp_path);

    if bytes.is_empty() {
        return Err("Screenshot is empty".to_string());
    }

    Ok(bytes)
}

#[cfg(target_os = "macos")]
fn ocr_image_bytes(image_data: &[u8]) -> Result<String, String> {
    use objc::{class, msg_send, sel, sel_impl};
    use cocoa::base::{id, nil, BOOL, YES};
    use block::ConcreteBlock;
    use std::sync::{Arc, Mutex};

    unsafe {
        // Create NSData from bytes
        let ns_data: id = msg_send![class!(NSData), alloc];
        let ns_data: id = msg_send![ns_data, initWithBytes:image_data.as_ptr() length:image_data.len()];

        // Create NSImage from data (needed for CGImage)
        let ns_image: id = msg_send![class!(NSImage), alloc];
        let ns_image: id = msg_send![ns_image, initWithData:ns_data];
        if ns_image == nil {
            return Err("Failed to create NSImage from data".to_string());
        }

        // Get CGImage from NSImage
        let ns_rect = cocoa::foundation::NSRect::new(
            cocoa::foundation::NSPoint::new(0.0, 0.0),
            cocoa::foundation::NSSize::new(0.0, 0.0),
        );
        let cg_image: id = msg_send![ns_image, CGImageForProposedRect:&ns_rect context:nil hints:nil];
        if cg_image == nil {
            return Err("Failed to get CGImage".to_string());
        }

        // Create VNRecognizeTextRequest
        let result_store: Arc<Mutex<Result<String, String>>> = Arc::new(Mutex::new(Ok(String::new())));
        let result_clone = result_store.clone();

        let completion_block = ConcreteBlock::new(move |request: id, error: id| {
            if error != nil {
                let desc: id = msg_send![error, localizedDescription];
                let cstr: *const std::ffi::c_char = msg_send![desc, UTF8String];
                let err_str = std::ffi::CStr::from_ptr(cstr).to_string_lossy().to_string();
                *result_clone.lock().unwrap() = Err(err_str);
                return;
            }

            let results: id = msg_send![request, results];
            let count: usize = msg_send![results, count];
            let mut text_parts: Vec<String> = Vec::new();

            for i in 0..count {
                let observation: id = msg_send![results, objectAtIndex:i];
                let candidates: id = msg_send![observation, topCandidates:1usize];
                let candidate_count: usize = msg_send![candidates, count];
                if candidate_count > 0 {
                    let candidate: id = msg_send![candidates, objectAtIndex:0usize];
                    let ns_string: id = msg_send![candidate, string];
                    let cstr: *const std::ffi::c_char = msg_send![ns_string, UTF8String];
                    let s = std::ffi::CStr::from_ptr(cstr).to_string_lossy().to_string();
                    text_parts.push(s);
                }
            }

            *result_clone.lock().unwrap() = Ok(text_parts.join("\n"));
        });
        let completion_block = completion_block.copy();

        let request: id = msg_send![class!(VNRecognizeTextRequest), alloc];
        let request: id = msg_send![request, initWithCompletionHandler:&*completion_block];

        // Set recognition level to accurate (1)
        let _: () = msg_send![request, setRecognitionLevel:1i64];
        // Enable automatic language detection
        let _: () = msg_send![request, setAutomaticallyDetectsLanguage:YES];

        // Create request handler with CGImage
        let handler: id = msg_send![class!(VNImageRequestHandler), alloc];
        let handler: id = msg_send![handler, initWithCGImage:cg_image options:nil];

        // Create NSArray with the request
        let requests: id = msg_send![class!(NSArray), arrayWithObject:request];

        // Perform the request
        let mut error_ptr: id = nil;
        let success: BOOL = msg_send![handler, performRequests:requests error:&mut error_ptr];

        if success != YES {
            if error_ptr != nil {
                let desc: id = msg_send![error_ptr, localizedDescription];
                let cstr: *const std::ffi::c_char = msg_send![desc, UTF8String];
                let err_str = std::ffi::CStr::from_ptr(cstr).to_string_lossy().to_string();
                return Err(format!("Vision request failed: {err_str}"));
            }
            return Err("Vision request failed".to_string());
        }

        result_store.lock().unwrap().clone()
    }
}

fn compute_similarity(a: &str, b: &str) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let words_a: HashSet<&str> = a.split_whitespace().collect();
    let words_b: HashSet<&str> = b.split_whitespace().collect();

    let intersection = words_a.intersection(&words_b).count();
    let union = words_a.union(&words_b).count();

    if union == 0 {
        return 1.0;
    }

    intersection as f64 / union as f64
}

const SIMILARITY_THRESHOLD: f64 = 0.85;
const MIN_INTERVAL_MS: u64 = 2000;
const MAX_INTERVAL_MS: u64 = 10000;
const DEFAULT_INTERVAL_MS: u64 = 3000;
const INTERVAL_INCREMENT_MS: u64 = 1000;
const BACKOFF_INCREMENT_MS: u64 = 2000;

const FLASH_MODEL: &str = "gemini-2.0-flash";
const PRO_MODEL: &str = "gemini-2.5-pro";

const FLASH_PROMPT_TEMPLATE: &str = r#"You are a screen content extractor. Given raw OCR text from a screen capture, do the following:
1. Ignore UI elements: menus, navigation bars, toolbars, status bars, notifications, window controls, tabs, sidebars.
2. Extract the MAIN content the user is looking at.
3. Structure your output as:
   - Context: (what app/site/document is being viewed, one line)
   - Content: (the main text content)
   - Question: (if there's a question or quiz visible, extract it with all options)

If the screen shows nothing meaningful (desktop, lock screen, empty window), reply with exactly: NO_RELEVANT_CONTENT

Raw OCR text:
"#;

#[cfg(target_os = "macos")]
pub fn start_watcher(app: tauri::AppHandle) {
    let state = app.state::<AppState>();
    if state.get_watcher_active() {
        eprintln!("[phantom] watcher: already active, ignoring start");
        return;
    }

    state.set_watcher_active(true);
    state.set_watcher_interval_ms(DEFAULT_INTERVAL_MS);
    state.set_last_ocr_text(String::new());
    let _ = app.emit("watcher-started", ());
    eprintln!("[phantom] watcher: started");

    tauri::async_runtime::spawn(async move {
        loop {
            let state = app.state::<AppState>();

            if !state.get_watcher_active() {
                eprintln!("[phantom] watcher: stopped");
                let _ = app.emit("watcher-stopped", ());
                break;
            }

            let interval_ms = state.get_watcher_interval_ms();
            tokio::time::sleep(std::time::Duration::from_millis(interval_ms)).await;

            // Re-check after sleep
            if !state.get_watcher_active() {
                eprintln!("[phantom] watcher: stopped after sleep");
                let _ = app.emit("watcher-stopped", ());
                break;
            }

            let api_key = state.get_api_key();
            if api_key.is_empty() {
                eprintln!("[phantom] watcher: no API key, skipping cycle");
                continue;
            }

            // Step 1: Screenshot
            let image_data = match tokio::task::spawn_blocking(capture_screenshot_png).await {
                Ok(Ok(data)) => data,
                Ok(Err(e)) => {
                    eprintln!("[phantom] watcher: screenshot error: {e}");
                    continue;
                }
                Err(e) => {
                    eprintln!("[phantom] watcher: task error: {e}");
                    continue;
                }
            };

            // Step 2: OCR (file is already deleted by capture_screenshot_png)
            let ocr_text = match tokio::task::spawn_blocking(move || ocr_image_bytes(&image_data)).await {
                Ok(Ok(text)) => text,
                Ok(Err(e)) => {
                    eprintln!("[phantom] watcher: OCR error: {e}");
                    continue;
                }
                Err(e) => {
                    eprintln!("[phantom] watcher: OCR task error: {e}");
                    continue;
                }
            };

            if ocr_text.trim().is_empty() {
                eprintln!("[phantom] watcher: OCR returned empty text");
                let current_interval = state.get_watcher_interval_ms();
                state.set_watcher_interval_ms((current_interval + INTERVAL_INCREMENT_MS).min(MAX_INTERVAL_MS));
                let _ = app.emit("watcher-ocr-tick", serde_json::json!({ "status": "no_change" }));
                continue;
            }

            // Step 3: Diff
            let last_text = state.get_last_ocr_text();
            let similarity = compute_similarity(&last_text, &ocr_text);
            eprintln!("[phantom] watcher: similarity={:.2}%", similarity * 100.0);

            if similarity > SIMILARITY_THRESHOLD {
                let current_interval = state.get_watcher_interval_ms();
                state.set_watcher_interval_ms((current_interval + INTERVAL_INCREMENT_MS).min(MAX_INTERVAL_MS));
                let _ = app.emit("watcher-ocr-tick", serde_json::json!({ "status": "no_change" }));
                continue;
            }

            // Content changed
            state.set_last_ocr_text(ocr_text.clone());
            state.set_watcher_interval_ms(MIN_INTERVAL_MS);
            let _ = app.emit("watcher-ocr-tick", serde_json::json!({ "status": "processing" }));

            // Check still active before API calls
            if !state.get_watcher_active() {
                break;
            }

            let spoof_ua = state.get_spoof_user_agent();
            let jitter = state.get_network_jitter();
            let proxy = state.get_proxy_url();
            let proxy_ref = if proxy.is_empty() { None } else { Some(proxy.as_str()) };

            // Step 4: Gemini Flash — filter and structure
            let flash_prompt = format!("{FLASH_PROMPT_TEMPLATE}{ocr_text}");
            let flash_result = crate::gemini::send_text_prompt(
                &api_key, FLASH_MODEL, &flash_prompt, spoof_ua, jitter, proxy_ref,
            ).await;

            let structured_content = match flash_result {
                Ok(content) => {
                    if content.trim() == "NO_RELEVANT_CONTENT" {
                        eprintln!("[phantom] watcher: Flash says no relevant content");
                        state.set_watcher_interval_ms(DEFAULT_INTERVAL_MS);
                        continue;
                    }
                    content
                }
                Err(e) => {
                    eprintln!("[phantom] watcher: Flash error: {e}");
                    let _ = app.emit("capture-error", format!("Watcher error: {e}"));
                    let current_interval = state.get_watcher_interval_ms();
                    state.set_watcher_interval_ms((current_interval + BACKOFF_INCREMENT_MS).min(MAX_INTERVAL_MS));
                    continue;
                }
            };

            // Check still active before Pro call
            if !state.get_watcher_active() {
                break;
            }

            // Step 5: Gemini Pro — process and respond
            let prompt = state.get_prompt();
            let response_language = state.get_response_language();
            let lang_instruction = match response_language.as_str() {
                "auto" | "" => String::new(),
                lang => format!("\n\nIMPORTANT: You MUST respond in {lang}."),
            };

            let pro_prompt = format!(
                "{prompt}{lang_instruction}\n\nScreen content:\n{structured_content}"
            );

            let pro_result = crate::gemini::send_text_prompt(
                &api_key, PRO_MODEL, &pro_prompt, spoof_ua, jitter, proxy_ref,
            ).await;

            // Final check before emitting
            if !state.get_watcher_active() {
                break;
            }

            match pro_result {
                Ok(response) => {
                    eprintln!("[phantom] watcher: Pro response ok, {} chars", response.len());
                    state.set_last_response(Some(response.clone()));
                    let _ = app.emit("capture-response", serde_json::json!({
                        "text": response,
                        "source": "watcher"
                    }));
                }
                Err(e) => {
                    eprintln!("[phantom] watcher: Pro error: {e}");
                    let _ = app.emit("capture-error", format!("Watcher error: {e}"));
                    let current_interval = state.get_watcher_interval_ms();
                    state.set_watcher_interval_ms((current_interval + BACKOFF_INCREMENT_MS).min(MAX_INTERVAL_MS));
                }
            }

            state.set_watcher_interval_ms(DEFAULT_INTERVAL_MS);
        }
    });
}

#[cfg(target_os = "macos")]
pub fn stop_watcher(app: &tauri::AppHandle) {
    let state = app.state::<AppState>();
    state.set_watcher_active(false);
    state.set_last_ocr_text(String::new());
    eprintln!("[phantom] watcher: stop requested");
}

#[cfg(not(target_os = "macos"))]
pub fn start_watcher(_app: tauri::AppHandle) {}

#[cfg(not(target_os = "macos"))]
pub fn stop_watcher(_app: &tauri::AppHandle) {}
```

- [ ] **Step 2: Register the module in lib.rs**

In `src-tauri/src/lib.rs`, add after line 17 (`mod whisper;`):

```rust
mod watcher;
```

- [ ] **Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles (may warn about unused, that's ok)

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/watcher.rs src-tauri/src/lib.rs
git commit -m "feat: add watcher module with Vision OCR, diff, and Flash/Pro pipeline"
```

---

### Task 4: Register Cmd+Shift+O shortcut and toggle handler

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add shortcut registration**

In the `shortcuts` array (line 296-301), add a new entry:

```rust
(Modifiers::SUPER | Modifiers::SHIFT, Code::KeyO),
```

- [ ] **Step 2: Add handler for the shortcut**

In the shortcut handler closure (around line 256-261), add before the closing `}` of the last `else if`:

```rust
} else if shortcut.matches(Modifiers::SUPER | Modifiers::SHIFT, Code::KeyO) {
    let handle = app.clone();
    let state = app.state::<AppState>();
    if state.get_watcher_active() {
        watcher::stop_watcher(app);
    } else {
        watcher::start_watcher(handle);
    }
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles with no errors

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: register Cmd+Shift+O shortcut for watcher toggle"
```

---

### Task 5: Add watcher commands for frontend

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs` (register commands)

- [ ] **Step 1: Add commands to commands.rs**

Add at the end of `commands.rs`:

```rust
#[tauri::command]
pub fn get_watcher_status(state: tauri::State<'_, AppState>) -> bool {
    state.get_watcher_active()
}

#[tauri::command]
pub fn toggle_watcher(app: tauri::AppHandle, state: tauri::State<'_, AppState>) {
    if state.get_watcher_active() {
        crate::watcher::stop_watcher(&app);
    } else {
        crate::watcher::start_watcher(app);
    }
}
```

- [ ] **Step 2: Register commands in lib.rs**

In `invoke_handler` (around line 290), add:

```rust
commands::get_watcher_status,
commands::toggle_watcher,
```

- [ ] **Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat: add watcher IPC commands for frontend"
```

---

### Task 6: Add watcher hook and MainPanel indicator

**Files:**
- Create: `src/hooks/useWatcher.ts`
- Modify: `src/hooks/useGemini.ts`
- Modify: `src/components/MainPanel/MainPanel.tsx`
- Modify: `src/components/MainPanel/MainPanel.css`

- [ ] **Step 1: Create useWatcher hook**

Create `src/hooks/useWatcher.ts`:

```typescript
import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface WatcherTickPayload {
  status: "no_change" | "processing";
}

export function useWatcher() {
  const [active, setActive] = useState(false);
  const [lastTick, setLastTick] = useState<string | null>(null);

  useEffect(() => {
    invoke<boolean>("get_watcher_status").then(setActive);
  }, []);

  useEffect(() => {
    const listeners = [
      listen("watcher-started", () => {
        setActive(true);
        setLastTick(null);
      }),
      listen("watcher-stopped", () => {
        setActive(false);
        setLastTick(null);
      }),
      listen<WatcherTickPayload>("watcher-ocr-tick", (event) => {
        setLastTick(event.payload.status);
      }),
    ];

    return () => {
      listeners.forEach((p) => p.then((unlisten) => unlisten()));
    };
  }, []);

  const toggleWatcher = useCallback(async () => {
    await invoke("toggle_watcher");
  }, []);

  return { active, lastTick, toggleWatcher };
}
```

- [ ] **Step 2: Update useGemini to handle "watcher" source**

In `src/hooks/useGemini.ts`, update the `GeminiSource` type (line 5):

```typescript
type GeminiSource = "screenshot" | "transcription" | "watcher" | null;
```

Update the `processing-start` listener to also handle watcher source (line 39):

```typescript
const src = event.payload;
setSource(src === "screenshot" ? "screenshot" : src === "watcher" ? "watcher" : "transcription");
```

- [ ] **Step 3: Add watcher indicator to MainPanel**

In `src/components/MainPanel/MainPanel.tsx`:

Import the hook (after line 14):

```typescript
import { useWatcher } from "../../hooks/useWatcher";
```

Add inside `MainPanel()` function (after line 34):

```typescript
const { active: watcherActive } = useWatcher();
```

Update the `Mode` type (line 23):

```typescript
type Mode = "idle" | "response" | "recording" | "processing" | "error" | "watching";
```

Update the mode calculation — add watching before idle (replace lines 46-52):

```typescript
const mode: Mode = (() => {
    if (activeError) return "error";
    if (isRecording) return "recording";
    if (response) return "response";
    if (geminiLoading) return "processing";
    if (watcherActive) return "watching";
    return "idle";
})();
```

Add the watching view before the idle return (after line 164):

```typescript
// --- WATCHING (watcher active, no response yet) ---
if (mode === "watching") {
    return (
      <div className="main-panel">
        <div className="main-titlebar" data-tauri-drag-region>
          <div className="main-title-left">
            <span className="watcher-dot" />
            <span className="main-title">Watching...</span>
          </div>
        </div>
        <div className="main-body main-processing-body">
          <div className="brain-loading">
            <svg
              className="watcher-eye-icon"
              width="28"
              height="28"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
              <circle cx="12" cy="12" r="3" />
            </svg>
          </div>
        </div>
      </div>
    );
}
```

- [ ] **Step 4: Add watcher CSS**

In `src/components/MainPanel/MainPanel.css`, add:

```css
.watcher-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #3b82f6;
  display: inline-block;
  margin-right: 6px;
  animation: watcher-pulse 2s ease-in-out infinite;
}

@keyframes watcher-pulse {
  0%, 100% { opacity: 1; box-shadow: 0 0 0 0 rgba(59, 130, 246, 0.4); }
  50% { opacity: 0.6; box-shadow: 0 0 0 4px rgba(59, 130, 246, 0); }
}

.watcher-eye-icon {
  animation: watcher-scan 3s ease-in-out infinite;
  color: rgba(255, 255, 255, 0.5);
}

@keyframes watcher-scan {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 1; }
}
```

- [ ] **Step 5: Verify frontend builds**

Run: `npm run build`
Expected: builds with no errors

- [ ] **Step 6: Commit**

```bash
git add src/hooks/useWatcher.ts src/hooks/useGemini.ts src/components/MainPanel/MainPanel.tsx src/components/MainPanel/MainPanel.css
git commit -m "feat: add watcher UI indicator and useWatcher hook"
```

---

### Task 7: Add watcher shortcut to ConfigPanel shortcuts tab

**Files:**
- Modify: `src/components/ConfigPanel/ConfigPanel.tsx`

- [ ] **Step 1: Add watcher shortcut entry**

In the shortcuts tab (after the "Toggle Settings" shortcut-row, around line 621), add:

```tsx
<div className="shortcut-row">
  <div className="shortcut-left">
    <span className="shortcut-icon watcher">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
        <circle cx="12" cy="12" r="3" />
      </svg>
    </span>
    <div className="shortcut-info">
      <span className="shortcut-name">Toggle Watcher</span>
      <span className="shortcut-desc">Start or stop continuous screen OCR</span>
    </div>
  </div>
  <kbd>⌘ ⇧ O</kbd>
</div>
```

- [ ] **Step 2: Verify frontend builds**

Run: `npm run build`
Expected: builds with no errors

- [ ] **Step 3: Commit**

```bash
git add src/components/ConfigPanel/ConfigPanel.tsx
git commit -m "feat: add watcher shortcut to ConfigPanel shortcuts tab"
```

---

### Task 8: Full integration test — build and manual verification

**Files:** None (verification only)

- [ ] **Step 1: Full Rust build**

Run: `cd src-tauri && cargo build`
Expected: compiles successfully

- [ ] **Step 2: Full frontend build**

Run: `npm run build`
Expected: builds successfully

- [ ] **Step 3: Run the Tauri dev server**

Run: `npm run tauri dev`
Expected: app launches

- [ ] **Step 4: Manual verification checklist**

1. Press `Cmd+Shift+O` — watcher indicator should appear in MainPanel
2. Wait for OCR cycles — check terminal logs for `[phantom] watcher:` messages
3. Change screen content — should trigger Flash + Pro pipeline
4. Press `Cmd+Shift+O` again — watcher should stop
5. Verify screenshot temp files are cleaned up: `ls /tmp/phantom_watcher*` should show nothing

- [ ] **Step 5: Commit any fixes if needed**
