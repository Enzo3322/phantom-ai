mod audio;
mod capture;
mod commands;
mod gemini;
mod state;
mod stealth;
mod whisper;

use state::AppState;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, ShortcutState};

pub fn create_panel(app: &tauri::AppHandle, label: &str) {
    let (width, height) = match label {
        "config" => (460.0, 520.0),
        "response" => (380.0, 160.0),
        "transcription" => (420.0, 400.0),
        _ => (400.0, 400.0),
    };

    let resizable = matches!(label, "response" | "transcription");

    let window = WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
        .title("")
        .inner_size(width, height)
        .decorations(false)
        .transparent(true)
        .skip_taskbar(true)
        .always_on_top(true)
        .visible(true)
        .resizable(resizable)
        .build();

    if let Ok(window) = window {
        let stealth_enabled = app.state::<AppState>().get_stealth_mode();
        stealth::apply_stealth(&window, stealth_enabled);
    }
}

fn toggle_window(app: &tauri::AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    } else {
        create_panel(app, label);
    }
}

fn show_window_no_focus(app: &tauri::AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.show();
    } else {
        create_panel(app, label);
    }
}

fn hide_all_windows(app: &tauri::AppHandle) {
    for label in &["config", "response", "transcription"] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.hide();
        }
    }
}

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
        run_on_main(&app, move || show_window_no_focus(&app_clone, "response")).await;
        let _ = app.emit("capture-error", "API key not configured. Press Cmd+Shift+C to open settings.");
        return;
    }

    let model = state.get_model();
    let prompt = state.get_prompt();

    eprintln!("[phantom] capture: checking permission");
    if !capture::check_screen_permission() {
        let app_clone = app.clone();
        run_on_main(&app, move || show_window_no_focus(&app_clone, "response")).await;
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
            run_on_main(&app, move || show_window_no_focus(&app_clone, "response")).await;
            return;
        }
        Err(e) => {
            eprintln!("[phantom] capture: task join error: {e}");
            state.set_processing(false);
            state.set_last_response(Some(format!("Error: {e}")));
            let app_clone = app.clone();
            run_on_main(&app, move || show_window_no_focus(&app_clone, "response")).await;
            return;
        }
    };

    eprintln!("[phantom] capture: showing response panel");
    let app_clone = app.clone();
    run_on_main(&app, move || show_window_no_focus(&app_clone, "response")).await;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let _ = app.emit("processing-start", ());

    eprintln!("[phantom] capture: calling gemini (model={model})");
    match gemini::analyze_screenshot(&api_key, &model, &base64_image, &prompt).await {
        Ok(response) => {
            eprintln!("[phantom] capture: gemini ok, {} chars", response.len());
            state.set_last_response(Some(response.clone()));
            state.set_processing(false);
            let _ = app.emit("capture-response", response);
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
    let is_recording = state.get_recording();

    if is_recording {
        // Stop recording
        let tx = state.recording_stop_tx.lock().unwrap_or_else(|e| e.into_inner()).take();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }
        state.set_recording(false);
        let _ = app.emit("recording-stopped", ());
        eprintln!("[phantom] recording stopped via shortcut");
    } else {
        // Show transcription panel and start recording
        let app_clone = app.clone();
        run_on_main(&app, move || show_window_no_focus(&app_clone, "transcription")).await;

        let model_size = state.get_whisper_model_size();
        let language = state.get_whisper_language();
        let source_str = state.get_audio_source();
        let vocab_seed = state.get_vocab_seed();
        let source = audio::AudioSource::from_str(&source_str);

        eprintln!("[phantom] recording config: model={model_size}, language={language}, source={source_str}, vocab_seed_len={}", vocab_seed.len());

        state.set_recording(true);
        state.set_transcription_text(String::new());

        let (audio_tx, audio_rx) = std::sync::mpsc::channel();

        let stop_flag = std::sync::Arc::new(
            std::sync::atomic::AtomicBool::new(false),
        );

        match audio::start_capture(source, audio_tx, stop_flag.clone()) {
            Ok(()) => {
                let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
                *state.recording_stop_tx.lock().unwrap_or_else(|e| e.into_inner()) = Some(stop_tx);

                let flag_clone = stop_flag.clone();
                std::thread::spawn(move || {
                    let _ = stop_rx.recv();
                    flag_clone.store(true, std::sync::atomic::Ordering::Relaxed);
                });

                whisper::start_transcription(
                    app.clone(),
                    audio_rx,
                    stop_flag,
                    model_size,
                    language,
                    vocab_seed,
                );

                let _ = app.emit("recording-started", ());
                eprintln!("[phantom] recording started via shortcut");
            }
            Err(e) => {
                state.set_recording(false);
                let _ = app.emit("transcription-error", format!("Failed to start recording: {e}"));
                eprintln!("[phantom] recording start failed: {e}");
            }
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
                        toggle_window(app, "response");
                    } else if shortcut.matches(Modifiers::SUPER | Modifiers::SHIFT, Code::KeyM) {
                        let handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            handle_toggle_recording(handle).await;
                        });
                    } else if shortcut.matches(Modifiers::SUPER | Modifiers::SHIFT, Code::KeyT) {
                        toggle_window(app, "transcription");
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
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            stealth::set_accessory_mode();

            let shortcuts = [
                (Modifiers::SUPER | Modifiers::SHIFT, Code::KeyS),
                (Modifiers::SUPER | Modifiers::SHIFT, Code::KeyC),
                (Modifiers::SUPER | Modifiers::SHIFT, Code::KeyA),
                (Modifiers::SUPER | Modifiers::SHIFT, Code::KeyM),
                (Modifiers::SUPER | Modifiers::SHIFT, Code::KeyT),
            ];

            for (modifiers, code) in &shortcuts {
                let shortcut = tauri_plugin_global_shortcut::Shortcut::new(
                    Some(*modifiers),
                    *code,
                );
                app.global_shortcut().register(shortcut)?;
            }

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                load_config_from_store(&handle);
            });

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
        if let Some(val) = store.get("opacity") {
            if let Some(n) = val.as_f64() {
                state.set_opacity(n);
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

        eprintln!(
            "[phantom] config loaded: model_size={}, language={}, audio_source={}",
            state.get_whisper_model_size(),
            state.get_whisper_language(),
            state.get_audio_source(),
        );
    }
}
