# Verification Matrix

## Automated Checks

| Check | Windows | macOS | Evidence |
|---|---|---|---|
| `npm run check` | PASS | PASS | Local Windows run passed on shared verification baseline `e2681b6`; local macOS P2 working-tree run passed from starting SHA `d141ac2` |
| `npm test` | PASS | PASS | Local Windows run passed on shared verification baseline `e2681b6` with 47 tests; local macOS P2 working-tree run passed with 49 tests |
| `cargo test --manifest-path src-tauri/Cargo.toml` | PASS | PASS | Local Windows run passed on shared verification baseline `e2681b6` with 48 Rust tests; local macOS P2 working-tree run passed with 57 tests after clearing proxy variables for localhost WireMock |
| Production build | NOT RUN | PASS | Windows CI bundle/build still pending; macOS `npm run tauri build` produced `Aura Translation.app` from the P2 working tree |
| Final shared CI SHA | NOT RUN | NOT RUN | Both host gates must pass on the same final `main` commit |

## Manual Platform Checks

| Behavior | Required Host | Status | Evidence |
|---|---|---|---|
| Config parse/read failure surfaces as recoverable structured notification | Windows | BLOCKED | Isolated `%APPDATA%` launch with malformed `config.json` kept `aura-translation.exe` alive and responding after 5s (`PID 25168`, `config.json:18`, `profiles.json:50`, `history.json:7`), but the recoverable `daemon-error` notification could not be inspected from this Codex desktop session because no tray shell handle was exposed (`Shell_TrayWnd=0`). |
| Profiles parse/read failure surfaces as recoverable structured notification | Windows | BLOCKED | Isolated `%APPDATA%` launch with malformed `profiles.json` kept `aura-translation.exe` alive and responding after 5s (`PID 28268`, `config.json:329`, `profiles.json:18`, `history.json:7`), but the recoverable `daemon-error` notification could not be inspected from this Codex desktop session because no tray shell handle was exposed (`Shell_TrayWnd=0`). |
| History parse/read failure surfaces as recoverable structured notification | Windows | BLOCKED | Isolated `%APPDATA%` launch with malformed `history.json` kept `aura-translation.exe` alive and responding after 5s (`PID 22596`, `config.json:329`, `profiles.json:50`, `history.json:18`), but the recoverable `daemon-error` notification could not be inspected from this Codex desktop session because no tray shell handle was exposed (`Shell_TrayWnd=0`). |
| Probe cache returns a cached result inside TTL and invalidates after relevant config change | Windows | BLOCKED | Requires interactive Settings-window access to drive repeated probe clicks and config saves. In this Codex Windows session the app process launches, but the tray/desktop shell is not enumerable (`Shell_TrayWnd=0`) so Settings could not be opened for UI-level verification. |
| Tray icon and tooltip reflect ready vs needs-setup state after config changes | Windows | BLOCKED | Direct tray verification is blocked in this Codex session: `explorer.exe` is present in session 5, but `FindWindow('Shell_TrayWnd')` returns `0`, so the tray tooltip/state cannot be observed. |
| Legacy provider-scoped secret remains readable during profile-scoped-key migration | Windows | N/A ON `e2681b6` | Shared runtime/source handoff for this verification remains `569d229` (`P1A` only). Profile-scoped secret migration is a later `P2` concern and is not present on the verified Windows baseline. |
| History filters narrow rows by text/status/language pair | Windows | N/A ON `e2681b6` | `P2` history-filter scope has not been assigned or delivered on the shared verification baseline. |
| Retry with original provider/model/base URL does not switch active profile | Windows | N/A ON `e2681b6` | `P2` retry-override scope has not been assigned or delivered on the shared verification baseline. |
| Config parse/read failure surfaces as recoverable structured notification | macOS | NOT RUN | |
| Profiles parse/read failure surfaces as recoverable structured notification | macOS | NOT RUN | |
| History parse/read failure surfaces as recoverable structured notification | macOS | NOT RUN | |
| Probe cache returns a cached result inside TTL and invalidates after relevant config change | macOS | NOT RUN | |
| Tray icon and tooltip reflect ready vs needs-setup state after config changes | macOS | NOT RUN | |
| Legacy provider-scoped secret remains readable during profile-scoped-key migration | macOS | NOT RUN | |
| History filters narrow rows by text/status/language pair | macOS | NOT RUN | |
| Retry with original provider/model/base URL does not switch active profile | macOS | NOT RUN | |
| Existing Aura mode, paste-back, and settings startup flow still behave as before | macOS | NOT RUN | |

## Regression Coverage

- `ui/lib/SettingsPanel.test.ts`
- `ui/lib/ProfileManager.test.ts`
- `ui/lib/HistoryList.test.ts`
- `src-tauri/src/history.rs` unit tests
- Any new Rust tests added for secrets/config/readiness/profile migration paths

## Known Limitations

- Linux secret-service support remains out of scope for this plan.
- Profile reorder, import/export, and history export remain out of scope for this plan.
- Aura Guard future phases remain unstarted and must not be treated as part of P1/P2 closure.
