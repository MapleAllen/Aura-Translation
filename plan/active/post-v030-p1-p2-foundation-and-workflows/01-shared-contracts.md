# Shared Contracts

## Ownership

Owner: Codex coordinator
Dependencies: `4fe5f24` documentation baseline

## Active Task Lock

Owner: none - P2 implementation lock released after Codex review
Task: none - awaiting pushed P2 handoff verification
Starting main SHA: n/a
Status: NO ACTIVE IMPLEMENTATION LOCK - P2 HANDOFF `19a1bad` RECORDED; CROSS-PLATFORM VERIFICATION PENDING

## Allowed Files

When an implementation lock is assigned for this plan, edits may be limited to:

- `src-tauri/src/config.rs`
- `src-tauri/src/secrets.rs`
- `src-tauri/src/profiles.rs`
- `src-tauri/src/history.rs`
- `src-tauri/src/readiness.rs`
- `src-tauri/src/translate.rs`
- `src-tauri/src/lib.rs`
- `ui/lib/appConfig.ts`
- `ui/lib/translationProfiles.ts`
- `ui/lib/translationHistory.ts`
- `ui/lib/notifications.ts`
- `ui/lib/SettingsPanel.svelte`
- `ui/lib/SettingsPanel.test.ts`
- `ui/lib/ProfileManager.svelte`
- `ui/lib/ProfileManager.test.ts`
- `ui/lib/HistoryList.svelte`
- `ui/lib/HistoryList.test.ts`
- `ui/lib/SetupStatusCard.svelte`
- `ui/lib/TranslationWindowView.svelte`
- `ui/lib/TranslationWindowView.test.ts`
- tray/icon assets only if required for readiness-badge parity
- touched module docs under `docs/`
- this plan folder and related ADR files under `plan/`

## Do Not Modify

- `src-tauri/src/aura_guard.rs`
- `src-tauri/src/hotkey.rs`
- macOS capability-model and paste-back code paths unless a plan amendment explicitly expands scope
- release/versioning metadata (`package.json`, Tauri version fields, release workflows) unless required by a reviewed follow-up lock
- unrelated completed plan archives

## Public Interfaces and Critical Functions

`src-tauri/src/lib.rs`

- `emit_daemon_error(app, code, message, recoverable)`
  - Responsibility: single structured backend-to-frontend error event path.
  - Compatibility: new P1 load-failure reporting must reuse this payload shape instead of inventing a second error channel.
- `save_config(app, state, profiles_state, config) -> Result<(), String>`
  - Responsibility: hotkey-safe config persistence entry point.
  - Compatibility: remains the only path that persists config and syncs live runtime side effects.
- `persist_config_and_sync(app, state, profiles_state, old_config, config) -> Result<(), String>`
  - Responsibility: persist secrets/config, sync active profile, refresh in-memory config, and emit updates.
  - Compatibility: any readiness-cache invalidation or tray refresh hook added for P1 must remain downstream of successful persistence only.

`src-tauri/src/config.rs`

- `Provider::secret_account_name() -> &'static str`
  - Current role: provider-scoped keychain account naming.
  - P2 impact: this cannot remain the only naming contract once profile-scoped keys are introduced; migration must preserve backward reads.
- `AppConfig::load() -> Self`
  - Responsibility: config disk load with provider-aware defaults.
  - Compatibility: fallback-to-default behavior stays; only reporting and migration hooks may change in P1.
- `AppConfig::save() -> Result<(), String>`
  - Responsibility: atomic persisted config write through `PersistedConfig`.
  - Compatibility: no direct plaintext key writes when `api_key_storage == system`.

`src-tauri/src/secrets.rs`

- `hydrate_api_key(config: &mut AppConfig) -> Result<(), String>`
- `migrate_legacy_plaintext_key(config: &mut AppConfig) -> Result<bool, String>`
- `persist_api_key(config: &mut AppConfig, old_config: &AppConfig) -> Result<(), String>`
- `load_provider_api_key(provider: &Provider) -> Result<String, String>`
  - P2 contract: profile-scoped secret resolution must be introduced without breaking existing provider-scoped lookups on first activation/save.

`src-tauri/src/profiles.rs`

