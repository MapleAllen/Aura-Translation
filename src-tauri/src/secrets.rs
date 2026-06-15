use crate::config::{ApiKeyStorage, AppConfig, Provider};

#[cfg_attr(test, allow(dead_code))]
const SERVICE_NAME: &str = "Aura Translation";

#[cfg(all(test, any(target_os = "windows", target_os = "macos")))]
fn test_secret_store() -> &'static std::sync::Mutex<std::collections::HashMap<String, String>>
{
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    static STORE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
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
            config.api_key =
                load_profile_api_key(&config.provider, Some(&config.active_profile_id))?;
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
            "当前构建不支持系统凭据存储。如需继续使用该服务商，请将 API Key 存储切换为明文配置备用。".to_string(),
        );
    }

    save_system_api_key(
        &config
            .provider
            .profile_secret_account_name(&config.active_profile_id),
        config.api_key.trim(),
    )?;
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
                    "当前构建不支持系统凭据存储。如需在此平台继续使用 API Key，请切换为明文配置备用。".to_string(),
                );
            }

            if config.api_key.is_empty() {
                delete_system_api_key(
                    &config
                        .provider
                        .profile_secret_account_name(&config.active_profile_id),
                )?;
            } else {
                save_system_api_key(
                    &config
                        .provider
                        .profile_secret_account_name(&config.active_profile_id),
                    &config.api_key,
                )?;
            }
        }
        ApiKeyStorage::PlaintextFallback | ApiKeyStorage::LegacyPlaintext => {
            config.api_key_storage = ApiKeyStorage::PlaintextFallback;
            if matches!(old_config.api_key_storage, ApiKeyStorage::System)
                && old_config.provider == config.provider
                && old_config.active_profile_id == config.active_profile_id
            {
                let _ = delete_system_api_key(
                    &config
                        .provider
                        .profile_secret_account_name(&config.active_profile_id),
                );
            }
        }
    }

    Ok(())
}

pub fn load_provider_api_key(
    provider: &Provider,
    profile_id: Option<&str>,
) -> Result<String, String> {
    if !provider.requires_api_key() {
        return Ok(String::new());
    }

    load_profile_api_key(provider, profile_id)
}

fn load_profile_api_key(provider: &Provider, profile_id: Option<&str>) -> Result<String, String> {
    if let Some(profile_id) = profile_id {
        let account_name = provider.profile_secret_account_name(profile_id);
        if let Some(api_key) = load_system_api_key(&account_name)? {
            return Ok(api_key);
        }
    }

    Ok(load_system_api_key(provider.secret_account_name())?.unwrap_or_default())
}

#[cfg(all(not(test), any(target_os = "windows", target_os = "macos")))]
fn load_system_api_key(account_name: &str) -> Result<Option<String>, String> {
    use keyring::{Entry, Error};

    let entry = Entry::new(SERVICE_NAME, account_name)
        .map_err(|err| format!("无法打开系统凭据项：{err}"))?;

    match entry.get_password() {
        Ok(api_key) => Ok(Some(api_key)),
        Err(Error::NoEntry) => Ok(None),
        Err(err) => Err(format!("无法从系统凭据库读取 API Key：{err}")),
    }
}

#[cfg(all(test, any(target_os = "windows", target_os = "macos")))]
fn load_system_api_key(account_name: &str) -> Result<Option<String>, String> {
    Ok(test_secret_store()
        .lock()
        .unwrap()
        .get(account_name)
        .cloned())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn load_system_api_key(_account_name: &str) -> Result<Option<String>, String> {
    Ok(None)
}

#[cfg(all(not(test), any(target_os = "windows", target_os = "macos")))]
fn save_system_api_key(account_name: &str, api_key: &str) -> Result<(), String> {
    use keyring::Entry;

    let entry = Entry::new(SERVICE_NAME, account_name)
        .map_err(|err| format!("无法打开系统凭据项：{err}"))?;
    entry
        .set_password(api_key)
        .map_err(|err| format!("无法将 API Key 存入系统凭据库：{err}"))
}

#[cfg(all(test, any(target_os = "windows", target_os = "macos")))]
fn save_system_api_key(account_name: &str, api_key: &str) -> Result<(), String> {
    test_secret_store()
        .lock()
        .unwrap()
        .insert(account_name.to_string(), api_key.to_string());
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn save_system_api_key(_account_name: &str, _api_key: &str) -> Result<(), String> {
    Err("当前构建不支持系统凭据存储。".to_string())
}

#[cfg(all(not(test), any(target_os = "windows", target_os = "macos")))]
fn delete_system_api_key(account_name: &str) -> Result<(), String> {
    use keyring::{Entry, Error};

    let entry = Entry::new(SERVICE_NAME, account_name)
        .map_err(|err| format!("无法打开系统凭据项：{err}"))?;

    match entry.delete_credential() {
        Ok(()) | Err(Error::NoEntry) => Ok(()),
        Err(err) => Err(format!("无法从系统凭据库移除 API Key：{err}")),
    }
}

#[cfg(all(test, any(target_os = "windows", target_os = "macos")))]
fn delete_system_api_key(account_name: &str) -> Result<(), String> {
    test_secret_store()
        .lock()
        .unwrap()
        .remove(account_name);
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn delete_system_api_key(_account_name: &str) -> Result<(), String> {
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

        let loaded = load_provider_api_key(&Provider::DeepSeek, Some("default"))
            .expect("secret should load");
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
            load_provider_api_key(&Provider::DeepSeek, Some("default"))
                .expect("secret should load"),
            "sk-test"
        );
    }

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    #[test]
    fn profile_secret_reads_fall_back_to_legacy_provider_entry() {
        save_system_api_key(Provider::DeepSeek.secret_account_name(), "sk-legacy").unwrap();

        assert_eq!(
            load_provider_api_key(&Provider::DeepSeek, Some("focus-jp")).unwrap(),
            "sk-legacy"
        );
    }

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    #[test]
    fn profiles_for_the_same_provider_keep_independent_keys() {
        let old_config = AppConfig::default();
        let mut first = secure_config();
        first.active_profile_id = "first".to_string();
        persist_api_key(&mut first, &old_config).unwrap();

        let mut second = secure_config();
        second.active_profile_id = "second".to_string();
        second.api_key = "sk-second".to_string();
        persist_api_key(&mut second, &first).unwrap();

        assert_eq!(
            load_provider_api_key(&Provider::DeepSeek, Some("first")).unwrap(),
            "sk-test"
        );
        assert_eq!(
            load_provider_api_key(&Provider::DeepSeek, Some("second")).unwrap(),
            "sk-second"
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
