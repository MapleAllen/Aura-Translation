# Review, Main-Branch Handoff, and Closure

## Review Inputs

- Documentation baseline commit: `4fe5f24`
- Shared-source modules:
  - `src-tauri/src/config.rs`
  - `src-tauri/src/secrets.rs`
  - `src-tauri/src/profiles.rs`
  - `src-tauri/src/history.rs`
  - `src-tauri/src/readiness.rs`
  - `src-tauri/src/translate.rs`
  - `src-tauri/src/lib.rs`
- Frontend modules:
  - `ui/lib/SettingsPanel.svelte`
  - `ui/lib/SetupStatusCard.svelte`
  - `ui/lib/ProfileManager.svelte`
  - `ui/lib/HistoryList.svelte`
- Current-state truth:
  - `docs/Config-And-Secrets/*`
  - `docs/Runtime-Readiness/*`
  - `docs/Translation-Profiles/*`
  - `docs/Translation-History/*`

## Windows Codex Review

Status: WINDOWS CI PASSED ON SHARED `main` `98c20a2`; INTERACTIVE WINDOWS VERIFICATION DEFERRED
Blocking findings:

- none recorded at the current shared baseline

Open follow-up:

- No Windows host is currently available. Windows CI passed on `98c20a2`, but all interactive tray, UI, and credential-store rows are intentionally deferred and must not be treated as verified target-host behavior.

## macOS Codex Review

Status: P2 SHARED-SOURCE HANDOFF `19a1bad` REVIEWED AND PUSHED AS `98c20a2`; MACOS TARGET-HOST EVIDENCE PARTIAL
Blocking findings:

- Resolved during P2 review: original-config history replay initially resolved only system credentials, which would fail for a `plaintext_fallback` profile. Replay now reads the matching stored profile key without activating or mutating that profile.
- No new blocking code findings remain on `98c20a2`.

## Plan Compliance

- No implementation work starts until a single active lock is recorded in `01-shared-contracts.md`.
- Every implementation stage must record a starting SHA pulled from a clean `main`.
- Any public-contract or scope change discovered during implementation requires a plan amendment before source edits continue.

## Documentation Updates

- Update the touched module description and plan files under `docs/` after implementation evidence exists.
- Keep `plan/active/` focused on execution state, not on replacing module documentation.
- Windows verifier backfilled `04-verification.md` on `e2681b6` with passing local command results (`npm run check`, `npm test`, `cargo test --manifest-path src-tauri/Cargo.toml`) and with corrupt-startup evidence gathered from isolated `%APPDATA%` launches.
- macOS verifier backfilled `04-verification.md` on `98c20a2` with isolated bundle-launch evidence plus real-bundle history filter checks in a forced `/tmp` user directory.

## Main-Branch Handoff Order

1. Coordinator freezes scope and records the first implementation lock.
2. Assigned owner pulls latest `main`, records starting SHA, and performs only the allowed shared-source edit set for `P1`.
3. Codex reviews findings first, then commits and pushes approved `P1` work.
4. Windows and macOS verifiers pull the pushed `main`, run assigned checks, and record evidence.
5. Coordinator records the next implementation lock for `P2` from the new clean `main`.
6. Assigned owner pulls latest `main`, records starting SHA, and performs only the allowed shared-source edit set for `P2`.
7. Codex reviews findings first, then commits and pushes approved `P2` work.
8. Windows and macOS verifiers confirm final shared checks and target-host behavior on the same SHA.
9. Codex syncs `docs/`, closes residual risks, and archives the plan only after the remaining required macOS target-host rows are either verified or formally split into a follow-up plan.

## Rollback Plan

- Revert the most recent `P1` or `P2` stage commit from `main` if review or target-host evidence reveals a blocking regression.
- If profile-scoped secret migration proves unsafe on either host, revert only the migration stage and keep `P1` runtime/readiness hardening intact.
- If retry override behavior regresses the translation window, revert the one-shot override API and keep history filtering independently if it remains safe.

## Final Integration Gate

- [x] Shared and platform-specific tests pass.
- [x] Windows and macOS CI pass.
- [ ] Required target-host evidence is recorded.
- [x] Windows tray and UI rows are explicitly deferred until a real Windows host is available.
- [x] Blocking review findings are resolved.
- [x] `docs/` reflects the implemented current state.
- [x] Remaining risks and deviations are recorded.
- [x] Both CI gates pass on the same final `main` SHA.
- [x] One task owner was active at a time.
- [ ] Plan moved to `plan/completed/`.
