use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Supported translation provider identifiers.
/// Each variant maps to a predefined API base URL and auth scheme.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    DeepSeek,
    OpenRouter,
    Ollama,
}

impl Provider {
    /// Returns the default API base URL for this provider.
    pub fn default_base_url(&self) -> &'static str {
        match self {
            Provider::DeepSeek => "https://api.deepseek.com",
            Provider::OpenRouter => "https://openrouter.ai/api",
            Provider::Ollama => "http://localhost:11434",
        }
    }

    /// Returns the default model list for this provider.
    pub fn default_models(&self) -> Vec<String> {
        match self {
            Provider::DeepSeek => vec![
                "deepseek-chat".to_string(),
                "deepseek-reasoner".to_string(),
            ],
            Provider::OpenRouter => vec![
                "mistralai/mistral-7b-instruct".to_string(),
                "google/gemma-3-27b-it".to_string(),
                "meta-llama/llama-3.3-70b-instruct".to_string(),
            ],
            Provider::Ollama => vec![
                "mistral".to_string(),
                "llama3".to_string(),
                "qwen2.5".to_string(),
            ],
        }
    }
}

impl Default for Provider {
    fn default() -> Self {
        Provider::DeepSeek
    }
}

/// Application configuration stored as plaintext JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub api_key: String,
    pub model: String,
    pub source_lang: String,
    pub target_lang: String,
    pub hotkey: String,
    /// The active translation provider.
    #[serde(default)]
    pub provider: Provider,
    /// The API base URL for the active provider.
    /// Defaults to the provider's canonical URL but can be overridden (e.g. for custom Ollama ports).
    #[serde(default = "default_api_base_url")]
    pub api_base_url: String,
    /// The ordered list of model identifiers shown in the Settings dropdown.
    /// Populated from the provider's defaults on first load.
    #[serde(default = "default_available_models")]
    pub available_models: Vec<String>,
}

fn default_api_base_url() -> String {
    Provider::default().default_base_url().to_string()
}

fn default_available_models() -> Vec<String> {
    Provider::default().default_models()
}

impl Default for AppConfig {
    fn default() -> Self {
        let provider = Provider::default();
        let api_base_url = provider.default_base_url().to_string();
        let available_models = provider.default_models();
        Self {
            api_key: String::new(),
            model: "deepseek-chat".to_string(),
            source_lang: "auto".to_string(),
            target_lang: "Chinese".to_string(),
            hotkey: "CmdOrCtrl+T".to_string(),
            provider,
            api_base_url,
            available_models,
        }
    }
}

impl AppConfig {
    /// Get the path to the config file in the user's config directory.
    fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("aura-translation");
        fs::create_dir_all(&config_dir).ok();
        config_dir.join("config.json")
    }

    /// Load config from disk, or return defaults if not found or malformed.
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            let config = Self::default();
            config.save().ok();
            config
        }
    }

    /// Save config to disk atomically via write-then-rename.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        let tmp_path = path.with_extension("json.tmp");
        fs::write(&tmp_path, content).map_err(|e| e.to_string())?;
        fs::rename(&tmp_path, &path).map_err(|e| e.to_string())?;
        Ok(())
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let c = AppConfig::default();
        assert_eq!(c.model, "deepseek-chat");
        assert_eq!(c.hotkey, "CmdOrCtrl+T");
        assert_eq!(c.api_base_url, "https://api.deepseek.com");
        assert!(!c.available_models.is_empty());
    }

    #[test]
    fn missing_new_fields_fall_back_to_defaults() {
        // Simulate a pre-Phase-4 config.json that lacks the new fields
        let legacy_json = r#"{
            "api_key": "sk-test",
            "model": "deepseek-v4-flash",
            "source_lang": "auto",
            "target_lang": "Chinese",
            "hotkey": "CmdOrCtrl+T"
        }"#;
        let config: AppConfig = serde_json::from_str(legacy_json).expect("should parse");
        assert_eq!(config.provider, Provider::DeepSeek);
        assert_eq!(config.api_base_url, "https://api.deepseek.com");
        assert!(!config.available_models.is_empty());
    }

    #[test]
    fn full_config_round_trips() {
        let original = AppConfig {
            api_key: "sk-abc".to_string(),
            model: "mistral".to_string(),
            source_lang: "English".to_string(),
            target_lang: "Japanese".to_string(),
            hotkey: "Alt+Shift+T".to_string(),
            provider: Provider::Ollama,
            api_base_url: "http://localhost:11434".to_string(),
            available_models: vec!["mistral".to_string(), "llama3".to_string()],
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.provider, Provider::Ollama);
        assert_eq!(restored.api_base_url, "http://localhost:11434");
        assert_eq!(restored.available_models, vec!["mistral", "llama3"]);
    }

    #[test]
    fn provider_default_base_urls_are_correct() {
        assert_eq!(Provider::DeepSeek.default_base_url(), "https://api.deepseek.com");
        assert_eq!(Provider::OpenRouter.default_base_url(), "https://openrouter.ai/api");
        assert_eq!(Provider::Ollama.default_base_url(), "http://localhost:11434");
    }
}
