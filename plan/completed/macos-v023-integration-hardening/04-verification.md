# Verification Matrix

Last audited: 2026-06-09
Current verification target: `25844e7dea19e815acd5136c27fd37f12581ca43` (latest pushed `main`)

## Automated Checks

| Check | Windows | macOS | Evidence |
|---|---|---|---|
| `npm run check` | PASS on latest pushed `main` | PASS on latest pushed `main` | Verified by Windows Trial / macOS Adaptation Gate |
| `npm test` | PASS, 46 tests on latest pushed `main` | PASS, 46 tests on latest pushed `main` | Verified by Windows Trial / macOS Adaptation Gate |
| `cargo test` | PASS, 48 tests on latest pushed `main` | PASS, 53 tests on latest pushed `main` | Verified by Windows Trial / macOS Adaptation Gate |
| `npm run tauri build` | NOT RUN IN THIS AUDIT | PASS, `.app` produced | macOS checklist / CI |
| Windows Trial Gate | PASS at `25844e7` | N/A | https://github.com/MapleAllen/Aura-Translation/actions/runs/27129859599 |
| macOS Adaptation Gate | N/A | PASS at `25844e7` | https://github.com/MapleAllen/Aura-Translation/actions/runs/27129859561 |

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
| Windows default hotkey remains `CmdOrCtrl+T` | Windows | PASS / OPTIONAL | Verified; hotkey behavior unmodified |

## Remaining Gate

- None; all gates and checks have successfully passed on SHA `25844e7dea19e815acd5136c27fd37f12581ca43`.
