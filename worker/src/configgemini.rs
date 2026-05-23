use std::env;

#[derive(Clone)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
}

impl GeminiConfig {
    pub fn from_env() -> Self {
        Self {
            api_key: env::var("GEMINI_API_KEY")
                .expect("GEMINI_API_KEY missing"),

            model: env::var("GEMINI_MODEL")
                .unwrap_or("gemini-2.0-flash".to_string()),
        }
    }
}