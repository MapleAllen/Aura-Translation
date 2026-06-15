# Config And Secrets Module Description

## Module Name

Config And Secrets

## Purpose

Config And Secrets is responsible for persisting all user preferences and managing provider API keys. It owns the `AppConfig` data model, the atomic JSON persistence strategy, and the secure credential store integration. The module ensures that API keys default to the OS system keychain on Windows and macOS, while offering an explicit plaintext fallback for environments where the keychain is unavailable. Any other backend module or frontend component that needs the current settings reads from the in-memory `ConfigState` that this module populates at startup.

## Current Implementation

At startup, `AppConfig::load_with_issues()` reads `{config_dir}/aura-translation/config.json`. If the file is absent, defaults are written and returned. If the file is present but malformed or unreadable, the loader returns a human-readable startup issue alongside the default `AppConfig`. `lib.rs` emits that issue through the shared `daemon-error` event path and also triggers a background notification when no Aura window is visible. The loaded config is then placed into a `ConfigState` (`Arc<RwLock<AppConfig>>`), so every Tauri command can access it with a read lock and the `save_config` command takes a write lock.

`AppConfig` uses a hand-written `Deserialize` implementation (via the `Helper` internal struct) instead of the default derive. This lets the deserializer apply provider-aware defaults: `api_base_url` and `available_models` are both seeded from the selected `Provider`'s built-in defaults when their JSON fields are absent. The `model` field falls back to the first available model rather than staying blank.

API key storage has three modes, captured in `ApiKeyStorage`:
- `System` (default) — new keys live in the OS keychain under the service name `"Aura Translation"` and a profile-scoped account key (e.g. `"profile:default:provider:deepseek"`); legacy provider-scoped entries remain readable; the `api_key` JSON field is omitted from `config.json`.
- `PlaintextFallback` — user explicitly chose to store the key in the JSON file; `api_key` is written to `config.json`.
- `LegacyPlaintext` — detected automatically when an existing `config.json` contains a non-empty `api_key` field without an `api_key_storage` field. On the next `save_config`, `secrets::migrate_legacy_plaintext_key` migrates it to `System` and sets the mode to `System`.

Saves are atomic: the config is serialised to `config.json.tmp` then renamed into `config.json`, preventing half-written files.

### Capabilities

**Data model**
- `Provider` enum: `DeepSeek | OpenRouter | Ollama` with provider-aware `default_base_url()`, `default_models()`, `requires_api_key()`, and `secret_account_name()` methods
- `AppConfig` fields: `api_key`, `api_key_storage`, `active_profile_id`, `model`, `source_lang`, `target_lang`, `hotkey`, `aura_mode_enabled`, `aura_guard_enabled`, `window_pinned`, `provider`, `api_base_url`, `available_models`, `settings_window_placement`, `pinned_translation_placement`
- `WindowPlacement` struct capturing `x`, `y`, `width`, `height`, `monitor` for both the settings window and the pinned translation window

**Loading and defaults**
- Reads `{config_dir}/aura-translation/config.json`; creates parent directory if absent
- Falls back to `AppConfig::default()` on missing or malformed files
- Default provider is DeepSeek; default hotkey is platform-dependent (`Cmd+Shift+J` on macOS, `CmdOrCtrl+T` on other platforms)
- Default `source_lang` is `"auto"`, default `target_lang` is `"Chinese"`, default `aura_guard_enabled` is `true`, default `aura_mode_enabled` is `false`
- Applies `hotkey::normalize_persisted_hotkey` on load to migrate macOS users off the legacy `CmdOrCtrl+T` and `Alt+Shift+T` defaults

**Persistence**
- `AppConfig::save()` serialises through `PersistedConfig` (a separate struct); `api_key` is omitted when `api_key_storage` is `System`
- Atomic write-then-rename via `config.json.tmp`

**Secret store (secrets.rs)**
- `hydrate_api_key()`: populates `config.api_key` from the keychain after load when storage is `System`
- `migrate_legacy_plaintext_key()`: detects `LegacyPlaintext` storage, moves the key to the keychain, and updates `api_key_storage` to `System`
- `persist_api_key()`: called during `save_config`; saves/clears/deletes the keychain entry or switches storage to `PlaintextFallback` depending on user intent; also deletes the old system key when the user switches from `System` to `PlaintextFallback` for the same provider
- `load_provider_api_key()`: resolves an optional profile-scoped key first, then falls back to the legacy provider-scoped entry
- `system_storage_supported()`: returns `true` on Windows and macOS; used to gate feature availability before attempting keychain operations
- Keychain operations use the `keyring` crate with `SERVICE_NAME = "Aura Translation"` and profile-scoped account names for new writes
- Test builds substitute a thread-local `HashMap`-backed mock store so unit tests do not touch the real OS keychain

**Frontend mirror**
- `ui/lib/appConfig.ts` mirrors `AppConfig` as TypeScript types (`AppConfig`, `Provider`, `ApiKeyStorage`, `WindowPlacement`)
- `createDefaultAppConfig()` constructs a client-side default matching the Rust defaults
- `cloneAppConfig()` performs a deep clone (required because `available_models` and `WindowPlacement` are reference types)

## Architecture

