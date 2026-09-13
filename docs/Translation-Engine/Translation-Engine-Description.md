# Translation Engine Module Description

## Module Name

Translation Engine

## Purpose

The Translation Engine is the core AI service bridge of Aura Translation. It accepts raw text from the frontend, constructs a structured prompt, and forwards the request to any **OpenAI-compatible chat completions endpoint** (DeepSeek, OpenRouter, Ollama, or self-hosted) with streaming enabled. It relays each translated token back to the UI in real time via Tauri event emission, allowing the user to watch the translation appear word-by-word rather than waiting for the full response.

The module targets any **OpenAI-compatible endpoint**, including self-hosted open-source models (e.g. Ollama running locally), so users who require privacy can operate the application entirely offline. Provider selection (DeepSeek, OpenRouter, Ollama), base URL, model list, and auth header format are driven by `AppConfig` and a centralized Rust provider mapping.

## Current Implementation

When the user triggers the hotkey, the root page (`+page.svelte`) invokes the `translate_text` Tauri command with arguments: `text`, `source_lang`, `target_lang`, `api_key`, `model`, `request_id`, `api_base_url`, and `provider`. The Rust handler in `translate.rs` builds a JSON body for the `/chat/completions` endpoint with `"stream": true` and a temperature of `0.3`. A shared `reqwest::Client` (stored in `tauri::State` and created once at app startup) sends the POST request, reusing connection pools across invocations. The auth header format is provider-specific: `Bearer` for DeepSeek/OpenRouter, `Bearer` + referer headers for OpenRouter, and none for local Ollama.

Before the HTTP call, a `tokio::sync::oneshot` cancellation channel is registered in the `CancellationRegistry` (a `HashMap<u64, oneshot::Sender<()>>` behind an `Arc<Mutex<…>>`, also stored in `tauri::State`) keyed by `request_id`. The response is consumed as a raw byte stream (`response.bytes_stream()`). A line-level buffer accumulates incoming bytes and scans for newlines to extract SSE (`data: …`) lines. The stream loop runs inside a `tokio::select!` that races the SSE `stream.next()` against the cancellation receiver. Each JSON line is parsed into a `StreamChunk` struct. Non-empty `delta.content` values are emitted to the frontend as `translation-chunk { request_id, content }` events. The sentinel line `data: [DONE]` triggers a final `translation-done { request_id }` event and terminates the loop. If the cancellation signal fires, the loop exits early and emits `translation-done { request_id }` to preserve any partial text already sent. Network, HTTP, and malformed SSE errors emit `translation-error { request_id, message }` and propagate an `Err(String)` back to the invoker.

A separate `cancel_translate` Tauri command accepts a `request_id`, looks up the corresponding sender in the registry, sends the cancellation signal, and removes the entry.

Transient network errors (connection failures, timeouts, HTTP 5xx) are retried with exponential back-off: 1 initial request plus up to 3 retries with delays of 200 ms, 600 ms, and 1800 ms. A `translation-retry { request_id, attempt }` event is emitted before each retry so the UI can indicate activity. HTTP 4xx errors (auth failure, bad request) are surfaced immediately without retry.

### Timeouts

The shared client is built by `build_http_client()` with an explicit `connect_timeout` (10 s) and `timeout` (45 s). `reqwest::Client::new()`, used previously, applied no timeout at all, so a provider that accepted a request and then went silent left the UI waiting indefinitely.

For a streaming body the `timeout` behaves as an idle budget per read rather than a cap on total duration, so a healthy stream of any length keeps running while a dropped connection surfaces as an error. `is_retryable_transport_error()` already classifies timeouts as retryable, so a stalled provider follows the existing retry path.

Transport failures are rendered by `describe_transport_error()`, which appends the underlying cause chain. `reqwest::Error`'s own `Display` stops at "error sending request for url (...)", which hid whether the request timed out, the connection was refused, or DNS failed.

### First-run trial translation

`trial_translate` runs one real translation for onboarding and returns the text to the caller instead of streaming it through the translation-window events, which are not mounted while Settings is in front. It uses a `CollectingSink` that reuses the same streaming engine rather than a second request path. Inputs are validated by `validate_trial_inputs()` before any network call, and nothing from the trial is persisted.

Language directionality is controlled by a system prompt: when `source_lang == "auto"`, the prompt asks the model to auto-detect and translate; otherwise it explicitly names both languages.

### Capabilities

**Core workflow**
- Accepts `text`, `source_lang`, `target_lang`, `api_key`, `model`, `request_id`, `api_base_url`, and `provider` as command arguments
- Auto-detects source language when `source_lang == "auto"`
- Posts to `{api_base_url}/chat/completions` with `"stream": true`
- Uses temperature `0.3` for deterministic, professional translation quality
- Streams tokens using SSE line parsing with a rolling string buffer inside a `tokio::select!` loop
- Retries transient failures with exponential back-off (1 initial request plus up to 3 retries, 200/600/1800 ms)

