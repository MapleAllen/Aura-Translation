use crate::config::{AppConfig, Provider};
use reqwest::{
    header::{HeaderName, HeaderValue},
    Client, StatusCode,
};
use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeStatusLevel {
    Ready,
    NeedsSetup,
}

#[derive(Clone, Serialize, PartialEq, Eq)]
pub struct RuntimeChecklistItem {
    pub code: String,
    pub label: String,
    pub ok: bool,
}

#[derive(Clone, Serialize, PartialEq, Eq)]
pub struct RuntimeStatus {
    pub level: RuntimeStatusLevel,
    pub summary: String,
    pub can_translate_now: bool,
    pub checklist: Vec<RuntimeChecklistItem>,
}

#[derive(Clone, Serialize, PartialEq, Eq)]
pub struct ProviderProbeResult {
    pub ok: bool,
    pub message: String,
}

pub fn is_translation_ready(config: &AppConfig) -> bool {
    has_base_url(config) && has_model(config) && has_required_api_key(config)
}

pub fn should_prompt_for_setup(config: &AppConfig) -> bool {
    !is_translation_ready(config)
}

pub fn build_runtime_status(config: &AppConfig) -> RuntimeStatus {
    let checklist = vec![
        RuntimeChecklistItem {
            code: "provider".to_string(),
            label: format!("服务商：{}", provider_label(&config.provider)),
            ok: true,
        },
        RuntimeChecklistItem {
            code: "base_url".to_string(),
            label: "API 地址已配置".to_string(),
            ok: has_base_url(config),
        },
        RuntimeChecklistItem {
            code: "model".to_string(),
            label: "默认模型已选择".to_string(),
            ok: has_model(config),
        },
        RuntimeChecklistItem {
            code: "api_key".to_string(),
            label: if requires_api_key(&config.provider) {
                "API Key 已保存".to_string()
            } else {
                "Ollama 不需要 API Key".to_string()
            },
            ok: has_required_api_key(config),
        },
    ];

    if is_translation_ready(config) {
        RuntimeStatus {
            level: RuntimeStatusLevel::Ready,
            summary: format!(
                "Aura 已准备好使用 {} 和模型 {} 翻译。",
                provider_label(&config.provider),
                config.model
            ),
            can_translate_now: true,
            checklist,
        }
    } else {
        RuntimeStatus {
            level: RuntimeStatusLevel::NeedsSetup,
            summary: missing_setup_message(config),
            can_translate_now: false,
            checklist,
        }
    }
}

pub async fn probe_provider(client: &Client, config: &AppConfig) -> ProviderProbeResult {
    if !has_base_url(config) {
        return ProviderProbeResult {
            ok: false,
            message: "测试服务商前，请先填写 API 地址。".to_string(),
        };
    }

    if !has_model(config) {
        return ProviderProbeResult {
            ok: false,
            message: "测试服务商前，请先选择默认模型。".to_string(),
        };
    }

    if !has_required_api_key(config) {
        return ProviderProbeResult {
            ok: false,
            message: "请先保存 API Key；如果想使用本地服务商，可以切换到 Ollama。"
                .to_string(),
        };
    }

    let headers = match provider_headers(&config.provider, &config.api_key) {
        Ok(headers) => headers,
        Err(message) => {
            return ProviderProbeResult { ok: false, message };
        }
    };

    let body = serde_json::json!({
        "model": config.model,
        "messages": [
            { "role": "system", "content": "Reply with OK only." },
            { "role": "user", "content": "ping" }
        ],
        "temperature": 0,
        "stream": false,
        "max_tokens": 4
    });

    let mut request = client
        .post(build_chat_completions_url(&config.api_base_url))
        .header("Content-Type", "application/json");
    for (name, value) in headers {
        request = request.header(name, value);
    }

    let response = match request.json(&body).send().await {
        Ok(response) => response,
        Err(err) => {
            return ProviderProbeResult {
                ok: false,
                message: format!(
                    "无法连接服务商，请检查网络连接和 API 地址。（{}）",
                    err
                ),
            };
        }
    };

    if response.status().is_success() {
        return ProviderProbeResult {
            ok: true,
            message: format!(
                "{} 的服务商测试成功，使用模型 {}。",
                provider_label(&config.provider),
                config.model
            ),
        };
    }

    let status = response.status();
    let raw_body = response.text().await.unwrap_or_default();
    ProviderProbeResult {
        ok: false,
        message: match status {
            StatusCode::UNAUTHORIZED => {
                "认证失败。请检查当前服务商的 API Key。".to_string()
            }
            StatusCode::FORBIDDEN => {
                "服务商拒绝了本次请求。请检查账号权限和模型访问权限。"
                    .to_string()
            }
            StatusCode::TOO_MANY_REQUESTS => {
                "服务商触发了限流。请稍等后重试。".to_string()
            }
            _ => {
                let trimmed = raw_body.trim();
                if trimmed.is_empty() {
                    format!("服务商测试失败，HTTP {}。", status.as_u16())
                } else {
                    let preview = preview_body(trimmed, 160);
                    format!(
                        "服务商测试失败，HTTP {}：{}",
                        status.as_u16(),
                        preview
                    )
                }
            }
        },
    }
}

