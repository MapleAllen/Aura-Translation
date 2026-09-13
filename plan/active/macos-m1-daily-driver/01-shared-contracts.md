# Shared Contracts

## Ownership

Owner: implementation owner (single active lock)
Dependencies: `docs/macOS-M1-Proposal.md` P0/P1 scope, baseline `a6d1d168567bbb1a605fc3ddabafcbda76f9f20c`

## Active Task Lock

Owner: implementation owner
Task: M1 stages 0-5 as described in `00-overview.md`
Starting main SHA: `a6d1d168567bbb1a605fc3ddabafcbda76f9f20c`
Status: ACTIVE

## Allowed Files

- `src-tauri/src/lib.rs`
- `src-tauri/src/interaction.rs` (new)
- `src-tauri/src/config.rs`
- `src-tauri/src/translate.rs`
- `ui/lib/appConfig.ts`
- `ui/lib/windowBehavior.ts`
- `ui/lib/windowBehavior.test.ts`
- `ui/lib/notifications.ts`
- `ui/lib/SettingsPanel.svelte`
- `ui/lib/SettingsPanel.test.ts`
- `ui/lib/SetupStatusCard.svelte`
- `ui/lib/TranslationPopup.svelte`
- `ui/lib/TranslationPopup.test.ts`
- `ui/lib/TranslationWindowView.svelte`
- `ui/lib/NotificationCenter.svelte`
- `ui/lib/ProfileManager.svelte`
- `ui/lib/HistoryList.svelte`
- `ui/app.css`
- `scripts/release/*-macos.sh`, `scripts/release/mock-translate-server.py` (new)
- `README.md`
- `docs/macOS-M1-Proposal.md`, `docs/macOS-Adaptation-Checklist.md`, touched module docs
- this plan folder

## Do Not Modify

- `src-tauri/src/aura_guard.rs`, `src-tauri/src/hotkey.rs`, `src-tauri/src/secrets.rs`,
  `src-tauri/src/profiles.rs`, `src-tauri/src/history.rs`, `src-tauri/src/readiness.rs`
  unless a plan amendment explicitly expands scope
- `scripts/release/*.ps1` (Windows acceptance scripts stay as-is)
- release/versioning metadata (`package.json` version, Tauri version fields, release workflows)
- `.github/workflows/*` unless a new automated check genuinely requires wiring
- unrelated archived plans under `plan/completed/`

## Public Interfaces and Critical Functions

`src-tauri/src/interaction.rs` (new, pure, no Tauri types)

- `RequestIdentity { text, source_lang, target_lang, provider, model, api_base_url, profile_id }`
  - Responsibility: the complete identity of a translation request.
  - Contract: two requests are reusable only when every field is equal. This is what makes
    "changing translation settings must not reuse a stale result" enforceable.
- `RequestIdentity::from_config(text: &str, config: &AppConfig) -> RequestIdentity`
- `HotkeyContext { window_visible, clipboard_empty, ready, matches_last_request }`
- `HotkeyOutcome { TranslateNew, Collapse, Recall, ShowEmptyState, Unavailable }`
- `decide_hotkey(ctx: HotkeyContext) -> HotkeyOutcome`
  - Contract: the single decision point for both automatic and manual modes. `Collapse` and
    `Recall` must never lead to a network request.
- `MonitorAction { Dispatch, Skip }`
- `decide_clipboard_tick(suppressed, duplicate) -> MonitorAction`
  - Contract: only reached while automatic translation is running and past the resume baseline.
- `ClipboardRunState { Paused, ResumeBaseline, Running }`
- `ClipboardResumeState::tick(mode_enabled) -> ClipboardRunState`
  - Contract: the mode gate and the pause/resume transition live here rather than in the caller.
    The first enabled tick after a pause returns `ResumeBaseline`, which the caller must use to
    record the sequence and translate nothing, so text copied during the pause is never sent.

`src-tauri/src/lib.rs`

- `TranslationRuntimeState`
  - Change: `last_requested_text: Option<String>` becomes
    `last_request_identity: Option<RequestIdentity>`.
  - Compatibility: internal only; no persisted shape changes.
- `RuntimeState` / `AppRuntimeState`
  - Change: gains `config_changed: Arc<tokio::sync::Notify>`.
  - Compatibility: additive.
- `handle_hotkey_pressed(app)`
  - Change: builds a `HotkeyContext`, calls `decide_hotkey`, and dispatches on the outcome.
    Both modes share the path.
  - Compatibility: the manual-mode entry point and event names stay the same; only the decision
    logic moves.
