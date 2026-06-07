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
- Updated `ui/lib/SettingsPanel.svelte` so macOS and Linux disable Aura mode and save `aura_mode_enabled: false`.
- Updated `ui/lib/TranslationPopup.svelte` so unsupported paste-back does not render.
- Added UI regression tests in `SettingsPanel.test.ts` and `TranslationPopup.test.ts`.
- Added `docs/macOS-Adaptation-Checklist.md` for manual macOS runtime verification.

## Phase 2: macOS CI Observation - NOT STARTED

Status: **Not Started**

Goals:

- Confirm the newly added workflow passes on GitHub-hosted macOS infrastructure.

Remaining features:

- Push or open a pull request from `codex/macos-adaptation` and inspect the `macOS Adaptation Gate` result.
- Confirm `npm run tauri build` produces a macOS bundle on `macos-latest`.
- Capture any macOS-specific build failures into this plan and update the implementation rules if new constraints appear.

## Phase 3: Manual macOS Runtime Validation - NOT STARTED

Status: **Not Started**

Goals:

- Verify the first macOS build behaves correctly as a desktop utility, not only as a compiled artifact.

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

## Open Questions

- **Capability payload timing:** Should `get_desktop_platform` remain enough for the first macOS trial, or should Phase 2 immediately replace it with structured capabilities?
- **macOS paste-back scope:** Is source-app paste-back important enough to justify Accessibility/Automation permissions, or should macOS keep copy-only output for the foreseeable future?
- **Artifact policy:** Should the macOS workflow upload `.app`/DMG artifacts during adaptation, or wait until a trial release gate exists?
- **Clipboard privacy:** If macOS Aura mode is implemented later, should it poll only while enabled, use a native listener, or require a more explicit user confirmation flow?
