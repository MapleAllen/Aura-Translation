# macOS Adaptation Plan

## Objective

Make Aura Translation a credible macOS desktop build without pretending Windows-only automation already works cross-platform. The end state is a macOS build that passes CI, supports manual hotkey translation, persists provider secrets in Keychain, behaves correctly as a tray/menu bar daemon, and clearly disables unsupported clipboard automation and source-app paste-back until they have tested native implementations.

## Design Principles

- **Manual hotkey translation is the first macOS path.** Do not block macOS adaptation on automatic clipboard monitoring.
- **Unsupported controls must be hidden or disabled.** The UI should not offer Aura mode or paste-back where the backend reports no support.
- **Backend reports platform truth.** The frontend should use backend-reported platform/capability state instead of guessing native behavior.
- **Windows behavior must remain intact.** macOS adaptation must not regress Windows Aura mode, paste-back, installer, or trial workflow.
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
- Switched the macOS default hotkey to `Alt+Shift+T`, and normalize the legacy `CmdOrCtrl+T` default on macOS load so the runtime avoids Finder's built-in tab shortcut.
- Updated `ui/lib/SettingsPanel.svelte` so macOS and Linux disable Aura mode and save `aura_mode_enabled: false`.
- Updated `ui/lib/TranslationPopup.svelte` so unsupported paste-back does not render.
- Added UI regression tests in `SettingsPanel.test.ts` and `TranslationPopup.test.ts`.
- Added `ui/lib/TauriMacConfig.test.ts` to guard the macOS-specific bundle target override.
- Added Rust regression tests for the macOS hotkey default and legacy-default migration path.
- Added `docs/macOS-Adaptation-Checklist.md` for manual macOS runtime verification.

## Phase 2: macOS CI Observation - NOT STARTED

Status: **Not Started**

Goals:

- Confirm the newly added workflow passes on GitHub-hosted macOS infrastructure.

Remaining features:

- Push or open a pull request from `codex/macos-adaptation` and inspect the `macOS Adaptation Gate` result.
- Confirm `npm run tauri build` produces the expected macOS `.app` bundle on `macos-latest`.
- Capture any macOS-specific build failures into this plan and update the implementation rules if new constraints appear.

## Phase 3: Manual macOS Runtime Validation - IN PROGRESS

Status: **In Progress**

Goals:

- Verify the first macOS build behaves correctly as a desktop utility, not only as a compiled artifact.

Observed findings:

- Launching the built `.app` succeeds on the current macOS host and the process stays resident as `com.aura.translation`.
- The old `CmdOrCtrl+T` default was reproduced as a real Finder shortcut conflict on this host before the macOS hotkey migration landed.
- Scripted window inspection through `System Events` is currently blocked on this host by Accessibility denial (`osascript` reported `-25211`), so several UI-facing checklist items still need direct interactive validation.

Remaining features:

- Run the checklist in `docs/macOS-Adaptation-Checklist.md` on a real macOS desktop.
- Confirm tray/menu bar launch and Settings access.
- Confirm global hotkey registration and manual translation from copied text.
- Confirm system API key storage uses macOS Keychain.
- Confirm native notifications appear for hidden completion, retry, and failure events.
- Confirm transparent frameless windows and pinned always-on-top behavior.
- Confirm Aura mode remains disabled and paste-back remains hidden or unavailable.

## Phase 4: Structured Capability Model - NOT STARTED

Status: **Not Started**

Goals:

- Replace raw platform checks with a backend capability payload if macOS support becomes feature-granular.

Remaining features:

- Add a command such as `get_desktop_capabilities()` returning `aura_mode_supported`, `paste_back_supported`, `system_secret_storage_supported`, and `manual_hotkey_supported`.
- Update Settings and Translation popup to consume capabilities rather than platform labels.
- Add regression tests for each capability combination.

## Phase 5: macOS Native Feature Expansion - NOT STARTED

Status: **Not Started**

Goals:

- Decide whether macOS should gain native equivalents for Windows-only automation.

Remaining features:

- Evaluate native clipboard listening for Aura mode without excessive polling or privacy surprises.
- Evaluate source-app paste-back through Accessibility/Automation permissions.
- Add permission diagnostics before exposing any macOS paste-back action.
- Keep manual hotkey translation available even if native automation is declined by the user.

## Implementation Rules

- Do not enable `aura_mode_enabled` persistence on non-Windows platforms until a tested trigger path exists.
- Do not render paste-back controls when `pasteBackSupported` is `false`.
- Do not add macOS native automation without an explicit permission and failure-state UX.
- Do not broaden the `windows` crate dependency outside the Windows target block.
- Do not treat CI build success as proof that tray, hotkey, Keychain, notification, transparency, or always-on-top behavior works at runtime.
- Do not require DMG packaging for the first macOS adaptation gate; treat `.app` output as the build artifact until a later distribution phase proves Finder-driven DMG generation.

## Open Questions

- **Capability payload timing:** Should `get_desktop_platform` remain enough for the first macOS trial, or should Phase 2 immediately replace it with structured capabilities?
- **macOS paste-back scope:** Is source-app paste-back important enough to justify Accessibility/Automation permissions, or should macOS keep copy-only output for the foreseeable future?
- **Artifact policy:** The current adaptation gate stops at `.app`; decide later whether DMG artifacts belong in a distribution-focused phase or should stay out of the adaptation workflow.
- **Clipboard privacy:** If macOS Aura mode is implemented later, should it poll only while enabled, use a native listener, or require a more explicit user confirmation flow?
