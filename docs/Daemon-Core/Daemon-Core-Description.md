# Daemon Core Module Description

## Module Name

Daemon Core

## Purpose

The Daemon Core is the system-level foundation of Aura Translation. It runs as an invisible background process, registering a global hotkey and a system tray icon so the application has zero visible presence until the user explicitly triggers it. When `Ctrl+T` is pressed, the daemon reads the clipboard, positions the popup window near the system tray, makes it visible, and fires a `trigger-translate` event carrying the clipboard text. It also persists the user's API key and preferences to a local JSON config file so they survive restarts.

The core non-functional targets this module must preserve:

- **Resource budget:** ≤20 MB idle RAM and negligible CPU when waiting for a hotkey. Tauri achieves this by embedding the UI in the host OS's native WebView (WebView2 on Windows, WebKit on macOS/Linux) rather than bundling a Chromium instance, yielding a binary footprint under 10 MB.
- **Cross-platform:** A single Rust codebase targets Windows, macOS, and Linux without platform-specific forks.
- **Zero taskbar footprint:** The window sets `skipTaskbar: true` and `visible: false` at launch; it surfaces only through the system tray and the global hotkey.

## Current Implementation

The entry point is `src-tauri/src/lib.rs`'s `pub fn run()`, called from `main.rs`. The Tauri application builder registers four plugins at construction time: `tauri_plugin_clipboard_manager`, `tauri_plugin_opener`, and (on desktop targets) `tauri_plugin_global_shortcut`. The global shortcut handler is registered as a builder-level closure rather than in `setup`, which is required by Tauri 2's plugin initialization order.

On `Ctrl+T` press, the handler reads the clipboard via `ClipboardExt::read_text()`, bails silently if the text is empty, then positions the `"main"` webview window at a calculated bottom-right offset (440×360 px, 16 px right margin, 60 px above the taskbar). The window is shown, focused, and a `trigger-translate` event carrying the raw clipboard text is emitted to all listeners.

In the `setup` closure, three tray menu items are built: **Settings**, a separator, and **Quit**. The tray icon uses the bundled app icon. Clicking **Settings** emits `show-settings` and brings the window to the foreground. Clicking **Quit** calls `app.exit(0)`.

A `window.on_window_event` listener watches for `WindowEvent::Focused(false)` and re-emits it as a `window-blur` event to the frontend, which triggers the dismiss animation.

Config persistence is handled by `config.rs`. `AppConfig` serializes to JSON and lives at `{config_dir}/aura-translation/config.json` (resolved via the `dirs` crate). `AppConfig::load()` deserializes on startup; if the file is absent it writes defaults. `AppConfig::save()` serializes with pretty-printing. Two Tauri commands (`get_config`, `save_config`) expose these methods to the frontend.

### Capabilities

**Global hotkey**
- Registers `Ctrl+T` (with `CONTROL` modifier and `Code::KeyT`) as a global shortcut on all desktop targets
- Reads the system clipboard via `tauri_plugin_clipboard_manager` on each trigger
- Silently ignores hotkey events when the clipboard is empty or whitespace-only

**Window management**
- Calculates a bottom-right anchor position using primary monitor dimensions and the DPI scale factor
- Target position: `screen_w - 440 - 16` × `screen_h - 360 - 60` (logical pixels)
- Shows and focuses the `"main"` webview window on trigger; hides it after focus loss (via frontend dismiss logic)

**System tray**
- Icon sourced from `app.default_window_icon()` (the bundled app icon)
- Tooltip: `"Aura Translation"`
- Menu items: `Settings` (emits `show-settings`), separator, `Quit` (calls `app.exit(0)`)

**Focus-loss propagation**
- Listens for `WindowEvent::Focused(false)` and re-emits `window-blur` to the frontend
- Enables the frontend to trigger the dismiss animation and hide the window on focus loss

**Config persistence**
- `AppConfig` struct with fields: `api_key: String`, `model: String`, `source_lang: String`, `target_lang: String`, `hotkey: String`
- Default values: `model = "deepseek-v4-flash"`, `source_lang = "auto"`, `target_lang = "Chinese"`, `hotkey = "CmdOrCtrl+T"`
- Config file: `{OS config dir}/aura-translation/config.json` (e.g., `%APPDATA%\aura-translation\config.json` on Windows)
- Automatically creates missing directories on first write

**Tauri commands exposed**
- `get_config() -> AppConfig`: loads and returns the current config
- `save_config(config: AppConfig) -> Result<(), String>`: serializes and writes the config to disk

## Architecture

Single-file Tauri application bootstrap with a companion config module. No MVVM or service-layer abstraction — all Rust logic is flat within the builder and setup closures. Tauri wraps the SvelteKit frontend in the host OS's native WebView (WebView2 on Windows, WebKit on macOS/Linux), avoiding the ~150 MB Chromium overhead of Electron and keeping the installed binary under 10 MB.