**Event protocol**
- Emits `translation-chunk ({ request_id, content })` for each non-empty delta token
- Emits `translation-done ({ request_id })` when the SSE stream terminates with `[DONE]` or when the request is cancelled
- Emits `translation-error ({ request_id, message })` on non-retryable failure, retry exhaustion, malformed SSE JSON, or stream read error
- Emits `translation-retry ({ request_id, attempt })` before each retry attempt

**Request cancellation**
- Each request is assigned a `request_id: u64` by the frontend
- A `oneshot::Sender` is stored in the `CancellationRegistry` keyed by `request_id`
- The stream loop runs inside `tokio::select!`, racing `stream.next()` against the cancellation receiver
- On cancellation: emits `translation-done { request_id }` (not `translation-error`) so the UI preserves partial text for the current request
- Registry entries are cleaned up on completion, error, or cancellation
- `cancel_translate(request_id)` Tauri command triggers cancellation from the frontend

**Connection pooling**
- A single `reqwest::Client` is created at app startup and shared across all invocations via `tauri::State<Client>`
- Connection pools are reused across consecutive translations, avoiding per-request TLS handshake overhead

**Model selection**
- Supports provider default model lists from `AppConfig`, including `deepseek-chat`, `deepseek-reasoner`, OpenRouter defaults, and local Ollama model names; model name is passed directly by the caller

**Error handling**
- Non-retryable network errors surface as `translation-error` events immediately
- HTTP non-2xx errors include the status code and raw body text; 4xx errors are surfaced immediately, 5xx errors are retried
- Stream read failures produce a `Stream error: …` event
- After all retry attempts are exhausted, the last error is surfaced as `translation-error { request_id, message }`

## Architecture

Stateful async service module with two Tauri commands (`translate_text`, `cancel_translate`) and two shared `tauri::State` resources (`reqwest::Client` for connection pooling, `CancellationRegistry` for in-flight request tracking). Uses `reqwest` for HTTP, `futures_util::StreamExt` for async stream iteration, and `tokio::select!` for cooperative cancellation.

### Rust Backend (`src-tauri/src/`)

- `translate.rs`
  - `CancellationRegistry` type alias: `Arc<Mutex<HashMap<u64, oneshot::Sender<()>>>>` — exported for registration in `lib.rs`
  - `translate_text(app, client, registry, text, source_lang, target_lang, api_key, model, request_id, api_base_url, provider) -> Result<(), String>`: primary Tauri command; sets up cancellation channel, delegates to `translate_stream`, cleans up registry on completion
  - `translate_stream(app, client, cancel_rx, text, source_lang, target_lang, api_key, model, api_base_url, provider, request_id) -> Result<(), String>`: internal function; runs the SSE loop inside `tokio::select!` racing stream chunks against cancellation
  - `cancel_translate(registry, request_id) -> Result<(), String>`: Tauri command; removes and signals the cancellation sender for the given request ID
  - `StreamChunk` / `StreamChoice` / `StreamDelta`: private deserialization structs for the OpenAI-compatible SSE JSON format
  - Buffer management: single mutable `String` buffer, drained line-by-line using `buffer.find('\n')` and string slicing

### Integration Points

- `src-tauri/src/lib.rs`
  - `translate::translate_text` and `translate::cancel_translate`: both registered in the `invoke_handler` at builder construction.
  - `reqwest::Client` and `CancellationRegistry`: registered as managed Tauri state via `.manage()` at builder construction.
- `ui/routes/+page.svelte`
  - `invoke('translate_text', { text, sourceLang, targetLang, apiKey, model, requestId, apiBaseUrl, provider })`: caller passes runtime arguments including a monotonically incrementing request ID; the call is awaited to detect command-level errors.
  - `invoke('cancel_translate', { requestId })`: called when the user clicks the cancel button, changes language mid-stream, or triggers a new translation while one is in progress.
  - `listen('translation-chunk', …)`: ignores stale `request_id` values, accumulates current-request tokens into `translatedText`, and transitions `appState` from `loading` → `streaming` on first token.
  - `listen('translation-done', …)`: ignores stale `request_id` values and transitions `appState` to `result`; also fires on clean cancellation.
  - `listen('translation-error', …)`: ignores stale `request_id` values, transitions `appState` to `error`, and stores the message.

## Current Limitations

- **Buffer edge cases** — malformed `data:` JSON is now surfaced as `translation-error`, but non-`data:` SSE lines are still ignored as comments/metadata.
- **Temperature is hardcoded** — `0.3` cannot be adjusted without a code change; no setting is exposed in the UI or config.
- **No token usage tracking** — the SSE `usage` field from the final chunk is discarded; token cost cannot be logged or surfaced.

## Future Directions

- Add user-configurable temperature via `AppConfig` with a slider in the Settings panel.
- Surface cumulative token count from the SSE final `usage` chunk to the UI for cost awareness.
