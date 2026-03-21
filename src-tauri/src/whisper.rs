use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tauri::{AppHandle, Emitter, Manager};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::state::AppState;

const SAMPLE_RATE: usize = 16000;
// Max utterance length before forced processing (30s — whisper's native window)
const MAX_UTTERANCE_SAMPLES: usize = SAMPLE_RATE * 30;
// Minimum utterance length to bother transcribing (0.5s)
const MIN_UTTERANCE_SAMPLES: usize = SAMPLE_RATE / 2;

// --- VAD Configuration ---
// Frame size for energy analysis (20ms)
const VAD_FRAME_SIZE: usize = SAMPLE_RATE / 50;
// Silence duration to consider end of utterance (800ms)
const SILENCE_DURATION_MS: usize = 800;
const SILENCE_FRAMES: usize = (SILENCE_DURATION_MS * SAMPLE_RATE) / (1000 * VAD_FRAME_SIZE);
// Minimum speech frames (out of a window) to confirm speech — 3 of 5 frames (~60ms of 100ms)
const MIN_SPEECH_FRAMES: usize = 3;
const SPEECH_WINDOW_FRAMES: usize = 5;
// Speech threshold multiplier over noise floor
const SPEECH_THRESHOLD_MULTIPLIER: f32 = 2.0;
// Minimum absolute threshold (for very quiet environments)
const MIN_SPEECH_THRESHOLD: f32 = 0.001;
// Noise floor estimation — exponential moving average factor
const NOISE_ALPHA: f32 = 0.05;

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

    let _ = app.emit("model-download-progress", serde_json::json!({
        "size": &size,
        "progress": 0.0
    }));

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

// ==========================================
// VAD - Voice Activity Detection state machine
// ==========================================

use crate::audio::{TaggedAudio, Speaker};

/// Completed utterance with speaker attribution
pub struct Utterance {
    pub audio: Vec<f32>,
    pub speaker: Speaker,
}

#[derive(Debug, PartialEq)]
enum VadState {
    Silence,
    MaybeSpeech,
    Speech,
    MaybeSilence,
}

struct Vad {
    state: VadState,
    noise_floor: f32,
    noise_initialized: bool,
    speech_window: Vec<bool>,
    silence_frame_count: usize,
    speech_buffer: Vec<f32>,
    pre_speech_buffer: Vec<f32>,
    frame_count: u64,
    // Speaker tracking: accumulate energy per source during an utterance
    user_energy: f32,
    other_energy: f32,
}

impl Vad {
    fn new() -> Self {
        Self {
            state: VadState::Silence,
            noise_floor: 0.0,
            noise_initialized: false,
            speech_window: Vec::with_capacity(SPEECH_WINDOW_FRAMES),
            silence_frame_count: 0,
            speech_buffer: Vec::new(),
            pre_speech_buffer: Vec::new(),
            frame_count: 0,
            user_energy: 0.0,
            other_energy: 0.0,
        }
    }

    fn reset_speaker_tracking(&mut self) {
        self.user_energy = 0.0;
        self.other_energy = 0.0;
    }

    fn dominant_speaker(&self) -> Speaker {
        if self.user_energy >= self.other_energy {
            Speaker::User
        } else {
            Speaker::Other
        }
    }

    fn threshold(&self) -> f32 {
        if !self.noise_initialized {
            return MIN_SPEECH_THRESHOLD;
        }
        (self.noise_floor * SPEECH_THRESHOLD_MULTIPLIER).max(MIN_SPEECH_THRESHOLD)
    }

    fn frame_rms(frame: &[f32]) -> f32 {
        if frame.is_empty() {
            return 0.0;
        }
        (frame.iter().map(|s| s * s).sum::<f32>() / frame.len() as f32).sqrt()
    }

    fn update_noise_floor(&mut self, rms: f32) {
        if !self.noise_initialized {
            self.noise_floor = rms;
            self.noise_initialized = true;
        } else {
            // Exponential moving average — slowly adapts to ambient noise
            self.noise_floor = self.noise_floor * (1.0 - NOISE_ALPHA) + rms * NOISE_ALPHA;
        }
    }

