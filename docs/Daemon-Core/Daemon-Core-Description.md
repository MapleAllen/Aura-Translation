# Daemon Core Module Description

## Module Name

Daemon Core

## Purpose

The Daemon Core is the system-level foundation of Aura Translation. It runs as an invisible background process, registering a configurable global hotkey and a system tray icon so the application has zero visible presence until the user explicitly triggers it. When the configured hotkey (default `CmdOrCtrl+T`) is pressed, the daemon reads the clipboard, positions the popup window near the system tray, makes it visible, and fires a `trigger-translate` event carrying the clipboard text. It also persists the user's API key and preferences to a local JSON config file so they survive restarts.

The core non-functional targets this module must preserve:

- **Resource budget:** ≤20 MB idle RAM and negligible CPU when waiting for a hotkey. Tauri achieves this by embedding the UI in the host OS's native WebView (WebView2 on Windows, WebKit on macOS/Linux) rather than bundling a Chromium instance. The latest verified Windows release build produced an approximately 11.9 MB executable and installers under 10 MB.
- **Cross-platform:** A single Rust codebase targets Windows, macOS, and Linux without platform-specific forks.
- **Zero taskbar footprint:** The window sets `skipTaskbar: true` and `visible: false` at launch; it surfaces only through the system tray and the global hotkey.

## Current Implementation

The entry point is `src-tauri/src/lib.rs`'s `pub fn run()`, called from `main.rs`. Before constructing the Tauri builder, the function creates three shared resources: a `reqwest::Client` for HTTP connection pooling across all translation requests, a `CancellationRegistry` (`Arc<Mutex<HashMap<u64, oneshot::Sender<()>>>>`) for tracking in-flight translation requests that can be cancelled, and a managed `ConfigState` loaded from disk at startup. These are registered via `.manage()` on the builder. The Tauri application builder then registers the clipboard manager, opener, and (on desktop targets) global shortcut plugins. The global shortcut handler is registered as a builder-level closure rather than in `setup`, which is required by Tauri 2's plugin initialization order.

On hotkey press, the handler reads the clipboard via `ClipboardExt::read_text()`, bails silently if the text is empty, then positions the `"main"` webview window at a calculated bottom-right offset (440×360 px, 16 px right margin, 60 px above the taskbar). The window is shown, focused, and a `trigger-translate` event carrying the raw clipboard text is emitted to all listeners.

In the `setup` closure, three tray menu items are built: **Settings**, a separator, and **Quit**. The tray icon uses the bundled app icon. Clicking **Settings** emits `show-settings` and brings the window to the foreground. Clicking **Quit** calls `app.exit(0)`.

A `window.on_window_event` listener watches for `WindowEvent::Focused(false)` and re-emits it as a `window-blur` event to the frontend, which triggers the dismiss animation.

A dedicated `hotkey.rs` module parses the `AppConfig.hotkey` string (e.g. `"CmdOrCtrl+T"`) into a Tauri `Shortcut` at startup and whenever the config is saved. The parser rejects bare single-key bindings and requires at least one modifier. If registration fails, a `hotkey-conflict` event is emitted to the frontend instead of silently failing.

Config persistence is handled by `config.rs`. `AppConfig` serializes to JSON and lives at `{config_dir}/aura-translation/config.json` (resolved via the `dirs` crate). `AppConfig::load()` deserializes once on startup; if the file is absent it writes defaults. The loaded config is then kept in managed `ConfigState`, so `get_config` returns the in-memory copy without disk I/O. `save_config` now behaves atomically from the user's perspective: if a new hotkey cannot be parsed or registered, no config changes are persisted; if disk persistence fails after hotkey re-registration, the previous hotkey is restored.

### Capabilities

**Global hotkey**
- Reads `AppConfig.hotkey` at startup and registers the parsed shortcut dynamically via `hotkey::parse_hotkey`
- Rejects bare single-key bindings; only modifier + letter/digit combinations are accepted
- On `save_config`, unregisters the old shortcut and registers the new one; emits `hotkey-registered` on success or `hotkey-conflict` on failure
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
- `AppConfig` struct with fields: `api_key: String`, `model: String`, `source_lang: String`, `target_lang: String`, `hotkey: String`, `provider: Provider`, `api_base_url: String`, `available_models: Vec<String>`
- Default values: `model = "deepseek-chat"`, `source_lang = "auto"`, `target_lang = "Chinese"`, `hotkey = "CmdOrCtrl+T"`, `provider = DeepSeek`
- Config file: `{OS config dir}/aura-translation/config.json` (e.g., `%APPDATA%\aura-translation\config.json` on Windows)
- Save is atomic (write to `.tmp` then `fs::rename` into place) to prevent corruption on crash
- Automatically creates missing directories on first write

