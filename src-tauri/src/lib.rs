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

fn hide_all_windows(app: &tauri::AppHandle) {
    for label in &["config", "response"] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.hide();
        }
    }
}

async fn handle_capture(app: tauri::AppHandle) {
    hide_all_windows(&app);

    // Wait for windows to fully hide
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let state = app.state::<AppState>();

    let api_key = state.api_key.lock().unwrap().clone();
    if api_key.is_empty() {
        let _ = app.emit("capture-error", "API key not configured. Press Cmd+Shift+C to open settings.");
        toggle_window(&app, "response");
        return;
    }

    let model = state.model.lock().unwrap().clone();
    let prompt = state.prompt.lock().unwrap().clone();

    if !capture::check_screen_permission() {
        let _ = app.emit("capture-error", "Screen recording permission required. Open System Settings > Privacy & Security > Screen Recording and grant access to Phantom.");
        toggle_window(&app, "response");
        return;
    }

    *state.is_processing.lock().unwrap() = true;
    let _ = app.emit("processing-start", ());

    // Show response panel with loading state
    toggle_window(&app, "response");

    let base64_image = match capture::capture_screen() {
        Ok(img) => img,
        Err(e) => {
            *state.is_processing.lock().unwrap() = false;
            let _ = app.emit("capture-error", e);
            return;
        }
    };

    match gemini::analyze_screenshot(&api_key, &model, &base64_image, &prompt).await {
        Ok(response) => {
            *state.last_response.lock().unwrap() = Some(response.clone());
            *state.is_processing.lock().unwrap() = false;
            let _ = app.emit("capture-response", response);
        }
        Err(e) => {
            let error_msg = format!("Error: {e}");
            *state.last_response.lock().unwrap() = Some(error_msg.clone());
            *state.is_processing.lock().unwrap() = false;
            let _ = app.emit("capture-error", error_msg);
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
            commands::capture_and_analyze,
            commands::check_permissions,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            stealth::set_accessory_mode();

            // Register global shortcuts
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

            // Load saved config from store
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
                *state.api_key.lock().unwrap() = s.to_string();
            }
        }
        if let Some(val) = store.get("model") {
            if let Some(s) = val.as_str() {
                *state.model.lock().unwrap() = s.to_string();
            }
        }
        if let Some(val) = store.get("prompt") {
            if let Some(s) = val.as_str() {
                *state.prompt.lock().unwrap() = s.to_string();
            }
        }
    }
}