    /// Feed tagged audio and return completed utterances with speaker attribution
    fn process(&mut self, tagged: &TaggedAudio) -> Vec<Utterance> {
        let mut utterances = Vec::new();

        // Track energy for speaker attribution
        let chunk_rms = Self::frame_rms(&tagged.samples);
        match tagged.source {
            Speaker::User => self.user_energy += chunk_rms,
            Speaker::Other => self.other_energy += chunk_rms,
        }

        for frame in tagged.samples.chunks(VAD_FRAME_SIZE) {
            if frame.len() < VAD_FRAME_SIZE / 2 {
                continue;
            }

            let rms = Self::frame_rms(frame);
            let is_speech = rms > self.threshold();
            self.frame_count += 1;

            // Log periodically for debugging
            if self.frame_count % 250 == 0 {
                eprintln!(
                    "[phantom] vad: state={:?} rms={:.5} threshold={:.5} noise_floor={:.5}",
                    self.state, rms, self.threshold(), self.noise_floor
                );
            }

            match self.state {
                VadState::Silence => {
                    // Update noise floor during silence
                    self.update_noise_floor(rms);

                    // Rolling pre-speech buffer (~300ms)
                    self.pre_speech_buffer.extend_from_slice(frame);
                    let max_pre = SAMPLE_RATE * 3 / 10; // 300ms
                    if self.pre_speech_buffer.len() > max_pre {
                        let drain = self.pre_speech_buffer.len() - max_pre;
                        self.pre_speech_buffer.drain(..drain);
                    }

                    if is_speech {
                        self.state = VadState::MaybeSpeech;
                        self.speech_window.clear();
                        self.speech_window.push(true);

                        self.speech_buffer.clear();
                        self.speech_buffer.extend_from_slice(&self.pre_speech_buffer);
                        self.speech_buffer.extend_from_slice(frame);
                    }
                }
                VadState::MaybeSpeech => {
                    self.speech_buffer.extend_from_slice(frame);
                    self.speech_window.push(is_speech);

                    if self.speech_window.len() >= SPEECH_WINDOW_FRAMES {
                        let speech_count = self.speech_window.iter().filter(|&&v| v).count();

                        if speech_count >= MIN_SPEECH_FRAMES {
                            // Enough speech frames in window — confirmed speech
                            self.state = VadState::Speech;
                            self.speech_window.clear();
                            eprintln!("[phantom] vad: speech started (rms={:.5}, thr={:.5})", rms, self.threshold());
                        } else {
                            // Not enough speech — false alarm
                            self.state = VadState::Silence;
                            self.speech_buffer.clear();
                            self.speech_window.clear();
                        }
                    }
                }
                VadState::Speech => {
                    self.speech_buffer.extend_from_slice(frame);

                    if !is_speech {
                        self.state = VadState::MaybeSilence;
                        self.silence_frame_count = 1;
                    }

                    // Force processing if utterance gets too long
                    if self.speech_buffer.len() >= MAX_UTTERANCE_SAMPLES {
                        eprintln!("[phantom] vad: max utterance length, forcing transcription");
                        let speaker = self.dominant_speaker();
                        let audio = std::mem::take(&mut self.speech_buffer);
                        utterances.push(Utterance { audio, speaker });
                        self.reset_speaker_tracking();
                    }
                }
                VadState::MaybeSilence => {
                    self.speech_buffer.extend_from_slice(frame);

                    if is_speech {
                        self.state = VadState::Speech;
                        self.silence_frame_count = 0;
                    } else {
                        self.silence_frame_count += 1;
                        if self.silence_frame_count >= SILENCE_FRAMES {
                            let duration = self.speech_buffer.len() as f32 / SAMPLE_RATE as f32;
                            let speaker = self.dominant_speaker();
                            eprintln!("[phantom] vad: utterance complete ({:.1}s, speaker={:?})", duration, speaker);

                            let audio = std::mem::take(&mut self.speech_buffer);
                            if audio.len() >= MIN_UTTERANCE_SAMPLES {
                                utterances.push(Utterance { audio, speaker });
                            }
                            self.state = VadState::Silence;
                            self.silence_frame_count = 0;
                            self.pre_speech_buffer.clear();
                            self.reset_speaker_tracking();
                        }
                    }
                }
            }
        }

        utterances
    }

