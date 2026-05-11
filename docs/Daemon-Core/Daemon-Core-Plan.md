# Daemon Core Plan

## Objective

Evolve the Daemon Core from a hardcoded, single-hotkey, cleartext-config MVP into a fully configurable, secure, and multi-monitor-aware system service. The end state is a daemon that reads its hotkey binding from config at startup, stores secrets in the OS keychain, detects hotkey conflicts gracefully, anchors the popup to the active monitor and tray edge, and exposes a robust surface for future plugin-style feature extension — all without disrupting the existing frontend event contract.

Throughout every phase, two non-negotiable constraints must hold: (1) the application must remain deployable from a single Rust/Svelte codebase to **Windows, macOS, and Linux** without platform-specific forks, and (2) idle resource consumption must stay within **≤20 MB RAM / <10 MB installed binary** so the daemon imposes no background tax on the user's machine.

## Design Principles

- **Hotkey registration must be driven by config.** The `AppConfig.hotkey` field must be the single source of truth; no key combination may be hardcoded in `lib.rs` after Phase 2.
- **Secrets must never touch the filesystem in cleartext.** The API key must be read from and written to the OS keychain; `config.json` may store everything else.
- **User-visible errors must be emitted as Tauri events.** Daemon failures (hotkey conflict, keychain error, tray build failure) must reach the frontend as named events, not silent `eprintln!` calls or Rust panics.
- **Window positioning logic is owned by Rust.** The frontend must not calculate its own position; it receives a pre-computed `LogicalPosition` from the daemon.
- **Config save must be atomic.** Write to a `.tmp` file and rename-into-place to prevent corruption on crash.
- **The event contract is stable.** `trigger-translate`, `window-blur`, and `show-settings` event names and payload types must not change.
- **Resource budget is an invariant.** No phase may introduce a dependency that raises idle RSS above 20 MB or pushes the installed binary above 10 MB. Prefer native OS APIs and avoid bundling additional runtimes.

---

## Phase 1: MVP Hardening — DONE

Status: **Done**

Goals:

- Ship a working daemon with tray, hotkey, and config persistence.

Completed work:

- Implemented `run()` in `src-tauri/src/lib.rs` with builder-level plugin registration.
- Registered `Ctrl+T` (`CONTROL + Code::KeyT`) as a global shortcut.
- Implemented clipboard read, window positioning (440×360, bottom-right), and `trigger-translate` event emission.
- Built tray menu with Settings and Quit items; wired `show-settings` and `app.exit(0)`.
- Implemented `window-blur` forwarding from `WindowEvent::Focused(false)`.
- Implemented `AppConfig` in `config.rs` with `load()` / `save()` and defaults.
- Exposed `get_config` and `save_config` Tauri commands.

---

## Phase 2: Dynamic Hotkey Registration — NOT STARTED

Status: **Not Started**

Goals:

- Make the registered hotkey reflect the `AppConfig.hotkey` field, not a hardcoded key combination.

Remaining features:

- Parse `AppConfig.hotkey` string (format: `"CmdOrCtrl+T"`) into a `Shortcut` struct at startup and on config save.
- Implement a `parse_hotkey(s: &str) -> Result<Shortcut, String>` utility function covering `Ctrl`, `Alt`, `Shift`, `CmdOrCtrl` modifiers and all alpha/digit key codes.
- On startup: unregister any previously registered shortcut, then register the parsed shortcut.
- On `save_config`: re-register the new hotkey immediately without requiring a restart.
- Emit a `hotkey-registered { hotkey: String }` event on success.
- Emit a `hotkey-conflict { hotkey: String, error: String }` event on registration failure; keep the previous hotkey active.
- Update the Settings UI (`SettingsPanel.svelte`) to show the configurable hotkey input field as a live binding capture widget (listen for keydown, display modifier+key, store as `AppConfig.hotkey`).

### Hotkey String Format

Use Electron-compatible accelerator strings: `"CmdOrCtrl+T"`, `"Alt+Shift+T"`, etc. The `parse_hotkey` function must map these to `tauri_plugin_global_shortcut::Modifiers` and `Code` values.

---

## Phase 3: Secure API Key Storage — NOT STARTED

Status: **Not Started**

Goals:

- Move the API key out of plaintext `config.json` into the OS keychain.

Remaining features:

