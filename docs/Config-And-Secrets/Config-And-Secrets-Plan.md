# Config And Secrets Plan

## Objective

Config And Secrets will become a reliable, versioned, and cross-platform preferences layer. When all phases are complete, the module will handle schema migrations without data loss, surface all I/O errors through the structured event system, support Linux credential storage alongside Windows and macOS, and provide validated window placement restore so the app always opens within visible display bounds.

## Design Principles

- **Atomic writes always**: every config or secrets write must use the write-then-rename pattern; no partial-file state should ever be observable.
- **API keys never appear in JSON when system storage is active**: `config.json` must not contain plaintext API keys when `api_key_storage` is `system`.
- **Defaults are provider-aware**: when fields are absent, defaults must reflect the selected provider, not a hardcoded global default.
- **Migration is transparent**: users must not need to manually fix their config after an app update; all migrations run automatically on first load or save.
- **Error surfaces are consistent**: config I/O failures must route through the same `daemon-error` pathway used by other backend failures, not silently fall back.
- **Storage parity is a goal**: behaviour differences between Windows, macOS, and Linux must be minimised; any gap is a known limitation, not a design choice.

## Phase 1: Structured Error Routing — NOT STARTED

Status: **Not Started**

Goals:

- Replace `eprintln!` in `AppConfig::load()` and `TranslationProfilesStore::load()` with structured `daemon-error` events.
- Surface config parse and read failures to the frontend notification system so users see actionable messages instead of silent fallbacks.

Remaining features:

- Emit a `daemon-error` event when `AppConfig::load()` encounters a parse or read failure, including the path and error detail.
- Emit a `daemon-error` event when `TranslationProfilesStore::load()` encounters a parse or read failure.
- Ensure the daemon still starts with defaults after emitting the error (no change to the fallback behaviour, only the reporting).
- Add a frontend handler for `daemon-error` that displays a dismissible notification with the file path and a suggestion to check Settings.

## Phase 2: Schema Versioning and Migration — NOT STARTED

Status: **Not Started**

Goals:

- Add a `schema_version` field to `config.json` and `profiles.json`.
- Establish a migration registry so future schema changes can be applied incrementally and tracked.

Remaining features:

- Add `schema_version: u32` to `AppConfig` and `TranslationProfilesStore`; default to `1` on first write.
- Write a `migrate_config` function that upgrades older schemas to the current version before deserialisation.
- Emit a `daemon-error` if a config file reports a schema version higher than the current binary supports.
- Document the migration contract: every schema change increments the version and adds a corresponding migration function.

## Phase 3: Window Placement Validation — NOT STARTED

Status: **Not Started**

Goals:

- Validate restored `WindowPlacement` coordinates against the available monitor geometry before applying them.
- Prevent windows from appearing off-screen after monitor configuration changes.

Remaining features:

- On `prepare_main_window()`, query the current monitor list and clamp `settings_window_placement` and `pinned_translation_placement` coordinates to the union of all monitor work areas.
- Fall back to the default centre-of-primary-monitor position when the saved placement is entirely outside all visible monitors.
- Write the validated placement back to config on every clean close (no change to existing save-on-close logic, only add clamping on restore).

## Phase 4: Linux Credential Storage — NOT STARTED

Status: **Not Started**

Goals:

- Enable `system` API key storage on Linux using the `keyring` crate's `libsecret`/`kwallet` backend.
- Remove the Linux-specific silent fallback path.

Remaining features:

- Enable the `keyring` crate feature flags for Linux secret-service support.
- Update `system_storage_supported()` to return `true` on Linux when the secret service is available at runtime.
- Add a graceful degradation path: if the Linux secret service is unavailable at runtime, surface an error through `daemon-error` and offer `plaintext_fallback` as the alternative.
- Update the Settings panel to reflect Linux as a supported platform for system storage.

## Phase 5: Per-Profile API Key Storage Mode — NOT STARTED

Status: **Not Started**

Goals:

- Allow each translation profile to carry its own `api_key_storage` mode independently of the global config.
- Remove the current implicit coupling where all profiles share the storage policy of the last saved config.

Remaining features:

- Store `api_key_storage` as a first-class field in `TranslationProfile` (it already exists as a field; verify it is actually read and applied during `activate()`).
- Ensure `secrets::persist_api_key` is called with the correct old and new storage modes when a profile is activated.
- Add UI in the profile editor to let users select the storage mode per profile.
- Document the expected behaviour when two profiles for the same provider use different storage modes.

## Implementation Rules

- Do not write plaintext `api_key` to `config.json` when `api_key_storage` is `system`; the `PersistedConfig` helper struct enforces this — do not bypass it.
- Do not call `AppConfig::save()` directly from the profile activation path; always route through `save_config` in `lib.rs` so hotkey re-registration and pin sync are not skipped.
- Do not remove the `legacy_plaintext` variant from `ApiKeyStorage`; older config files may still contain it and the migration path must remain intact.
- Do not panic on keychain failures; all `keyring` operations return `Result` and must be propagated as `Err(String)`.

## Open Questions

- **What is the correct UX when config parse fails at startup?** Should the app proceed with defaults silently (current behaviour) or open Settings automatically and show a banner? The answer determines whether Phase 1 needs a new `daemon-error` sub-type or can reuse the existing one.
- **Should `schema_version` live inside the JSON object or be a separate sidecar file?** A sidecar avoids the deserialisation chicken-and-egg problem but adds file system complexity.
- **Is there a user-facing need for config export/import?** If yes, Phase 2 should include a stable serialisation contract; if not, schema versioning can remain internal.
