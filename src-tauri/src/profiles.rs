use crate::config::{ApiKeyStorage, AppConfig, Provider};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DEFAULT_PROFILE_ID: &str = "default";
const DEFAULT_PROFILE_NAME: &str = "Default";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranslationProfile {
    pub id: String,
    pub name: String,
    pub api_key: String,
    pub api_key_storage: ApiKeyStorage,
    pub model: String,
    pub source_lang: String,
    pub target_lang: String,
    pub provider: Provider,
    pub api_base_url: String,
    pub available_models: Vec<String>,
}

impl TranslationProfile {
    pub fn from_config(id: impl Into<String>, name: impl Into<String>, config: &AppConfig) -> Self {
        let api_key = if matches!(
            config.api_key_storage,
            ApiKeyStorage::PlaintextFallback | ApiKeyStorage::LegacyPlaintext
        ) {
            config.api_key.clone()
        } else {
            String::new()
        };

        Self {
            id: id.into(),
            name: name.into(),
            api_key,
            api_key_storage: config.api_key_storage.clone(),
            model: config.model.clone(),
            source_lang: config.source_lang.clone(),
            target_lang: config.target_lang.clone(),
            provider: config.provider.clone(),
            api_base_url: config.api_base_url.clone(),
            available_models: config.available_models.clone(),
        }
    }

    pub fn apply_to_config(&self, config: &mut AppConfig) {
        config.api_key = self.api_key.clone();
        config.api_key_storage = self.api_key_storage.clone();
        config.model = self.model.clone();
        config.source_lang = self.source_lang.clone();
        config.target_lang = self.target_lang.clone();
        config.provider = self.provider.clone();
        config.api_base_url = self.api_base_url.clone();
        config.available_models = self.available_models.clone();
        config.active_profile_id = self.id.clone();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranslationProfilesStore {
    pub active_profile_id: String,
    pub profiles: Vec<TranslationProfile>,
}

impl Default for TranslationProfilesStore {
    fn default() -> Self {
        Self {
            active_profile_id: DEFAULT_PROFILE_ID.to_string(),
            profiles: Vec::new(),
        }
    }
}

impl TranslationProfilesStore {
    pub fn load_with_issues() -> (Self, Vec<String>) {
        let path = profiles_path();
        let mut issues = Vec::new();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<Self>(&content) {
                    Ok(store) => (store, issues),
                    Err(err) => {
                        issues.push(format!(
                            "翻译配置方案文件 {} 解析失败：{}。Aura 已以空方案状态启动。",
                            path.display(),
                            err
                        ));
                        (Self::default(), issues)
                    }
                },
                Err(err) => {
                    issues.push(format!(
                        "翻译配置方案文件 {} 读取失败：{}。Aura 已以空方案状态启动。",
                        path.display(),
                        err
                    ));
                    (Self::default(), issues)
                }
            }
        } else {
            let store = Self::default();
            let _ = store.save();
            (store, issues)
        }
    }

    pub fn snapshot(&self) -> Self {
        self.clone()
    }

    pub fn ensure_seeded_from_config(&mut self, config: &mut AppConfig) -> Result<(), String> {
        if self.profiles.is_empty() {
            let profile = TranslationProfile::from_config(
                config.active_profile_id.clone(),
                DEFAULT_PROFILE_NAME,
                config,
            );
            self.active_profile_id = profile.id.clone();
            self.profiles.push(profile);
            self.save()?;
            return Ok(());
        }

        if !self
            .profiles
            .iter()
            .any(|profile| profile.id == self.active_profile_id)
        {
            self.active_profile_id = self
                .profiles
                .first()
                .map(|profile| profile.id.clone())
                .unwrap_or_else(|| DEFAULT_PROFILE_ID.to_string());
        }

        if let Some(profile) = self
            .profiles
            .iter()
            .find(|profile| profile.id == self.active_profile_id)
            .cloned()
        {
            profile.apply_to_config(config);
        }

        Ok(())
    }

    pub fn sync_active_profile_from_config(&mut self, config: &AppConfig) -> Result<(), String> {
        let active_id = config.active_profile_id.clone();
        if let Some(profile) = self
            .profiles
            .iter_mut()
            .find(|profile| profile.id == active_id)
        {
            *profile = TranslationProfile::from_config(&active_id, &profile.name, config);
        } else {
            self.profiles.push(TranslationProfile::from_config(
                &active_id,
                DEFAULT_PROFILE_NAME,
                config,
            ));
        }
        self.active_profile_id = active_id;
        self.save()
    }

