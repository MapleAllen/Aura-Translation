# Translation Engine Module Description

## Module Name

Translation Engine

## Purpose

The Translation Engine is the core AI service bridge of Aura Translation. It accepts raw text from the frontend, constructs a structured prompt, and forwards the request to the **DeepSeek Chat Completions API** (the sole supported provider in the current implementation) with streaming enabled. It relays each translated token back to the UI in real time via Tauri event emission, allowing the user to watch the translation appear word-by-word rather than waiting for the full response.

The module is the application's only external-network surface. The long-term architectural goal is to support any **OpenAI-compatible endpoint**, including self-hosted open-source models (e.g., Ollama running a local Mistral or NLLB model), so that users who require privacy and full control over their translation pipeline can operate the application entirely offline.

## Current Implementation

When the user presses `Ctrl+T`, the root page (`+page.svelte`) invokes the `translate_text` Tauri command with six arguments: `text`, `source_lang`, `target_lang`, `api_key`, `model`, and `request_id`. The Rust handler in `translate.rs` builds a JSON body for the DeepSeek `/chat/completions` endpoint with `"stream": true` and a temperature of `0.3`. A shared `reqwest::Client` (stored in `tauri::State` and created once at app startup) sends the POST request with a `Bearer` authorization header, reusing connection pools across invocations.

Before the HTTP call, a `tokio::sync::oneshot` cancellation channel is registered in the `CancellationRegistry` (a `HashMap<u64, oneshot::Sender<()>>` behind an `Arc<Mutex<…>>`, also stored in `tauri::State`) keyed by `request_id`. The response is consumed as a raw byte stream (`response.bytes_stream()`). A line-level buffer accumulates incoming bytes and scans for newlines to extract SSE (`data: …`) lines. The stream loop runs inside a `tokio::select!` that races the SSE `stream.next()` against the cancellation receiver. Each JSON line is parsed into a `StreamChunk` struct. Non-empty `delta.content` values are emitted to the frontend as `translation-chunk` events. The sentinel line `data: [DONE]` triggers a final `translation-done` event and terminates the loop. If the cancellation signal fires, the loop exits early and emits `translation-done` to preserve any partial text already sent. Network or HTTP errors at any stage emit a `translation-error` event and propagate an `Err(String)` back to the invoker.

A separate `cancel_translate` Tauri command accepts a `request_id`, looks up the corresponding sender in the registry, sends the cancellation signal, and removes the entry.

Language directionality is controlled by a system prompt: when `source_lang == "auto"`, the prompt asks the model to auto-detect and translate; otherwise it explicitly names both languages.

### Capabilities

**Core workflow**
- Accepts `text`, `source_lang`, `target_lang`, `api_key`, `model`, and `request_id` as command arguments
- Auto-detects source language when `source_lang == "auto"`
- Posts to `https://api.deepseek.com/chat/completions` with `"stream": true`
- Uses temperature `0.3` for deterministic, professional translation quality
- Streams tokens using SSE line parsing with a rolling string buffer inside a `tokio::select!` loop

**Event protocol**
- Emits `translation-chunk (String)` for each non-empty delta token
- Emits `translation-done (())` when the SSE stream terminates with `[DONE]` or when the request is cancelled
- Emits `translation-error (String)` on network failure, non-2xx HTTP status, or stream read error

**Request cancellation**
- Each request is assigned a `request_id: u64` by the frontend
- A `oneshot::Sender` is stored in the `CancellationRegistry` keyed by `request_id`
- The stream loop runs inside `tokio::select!`, racing `stream.next()` against the cancellation receiver
- On cancellation: emits `translation-done` (not `translation-error`) so the UI preserves partial text
- Registry entries are cleaned up on completion, error, or cancellation
- `cancel_translate(request_id)` Tauri command triggers cancellation from the frontend

**Connection pooling**
- A single `reqwest::Client` is created at app startup and shared across all invocations via `tauri::State<Client>`
- Connection pools are reused across consecutive translations, avoiding per-request TLS handshake overhead

**Model selection**
- Supports `deepseek-v4-flash` (fast) and `deepseek-v4-pro` (quality) as selectable models; model name is passed directly by the caller

**Error handling**
- Network errors surface as `translation-error` events with a `Network error: …` prefix
- HTTP non-2xx errors include the status code and raw body text
- Stream read failures produce a `Stream error: …` event

