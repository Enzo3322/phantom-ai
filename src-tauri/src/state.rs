use std::sync::Mutex;

pub struct AppState {
    pub api_key: Mutex<String>,
    pub model: Mutex<String>,
    pub prompt: Mutex<String>,
    pub last_response: Mutex<Option<String>>,
    pub is_processing: Mutex<bool>,
    pub glass_effect: Mutex<bool>,
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

    pub fn get_glass_effect(&self) -> bool {
        *self.glass_effect.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn set_glass_effect(&self, val: bool) {
        *self.glass_effect.lock().unwrap_or_else(|e| e.into_inner()) = val;
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
            glass_effect: Mutex::new(true),
        }
    }
}
