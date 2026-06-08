# Verification Matrix

Last audited: 2026-06-08
Current main SHA: `7ba8c40`

## Automated Checks

| Check | Windows | macOS | Evidence |
|---|---|---|---|
| `npm run check` | PASS on 2026-06-08 | PASS on 2026-06-07 | Windows local audit; macOS checklist |
| `npm test` | PASS, 46 tests | PASS, 46 tests | Windows local audit; macOS checklist |
| `cargo test` | FAIL, 47/48 | PASS, 50 tests | Windows hotkey assertion blocker; macOS checklist |
| `npm run tauri build` | NOT RUN IN THIS AUDIT | PASS, `.app` produced | macOS checklist |
| Windows Trial Gate | FAIL | N/A | https://github.com/MapleAllen/Aura-Translation/actions/runs/27127292677 |
| macOS Adaptation Gate | N/A | PASS | https://github.com/MapleAllen/Aura-Translation/actions/runs/27127293378 |

## Manual Platform Checks

| Behavior | Required Host | Status | Evidence |
|---|---|---|---|
| Tray/menu bar and Settings access | macOS | PASS | `03-macos.md`; macOS checklist |
| Global hotkey manual translation | macOS | PASS | `03-macos.md`; macOS checklist |
| Keychain credential storage | macOS | PASS | `03-macos.md`; macOS checklist |
| Native notifications | macOS | PASS | `03-macos.md`; macOS checklist |
| Transparent/borderless windows | macOS | PASS | `03-macos.md`; macOS checklist |
| Always-on-top pinned behavior | macOS | PASS | `03-macos.md`; macOS checklist |
| Aura mode disabled | macOS | PASS | `03-macos.md`; macOS checklist |
| Paste-back unavailable | macOS | PASS | `03-macos.md`; macOS checklist |
| Windows default hotkey remains `CmdOrCtrl+T` | Windows | PENDING / OPTIONAL | Required only if runtime code changes |

## Remaining Gate

- Fix Windows Rust test and warnings.
- Pass all Windows local checks.
- Push approved fix to `main`.
- Confirm both CI gates pass on the same final `main` SHA.
