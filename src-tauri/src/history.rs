use crate::config::Provider;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_HISTORY_ENTRIES: usize = 50;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranslationUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TranslationHistoryStatus {
    Success,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranslationHistoryEntry {
    pub id: String,
    pub source_text: String,
    pub translated_text: String,
    pub error_message: Option<String>,
    pub source_lang: String,
    pub target_lang: String,
    pub provider: Provider,
    pub model: String,
    #[serde(default)]
    pub usage: Option<TranslationUsage>,
    pub status: TranslationHistoryStatus,
    pub created_at_ms: u64,
}

#[derive(Default)]
pub struct TranslationHistoryStore {
    entries: Vec<TranslationHistoryEntry>,
}

impl TranslationHistoryStore {
    pub fn load_with_issues() -> (Self, Vec<String>) {
        let path = history_path();
        let mut issues = Vec::new();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<Vec<TranslationHistoryEntry>>(&content) {
                    Ok(entries) => (Self { entries }, issues),
                    Err(err) => {
                        issues.push(format!(
                            "翻译历史文件 {} 解析失败：{}。Aura 已以空历史状态启动。",
                            path.display(),
                            err
                        ));
                        (Self::default(), issues)
                    }
                },
                Err(err) => {
                    issues.push(format!(
                        "翻译历史文件 {} 读取失败：{}。Aura 已以空历史状态启动。",
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

    pub fn list(&self) -> Vec<TranslationHistoryEntry> {
        self.entries.clone()
    }

    pub fn record_success(
        &mut self,
        source_text: &str,
        translated_text: &str,
        source_lang: &str,
        target_lang: &str,
        provider: &Provider,
        model: &str,
        usage: Option<TranslationUsage>,
    ) -> Result<(), String> {
        self.insert_entry(TranslationHistoryEntry {
            id: next_history_id(),
            source_text: source_text.to_string(),
            translated_text: translated_text.to_string(),
            error_message: None,
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
            provider: provider.clone(),
            model: model.to_string(),
            usage,
            status: TranslationHistoryStatus::Success,
            created_at_ms: current_timestamp_ms(),
        })
    }

    pub fn record_error(
        &mut self,
        source_text: &str,
        error_message: &str,
        source_lang: &str,
        target_lang: &str,
        provider: &Provider,
        model: &str,
        usage: Option<TranslationUsage>,
    ) -> Result<(), String> {
        self.insert_entry(TranslationHistoryEntry {
            id: next_history_id(),
            source_text: source_text.to_string(),
            translated_text: String::new(),
            error_message: Some(error_message.to_string()),
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
            provider: provider.clone(),
            model: model.to_string(),
            usage,
            status: TranslationHistoryStatus::Error,
            created_at_ms: current_timestamp_ms(),
        })
    }

    pub fn find(&self, entry_id: &str) -> Option<TranslationHistoryEntry> {
        self.entries.iter().find(|entry| entry.id == entry_id).cloned()
    }

    pub fn delete(&mut self, entry_id: &str) -> Result<Vec<TranslationHistoryEntry>, String> {
        self.entries.retain(|entry| entry.id != entry_id);
        self.save()?;
        Ok(self.list())
    }

    pub fn clear(&mut self) -> Result<Vec<TranslationHistoryEntry>, String> {
        self.entries.clear();
        self.save()?;
        Ok(Vec::new())
    }

    fn insert_entry(&mut self, entry: TranslationHistoryEntry) -> Result<(), String> {
        self.entries.insert(0, entry);
        if self.entries.len() > MAX_HISTORY_ENTRIES {
            self.entries.truncate(MAX_HISTORY_ENTRIES);
        }
        self.save()
    }

    fn save(&self) -> Result<(), String> {
        #[cfg(test)]
        {
            Ok(())
        }

        #[cfg(not(test))]
        {
            let path = history_path();
            let content =
                serde_json::to_string_pretty(&self.entries).map_err(|err| err.to_string())?;
            let tmp_path = path.with_extension("json.tmp");
            fs::write(&tmp_path, content).map_err(|err| err.to_string())?;
            fs::rename(&tmp_path, &path).map_err(|err| err.to_string())?;
            Ok(())
        }
    }
}

fn history_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aura-translation");
    fs::create_dir_all(&config_dir).ok();
    config_dir.join("history.json")
}

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn next_history_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("history-{}-{}", current_timestamp_ms(), sequence)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_entries_newest_first_and_prunes_to_limit() {
        let mut store = TranslationHistoryStore::default();

        for index in 0..55 {
            store
                .record_success(
                    &format!("source {index}"),
                    &format!("translated {index}"),
                    "auto",
                    "Chinese",
                    &Provider::DeepSeek,
                    "deepseek-chat",
                    None,
                )
                .expect("history record should succeed");
        }

        let entries = store.list();
        assert_eq!(entries.len(), MAX_HISTORY_ENTRIES);
        assert_eq!(entries.first().unwrap().source_text, "source 54");
        assert_eq!(entries.last().unwrap().source_text, "source 5");
    }

    #[test]
    fn deletes_and_clears_entries() {
        let mut store = TranslationHistoryStore::default();
        store
            .record_error(
                "hello",
                "Translation failed.",
                "English",
                "Chinese",
                &Provider::OpenRouter,
                "mistral",
                None,
            )
            .expect("history record should succeed");
        let entry_id = store.list().first().unwrap().id.clone();

        let after_delete = store.delete(&entry_id).expect("delete should succeed");
        assert!(after_delete.is_empty());

        store
            .record_success(
                "hello",
                "你好",
                "English",
                "Chinese",
                &Provider::DeepSeek,
                "deepseek-chat",
                Some(TranslationUsage {
                    prompt_tokens: 12,
                    completion_tokens: 4,
                    total_tokens: 16,
                }),
            )
            .expect("history record should succeed");
        assert_eq!(store.list().len(), 1);
        assert_eq!(store.list()[0].usage.as_ref().unwrap().total_tokens, 16);
        assert!(store.clear().expect("clear should succeed").is_empty());
    }
}
