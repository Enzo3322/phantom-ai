use crate::process_stealth::DetectedProctor;
use crate::state::AppState;
use serde::Serialize;
use tauri::Manager;

#[tauri::command]
pub fn open_settings(app: tauri::AppHandle) {
    crate::toggle_window(&app, "config");
}

#[tauri::command]
pub async fn complete_onboarding(
    api_key: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;

    state.set_api_key(api_key.clone());
    state.set_has_onboarded(true);

    let store = app.store("config.json").map_err(|e| e.to_string())?;
    store.set("api_key", serde_json::json!(api_key));
    store.set("has_onboarded", serde_json::json!(true));
    store.save().map_err(|e| e.to_string())?;

    // Close welcome window and open main panel
    if let Some(welcome) = app.get_webview_window("welcome") {
        let _ = welcome.close();
    }
    crate::create_panel(&app, "main");

    Ok(())
}

#[tauri::command]
pub fn get_onboarding_status(state: tauri::State<'_, AppState>) -> bool {
    state.get_has_onboarded()
}

#[tauri::command]
pub fn open_external_url(url: String) -> Result<(), String> {
    std::process::Command::new("open")
        .arg(&url)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Serialize)]
pub struct Config {
    pub api_key: String,
    pub model: String,
    pub prompt: String,
    pub stealth_mode: bool,
    pub whisper_model_size: String,
    pub whisper_language: String,
    pub audio_source: String,
    pub vocab_seed: String,
    pub response_language: String,
    pub dodge_on_hover: bool,
    pub process_disguise_name: String,
    pub passthrough_mode: bool,
    pub network_jitter: bool,
    pub proxy_url: String,
    pub spoof_user_agent: bool,
}

#[tauri::command]
pub fn get_config(state: tauri::State<'_, AppState>) -> Config {
    Config {
        api_key: state.get_api_key(),
        model: state.get_model(),
        prompt: state.get_prompt(),
        stealth_mode: state.get_stealth_mode(),
        whisper_model_size: state.get_whisper_model_size(),
        whisper_language: state.get_whisper_language(),
        audio_source: state.get_audio_source(),
        vocab_seed: state.get_vocab_seed(),
        response_language: state.get_response_language(),
        dodge_on_hover: state.get_dodge_on_hover(),
        process_disguise_name: state.get_process_disguise_name(),
        passthrough_mode: state.get_passthrough_mode(),
        network_jitter: state.get_network_jitter(),
        proxy_url: state.get_proxy_url(),
        spoof_user_agent: state.get_spoof_user_agent(),
    }
}

#[tauri::command]
pub fn save_config(
    api_key: String,
    model: String,
    prompt: String,
    stealth_mode: bool,
    whisper_model_size: String,
    whisper_language: String,
    audio_source: String,
    vocab_seed: String,
    response_language: String,
    dodge_on_hover: bool,
    process_disguise_name: String,
    passthrough_mode: bool,
    network_jitter: bool,
    proxy_url: String,
    spoof_user_agent: bool,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) {
    state.set_api_key(api_key);
    state.set_model(model);
    state.set_prompt(prompt);
    state.set_stealth_mode(stealth_mode);
    state.set_whisper_model_size(whisper_model_size);
    state.set_whisper_language(whisper_language);
    state.set_audio_source(audio_source);
    state.set_vocab_seed(vocab_seed);
    state.set_response_language(response_language);
    state.set_dodge_on_hover(dodge_on_hover);
    state.set_process_disguise_name(process_disguise_name);
    state.set_passthrough_mode(passthrough_mode);
    state.set_network_jitter(network_jitter);
    state.set_proxy_url(proxy_url);
    state.set_spoof_user_agent(spoof_user_agent);

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

#[tauri::command]
pub fn get_recording_status(state: tauri::State<'_, AppState>) -> bool {
    state.get_recording()
}

#[tauri::command]
pub fn get_transcription(state: tauri::State<'_, AppState>) -> String {
    state.get_transcription_text()
}

#[tauri::command]
pub async fn start_recording(app: tauri::AppHandle) -> Result<(), String> {
    crate::recording::start(&app)
}

#[tauri::command]
pub fn stop_recording(app: tauri::AppHandle) -> Result<String, String> {
    crate::recording::stop(&app)
}

#[tauri::command]
pub async fn download_whisper_model(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    size: String,
) -> Result<(), String> {
    state.set_whisper_model_size(size.clone());
    crate::whisper::download_model(app, size).await
}

#[tauri::command]
pub fn get_available_models(app: tauri::AppHandle) -> Vec<crate::whisper::ModelInfo> {
    crate::whisper::get_available_models(&app)
}

#[tauri::command]
pub async fn send_transcription_to_gemini(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    text: String,
    prompt: String,
) -> Result<(), String> {
    use tauri::Emitter;

    let api_key = state.get_api_key();
    if api_key.is_empty() {
        return Err("API key not configured".to_string());
    }

    let model = state.get_model();
    let response_language = state.get_response_language();
    let _ = app.emit("processing-start", "transcription");

    let spoof_ua = state.get_spoof_user_agent();
    let jitter = state.get_network_jitter();
    let proxy = state.get_proxy_url();
    let proxy_ref = if proxy.is_empty() { None } else { Some(proxy.as_str()) };

    match crate::gemini::send_to_gemini(&api_key, &model, &text, &prompt, &response_language, spoof_ua, jitter, proxy_ref).await {
        Ok(response) => {
            state.set_last_response(Some(response.clone()));
            let _ = app.emit("capture-response", serde_json::json!({ "text": response, "source": "transcription" }));
            Ok(())
        }
        Err(e) => {
            let _ = app.emit("capture-error", format!("Error: {e}"));
            Err(e)
        }
    }
}

#[tauri::command]
pub fn scan_proctoring(state: tauri::State<'_, AppState>) -> Vec<DetectedProctor> {
    let detected = crate::process_stealth::scan_proctoring_software();
    *state.detected_proctors.lock().unwrap_or_else(|e| e.into_inner()) = detected.clone();
    detected
}

#[tauri::command]
pub fn get_detected_proctors(state: tauri::State<'_, AppState>) -> Vec<DetectedProctor> {
    state.get_detected_proctors()
}

#[tauri::command]
pub fn toggle_passthrough(app: tauri::AppHandle, state: tauri::State<'_, AppState>) {
    let current = state.get_passthrough_mode();
    let new_val = !current;
    state.set_passthrough_mode(new_val);

    if let Some(window) = app.get_webview_window("main") {
        crate::stealth::set_passthrough_mode(&window, new_val);
    }
}

#[tauri::command]
pub fn get_display_info() -> Vec<crate::display_stealth::DisplayInfo> {
    crate::display_stealth::enumerate_displays()
}

#[tauri::command]
pub fn full_proctor_scan() -> crate::proctor_detect::ProctorScanResult {
    crate::proctor_detect::full_scan()
}

#[tauri::command]
pub fn get_env_report() -> crate::env_report::EnvironmentReport {
    crate::env_report::generate_report()
}

#[tauri::command]
pub async fn type_text(text: String, humanized: bool) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        crate::clipboard_stealth::type_text_via_cgevents(&text, humanized);
    })
    .await
    .map_err(|e| format!("Failed to type text: {e}"))
}

#[tauri::command]
pub fn ephemeral_paste(text: String) {
    crate::clipboard_stealth::ephemeral_paste(&text);
}
