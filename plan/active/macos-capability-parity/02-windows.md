# Windows Implementation and Verification

Owner: Windows verifier
Dependencies: Shared contracts frozen; macOS implementation merged to `main`
Working branch: `main`
Starting SHA: RESOLVE AND RECORD BEFORE SOURCE EDITS

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

- [ ] Starting SHA recorded before source edits.
- [ ] Windows automated checks pass.
- [ ] Windows manual verification evidence recorded.
- [ ] Windows Trial Gate passes on the shared final SHA.
- [ ] Any Windows regression fix stayed within the allowed-file set and was reviewed before push.

## Deviations and Remaining Risks

- None yet.
