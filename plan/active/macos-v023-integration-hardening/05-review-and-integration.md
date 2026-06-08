# Review, Main-Branch Handoff, and Closure

## Workflow Decision

All future collaboration occurs sequentially on `main`. Worktree and implementation-branch coordination is retired.

## Current Review Status

### Windows Codex Review

Status: BLOCKED UNTIL WINDOWS FIX EXISTS

Required review focus:

- platform-specific default hotkey test expectations;
- no runtime hotkey behavior regression;
- macOS startup tests remain present while Windows warnings disappear;
- local Windows tests and Windows Trial Gate pass.

### macOS Codex Review

Status: PARTIAL

- Manual runtime verification is recorded as complete.
- macOS Adaptation Gate passed on `7ba8c40`.
- Final sign-off waits for the Windows fix and a macOS gate pass on the same final `main` SHA.

## Accepted Deviations

- `3f199d2` and `7fd945e` entered `main`; they remain in history.
- v0.2.3 was merged before Windows hardening and final review.
- Multi-branch/worktree workflow was replaced by sequential main-branch handoffs.

## Main-Branch Handoff

1. Windows implementer works from task lock starting at `7ba8c40`.
2. Codex reviews working-tree changes and local evidence.
3. Codex commits and pushes approved Windows fix to `main`.
4. Both CI gates run on the resulting SHA.
5. macOS verifier confirms no affected manual checks need repetition, or reruns them if needed.
6. Codex updates docs, completes review, and archives this plan.

## Final Closure Gate

- [ ] Shared and platform-specific tests pass.
- [ ] Windows and macOS CI pass on the same final `main` SHA.
- [x] Required macOS target-host evidence is recorded.
- [ ] Windows and macOS Codex reviews have no unresolved blocking findings.
- [x] `docs/` reflects the implemented current state.
- [x] Process deviations are recorded.
- [x] One sequential task owner is active at a time.
- [ ] Plan moved to `plan/completed/`.
