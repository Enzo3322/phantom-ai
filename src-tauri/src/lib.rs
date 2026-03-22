mod audio;
mod capture;
mod clipboard_stealth;
mod commands;
mod display_stealth;
mod dodge;
mod env_report;
mod gemini;
mod hallucination;
mod network_stealth;
mod process_stealth;
mod proctor_detect;
mod recording;
mod state;
mod stealth;
mod vad;
mod usage_db;
mod watcher;
mod whisper;

use state::AppState;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, ShortcutState};
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};

pub fn create_panel(app: &tauri::AppHandle, label: &str) {
    let (width, height) = match label {
        "config" => (460.0, 520.0),
        "main" => (380.0, 48.0),
        "welcome" => (500.0, 480.0),
        _ => (400.0, 400.0),
    };

    let resizable = matches!(label, "main");
    let always_on_top = label != "welcome";

    let mut builder = WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
        .title("")
        .inner_size(width, height)
        .decorations(false)
        .transparent(true)
        .skip_taskbar(true)
        .always_on_top(always_on_top)
        .visible(true)
        .resizable(resizable);

    if label == "welcome" {
        builder = builder.center();
    }

    let window = builder.build();

    if let Ok(window) = window {
        let stealth_enabled = app.state::<AppState>().get_stealth_mode();
        stealth::apply_stealth(&window, stealth_enabled);
    }
}

pub fn toggle_window(app: &tauri::AppHandle, label: &str) {
    let needs_focus = label != "main";

    if let Some(window) = app.get_webview_window(label) {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            if needs_focus {
                activate_app();
                let _ = window.show();
                let _ = window.set_focus();
            } else {
                let _ = window.show();
                #[cfg(target_os = "macos")]
                {
                    use cocoa::base::id;
                    if let Ok(ns_window) = window.ns_window() {
                        unsafe {
                            let _: () = msg_send![ns_window as id, orderFrontRegardless];
                        }
                    }
                }
            }
        }
    } else {
        if needs_focus {
            activate_app();
        }
        create_panel(app, label);
    }
}

fn show_window(app: &tauri::AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.show();
        #[cfg(target_os = "macos")]
        {
            use cocoa::base::id;
            if let Ok(ns_window) = window.ns_window() {
                unsafe {
                    let _: () = msg_send![ns_window as id, orderFrontRegardless];
                }
            }
        }
    } else {
        create_panel(app, label);
    }
}

fn hide_all_windows(app: &tauri::AppHandle) {
    // Only hide config panel during capture.
    // Main panel stays alive — stealth mode makes it invisible to screenshots,
    // and hiding it in macOS accessory mode makes it impossible to show again.
    if let Some(window) = app.get_webview_window("config") {
        let _ = window.hide();
    }
}

#[cfg(target_os = "macos")]
fn activate_app() {
    use cocoa::appkit::NSApp;
    unsafe {
        let app = NSApp();
        let _: () = msg_send![app, activateIgnoringOtherApps: true];
    }
}

#[cfg(not(target_os = "macos"))]
fn activate_app() {}

async fn run_on_main<F: FnOnce() + Send + 'static>(app: &tauri::AppHandle, f: F) {
    let _ = app.run_on_main_thread(f);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

