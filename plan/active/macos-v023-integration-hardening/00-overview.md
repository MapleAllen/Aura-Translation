# macos-v023-integration-hardening Overview

Created: 2026-06-07
Status: ACTIVE
Integration owner: Codex coordinator

## Goal

Integrate `v0.2.3` onto `main` without the non-product CI-summary commits, clear the Windows CI blocker caused by the cross-platform hotkey test, capture real macOS runtime evidence, and bring `docs/` back in sync with the verified state.

## Scope

- Create an integration baseline from `main` by fast-forwarding to `v0.2.3`.
- Fix the Windows blocker in `src-tauri/src/hotkey.rs` test coverage.
- Remove the Windows Rust warning caused by macOS-only startup test imports.
- Re-run and pass the Windows Trial Gate on the hardening branch.
- Complete the real-device macOS verification checklist for tray, hotkey, Keychain, notifications, transparent windows, and always-on-top behavior.
- Update the macOS plan, checklist, and shared verification matrix to reflect verified 2026-06-07 status and any new evidence from this hardening pass.
- Review and merge only the release tag plus intentional fixes into `main`.

## Non-Goals

- Do not merge `3f199d2` or `7fd945e` into product history. Those commits only update Windows CI summary artifacts.
- Do not add new macOS automation features such as clipboard listening, paste-back, or a capability-model refactor.
- Do not change release packaging policy beyond the already-tagged `v0.2.3` scope.
- Do not claim macOS runtime behavior from CI alone.

## Current-State Evidence

- `main` / `origin/main` currently point to `ebe2162a6cf8ed2ecf098ce92a2c37e0967c90d0`.
- `git merge-base --is-ancestor ebe2162 6a826fb` succeeded locally, so `git merge --ff-only v0.2.3` is valid from `main`.
- Tag `v0.2.3` resolves to commit `6a826fb546a06dca790b397026c0faeb996ee296`.
- `origin/codex/macos-adaptation-runtime` currently points to `7fd945e31635e14c85bbe32846fa2c00e9aaaade`.
- Commits after the tag on the runtime branch are:
  - `3f199d2` `chore: mark windows trial run start [skip ci]`
  - `7fd945e` `chore: record windows trial summary [skip ci]`
- The blocking Windows failure is in `v0.2.3:src-tauri/src/hotkey.rs` where `platform_default_hotkey_is_parseable` asserts `Code::KeyJ` on every platform, but Windows still defaults to `CmdOrCtrl+T` and therefore parses to `Code::KeyT`.
- The user-reported `macOS Adaptation Gate` succeeded on 2026-06-07, but [docs/MacOS-Adaptation/MacOS-Adaptation-Plan.md](/E:/GitHub/Aura-Translation/docs/MacOS-Adaptation/MacOS-Adaptation-Plan.md) still marks Phase 2 as `NOT STARTED`.
- The user also reported two non-blocking Windows Rust `unused import` warnings in the macOS-only startup test module because imports were not guarded together with `#[cfg(target_os = "macos")]`.

## Invariants

- `docs/` must be the final source of current-state truth once this effort completes.
- One owner edits each shared file during execution; no concurrent editing of the same file across agents.
- No implementation agent pushes directly to `main`.
- Product integration must contain the release tag baseline plus deliberate fixes only.
- Manual macOS claims require host evidence with timestamp, host info, and observed behavior.

## Dependencies

- Integration branch created from local `main` and fast-forwarded to `v0.2.3`.
- A Windows host or CI run capable of executing the Windows Trial Gate.
- A real macOS host capable of interactive runtime validation.
- Shared agreement that `artifacts/windows-trial/ci-summary.json` is evidence-only and not a merge target for this hardening effort.

## Risks and Unknowns

- The macOS verification pass may uncover a product bug rather than a documentation gap. If that happens, stop verification-only work and amend this plan before code changes.
- The Windows warning cleanup may touch `src-tauri/src/lib.rs`, a shared Rust file. Ownership is explicitly assigned in `01-shared-contracts.md` to avoid overlap.
- CI on the new integration branch may diverge from the previously reported 2026-06-07 macOS success and require a follow-up plan amendment.

## Acceptance Criteria

- `codex/macos-v023-integration` is created from `main` and fast-forwarded to `v0.2.3`.
- The hotkey regression test passes on Windows and correctly distinguishes macOS `KeyJ` from non-macOS `KeyT`.
- The Windows Rust test suite no longer emits the known macOS-only startup import warnings.
- The Windows Trial Gate passes on the hardening branch, with evidence recorded but without merging CI-summary-only commits.
- Real macOS evidence is recorded for tray/menu bar access, manual hotkey translation, Keychain storage, notifications, transparent windows, and always-on-top behavior.
- [docs/MacOS-Adaptation/MacOS-Adaptation-Plan.md](/E:/GitHub/Aura-Translation/docs/MacOS-Adaptation/MacOS-Adaptation-Plan.md), [docs/macOS-Adaptation-Checklist.md](/E:/GitHub/Aura-Translation/docs/macOS-Adaptation-Checklist.md), and this plan's verification matrix all match the verified state.
- Reviews complete with no unresolved blocking findings before merge to `main`.

## Branch and Merge Strategy

Baseline setup, owned by the integration coordinator:

```powershell
git switch main
git switch -c codex/macos-v023-integration
git merge --ff-only v0.2.3
```

Implementation branches after the baseline is created:

- Windows implementer: `codex/macos-v023-windows-hardening`
- macOS implementer: `codex/macos-v023-macos-verification`
- Shared implementer: `codex/macos-v023-shared-status`

Rules:

- Branch each implementation branch from `codex/macos-v023-integration`, not from `main`.
- Do not cherry-pick `3f199d2` or `7fd945e`.
- If the Windows workflow auto-commits `artifacts/windows-trial/ci-summary.json` on a branch, treat that commit as branch-local evidence and exclude it from the final integration history.
