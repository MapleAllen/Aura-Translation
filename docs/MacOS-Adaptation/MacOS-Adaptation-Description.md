# macOS Adaptation Module Description

## Module Name

macOS Adaptation

## Purpose

The macOS Adaptation module defines how Aura Translation reaches feature parity on macOS. It started as a build-and-behavior gate that disabled Windows-only features on macOS, added a macOS CI build gate, and recorded the manual checks required before a macOS trial could be considered credible. It now also covers the backend-driven capability model, native macOS clipboard monitoring for Aura mode, and Accessibility-authorized source-app paste-back.

## Current Implementation

The macOS adaptation has progressed beyond its initial build-and-behavior gate into a cross-platform capability model with native macOS runtime support. The Tauri backend exposes structured capabilities through `get_system_capabilities()`, returning per-feature status (`ready`, `needs_permission`, `unsupported`) instead of a raw platform string. The Svelte frontend consumes these capabilities to decide which features to enable, disable, or gate behind a permission prompt. Aura mode (automatic clipboard monitoring) now works on macOS via `NSPasteboard` change-count polling, and source-app paste-back works on macOS via `NSRunningApplication` process activation and `CGEvent` key injection, gated behind the macOS Accessibility permission (`AXIsProcessTrusted`).

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
- `src-tauri/src/capabilities.rs::get_system_capabilities()` returns `SystemCapabilities { aura_mode, paste_back }` where each field is a `FeatureCapability` (`Ready`, `NeedsPermission`, or `Unsupported`).
- On macOS, paste-back capability queries `AXIsProcessTrusted()` and returns `NeedsPermission` when Accessibility has not been granted.
- On Windows, both features return `Ready`. On unsupported platforms (Linux), both return `Unsupported`.
- `src-tauri/src/lib.rs::request_accessibility_permission()` triggers the macOS native Accessibility prompt via `AXIsProcessTrustedWithOptions`.
- `src-tauri/Cargo.toml` keeps the `windows` crate in the `target_os = "windows"` dependency block.
- `src-tauri/src/hotkey.rs::default_hotkey()` returns the platform-safe default accelerator.

**macOS native features**
- macOS Aura mode uses `NSPasteboard::generalPasteboard().changeCount()` polling inside `spawn_clipboard_monitor` to detect clipboard changes.
- macOS paste-back captures the frontmost application PID at trigger time, validates that the process is still running at paste-back time, focuses it via `NSRunningApplication::activateWithOptions`, and posts simulated `Cmd+V` via `CGEventCreateKeyboardEvent` to `HIDEventTap`.
- The translation window refreshes capabilities before and after paste-back to detect mid-session Accessibility authorization changes.

**Frontend capability consumption**
- `ui/lib/SettingsPanel.svelte` calls `get_system_capabilities` and disables Aura mode only when `capabilities.aura_mode` is `unsupported`.
- `ui/lib/TranslationPopup.svelte` hides paste-back when `capabilities.paste_back` is `unsupported`, disables it when `needs_permission`, and enables it when `ready`.
- `ui/lib/TranslationWindowView.svelte` holds separate reactive state for static capabilities (fetched once at mount, refreshed around paste-back) and dynamic `pasteBackStatus` (fetched per translation).
- macOS startup still auto-opens the Settings window on first launch or whenever the runtime still needs setup.

**Verification tracking**
- `docs/macOS-Adaptation-Checklist.md` defines the manual runtime checklist including Aura mode clipboard monitoring, paste-back with Accessibility permission flow, and all original Phase 1 checks.

## Architecture

The module is a cross-cutting platform gate layered over the existing Daemon Core and UI Shell modules. It does not introduce a new runtime service; instead it narrows platform-specific dependency boundaries and routes platform behavior through backend-reported state.

### CI (`.github/workflows/`)

- `macos-adaptation.yml`
  - Defines the macOS build gate on `macos-latest`.
  - Uses Node 20, stable Rust, npm cache, and Rust cache.
  - Builds a Tauri macOS bundle after checks and tests pass.

### Backend (`src-tauri/`)

- `src-tauri/src/capabilities.rs`
  - `get_system_capabilities()`: returns structured `SystemCapabilities` with per-feature `FeatureCapability` status.
  - macOS checks `AXIsProcessTrusted()` to determine paste-back permission state.

