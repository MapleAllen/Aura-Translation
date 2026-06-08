# Shared Contracts

## Main-Branch Coordination Contract

All work is sequential on `main`. A task lock is active only when this plan names one owner, one task, one starting SHA, and an allowed-file set.

Current task lock:

- Owner: none
- Task: source implementation lock released after reviewed merge; awaiting verification evidence on current `main`
- Starting SHA: `f7eaae18dd131b0043c5486095f1771e48b0c55e`
- Status: NO ACTIVE IMPLEMENTATION OWNER
- Allowed files: none until the coordinator assigns the next verification or closure task lock
- Do not modify:
  - product source files without a new assigned lock
  - macOS runtime behavior
  - unrelated evidence files
  - `artifacts/windows-trial/ci-summary.json` manually

No implementation agent may edit the repository again until the coordinator records the next owner, scope, and allowed files.

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
5. After approval, Codex commits and pushes to `main`, records the handoff SHA, and updates or clears the next task lock.
6. Never start Windows and macOS implementation simultaneously on different clones.

## Documentation Contract

- `docs/` is current-state truth.
- `plan/active/` records execution state, evidence, deviations, and handoffs.
- Update docs only after evidence exists.
- Archive this plan only after both CI gates pass and review is complete.

## Deviations

- Multi-branch ownership and integration-branch rules are retired.
- CI-summary commits already in `main` are retained as historical evidence.