- `TranslationProfile::from_config(...) -> TranslationProfile`
- `TranslationProfile::apply_to_config(&self, config: &mut AppConfig)`
- `TranslationProfilesStore::ensure_seeded_from_config(&mut self, config: &mut AppConfig) -> Result<(), String>`
- `TranslationProfilesStore::activate(&mut self, profile_id, config) -> Result<(), String>`
- `TranslationProfilesStore::sync_active_profile_from_config(&mut self, config) -> Result<(), String>`
  - P1/P2 contract: profile activation must continue to keep `AppConfig` and the active stored profile aligned.

`src-tauri/src/history.rs`

- `TranslationHistoryStore::load() -> Self`
- `TranslationHistoryStore::record_success(...) -> Result<(), String>`
- `TranslationHistoryStore::record_error(...) -> Result<(), String>`
- `TranslationHistoryStore::find(&self, entry_id) -> Option<TranslationHistoryEntry>`
  - P1/P2 contract: load failures become structured errors; insert ordering and persisted shape remain backward-compatible.
  - P2 contract amendment: `api_base_url` and optional `profile_id` are backward-compatible defaulted fields used only to make original-config retry faithful.

`src-tauri/src/readiness.rs`

- `build_runtime_status(config: &AppConfig) -> RuntimeStatus`
- `probe_provider(client: &Client, config: &AppConfig) -> ProviderProbeResult`
  - P1 contract: synchronous status helpers stay pure; probe cache wraps the async command boundary rather than mutating these helpers into stateful functions.

`src-tauri/src/translate.rs`

- Translation entry point that consumes provider/model/base URL inputs
  - P2 contract: if retry-with-original-provider requires override fields, the override must be explicit and one-shot; it must not mutate active-profile config as a side effect.
  - P2 contract amendment: history retry may carry a backward-compatible optional original profile ID so profile-scoped credentials can be resolved without mutating the active profile.

## Tasks

1. Freeze the shared `daemon-error` reporting contract for config/profile/history load failures.
2. Freeze the readiness-cache invalidation and tray-status update contract around successful config saves.
3. Freeze the profile-scoped secret naming and legacy-fallback migration contract.
4. Freeze the one-shot history retry override API so retries can use stored provider/model/base URL without switching the active profile.
5. Assign one implementation lock at a time for `P1`, then `P2`.

## Automated Verification

- `npm run check`
- `npm test`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- Windows CI gate on the final shared SHA
- macOS CI/build gate on the final shared SHA

## Completion Evidence

- This file names the final lock owner, starting SHA, allowed-file set, and handoff SHA for each completed stage.
- Any contract amendment is recorded here and, if architectural, in `plan/decisions/`.
- `P1A` completed local shared verification on macOS:
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `npm run check`
  - `npm test`
- `P1A` handoff SHA pushed to `origin/main`: `569d229`
- Windows verifier evidence was recorded on shared `main` by `cba33c9`; the verified runtime/source handoff remains `569d229`, and `cba33c9` is the current shared plan baseline.
- Current `P1A` implementation scope: `src-tauri/src/config.rs`, `src-tauri/src/profiles.rs`, `src-tauri/src/history.rs`, `src-tauri/src/lib.rs`, plus synchronized `docs/` and plan evidence files.
- Complete `P2` working-tree implementation started from `d141ac27d493c26fa272b9ba5855a8574ff9d91f`.
- Local `P2` shared verification passed on macOS:
  - `npm run check`
  - `npm test` (49 tests)
  - `cargo test --manifest-path src-tauri/Cargo.toml` with proxy variables cleared for localhost WireMock (57 tests)
  - `npm run tauri build` (macOS `.app` bundle produced)
- Current `P2` implementation scope: profile-scoped system credentials with legacy fallback, client-side history search/filter, backward-compatible retry metadata, and explicit one-shot original-config replay.
- P2 review fixed plaintext-fallback history replay so it resolves the original profile's stored plaintext key without activating that profile.
- Reviewed `P2` shared-source handoff commit: `19a1bad`.

## Deviations

- Windows verifier environment did not expose an inspectable tray shell handle, so tray/UI visibility rows remain a deferred target-host follow-up rather than a blocker for the next implementation lock.
- `TranslationWindowView.svelte` and its test surface were added to the allowed set because the explicit one-shot retry override cannot reach `translate_text` without the translation-window request boundary.
