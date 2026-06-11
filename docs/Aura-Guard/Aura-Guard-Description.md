# Aura Guard Module Description

## Module Name

Aura Guard

## Purpose

Aura Guard is a clipboard content screening layer for Aura mode. When the user has Aura mode and Aura Guard both enabled, every clipboard change is screened before translation is triggered. If the clipboard text looks like a password, API key, JWT, or other credential, Aura Guard silently skips the translation request without surfacing an error to the user. Its job is to prevent Aura mode from accidentally translating and logging sensitive material that the user happened to copy.

## Current Implementation

`aura_guard.rs` exposes a single public entry point: `detect_sensitive_clipboard(text: &str) -> Option<AuraGuardBlock>`. Returning `Some(AuraGuardBlock { reason })` means the text should be blocked; returning `None` means it is safe to translate.

The function applies four heuristics in order:

1. **Empty check**: blank or whitespace-only text returns `None` immediately (no blocking needed).
2. **Keyword scan** (`contains_secret_keyword`): searches the lowercased text for a set of literal substrings: `"password"`, `"passwd"`, `"pwd="`, `"secret"`, `"api_key"`, `"apikey"`, `"token="`, `"bearer "`, `"authorization:"`. Any match blocks.
3. **Known prefix scan** (`looks_like_known_secret_prefix`): checks whether the text starts with a well-known token prefix: `"sk-"` (OpenAI/DeepSeek keys), `"ghp_"`, `"github_pat_"` (GitHub personal access tokens), `"glpat-"` (GitLab tokens), `"xoxb-"`, `"xoxp-"` (Slack tokens), `"AKIA"` (AWS access keys), `"AIza"` (Google API keys). JWT detection (`looks_like_jwt`) also runs in this step: three dot-separated segments, first segment starting with `"eyJ"`, all segments at least 8 characters of base64url-safe characters.
4. **Secret blob heuristic** (`looks_like_secret_blob`): flags text that is at least 24 characters long with no whitespace, composed entirely of `[a-zA-Z0-9\-_./+=]`, and containing at least one letter and one digit. This catches high-entropy credential strings that do not match known prefixes.

The screening result includes a Chinese reason string. Reason strings are:
- `"Aura 模式已跳过本次剪贴板变化：内容疑似密码或令牌。"` — keyword match
- `"Aura 模式已跳过本次剪贴板变化：内容匹配常见凭据格式。"` — prefix or JWT match
- `"Aura 模式已跳过本次剪贴板变化：内容疑似较长的密钥字符串。"` — blob heuristic match

The daemon calls `detect_sensitive_clipboard` in the Aura mode clipboard handler before deciding whether to invoke translation. If a block is returned, the translation is skipped silently (no notification shown to the user by default).

### Capabilities

**Heuristic screening**
- Keyword-based detection: 9 literal keywords covering common secret field names and auth header prefixes
- Known-prefix detection: 8 vendor-specific token prefixes
- JWT detection: 3-part dot-separated structure starting with `"eyJ"`, all parts base64url-safe
- High-entropy blob detection: ≥24 chars, no whitespace, alphanumeric+`-_./+=`, at least one letter and one digit

**Graceful pass-through**
- Regular sentences, code snippets, and short text that do not match any heuristic pass through without blocking
- Empty and whitespace-only text is not blocked (Aura mode already ignores blank clipboard changes independently)

## Architecture

Stateless pure-function module. No managed state, no async operations. All four detection functions are private; only `detect_sensitive_clipboard` is public. Called synchronously in the Aura mode clipboard event handler inside `lib.rs`.

### Rust Backend (`src-tauri/src/`)

- `aura_guard.rs`
  - `AuraGuardBlock`: `{ reason: String }` — returned when the content is blocked
  - `detect_sensitive_clipboard(text: &str) -> Option<AuraGuardBlock>`: public entry point; runs four heuristics in order and returns the first match or `None`
  - `contains_secret_keyword(lowered: &str) -> bool` (private): checks 9 literal substrings in the lowercased text
  - `looks_like_known_secret_prefix(text: &str) -> bool` (private): checks 8 literal token prefixes using `starts_with`
  - `looks_like_jwt(text: &str) -> bool` (private): checks 3-part structure, `"eyJ"` prefix, length ≥ 8 per part, base64url-safe characters only
  - `looks_like_secret_blob(text: &str) -> bool` (private): checks length ≥ 24, no whitespace, allowed charset, at least one letter and one digit

### Integration Points

- `src-tauri/src/lib.rs`
  - Aura mode clipboard handler: calls `aura_guard::detect_sensitive_clipboard(clipboard_text)` when `AppConfig.aura_guard_enabled` is `true`
  - If a `Some(AuraGuardBlock)` is returned, the translation trigger is skipped; the `reason` string is currently not surfaced to the user

- `src-tauri/src/config.rs`
  - `AppConfig.aura_guard_enabled: bool`: controls whether the screening runs; defaults to `true`
  - Exposed in Settings so users can disable Aura Guard if they find it overly aggressive

- `ui/lib/SettingsPanel.svelte`
  - Renders an `aura_guard_enabled` toggle in the Aura Mode section of Settings

## Current Limitations

- **False positive risk on long base64 encoded content**: the blob heuristic may flag valid long base64 strings that are not credentials (e.g. a binary file embedded in JSON).
- **Reason strings are not surfaced to the user**: the `reason` field is populated but currently not shown; users have no visibility into why a clipboard change was skipped.
- **Keyword list is English-only**: the keyword scan does not detect credential field names in other languages or scripts.
- **No user-configurable allow-list**: there is no mechanism for users to mark specific clipboard content as safe and exempt it from future screening.
- **JWT detection requires exactly 3 segments**: valid JWTs with non-standard segment counts (e.g. encrypted JWE tokens) may not be detected.

## Future Directions

- Surface the block reason as a brief, dismissible notification or tray tooltip so users understand why Aura mode did not trigger.
- Add a configurable allow-list of regex patterns or literal prefixes that bypass screening.
- Extend the keyword list to cover common credential field names in other languages.
- Replace the blob heuristic with an entropy-based scorer for higher precision on high-entropy strings.
- Add detection for additional token formats (AWS session tokens, Azure SAS tokens, GCP service account keys).
