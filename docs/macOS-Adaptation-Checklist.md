# macOS Adaptation Checklist

This checklist is the current macOS adaptation gate. It is not a public release gate yet; it proves that the macOS build can run the currently supported desktop workflows on a real macOS host.

## Automated Gates

The macOS GitHub Actions workflow runs on `macos-latest`:

```bash
npm ci
npm run check
npm test
cargo test --manifest-path ./src-tauri/Cargo.toml
npm run tauri build
```

## Latest Local Run - 2026-06-09

Host: local macOS desktop (`aarch64-apple-darwin`)

Automated results:

- `npm ci`: passed
- `npm run check`: passed
- `npm test`: passed (`47` UI tests)
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (`53` Rust tests)
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
8. Confirm Aura mode automatic clipboard translation is supported on macOS when capabilities are ready.
9. Confirm paste-back is supported on macOS, requesting Accessibility permissions if needed, and focusing and pasting back into the target app.

## Latest Manual Validation Notes - 2026-06-09

Verified on this host:

- **Application Lifecycle**: The built `.app` launches successfully and stays resident as `com.aura.translation` in the menu bar.
- **Onboarding Flow**: macOS startup correctly auto-opens the Settings panel on first launch/incomplete configuration.
- **Hotkey Verification**: Default global hotkey `Cmd+Shift+J` successfully triggers the manual translation flow, and legacy shortcuts are migrated properly in config.
- **Keychain Storage**: API credentials successfully save, read, and delete via the macOS native Keychain.
- **Visuals and Window States**: Frameless translucent window styling, pinned always-on-top behaviors, and close/reopen routines operate as expected.
- **Aura Mode on macOS**: Clipboard changes are tracked using native `NSPasteboard` polling and successfully trigger translations automatically when enabled.
- **Paste-back on macOS**: App-level targeting successfully focuses the originating process and simulates key events to inject translation results. Gracefully requests Accessibility permissions when needed.
- **Native Notifications**: System alerts/notifications fire correctly on completion, retry, or connection failure events.
- **Tray Menu**: Menu bar icon interactions and click responses behave correctly.

Blocked or still pending on this host:

- None. All manual validation checks successfully completed and verified.

Reproduction notes for the hotkey conflict:

1. Launch Aura Translation with a config that still persists `CmdOrCtrl+T`.
2. Bring Finder to the foreground.
3. Press `Cmd+T`.
4. Finder consumes the shortcut for its own tab behavior instead of surfacing Aura's translation path.

## Current Scope

- macOS supports manual hotkey translation, Settings, profiles, history, Keychain-backed secrets, notifications, tray/menu bar control, and pinned-window behavior.
- macOS fully supports Aura mode automatic clipboard monitoring.
- macOS supports source-app paste-back, utilizing a backend-driven capability model and requesting Accessibility permissions (AXIsProcessTrusted) when required.
