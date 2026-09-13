# First Run and Experience

Owner: implementation owner
Dependencies: `02-state-machine-and-shell.md` landed
Working branch: `main`

## Allowed Files

- `src-tauri/src/config.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/translate.rs`
- `ui/lib/appConfig.ts`
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

## Do Not Modify

- `aura_guard.rs`, `hotkey.rs`, `secrets.rs`, `profiles.rs`, `history.rs` behaviour
- released configuration compatibility: existing files must keep loading

## Implementation Tasks

### Runtime resilience (backend)

1. Build the managed client with a connection timeout and an overall request timeout instead of
   `Client::new()`.
2. Add a stream stall timeout in the consumption loop, reusing the existing error and
   notification path so cancellation and retry semantics are untouched.
3. Extend the wiremock suite with a delayed-response case that asserts a stall surfaces a
   recoverable error rather than hanging.

### Opt-in tracing

4. Add `trace_ui_event` and emit timestamped records to stderr only when `AURA_TRACE=1`. Emit at
   process start, after tray construction, around translation-window show, and on `mark_ui_ready`.

### Configuration

5. Add `setup_completed` and `notifications_enabled` to `AppConfig`, the deserialize helper, and
   `PersistedConfig`.
6. Infer `setup_completed` for existing installations from translation readiness or a saved
   settings placement, so nobody is re-onboarded.
7. Mirror both fields in `ui/lib/appConfig.ts`.

### First run

8. When `setup_completed` is false, the overview section renders three steps:
   choose service -> enter credential and verify -> run a trial translation. Provider base URL and
   model stay prefilled; custom URLs and model lists move under advanced.
9. Verification reuses the existing `probe_provider` path and its probe cache.
10. The trial translation issues one real `translate_text` call; success persists
    `setup_completed = true`.
11. Failure keeps the user in place with a readable message and a clear retry affordance, and never
    sets `setup_completed`.
12. On success, show the real hotkey from configuration plus a short note that the menu bar can
    open, pause, and quit.

### Settings convergence

13. Collapse navigation from six entries to general, translation service, and history, plus a
    collapsed advanced group containing profiles, model list, base URL, and token usage.
14. Keep the `data-testid="settings-nav-{id}"` naming so existing tests keep their anchors.

### Notifications

15. Gate background translation success/retry/failure notifications on `notifications_enabled`.
    Startup configuration-corruption notifications stay ungated.
16. Expose the setting in the general section with wording that explains it only affects
    background translation chatter.

### Popup hierarchy

17. Make the translation and the copy action the primary content. Move language direction,
    provider, model, and token usage into a collapsed detail area, and keep the source text
    collapsible.
18. Respect manual resizing: after the user resizes the window, stop auto-fitting to content until
    new text arrives.

## Edge Cases

- Ollama requires no key; verification must succeed without one.
- A failed trial translation must leave the configuration saved but not onboarded.
- Trial translation with an empty model list falls back to the provider default.
- Notification setting changes take effect without a restart.
- Long translations scroll inside the popup rather than growing the window without bound.

## Automated Verification

- `npm test` covering onboarding steps, navigation grouping, new config fields, and popup detail
  collapsing
- `cargo test --manifest-path src-tauri/Cargo.toml` covering the stall timeout and config
  defaults
- `npm run check`

## Manual Verification

- Fresh configuration with a working credential: complete first translation within three minutes
  and confirm the settings window does not reappear on the next launch.
- Upgrade path: keep an existing `config.json` without the new fields and confirm no onboarding is
  shown.

## Completion Evidence

- `translate::build_http_client()` replaces `Client::new()` with `connect_timeout` (10 s) and
  `timeout` (45 s). For streaming bodies this acts as an idle budget per read, so a healthy long
  stream keeps running while a silent provider fails instead of hanging.
- `describe_transport_error()` appends the cause chain, because `reqwest::Error`'s own `Display`
  stopped at "error sending request for url (...)" and hid timeouts, refused connections, and DNS
  failures from the user.
- Stream read failures now report "Stream stalled: no data for 45s" when the error is a timeout.
- `trace_ui_event` / `AURA_TRACE` added with `ProcessStart` captured at the top of `run()`. Events:
  `setup-start`, `tray-ready`, each `window-show:*`, and `ui-ready:<label>`. Silent and free when the
  variable is unset.
- `AppConfig` gained `setup_completed` and `notifications_enabled`, both optional in the deserialize
  helper and both written through `PersistedConfig`. Missing `setup_completed` is inferred from
  translation readiness or a saved settings placement; an explicit value always wins.
- `should_show_settings_on_startup` now keys on `setup_completed` instead of window placement, with
  tests covering first launch, a completed install, and a configured user who never finished setup.
- `complete_setup()` added as the only command that sets the flag; the frontend calls it only after a
  successful trial translation.
- `trial_translate()` added with a `CollectingSink` that reuses the existing streaming engine, and
  `validate_trial_inputs()` so a missing model or credential is reported before any request.
- `SetupStatusCard.svelte` renders the three steps with per-step state, the credential verification
  action, the trial action, and a success panel that names the real hotkey and the menu-bar entry.
- Settings navigation collapsed from six sections to General, Translation service, and History plus
  an advanced group holding Profiles; `data-testid="settings-nav-{id}"` naming preserved.
- A `后台翻译通知` toggle was added to General; `notify_translation_background` is gated on it while
  startup configuration notifications stay unconditional.
- The popup now leads with the translation and `复制译文`; language, provider, model, and token usage
  moved behind a `详情` toggle, and the source behind `查看原文`.
- Automated: 75 Rust tests, 70 UI tests, `npm run check` clean.