### Rust Backend (`src-tauri/src/`)

- `lib.rs`
  - `run()`: builds and runs the Tauri application; registers plugins, commands, tray, hotkey, and window events.
  - `get_config()`: Tauri command; delegates to `AppConfig::load()`.
  - `save_config(config)`: Tauri command; delegates to `config.save()`.
  - Global shortcut handler (closure): reads clipboard → positions window → shows window → emits `trigger-translate`.
  - Tray menu handler (closure): matches `"settings"` or `"quit"` event IDs.
  - Window event handler (closure): re-emits `window-blur` on `Focused(false)`.

- `config.rs`
  - `AppConfig`: serializable struct holding all user preferences.
  - `AppConfig::config_path() -> PathBuf`: resolves `{config_dir}/aura-translation/config.json`; creates missing directories.
  - `AppConfig::load() -> Self`: reads and deserializes the config file, or writes and returns defaults.
  - `AppConfig::save(&self) -> Result<(), String>`: serializes to pretty JSON and writes atomically via `fs::write`.

- `main.rs`
  - Calls `aura_translation_lib::run()` — no logic of its own.

- `build.rs`
  - Standard Tauri build script (`tauri_build::build()`); generates the Tauri context.

### Window Configuration (`src-tauri/tauri.conf.json`)

- `width: 440`, `height: 360` — fixed, non-resizable
- `decorations: false` — frameless window
- `transparent: true` — required for glassmorphic background
- `alwaysOnTop: true` — floats above all other windows
- `skipTaskbar: true` — no taskbar entry
- `visible: false` — hidden at startup; shown only on hotkey trigger

### Integration Points

- `src-tauri/src/translate.rs`
  - `translate::translate_text`: registered in `generate_handler!` in `lib.rs`.

- `ui/routes/+page.svelte`
  - `listen('trigger-translate', …)`: receives the clipboard text payload.
  - `listen('window-blur', …)`: triggers the dismiss animation.
  - `listen('show-settings', …)`: opens the settings panel.
  - `invoke('get_config')`: loads config on mount and before each translation.
  - `invoke('save_config', { config })`: writes config from the Settings panel.

- `src-tauri/capabilities/` (Tauri permission grants)
  - Must include `clipboard:read-text`, `global-shortcut:all`, `core:window:allow-show`, `core:window:allow-hide`, `core:window:allow-set-position`, `core:window:allow-set-focus`, `core:event:allow-emit` for the daemon to function.

## Current Limitations

- **Hotkey is hardcoded** — `Ctrl+T` is registered in code (`Code::KeyT` with `Modifiers::CONTROL`). The `hotkey` field in `AppConfig` is stored and displayed in the UI but is **not** actually used to register the shortcut; changing it in Settings has no effect.
- **No hotkey conflict detection** — if `Ctrl+T` is already claimed by another application, registration silently fails with only an `eprintln!` log; no user-visible error is shown.
- **Position is static** — window is always placed at the bottom-right corner; there is no detection of whether the system tray is on a different edge (left, top).
- **Single monitor support** — uses `primary_monitor()` only; on multi-monitor setups the window always opens on the primary screen regardless of where the tray icon is.
- **Config written as plaintext JSON** — the API key is stored in cleartext in `config.json`; no OS keychain integration.
- **No graceful error surface for tray build failures** — tray icon construction errors propagate as Rust panics (`?` in setup closure).
- **Focus-loss on Settings** — the `window-blur` handler in the frontend suppresses dismiss when `showSettings` is true, but the Rust side does not know the settings state; a race condition exists if blur fires during a settings transition.
- **`tauri_plugin_opener` registered but unused** — the opener plugin is initialized but no shell command or URL opening is currently performed from Rust.

## Future Directions

- Replace hardcoded `Code::KeyT` hotkey registration with a dynamic registration derived from the `AppConfig.hotkey` field.
- Add a hotkey conflict detection path that emits a user-visible `hotkey-conflict` error event if `register()` fails.
- Support system tray edge detection to anchor the popup window to the correct screen corner.
- Add multi-monitor support by finding the monitor containing the cursor rather than using `primary_monitor()`.
- Integrate OS keychain (Windows Credential Manager, macOS Keychain) for secure API key storage via `tauri-plugin-stronghold` or `keyring-rs`.
- Replace `fs::write` with an atomic rename-into-place pattern to prevent config corruption on crash during save.
- Add a `tray-left-click` handler to show/hide the popup as an alternative to the hotkey.
- Support user-configurable window offset (right margin, bottom margin) in `AppConfig`.
