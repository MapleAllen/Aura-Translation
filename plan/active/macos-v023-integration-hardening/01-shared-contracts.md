# Shared Contracts

## Main-Branch Coordination Contract

All work is sequential on `main`. A task lock is active only when this plan names one owner, one task, one starting SHA, and an allowed-file set.

Current task lock:

- Owner: Windows implementer
- Task: fix Windows hotkey regression test and startup-test warnings
- Starting SHA: RESOLVE AND RECORD AFTER THIS PLAN UPDATE IS PUSHED
- Status: BLOCKED UNTIL COORDINATOR COMMITS AND PUSHES THIS PLAN UPDATE
- Allowed files:
  - `src-tauri/src/hotkey.rs`
  - `src-tauri/src/lib.rs`
  - `plan/active/macos-v023-integration-hardening/02-windows.md`
- Do not modify:
  - macOS runtime behavior
  - unrelated source files
  - other task evidence files
  - `artifacts/windows-trial/ci-summary.json` manually

No other implementation agent may edit the repository until this lock is reviewed, committed, pushed, and released.

## Public Interfaces and Critical Functions

`src-tauri/src/hotkey.rs`

- `default_hotkey()`: macOS returns `Cmd+Shift+J`; non-macOS returns `CmdOrCtrl+T`.
- `normalize_persisted_hotkey()`: migrates only known macOS legacy defaults.
- `parse_hotkey()`: preserves current parsing and error behavior.

`src-tauri/src/lib.rs`

- `register_startup_hotkey()`: behavior is unchanged.
- `should_show_settings_on_startup()`: behavior is unchanged; only test import cfg cleanup is allowed.

## Handoff Rules

1. Pull latest `main` and confirm a clean worktree before starting.
2. Record the pulled `origin/main` SHA in the assigned task file before source edits. If `main` changes afterward, stop and request a refreshed lock.
3. Do not edit outside the allowed-file set.
4. Leave changes for Codex review; do not commit or push as the implementation agent.
5. After approval, Codex commits and pushes to `main`, records the handoff SHA, and updates the next task lock.
6. Never start Windows and macOS implementation simultaneously on different clones.

## Documentation Contract

- `docs/` is current-state truth.
- `plan/active/` records execution state, evidence, deviations, and handoffs.
- Update docs only after evidence exists.
- Archive this plan only after both CI gates pass and review is complete.

## Deviations

- Multi-branch ownership and integration-branch rules are retired.
- CI-summary commits already in `main` are retained as historical evidence.