## Architecture

Stateful async service module with two Tauri commands (`translate_text`, `cancel_translate`) and two shared `tauri::State` resources (`reqwest::Client` for connection pooling, `CancellationRegistry` for in-flight request tracking). Uses `reqwest` for HTTP, `futures_util::StreamExt` for async stream iteration, and `tokio::select!` for cooperative cancellation.

### Rust Backend (`src-tauri/src/`)

- `translate.rs`
  - `CancellationRegistry` type alias: `Arc<Mutex<HashMap<u64, oneshot::Sender<()>>>>` — exported for registration in `lib.rs`
  - `translate_text(app, client, registry, text, source_lang, target_lang, api_key, model, request_id) -> Result<(), String>`: primary Tauri command; sets up cancellation channel, delegates to `translate_stream`, cleans up registry on completion
  - `translate_stream(app, client, cancel_rx, text, source_lang, target_lang, api_key, model) -> Result<(), String>`: internal function; runs the SSE loop inside `tokio::select!` racing stream chunks against cancellation
  - `cancel_translate(registry, request_id) -> Result<(), String>`: Tauri command; removes and signals the cancellation sender for the given request ID
  - `StreamChunk` / `StreamChoice` / `StreamDelta`: private deserialization structs for the OpenAI-compatible SSE JSON format
  - Buffer management: single mutable `String` buffer, drained line-by-line using `buffer.find('\n')` and string slicing

### Integration Points

- `src-tauri/src/lib.rs`
  - `translate::translate_text` and `translate::cancel_translate`: both registered in the `invoke_handler` at builder construction.
  - `reqwest::Client` and `CancellationRegistry`: registered as managed Tauri state via `.manage()` at builder construction.
- `ui/routes/+page.svelte`
  - `invoke('translate_text', { text, sourceLang, targetLang, apiKey, model, requestId })`: caller passes all six runtime arguments including a monotonically incrementing request ID; the call is awaited to detect command-level errors.
  - `invoke('cancel_translate', { requestId })`: called when the user clicks the cancel button, changes language mid-stream, or triggers a new translation while one is in progress.
  - `listen('translation-chunk', …)`: accumulates tokens into `translatedText`; transitions `appState` from `loading` → `streaming` on first token; guarded to only append when `appState === 'streaming'`.
  - `listen('translation-done', …)`: transitions `appState` to `result`; also fires on clean cancellation.
  - `listen('translation-error', …)`: transitions `appState` to `error` and stores the message.

## Current Limitations

- **Fixed API endpoint** — the DeepSeek base URL (`https://api.deepseek.com/chat/completions`) is a hardcoded string literal; switching to an alternative provider or a self-hosted model requires a code change.
- **No retry logic** — transient network errors are surfaced immediately as `translation-error` events with no retry or back-off.
- **Buffer does not handle `\r\n`** — SSE line splitting uses only `\n`; `\r\n` line endings from a proxy or non-standard server could leave `\r` artifacts in emitted tokens.
- **Temperature is hardcoded** — `0.3` cannot be adjusted without a code change; no setting is exposed in the UI or config.
- **No token usage tracking** — the SSE `usage` field from the final chunk is discarded; token cost cannot be logged or surfaced.
- **Dismiss does not trigger cancellation from Rust** — the frontend calls `cancel_translate` on user-initiated cancel and mid-stream language switch, but dismissing the popup via `Esc` or focus loss does not currently invoke `cancel_translate`; the Rust task continues until the SSE stream finishes naturally.

## Future Directions

- Replace the hardcoded API endpoint with a configurable `api_base_url` field in `AppConfig`.
- Add exponential back-off retry (up to 3 attempts) for network-level failures.
- Support `\r\n` line ending normalization in the SSE buffer.
- Add user-configurable temperature via `AppConfig` with a slider in the Settings panel.
- Surface cumulative token count from the SSE final `usage` chunk to the UI for cost awareness.
- Support streaming from alternative OpenAI-compatible endpoints (Ollama, OpenRouter, self-hosted NLLB, etc.) to enable fully offline, privacy-preserving translation without a cloud API key.
- Wire `cancel_translate` into the dismiss flow so that hiding the popup also cancels the in-flight Rust task.
