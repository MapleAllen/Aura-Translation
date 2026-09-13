# Verification Matrix

Backfilled as stages land. Every row names the artifact or command that produced it.

## Automated Checks

| Check | Status | Evidence |
|---|---|---|
| `npm run check` | PASS | 0 errors / 0 warnings |
| `npm test` | PASS | 10 files, 70 tests |
| `cargo test --manifest-path src-tauri/Cargo.toml` | PASS | 75 tests |
| `npm run build` | PASS | static output written to `build/` |
| `plan/active/macos-m1-daily-driver` interaction state machine unit tests | PASS | `src-tauri/src/interaction.rs`, 9 tests |
| macOS CI workflow on the final SHA | PENDING | |

## Behaviour Coverage

| Behaviour | Asserted by | Status |
|---|---|---|
| Same text plus visible window collapses without a request | `interaction::tests::same_text_with_visible_window_collapses_without_requesting` | PASS |
| Same text plus hidden window recalls without a request | `interaction::tests::same_text_with_hidden_window_recalls_without_requesting` | PASS |
| New text translates | `interaction::tests::new_text_translates` | PASS |
| Empty clipboard opens an editable empty state | `interaction::tests::empty_clipboard_opens_an_editable_empty_state` | PASS |
| Configuration change produces a different request identity | `interaction::tests::identity_covers_text_and_every_configuration_field` | PASS |
| Disabled monitor never dispatches | `interaction::tests::disabled_monitor_never_dispatches` | PASS |
| Re-enabling establishes a resume baseline before dispatching | `interaction` resume tests | PASS |
| A long pause still costs exactly one baseline sample | `interaction::tests::a_longer_pause_still_costs_exactly_one_baseline_sample` | PASS |
| Request identity reflects the dispatched request, not the intended one | `interaction::tests::request_identity_comes_from_the_dispatched_parameters` | PASS |
| Empty clipboard opens a real input box | `TranslationWindowView.test.ts` | PASS |
| Empty entry point survives a failed translation | `TranslationWindowView.test.ts` | PASS |
| Notification toggle marks the config unsaved | `SettingsPanel.test.ts` | PASS |
| No component uses an arbitrary color literal | `theme.test.ts` scans the whole category | PASS |
| `npm test` exits zero with no unhandled errors | verified; the run previously exited 1 | PASS |
| Escape collapses once and respects IME composition | `windowBehavior` tests | PASS |
| Silent provider fails instead of hanging | `translate::tests::silent_provider_fails_instead_of_hanging` | PASS |
| Transport failures carry their cause | `describe_transport_error`, exercised by the timeout test | PASS |
| Trial inputs validated before any request | `translate::tests::trial_inputs_are_validated_before_any_request` | PASS |
| Onboarding completes only after a successful trial translation | `SettingsPanel.test.ts` | PASS |
| A failed trial leaves setup incomplete and the wizard open | `SettingsPanel.test.ts` | PASS |
| Existing configuration without new fields is not re-onboarded | `config` unit tests | PASS |
| Background notification preference round-trips | `SettingsPanel.test.ts`, `config` round-trip test | PASS |
| Popup shows translation and copy as primary content | `TranslationPopup.test.ts` | PASS |
| Auto-fit does not fight a user-chosen window height | `windowBehavior` tests | PASS |
| Dark appearance declares every light token | `theme.test.ts` | PASS |
| No hardcoded light surface remains in components | `theme.test.ts` | PASS |
| Reduced-motion preference is honoured | `theme.test.ts` + `app.css` | PASS |
| Mock provider streams and counts requests | manual run of `mock-translate-server.py` | PASS |


## Manual macOS Checks

| Behaviour | Status | Evidence |
|---|---|---|
| Background resource and absence of clipboard frames while disabled | PENDING | |
| Same-text recall and Esc collapse against the mock server | PENDING | |
| Cold start and warm recall percentiles | PENDING | `measure-startup-macos.sh` not yet run on a host |
| Dark mode and system font rendering | PENDING | tokens and system font stacks are asserted in CI; visual confirmation needs a host |
| Fullscreen, dual display, unplug, pinned window | PENDING | |
| Fresh-config first translation within three minutes | PENDING | |
| Release bundle launches and the menu bar becomes operable | PASS | `AURA_TRACE=1` on the built `.app`, `tray-ready` at 113-251 ms |
| `setup_completed` is honoured by the shipped bundle | PASS | with the flag true, the trace shows no `window-show:settings`; with it absent, the settings window opens |
| Menu bar exposes open / automatic translation / settings / quit | PENDING | |
| Same-text recall and Esc collapse against the stub counter | PENDING | `accept-interaction-macos.sh` not yet run on a host |
| Five-day daily-driver trial | PENDING | self-reported evidence, never a CI gate |

## Known Limitations

- Dependency audit: 16 affected nodes with no available fix. Accepted for M1, tracked separately.
- The release `.app` could not be exercised end to end in this environment; target-host rows stay open
  until someone runs the acceptance scripts on a Mac desktop.
- The macOS config lives at `~/Library/Application Support/aura-translation/config.json`, not
  `~/.config/...`. This was confirmed by running the release bundle with an isolated `HOME`; the
  acceptance script was writing to the wrong path until the bundle test caught it.
- `cargo clippy` reports 8 warnings. They are pre-existing style lints (`too_many_arguments`, a
  single-variant tray match) and are not part of the CI gate; none were introduced as errors.
- Five-day trial and unread first-time-user rows are self-reported, not automated gates.
- Intel and minimum macOS version remain unverified.

## Regression Coverage

- `ui/lib/SettingsPanel.test.ts`
- `ui/lib/TranslationPopup.test.ts`
- `ui/lib/windowBehavior.test.ts`
- `ui/lib/HistoryList.test.ts`, `ui/lib/ProfileManager.test.ts`, `ui/lib/NotificationCenter.test.ts`,
  `ui/lib/RootPageRouting.test.ts`, `ui/lib/TauriMacConfig.test.ts`,
  `ui/lib/WindowCapabilities.test.ts`
- `src-tauri/src/interaction.rs`, `src-tauri/src/config.rs`, `src-tauri/src/translate.rs`