async fn handle_capture(app: tauri::AppHandle) {
    eprintln!("[phantom] capture: starting");

    let app_clone = app.clone();
    run_on_main(&app, move || hide_all_windows(&app_clone)).await;
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let state = app.state::<AppState>();

    let api_key = state.get_api_key();
    if api_key.is_empty() {
        let app_clone = app.clone();
        run_on_main(&app, move || show_window(&app_clone, "main")).await;
        let _ = app.emit("capture-error", "API key not configured. Press Cmd+Shift+C to open settings.");
        return;
    }

    let model = state.get_model();
    let prompt = state.get_prompt();
    let response_language = state.get_response_language();

    eprintln!("[phantom] capture: checking permission");
    if !capture::check_screen_permission() {
        let app_clone = app.clone();
        run_on_main(&app, move || show_window(&app_clone, "main")).await;
        let _ = app.emit("capture-error", "Screen recording permission required.");
        return;
    }

    state.set_last_response(None);
    state.set_processing(true);

    eprintln!("[phantom] capture: taking screenshot");
    let capture_result = tokio::task::spawn_blocking(capture::capture_screen).await;

    let base64_image = match capture_result {
        Ok(Ok(img)) => {
            eprintln!("[phantom] capture: screenshot ok, {} bytes base64", img.len());
            img
        }
        Ok(Err(e)) => {
            eprintln!("[phantom] capture: screenshot error: {e}");
            state.set_processing(false);
            state.set_last_response(Some(format!("Error: {e}")));
            let app_clone = app.clone();
            run_on_main(&app, move || show_window(&app_clone, "main")).await;
            return;
        }
        Err(e) => {
            eprintln!("[phantom] capture: task join error: {e}");
            state.set_processing(false);
            state.set_last_response(Some(format!("Error: {e}")));
            let app_clone = app.clone();
            run_on_main(&app, move || show_window(&app_clone, "main")).await;
            return;
        }
    };

    eprintln!("[phantom] capture: showing response panel");
    let app_clone = app.clone();
    run_on_main(&app, move || show_window(&app_clone, "main")).await;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let _ = app.emit("processing-start", "screenshot");

    let spoof_ua = state.get_spoof_user_agent();
    let jitter = state.get_network_jitter();
    let proxy = state.get_proxy_url();
    let proxy_ref = if proxy.is_empty() { None } else { Some(proxy.as_str()) };

    eprintln!("[phantom] capture: calling gemini (model={model})");
    match gemini::analyze_screenshot(&api_key, &model, &base64_image, &prompt, &response_language, spoof_ua, jitter, proxy_ref).await {
        Ok((response, usage)) => {
            eprintln!("[phantom] capture: gemini ok, {} chars", response.len());
            state.set_last_response(Some(response.clone()));
            state.set_processing(false);
            let _ = app.emit("capture-response", serde_json::json!({ "text": response, "source": "screenshot" }));
            if let Some(db_path) = state.get_usage_db_path() {
                usage_db::record_usage(&db_path, "screenshot", &model, usage.input_tokens, usage.output_tokens);
            }
        }
        Err(e) => {
            eprintln!("[phantom] capture: gemini error: {e}");
            state.set_processing(false);
            state.set_last_response(Some(format!("Error: {e}")));
            let _ = app.emit("capture-error", format!("Error: {e}"));
        }
    }
    eprintln!("[phantom] capture: done");
}

