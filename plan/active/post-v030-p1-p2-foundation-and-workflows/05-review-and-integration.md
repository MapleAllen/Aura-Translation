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

Status: NOT STARTED
Blocking findings:

- none recorded yet

## macOS Codex Review

Status: NOT STARTED
Blocking findings:

- none recorded yet

## Plan Compliance

- No implementation work starts until a single active lock is recorded in `01-shared-contracts.md`.
- Every implementation stage must record a starting SHA pulled from a clean `main`.
- Any public-contract or scope change discovered during implementation requires a plan amendment before source edits continue.

## Documentation Updates

- Update the touched module description and plan files under `docs/` after implementation evidence exists.
- Keep `plan/active/` focused on execution state, not on replacing module documentation.

## Main-Branch Handoff Order

1. Coordinator freezes scope and records the first implementation lock.
2. Assigned owner pulls latest `main`, records starting SHA, and performs only the allowed shared-source edit set for `P1`.
3. Codex reviews findings first, then commits and pushes approved `P1` work.
4. Windows and macOS verifiers pull the pushed `main`, run assigned checks, and record evidence.
5. Coordinator records the next implementation lock for `P2` from the new clean `main`.
6. Assigned owner pulls latest `main`, records starting SHA, and performs only the allowed shared-source edit set for `P2`.
7. Codex reviews findings first, then commits and pushes approved `P2` work.
8. Windows and macOS verifiers confirm final shared checks and target-host behavior on the same SHA.
9. Codex syncs `docs/`, closes residual risks, and archives the plan.

## Rollback Plan

- Revert the most recent `P1` or `P2` stage commit from `main` if review or target-host evidence reveals a blocking regression.
- If profile-scoped secret migration proves unsafe on either host, revert only the migration stage and keep `P1` runtime/readiness hardening intact.
- If retry override behavior regresses the translation window, revert the one-shot override API and keep history filtering independently if it remains safe.

## Final Integration Gate

- [ ] Shared and platform-specific tests pass.
- [ ] Windows and macOS CI pass.
- [ ] Required target-host evidence is recorded.
- [ ] Blocking review findings are resolved.
- [ ] `docs/` reflects the implemented current state.
- [ ] Remaining risks and deviations are recorded.
- [ ] Both CI gates pass on the same final `main` SHA.
- [ ] One task owner was active at a time.
- [ ] Plan moved to `plan/completed/`.
