use std::sync::Mutex;

pub struct AppState {
    pub api_key: Mutex<String>,
    pub model: Mutex<String>,
    pub prompt: Mutex<String>,
    pub last_response: Mutex<Option<String>>,
    pub is_processing: Mutex<bool>,
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
        }
    }
}
