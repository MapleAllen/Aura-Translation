# Verification Matrix

Last audited: 2026-06-08
Current main SHA: `6039ec4`

## Automated Checks

| Check | Windows | macOS | Evidence |
|---|---|---|---|
| `npm run check` | PASS on 2026-06-08 at `7ba8c40` | PASS on 2026-06-08 at `6039ec4` | Windows local audit; Codex macOS rerun |
| `npm test` | PASS, 46 tests at `7ba8c40` | PASS, 46 tests at `6039ec4` | Windows local audit; Codex macOS rerun |
| `cargo test` | FAIL, 47/48 at `7ba8c40` | PASS, 53 tests at `6039ec4` | Windows hotkey assertion blocker on old SHA; Codex macOS rerun on current SHA |
| `npm run tauri build` | NOT RUN IN THIS AUDIT | PASS, `.app` produced | macOS checklist |
| Windows Trial Gate | FAIL at `7ba8c40` | N/A | https://github.com/MapleAllen/Aura-Translation/actions/runs/27127292677 |
| macOS Adaptation Gate | N/A | PASS at `7ba8c40` | https://github.com/MapleAllen/Aura-Translation/actions/runs/27127293378 |

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

- Re-run or confirm Windows local checks on `6039ec4`.
- Re-run the Windows Trial Gate on `6039ec4`.
- Confirm the macOS Adaptation Gate on `6039ec4`.
- Confirm both CI gates pass on the same final `main` SHA.
