use crate::state::AppState;
use serde::Serialize;
use tauri::Emitter;

#[derive(Serialize)]
pub struct Config {
    pub api_key: String,
    pub model: String,
    pub prompt: String,
    pub opacity: f64,
    pub stealth_mode: bool,
    pub whisper_model_size: String,
    pub whisper_language: String,
    pub audio_source: String,
    pub vocab_seed: String,
}

#[tauri::command]
pub fn get_config(state: tauri::State<'_, AppState>) -> Config {
    Config {
        api_key: state.get_api_key(),
        model: state.get_model(),
        prompt: state.get_prompt(),
        opacity: state.get_opacity(),
        stealth_mode: state.get_stealth_mode(),
        whisper_model_size: state.get_whisper_model_size(),
        whisper_language: state.get_whisper_language(),
        audio_source: state.get_audio_source(),
        vocab_seed: state.get_vocab_seed(),
    }
}

#[tauri::command]
pub fn save_config(
    api_key: String,
    model: String,
    prompt: String,
    opacity: f64,
    stealth_mode: bool,
    whisper_model_size: String,
    whisper_language: String,
    audio_source: String,
    vocab_seed: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) {
    state.set_api_key(api_key);
    state.set_model(model);
    state.set_prompt(prompt);
    state.set_opacity(opacity);
    state.set_stealth_mode(stealth_mode);
    state.set_whisper_model_size(whisper_model_size);
    state.set_whisper_language(whisper_language);
    state.set_audio_source(audio_source);
    state.set_vocab_seed(vocab_seed);

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
pub async fn start_recording(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    if state.get_recording() {
        return Err("Already recording".to_string());
    }

    let model_size = state.get_whisper_model_size();
    let language = state.get_whisper_language();
    let source_str = state.get_audio_source();
    let vocab_seed = state.get_vocab_seed();

    let source = crate::audio::AudioSource::from_str(&source_str);

    state.set_recording(true);
    state.set_transcription_text(String::new());

    let (audio_tx, audio_rx) = std::sync::mpsc::channel();
    let stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    crate::audio::start_capture(source, audio_tx, stop_flag.clone())
        .map_err(|e| {
            state.set_recording(false);
            e
        })?;

    // Store stop signal
    let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
    *state.recording_stop_tx.lock().unwrap_or_else(|e| e.into_inner()) = Some(stop_tx);

    // Spawn a thread that waits for stop signal then sets the stop flag
    let flag_clone = stop_flag.clone();
    std::thread::spawn(move || {
        let _ = stop_rx.recv();
        flag_clone.store(true, std::sync::atomic::Ordering::Relaxed);
    });

    // Start whisper transcription
    crate::whisper::start_transcription(
        app.clone(),
        audio_rx,
        stop_flag,
        model_size,
        language,
        vocab_seed,
    );

    let _ = app.emit("recording-started", ());
    eprintln!("[phantom] recording started");

    Ok(())
}

#[tauri::command]
pub fn stop_recording(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    if !state.get_recording() {
        return Err("Not recording".to_string());
    }

    // Send stop signal
    let tx = state.recording_stop_tx.lock().unwrap_or_else(|e| e.into_inner()).take();
    if let Some(tx) = tx {
        let _ = tx.send(());
    }

    state.set_recording(false);
    let _ = app.emit("recording-stopped", ());
    eprintln!("[phantom] recording stopped");

    Ok(state.get_transcription_text())
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
    let _ = app.emit("processing-start", ());

    match crate::whisper::send_to_gemini(&api_key, &model, &text, &prompt).await {
        Ok(response) => {
            state.set_last_response(Some(response.clone()));
            let _ = app.emit("capture-response", response);
            Ok(())
        }
        Err(e) => {
            let _ = app.emit("capture-error", format!("Error: {e}"));
            Err(e)
        }
    }
}
