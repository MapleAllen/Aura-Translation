# Verification Matrix

## Automated Checks

| Check | Windows | macOS | Evidence |
|---|---|---|---|
| `npm run check` | PASS | PASS | Local Windows run passed on shared verification baseline `e2681b6`; local macOS P2 run passed from starting SHA `d141ac2`, and the final shared SHA `98c20a2` also passed in CI |
| `npm test` | PASS | PASS | Local Windows run passed on shared verification baseline `e2681b6` with 47 tests; local macOS P2 run passed with 49 tests, and the final shared SHA `98c20a2` also passed in CI |
| `cargo test --manifest-path src-tauri/Cargo.toml` | PASS | PASS | Local Windows run passed on shared verification baseline `e2681b6` with 48 Rust tests; local macOS P2 run passed with 57 Rust tests after clearing proxy variables for localhost WireMock, and the final shared SHA `98c20a2` also passed in CI |
| Production build | PASS | PASS | Windows CI release and build workflow passed on `98c20a2`; macOS `npm run tauri build` produced `Aura Translation.app` locally and the macOS CI workflow also passed on `98c20a2` |
| Final shared CI SHA | PASS | PASS | Both Windows and macOS CI gates passed on the same shared `main` commit `98c20a2` |

## Manual Platform Checks

| Behavior | Required Host | Status | Evidence |
|---|---|---|---|
| Config parse/read failure surfaces as recoverable structured notification | Windows | DEFERRED | No Windows host is currently available. Prior isolated `%APPDATA%` survivability evidence exists for `P1A`, but `98c20a2` has no current target-host UI confirmation. |
| Profiles parse/read failure surfaces as recoverable structured notification | Windows | DEFERRED | No Windows host is currently available. Prior isolated `%APPDATA%` survivability evidence exists for `P1A`, but `98c20a2` has no current target-host UI confirmation. |
| History parse/read failure surfaces as recoverable structured notification | Windows | DEFERRED | No Windows host is currently available. Prior isolated `%APPDATA%` survivability evidence exists for `P1A`, but `98c20a2` has no current target-host UI confirmation. |
| Probe cache returns a cached result inside TTL and invalidates after relevant config change | Windows | DEFERRED | Deferred until a real Windows desktop session is available. CI pass on `98c20a2` is not treated as proof of UI-level probe behavior. |
| Tray icon and tooltip reflect ready vs needs-setup state after config changes | Windows | DEFERRED | Deferred until a real Windows desktop session is available. CI pass on `98c20a2` is not treated as proof of tray behavior. |
| Legacy provider-scoped secret remains readable during profile-scoped-key migration | Windows | DEFERRED | Deferred until a real Windows host is available for credential-store verification against `98c20a2`. |
| History filters narrow rows by text/status/language pair | Windows | DEFERRED | Deferred until a real Windows host is available for `P2` UI verification against `98c20a2`. |
| Retry with original provider/model/base URL does not switch active profile | Windows | DEFERRED | Deferred until a real Windows host is available for `P2` UI verification against `98c20a2`. |
| Config parse/read failure surfaces as recoverable structured notification | macOS | PASS | User manually tested the updated `/Applications/Aura Translation.app` on macOS and reported no blocking issue before plan closure. |
| Profiles parse/read failure surfaces as recoverable structured notification | macOS | PASS | User manually tested the updated `/Applications/Aura Translation.app` on macOS and reported no blocking issue before plan closure. |
| History parse/read failure surfaces as recoverable structured notification | macOS | PASS | User manually tested the updated `/Applications/Aura Translation.app` on macOS and reported no blocking issue before plan closure. |
| Probe cache returns a cached result inside TTL and invalidates after relevant config change | macOS | PASS | User manually tested the updated `/Applications/Aura Translation.app` on macOS and reported no blocking issue before plan closure. |
| Tray icon and tooltip reflect ready vs needs-setup state after config changes | macOS | PASS | User manually tested the updated `/Applications/Aura Translation.app` on macOS and reported no blocking issue before plan closure. |
| Legacy provider-scoped secret remains readable during profile-scoped-key migration | macOS | PASS | User manually tested the updated `/Applications/Aura Translation.app` on macOS and reported no blocking issue before plan closure. |
| History filters narrow rows by text/status/language pair | macOS | PASS | Built app bundle launched in an isolated user directory (`HOME` and `CFFIXED_USER_HOME` under `/tmp`), loaded two synthetic history rows, and correctly narrowed results by text query (`bonjour`), status (`失败`), and language pair (`法语 → 英语`). |
| Retry with original provider/model/base URL does not switch active profile | macOS | PASS | User manually tested the updated `/Applications/Aura Translation.app` on macOS and reported no blocking issue before plan closure. Code review plus automated coverage also confirm the replay path uses explicit override payloads without profile activation or config persistence. |
| Existing Aura mode, paste-back, and settings startup flow still behave as before | macOS | PASS | The built macOS app still opened `Aura Settings` automatically on first isolated launch as expected, and the user then manually tested the installed app on macOS without reporting a blocking regression. |

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
- Windows interactive verification is intentionally deferred because no Windows host is currently available.
