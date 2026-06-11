# Verification Matrix

## Automated Checks

| Check | Windows | macOS | Evidence |
|---|---|---|---|
| `npm run check` | NOT RUN | PASS | Local macOS run passed during `P1A` on top of `00d91ec` worktree |
| `npm test` | NOT RUN | PASS | Local macOS run passed during `P1A` with 47 tests |
| `cargo test --manifest-path src-tauri/Cargo.toml` | NOT RUN | PASS | Local macOS run passed during `P1A` with 53 Rust tests |
| Production build | NOT RUN | NOT RUN | Windows CI bundle/build; macOS `npm run tauri build` plus CI |
| Final shared CI SHA | NOT RUN | NOT RUN | Both host gates must pass on the same final `main` commit |

## Manual Platform Checks

| Behavior | Required Host | Status | Evidence |
|---|---|---|---|
| Config parse/read failure surfaces as recoverable structured notification | Windows | NOT RUN | |
| Profiles parse/read failure surfaces as recoverable structured notification | Windows | NOT RUN | |
| History parse/read failure surfaces as recoverable structured notification | Windows | NOT RUN | |
| Probe cache returns a cached result inside TTL and invalidates after relevant config change | Windows | NOT RUN | |
| Tray icon and tooltip reflect ready vs needs-setup state after config changes | Windows | NOT RUN | |
| Legacy provider-scoped secret remains readable during profile-scoped-key migration | Windows | NOT RUN | |
| History filters narrow rows by text/status/language pair | Windows | NOT RUN | |
| Retry with original provider/model/base URL does not switch active profile | Windows | NOT RUN | |
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
