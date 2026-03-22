use std::sync::Mutex;

pub struct AppState {
    pub api_key: Mutex<String>,
    pub model: Mutex<String>,
    pub prompt: Mutex<String>,
    pub last_response: Mutex<Option<String>>,
    pub is_processing: Mutex<bool>,
    pub opacity: Mutex<f64>,
    pub stealth_mode: Mutex<bool>,
    pub is_recording: Mutex<bool>,
    pub transcription_text: Mutex<String>,
    pub whisper_model_size: Mutex<String>,
    pub whisper_language: Mutex<String>,
    pub audio_source: Mutex<String>,
    pub vocab_seed: Mutex<String>,
    pub response_language: Mutex<String>,
    pub has_onboarded: Mutex<bool>,
}

impl AppState {
    pub fn get_api_key(&self) -> String {
        self.api_key.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn get_model(&self) -> String {
        self.model.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn get_prompt(&self) -> String {
        self.prompt.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn get_last_response(&self) -> Option<String> {
        self.last_response.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn get_processing(&self) -> bool {
        *self.is_processing.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn get_opacity(&self) -> f64 {
        *self.opacity.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn get_stealth_mode(&self) -> bool {
        *self.stealth_mode.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn get_recording(&self) -> bool {
        *self.is_recording.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn get_transcription_text(&self) -> String {
        self.transcription_text.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn get_whisper_model_size(&self) -> String {
        self.whisper_model_size.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn get_whisper_language(&self) -> String {
        self.whisper_language.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn get_audio_source(&self) -> String {
        self.audio_source.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn get_vocab_seed(&self) -> String {
        self.vocab_seed.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn set_api_key(&self, val: String) {
        *self.api_key.lock().unwrap_or_else(|e| e.into_inner()) = val;
    }

    pub fn set_model(&self, val: String) {
        *self.model.lock().unwrap_or_else(|e| e.into_inner()) = val;
    }

    pub fn set_prompt(&self, val: String) {
        *self.prompt.lock().unwrap_or_else(|e| e.into_inner()) = val;
    }

    pub fn set_last_response(&self, val: Option<String>) {
        *self.last_response.lock().unwrap_or_else(|e| e.into_inner()) = val;
    }

    pub fn set_processing(&self, val: bool) {
        *self.is_processing.lock().unwrap_or_else(|e| e.into_inner()) = val;
    }

    pub fn set_opacity(&self, val: f64) {
        *self.opacity.lock().unwrap_or_else(|e| e.into_inner()) = val;
    }

    pub fn set_stealth_mode(&self, val: bool) {
        *self.stealth_mode.lock().unwrap_or_else(|e| e.into_inner()) = val;
    }

    pub fn set_recording(&self, val: bool) {
        *self.is_recording.lock().unwrap_or_else(|e| e.into_inner()) = val;
    }

    pub fn set_transcription_text(&self, val: String) {
        *self.transcription_text.lock().unwrap_or_else(|e| e.into_inner()) = val;
    }

    pub fn set_whisper_model_size(&self, val: String) {
        *self.whisper_model_size.lock().unwrap_or_else(|e| e.into_inner()) = val;
    }

    pub fn set_whisper_language(&self, val: String) {
        *self.whisper_language.lock().unwrap_or_else(|e| e.into_inner()) = val;
    }

    pub fn set_audio_source(&self, val: String) {
        *self.audio_source.lock().unwrap_or_else(|e| e.into_inner()) = val;
    }

    pub fn set_vocab_seed(&self, val: String) {
        *self.vocab_seed.lock().unwrap_or_else(|e| e.into_inner()) = val;
    }

    pub fn get_response_language(&self) -> String {
        self.response_language.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn set_response_language(&self, val: String) {
        *self.response_language.lock().unwrap_or_else(|e| e.into_inner()) = val;
    }

    pub fn get_has_onboarded(&self) -> bool {
        *self.has_onboarded.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn set_has_onboarded(&self, val: bool) {
        *self.has_onboarded.lock().unwrap_or_else(|e| e.into_inner()) = val;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            api_key: Mutex::new(String::new()),
            model: Mutex::new("gemini-2.0-flash".to_string()),
            prompt: Mutex::new(
                "Analyze this screenshot and answer any questions visible on screen. Be concise and direct."
                    .to_string(),
            ),
            last_response: Mutex::new(None),
            is_processing: Mutex::new(false),
            opacity: Mutex::new(0.85),
            stealth_mode: Mutex::new(true),
            is_recording: Mutex::new(false),
            transcription_text: Mutex::new(String::new()),
            whisper_model_size: Mutex::new("small".to_string()),
            whisper_language: Mutex::new("auto".to_string()),
            audio_source: Mutex::new("both".to_string()),
            vocab_seed: Mutex::new(String::new()),
            response_language: Mutex::new("auto".to_string()),
            has_onboarded: Mutex::new(false),
        }
    }
}
