# Review, Main-Branch Handoff, and Closure

## Review Inputs

- Shared contracts in `01-shared-contracts.md`
- macOS implementation evidence in `03-macos.md`
- Windows regression-verification evidence in `02-windows.md`
- Updated current-state docs under `docs/`

## Windows Codex Review

Status: COMPLETED (2026-06-09); NO SOURCE EDITS REQUIRED
Blocking findings:

- None. All automated checks (`npm run check`, `npm test`, `cargo test`) pass on Windows.
- No source edits were made; no regression fix review is needed.
- Manual verification passed on Windows host.

## macOS Codex Review

Status: LOCAL REVIEW PASSED; TARGET-HOST EVIDENCE RECORDED IN `04-verification.md` (2026-06-09)
Blocking findings:

- None at the current working-tree state.

Resolved during local review:

- Refreshed translation-window capability state around macOS paste-back permission changes so the UI no longer waits for a remount to pick up `needs_permission` vs `ready`.
- Corrected the pinned-composer shortcut wording and current-state UI-shell docs to match the actual cross-platform `Meta/Ctrl + Enter` behavior.

## Plan Compliance

- One task owner at a time on `main`.
- Starting SHA must be recorded before any source edits in the assigned task file.
- If implementation reality forces a contract change, update the plan before continuing source work.
- CI success alone is not enough to close macOS behavior claims; target-host evidence is mandatory.

## Documentation Updates

- Update `docs/UI-Shell/UI-Shell-Description.md` for Aura-mode and paste-back current state.
- Update `docs/Daemon-Core/Daemon-Core-Description.md` for capability reporting, macOS automation behavior, and remaining limitations.
- Update `docs/macOS-Adaptation-Checklist.md` with any new permission, Aura-mode, and paste-back verification steps.

## Main-Branch Handoff Order

1. Coordinator pushes this plan update and leaves the implementation lock unassigned.
2. macOS implementer pulls latest `main`, records the starting SHA, and performs the approved implementation scope.
3. Codex reviews the macOS stage, commits, and pushes approved changes to `main`.
4. Windows verifier pulls the new `main`, runs regression verification, and fixes only Windows regressions if needed.
5. Codex reviews any Windows regression fix, commits, and pushes to `main`.
6. macOS verifier pulls the latest `main`, captures target-host permission/runtime evidence, and updates the verification matrix.
7. Codex performs the final review, aligns docs, and archives the plan to `plan/completed/`.

## Rollback Plan

- If macOS capability work destabilizes Windows behavior, revert or narrow the offending stage on `main` before continuing.
- If macOS permissions or automation prove infeasible for the intended UX, stop and amend the plan scope instead of shipping a misleading partial claim.

## Final Integration Gate

- [x] Shared and platform-specific tests pass. (Win + macOS local: 2026-06-09 @ a1345a0)
- [x] Windows and macOS CI pass. (Passed on a1345a0)
- [x] Required macOS target-host evidence is recorded. (See `04-verification.md`)
- [x] Blocking review findings are resolved.
- [x] `docs/` reflects the implemented current state.
- [x] Remaining risks and deviations are recorded.
- [x] Both CI gates pass on the same final `main` SHA. (Passed on a1345a0)
- [x] One task owner was active at a time.
- [x] Plan moved to `plan/completed/`.
