use crate::state::AppState;
use serde::Serialize;

#[derive(Serialize)]
pub struct Config {
    pub api_key: String,
    pub model: String,
    pub prompt: String,
    pub opacity: f64,
    pub stealth_mode: bool,
}

#[tauri::command]
pub fn get_config(state: tauri::State<'_, AppState>) -> Config {
    Config {
        api_key: state.get_api_key(),
        model: state.get_model(),
        prompt: state.get_prompt(),
        opacity: state.get_opacity(),
        stealth_mode: state.get_stealth_mode(),
    }
}

#[tauri::command]
pub fn save_config(
    api_key: String,
    model: String,
    prompt: String,
    opacity: f64,
    stealth_mode: bool,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) {
    state.set_api_key(api_key);
    state.set_model(model);
    state.set_prompt(prompt);
    state.set_opacity(opacity);
    state.set_stealth_mode(stealth_mode);

    #[cfg(target_os = "macos")]
    crate::stealth::set_stealth_for_all_windows(&app, stealth_mode);
}

#[tauri::command]
pub fn get_last_response(state: tauri::State<'_, AppState>) -> Option<String> {
    state.get_last_response()
}

#[tauri::command]
pub fn get_processing_status(state: tauri::State<'_, AppState>) -> bool {
    state.get_processing()
}

#[tauri::command]
pub fn check_permissions() -> bool {
    crate::capture::check_screen_permission()
}