    /// Flush any remaining speech when recording stops
    fn flush(&mut self) -> Option<Utterance> {
        if self.speech_buffer.len() >= MIN_UTTERANCE_SAMPLES {
            let speaker = self.dominant_speaker();
            eprintln!(
                "[phantom] vad: flushing remaining buffer ({:.1}s, speaker={:?})",
                self.speech_buffer.len() as f32 / SAMPLE_RATE as f32, speaker
            );
            let audio = std::mem::take(&mut self.speech_buffer);
            self.reset_speaker_tracking();
            Some(Utterance { audio, speaker })
        } else {
            None
        }
    }
}

// ==========================================
// Transcription pipeline
// ==========================================

pub fn start_transcription(
    app: AppHandle,
    audio_rx: mpsc::Receiver<TaggedAudio>,
    stop_flag: Arc<AtomicBool>,
    model_size: String,
    language: String,
    vocab_seed: String,
) {
    std::thread::spawn(move || {
        let path = model_path(&app, &model_size);
        if !path.exists() {
            let _ = app.emit("transcription-error",
                format!("Whisper model '{model_size}' not downloaded. Please download it in settings."));
            return;
        }

        eprintln!("[phantom] loading whisper model: {}", path.display());

        let ctx = match WhisperContext::new_with_params(
            path.to_str().unwrap_or_default(),
            WhisperContextParameters::default(),
        ) {
            Ok(ctx) => ctx,
            Err(e) => {
                let _ = app.emit("transcription-error", format!("Failed to load whisper model: {e}"));
                return;
            }
        };

        eprintln!("[phantom] whisper model loaded, starting VAD-based transcription (lang={language})");

        let mut vad = Vad::new();
        let mut full_transcript = String::new();
        let mut locked_language = language.clone();
        let state = app.state::<AppState>();

        loop {
            match audio_rx.recv_timeout(std::time::Duration::from_millis(50)) {
                Ok(tagged) => {
                    let utterances = vad.process(&tagged);

                    for utt in utterances {
                        if locked_language == "auto" && full_transcript.is_empty() {
                            if let Some(detected) = detect_language(&ctx, &utt.audio) {
                                eprintln!("[phantom] auto-detected language: {detected}");
                                locked_language = detected;
                            }
                        }

                        if let Some(text) = transcribe_segment(&ctx, &utt.audio, &locked_language, &full_transcript, &vocab_seed) {
                            if !text.is_empty() {
                                let label = match utt.speaker {
                                    Speaker::User => "[You]",
                                    Speaker::Other => "[Other]",
                                };

                                if !full_transcript.is_empty() {
                                    full_transcript.push('\n');
                                }
                                full_transcript.push_str(&format!("{label} {text}"));

                                state.set_transcription_text(full_transcript.clone());
                                let _ = app.emit("transcription-partial", full_transcript.clone());
                                eprintln!("[phantom] transcript: {label} {text}");
                            }
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    eprintln!("[phantom] audio channel disconnected");
                    break;
                }
            }

            if stop_flag.load(Ordering::Relaxed) {
                if let Some(utt) = vad.flush() {
                    if let Some(text) = transcribe_segment(&ctx, &utt.audio, &locked_language, &full_transcript, &vocab_seed) {
                        if !text.is_empty() {
                            let label = match utt.speaker {
                                Speaker::User => "[You]",
                                Speaker::Other => "[Other]",
                            };
                            if !full_transcript.is_empty() {
                                full_transcript.push('\n');
                            }
                            full_transcript.push_str(&format!("{label} {text}"));
                        }
                    }
                }
                break;
            }
        }

        let final_text = full_transcript.trim().to_string();
        state.set_transcription_text(final_text.clone());
        let _ = app.emit("transcription-complete", final_text);
        eprintln!("[phantom] transcription complete");
    });
}

/// Detect the language of an audio segment using whisper's built-in detection
fn detect_language(ctx: &WhisperContext, audio: &[f32]) -> Option<String> {
    let mut state = ctx.create_state().ok()?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_n_threads(4);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_language(Some("auto"));
    // Only process a small segment for detection
    params.set_single_segment(true);
    params.set_suppress_blank(true);

    params.set_temperature(0.0);
    params.set_temperature_inc(0.0);

    state.full(params, audio).ok()?;

    let detected = state.full_lang_id_from_state().ok()?;
    let lang = whisper_rs::get_lang_str(detected)?;

    Some(lang.to_string())
}

/// Trim trailing silence from an utterance to prevent whisper from hallucinating at the end
fn trim_trailing_silence(audio: &[f32]) -> &[f32] {
    let frame = SAMPLE_RATE / 50; // 20ms frames
    let mut last_speech_end = audio.len();

    // Walk backwards finding last frame with speech energy
    for start in (0..audio.len()).rev().step_by(frame) {
        let end = (start + frame).min(audio.len());
        let chunk = &audio[start..end];
        let rms = Vad::frame_rms(chunk);
        if rms > MIN_SPEECH_THRESHOLD * 2.0 {
            // Add a small tail (100ms) after last speech for natural endings
            last_speech_end = (end + SAMPLE_RATE / 10).min(audio.len());
            break;
        }
    }

    &audio[..last_speech_end]
}

fn transcribe_segment(
    ctx: &WhisperContext,
    audio: &[f32],
    language: &str,
    _previous_text: &str,
    vocab_seed: &str,
) -> Option<String> {
    // 1. Trim trailing silence — prevents hallucinations from silence at end of utterance
    let audio = trim_trailing_silence(audio);

    if audio.len() < MIN_UTTERANCE_SAMPLES {
        return None;
    }

    let mut state = ctx.create_state().ok()?;

    // beam_size=1 has LOWEST hallucination rate (research confirmed)
    // higher beam sizes improve accuracy but increase hallucination on non-speech
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_n_threads(4);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_translate(false);

    // For shorter utterances (< 8s), force single segment to prevent repetition tail
    let is_short = audio.len() < SAMPLE_RATE * 8;
    params.set_single_segment(is_short);

    if language == "auto" {
        params.set_language(Some("auto"));
    } else {
        params.set_language(Some(language));
    }

    // --- Anti-hallucination settings (values from whisper.cpp docs + research) ---
    params.set_suppress_blank(true);
    // DO NOT enable suppress_non_speech_tokens — it CAUSES hallucination
    // (disabled by default in whisper.cpp commit a94897bc)
    params.set_no_speech_thold(0.6);
    params.set_entropy_thold(2.4);
    params.set_logprob_thold(-1.0);
    params.set_max_initial_ts(1.0);

    // Initial prompt: ONLY vocab seed for term biasing
    // DO NOT condition on previous transcript — causes hallucination feedback loops
    // (whisper.cpp issue #1511, discussion #1490)
    if !vocab_seed.is_empty() {
        params.set_initial_prompt(vocab_seed);
    }

    // Deterministic decoding, NO temperature fallback (prevents repetitive loops)
    params.set_temperature(0.0);
    params.set_temperature_inc(0.0);

    state.full(params, audio).ok()?;

    let num_segments = state.full_n_segments().ok()?;
    let mut segments: Vec<String> = Vec::new();

    for i in 0..num_segments {
        if let Ok(segment_text) = state.full_get_segment_text(i) {
            let trimmed = segment_text.trim();
            if trimmed.is_empty() || is_hallucination(trimmed) {
                continue;
            }

            // 3. Dedup — skip segments that repeat content already collected
            let lower = trimmed.to_lowercase();
            let is_repeat = segments.iter().any(|prev| {
                let prev_lower = prev.to_lowercase();
                // Exact duplicate
                if prev_lower == lower {
                    return true;
                }
                // One contains the other (partial repeat)
                if prev_lower.len() > 10 && lower.contains(&prev_lower[..prev_lower.len() * 2 / 3]) {
                    return true;
                }
                if lower.len() > 10 && prev_lower.contains(&lower[..lower.len() * 2 / 3]) {
                    return true;
                }
                false
            });

            if is_repeat {
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

/// Filter known whisper hallucination patterns
fn is_hallucination(text: &str) -> bool {
    let lower = text.to_lowercase();
    let trimmed = lower.trim();

    if trimmed.len() < 2 {
        return true;
    }

    // All-punctuation, whitespace, or musical symbols
    if trimmed.chars().all(|c| !c.is_alphanumeric()) {
        return true;
    }

    // --- Exact match hallucinations ---
    let exact = [
        "thank you", "thank you for watching", "thanks for watching",
        "thank you so much", "thanks for listening",
        "subscribe", "like and subscribe", "please subscribe",
        "see you next time", "see you in the next video", "see you later",
        "bye bye", "bye", "goodbye", "good night",
        "the end", "you", "so", "oh", "hmm", "um", "uh", "ah", "huh",
        "what", "what?", "why", "how", "yes", "no", "ok", "okay",
        "music", "applause", "silence", "laughter",
        "obrigado", "obrigada", "tchau", "valeu",
        "legendas pela comunidade",
        "sim", "não", "tá", "né", "bom", "bem", "então",
    ];

    if exact.contains(&trimmed) {
        return true;
    }

    // --- Prefix/contains match for common hallucination patterns ---
    let patterns = [
        "subtitles by", "amara.org", "www.", "http",
        "translated by", "captions by", "copyright",
        "all rights reserved", "please like",
        "don't forget to", "hit the bell",
        "follow me on", "check out",
    ];

    for pattern in &patterns {
        if trimmed.starts_with(pattern) || trimmed.contains(pattern) {
            return true;
        }
    }

    // --- Question-form hallucinations (whisper generates random questions) ---
    let question_hallucinations = [
        "what's the reason", "what is the reason",
        "what do you think", "what do you mean",
        "what is this", "what is that",
        "what are you doing", "what happened",
        "how are you", "how do you do",
        "who are you", "who is this",
        "where are you", "where is this",
        "why is that", "why not",
        "is that so", "is it", "is that right",
        "do you know", "did you know",
        "can you", "could you",
        "really", "right",
    ];

    if question_hallucinations.contains(&trimmed) {
        return true;
    }

    let words: Vec<&str> = trimmed.split_whitespace().collect();

    // --- Very short segments are suspicious ---
    if words.len() <= 2 && trimmed.len() < 10 {
        return true;
    }

    // --- Single word repeated: "you you you you" ---
    if words.len() >= 3 {
        let first = words[0];
        if words.iter().filter(|&&w| w == first).count() >= words.len() * 2 / 3 {
            return true;
        }
    }

    // --- Bigram repeated: "Thank you. Thank you. Thank you." ---
    if words.len() >= 4 {
        let bigram = format!("{} {}", words[0], words.get(1).unwrap_or(&""));
        if !bigram.trim().is_empty() && trimmed.matches(&bigram).count() >= 2 {
            return true;
        }
    }

    // --- Detect language mismatch: segment entirely in a foreign script ---
    // If most chars are non-Latin (e.g., Arabic, Cyrillic, CJK), likely hallucination
    // Detect foreign scripts (Arabic, Cyrillic, CJK, etc.) — likely hallucination
    let alpha_chars: Vec<char> = trimmed.chars().filter(|c| c.is_alphabetic()).collect();
    if alpha_chars.len() > 5 {
        let non_latin = alpha_chars.iter().filter(|c| {
            let cp = **c as u32;
            // Latin Basic (0000–007F) + Latin-1 Supplement (0080–00FF) + Latin Extended (0100–024F)
            cp > 0x024F
        }).count();
        if non_latin > alpha_chars.len() / 2 {
            return true;
        }
    }

    false
}

pub async fn send_to_gemini(
    api_key: &str,
    model: &str,
    text: &str,
    prompt: &str,
) -> Result<String, String> {
    let full_prompt = format!("{prompt}\n\nTranscription:\n{text}");

    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    #[derive(serde::Serialize)]
    struct Request {
        contents: Vec<Content>,
    }
    #[derive(serde::Serialize)]
    struct Content {
        parts: Vec<Part>,
    }
    #[derive(serde::Serialize)]
    struct Part {
        text: String,
    }

    let request = Request {
        contents: vec![Content {
            parts: vec![Part { text: full_prompt }],
        }],
    };

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    let status = response.status();
    let raw = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    if !status.is_success() {
        return Err(format!("Gemini API error ({}): {}", status, &raw[..raw.len().min(300)]));
    }

    #[derive(serde::Deserialize)]
    struct GeminiResponse {
        candidates: Option<Vec<Candidate>>,
    }
    #[derive(serde::Deserialize)]
    struct Candidate {
        content: CandidateContent,
    }
    #[derive(serde::Deserialize)]
    struct CandidateContent {
        parts: Vec<ResponsePart>,
    }
    #[derive(serde::Deserialize)]
    struct ResponsePart {
        text: String,
    }

    let body: GeminiResponse = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    body.candidates
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.content.parts.into_iter().next())
        .map(|p| p.text)
        .ok_or_else(|| "Empty response from Gemini".to_string())
}
