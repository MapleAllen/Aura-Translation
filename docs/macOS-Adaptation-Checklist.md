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

## Latest Manual Validation Notes - 2026-06-07

Verified on this host:

- The built `.app` launches successfully and stays resident as `com.aura.translation`.
- The previous macOS default hotkey `CmdOrCtrl+T` was reproduced as a real Finder conflict: pressing it in Finder opened the native Finder tab flow instead of surfacing Aura's translation path.
- The macOS runtime now normalizes both older defaults to `Cmd+Shift+J` in memory during config load, and startup/fallback hotkey registration uses that platform default.
- macOS startup now auto-opens Settings on first launch or incomplete setup, so the adaptation build no longer relies on tray discovery just to reach the configuration UI.

Blocked or still pending on this host:

- Tray/menu bar icon click behavior has not been conclusively validated yet.
- End-to-end global hotkey triggering still needs a direct desktop interaction pass after the host's automation/accessibility path is enabled.
- Keychain save/read/delete through the Settings UI is still pending because the Settings window was not yet driven interactively.
- Native notification appearance is still pending the same visible-window/manual interaction pass.
- Transparent borderless rendering, always-on-top behavior, and close/hide/reopen multi-window behavior are still pending because the translation/settings windows were not yet surfaced through an interactive desktop path.
- The actual hidden-state validation for Aura automatic clipboard controls and paste-back controls is still pending direct Settings/translation window inspection on this host.

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
