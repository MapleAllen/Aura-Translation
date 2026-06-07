# Windows Implementation and Verification

Owner: Windows implementer
Dependencies: Shared contracts frozen

## Allowed Files

- `src-tauri/src/hotkey.rs`
- `src-tauri/src/lib.rs` only for the macOS-only startup test import cleanup
- `plan/active/macos-v023-integration-hardening/02-windows.md`

## Do Not Modify

- `docs/MacOS-Adaptation/MacOS-Adaptation-Plan.md`
- `docs/macOS-Adaptation-Checklist.md`
- `artifacts/windows-trial/ci-summary.json` as part of product history
- `src-tauri/tauri.macos.conf.json`
- Any macOS-specific runtime behavior outside the warning cleanup already noted above

## Platform-Specific Functions and Behavior

- `platform_default_hotkey_is_parseable` must assert the actual platform key:
  - macOS: `Code::KeyJ`
  - non-macOS: `Code::KeyT`
- Windows default runtime hotkey behavior must remain `CmdOrCtrl+T`.
- Warning cleanup must not weaken or remove the macOS-only startup tests; it only needs to stop Windows from compiling unused imports.

## Implementation Tasks

1. Branch from `codex/macos-v023-integration` into `codex/macos-v023-windows-hardening`.
2. Fix `src-tauri/src/hotkey.rs` so the platform-default parseability test matches the real platform default instead of asserting `KeyJ` everywhere.
3. Clean the macOS-only startup test module imports in `src-tauri/src/lib.rs` so Windows Rust tests stop emitting the known `unused import` warnings.
4. Run the local preflight checks before any CI push:
   - `cargo test --manifest-path .\src-tauri\Cargo.toml`
   - `npm run check`
   - `npm test`
5. Push the branch and wait for the Windows Trial Gate to pass.
6. Capture the passing run URL, SHA, and any CI-summary artifact commit hash in this file. If the workflow adds a summary-only commit, do not merge that commit into the integration branch.

## Automated Verification

Required local commands:

```powershell
cargo test --manifest-path .\src-tauri\Cargo.toml
npm run check
npm test
```

Required CI gate:

- Windows Trial Gate passes on the hardening branch.
- Evidence must include the run URL and the validated branch SHA that contains the product code fix.

## Windows Manual Verification

- If the Rust changes are strictly test-only, manual runtime verification is optional.
- If runtime hotkey logic changes for any reason, perform a spot check on Windows:
  - open Settings,
  - confirm the default hotkey still shows `CmdOrCtrl+T`,
  - trigger a translation once to confirm no regression.

## Completion Evidence

- Commit hash with the test fix and warning cleanup.
- Local command results recorded.
- Windows Trial Gate run URL and outcome recorded.
- Explicit note stating whether a CI-summary-only commit was generated and excluded from final integration.

## Deviations and Remaining Risks

- If the fix requires touching files outside the allowed scope, stop and amend `01-shared-contracts.md` before proceeding.
- If Windows CI fails for a reason unrelated to the hotkey test or warnings, record the new blocker here and escalate for replanning.
