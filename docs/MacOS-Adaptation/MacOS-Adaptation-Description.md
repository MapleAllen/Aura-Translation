# macOS Adaptation Module Description

## Module Name

macOS Adaptation

## Purpose

The macOS Adaptation module defines how Aura Translation reaches feature parity on macOS. It started as a build-and-behavior gate that disabled Windows-only features on macOS, added a macOS CI build gate, and recorded the manual checks required before a macOS trial could be considered credible. It now also covers the backend-driven capability model and native macOS clipboard monitoring for Aura mode.

## Current Implementation

The macOS adaptation has progressed beyond its initial build-and-behavior gate into a cross-platform capability model with native macOS runtime support. The Tauri backend exposes structured capabilities through `get_system_capabilities()`, returning feature status (`ready` or `unsupported`) instead of a raw platform string. The Svelte frontend consumes these capabilities to decide which features to enable or disable. Aura mode (automatic clipboard monitoring) now works on macOS via `NSPasteboard` change-count polling. Source-app paste-back and its Accessibility permission prompt have been removed from both macOS and Windows paths.

The repository also has a dedicated macOS workflow in `.github/workflows/macos-adaptation.yml`. It runs the same core validation commands expected for first-phase macOS support on `macos-latest`: dependency install, Svelte diagnostics, UI tests, Rust tests, and Tauri bundle build.

To keep the first adaptation gate focused on runtime viability instead of distribution packaging, macOS now merges `src-tauri/tauri.macos.conf.json` during Tauri builds and limits the platform bundle target to `.app`. The shared `tauri.conf.json` still uses `"targets": "all"` for other platforms, so Windows packaging behavior remains unchanged.

macOS also overrides the legacy default hotkey. New configs now use `Cmd+Shift+J`, and existing macOS configs that still persisted either the old Windows-first default `CmdOrCtrl+T` or the earlier macOS fallback `Alt+Shift+T` are normalized in memory during load so the runtime converges on one easier-to-press manual translation shortcut.

### Capabilities

**macOS CI gate**
- Runs on GitHub Actions `macos-latest`.
- Executes `npm ci`, `npm run check`, `npm test`, `cargo test --manifest-path ./src-tauri/Cargo.toml`, and `npm run tauri build`.
- Produces a macOS `.app` bundle during adaptation instead of also requiring DMG packaging.
- Triggers on `main`, `codex/**`, pull requests, and manual dispatch.

**Structured capability reporting**
- `src-tauri/src/capabilities.rs::get_system_capabilities()` returns `SystemCapabilities { aura_mode }` where `aura_mode` is a `FeatureCapability` (`Ready` or `Unsupported`).
- On Windows and macOS, Aura mode returns `Ready`. On unsupported platforms (Linux), Aura mode returns `Unsupported`.
- `src-tauri/Cargo.toml` keeps the `windows` crate in the `target_os = "windows"` dependency block.
- `src-tauri/src/hotkey.rs::default_hotkey()` returns the platform-safe default accelerator.

**macOS native features**
- macOS Aura mode uses `NSPasteboard::generalPasteboard().changeCount()` polling inside `spawn_clipboard_monitor` to detect clipboard changes.
- Polling runs only while automatic translation is enabled. When it is off, the loop reads the configuration first and never calls into `NSPasteboard`, then clears the clipboard baseline so text copied while paused is not translated on resume.
- While disabled the loop idles at a 1 s configuration check, and an `Arc<tokio::sync::Notify>` notified by `persist_config_and_sync` makes a toggle take effect immediately rather than after the next period.

**Frontend capability consumption**
- `ui/lib/SettingsPanel.svelte` calls `get_system_capabilities` and disables Aura mode only when `capabilities.aura_mode` is `unsupported`.
- `ui/lib/TranslationPopup.svelte` result actions only expose copying the translated text.
- macOS startup auto-opens the Settings window while `setup_completed` is false. The flag is set by `complete_setup` only after a successful trial translation, so a configured user is not re-onboarded and a user who quit mid-setup is.

**Verification tracking**
- `docs/macOS-Adaptation-Checklist.md` defines the manual runtime checklist including Aura mode clipboard monitoring and all original Phase 1 checks.

## Architecture

The module is a cross-cutting platform gate layered over the existing Daemon Core and UI Shell modules. It does not introduce a new runtime service; instead it narrows platform-specific dependency boundaries and routes platform behavior through backend-reported state.

### CI (`.github/workflows/`)

- `macos-adaptation.yml`
  - Defines the macOS build gate on `macos-latest`.
  - Uses Node 20, stable Rust, npm cache, and Rust cache.
  - Builds a Tauri macOS bundle after checks and tests pass.