- Add `tauri-plugin-stronghold` or the `keyring` crate as a dependency.
- Define a new `get_api_key() -> Result<String, String>` Tauri command that reads from the OS keychain using service name `"aura-translation"` and account name `"deepseek-api-key"`.
- Define a new `save_api_key(key: String) -> Result<(), String>` Tauri command that writes to the keychain.
- Remove `api_key` from `AppConfig` and `config.json`; pass the API key directly from the frontend via `get_api_key()` before invoking `translate_text`.
- Update `SettingsPanel.svelte` to call `get_api_key` on open and `save_api_key` on save, separately from `save_config`.
- On first launch, if no keychain entry exists, return an empty string so the UI prompts the user to configure.

---

## Phase 4: Multi-Monitor & Tray Edge Awareness — NOT STARTED

Status: **Not Started**

Goals:

- Open the popup near the actual tray icon, regardless of monitor configuration or taskbar position.

Remaining features:

- Detect the monitor containing the cursor at hotkey time using `app.cursor_position()` and `app.available_monitors()`.
- Calculate the bottom-right anchor relative to the detected monitor's work area (excluding the taskbar).
- Detect taskbar edge (top/bottom/left/right) using the monitor's work area vs. full size delta.
- Anchor the popup to the correct edge: bottom-right for bottom taskbar, top-right for top taskbar, bottom-right (shifted) for left/right taskbar.
- Add `window_offset_x: i32` and `window_offset_y: i32` fields to `AppConfig` with defaults `16` and `60` for user-adjustable margins.

---

## Phase 5: Atomic Config Save & Error Surface — NOT STARTED

Status: **Not Started**

Goals:

- Harden config persistence and make daemon errors visible to the user.

Remaining features:

- Replace `fs::write` in `AppConfig::save` with a write-to-`.tmp`-then-rename pattern for crash-safe atomicity.
- Wrap all Tauri setup errors (tray build, shortcut registration) in a `DaemonError { code: String, message: String }` enum and emit as `daemon-error` events instead of using `?` (which panics on failure).
- Add a frontend listener for `daemon-error` in `+page.svelte` that surfaces the message as an error overlay.
- Log all daemon lifecycle events (startup, hotkey registration, config load/save, tray creation) to a rotating `aura-translation.log` file in the app data directory using the `tracing` crate.

---

## Phase 6: Testing Strategy — NOT STARTED

Status: **Not Started**

Goals:

- Establish regression coverage for config parsing, hotkey string parsing, and positioning logic.

Remaining features:

- Add unit tests for `parse_hotkey`: valid strings, unknown modifiers, unknown key codes, empty string.
- Add unit tests for `AppConfig::load`: missing file (should return defaults), malformed JSON (should return defaults), valid JSON with all fields, valid JSON with missing fields (partial forward-compatibility).
- Add unit tests for `AppConfig::save`: verify the output file matches the expected JSON schema.
- Add unit tests for window positioning: given a mock monitor size + scale factor + taskbar edge, verify the computed `LogicalPosition` is within the expected quadrant.

---

## Implementation Rules

- Do not hardcode any key combination in `lib.rs` after Phase 2 — all hotkey data must come from `AppConfig`.
- Do not store `api_key` in `config.json` after Phase 3 — only keychain storage is permitted.
- Do not use `eprintln!` for user-facing errors — emit a named Tauri event instead.
- Do not use `?` in the `setup` closure for non-fatal errors — emit `daemon-error` and return `Ok(())` to prevent a startup panic.
- Do not position the window using `primary_monitor()` after Phase 4 — always use the cursor-detected monitor.

## Open Questions

- **Hotkey capture UI:** Should the Settings panel use a `keydown` listener to capture hotkey combinations, or a text field with format validation? A capture widget is more user-friendly but requires preventing the hotkey from firing while the field is focused. Decide before Phase 2.
- **Keychain fallback:** If the OS keychain is unavailable (e.g., headless CI, sandboxed environment), should the app fall back to plaintext storage with a warning, or refuse to save? Decide before Phase 3.
- **Tray left-click behavior:** Should left-clicking the tray icon toggle the popup, open Settings, or do nothing? This affects Phase 5 event routing. Decide before Phase 5.
- **Log rotation policy:** Should the log file rotate daily, on size (e.g., 1 MB max), or both? Decide before Phase 5.
