# Windows Implementation and Verification

Owner: deferred until a Windows host is available
Dependencies: Shared contracts frozen
Working branch: `main`
Starting SHA: `98c20a284ad40784ef1a7d2b7b11f80e101fa463` for the deferred verification baseline

## Allowed Files

- Shared-contract allowed files from `01-shared-contracts.md`
- Windows-specific tray/icon assets if required
- Windows evidence rows in this plan

## Do Not Modify

- macOS-only clipboard automation and paste-back logic
- unrelated release packaging workflow logic
- `docs/MacOS-Adaptation/*` unless a reviewed regression requires documentation correction

## Platform-Specific Functions and Behavior

- System key storage on Windows must preserve access to existing provider-scoped secrets while enabling profile-scoped secret isolation for new writes.
- Readiness tray state must remain legible with Windows tray/icon constraints.
- Retry-with-original-provider must not silently switch the active profile or alter saved provider credentials.
- Structured startup/load failures must surface as recoverable user-visible notifications without preventing the daemon from booting.

## Implementation Tasks

- No Windows source work remains for this plan slice.
- Defer all interactive Windows verification until a real Windows host is available.
- Preserve `98c20a2` as the shared verification baseline for any future Windows follow-up.

## Automated Verification

- `npm run check`
- `npm test`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- Windows CI workflow(s) on the final shared SHA

## Windows Manual Verification

- Deferred in full. The latest shared CI already passed on `98c20a2`, but no direct Windows UI, tray, or credential-store evidence should be claimed until the follow-up host exists.

## Completion Evidence

- Shared CI passed on `98c20a2`.
- Prior Windows Codex evidence remains recorded for `P1A` startup-corruption survivability on `e2681b6`, but not for `P2` UI or tray behaviors.
- A later Windows follow-up must backfill target-host rows against `98c20a2` or a newer approved baseline.

## Deviations and Remaining Risks

- No Windows host is currently available, so interactive verification is intentionally deferred.
- Do not archive this plan on the basis of Windows CI alone if later macOS closure work reveals a regression that needs source changes.
