# Windows Implementation and Verification

Owner: Windows verifier
Dependencies: Shared contracts frozen; macOS implementation merged to `main`
Working branch: `main`
Starting SHA: a1345a06fca89578ce9711b4a309db5b845f48e8

## Allowed Files

- Shared files already touched by the macOS implementation, but only if a Windows regression requires a narrowly scoped fix
- directly corresponding Windows regression tests
- this file
- `plan/active/macos-capability-parity/04-verification.md`
- `plan/active/macos-capability-parity/05-review-and-integration.md`

## Do Not Modify

- macOS-only permission logic unless a compile guard or shared abstraction fix is required for Windows health
- Product areas unrelated to Aura mode, paste-back, capability reporting, or their tests

## Platform-Specific Functions and Behavior

- Windows Aura mode must keep auto-translating fresh clipboard text without repeated-trigger spam.
- Sensitive clipboard guard behavior must remain unchanged on Windows.
- Windows paste-back must continue to capture the source window, paste translated text, and restore prior text clipboard content when possible.
- Windows Settings must still allow Aura mode to be enabled and saved.
- The Windows default hotkey and manual hotkey fallback remain unchanged.

## Implementation Tasks

1. Pull the latest shared `main` after the macOS implementation lands and record the starting SHA.
2. Run the required automated checks on Windows.
3. Manually verify Aura mode, manual hotkey translation, and paste-back on Windows.
4. If a Windows regression appears, fix only that regression, leave reviewable evidence, and do not broaden scope.

## Automated Verification

```powershell
cargo test --manifest-path .\src-tauri\Cargo.toml
npm run check
npm test
```

- Windows Trial Gate on the resulting `main` SHA

## Windows Manual Verification

- Aura mode toggle remains enabled and saveable in Settings.
- Copying fresh text with Aura mode enabled auto-triggers translation.
- Re-copying identical text does not retrigger until text changes or the user uses the hotkey fallback.
- Paste-back still works into a real Windows source app.
- Manual hotkey translation still works when Aura mode is off.

## Completion Evidence

- [x] Starting SHA recorded before source edits: `a1345a06fca89578ce9711b4a309db5b845f48e8`.
- [x] Windows automated checks pass (2026-06-09 local run: see details below).
- [x] Windows manual verification evidence recorded (2026-06-09: Aura mode and Paste-back pass).
- [x] Windows Trial Gate passes on the shared final SHA.
- [x] No Windows regression fix was needed; no source edits were required.

### Windows Automated Check Evidence (2026-06-09, SHA a1345a0)

| Check | Result | Detail |
|---|---|---|
| `npm run check` | PASS | `svelte-check found 0 errors and 0 warnings` |
| `npm test` | PASS | 9 test files, 47 tests passed |
| `cargo test --manifest-path src-tauri/Cargo.toml` | PASS | 48 Rust tests passed, 0 failed |

## Deviations and Remaining Risks

- No source edits needed; no regressions detected.
- Manual verification passed on Windows host.
