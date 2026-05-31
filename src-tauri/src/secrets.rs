use crate::config::{ApiKeyStorage, AppConfig, Provider};

#[cfg_attr(test, allow(dead_code))]
const SERVICE_NAME: &str = "Aura Translation";

#[cfg(all(test, any(target_os = "windows", target_os = "macos")))]
fn test_secret_store() -> &'static std::sync::Mutex<std::collections::HashMap<&'static str, String>>
{
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    static STORE: OnceLock<Mutex<HashMap<&'static str, String>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn system_storage_supported() -> bool {
    cfg!(any(target_os = "windows", target_os = "macos"))
}

pub fn hydrate_api_key(config: &mut AppConfig) -> Result<(), String> {
    if !config.provider.requires_api_key() {
        config.api_key.clear();
        return Ok(());
    }

    match config.api_key_storage {
        ApiKeyStorage::System => {
            config.api_key = load_system_api_key(&config.provider)?.unwrap_or_default();
        }
        ApiKeyStorage::PlaintextFallback | ApiKeyStorage::LegacyPlaintext => {}
    }

    Ok(())
}

pub fn migrate_legacy_plaintext_key(config: &mut AppConfig) -> Result<bool, String> {
    if config.api_key_storage != ApiKeyStorage::LegacyPlaintext {
        return Ok(false);
    }

    if !config.provider.requires_api_key() || config.api_key.trim().is_empty() {
        config.api_key_storage = ApiKeyStorage::System;
        config.api_key.clear();
        return Ok(true);
    }

    if !system_storage_supported() {
        return Err(
            "System credential storage is not available in this build. Switch API key storage to plaintext fallback if you need to keep using this provider.".to_string(),
        );
    }

    save_system_api_key(&config.provider, config.api_key.trim())?;
    config.api_key_storage = ApiKeyStorage::System;
    Ok(true)
}

pub fn persist_api_key(config: &mut AppConfig, old_config: &AppConfig) -> Result<(), String> {
    if !config.provider.requires_api_key() {
        config.api_key.clear();
        return Ok(());
    }

    config.api_key = config.api_key.trim().to_string();

    match config.api_key_storage {
        ApiKeyStorage::System => {
            if !system_storage_supported() {
                return Err(
                    "System credential storage is not available in this build. Switch API key storage to plaintext fallback to keep using an API key on this platform.".to_string(),
                );
            }

            if config.api_key.is_empty() {
                delete_system_api_key(&config.provider)?;
            } else {
                save_system_api_key(&config.provider, &config.api_key)?;
            }
        }
        ApiKeyStorage::PlaintextFallback | ApiKeyStorage::LegacyPlaintext => {
            config.api_key_storage = ApiKeyStorage::PlaintextFallback;
            if matches!(old_config.api_key_storage, ApiKeyStorage::System)
                && old_config.provider == config.provider
            {
                let _ = delete_system_api_key(&config.provider);
            }
        }
    }

    Ok(())
}

pub fn load_provider_api_key(provider: &Provider) -> Result<String, String> {
    if !provider.requires_api_key() {
        return Ok(String::new());
    }

    Ok(load_system_api_key(provider)?.unwrap_or_default())
}

#[cfg(all(not(test), any(target_os = "windows", target_os = "macos")))]
fn load_system_api_key(provider: &Provider) -> Result<Option<String>, String> {
    use keyring::{Entry, Error};

    let entry = Entry::new(SERVICE_NAME, provider.secret_account_name())
        .map_err(|err| format!("Failed to open the system credential entry: {err}"))?;

    match entry.get_password() {
        Ok(api_key) => Ok(Some(api_key)),
        Err(Error::NoEntry) => Ok(None),
        Err(err) => Err(format!(
            "Failed to read the API key from the system credential store: {err}"
        )),
    }
}

#[cfg(all(test, any(target_os = "windows", target_os = "macos")))]
fn load_system_api_key(provider: &Provider) -> Result<Option<String>, String> {
    Ok(test_secret_store()
        .lock()
        .unwrap()
        .get(provider.secret_account_name())
        .cloned())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn load_system_api_key(_provider: &Provider) -> Result<Option<String>, String> {
    Ok(None)
}

#[cfg(all(not(test), any(target_os = "windows", target_os = "macos")))]
fn save_system_api_key(provider: &Provider, api_key: &str) -> Result<(), String> {
    use keyring::Entry;

    let entry = Entry::new(SERVICE_NAME, provider.secret_account_name())
        .map_err(|err| format!("Failed to open the system credential entry: {err}"))?;
    entry
        .set_password(api_key)
        .map_err(|err| format!("Failed to store the API key in the system credential store: {err}"))
}

#[cfg(all(test, any(target_os = "windows", target_os = "macos")))]
fn save_system_api_key(provider: &Provider, api_key: &str) -> Result<(), String> {
    test_secret_store()
        .lock()
        .unwrap()
        .insert(provider.secret_account_name(), api_key.to_string());
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn save_system_api_key(_provider: &Provider, _api_key: &str) -> Result<(), String> {
    Err("System credential storage is not available in this build.".to_string())
}

#[cfg(all(not(test), any(target_os = "windows", target_os = "macos")))]
fn delete_system_api_key(provider: &Provider) -> Result<(), String> {
    use keyring::{Entry, Error};

    let entry = Entry::new(SERVICE_NAME, provider.secret_account_name())
        .map_err(|err| format!("Failed to open the system credential entry: {err}"))?;

    match entry.delete_credential() {
        Ok(()) | Err(Error::NoEntry) => Ok(()),
        Err(err) => Err(format!(
            "Failed to remove the API key from the system credential store: {err}"
        )),
    }
}

#[cfg(all(test, any(target_os = "windows", target_os = "macos")))]
fn delete_system_api_key(provider: &Provider) -> Result<(), String> {
    test_secret_store()
        .lock()
        .unwrap()
        .remove(provider.secret_account_name());
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn delete_system_api_key(_provider: &Provider) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Provider;

    fn secure_config() -> AppConfig {
        let mut config = AppConfig::default();
        config.provider = Provider::DeepSeek;
        config.api_key = "sk-test".to_string();
        config.api_key_storage = ApiKeyStorage::System;
        config
    }

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    #[test]
    fn persists_system_api_keys_outside_config() {
        let old_config = AppConfig::default();
        let mut config = secure_config();
        persist_api_key(&mut config, &old_config).expect("system store should accept the password");

        let loaded = load_provider_api_key(&Provider::DeepSeek).expect("secret should load");
        assert_eq!(loaded, "sk-test");
        assert_eq!(config.api_key_storage, ApiKeyStorage::System);
    }

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    #[test]
    fn migrates_legacy_plaintext_key_once() {
        let mut config = secure_config();
        config.api_key_storage = ApiKeyStorage::LegacyPlaintext;

        assert!(migrate_legacy_plaintext_key(&mut config).expect("migration should succeed"));
        assert_eq!(config.api_key_storage, ApiKeyStorage::System);
        assert_eq!(
            load_provider_api_key(&Provider::DeepSeek).expect("secret should load"),
            "sk-test"
        );
    }

    #[test]
    fn clears_auth_free_provider_keys() {
        let old_config = AppConfig::default();
        let mut config = AppConfig::default();
        config.provider = Provider::Ollama;
        config.api_key = "sk-should-be-cleared".to_string();
        config.api_key_storage = ApiKeyStorage::System;

        persist_api_key(&mut config, &old_config).expect("ollama should not need a secret");
        assert!(config.api_key.is_empty());
    }
}
