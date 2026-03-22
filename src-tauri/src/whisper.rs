use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::hallucination;
use crate::vad;

fn whisper_threads() -> i32 {
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    (cores / 2).max(4) as i32
}

// --- Model Management ---

pub fn models_dir(app: &AppHandle) -> PathBuf {
    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".phantom")
    });
    data_dir.join("models")
}

pub fn model_path(app: &AppHandle, size: &str) -> PathBuf {
    models_dir(app).join(format!("ggml-{size}.bin"))
}

pub fn is_model_downloaded(app: &AppHandle, size: &str) -> bool {
    model_path(app, size).exists()
}

#[derive(serde::Serialize, Clone)]
pub struct ModelInfo {
    pub size: String,
    pub label: String,
    pub downloaded: bool,
    pub file_size_mb: u64,
}

pub fn get_available_models(app: &AppHandle) -> Vec<ModelInfo> {
    let sizes = [
        ("tiny", "Tiny (~75MB)", 75),
        ("base", "Base (~142MB)", 142),
        ("small", "Small (~466MB)", 466),
        ("medium", "Medium (~1.5GB)", 1500),
        ("large-v3-turbo", "Large V3 Turbo (~1.6GB)", 1600),
        ("large-v3", "Large V3 (~3.1GB)", 3100),
    ];

    sizes
        .iter()
        .map(|(size, label, mb)| ModelInfo {
            size: size.to_string(),
            label: label.to_string(),
            downloaded: is_model_downloaded(app, size),
            file_size_mb: *mb,
        })
        .collect()
}

pub async fn download_model(app: AppHandle, size: String) -> Result<(), String> {
    let dir = models_dir(&app);
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create models directory: {e}"))?;

    let path = model_path(&app, &size);
    if path.exists() {
        return Ok(());
    }

    let url = format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{size}.bin"
    );

    eprintln!("[phantom] downloading whisper model: {url}");

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Download request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let part_path = path.with_extension("bin.part");

    let _ = app.emit(
        "model-download-progress",
        serde_json::json!({
            "size": &size,
            "progress": 0.0
        }),
    );

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download: {e}"))?;

    std::fs::write(&part_path, &bytes)
        .map_err(|e| format!("Failed to write model file: {e}"))?;

    std::fs::rename(&part_path, &path)
        .map_err(|e| format!("Failed to rename model file: {e}"))?;

    let _ = app.emit("model-download-complete", &size);
    eprintln!("[phantom] whisper model downloaded: {size}");

    Ok(())
}

// --- Model Loading ---

pub fn load_model(app: &AppHandle, model_size: &str) -> Result<WhisperContext, String> {
    let path = model_path(app, model_size);
    if !path.exists() {
        return Err(format!(
            "Whisper model '{model_size}' not downloaded. Please download it in settings."
        ));
    }

    eprintln!("[phantom] loading whisper model: {}", path.display());

    WhisperContext::new_with_params(
        path.to_str().unwrap_or_default(),
        WhisperContextParameters::default(),
    )
    .map_err(|e| format!("Failed to load whisper model: {e}"))
}

// --- Language Detection ---

pub fn detect_language(ctx: &WhisperContext, audio: &[f32]) -> Option<String> {
    let mut state = ctx.create_state().ok()?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_n_threads(whisper_threads());
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_language(Some("auto"));
    params.set_single_segment(true);
    params.set_suppress_blank(true);
    params.set_temperature(0.0);
    params.set_temperature_inc(0.0);

    state.full(params, audio).ok()?;

    let detected = state.full_lang_id_from_state().ok()?;
    let lang = whisper_rs::get_lang_str(detected)?;

    Some(lang.to_string())
}

// --- Segment Transcription ---

pub fn transcribe_segment(
    ctx: &WhisperContext,
    audio: &[f32],
    language: &str,
    _previous_text: &str,
    vocab_seed: &str,
) -> Option<String> {
    let audio = hallucination::trim_trailing_silence(audio);

    if audio.len() < vad::MIN_UTTERANCE_SAMPLES {
        return None;
    }

    let mut state = ctx.create_state().ok()?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_n_threads(whisper_threads());
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_translate(false);

    let is_short = audio.len() < vad::SAMPLE_RATE * 8;
    params.set_single_segment(is_short);

    if language == "auto" {
        params.set_language(Some("auto"));
    } else {
        params.set_language(Some(language));
    }

    // Anti-hallucination settings
    params.set_suppress_blank(true);
    params.set_no_speech_thold(0.6);
    params.set_entropy_thold(2.4);
    params.set_logprob_thold(-1.0);
    params.set_max_initial_ts(1.0);

    if !vocab_seed.is_empty() {
        params.set_initial_prompt(vocab_seed);
    }

    // Deterministic decoding, no temperature fallback
    params.set_temperature(0.0);
    params.set_temperature_inc(0.0);

    state.full(params, audio).ok()?;

    let num_segments = state.full_n_segments().ok()?;
    let mut segments: Vec<String> = Vec::new();

    for i in 0..num_segments {
        if let Ok(segment_text) = state.full_get_segment_text(i) {
            let trimmed = segment_text.trim();
            if trimmed.is_empty() || hallucination::is_hallucination(trimmed) {
                continue;
            }

            if hallucination::is_duplicate_segment(trimmed, &segments) {
                eprintln!("[phantom] whisper: skipping repeated segment: {trimmed}");
                continue;
            }

            segments.push(trimmed.to_string());
        }
    }

    let result = segments.join(" ");
    let result = result.trim().to_string();

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}
