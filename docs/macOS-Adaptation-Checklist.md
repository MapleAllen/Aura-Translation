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

## Current Scope

- macOS first phase supports manual hotkey translation, Settings, profiles, history, Keychain-backed secrets, notifications, tray/menu bar control, and pinned-window behavior.
- Aura mode automatic clipboard monitoring remains Windows-only.
- Source-app paste-back remains Windows-only until a tested macOS Accessibility/Automation permission flow is designed.
