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

## Latest Local Run - 2026-06-18

Host: local macOS desktop (`aarch64-apple-darwin`)

Automated results:

- `npm ci`: passed
- `npm run check`: passed
- `npm test`: passed (`49` UI tests)
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (`57` Rust tests)
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
9. Confirm result actions expose copy only; source-app paste-back and Accessibility permission prompts are absent.

### M1 interaction and appearance checks

These cover the M1 behaviour contract. The counter-based rows are driven by
`scripts/release/accept-interaction-macos.sh`, which runs a request-counting stub provider.

10. Left click the menu-bar icon and confirm the menu offers open-translation, automatic-translation,
    settings, and quit, with the automatic-translation item reflecting the current setting.
11. Copy text, press the hotkey, then press it again with the window visible: the window collapses
    and the stub request counter does not increase.
12. Press the hotkey again with the window hidden: the previous result returns and the counter does
    not increase.
13. Press `Esc` once: the window collapses. While pinned, one `Esc` still collapses it and clears pinning.
14. Confirm `Esc` does not close the window while a Chinese IME composition is active.
15. With automatic translation off, copy text and confirm the stub counter does not move; turn it on
    and confirm the text copied while it was off is not translated.
16. Change the language direction or model, then press the hotkey on the same text: a new request is
    issued, because reuse is keyed on the full request identity.
17. Press the hotkey with an empty clipboard: an editable empty state opens instead of an error.
18. Switch macOS to dark appearance and confirm both windows follow it with no white panels left.
19. Confirm text renders in the system font (SF Pro / PingFang) rather than a Windows-first stack.
20. Resize the translation window by dragging, then let content stream: the window keeps the chosen
    height instead of snapping back.
21. With automatic translation off, run `scripts/release/measure-resources-macos.sh` and confirm the
    stack sample contains zero `NSPasteboard` frames.

## M1 Automated Verification - 2026-09-13

Baseline `a6d1d16` plus the M1 working tree. Host: local macOS desktop (`aarch64-apple-darwin`).

- `npm run check`: passed, 0 errors / 0 warnings
- `npm test`: passed (`64` UI tests across 10 files)
- `npm run build`: passed, static output written to `build/`
- `cargo test --manifest-path src-tauri/Cargo.toml --locked`: passed (`75` Rust tests)
- `scripts/release/mock-translate-server.py`: exercised by hand for both the streaming and
  non-streaming paths and for counter reset
- `python3 -m unittest discover -s scripts/release -p 'test_*.py'`: passed, 11 tests covering the
  cold/warm split, recall-latency arithmetic, the sample-count gates, and malformed trace lines

The M1 interaction rules are asserted without a Tauri `AppHandle` in
`src-tauri/src/interaction.rs`, which is what makes "recalling the same text issues no request"
checkable in CI rather than only on a host.

Release-bundle smoke runs on this host (isolated `HOME`, `AURA_TRACE=1`):

- The bundle launches and the menu bar becomes operable at `tray-ready`.
- With `setup_completed: true` in
  `~/Library/Application Support/aura-translation/config.json`, no onboarding window opens; the
  trace shows only `setup-start` and `tray-ready`. With the flag absent or false, the settings
  window opens. This confirms the onboarding flag is honoured end to end.
- The macOS config path is `~/Library/Application Support/aura-translation/config.json`, because
  `dirs::config_dir()` does not resolve to `~/.config` on macOS. `accept-interaction-macos.sh`
  writes there after this was found by testing the bundle.

Still requiring a real-host run (not yet recorded here):

- M1 manual checks 10-21 above
- `scripts/release/measure-resources-macos.sh`, including the zero-`NSPasteboard` gate
- `scripts/release/measure-startup-macos.sh` cold-start and warm-recall percentiles
- The five-day daily-driver trial

## Latest Manual Validation Notes - 2026-06-09

Verified on this host:

- **Application Lifecycle**: The built `.app` launches successfully and stays resident as `com.aura.translation` in the menu bar.
- **Onboarding Flow**: macOS startup correctly auto-opens the Settings panel on first launch/incomplete configuration.
- **Hotkey Verification**: Default global hotkey `Cmd+Shift+J` successfully triggers the manual translation flow, and legacy shortcuts are migrated properly in config.
- **Keychain Storage**: API credentials successfully save, read, and delete via the macOS native Keychain.
- **Visuals and Window States**: Frameless translucent window styling, pinned always-on-top behaviors, and close/reopen routines operate as expected.
- **Aura Mode on macOS**: Clipboard changes are tracked using native `NSPasteboard` polling and successfully trigger translations automatically when enabled.
- **Result actions on macOS**: The translation result exposes copy only; source-app paste-back and Accessibility permission prompts are absent.
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
- Source-app paste-back has been removed; macOS no longer requests Accessibility permission for this flow.
