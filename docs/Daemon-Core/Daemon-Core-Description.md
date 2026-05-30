# Daemon Core Module Description

## Module Name

Daemon Core

## Purpose

The Daemon Core is the system-facing runtime for Aura Translation. It owns startup, tray integration, global hotkey registration, lazy main-window creation, window positioning, config persistence, and the backend commands the frontend consumes. Its job is to keep the app effectively invisible until the user triggers translation or opens Settings, while still exposing enough control to support pinned-window comparison and provider-backed streaming translation.

## Current Implementation

The entry point is `src-tauri/src/lib.rs::run()`, called from `main.rs`. At startup it creates four managed state objects:

- a shared `reqwest::Client`
- a `CancellationRegistry`
- a `ConfigState` (`Arc<RwLock<AppConfig>>`)
- a `UiReadyState` (`Arc<RwLock<bool>>`)

The Tauri builder registers the clipboard manager plugin and, on desktop targets, the global shortcut plugin with a builder-level handler. That handler reads clipboard text on shortcut press and ignores blank content. Instead of assuming the main window already exists, the daemon calls `show_translation_window()` or `show_settings_window()`, both of which ensure the `main` webview exists, apply current window preferences, show and focus the window, wait up to 5 seconds for the frontend to call `mark_ui_ready`, and only then emit the corresponding frontend event.

Window creation is lazy because `tauri.conf.json` sets `"create": false` for the `main` window. `ensure_main_window()` uses `WebviewWindowBuilder::from_config()` to instantiate the configured window when first needed, then attaches a `Focused(false)` listener that re-emits `window-blur` to the frontend.

`AppConfig` is loaded once at startup and kept in managed memory. `get_config` returns that in-memory copy with no disk I/O. `save_config` persists the new config, re-registers the hotkey when required, updates the in-memory state, and synchronizes `window_pinned` onto any already-created main window through `set_always_on_top`.

### Capabilities

**Global hotkey**
- Reads the configured accelerator from `AppConfig.hotkey`
- Registers the startup hotkey via `hotkey::parse_hotkey`
- On trigger, reads the clipboard through `tauri_plugin_clipboard_manager`
- Ignores empty or whitespace-only clipboard content
- Falls back to `CmdOrCtrl+T` if parsing or registration fails at startup

**Lazy window lifecycle**
- Main window is configured with `create: false` and is built only when first needed
- `UiReadyState` prevents backend events from racing ahead of the frontend mount lifecycle
- `mark_ui_ready` is the frontend handshake that flips the ready flag
- `show_translation_window()` emits `trigger-translate` only after the UI is ready
- `show_settings_window()` emits `show-settings` only after the UI is ready

**Window behavior and positioning**
- Applies `window_pinned` to the live Tauri window with `set_always_on_top`
- Uses `current_monitor()` first, then `primary_monitor()` as fallback
- Positions the window inside the monitor work area using the actual window size and a roughly 18 px right/bottom inset
- Supports a resizable frameless window defined in `tauri.conf.json` (`640x460`, min `560x380`)

**System tray**
- Builds a tray icon from the default bundled app icon
- Tooltip: `Aura Translation`
- Menu actions:
  - `Settings`: opens the lazily created main window and emits `show-settings`
  - `Quit`: calls `app.exit(0)`

**Focus-loss propagation**
- Re-emits `WindowEvent::Focused(false)` as `window-blur`
- Lets the frontend decide whether blur should dismiss the shell based on settings visibility and pin state

**Config persistence**
- `AppConfig` fields: `api_key`, `model`, `source_lang`, `target_lang`, `hotkey`, `window_pinned`, `provider`, `api_base_url`, `available_models`
- Config path: `{config_dir}/aura-translation/config.json`
- Defaults are provider-aware through custom `Deserialize`
- Saves are atomic from the filesystem perspective: write to `config.json.tmp`, then rename into place

