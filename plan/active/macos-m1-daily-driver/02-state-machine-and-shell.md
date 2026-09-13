# State Machine and Shell

Owner: implementation owner
Dependencies: `01-shared-contracts.md` frozen
Working branch: `main`

## Allowed Files

- `src-tauri/src/interaction.rs` (new)
- `src-tauri/src/lib.rs`
- `ui/lib/windowBehavior.ts`
- `ui/lib/windowBehavior.test.ts`
- `ui/lib/TranslationWindowView.svelte`

## Do Not Modify

- `translate.rs` request/streaming semantics (stage 2 is a separate slice)
- config schema (stage 3)
- component styling and theming (stage 4)

## Why the extraction comes first

The proposal's P0 acceptance method is "use a mock service counter to prove that recalling the
same text issues no new request". Today that rule lives inline in `lib.rs::handle_hotkey_pressed`
and is reachable only through a Tauri `AppHandle`, so it cannot be asserted in CI.

`lib.rs` currently has 3 unit tests, none covering the hotkey, monitoring, or menu decisions. The
extraction is therefore a prerequisite for the acceptance method, not a refactor for its own sake.
It follows the pattern already used in this repository: `should_show_settings_on_startup` and
`resolve_history_replay_api_key` are pure helpers tested from `lib.rs` without an `AppHandle`.

## Implementation Tasks

1. Create `src-tauri/src/interaction.rs` and register `mod interaction;` in `lib.rs`.
   - `RequestIdentity` plus `from_config`, deriving `PartialEq, Eq, Clone, Debug`.
   - `HotkeyContext`, `HotkeyOutcome`, `decide_hotkey`.
   - `MonitorAction`, `decide_clipboard_tick`.
   - Inline `#[cfg(test)] mod tests` covering every branch.
2. Replace `TranslationRuntimeState::last_requested_text` with `last_request_identity`.
   Update the two write sites (`trigger_translation`, `trigger_translation_with_override`) and the
   one read site (`handle_hotkey_pressed`).
3. Rewrite `handle_hotkey_pressed` to use `decide_hotkey`:
   - `TranslateNew` -> `trigger_translation`
   - `Collapse` -> hide the window, leave any in-flight request running silently
   - `Recall` -> `show_existing_translation_window`
   - `ShowEmptyState` -> show the window in an empty, directly editable state
   - `Unavailable` -> open the settings window
   Both automatic and manual modes go through this one path.
4. Add `config_changed: Arc<tokio::sync::Notify>` to `AppRuntimeState`, notify it at the end of
   `persist_config_and_sync`.
5. Rewrite the monitor loop so `aura_mode_enabled` is checked before any `NSPasteboard` call, and
   so the disabled branch resets `last_sequence` and `last_dispatched_text` and touches no
   pasteboard API.
6. Extend `build_tray_menu` with `open-translation` and `aura-mode-toggle`, keep existing ids, and
   set `show_menu_on_left_click(true)`. Add the matching `on_menu_event` handlers; route the
   toggle through the same persist path as `save_config` and gate it on
   `capabilities::get_system_capabilities().aura_mode`.
7. Add a window-level Escape handler to `TranslationWindowView.svelte` guarded by a new pure
   helper `shouldDismissOnEscape` in `windowBehavior.ts`.
8. Unbind the draft composer from the pinned state in `TranslationWindowView.svelte`.

## Behaviour Contract

| Action / state | M1 behaviour |
|---|---|
| New non-empty text, hotkey | Translate; cancel the previous request; a stale result must not overwrite a newer one |
| Same request, window visible | Collapse; no new request |
| Same request, window hidden | Recall the existing result; no new request |
| Same text, but language/provider/model/base URL/profile changed | Treat as a new request |
| Clipboard empty | Open an empty state that accepts direct input, with a short hint |
| Esc / close on the translation window | Hide it; un-pin if pinned; cancel the in-flight request and suppress its completion notification |
| Retranslate button | Explicitly allowed to issue a new request for the same text |
| Menu-bar icon click | Show the action menu: open translation, automatic-translation toggle, settings, quit |
| Automatic translation off | No clipboard reads and no automatic requests; the manual hotkey still works |
| Quit | End the process, release the hotkey, stop monitoring, drop in-flight requests |

Window blur keeps the request running but silent; an explicit close cancels it.

## Edge Cases

- In-flight request plus a same-text hotkey: collapse only, never cancel.
- Pinned plus Esc: un-pin, persist the placement, then hide in one action.
- IME composition active: Escape must not close the window.
- Re-enabling automatic translation must not translate text copied while it was off.
- A configuration change while the monitor is asleep must be observed promptly; the `Notify` wake
  path is the mechanism.
- Platforms where `aura_mode` is unsupported: the toggle menu item is disabled rather than hidden.

## Automated Verification

- `cargo test --manifest-path src-tauri/Cargo.toml` including the new `interaction` tests
- `npm test` including the new `windowBehavior` cases
- `npm run check`

## Manual Verification

- Repeat the hotkey on unchanged text in manual mode and confirm the window collapses instead of
  re-requesting; confirm the same via the tray menu.
- Toggle automatic translation from the menu bar and confirm the item state follows the
  configuration.
- With automatic translation off, copy text, re-enable, and confirm no translation is issued for
  the stale clipboard content.

## Completion Evidence

- `src-tauri/src/interaction.rs` added with 9 unit tests; registered as `mod interaction;` in `lib.rs`.
- `TranslationRuntimeState::last_requested_text` replaced by `last_request_identity`, written in
  `trigger_translation` and in the clipboard monitor, read by `handle_hotkey_pressed`.
- `handle_hotkey_pressed` rewritten around `interaction::decide_hotkey()`. Both modes share the path;
  `Collapse` and `Recall` cannot reach `trigger_translation`.
- `show_empty_translation_window` added so an empty clipboard opens an editable empty state in both
  modes, replacing the previous split between a stale recall and a `hotkey-empty-clipboard` error.
- `spawn_clipboard_monitor` restructured on macOS and Windows: configuration is read before any
  clipboard call, and the disabled branch resets `last_sequence` and `last_dispatched_text` without
  touching `NSPasteboard`. New `wait_for_monitor_tick` and `reset_clipboard_baseline` helpers.
- Monitor wake path: an `Arc<tokio::sync::Notify>` is managed as state and notified at the end of
  `persist_config_and_sync`, so `save_config`, profile activation, and the tray toggle all wake it.
- Tray menu now leads with `打开翻译` and a `自动翻译` check item (`TRAY_OPEN_TRANSLATION_ID`,
  `TRAY_AURA_MODE_ID`), keeps the existing `settings`, `quit`, and `profile:<id>` ids, and shows on
  left click. Right click retains the previous one-click recall. `toggle_aura_mode_from_tray`
  persists through `persist_config_and_sync`.
- `TranslationWindowView.svelte` gained a window-level Escape handler guarded by
  `shouldDismissOnEscape` (IME-safe) and an `Esc` path that un-pins before collapsing.
- The draft composer is no longer tied to pinning; `showComposer` follows whether there is source text.
- `ui/lib/windowBehavior.ts` gained `shouldRecordUserResize` and `shouldAutoFit` for the resize work.
- Automated: 75 Rust tests, 70 UI tests, `npm run check` clean.
