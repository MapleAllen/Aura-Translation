# Translation Engine Module Description

## Module Name

Translation Engine

## Purpose

The Translation Engine is the core AI service bridge of Aura Translation. It accepts raw text from the frontend, constructs a structured prompt, and forwards the request to the **DeepSeek Chat Completions API** (the sole supported provider in the current implementation) with streaming enabled. It relays each translated token back to the UI in real time via Tauri event emission, allowing the user to watch the translation appear word-by-word rather than waiting for the full response.

The module is the application's only external-network surface. The long-term architectural goal is to support any **OpenAI-compatible endpoint**, including self-hosted open-source models (e.g., Ollama running a local Mistral or NLLB model), so that users who require privacy and full control over their translation pipeline can operate the application entirely offline.

## Current Implementation

When the user presses `Ctrl+T`, the root page (`+page.svelte`) invokes the `translate_text` Tauri command with five arguments: `text`, `source_lang`, `target_lang`, `api_key`, and `model`. The Rust handler in `translate.rs` builds a JSON body for the DeepSeek `/chat/completions` endpoint with `"stream": true` and a temperature of `0.3`. A `reqwest::Client` (created fresh per call) sends the POST request with a `Bearer` authorization header.

The response is consumed as a raw byte stream (`response.bytes_stream()`). A line-level buffer accumulates incoming bytes and scans for newlines to extract SSE (`data: …`) lines. Each JSON line is parsed into a `StreamChunk` struct. Non-empty `delta.content` values are emitted to the frontend as `translation-chunk` events. The sentinel line `data: [DONE]` triggers a final `translation-done` event and terminates the loop. Network or HTTP errors at any stage emit a `translation-error` event and propagate an `Err(String)` back to the invoker.

Language directionality is controlled by a system prompt: when `source_lang == "auto"`, the prompt asks the model to auto-detect and translate; otherwise it explicitly names both languages.

### Capabilities

**Core workflow**
- Accepts `text`, `source_lang`, `target_lang`, `api_key`, and `model` as command arguments
- Auto-detects source language when `source_lang == "auto"`
- Posts to `https://api.deepseek.com/chat/completions` with `"stream": true`
- Uses temperature `0.3` for deterministic, professional translation quality
- Streams tokens using SSE line parsing with a rolling string buffer

**Event protocol**
- Emits `translation-chunk (String)` for each non-empty delta token
- Emits `translation-done (())` when the SSE stream terminates with `[DONE]`
- Emits `translation-error (String)` on network failure, non-2xx HTTP status, or stream read error

**Model selection**
- Supports `deepseek-v4-flash` (fast) and `deepseek-v4-pro` (quality) as selectable models; model name is passed directly by the caller

**Error handling**
- Network errors surface as `translation-error` events with a `Network error: …` prefix
- HTTP non-2xx errors include the status code and raw body text
- Stream read failures produce a `Stream error: …` event

## Architecture

Single-module, async service function. No persistent state; all request context is passed as function arguments. Uses `reqwest` for HTTP and `futures_util::StreamExt` for async stream iteration.

### Rust Backend (`src-tauri/src/`)

- `translate.rs`
  - Primary entry point: `pub async fn translate_text(app, text, source_lang, target_lang, api_key, model) -> Result<(), String>`
  - Registered as a Tauri command in `lib.rs` via `tauri::generate_handler!`
  - `StreamChunk` / `StreamChoice` / `StreamDelta`: private deserialization structs for the OpenAI-compatible SSE JSON format
  - Buffer management: single mutable `String` buffer, drained line-by-line using `buffer.find('\n')` and string slicing

### Integration Points

- `src-tauri/src/lib.rs`
  - `translate::translate_text`: registered in the `invoke_handler` at builder construction — this is the only Tauri command exposed from this module.
- `ui/routes/+page.svelte`
  - `invoke('translate_text', { text, sourceLang, targetLang, apiKey, model })`: caller passes all five runtime arguments; the call is awaited to detect command-level errors.
  - `listen('translation-chunk', …)`: accumulates tokens into `translatedText`; transitions `appState` from `loading` → `streaming` on first token.
  - `listen('translation-done', …)`: transitions `appState` to `result`.
  - `listen('translation-error', …)`: transitions `appState` to `error` and stores the message.

## Current Limitations

- **Per-request `reqwest::Client` instantiation** — a new `Client` is created for every `translate_text` invocation; connection-pool reuse is not exploited.
- **No request cancellation** — if the user dismisses the popup mid-stream, the Rust task continues running until the SSE stream finishes or the process exits; there is no `AbortHandle` or cancellation token wired to the dismiss flow.
- **Fixed API endpoint** — the DeepSeek base URL (`https://api.deepseek.com/chat/completions`) is a hardcoded string literal; switching to an alternative provider or a self-hosted model requires a code change.
- **No retry logic** — transient network errors are surfaced immediately as `translation-error` events with no retry or back-off.
- **Buffer does not handle `\r\n`** — SSE line splitting uses only `\n`; `\r\n` line endings from a proxy or non-standard server could leave `\r` artifacts in emitted tokens.
- **Temperature is hardcoded** — `0.3` cannot be adjusted without a code change; no setting is exposed in the UI or config.
- **No token usage tracking** — the SSE `usage` field from the final chunk is discarded; token cost cannot be logged or surfaced.

## Future Directions

- Add a `reqwest::Client` instance shared across calls via `tauri::State` to reuse connection pools.
- Support request cancellation using a `tokio::sync::oneshot` channel triggered by the dismiss event.
- Replace the hardcoded API endpoint with a configurable `api_base_url` field in `AppConfig`.
- Add exponential back-off retry (up to 3 attempts) for network-level failures.
- Support `\r\n` line ending normalization in the SSE buffer.
- Add user-configurable temperature via `AppConfig` with a slider in the Settings panel.
- Surface cumulative token count from the SSE final `usage` chunk to the UI for cost awareness.
- Support streaming from alternative OpenAI-compatible endpoints (Ollama, OpenRouter, self-hosted NLLB, etc.) to enable fully offline, privacy-preserving translation without a cloud API key.
