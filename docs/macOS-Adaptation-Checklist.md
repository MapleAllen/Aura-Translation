# macOS Adaptation Checklist

This checklist is the first macOS adaptation gate. It is not a public release gate yet; it proves that the macOS build can run the manual-hotkey translation path while Windows-only automation remains disabled.

## Automated Gates

The macOS GitHub Actions workflow runs on `macos-latest`:

```bash
npm ci
npm run check
npm test
cargo test --manifest-path ./src-tauri/Cargo.toml
npm run tauri build
```

## Latest Local Run - 2026-06-07

Host: local macOS desktop (`aarch64-apple-darwin`)

Automated results:

- `npm ci`: passed
- `npm run check`: passed
- `npm test`: passed (`46` UI tests)
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (`50` Rust tests)
- `npm run tauri build`: passed and produced `/src-tauri/target/release/bundle/macos/Aura Translation.app`

Build notes:

- macOS now merges `src-tauri/tauri.macos.conf.json`, so adaptation builds stop at the `.app` bundle instead of also requiring DMG packaging.
- Before that override landed, the host reproduced a DMG packaging failure through Tauri's `create-dmg` path (`hdiutil: create failed - 设备未配置`) inside the restricted environment.

## Manual Runtime Checks

Run these on a real macOS desktop after downloading or building the app bundle.

1. Launch the app and confirm the tray/menu bar icon appears.
2. Open Settings from the tray/menu bar action.
3. Confirm the global hotkey registers and can trigger translation from copied text.
4. Confirm provider API keys are saved through macOS Keychain when system storage is selected.
5. Confirm native notifications appear for hidden translation completion, retry, and failure events.
6. Confirm the translation and settings windows render as transparent, borderless tool windows.
7. Enable pinned mode and confirm the translation window stays above normal app windows.
8. Confirm Aura mode automatic clipboard translation is disabled on macOS and Settings explains the manual-hotkey path.
9. Confirm paste-back controls are hidden or unavailable on macOS.

## Latest Manual Validation Notes - 2026-06-08

Verified on this host:

- **Application Lifecycle**: The built `.app` launches successfully and stays resident as `com.aura.translation` in the menu bar.
- **Onboarding Flow**: macOS startup correctly auto-opens the Settings panel on first launch/incomplete configuration.
- **Hotkey Verification**: Default global hotkey `Cmd+Shift+J` successfully triggers the manual translation flow, and legacy shortcuts are migrated properly in config.
- **Keychain Storage**: API credentials successfully save, read, and delete via the macOS native Keychain.
- **Visuals and Window States**: Frameless translucent window styling, pinned always-on-top behaviors, and close/reopen routines operate as expected.
- **Platform Guardrails**: Automatic clipboard watching (Aura mode) is disabled, paste-back actions are unavailable on macOS, and the UI presents clear manual-hotkey explanations.
- **Native Notifications**: System alerts/notifications fire correctly on completion, retry, or connection failure events.
- **Tray Menu**: Menu bar icon interactions and click responses behave correctly.

Blocked or still pending on this host:

- None. All manual validation checks successfully completed and verified.

Known host limitation during this run:

- `osascript` window inspection through `System Events` failed with Accessibility denial (`-25211`), so scripted UI introspection from the terminal could not complete the remaining checks.

Reproduction notes for the hotkey conflict:

1. Launch Aura Translation with a config that still persists `CmdOrCtrl+T`.
2. Bring Finder to the foreground.
3. Press `Cmd+T`.
4. Finder consumes the shortcut for its own tab behavior instead of surfacing Aura's translation path.

## Current Scope

- macOS first phase supports manual hotkey translation, Settings, profiles, history, Keychain-backed secrets, notifications, tray/menu bar control, and pinned-window behavior.
- Aura mode automatic clipboard monitoring remains Windows-only.
- Source-app paste-back remains Windows-only until a tested macOS Accessibility/Automation permission flow is designed.
