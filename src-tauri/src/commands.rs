use crate::capture;
use crate::gemini;
use crate::state::AppState;
use serde::Serialize;

#[derive(Serialize)]
pub struct Config {
    pub api_key: String,
    pub model: String,
    pub prompt: String,
}

#[tauri::command]
pub fn get_config(state: tauri::State<'_, AppState>) -> Config {
    Config {
        api_key: state.api_key.lock().unwrap().clone(),
        model: state.model.lock().unwrap().clone(),
        prompt: state.prompt.lock().unwrap().clone(),
    }
}

#[tauri::command]
pub fn save_config(
    api_key: String,
    model: String,
    prompt: String,
    state: tauri::State<'_, AppState>,
) {
    *state.api_key.lock().unwrap() = api_key;
    *state.model.lock().unwrap() = model;
    *state.prompt.lock().unwrap() = prompt;
}

#[tauri::command]
pub fn get_last_response(state: tauri::State<'_, AppState>) -> Option<String> {
    state.last_response.lock().unwrap().clone()
}

#[tauri::command]
pub fn get_processing_status(state: tauri::State<'_, AppState>) -> bool {
    *state.is_processing.lock().unwrap()
}

#[tauri::command]
pub async fn capture_and_analyze(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let api_key = state.api_key.lock().unwrap().clone();
    if api_key.is_empty() {
        return Err("API key not configured. Open config panel with Cmd+Shift+C".to_string());
    }

    let model = state.model.lock().unwrap().clone();
    let prompt = state.prompt.lock().unwrap().clone();

    *state.is_processing.lock().unwrap() = true;

    let base64_image = capture::capture_screen()?;
    let result = gemini::analyze_screenshot(&api_key, &model, &base64_image, &prompt).await;

    *state.is_processing.lock().unwrap() = false;

    match result {
        Ok(response) => {
            *state.last_response.lock().unwrap() = Some(response.clone());
            Ok(response)
        }
        Err(e) => {
            *state.last_response.lock().unwrap() = Some(format!("Error: {e}"));
            Err(e)
        }
    }
}

#[tauri::command]
pub fn check_permissions() -> bool {
    capture::check_screen_permission()
}