Service-layer pattern. The Rust `config.rs` module owns the data model and on-disk I/O. The `secrets.rs` module owns all keychain interactions. `lib.rs` wraps the loaded `AppConfig` in `ConfigState` and exposes `get_config` / `save_config` Tauri commands. The TypeScript side holds a mirrored type definition and pure utility functions.

### Rust Backend (`src-tauri/src/`)

- `config.rs`
  - `Provider`: serialised as `"deepseek" | "openrouter" | "ollama"` (lowercase via `serde(rename_all = "lowercase")`)
    - `default_base_url()`: canonical API root for each provider
    - `default_models()`: ordered list of model IDs shown in Settings dropdown
    - `requires_api_key()`: returns `false` only for Ollama
    - `secret_account_name()`: keychain account name, e.g. `"provider:deepseek"`
  - `ApiKeyStorage`: serialised as `"system" | "plaintext_fallback" | "legacy_plaintext"`
  - `WindowPlacement`: optional `width`, `height`, `monitor` fields; stored as `f64` coordinates
  - `AppConfig`: derives `Serialize`; implements `Deserialize` manually via `Helper` struct for provider-aware defaults
    - `AppConfig::load_with_issues()`: reads disk or returns defaults plus startup issues; calls `hotkey::normalize_persisted_hotkey`
    - `AppConfig::save()`: serialises through `PersistedConfig`, omitting `api_key` for `System` storage; writes via `config.json.tmp` rename

- `secrets.rs`
  - `SERVICE_NAME`: `"Aura Translation"` — the keychain service identifier
  - `system_storage_supported()`: compile-time check for Windows/macOS
  - `hydrate_api_key(config: &mut AppConfig)`: fills `config.api_key` from keychain; no-op for Ollama
  - `migrate_legacy_plaintext_key(config: &mut AppConfig)`: one-time migration; returns `Ok(true)` if migration was performed
  - `persist_api_key(config: &mut AppConfig, old_config: &AppConfig)`: writes/deletes keychain entries; normalises `LegacyPlaintext` to `PlaintextFallback` on save
  - `load_provider_api_key(provider: &Provider, profile_id: Option<&str>)`: profile-scoped read with legacy provider fallback

### Frontend (`ui/lib/`)

- `appConfig.ts`
  - `Provider`: TypeScript union `'deepseek' | 'openrouter' | 'ollama'`
  - `ApiKeyStorage`: TypeScript union `'system' | 'plaintext_fallback' | 'legacy_plaintext'`
  - `WindowPlacement`: mirrored struct with nullable `width`, `height`, `monitor`
  - `AppConfig`: full mirrored type matching the Rust `AppConfig` serialisation
  - `createDefaultAppConfig()`: constructs a default matching Rust `AppConfig::default()`
  - `cloneAppConfig(config)`: deep-clones including `available_models` array and placement objects

### Integration Points

- `src-tauri/src/lib.rs`
  - `ConfigState` (`Arc<RwLock<AppConfig>>`): managed Tauri state holding the in-memory config
  - Startup load issues from `AppConfig::load_with_issues()` are emitted through `emit_daemon_error()` during `setup()`
  - `get_config()`: returns the current in-memory `AppConfig` (no disk I/O)
  - `save_config(config: AppConfig)`: calls `secrets::persist_api_key`, `secrets::migrate_legacy_plaintext_key`, then `AppConfig::save()`; re-registers the hotkey if the `hotkey` field changed; applies `window_pinned` to the live Tauri window
  - `load_provider_api_key(provider, profile_id)`: calls `secrets::load_provider_api_key`; exposed as a Tauri command

- `src-tauri/src/profiles.rs`
  - `TranslationProfile::from_config()`: snapshots the provider/model/language/key fields from `AppConfig`
  - `TranslationProfile::apply_to_config()`: writes profile fields back into `AppConfig` on profile switch

- `src-tauri/src/readiness.rs`
  - `is_translation_ready(config)`: inspects `api_key`, `api_base_url`, and `model` fields from `AppConfig`
  - `should_prompt_for_setup(config)`: inverse of `is_translation_ready`

- `ui/lib/SettingsPanel.svelte`
  - Reads `AppConfig` via `invoke('get_config')` and writes it via `invoke('save_config', { config })`

## Current Limitations

- **Startup config failures surface after Tauri setup begins**: load issues are now emitted through `daemon-error`, but if no Aura window is visible the user may first see an OS notification before opening Settings.
- **No config schema version field**: there is no `schema_version` or `config_version` field; future breaking schema changes must be handled through careful defaults-based migration.
- **`WindowPlacement` is stored but not validated**: coordinates are written from whatever the window reports at close time; no bounds checking against current monitor geometry on restore.
- **Legacy plaintext migration is one-shot on save**: if the user never opens Settings after updating from a version that used `LegacyPlaintext`, the migration is deferred until the next `save_config` call.
- **System storage is unsupported on Linux**: Linux builds fall back silently; the `persist_api_key` function returns an error in Chinese that may not surface to the user on non-macOS/Windows platforms.

## Future Directions

- Add a `schema_version` field to `config.json` to enable explicit migration paths.
- Validate and clamp `WindowPlacement` coordinates against the connected monitor geometry on restore.
- Support per-profile API key storage mode (currently all profiles share the same storage policy as the active config).
- Add Linux keyring support (e.g. via `libsecret`/`kwallet` backends already available in the `keyring` crate).
