# macOS Adaptation Plan

## Objective

Make Aura Translation a credible macOS desktop build with native cross-platform feature parity. The end state is a macOS build that passes CI, supports manual hotkey translation, persists provider secrets in Keychain, behaves correctly as a tray/menu bar daemon, and provides native macOS Aura-mode clipboard monitoring through a backend-driven capability model.

## Design Principles

- **Manual hotkey translation is the first macOS path.** Do not block macOS adaptation on automatic clipboard monitoring.
- **Unsupported controls must be hidden or disabled.** The UI should not offer Aura mode where the backend reports no support.
- **Backend reports platform truth.** The frontend should use backend-reported platform/capability state instead of guessing native behavior.
- **Windows behavior must remain intact.** macOS adaptation must not regress Windows Aura mode, installer, or trial workflow.
- **CI proves build viability; true desktop behavior needs a Mac.** Tray/menu bar, hotkey, Keychain, notifications, transparency, and always-on-top require real macOS runtime checks.
- **Platform dependencies stay scoped.** Windows-native crates and APIs must remain behind Windows target dependencies and `#[cfg(windows)]`.

## Phase 1: Build Gate And Unsupported Feature Guardrails - DONE

Status: **Done**

Goals:

- Add the first macOS build gate and prevent unsupported Windows-only features from appearing enabled on macOS.

Completed work:

- Added `.github/workflows/macos-adaptation.yml` with `npm ci`, `npm run check`, `npm test`, `cargo test --manifest-path ./src-tauri/Cargo.toml`, and `npm run tauri build` on `macos-latest`.
- Added `src-tauri/src/lib.rs::get_desktop_platform()` and registered it in the Tauri invoke handler.
- Moved the `windows` crate into the `target_os = "windows"` dependency block in `src-tauri/Cargo.toml`.
- Added `src-tauri/tauri.macos.conf.json` so adaptation builds bundle `.app` only on macOS while the shared config still allows `"targets": "all"` elsewhere.
- Switched the macOS default hotkey to `Cmd+Shift+J`, and normalize both the legacy `CmdOrCtrl+T` default and the earlier macOS fallback `Alt+Shift+T` on macOS load so the runtime converges on one easier shortcut.
- Added a macOS startup fallback that auto-opens Settings on first launch or incomplete setup, so manual validation does not depend on tray discovery.
- Updated `ui/lib/SettingsPanel.svelte` so macOS and Linux disable Aura mode and save `aura_mode_enabled: false`.
- Updated `ui/lib/TranslationPopup.svelte` so unsupported actions do not render.
- Added UI regression tests in `SettingsPanel.test.ts` and `TranslationPopup.test.ts`.
- Added `ui/lib/TauriMacConfig.test.ts` to guard the macOS-specific bundle target override.
- Added Rust regression tests for the macOS hotkey default and legacy-default migration path.
- Added `docs/macOS-Adaptation-Checklist.md` for manual macOS runtime verification.

## Phase 2: macOS CI Observation - DONE

Status: **Done**

Goals:

- Confirm the newly added workflow passes on GitHub-hosted macOS infrastructure.

Completed work:

- Confirmed the `macOS Adaptation Gate` passes on GitHub-hosted `macos-latest`.
- Confirmed `npm ci`, `npm run check`, `npm test`, Rust tests, and `npm run tauri build` pass in the macOS workflow.
- Confirmed the adaptation build produces the expected macOS `.app` bundle.
- Confirmed the macOS-specific `.app` target avoids coupling the adaptation gate to DMG generation.

## Phase 3: Manual macOS Runtime Validation - DONE

Status: **Done**

Goals:

- Verify the first macOS build behaves correctly as a desktop utility, not only as a compiled artifact.

Completed work:

- Launching the built `.app` succeeds on the current macOS host and the process stays resident as `com.aura.translation`.
- The old `CmdOrCtrl+T` default was reproduced as a real Finder shortcut conflict on this host before the macOS hotkey migration landed.
- Confirmed tray/menu bar launch, interaction, and Settings access.
- Confirmed `Cmd+Shift+J` global hotkey registration and manual translation from copied text.
- Confirmed system API key save/read/delete through macOS Keychain.
- Confirmed native notifications for completion, retry, and failure flows.
- Confirmed transparent frameless windows, close/reopen behavior, and pinned always-on-top behavior.
- Confirmed Aura mode clipboard monitoring works via NSPasteboard change-count polling.
- Paste-back was validated during this phase but has since been removed from the supported product scope.
- Recorded direct manual evidence in `docs/macOS-Adaptation-Checklist.md`.

Known limitation:

- Scripted window inspection through `System Events` remains blocked by Accessibility denial (`osascript` reported `-25211`), but direct manual inspection completed the required runtime verification.

## Phase 4: Structured Capability Model - DONE

Status: **Done**

Goals:

- Replace raw platform checks with a backend capability payload so macOS support is feature-granular.

Completed work:

- Added `src-tauri/src/capabilities.rs` defining `FeatureCapability` (`Ready`, `Unsupported`) and `SystemCapabilities { aura_mode }`.
- `get_system_capabilities()` returns structured Aura mode capability status: `Ready` on Windows and macOS, `Unsupported` on Linux.
- Removed `get_desktop_platform()` command from the backend.
- Added `ui/lib/capabilities.ts` with TypeScript types for the capability payload.
- Updated `SettingsPanel.svelte` to consume `get_system_capabilities` instead of `get_desktop_platform`.
- Updated `TranslationWindowView.svelte` so it no longer owns paste-back status.
- Updated `TranslationPopup.svelte` so result actions expose copy only.
- Updated `SettingsPanel.test.ts` to mock `get_system_capabilities` instead of `get_desktop_platform`.
- All frontend and backend tests pass with the new capability model.

## Phase 5: macOS Native Feature Expansion - DONE

Status: **Done**

Goals:

- Implement native macOS equivalents for Aura mode clipboard monitoring.

Completed work:

- Implemented macOS Aura mode via `NSPasteboard::generalPasteboard().changeCount()` polling inside `spawn_clipboard_monitor`.
- Removed source-app paste-back from macOS and Windows paths.
- Removed the macOS Accessibility permission prompt and paste-back-specific Cargo dependencies.
- Manual hotkey translation remains available regardless of Aura mode state.

Known limitation:

- Source-app paste-back is intentionally outside the supported product scope.

## Implementation Rules

- Do not broaden the `windows` crate dependency outside the Windows target block.
- Do not treat CI build success as proof that tray, hotkey, Keychain, notification, transparency, or always-on-top behavior works at runtime.
- Do not require DMG packaging for the first macOS adaptation gate; treat `.app` output as the build artifact until a later distribution phase proves Finder-driven DMG generation.
- Do not use raw OS platform strings in the frontend; always consume the backend capability payload.

## Resolved Decisions

- **Capability payload timing:** Resolved. `get_desktop_platform` has been fully replaced by `get_system_capabilities` with structured Aura mode capability status.
- **macOS paste-back scope:** Resolved. Source-app paste-back has been removed from the supported product scope.
- **Clipboard privacy:** Resolved. macOS Aura mode uses `NSPasteboard` change-count polling only while Aura mode is enabled; polling stops when disabled.

## Open Questions

- **Artifact policy:** The current adaptation gate stops at `.app`; decide later whether DMG artifacts belong in a distribution-focused phase or should stay out of the adaptation workflow.
- **Linux support:** Aura mode is currently unsupported on Linux. Decide if and when to add Linux clipboard monitoring.