fn requires_api_key(provider: &Provider) -> bool {
    !matches!(provider, Provider::Ollama)
}

fn has_required_api_key(config: &AppConfig) -> bool {
    !requires_api_key(&config.provider) || !config.api_key.trim().is_empty()
}

fn has_model(config: &AppConfig) -> bool {
    !config.model.trim().is_empty()
}

fn has_base_url(config: &AppConfig) -> bool {
    !config.api_base_url.trim().is_empty()
}

fn missing_setup_message(config: &AppConfig) -> String {
    if !has_required_api_key(config) {
        return "请保存 API Key；如果想使用本地服务商，可以切换到 Ollama。".to_string();
    }

    if !has_model(config) {
        return "开始翻译前，请先选择默认模型。".to_string();
    }

    if !has_base_url(config) {
        return "请填写 API 地址，让 Aura 知道请求发送到哪里。".to_string();
    }

    "开始翻译前，请完成下面缺失的配置项。".to_string()
}

fn provider_label(provider: &Provider) -> &'static str {
    match provider {
        Provider::DeepSeek => "DeepSeek",
        Provider::OpenRouter => "OpenRouter",
        Provider::Ollama => "Ollama",
    }
}

fn preview_body(body: &str, max_chars: usize) -> String {
    let mut chars = body.chars();
    let preview = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{preview}...")
    } else {
        preview
    }
}

fn build_chat_completions_url(api_base_url: &str) -> String {
    format!("{}/chat/completions", api_base_url.trim_end_matches('/'))
}

fn provider_headers(
    provider: &Provider,
    api_key: &str,
) -> Result<Vec<(HeaderName, HeaderValue)>, String> {
    match provider {
        Provider::Ollama => Ok(Vec::new()),
        Provider::DeepSeek => Ok(vec![authorization_header(api_key)?]),
        Provider::OpenRouter => Ok(vec![
            authorization_header(api_key)?,
            (
                HeaderName::from_static("http-referer"),
                HeaderValue::from_static("https://github.com/MapleAllen/Aura-Translation"),
            ),
            (
                HeaderName::from_static("x-title"),
                HeaderValue::from_static("Aura Translation"),
            ),
        ]),
    }
}

fn authorization_header(api_key: &str) -> Result<(HeaderName, HeaderValue), String> {
    let value = HeaderValue::from_str(&format!("Bearer {}", api_key))
        .map_err(|err| format!("API Key 请求头无效：{}", err))?;
    Ok((HeaderName::from_static("authorization"), value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ApiKeyStorage;
    use crate::hotkey;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    fn deepseek_config() -> AppConfig {
        AppConfig {
            api_key: "sk-test".to_string(),
            api_key_storage: ApiKeyStorage::System,
            active_profile_id: "default".to_string(),
            model: "deepseek-chat".to_string(),
            source_lang: "auto".to_string(),
            target_lang: "Chinese".to_string(),
            hotkey: hotkey::default_hotkey().to_string(),
            aura_mode_enabled: false,
            aura_guard_enabled: true,
            window_pinned: false,
            provider: Provider::DeepSeek,
            api_base_url: "https://api.deepseek.com".to_string(),
            available_models: vec!["deepseek-chat".to_string()],
            settings_window_placement: None,
            pinned_translation_placement: None,
        }
    }

    #[test]
    fn ready_status_requires_no_extra_action() {
        let status = build_runtime_status(&deepseek_config());
        assert_eq!(status.level, RuntimeStatusLevel::Ready);
        assert!(status.can_translate_now);
        assert!(status.checklist.iter().all(|item| item.ok));
    }

    #[test]
    fn missing_api_key_requires_setup() {
        let mut config = deepseek_config();
        config.api_key.clear();

        let status = build_runtime_status(&config);
        assert_eq!(status.level, RuntimeStatusLevel::NeedsSetup);
        assert!(!status.can_translate_now);
        assert!(should_prompt_for_setup(&config));
        assert_eq!(
            status
                .checklist
                .iter()
                .find(|item| item.code == "api_key")
                .unwrap()
                .ok,
            false
        );
    }

    #[test]
    fn ollama_is_ready_without_api_key() {
        let mut config = deepseek_config();
        config.provider = Provider::Ollama;
        config.api_key.clear();
        config.api_base_url = "http://localhost:11434".to_string();
        config.model = "qwen2.5".to_string();

        let status = build_runtime_status(&config);
        assert_eq!(status.level, RuntimeStatusLevel::Ready);
        assert!(status.can_translate_now);
    }

    #[tokio::test]
    async fn provider_probe_handles_multibyte_error_body_preview() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(500).set_body_string("错误".repeat(90)))
            .mount(&server)
            .await;

        let mut config = deepseek_config();
        config.api_base_url = server.uri();

        let result = probe_provider(&Client::new(), &config).await;

        assert!(!result.ok);
        assert!(result.message.contains("HTTP 500"));
        assert!(result.message.ends_with("..."));
    }
}