**Hotkey re-registration**
- If `hotkey` changes, `save_config` unregisters all current shortcuts, validates and registers the new one, then persists config
- If new hotkey registration fails, the previous hotkey is restored
- Emits `hotkey-conflict` on registration rejection
- Emits `hotkey-registered` after a successful hotkey save

**Daemon error surface**
- `emit_daemon_error()` pushes structured `daemon-error` events to the frontend
- Used for tray-build failures, window creation failures, UI-ready timeout, hotkey restore failures, and pin-application failures

**Exposed Tauri commands**
- `get_config() -> AppConfig`
- `get_provider_defaults(provider) -> ProviderDefaults`
- `mark_ui_ready()`
- `save_config(config: AppConfig) -> Result<(), String>`
- `translate_text(...)`
- `cancel_translate(request_id)`

## Architecture

Single Tauri application bootstrap in `lib.rs` with companion modules for config, hotkey parsing, and translation streaming.

### Rust Backend (`src-tauri/src/`)

- `lib.rs`
  - `run()`: builds the Tauri application, registers managed state, plugins, tray, commands, and startup hotkey
  - `ensure_main_window()`: lazily creates the main webview from `tauri.conf.json`
  - `wait_for_ui_ready()`: polls `UiReadyState` until the frontend reports readiness or timeout expires
  - `prepare_main_window()`: applies pinning preferences and positions the window
  - `show_translation_window()` / `show_settings_window()`: show the window and emit frontend events after readiness
  - `save_config()`: hotkey-safe config persistence and live pin-state sync

- `config.rs`
  - `Provider`: `DeepSeek | OpenRouter | Ollama`
  - `Provider::default_base_url()` / `default_models()`
  - `AppConfig`: provider-aware config model with custom deserialization defaults
  - `AppConfig::load()` / `save()`

- `hotkey.rs`
  - `parse_hotkey()`: parses Electron-style accelerator strings requiring at least one modifier plus an alphanumeric key

- `translate.rs`
  - Request streaming and cancellation backend used by the UI shell

- `main.rs`
  - Calls `aura_translation_lib::run()`

### Window Configuration (`src-tauri/tauri.conf.json`)

- `label: "main"`
- `width: 640`, `height: 460`
- `minWidth: 560`, `minHeight: 380`
- `decorations: false`
- `transparent: true`
- `alwaysOnTop: false` at config level; pinning is applied dynamically at runtime
- `skipTaskbar: true`
- `resizable: true`
- `create: false`
- `visible: false`

### Integration Points

- `ui/routes/+page.svelte`
  - `invoke('get_config')`
  - `invoke('save_config', { config })`
  - `invoke('mark_ui_ready')`
  - `listen('trigger-translate')`
  - `listen('show-settings')`
  - `listen('window-blur')`
  - `listen('hotkey-conflict')`
  - `listen('hotkey-registered')`
  - `listen('daemon-error')`

- `src-tauri/src/translate.rs`
  - `translate_text` and `cancel_translate` are registered in the same invoke handler and share the managed HTTP client and cancellation registry

- `src-tauri/capabilities/`
  - Grants include clipboard, global shortcut, window show/hide/focus, and core event permissions required by the shell

## Current Limitations

- **Monitor targeting is still window-centric**: positioning uses the current or primary monitor, not cursor location or tray-edge detection.
- **Config parse/read failures still fall back with `eprintln!`**: startup config load does not yet route those failures through `daemon-error`.
- **Tray interaction is menu-only**: there is no left-click toggle behavior on the tray icon.
- **No lifecycle log file**: daemon events surface to the UI but are not persisted to rotating logs.
- **UI-ready wait uses polling**: readiness is checked every 25 ms rather than through a one-shot event or condition variable.

## Future Directions

- Add cursor-aware multi-monitor positioning and taskbar-edge detection.
- Emit config load failures through the same structured daemon event pathway used elsewhere.
- Add optional tray left-click show/hide behavior.
- Add rotating daemon logs in the app data directory.
- Make window offsets user-configurable in `AppConfig`.
