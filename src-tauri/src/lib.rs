mod capture;
mod commands;
mod gemini;
mod state;
mod stealth;

use state::AppState;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, ShortcutState};

fn create_panel(app: &tauri::AppHandle, label: &str) {
    let (width, height) = match label {
        "config" => (460.0, 520.0),
        "response" => (380.0, 420.0),
        _ => (400.0, 400.0),
    };

    let window = WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
        .title("")
        .inner_size(width, height)
        .decorations(false)
        .transparent(true)
        .skip_taskbar(true)
        .always_on_top(true)
        .visible(true)
        .resizable(false)
        .center()
        .build();

    if let Ok(window) = window {
        #[cfg(target_os = "macos")]
        {
            use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
            let _ = apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, Some(14.0));
        }

        stealth::apply_stealth(&window);
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

fn show_window(app: &tauri::AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        create_panel(app, label);
    }
}

fn hide_all_windows(app: &tauri::AppHandle) {
    for label in &["config", "response"] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.hide();
        }
    }
}

async fn run_on_main<F: FnOnce() + Send + 'static>(app: &tauri::AppHandle, f: F) {
    let _ = app.run_on_main_thread(f);
    // Give the main thread time to execute
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

async fn handle_capture(app: tauri::AppHandle) {
    eprintln!("[phantom] capture: starting");

    // Hide windows on main thread
    let app_clone = app.clone();
    run_on_main(&app, move || hide_all_windows(&app_clone)).await;
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let state = app.state::<AppState>();

    let api_key = state.get_api_key();
    if api_key.is_empty() {
        let app_clone = app.clone();
        run_on_main(&app, move || show_window(&app_clone, "response")).await;
        let _ = app.emit("capture-error", "API key not configured. Press Cmd+Shift+C to open settings.");
        return;
    }

    let model = state.get_model();
    let prompt = state.get_prompt();

    eprintln!("[phantom] capture: checking permission");
    if !capture::check_screen_permission() {
        let app_clone = app.clone();
        run_on_main(&app, move || show_window(&app_clone, "response")).await;
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
            run_on_main(&app, move || show_window(&app_clone, "response")).await;
            return;
        }
        Err(e) => {
            eprintln!("[phantom] capture: task join error: {e}");
            state.set_processing(false);
            state.set_last_response(Some(format!("Error: {e}")));
            let app_clone = app.clone();
            run_on_main(&app, move || show_window(&app_clone, "response")).await;
            return;
        }
    };

    // Show response panel on main thread, then wait for webview to load
    eprintln!("[phantom] capture: showing response panel");
    let app_clone = app.clone();
    run_on_main(&app, move || show_window(&app_clone, "response")).await;
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
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            stealth::set_accessory_mode();

            let shortcut_s = tauri_plugin_global_shortcut::Shortcut::new(
                Some(Modifiers::SUPER | Modifiers::SHIFT),
                Code::KeyS,
            );
            let shortcut_c = tauri_plugin_global_shortcut::Shortcut::new(
                Some(Modifiers::SUPER | Modifiers::SHIFT),
                Code::KeyC,
            );
            let shortcut_a = tauri_plugin_global_shortcut::Shortcut::new(
                Some(Modifiers::SUPER | Modifiers::SHIFT),
                Code::KeyA,
            );

            app.global_shortcut().register(shortcut_s)?;
            app.global_shortcut().register(shortcut_c)?;
            app.global_shortcut().register(shortcut_a)?;

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
    }
}
