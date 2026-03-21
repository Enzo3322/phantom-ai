use crate::state::AppState;
use serde::Serialize;

#[derive(Serialize)]
pub struct Config {
    pub api_key: String,
    pub model: String,
    pub prompt: String,
    pub glass_effect: bool,
}

#[tauri::command]
pub fn get_config(state: tauri::State<'_, AppState>) -> Config {
    Config {
        api_key: state.get_api_key(),
        model: state.get_model(),
        prompt: state.get_prompt(),
        glass_effect: state.get_glass_effect(),
    }
}

#[tauri::command]
pub fn save_config(
    api_key: String,
    model: String,
    prompt: String,
    glass_effect: bool,
    state: tauri::State<'_, AppState>,
) {
    state.set_api_key(api_key);
    state.set_model(model);
    state.set_prompt(prompt);
    state.set_glass_effect(glass_effect);
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