- `spawn_clipboard_monitor(app)`
  - Change: the loop waits on `config_changed` or a poll period, checks `aura_mode_enabled`
    **before** touching `NSPasteboard`, and resets the clipboard baseline while disabled.
  - Compatibility: the enabled path keeps the existing 275ms cadence, Aura Guard screening,
    suppression handling, and duplicate suppression.
- `persist_config_and_sync(...)`
  - Change: notifies `config_changed` after a successful persist.
  - Compatibility: remains the single downstream sync point for `save_config` and
    `activate_profile_by_id`; it is only extended, not reordered.
- `build_tray_menu(app)` / `build_tray(app)`
  - Change: top level becomes open-translation, automatic-translation toggle (check item),
    profiles submenu, settings, quit. The menu now shows on left click.
  - Compatibility: existing ids `settings`, `quit`, `profile:<id>` are preserved so current
    handlers keep working.
- `trace_ui_event(app, event, t_ms)`
  - Responsibility: opt-in (`AURA_TRACE=1`) structured timing output to stderr so the proposal's
    latency and cold-start thresholds become measurable.
  - Contract: no output and no cost when the variable is unset; never user-visible.

`src-tauri/src/config.rs`

- `AppConfig`
  - Change: adds `setup_completed: bool` and `notifications_enabled: bool`.
  - Compatibility: both are optional in the deserialize helper. A missing `setup_completed` is
    inferred from `readiness::is_translation_ready(&config) || settings_window_placement.is_some()`
    so upgrading users are never re-onboarded. A missing `notifications_enabled` defaults to
    `true`, preserving current behaviour.
- `PersistedConfig`
  - Change: serializes both new fields.
  - Compatibility: existing files still load; writes stay atomic.

`src-tauri/src/translate.rs`

- Managed HTTP client construction moves to a builder with `connect_timeout` and `timeout`.
  - Compatibility: `translate_stream_with_sink` keeps taking `&Client`, so existing wiremock tests
    that pass `Client::new()` are unaffected.
- Stream consumption gains a stall timeout.
  - Compatibility: on stall it uses the existing `emit_error` plus `notify_background_error` path;
    cancellation and retry semantics are unchanged.
- `notify_translation_background(...)` gains a `notifications_enabled` guard.
  - Compatibility: startup configuration-corruption notifications in `lib.rs` are not gated, because
    they report a broken install rather than translation chatter.

`ui/lib/windowBehavior.ts`

- Adds `shouldDismissOnEscape(isComposing: boolean): boolean`.
  - Contract: returns false while an IME composition is active. Pinning deliberately does not
    participate: one Escape always collapses, and the caller un-pins as part of collapsing.
- Adds `shouldRecordUserResize(programmatic: boolean, windowPinned: boolean): boolean` and
  `shouldAutoFit(windowPinned: boolean, userResizedHeight: number | null): boolean`.
  - Contract: the auto-fit `setSize` must not be mistaken for a user resize, and a height the user
    chose must stop auto-fitting.

`ui/lib/theme.ts` (new)

- `LIGHT_THEME` / `DARK_THEME` palettes plus `FONT_STACKS`.
  - Contract: both appearances declare exactly the same token set, no token reuses its light value
    in dark mode, and both font stacks lead with `-apple-system`.
- `toSemanticSurfaceClasses(classNames: string): string`
  - Contract: replaces `bg-white*` literals with semantic tokens, longest match first.
- `prefersReducedMotion(): boolean`
  - Contract: safe outside a browser; true when macOS reports Reduce motion.

`ui/lib/appConfig.ts`

- `AppConfig` type gains `setup_completed` and `notifications_enabled`, and
  `createDefaultAppConfig()` mirrors the Rust defaults.

## Tasks

1. Freeze the request-identity contract so request reuse is keyed by the full configuration.
2. Freeze `decide_hotkey` as the single hotkey decision point for both modes.
3. Freeze `decide_clipboard_tick` so a disabled monitor cannot dispatch.
4. Freeze the `config_changed` notification contract on successful config persistence.
5. Freeze the tray menu id set and the added `open-translation` / `aura-mode-toggle` ids.
6. Freeze the two new config fields, including their backward-compatible defaults.

## Automated Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run check`
- `npm test`
- `npm run build`
- macOS CI workflow on the final shared SHA

## Completion Evidence

- Filled in as each stage lands; final state recorded in `05-verification.md`.

## Deviations

- Recorded here as they occur.
