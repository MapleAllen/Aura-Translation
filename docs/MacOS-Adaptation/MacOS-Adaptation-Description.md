# macOS Adaptation Module Description

## Module Name

macOS Adaptation

## Purpose

The macOS Adaptation module defines how Aura Translation moves from a Windows-first desktop app toward a supported macOS build. It protects the first macOS phase by keeping Windows-only features disabled, adding a macOS CI build gate, and recording the manual desktop checks required before a macOS trial can be considered credible.

## Current Implementation

The current macOS adaptation is a build-and-behavior gate rather than a separate runtime subsystem. The Tauri backend exposes the desktop platform through `get_desktop_platform()`, and the Svelte Settings UI uses that value to disable Aura mode outside Windows. Source-app paste-back already reports unsupported on non-Windows platforms through backend status, and the translation popup now hides the paste-back action when that status is false.

The repository also has a dedicated macOS workflow in `.github/workflows/macos-adaptation.yml`. It runs the same core validation commands expected for first-phase macOS support on `macos-latest`: dependency install, Svelte diagnostics, UI tests, Rust tests, and Tauri bundle build.

To keep the first adaptation gate focused on runtime viability instead of distribution packaging, macOS now merges `src-tauri/tauri.macos.conf.json` during Tauri builds and limits the platform bundle target to `.app`. The shared `tauri.conf.json` still uses `"targets": "all"` for other platforms, so Windows packaging behavior remains unchanged.

macOS also overrides the legacy default hotkey. New configs now use `Cmd+Shift+J`, and existing macOS configs that still persisted either the old Windows-first default `CmdOrCtrl+T` or the earlier macOS fallback `Alt+Shift+T` are normalized in memory during load so the runtime converges on one easier-to-press manual translation shortcut.

### Capabilities

**macOS CI gate**
- Runs on GitHub Actions `macos-latest`.
- Executes `npm ci`, `npm run check`, `npm test`, `cargo test --manifest-path ./src-tauri/Cargo.toml`, and `npm run tauri build`.
- Produces a macOS `.app` bundle during adaptation instead of also requiring DMG packaging.
- Triggers on `main`, `codex/**`, pull requests, and manual dispatch.

**Platform capability reporting**
- `src-tauri/src/lib.rs::get_desktop_platform()` returns `std::env::consts::OS`.
- The command is registered in the Tauri invoke handler with the other desktop commands.
- `src-tauri/Cargo.toml` keeps the `windows` crate in the `target_os = "windows"` dependency block.
- `src-tauri/src/hotkey.rs::default_hotkey()` returns the platform-safe default accelerator.

**macOS UI fallback**
- `ui/lib/SettingsPanel.svelte` calls `get_desktop_platform` when loading settings.
- macOS and Linux normalize `config.aura_mode_enabled` to `false`.
- macOS and Linux disable the Aura mode switch and show copy explaining that manual hotkey translation is the supported path.
- `ui/lib/TranslationPopup.svelte` hides paste-back when `pasteBackSupported` is `false`.
- macOS startup now auto-opens the Settings window on first launch or whenever the runtime still needs setup, so manual testing does not depend on successfully discovering the tray first.

**Verification tracking**
- `docs/macOS-Adaptation-Checklist.md` defines the first manual runtime checklist for tray/menu bar behavior, global hotkeys, Keychain, notifications, transparent windows, and always-on-top behavior.

## Architecture

The module is a cross-cutting platform gate layered over the existing Daemon Core and UI Shell modules. It does not introduce a new runtime service; instead it narrows platform-specific dependency boundaries and routes platform behavior through backend-reported state.

### CI (`.github/workflows/`)

- `macos-adaptation.yml`
  - Defines the macOS build gate on `macos-latest`.
  - Uses Node 20, stable Rust, npm cache, and Rust cache.
  - Builds a Tauri macOS bundle after checks and tests pass.

### Backend (`src-tauri/`)

- `src-tauri/src/lib.rs`
  - `get_desktop_platform()`: returns the backend platform string used by Settings.
  - `paste_back_supported()`: returns `true` only on Windows and `false` elsewhere.
  - `paste_translation_back()`: rejects non-Windows calls without attempting platform automation.
  - Startup and fallback hotkey registration now use the platform default hotkey instead of hard-coding `CmdOrCtrl+T`.
  - macOS setup now auto-opens Settings on first launch or incomplete setup through `should_show_settings_on_startup()`.

- `src-tauri/Cargo.toml`
  - `tauri-plugin-global-shortcut` remains a desktop dependency.
  - `windows` is scoped to `target_os = "windows"`.
  - `keyring` uses `windows-native` on Windows and `apple-native` on macOS.

- `src-tauri/tauri.macos.conf.json`
  - Overrides `bundle.targets` to `["app"]` only on macOS.
  - Avoids coupling the adaptation gate to Finder-driven DMG decoration and packaging behavior.

### Frontend (`ui/lib/`)

- `SettingsPanel.svelte`
  - Loads `get_desktop_platform` alongside config, runtime status, history, and profiles.
  - Uses `supportsAuraModeOnPlatform()` to disable unsupported Aura mode controls.
  - Uses `configForCurrentPlatform()` to prevent unsupported config persistence.

- `TranslationPopup.svelte`
  - Uses `showPasteBackAction` to avoid rendering paste-back on unsupported platforms.

- `SettingsPanel.test.ts`
  - Covers macOS Aura mode disablement and save normalization.

- `TranslationPopup.test.ts`
  - Covers paste-back hiding when the platform does not support it.

- `TauriMacConfig.test.ts`
  - Guards the macOS-specific Tauri config so adaptation builds keep bundling `.app` only.

- `src-tauri/src/config.rs` and `src-tauri/src/hotkey.rs` tests
  - Guard the macOS default hotkey and the legacy-default migration path.

### Integration Points

- `src-tauri/src/lib.rs`
  - `get_desktop_platform`: invoked by Settings.
  - `get_paste_back_status`: indirectly controls whether the translation popup renders paste-back.

- `ui/lib/TranslationWindowView.svelte`
  - Passes `pasteBackStatus.supported` and `pasteBackStatus.available` into `TranslationPopup`.

- `docs/macOS-Adaptation-Checklist.md`
  - Tracks manual checks that cannot be validated from a Windows development environment.

## Current Limitations

- The macOS CI workflow has been added but has not been observed in this local Windows environment.
- The adaptation build currently stops at `.app` packaging on macOS; DMG generation is deferred until a later distribution-focused phase because Tauri's `create-dmg` step depends on Finder/`hdiutil` behavior that is not required for the runtime gate.
- The current host reproduced a real macOS shortcut conflict with the old `CmdOrCtrl+T` default in Finder before the hotkey migration landed.
- Manual runtime behavior still requires a real macOS desktop for tray/menu bar, global hotkey, Keychain, notifications, transparent windows, and always-on-top validation.
- Aura mode automatic clipboard monitoring remains Windows-only.
- Source-app paste-back remains Windows-only and has no macOS Accessibility/Automation permission flow.
- The frontend currently consumes a raw platform string instead of a richer structured capability payload.

## Future Directions

- Add a structured backend capability command for feature-specific availability.
- Support macOS-specific hotkey and notification diagnostics if Tauri permission behavior differs from Windows.
- Design a macOS Accessibility/Automation workflow before enabling source-app paste-back.
- Add a macOS trial release checklist once CI and manual runtime checks are proven.
- Add artifact upload to the macOS workflow when the build becomes a trial release gate.
