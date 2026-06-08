# Cross-Platform Development Plans

`plan/active/` contains execution contracts for work in progress. `plan/completed/` preserves completed plans. `plan/decisions/` contains architecture decision records.

Rules:

- Current implementation truth belongs in `docs/`, not here.
- All agents collaborate sequentially on `main`; do not use worktrees or implementation branches.
- Each task has one active owner, starting SHA, and allowed-file set.
- The next owner starts only after reviewed work is committed/pushed and they pull the latest `main`.
- Platform behavior requires evidence from its target OS.
- Codex reviews implementation-agent changes before committing and pushing to `main`.