### Backend (`src-tauri/`)

- `src-tauri/src/capabilities.rs`
  - `get_system_capabilities()`: returns structured `SystemCapabilities` with Aura mode support status.

- `src-tauri/src/lib.rs`
  - `spawn_clipboard_monitor()`: macOS uses `NSPasteboard` change-count polling; Windows uses clipboard sequence number polling.
  - Startup and fallback hotkey registration use the platform default hotkey.
  - macOS setup auto-opens Settings while `setup_completed` is false, through `should_show_settings_on_startup()`.

- `src-tauri/Cargo.toml`
  - `tauri-plugin-global-shortcut` remains a desktop dependency.
  - `windows` is scoped to `target_os = "windows"`.
  - `keyring` uses `windows-native` on Windows and `apple-native` on macOS.
  - macOS-specific dependencies: `objc2-app-kit` with the `NSPasteboard` feature.

- `src-tauri/tauri.macos.conf.json`
  - Overrides `bundle.targets` to `["app"]` only on macOS.
  - Avoids coupling the adaptation gate to Finder-driven DMG decoration and packaging behavior.

### Frontend (`ui/lib/`)

- `capabilities.ts`
  - Defines the `SystemCapabilities` and `FeatureCapability` TypeScript types consumed by the frontend.

- `SettingsPanel.svelte`
  - Loads `get_system_capabilities` alongside config, runtime status, history, and profiles.
  - Uses the returned capability payload to disable or enable Aura mode controls.
  - Normalizes config before save for unsupported capabilities.

- `TranslationWindowView.svelte`
  - Owns translation window lifecycle, sizing, event listeners, copy action, and retry/cancel handling.

- `TranslationPopup.svelte`
  - Renders translation states and result actions without source-app paste-back controls.

- `SettingsPanel.test.ts`
  - Covers Aura mode enablement and save normalization based on capabilities.

- `TranslationPopup.test.ts`
  - Covers result actions without paste-back controls.

- `TauriMacConfig.test.ts`
  - Guards the macOS-specific Tauri config so adaptation builds keep bundling `.app` only.

- `src-tauri/src/config.rs` and `src-tauri/src/hotkey.rs` tests
  - Guard the macOS default hotkey and the legacy-default migration path.

### Integration Points

- `src-tauri/src/capabilities.rs`
  - `get_system_capabilities`: invoked by Settings at mount time.

- `docs/macOS-Adaptation-Checklist.md`
  - Tracks manual checks that cannot be validated from a Windows development environment.

**M1 acceptance tooling (`scripts/release/`)**
- `mock-translate-server.py`: request-counting OpenAI-compatible stub with proper HTTP/1.1 chunked SSE framing.
- `accept-interaction-macos.sh`: starts the stub, writes an isolated config, and prints the counter-based interaction table. It writes to `~/Library/Application Support/aura-translation/`, because `dirs::config_dir()` does not resolve to `~/.config` on macOS; writing to the wrong path leaves the app on its defaults and invalidates the run.
- `measure-resources-macos.sh`: reports CPU, memory, and whether any `NSPasteboard` frame appears in a `sample` capture. The gate is the frame count rather than a CPU threshold, because a CPU number cannot distinguish a fixed build from a broken one.
- `measure-startup-macos.sh`: parses `AURA_TRACE=1` records to report cold start and warm-recall percentiles.
- All four are POSIX shell or Python and are verified against the bash 3.2 that ships with macOS; the Windows helpers remain PowerShell.

## Current Limitations

- The adaptation build stops at `.app` packaging on macOS; DMG generation is deferred until a later distribution-focused phase.
- Manual runtime behavior still requires a real macOS desktop for full validation of tray/menu bar, global hotkey, Keychain, notifications, transparent windows, and always-on-top behavior.
- The M1 acceptance scripts have been exercised for startup and config handling (see `docs/macOS-Adaptation-Checklist.md`), but the interaction, resource, and latency rows still require a target-host run.
- Aura mode is unsupported on Linux.
- Intel Macs and the minimum supported macOS version remain unverified; both must be settled before any public distribution.

## Future Directions

- Replace `NSPasteboard` change-count polling with a native clipboard listener if lower-latency Aura mode behavior becomes necessary. macOS exposes no general-purpose pasteboard change notification today, which is why the enabled path still polls.
- Consider a per-platform design token set; the current radius, spacing, and focus rings are tuned for macOS only.
- Support macOS-specific hotkey and notification diagnostics if Tauri permission behavior differs from Windows.
- Add a macOS trial release checklist once CI and manual runtime checks are proven.
- Add artifact upload to the macOS workflow when the build becomes a trial release gate.
- Add Linux support for clipboard monitoring.
