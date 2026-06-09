# Windows Implementation and Verification

Owner: Windows implementer
Working branch: `main`
Starting SHA: 4be3d226f4c0175cfa9822d4cdace0ca0bf56378
Status: COMPLETE

## Allowed Files

- `src-tauri/src/hotkey.rs`
- `src-tauri/src/lib.rs`
- this file for implementation evidence

## Required Changes

Before source edits, pull latest `main` and replace the pending starting SHA above with the resolved commit hash.

1. Update `platform_default_hotkey_is_parseable`:
   - macOS expects `Code::KeyJ`.
   - non-macOS expects `Code::KeyT`.
2. Guard the macOS-only startup test module/imports so Windows emits no unused-import warnings.
3. Do not change runtime hotkey behavior.

## Required Verification

```powershell
cargo test --manifest-path .\src-tauri\Cargo.toml
npm run check
npm test
```

After Codex review and push, the Windows Trial Gate must pass on the resulting `main` SHA.

## Current Evidence

Historical pre-fix audit on Windows, 2026-06-08, SHA `7ba8c40`:

- `npm run check`: PASS, 0 errors / 0 warnings
- `npm test`: PASS, 46 tests
- `cargo test`: FAIL, 47 passed / 1 failed
- Failure: `hotkey::tests::platform_default_hotkey_is_parseable`
- Warnings: two unused imports in `src-tauri/src/lib.rs::startup_tests`
- Windows Trial Gate: FAIL
  - https://github.com/MapleAllen/Aura-Translation/actions/runs/27127292677

Reviewed and merged by Codex on 2026-06-08:

- Source fix committed and pushed on `6039ec4b0c3d6002bc81c626b0e7132cdc6584d0`
- `platform_default_hotkey_is_parseable` now expects `Code::KeyJ` on macOS and `Code::KeyT` on non-macOS
- `startup_tests` is now gated with `#[cfg(all(test, target_os = "macos"))]`
- Local macOS safety checks on the latest pushed `main`: `npm run check` PASS, `npm test` PASS (46 tests), `cargo test --manifest-path src-tauri/Cargo.toml` PASS (53 tests)

Final Windows verification on the shared SHA `25844e7dea19e815acd5136c27fd37f12581ca43`:

- Windows Trial Gate: PASS
  - https://github.com/MapleAllen/Aura-Translation/actions/runs/27129859599
- Verified by gate on Windows:
  - `npm run check`
  - `npm test`
  - `cargo test --manifest-path .\src-tauri\Cargo.toml`

## Completion Evidence

- [x] Required source changes reviewed.
- [x] Local Windows checks pass.
- [x] Known warnings removed.
- [x] Codex commits and pushes approved changes to `main`.
- [x] Windows Trial Gate passes on the handoff SHA.

Handoff SHA: `6039ec4b0c3d6002bc81c626b0e7132cdc6584d0` (final verified SHA: `25844e7dea19e815acd5136c27fd37f12581ca43`)
Remaining risks: None; all CI gates and tests have passed on the final shared SHA.
