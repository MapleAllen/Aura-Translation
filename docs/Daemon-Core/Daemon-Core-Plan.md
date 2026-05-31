# Daemon Core Plan

## Objective

Evolve the Daemon Core into a robust desktop service layer that can safely create its UI on demand, keep runtime preferences synchronized with the frontend, and remain resilient when hotkey registration, tray setup, or window creation fail. The target state is a daemon that stays lightweight, supports pinned always-on-top behavior, chooses the right monitor intelligently, and surfaces operational failures without requiring developer logs.

## Design Principles

- **Backend owns desktop semantics.** Window creation, placement, pinning, and hotkey lifecycle live in Rust, not in the web UI.
- **Frontend readiness must be explicit.** Lazily created windows must not receive events before the webview is ready to consume them.
- **Config is live state.** Saved preferences should update existing runtime objects immediately where feasible.
- **Error surfaces are structured.** User-relevant daemon failures should be emitted as named payloads, not only logged.
- **Fallbacks must be safe.** When hotkey registration fails, restore a known-good shortcut or surface a hard failure clearly.
- **Keep the idle footprint low.** New features should preserve the app's daemon-first resource profile.

---

## Phase 1: Tray, Hotkey, and Config Foundation - DONE

Status: **Done**

Goals:

- Ship a runnable tray daemon with config-backed startup behavior.

Completed work:

- Built the Tauri bootstrap in `lib.rs`.
- Added the tray menu with Settings and Quit.
- Added a tray left-click primary action that opens Settings when Aura is not ready and recalls the latest translation bubble otherwise.
- Added startup config loading and persistence.
- Registered translation commands and shared backend state.
- Added blur-event forwarding to the frontend.

---

## Phase 2: Dynamic Hotkey Registration - DONE

Status: **Done**

Goals:

- Make hotkeys configurable and safe to update at runtime.

Completed work:

- Implemented `parse_hotkey()` for Electron-style accelerators.
- Registered startup hotkeys from `AppConfig.hotkey`.
- Re-registered hotkeys in `save_config`.
- Added fallback registration to `CmdOrCtrl+T`.
- Added `hotkey-conflict` and `hotkey-registered` events.
- Added hotkey parser tests.

---

## Phase 3: Lazy Main Window & UI Readiness Handshake - DONE

Status: **Done**

Goals:

- Avoid startup window cost while keeping event delivery reliable.

Completed work:

- Switched the main window to `create: false`.
- Added `ensure_main_window()` using `WebviewWindowBuilder::from_config`.
- Added `UiReadyState` and the `mark_ui_ready` Tauri command.
- Added `wait_for_ui_ready()` before emitting `trigger-translate` and `show-settings`.
- Attached blur listeners to lazily created windows.

---

## Phase 4: Live Window Preferences - DONE

Status: **Done**

Goals:

- Let backend window behavior track frontend preference changes without restart.

Completed work:

- Added `window_pinned` to `AppConfig`.
- Applied pin state to existing windows with `set_always_on_top`.
- Synced live window preferences after successful config saves.
- Supported shared shell behavior for transient and pinned usage modes.

---

## Phase 5: Error Surface & Save Safety - PARTIAL

Status: **Partial**

Goals:

- Make daemon failures diagnosable without corrupting runtime state.

Completed work:

- Added atomic config save via temp file + rename.
- Added structured `daemon-error` emissions for tray, window, readiness, restore, and pinning failures.
- Restored previous hotkeys when re-registration fails.

Remaining features:

- Route startup config read/parse failures through `daemon-error` instead of `eprintln!`.
- Add persistent daemon lifecycle logging.
- Decide whether non-recoverable backend errors should ever force app exit instead of surfacing in-UI.

---

## Phase 6: Monitor & Tray-Aware Positioning - NOT STARTED

Status: **Not Started**

Goals:

- Open the window near the user's active desktop context instead of a generic fallback position.

Remaining features:

- Detect the monitor containing the cursor when the hotkey fires.
- Distinguish taskbar edge and work-area geometry.
- Add user-configurable offsets in `AppConfig`.
- Add regression coverage for placement math across common monitor layouts.

---

## Phase 7: Secret Storage - DONE

Status: **Done**

Goals:

- Move provider credentials out of plaintext config when practical.

Completed work:

- Added `api_key_storage` to `AppConfig` and stopped persisting API keys into `config.json` while system storage mode is active.
- Added a dedicated `secrets.rs` layer for provider-scoped secret load/save/delete operations.
- Migrated legacy plaintext API keys into the system credential store on first startup when supported.
- Added `load_provider_api_key` so Settings can restore a provider-specific key without reusing the previous provider's value.
- Preserved an explicit plaintext fallback mode for unsupported builds or deliberate local fallback.

---

## Phase 8: Named Translation Profiles - DONE

Status: **Done**

Goals:

- Let Aura keep multiple provider/model/language presets and switch them from both Settings and the tray.

Completed work:

- Added `profiles.rs` with a dedicated `profiles.json` store and active-profile synchronization.
- Added profile CRUD commands for the Settings window.
- Added a tray `Profiles` submenu with checked active-state switching.
- Kept hotkeys and window placement global while profile state remains translation-scoped.

---

## Phase 9: Source App Paste-Back - DONE

Status: **Done**

Goals:

- Let the translation bubble return translated text directly to the original Windows app without leaving Aura copied onto the clipboard.

Completed work:

- Captured the foreground source window before showing the translation bubble.
- Added Windows paste-back commands and bubble availability status.
- Reused clipboard suppression so temporary clipboard swaps do not retrigger Aura mode.
- Restored the previous text clipboard after paste-back when the previous clipboard content was text.

---

## Phase 10: Usage Visibility - DONE

Status: **Done**

Goals:

- Surface provider-reported token usage in the live bubble and recent history without coupling Aura to provider-specific pricing rules.

Completed work:

- Requested stream usage metadata from compatible providers.
- Added `translation-usage` event delivery to the frontend.
- Persisted usage snapshots on successful history entries for later inspection in Settings.
- Kept pricing out of scope so the UI shows usage only, not currency estimates.

---

## Phase 11: Testing Strategy - PARTIAL

Status: **Partial**

Goals:

- Keep daemon-level behavior stable as window lifecycle and config shape evolve.

Completed work:

- Added `hotkey.rs` parser coverage for valid combos, invalid modifiers, empty input, and unsupported keys.
- Added `config.rs` coverage for defaults, backward-compatible deserialization, and full round-trip serialization.

Remaining features:

- Add tests for `AppConfig::load()` failure paths and disk persistence.
- Add tests for window positioning math.
- Add tests for `save_config()` hotkey rollback behavior.
- Add tests or harnesses around the UI-ready wait path.

## Implementation Rules

- Do not emit frontend-facing events into a lazily created window before `mark_ui_ready` has completed.
- Do not change pin state in the frontend without persisting or reverting it.
- Do not silently drop hotkey-registration failures.
- Do not reintroduce eager window creation unless startup measurements justify it.
- Do not add monitor-placement heuristics to the frontend.

## Open Questions

- **Readiness signaling:** Is the polling-based `UiReadyState` sufficient, or should it become an event-driven handshake?
- **Secret storage fallback:** If keychain support increases binary size too much, is plaintext-with-warning an acceptable long-term compromise?