async fn handle_toggle_recording(app: tauri::AppHandle) {
    let state = app.state::<AppState>();

    if state.get_recording() {
        let _ = recording::stop(&app);
    } else {
        let app_clone = app.clone();
        run_on_main(&app, move || show_window(&app_clone, "main")).await;

        if let Err(e) = recording::start(&app) {
            let _ = app.emit("transcription-error", e);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }

                    if shortcut.matches(Modifiers::SUPER | Modifiers::SHIFT, Code::KeyS) {
                        let handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            handle_capture(handle).await;
                        });
                    } else if shortcut.matches(Modifiers::SUPER | Modifiers::SHIFT, Code::KeyC) {
                        toggle_window(app, "config");
                    } else if shortcut.matches(Modifiers::SUPER | Modifiers::SHIFT, Code::KeyA) {
                        toggle_window(app, "main");
                    } else if shortcut.matches(Modifiers::SUPER | Modifiers::SHIFT, Code::KeyM) {
                        let handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            handle_toggle_recording(handle).await;
                        });
                    } else if shortcut.matches(Modifiers::SUPER | Modifiers::SHIFT, Code::KeyO) {
                        let handle = app.clone();
                        let state = app.state::<AppState>();
                        if state.get_watcher_active() {
                            watcher::stop_watcher(app);
                        } else {
                            watcher::start_watcher(handle);
                        }
                    }
                })
                .build(),
        )
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::get_last_response,
            commands::get_processing_status,
            commands::check_permissions,
            commands::start_recording,
            commands::stop_recording,
            commands::get_recording_status,
            commands::get_transcription,
            commands::download_whisper_model,
            commands::get_available_models,
            commands::send_transcription_to_gemini,
            commands::open_settings,
            commands::complete_onboarding,
            commands::get_onboarding_status,
            commands::open_external_url,
            commands::scan_proctoring,
            commands::get_detected_proctors,
            commands::toggle_passthrough,
            commands::get_display_info,
            commands::full_proctor_scan,
            commands::get_env_report,
            commands::type_text,
            commands::ephemeral_paste,
            commands::get_watcher_status,
            commands::toggle_watcher,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            stealth::set_accessory_mode();

            let shortcuts = [
                (Modifiers::SUPER | Modifiers::SHIFT, Code::KeyS),
                (Modifiers::SUPER | Modifiers::SHIFT, Code::KeyC),
                (Modifiers::SUPER | Modifiers::SHIFT, Code::KeyM),
                (Modifiers::SUPER | Modifiers::SHIFT, Code::KeyA),
                (Modifiers::SUPER | Modifiers::SHIFT, Code::KeyO),
            ];

            for (modifiers, code) in &shortcuts {
                let shortcut = tauri_plugin_global_shortcut::Shortcut::new(
                    Some(*modifiers),
                    *code,
                );
                app.global_shortcut().register(shortcut)?;
            }

            // Load config synchronously so we can decide which window to show
            let handle = app.handle().clone();
            load_config_from_store(&handle);

            // Show welcome or main panel based on onboarding status
            let state = handle.state::<AppState>();
            if !state.get_has_onboarded() || state.get_api_key().is_empty() {
                create_panel(&handle, "welcome");
            } else {
                create_panel(&handle, "main");
            }

            process_stealth::apply_process_stealth(&handle);
            dodge::start_dodge_watcher(handle.clone());

            // Initialize token usage database
            if let Ok(app_data) = handle.path().app_data_dir() {
                let _ = std::fs::create_dir_all(&app_data);
                let db_path = app_data.join("phantom_usage.db");
                let db_path_str = db_path.to_string_lossy().to_string();
                match usage_db::open_db(&db_path) {
                    Ok(_) => {
                        eprintln!("[phantom] usage db initialized at: {db_path_str}");
                        handle.state::<AppState>().set_usage_db_path(Some(db_path_str));
                    }
                    Err(e) => {
                        eprintln!("[phantom] usage db init failed: {e}");
                    }
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Phantom");
}

fn load_config_from_store(app: &tauri::AppHandle) {
    use tauri_plugin_store::StoreExt;

    if let Ok(store) = app.store("config.json") {
        let state = app.state::<AppState>();

        if let Some(val) = store.get("api_key") {
            if let Some(s) = val.as_str() {
                state.set_api_key(s.to_string());
            }
        }
        if let Some(val) = store.get("model") {
            if let Some(s) = val.as_str() {
                state.set_model(s.to_string());
            }
        }
        if let Some(val) = store.get("prompt") {
            if let Some(s) = val.as_str() {
                state.set_prompt(s.to_string());
            }
        }
        if let Some(val) = store.get("stealth_mode") {
            if let Some(b) = val.as_bool() {
                state.set_stealth_mode(b);
            }
        }
        if let Some(val) = store.get("whisper_model_size") {
            if let Some(s) = val.as_str() {
                state.set_whisper_model_size(s.to_string());
            }
        }
        if let Some(val) = store.get("whisper_language") {
            if let Some(s) = val.as_str() {
                eprintln!("[phantom] loaded whisper_language from store: {s}");
                state.set_whisper_language(s.to_string());
            }
        }
        if let Some(val) = store.get("audio_source") {
            if let Some(s) = val.as_str() {
                state.set_audio_source(s.to_string());
            }
        }
        if let Some(val) = store.get("vocab_seed") {
            if let Some(s) = val.as_str() {
                state.set_vocab_seed(s.to_string());
            }
        }
        if let Some(val) = store.get("response_language") {
            if let Some(s) = val.as_str() {
                state.set_response_language(s.to_string());
            }
        }
        if let Some(val) = store.get("has_onboarded") {
            if let Some(b) = val.as_bool() {
                state.set_has_onboarded(b);
            }
        }
        if let Some(val) = store.get("dodge_on_hover") {
            if let Some(b) = val.as_bool() {
                state.set_dodge_on_hover(b);
            }
        }
        if let Some(val) = store.get("process_disguise_name") {
            if let Some(s) = val.as_str() {
                state.set_process_disguise_name(s.to_string());
            }
        }
        if let Some(val) = store.get("passthrough_mode") {
            if let Some(b) = val.as_bool() {
                state.set_passthrough_mode(b);
            }
        }
        if let Some(val) = store.get("network_jitter") {
            if let Some(b) = val.as_bool() {
                state.set_network_jitter(b);
            }
        }
        if let Some(val) = store.get("proxy_url") {
            if let Some(s) = val.as_str() {
                state.set_proxy_url(s.to_string());
            }
        }
        if let Some(val) = store.get("spoof_user_agent") {
            if let Some(b) = val.as_bool() {
                state.set_spoof_user_agent(b);
            }
        }

        eprintln!(
            "[phantom] config loaded: model_size={}, language={}, audio_source={}",
            state.get_whisper_model_size(),
            state.get_whisper_language(),
            state.get_audio_source(),
        );
    }
}
