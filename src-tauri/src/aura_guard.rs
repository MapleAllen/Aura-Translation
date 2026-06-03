use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct AuraGuardBlock {
    pub reason: String,
}

pub fn detect_sensitive_clipboard(text: &str) -> Option<AuraGuardBlock> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lowered = trimmed.to_ascii_lowercase();
    if contains_secret_keyword(&lowered) {
        return Some(AuraGuardBlock {
            reason: "Aura 模式已跳过本次剪贴板变化：内容疑似密码或令牌。".to_string(),
        });
    }

    if looks_like_known_secret_prefix(trimmed) || looks_like_jwt(trimmed) {
        return Some(AuraGuardBlock {
            reason: "Aura 模式已跳过本次剪贴板变化：内容匹配常见凭据格式。".to_string(),
        });
    }

    if looks_like_secret_blob(trimmed) {
        return Some(AuraGuardBlock {
            reason: "Aura 模式已跳过本次剪贴板变化：内容疑似较长的密钥字符串。".to_string(),
        });
    }

    None
}

fn contains_secret_keyword(lowered: &str) -> bool {
    [
        "password",
        "passwd",
        "pwd=",
        "secret",
        "api_key",
        "apikey",
        "token=",
        "bearer ",
        "authorization:",
    ]
    .iter()
    .any(|keyword| lowered.contains(keyword))
}

fn looks_like_known_secret_prefix(text: &str) -> bool {
    [
        "sk-",
        "ghp_",
        "github_pat_",
        "glpat-",
        "xoxb-",
        "xoxp-",
        "AKIA",
        "AIza",
    ]
    .iter()
    .any(|prefix| text.starts_with(prefix))
}

fn looks_like_jwt(text: &str) -> bool {
    let parts: Vec<&str> = text.split('.').collect();
    if parts.len() != 3 || !text.starts_with("eyJ") {
        return false;
    }

    parts.iter().all(|part| {
        part.len() >= 8
            && part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '='))
    })
}

fn looks_like_secret_blob(text: &str) -> bool {
    if text.len() < 24 || text.chars().any(char::is_whitespace) {
        return false;
    }

    if !text
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | '+' | '='))
    {
        return false;
    }

    let has_letter = text.chars().any(|c| c.is_ascii_alphabetic());
    let has_digit = text.chars().any(|c| c.is_ascii_digit());
    has_letter && has_digit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_password_like_text() {
        let block = detect_sensitive_clipboard("password=supersecret123").unwrap();
        assert!(block.reason.contains("密码或令牌"));
    }

    #[test]
    fn blocks_known_secret_prefixes() {
        let block = detect_sensitive_clipboard("sk-8f5c1b2a9d7e3f4a6b8c0d1e2f3a4b5").unwrap();
        assert!(block.reason.contains("常见凭据格式"));
    }

    #[test]
    fn blocks_jwt_like_values() {
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkF1cmEiLCJpYXQiOjE1MTYyMzkwMjJ9.signaturePart123";
        let block = detect_sensitive_clipboard(token).unwrap();
        assert!(block.reason.contains("常见凭据格式"));
    }

    #[test]
    fn allows_regular_sentences() {
        assert!(detect_sensitive_clipboard("Translate this paragraph into Chinese.").is_none());
    }
}
