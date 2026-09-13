# Runtime Readiness Module Description

## Module Name

Runtime Readiness

## Purpose

Runtime Readiness evaluates whether Aura has enough configuration to translate and communicates that status to both the daemon and the frontend. It produces a structured checklist that the Settings panel renders to guide the user through first-time setup, and it provides a live provider probe command so users can verify their API key and endpoint before relying on Aura for real work.

## Current Implementation

`readiness.rs` exposes a set of pure functions that inspect `AppConfig` fields without any async I/O (except `probe_provider`, which is async). The module itself still has no managed state of its own, but `lib.rs` now wraps provider probing with a `ReadinessState` cache so repeated probe requests can reuse a recent result for 30 seconds when the provider, base URL, model, and hydrated API key have not changed.

`is_translation_ready(config)` returns `true` when `api_base_url` is non-empty, `model` is non-empty, and the provider's API key requirement is satisfied (`api_key` is non-empty or the provider is Ollama). `should_prompt_for_setup(config)` is its inverse.

`is_translation_ready` has two consumers:

- `handle_tray_primary_action()` routes a menu-bar "open translation" action to Settings when Aura still needs setup, instead of opening an empty translation window.
- `interaction::is_ready()` wraps it as the hotkey readiness gate, so an incomplete configuration routes to setup rather than failing a request.

Automatic startup opening of Settings is no longer driven by readiness. It keys on the explicit `setup_completed` flag, because readiness alone could not distinguish "never configured" from "configured then deliberately cleared the key", and window placement could not distinguish "finished setup" from "moved the window and quit".

`build_runtime_status(config)` constructs a `RuntimeStatus` value containing:
- `level`: `Ready` or `NeedsSetup`
- `summary`: a human-readable Chinese string describing the ready state or the first missing item
- `can_translate_now`: boolean shorthand for `level == Ready`
- `checklist`: four `RuntimeChecklistItem` records (provider, base_url, model, api_key), each with a `code`, a localised `label`, and an `ok` boolean

`probe_provider(client, config)` sends a minimal, non-streaming POST to `{api_base_url}/chat/completions` with `max_tokens: 4` and a `"ping"` message. It returns a `ProviderProbeResult` (`{ ok: bool, message: String }`). The function validates preconditions before making the network request: missing base URL, model, or API key all return `ok: false` without a network call. On HTTP errors, it maps 401→auth failure, 403→permission denied, 429→rate limit, and any other status to a raw body preview (truncated at 160 chars, multibyte-safe).

### Capabilities

**Readiness inspection (synchronous)**
- `is_translation_ready(config)`: checks base URL, model, and API key
- `should_prompt_for_setup(config)`: inverse of `is_translation_ready`
- `build_runtime_status(config)`: full structured status with checklist for the Settings UI
  - Checklist codes: `"provider"`, `"base_url"`, `"model"`, `"api_key"`
  - Checklist labels are in Chinese and reflect the active provider's name and the Ollama no-key exception

**Consumers in the M1 interaction contract**
- `interaction::is_ready()` exposes `is_translation_ready` as the hotkey readiness gate
- `complete_setup` persists the onboarding flag but does not itself check readiness; the trial
  translation is what proves the configuration works
- The first-run wizard reuses `probe_provider` for credential verification rather than adding a
  second connectivity check

**Provider probe (async)**
- `probe_provider(client, config)`: sends a minimal non-streaming completion request
  - Checks preconditions before making any network call
  - Returns structured `ok` + `message` result
  - Adds `HTTP-Referer` and `X-Title` headers for OpenRouter requests
  - Truncates raw error bodies at 160 characters (multibyte-safe via `chars()` iterator)

## Architecture

Stateless utility module. All functions are pure (except `probe_provider`'s HTTP call). No managed state. Called from `lib.rs` Tauri commands (`get_runtime_status`, `probe_provider_connection`) and from the startup readiness check.

### Rust Backend (`src-tauri/src/`)

- `readiness.rs`
  - `RuntimeStatusLevel`: `Ready | NeedsSetup` (serialised as `"ready" | "needs_setup"`)
  - `RuntimeChecklistItem`: `{ code: String, label: String, ok: bool }`
  - `RuntimeStatus`: `{ level, summary, can_translate_now, checklist }`
  - `ProviderProbeResult`: `{ ok: bool, message: String }`
  - `is_translation_ready(config: &AppConfig) -> bool`
  - `should_prompt_for_setup(config: &AppConfig) -> bool`
  - `build_runtime_status(config: &AppConfig) -> RuntimeStatus`
  - `probe_provider(client: &Client, config: &AppConfig) -> ProviderProbeResult` (async)
  - `build_chat_completions_url(api_base_url)`: appends `/chat/completions` to the base URL after trimming trailing slashes
  - `provider_headers(provider, api_key)`: returns `Authorization: Bearer {key}` for DeepSeek; adds `HTTP-Referer` and `X-Title` for OpenRouter; empty for Ollama
  - `preview_body(body, max_chars)`: multibyte-safe truncation via `chars().take(max_chars)`

### Integration Points

- `src-tauri/src/lib.rs`
  - `get_runtime_status()`: calls `readiness::build_runtime_status` with the current `ConfigState` and returns the result
  - `probe_provider()`: hydrates the current API key, checks `ReadinessState`, and only calls `readiness::probe_provider` when the cached result is absent or stale
  - `ReadinessState`: stores the last probe fingerprint, timestamp, and `ProviderProbeResult`; invalidated after successful config persistence
  - Tray setup and config-save flow: call `readiness::build_runtime_status(config)` to keep the tray tooltip summary aligned with the current readiness state
  - Startup: calls `readiness::should_prompt_for_setup(config)` to decide whether to open the settings window on first launch

- `ui/lib/SetupStatusCard.svelte`
  - Displays the `RuntimeStatus` checklist with OK/fail indicators per item
  - Reads `level` to decide whether to show the setup prompt or the ready badge

- `ui/lib/SettingsPanel.svelte`
  - Calls `invoke('get_runtime_status')` on mount and after settings save
  - Calls `invoke('probe_provider_connection')` when the user clicks the "Test connection" button
  - Passes `RuntimeStatus` to `SetupStatusCard`

## Current Limitations

- **Provider probe uses `max_tokens: 4`**: the probe request is minimal but still consumes API quota; there is no free "ping" endpoint for OpenAI-compatible providers.
- **No tray icon badge variants yet**: readiness now syncs to the tray tooltip, but the icon itself still does not switch between ready and warning states.
- **Checklist labels are hard-coded in Chinese**: internationalisation requires changing the Rust source.
- **No periodic background readiness check**: readiness is only evaluated on demand (at startup and after settings save), not continuously.

## Future Directions

- Add separate ready/warning tray icon variants so readiness is visible even before the tooltip opens.
- Internationalise checklist labels by moving them to the frontend rather than generating them in Rust.
- Add a `health_check` Tauri command that can be polled by the frontend to detect provider connectivity degradation during a session.