**Tauri commands exposed**
- `get_config() -> AppConfig`: returns the current managed in-memory config
- `save_config(config: AppConfig) -> Result<(), String>`: re-registers the hotkey first, then persists the config only if the new binding is valid and active
- `translate_text(…)`: delegated to `translate::translate_text`; registered in `generate_handler!`
- `cancel_translate(request_id)`: delegated to `translate::cancel_translate`; cancels an in-flight translation by request ID

**Managed state**
- `reqwest::Client`: shared HTTP client created once at startup; reuses connection pools across all translation requests
- `CancellationRegistry`: `Arc<Mutex<HashMap<u64, oneshot::Sender<()>>>>` tracking in-flight translation requests that can be cancelled via `cancel_translate`

## Architecture

Single-file Tauri application bootstrap with companion config, hotkey, and translation service modules. The builder creates managed `tauri::State` resources (`reqwest::Client` for connection pooling, `CancellationRegistry` for request cancellation, and `ConfigState` for the current config), registers five Tauri commands, and delegates all translation logic to `translate.rs`. Tauri wraps the SvelteKit frontend in the host OS's native WebView (WebView2 on Windows, WebKit on macOS/Linux), avoiding the ~150 MB Chromium overhead of Electron.

### Rust Backend (`src-tauri/src/`)

- `lib.rs`
  - `run()`: creates shared `reqwest::Client`, `CancellationRegistry`, and `ConfigState`, registers them via `.manage()`, builds and runs the Tauri application; registers plugins, commands, tray, hotkey, and window events.
  - `get_config()`: Tauri command; returns the managed `ConfigState` copy.
  - `get_provider_defaults(provider)`: Tauri command; returns the provider's default base URL and model list.
  - `save_config(config)`: Tauri command; writes config atomically, updates `ConfigState`, and re-registers the hotkey if needed.
  - `translate::translate_text`: Tauri command; delegated to the translation module.
  - `translate::cancel_translate`: Tauri command; delegated to the translation module.
  - Global shortcut handler (closure): reads clipboard → positions window → shows window → emits `trigger-translate`.
  - Tray menu handler (closure): matches `"settings"` or `"quit"` event IDs.
  - Window event handler (closure): re-emits `window-blur` on `Focused(false)`.

- `config.rs`
  - `AppConfig`: serializable struct holding all user preferences.
  - `AppConfig::config_path() -> PathBuf`: resolves `{config_dir}/aura-translation/config.json`; creates missing directories.
  - `AppConfig::load() -> Self`: reads and deserializes the config file, or writes and returns defaults.
  - `AppConfig::save(&self) -> Result<(), String>`: serializes to pretty JSON, writes to `.tmp`, then atomically renames into place.

- `hotkey.rs`
  - `parse_hotkey(s: &str) -> Result<Shortcut, String>`: parses Electron-style accelerator strings into Tauri `Shortcut` values.
  - Unit-test coverage for valid combos, unknown modifiers, unsupported keys, and empty input.

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
  - Current grants include `clipboard-manager:allow-read-text`, `clipboard-manager:allow-write-text`, `global-shortcut:allow-register`, `global-shortcut:allow-unregister`, `global-shortcut:allow-is-registered`, `core:window:allow-show`, `core:window:allow-hide`, `core:window:allow-set-focus`, and `core:event:default`.

## Current Limitations

- **Position is static** — window is always placed at the bottom-right corner; there is no detection of whether the system tray is on a different edge (left, top).
- **Single monitor support** — uses `primary_monitor()` only; on multi-monitor setups the window always opens on the primary screen regardless of where the tray icon is.
- **API key stored in plaintext JSON** — the API key is stored in cleartext in `config.json`; no OS keychain integration. The config save itself is now atomic (write-then-rename).
- **Daemon lifecycle is not logged yet** — tray and hotkey errors are surfaced to the frontend, but there is no rotating log file for post-mortem diagnostics.
- **Focus-loss on Settings** — the `window-blur` handler in the frontend suppresses dismiss when `showSettings` is true, but the Rust side does not know the settings state; a race condition exists if blur fires during a settings transition.
- **`tauri_plugin_opener` registered but unused** — the opener plugin is initialized but no shell command or URL opening is currently performed from Rust.

## Future Directions

- Support system tray edge detection to anchor the popup window to the correct screen corner.
- Add multi-monitor support by finding the monitor containing the cursor rather than using `primary_monitor()`.
- Integrate OS keychain (Windows Credential Manager, macOS Keychain) for secure API key storage via `tauri-plugin-stronghold` or `keyring-rs`.
- Add a `tray-left-click` handler to show/hide the popup as an alternative to the hotkey.
- Support user-configurable window offset (right margin, bottom margin) in `AppConfig`.
