# Translation Engine Plan

## Objective

Evolve the Translation Engine from a functional MVP (DeepSeek-only, single-client, no cancellation) into a robust, **provider-agnostic** streaming translation service. The current implementation exclusively targets the DeepSeek API (`deepseek-v4-flash` / `deepseek-v4-pro`). The end state is a stateful Tauri service layer that pools HTTP connections, supports in-flight cancellation, retries transient failures transparently, and can route requests to **any OpenAI-compatible backend** — including self-hosted open-source models (e.g., Ollama + Mistral, or a local NLLB inference server) — for users who require full privacy and offline capability. None of these evolutions may change the frontend event contract.

## Design Principles

- **Event contract is stable.** The `translation-chunk`, `translation-done`, and `translation-error` Tauri events are the public API. Their payloads and semantics must not change across phases; only new events may be added.
- **All network I/O is async.** No blocking calls on the Tokio thread pool. All timeouts and retries must use `tokio::time` primitives.
- **Config drives behavior, not code.** API endpoint, model list, temperature, and retry policy must be readable from `AppConfig` rather than hardcoded.
- **Cancellation is cooperative.** Stream cancellation must be signaled via a channel, not via process-level signals or panics, so state can be cleaned up cleanly.
- **Provider abstraction is additive.** Adding a new provider must not require changing the streaming core — only a new variant in a provider enum and a URL/auth mapping.
- **No silent data mutation.** Failed saves, parse errors, or dropped tokens must always surface via `translation-error`; never swallow and continue.

---

## Phase 1: MVP Stabilization — DONE

Status: **Done**

Goals:

- Ship a working end-to-end streaming translation flow from clipboard to UI.

Completed work:

- Implemented `translate_text` Tauri command in `src-tauri/src/translate.rs`.
- Integrated `reqwest` + `futures_util` SSE byte-stream parsing with a rolling line buffer.
- Defined `StreamChunk` / `StreamChoice` / `StreamDelta` deserialization structs.
- Wired `translation-chunk`, `translation-done`, `translation-error` event protocol.
- Connected auto-detect prompt branch (`source_lang == "auto"`).
- Registered command in `lib.rs` via `tauri::generate_handler!`.

---

## Phase 2: Connection Pool & Request Lifecycle — DONE

Status: **Done**

Goals:

- Eliminate per-request `Client` construction overhead.
- Enable in-flight cancellation when the user dismisses the popup.

Completed work:

- Created a shared `reqwest::Client` at app startup and stored it in `tauri::State<reqwest::Client>` in `lib.rs`.
- Injected the shared client into `translate_text` via `tauri::State` parameter; removed per-call `Client::new()`.
- Added a `CancellationRegistry` (`Arc<Mutex<HashMap<u64, tokio::sync::oneshot::Sender<()>>>>`) stored in `tauri::State`.
- Added a required `request_id: u64` parameter to `translate_text`; the frontend generates a monotonically incrementing ID per translation.
- Extracted `translate_stream()` as a separate internal function for clean registry cleanup.
- Wrapped the stream loop in a `tokio::select!` that races `stream.next()` against the cancellation `oneshot::Receiver`.
- On cancellation: emits `translation-done` (not `translation-error`) so the UI transitions to `result` state with partial text preserved.
- Added `cancel_translate(request_id)` Tauri command that looks up the sender in the registry and signals cancellation.
- Registered both `translate_text` and `cancel_translate` in `generate_handler!` in `lib.rs`.

### Cancellation Flow (Implemented)

```
Frontend: invoke('cancel_translate', { requestId })
  → Rust: CancellationRegistry.remove(id).send(())
  → translate_stream loop: tokio::select! { cancel => emit('translation-done'), break }
```

---

## Phase 3: Retry & Resilience — DONE

Status: **Done**

Goals:

- Recover from transient network errors without surfacing them to the user.

Completed work:

