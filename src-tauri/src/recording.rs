use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

use crate::audio::{self, Speaker};
use crate::state::AppState;
use crate::vad::{self, Vad};
use crate::whisper;

static STOP_TX: Mutex<Option<mpsc::Sender<()>>> = Mutex::new(None);

pub fn start(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();

    if state.get_recording() {
        return Err("Already recording".to_string());
    }

    let model_size = state.get_whisper_model_size();
    let language = state.get_whisper_language();
    let source_str = state.get_audio_source();
    let vocab_seed = state.get_vocab_seed();
    let source = audio::AudioSource::from_str(&source_str);

    state.set_recording(true);
    state.set_transcription_text(String::new());

    let (audio_tx, audio_rx) = mpsc::channel();
    let stop_flag = Arc::new(AtomicBool::new(false));

    audio::start_capture(source, audio_tx, stop_flag.clone()).map_err(|e| {
        state.set_recording(false);
        e
    })?;

    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    *STOP_TX.lock().unwrap_or_else(|e| e.into_inner()) = Some(stop_tx);

    let flag_clone = stop_flag.clone();
    std::thread::spawn(move || {
        let _ = stop_rx.recv();
        flag_clone.store(true, Ordering::Relaxed);
    });

    run_transcription_loop(app.clone(), audio_rx, stop_flag, model_size, language, vocab_seed);

    let _ = app.emit("recording-started", ());
    eprintln!("[phantom] recording started");

    Ok(())
}

pub fn stop(app: &AppHandle) -> Result<String, String> {
    let state = app.state::<AppState>();

    if !state.get_recording() {
        return Err("Not recording".to_string());
    }

    let tx = STOP_TX.lock().unwrap_or_else(|e| e.into_inner()).take();
    if let Some(tx) = tx {
        let _ = tx.send(());
    }

    state.set_recording(false);
    let _ = app.emit("recording-stopped", ());
    eprintln!("[phantom] recording stopped");

    Ok(state.get_transcription_text())
}

fn run_transcription_loop(
    app: AppHandle,
    audio_rx: mpsc::Receiver<audio::TaggedAudio>,
    stop_flag: Arc<AtomicBool>,
    model_size: String,
    language: String,
    vocab_seed: String,
) {
    std::thread::spawn(move || {
        let ctx = match whisper::load_model(&app, &model_size) {
            Ok(ctx) => ctx,
            Err(e) => {
                let _ = app.emit("transcription-error", e);
                return;
            }
        };

        eprintln!(
            "[phantom] whisper model loaded, starting VAD-based transcription (lang={language})"
        );

        let mut vad = Vad::new();
        let mut full_transcript = String::new();
        let mut locked_language = language.clone();
        let state = app.state::<AppState>();

        // Preview: track last preview size to avoid re-transcribing same audio
        const PREVIEW_INTERVAL_SAMPLES: usize = vad::SAMPLE_RATE * 3; // every ~3s
        let mut last_preview_samples: usize = 0;

        loop {
            match audio_rx.recv_timeout(std::time::Duration::from_millis(50)) {
                Ok(tagged) => {
                    for utt in vad.process(&tagged) {
                        // Clear preview when confirmed utterance arrives
                        let _ = app.emit("transcription-preview", "");
                        last_preview_samples = 0;

                        if locked_language == "auto" && full_transcript.is_empty() {
                            if let Some(detected) = whisper::detect_language(&ctx, &utt.audio) {
                                eprintln!("[phantom] auto-detected language: {detected}");
                                locked_language = detected;
                            }
                        }

                        if let Some(text) = whisper::transcribe_segment(
                            &ctx,
                            &utt.audio,
                            &locked_language,
                            &full_transcript,
                            &vocab_seed,
                        ) {
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

                    // Preview: while speaking, periodically transcribe the partial buffer
                    if vad.is_speaking() {
                        let current_samples = vad.speech_buffer_samples();
                        if current_samples >= last_preview_samples + PREVIEW_INTERVAL_SAMPLES {
                            if let Some((audio, speaker)) = vad.peek_buffer() {
                                last_preview_samples = current_samples;
                                let lang = if locked_language == "auto" { "auto" } else { &locked_language };
                                if let Some(text) = whisper::transcribe_segment(&ctx, &audio, lang, &full_transcript, &vocab_seed) {
                                    if !text.is_empty() {
                                        let label = match speaker {
                                            Speaker::User => "[You]",
                                            Speaker::Other => "[Other]",
                                        };
                                        let _ = app.emit("transcription-preview", format!("{label} {text}"));
                                        eprintln!("[phantom] preview: {label} {text}");
                                    }
                                }
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
                    let rms = Vad::frame_rms(&utt.audio);
                    if utt.audio.len() >= vad::MIN_UTTERANCE_SAMPLES * 2
                        && rms > vad::MIN_SPEECH_THRESHOLD
                    {
                        if let Some(text) = whisper::transcribe_segment(
                            &ctx,
                            &utt.audio,
                            &locked_language,
                            &full_transcript,
                            &vocab_seed,
                        ) {
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
                    } else {
                        eprintln!(
                            "[phantom] skipping flush buffer: too short or low energy (samples={}, rms={:.4})",
                            utt.audio.len(),
                            rms
                        );
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