- `src-tauri/src/lib.rs`
  - `request_accessibility_permission()`: triggers the macOS native Accessibility prompt via `AXIsProcessTrustedWithOptions`.
  - `current_foreground_window_handle()`: returns the current foreground app PID on macOS (HWND on Windows).
  - `is_valid_paste_back_window()`: checks whether the target process is still running (macOS) or the HWND is still valid (Windows).
  - `focus_window_for_paste_back()`: focuses the target application via `NSRunningApplication` (macOS) or Win32 (Windows).
  - `send_cmd_v()`: posts simulated `Cmd+V` via CoreGraphics `CGEvent` (macOS) or Win32 input events (Windows).
  - `paste_translation_back()`: cross-platform paste-back orchestration using the above helpers.
  - `spawn_clipboard_monitor()`: macOS uses `NSPasteboard` change-count polling; Windows uses clipboard sequence number polling.
  - Startup and fallback hotkey registration use the platform default hotkey.
  - macOS setup auto-opens Settings on first launch or incomplete setup through `should_show_settings_on_startup()`.

- `src-tauri/Cargo.toml`
  - `tauri-plugin-global-shortcut` remains a desktop dependency.
  - `windows` is scoped to `target_os = "windows"`.
  - `keyring` uses `windows-native` on Windows and `apple-native` on macOS.
  - macOS-specific dependencies: `core-graphics`, `objc2`, `objc2-app-kit` (with `NSRunningApplication`, `NSWorkspace`, `NSPasteboard` features), `objc2-foundation`.

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
  - Holds separate reactive state for static `capabilities` (fetched at mount, refreshed around paste-back) and dynamic `pasteBackStatus` (fetched per translation).
  - Paste-back button visibility gates on `capabilities.paste_back !== 'unsupported'`; enablement gates on capability `ready` and runtime `available`.

- `TranslationPopup.svelte`
  - Receives `pasteBackCapability` and `pasteBackAvailable` as separate props.
  - Hides paste-back when capability is `unsupported`, shows permission tooltip when `needs_permission`.

- `SettingsPanel.test.ts`
  - Covers Aura mode enablement and save normalization based on capabilities.

- `TranslationPopup.test.ts`
  - Covers paste-back hiding and permission-state rendering.

- `TauriMacConfig.test.ts`
  - Guards the macOS-specific Tauri config so adaptation builds keep bundling `.app` only.

- `src-tauri/src/config.rs` and `src-tauri/src/hotkey.rs` tests
  - Guard the macOS default hotkey and the legacy-default migration path.

### Integration Points

- `src-tauri/src/capabilities.rs`
  - `get_system_capabilities`: invoked by Settings and TranslationWindowView at mount time.

- `src-tauri/src/lib.rs`
  - `request_accessibility_permission`: invoked by the paste-back button when capability is `needs_permission`.
  - `get_paste_back_status`: returns runtime availability of the paste-back target.

- `ui/lib/TranslationWindowView.svelte`
  - Passes `pasteBackCapability` (from static capabilities) and `pasteBackAvailable` (from runtime status) into `TranslationPopup`.

- `docs/macOS-Adaptation-Checklist.md`
  - Tracks manual checks that cannot be validated from a Windows development environment.

## Current Limitations

- The adaptation build stops at `.app` packaging on macOS; DMG generation is deferred until a later distribution-focused phase.
- Manual runtime behavior still requires a real macOS desktop for full validation of tray/menu bar, global hotkey, Keychain, notifications, transparent windows, and always-on-top behavior.
- macOS paste-back requires the user to grant Accessibility permissions; there is no silent fallback if the user declines.
- macOS paste-back targets applications (PID-level), not individual windows (unlike Windows HWND-level targeting).
- The `NSApplicationActivationOptions::ActivateIgnoringOtherApps` constant used for app focusing is deprecated on macOS 14+ but remains functional and is currently suppressed with `#[allow(deprecated)]`.
- Aura mode and paste-back are unsupported on Linux.

## Future Directions

- Replace `NSPasteboard` change-count polling with a native clipboard listener if lower-latency Aura mode behavior becomes necessary.
- Find a non-deprecated replacement for `ActivateIgnoringOtherApps` on macOS 14+ when one becomes available in AppKit bindings.
- Support macOS-specific hotkey and notification diagnostics if Tauri permission behavior differs from Windows.
- Add a macOS trial release checklist once CI and manual runtime checks are proven.
- Add artifact upload to the macOS workflow when the build becomes a trial release gate.
- Add Linux support for clipboard monitoring and paste-back.