- Implemented exponential back-off retry loop in `translate_stream` with 3 attempts, delays of 200 ms, 600 ms, 1800 ms using `tokio::time::sleep`.
- HTTP 4xx errors (auth failure, bad request) are surfaced immediately without retry.
- HTTP 5xx errors and `reqwest::Error::is_connect()` / `is_timeout()` are retried.
- Emits a `translation-retry { attempt: u8 }` event before each retry so the UI can show a "retrying…" indicator.
- `max_retries` and `base_retry_ms` are currently hardcoded to `3` and `200`; exposing them in `AppConfig` is deferred to a future config-expansion pass.

---

## Phase 4: Provider Abstraction & Self-Hosted Support — DONE

Status: **Done**

Goals:

- Support any OpenAI-compatible chat completions endpoint without code changes, enabling both cloud providers (DeepSeek, OpenRouter) and self-hosted open-source models (Ollama, llama.cpp, NLLB-serving) for fully offline, privacy-preserving translation.

Completed work:

- Added `Provider` enum (`DeepSeek`, `OpenRouter`, `Ollama`) to `config.rs` with `default_base_url()` and `default_models()`.
- Added `api_base_url: String` and `provider: Provider` fields to `AppConfig` with serde defaults for backward compatibility.
- `translate.rs` constructs the full URL as `format!("{}/chat/completions", api_base_url)`.
- Auth header format is provider-specific: `Bearer` for DeepSeek, `Bearer` + referer headers for OpenRouter, no auth for Ollama.
- Settings UI (`SettingsPanel.svelte`) exposes a provider dropdown and auto-populates `api_base_url` and `available_models` on provider change.
- `available_models: Vec<String>` is stored in `AppConfig` and drives the Settings model dropdown dynamically.

---

## Phase 5: Observability & Cost Tracking — NOT STARTED

Status: **Not Started**

Goals:

- Surface token usage and session statistics to the user.

Remaining features:

- Parse the final SSE chunk's `usage` field (`prompt_tokens`, `completion_tokens`, `total_tokens`) from the DeepSeek response.
- Define a `TranslationUsage { prompt_tokens, completion_tokens, model, duration_ms }` struct and emit it as a `translation-usage` Tauri event after `translation-done`.
- Store the last 50 `TranslationUsage` records in a rotating in-memory `VecDeque` in `tauri::State`.
- Add a `get_usage_history` Tauri command that returns the history to the frontend.
- Display a subtle cost indicator in the `TranslationPopup` footer (e.g., `~$0.0002 · 312 tokens`).

---

## Phase 6: Testing Strategy — NOT STARTED

Status: **Not Started**

Goals:

- Establish regression coverage for the SSE parsing and provider routing logic.

Remaining features:

- Add unit tests for the SSE line-buffer parsing logic (valid JSON, `[DONE]`, malformed lines, `\r\n` endings, multi-line payloads split across chunks).
- Add integration tests using `wiremock-rs` to stub the DeepSeek endpoint and verify event sequences.
- Add a test for cancellation: verify that cancelling mid-stream results in `translation-done` and not `translation-error`.
- Add a test for the retry logic: verify that 3 connection failures followed by success produces exactly 2 `translation-retry` events and 1 `translation-done`.

---

## Implementation Rules

- Do not create a new `reqwest::Client` per invocation — always use the shared state client (after Phase 2).
- Do not swallow parse errors in the SSE buffer — emit `translation-error` and return `Err`.
- Do not retry HTTP 4xx errors; they indicate a user-configuration problem that retry cannot solve.
- Do not change the payload type of `translation-chunk` (must remain `String`), `translation-done` (must remain `()`), or `translation-error` (must remain `String`).
- Do not block the Tauri main thread — all I/O must remain inside `async` commands.

## Open Questions

- **`\r\n` normalization scope:** Should the buffer normalize before splitting (replace all `\r\n` → `\n`), or handle `\r` as a trim character per line? The latter is cheaper but may miss edge cases. Decide before Phase 6 test authoring.
- **Token cost display:** Should cost estimation use a hardcoded per-token price table per model, or should the user configure the price? A configurable table is more maintainable but adds UI surface. Decide before Phase 5.

## Resolved Questions

- **Should cancellation preserve partial text?** Resolved in Phase 2: Yes. `translation-done` is emitted on cancel so the UI shows whatever text arrived before cancellation. The frontend transitions to `result` state if partial text exists, or `idle` if no text arrived.