    pub fn create_and_activate(
        &mut self,
        name: &str,
        config: &mut AppConfig,
    ) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Profile names cannot be empty.".to_string());
        }
        let profile_id = next_profile_id();
        let profile = TranslationProfile::from_config(&profile_id, name, config);
        self.active_profile_id = profile_id.clone();
        config.active_profile_id = profile_id.clone();
        self.profiles.insert(0, profile);
        self.save()
    }

    pub fn rename(&mut self, profile_id: &str, name: &str) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Profile names cannot be empty.".to_string());
        }
        let Some(profile) = self
            .profiles
            .iter_mut()
            .find(|profile| profile.id == profile_id)
        else {
            return Err(format!("Profile '{}' was not found.", profile_id));
        };
        profile.name = name.to_string();
        self.save()
    }

    pub fn activate(&mut self, profile_id: &str, config: &mut AppConfig) -> Result<(), String> {
        let Some(profile) = self
            .profiles
            .iter()
            .find(|profile| profile.id == profile_id)
            .cloned()
        else {
            return Err(format!("Profile '{}' was not found.", profile_id));
        };

        self.active_profile_id = profile_id.to_string();
        profile.apply_to_config(config);
        self.save()
    }

    pub fn delete(&mut self, profile_id: &str, config: &mut AppConfig) -> Result<(), String> {
        if self.profiles.len() <= 1 {
            return Err("Aura must keep at least one translation profile.".to_string());
        }

        self.profiles.retain(|profile| profile.id != profile_id);
        if self.active_profile_id == profile_id {
            let next_profile =
                self.profiles.first().cloned().ok_or_else(|| {
                    "Aura must keep at least one translation profile.".to_string()
                })?;
            self.active_profile_id = next_profile.id.clone();
            next_profile.apply_to_config(config);
        }

        self.save()
    }
}

fn profiles_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aura-translation");
    fs::create_dir_all(&config_dir).ok();
    config_dir.join("profiles.json")
}

fn next_profile_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("profile-{}-{}", timestamp, sequence)
}

impl TranslationProfilesStore {
    fn save(&self) -> Result<(), String> {
        #[cfg(test)]
        {
            return Ok(());
        }

        #[cfg(not(test))]
        {
            let path = profiles_path();
            let content = serde_json::to_string_pretty(self).map_err(|err| err.to_string())?;
            let tmp_path = path.with_extension("json.tmp");
            fs::write(&tmp_path, content).map_err(|err| err.to_string())?;
            fs::rename(&tmp_path, &path).map_err(|err| err.to_string())?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> AppConfig {
        AppConfig::default()
    }

    #[test]
    fn seeds_default_profile_from_config() {
        let mut store = TranslationProfilesStore::default();
        let mut config = config();

        store
            .ensure_seeded_from_config(&mut config)
            .expect("seed should succeed");

        assert_eq!(store.profiles.len(), 1);
        assert_eq!(store.profiles[0].name, "Default");
        assert_eq!(store.active_profile_id, config.active_profile_id);
    }

    #[test]
    fn creates_and_activates_named_profiles() {
        let mut store = TranslationProfilesStore::default();
        let mut config = config();
        store.ensure_seeded_from_config(&mut config).unwrap();

        config.target_lang = "Japanese".to_string();
        store
            .create_and_activate("Japanese", &mut config)
            .expect("create should succeed");

        assert_eq!(store.profiles.len(), 2);
        assert_eq!(store.active_profile_id, config.active_profile_id);
        assert_eq!(store.profiles[0].name, "Japanese");
    }

    #[test]
    fn deleting_active_profile_falls_back_to_remaining_profile() {
        let mut store = TranslationProfilesStore::default();
        let mut config = config();
        store.ensure_seeded_from_config(&mut config).unwrap();

        config.target_lang = "Japanese".to_string();
        store.create_and_activate("Japanese", &mut config).unwrap();
        let active_profile_id = store.active_profile_id.clone();

        store
            .delete(&active_profile_id, &mut config)
            .expect("delete should succeed");

        assert_eq!(store.profiles.len(), 1);
        assert_eq!(store.active_profile_id, DEFAULT_PROFILE_ID);
        assert_eq!(config.target_lang, "Chinese");
    }
}
