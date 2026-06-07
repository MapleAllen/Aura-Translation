# Review and Integration

## Review Inputs

- Baseline branch: `codex/macos-v023-integration`
- Baseline command sequence:

```powershell
git switch main
git switch -c codex/macos-v023-integration
git merge --ff-only v0.2.3
```

- Windows implementation branch: `codex/macos-v023-windows-hardening`
- macOS verification branch: `codex/macos-v023-macos-verification`
- Shared status branch: `codex/macos-v023-shared-status`
- Excluded commits from final product history:
  - `3f199d2`
  - `7fd945e`

## Windows Codex Review

Status: NOT STARTED
Blocking findings:

- None yet. Expected initial review focus:
  - `src-tauri/src/hotkey.rs` test now matches platform defaults exactly.
  - `src-tauri/src/lib.rs` warning cleanup does not weaken macOS startup coverage.
  - Windows Trial Gate evidence points at the product-code SHA, not only an artifact-update SHA.

## macOS Codex Review

Status: NOT STARTED
Blocking findings:

- None yet. Expected initial review focus:
  - every required runtime check has real-host evidence,
  - any failed check is clearly classified as blocker vs residual risk,
  - CI success is not used as a substitute for tray/hotkey/Keychain/notification/window behavior.

## Plan Compliance

- Verify that only the assigned owner edited each shared file.
- Verify that no one merged or cherry-picked the CI-summary-only commits.
- Verify that any new issue discovered during Windows or macOS work was recorded before scope expanded.

## Documentation Updates

Required before merge:

- Update [docs/MacOS-Adaptation/MacOS-Adaptation-Plan.md](/E:/GitHub/Aura-Translation/docs/MacOS-Adaptation/MacOS-Adaptation-Plan.md) so Phase 2 no longer says `NOT STARTED`.
- Update [docs/macOS-Adaptation-Checklist.md](/E:/GitHub/Aura-Translation/docs/macOS-Adaptation-Checklist.md) with verified/pending status from the current hardening pass.
- Update `04-verification.md` with final branch-level evidence and links or hashes for both gates.

## Merge Order

1. Integration owner creates `codex/macos-v023-integration` from `main` and fast-forwards it to `v0.2.3`.
2. Windows implementer lands the hotkey test fix and warning cleanup into the integration branch after review.
3. macOS implementer records real-host verification against the integrated SHA.
4. Shared implementer updates docs and verification matrix from the actual evidence.
5. Integration owner reruns or confirms both CI gates on the integrated branch.
6. Integration owner merges the reviewed hardening branch back to `main`.

## Rollback Plan

- If the baseline fast-forward succeeds but hardening work fails review, keep `main` untouched and discard or rewrite the integration branch.
- If a bad hardening commit lands on the integration branch, revert that commit on the integration branch rather than changing scope mid-review.
- If a macOS runtime blocker is discovered, stop before `main` merge and open a follow-up plan for the product fix.

## Final Integration Gate

- [ ] Shared and platform-specific tests pass.
- [ ] Windows and macOS CI pass.
- [ ] Required target-host evidence is recorded.
- [ ] Blocking review findings are resolved.
- [ ] `docs/` reflects the implemented current state.
- [ ] Remaining risks and deviations are recorded.
- [ ] One integration owner is assigned to commit and push.
- [ ] Final history excludes `3f199d2` and `7fd945e`.
