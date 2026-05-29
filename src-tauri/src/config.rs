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
            Provider::DeepSeek => {
                vec!["deepseek-chat".to_string(), "deepseek-reasoner".to_string()]
            }
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
#[derive(Debug, Clone, Serialize)]
pub struct AppConfig {
    pub api_key: String,
    pub model: String,
    pub source_lang: String,
    pub target_lang: String,
    pub hotkey: String,
    /// Whether the window should stay pinned above other windows and remain visible on blur.
    pub window_pinned: bool,
    /// The active translation provider.
    pub provider: Provider,
    /// The API base URL for the active provider.
    /// Defaults to the provider's canonical URL but can be overridden (e.g. for custom Ollama ports).
    pub api_base_url: String,
    /// The ordered list of model identifiers shown in the Settings dropdown.
    /// Populated from the provider's defaults on first load.
    pub available_models: Vec<String>,
}

// Custom Deserialize so `api_base_url` and `available_models` default to the
// *actual* provider's values, not a static Provider::default().
impl<'de> Deserialize<'de> for AppConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            #[serde(default)]
            api_key: String,
            #[serde(default)]
            model: String,
            #[serde(default = "default_source_lang")]
            source_lang: String,
            #[serde(default = "default_target_lang")]
            target_lang: String,
            #[serde(default = "default_hotkey")]
            hotkey: String,
            #[serde(default)]
            window_pinned: bool,
            #[serde(default)]
            provider: Provider,
            api_base_url: Option<String>,
            available_models: Option<Vec<String>>,
        }

        fn default_source_lang() -> String {
            "auto".to_string()
        }
        fn default_target_lang() -> String {
            "Chinese".to_string()
        }
        fn default_hotkey() -> String {
            "CmdOrCtrl+T".to_string()
        }

        let helper = Helper::deserialize(deserializer)?;
        let provider = helper.provider;
        let api_base_url = helper
            .api_base_url
            .unwrap_or_else(|| provider.default_base_url().to_string());
        let available_models = helper
            .available_models
            .unwrap_or_else(|| provider.default_models());
        let model = if helper.model.is_empty() {
            available_models
                .first()
                .cloned()
                .unwrap_or_else(|| "deepseek-chat".to_string())
        } else {
            helper.model
        };

        Ok(AppConfig {
            api_key: helper.api_key,
            model,
            source_lang: helper.source_lang,
            target_lang: helper.target_lang,
            hotkey: helper.hotkey,
            window_pinned: helper.window_pinned,
            provider,
            api_base_url,
            available_models,
        })
    }
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
            window_pinned: false,
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

    /// Load config from disk, or return defaults if not found.
    /// Logs parse/read errors before falling back to defaults.
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!(
                            "Failed to parse config at {}: {}. Using defaults.",
                            path.display(),
                            e
                        );
                        Self::default()
                    }
                },
                Err(e) => {
                    eprintln!(
                        "Failed to read config at {}: {}. Using defaults.",
                        path.display(),
                        e
                    );
                    Self::default()
                }
            }
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
        assert!(!c.window_pinned);
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
        assert!(!config.window_pinned);
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
            window_pinned: true,
            provider: Provider::Ollama,
            api_base_url: "http://localhost:11434".to_string(),
            available_models: vec!["mistral".to_string(), "llama3".to_string()],
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.provider, Provider::Ollama);
        assert!(restored.window_pinned);
        assert_eq!(restored.api_base_url, "http://localhost:11434");
        assert_eq!(restored.available_models, vec!["mistral", "llama3"]);
    }

    #[test]
    fn provider_default_base_urls_are_correct() {
        assert_eq!(
            Provider::DeepSeek.default_base_url(),
            "https://api.deepseek.com"
        );
        assert_eq!(
            Provider::OpenRouter.default_base_url(),
            "https://openrouter.ai/api"
        );
        assert_eq!(
            Provider::Ollama.default_base_url(),
            "http://localhost:11434"
        );
    }
}
