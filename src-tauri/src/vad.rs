use crate::audio::{Speaker, TaggedAudio};

pub const SAMPLE_RATE: usize = 16000;
pub const MAX_UTTERANCE_SAMPLES: usize = SAMPLE_RATE * 30;
pub const MIN_UTTERANCE_SAMPLES: usize = SAMPLE_RATE / 2;

const VAD_FRAME_SIZE: usize = SAMPLE_RATE / 50; // 20ms
const SILENCE_DURATION_MS: usize = 500;
const SILENCE_FRAMES: usize = (SILENCE_DURATION_MS * SAMPLE_RATE) / (1000 * VAD_FRAME_SIZE);
const MIN_SPEECH_FRAMES: usize = 3;
const SPEECH_WINDOW_FRAMES: usize = 5;
const SPEECH_THRESHOLD_MULTIPLIER: f32 = 2.0;
pub const MIN_SPEECH_THRESHOLD: f32 = 0.001;
const NOISE_ALPHA: f32 = 0.05;

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

pub struct Vad {
    state: VadState,
    noise_floor: f32,
    noise_initialized: bool,
    speech_window: Vec<bool>,
    silence_frame_count: usize,
    speech_buffer: Vec<f32>,
    pre_speech_buffer: Vec<f32>,
    frame_count: u64,
    user_energy: f32,
    other_energy: f32,
}

impl Vad {
    pub fn new() -> Self {
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

    pub fn frame_rms(frame: &[f32]) -> f32 {
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
            self.noise_floor = self.noise_floor * (1.0 - NOISE_ALPHA) + rms * NOISE_ALPHA;
        }
    }

    pub fn process(&mut self, tagged: &TaggedAudio) -> Vec<Utterance> {
        let mut utterances = Vec::new();

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

            if self.frame_count % 250 == 0 {
                eprintln!(
                    "[phantom] vad: state={:?} rms={:.5} threshold={:.5} noise_floor={:.5}",
                    self.state,
                    rms,
                    self.threshold(),
                    self.noise_floor
                );
            }

            match self.state {
                VadState::Silence => {
                    self.update_noise_floor(rms);

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
                        let speech_count =
                            self.speech_window.iter().filter(|&&v| v).count();

                        if speech_count >= MIN_SPEECH_FRAMES {
                            self.state = VadState::Speech;
                            self.speech_window.clear();
                            eprintln!(
                                "[phantom] vad: speech started (rms={:.5}, thr={:.5})",
                                rms,
                                self.threshold()
                            );
                        } else {
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

                    if self.speech_buffer.len() >= MAX_UTTERANCE_SAMPLES {
                        eprintln!(
                            "[phantom] vad: max utterance length, forcing transcription"
                        );
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
                            let duration =
                                self.speech_buffer.len() as f32 / SAMPLE_RATE as f32;
                            let speaker = self.dominant_speaker();
                            eprintln!(
                                "[phantom] vad: utterance complete ({:.1}s, speaker={:?})",
                                duration, speaker
                            );

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

    pub fn is_speaking(&self) -> bool {
        matches!(self.state, VadState::Speech | VadState::MaybeSilence)
    }

    pub fn speech_buffer_samples(&self) -> usize {
        self.speech_buffer.len()
    }

    pub fn peek_buffer(&self) -> Option<(&[f32], Speaker)> {
        if self.speech_buffer.len() >= MIN_UTTERANCE_SAMPLES {
            Some((&self.speech_buffer, self.dominant_speaker()))
        } else {
            None
        }
    }

    pub fn flush(&mut self) -> Option<Utterance> {
        if self.speech_buffer.len() >= MIN_UTTERANCE_SAMPLES {
            let speaker = self.dominant_speaker();
            eprintln!(
                "[phantom] vad: flushing remaining buffer ({:.1}s, speaker={:?})",
                self.speech_buffer.len() as f32 / SAMPLE_RATE as f32,
                speaker
            );
            let audio = std::mem::take(&mut self.speech_buffer);
            self.reset_speaker_tracking();
            Some(Utterance { audio, speaker })
        } else {
            None
        }
    }
}
