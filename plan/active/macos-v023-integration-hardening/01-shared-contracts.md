# Shared Contracts

## Ownership

Owner: Shared implementer for plan/docs status; Windows implementer for the shared Rust files listed below
Dependencies: `00-overview.md` frozen and integration baseline created

Shared file ownership freeze for this effort:

- Windows implementer owns:
  - `src-tauri/src/hotkey.rs`
  - `src-tauri/src/lib.rs` only for warning cleanup in the startup test module
- macOS implementer owns:
  - `plan/active/macos-v023-integration-hardening/03-macos.md`
- Shared implementer owns:
  - `plan/active/macos-v023-integration-hardening/00-overview.md`
  - `plan/active/macos-v023-integration-hardening/01-shared-contracts.md`
  - `plan/active/macos-v023-integration-hardening/04-verification.md`
  - `plan/active/macos-v023-integration-hardening/05-review-and-integration.md`
  - `docs/MacOS-Adaptation/MacOS-Adaptation-Plan.md`
  - `docs/macOS-Adaptation-Checklist.md`

## Allowed Files

- `plan/active/macos-v023-integration-hardening/*.md`
- `docs/MacOS-Adaptation/MacOS-Adaptation-Plan.md`
- `docs/macOS-Adaptation-Checklist.md`
- `src-tauri/src/hotkey.rs`
- `src-tauri/src/lib.rs`

## Do Not Modify

- `artifacts/windows-trial/ci-summary.json` as a product change
- `scripts/release/write-ci-summary.ps1`
- `.github/workflows/windows-trial.yml` unless a new workflow defect is proven and the plan is amended first
- `.github/workflows/macos-adaptation.yml` unless a new workflow defect is proven and the plan is amended first
- `src-tauri/tauri.macos.conf.json`
- `main`

## Public Interfaces and Critical Functions

`src-tauri/src/hotkey.rs`

- `pub fn default_hotkey() -> &'static str`
  - Responsibility: expose the platform default hotkey string.
  - Compatibility: macOS returns `Cmd+Shift+J`; non-macOS returns `CmdOrCtrl+T`.
  - Callers: config defaults, startup registration, tests.

- `pub fn normalize_persisted_hotkey(hotkey: &str) -> String`
  - Responsibility: migrate legacy persisted defaults on macOS while preserving user-entered custom hotkeys.
  - Compatibility: only the known legacy defaults should normalize on macOS; non-macOS behavior stays unchanged.

- `pub fn parse_hotkey(s: &str) -> Result<Shortcut, String>`
  - Responsibility: parse persisted or default accelerator strings into the Tauri shortcut type.
  - Error contract: returns `Err(String)` for empty input, missing modifiers, unknown modifiers, or unsupported key fragments.
  - Compatibility: parsing behavior must continue to support Windows `CmdOrCtrl+T` and macOS `Cmd+Shift+J`.

`src-tauri/src/lib.rs`

- `fn register_startup_hotkey(app: &AppHandle)`
  - Responsibility: register the configured hotkey at startup and fall back to `hotkey::default_hotkey()` if parsing or registration fails.
  - Side effects: emits daemon errors and registers global shortcuts with the Tauri plugin.
  - Compatibility: this hardening effort must not change startup hotkey behavior beyond keeping it aligned with the platform default contract above.

- `fn should_show_settings_on_startup(config: &AppConfig) -> bool`
  - Responsibility: decide whether macOS first-launch/incomplete-setup flow should surface Settings automatically.
  - Compatibility: current `v0.2.3` behavior remains unchanged; only the test module warning cleanup is in scope.

Documentation contract:

- `docs/MacOS-Adaptation/MacOS-Adaptation-Plan.md` must reflect that Phase 2 macOS CI succeeded on 2026-06-07 and that Phase 3 is at least partially exercised, with any remaining gaps called out explicitly.
- `docs/macOS-Adaptation-Checklist.md` must separate verified items from pending items and preserve the rule that CI is not runtime proof.
- `04-verification.md` is the canonical execution-time matrix for branch-level evidence.

## Tasks

1. Freeze the integration baseline and file ownership before any implementation branch begins work.
2. Keep the shared contract stable while Windows and macOS work proceed in parallel.
3. Update shared documentation only after evidence exists; do not guess or backfill unverified behavior.
4. Record any scope change or newly discovered blocking issue in this file or `05-review-and-integration.md` before additional implementation continues.

## Automated Verification

- Verify the baseline ancestry locally:

```powershell
git merge-base --is-ancestor ebe2162 6a826fb
```

- Verify only the intended product commits are on the integration branch before final merge:

```powershell
git log --oneline main..codex/macos-v023-integration
```

## Completion Evidence

- Integration baseline branch exists and points to `v0.2.3` plus reviewed hardening commits only.
- Shared ownership boundaries were respected.
- Shared docs and verification matrix were updated from real evidence, not assumptions.

## Deviations

- None at plan creation time.
