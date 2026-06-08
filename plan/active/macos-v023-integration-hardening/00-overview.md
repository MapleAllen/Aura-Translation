# macos-v023-integration-hardening Overview

Created: 2026-06-07
Last audited: 2026-06-08
Status: ACTIVE - SOURCE PATCH MERGED, FINAL GATES PENDING
Coordination owner: Codex coordinator
Working branch: `main`

## Goal

Finish the v0.2.3 cross-platform hardening work already merged into `main`: clear the Windows regression, preserve the verified macOS behavior, synchronize execution evidence and current-state documentation, complete review, and archive this plan.

## Current State

- `main` and the pushed remote `main` point to `6039ec4`.
- v0.2.3 product code and macOS manual-validation documentation are already merged.
- Windows source fix for the hotkey regression and startup-test warnings merged on `6039ec4`.
- macOS Adaptation Gate most recently passed for `7ba8c40`.
- Windows Trial Gate most recently failed for `7ba8c40`.
- Local Windows verification on 2026-06-08:
  - `npm run check`: PASS
  - `npm test`: PASS, 46 tests
  - `cargo test --manifest-path .\src-tauri\Cargo.toml`: FAIL, 47 passed / 1 failed
- The original Rust blocker in `platform_default_hotkey_is_parseable` is fixed on `6039ec4`.
- The known Windows startup-test warnings are fixed on `6039ec4`.
- Local macOS shared checks on `6039ec4` passed: `npm run check`, `npm test`, and `cargo test --manifest-path src-tauri/Cargo.toml`.
- macOS manual verification and structured evidence remain synchronized across `docs/`, `03-macos.md`, and `04-verification.md`.

## Process Decision

All future Windows, macOS, Antigravity, and Codex work occurs sequentially on `main`.

- Do not create implementation, integration, or review branches.
- Before a handoff, the current owner commits and pushes reviewed work to `main`.
- The next owner starts only after pulling the latest `main` and confirming a clean worktree.
- Only one active implementation owner may edit the repository at a time.
- The active task file records the task lock, starting SHA, allowed files, evidence, and handoff SHA.

## Scope

- Fix the Windows hotkey regression test.
- Remove the known Windows Rust warnings without weakening macOS coverage.
- Pass local Windows checks and the Windows Trial Gate on `main`.
- Synchronize macOS CI/manual evidence into this plan and `docs/`.
- Review current `main`, record deviations, and archive this plan after all gates pass.

## Non-Goals

- Do not add new macOS clipboard automation, paste-back, or capability-model features.
- Do not rewrite published history to remove CI-summary commits already merged into `main`.
- Do not treat CI as proof of interactive macOS behavior.

## Known Deviations

- The original multi-branch/worktree workflow was abandoned because agents could not reliably access each other's worktrees.
- `3f199d2` and `7fd945e` entered `main` despite the original exclusion rule. They are accepted as historical evidence commits; no history rewrite will be attempted.
- Product code was merged before the Windows regression was fixed and before final review gates were completed.
- The plan remained stale after merge and is being corrected by this audit.

## Acceptance Criteria

- Windows hotkey regression test distinguishes macOS `KeyJ` from non-macOS `KeyT`.
- Windows Rust tests pass without the two known startup-test import warnings.
- `npm run check`, `npm test`, and `cargo test` pass on Windows.
- Windows Trial Gate and macOS Adaptation Gate pass on the same current `main` SHA.
- macOS manual evidence is recorded in `03-macos.md`, `04-verification.md`, and current-state docs.
- Codex reviews complete with no unresolved blocking findings.
- Plan deviations and remaining risks are recorded.
- This folder moves from `plan/active/` to `plan/completed/`.

## Main-Branch Handoff Sequence

1. Coordinator assigns one task lock in the relevant plan file.
2. Assigned implementer pulls latest `main`, verifies the starting SHA, and performs only the allowed task.
3. Codex reviews the working-tree changes and evidence.
4. After approval, Codex commits and pushes to `main`.
5. The next platform/role pulls the new `main` and continues with verification and closure evidence.
