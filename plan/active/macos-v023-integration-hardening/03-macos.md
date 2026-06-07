# macOS Implementation and Verification

Owner: macOS implementer
Dependencies: Shared contracts frozen

## Allowed Files

- `plan/active/macos-v023-integration-hardening/03-macos.md`

## Do Not Modify

- Product source files during the initial verification pass
- `docs/MacOS-Adaptation/MacOS-Adaptation-Plan.md`
- `docs/macOS-Adaptation-Checklist.md`
- `artifacts/windows-trial/ci-summary.json`
- Any shared contract file not explicitly owned by the macOS implementer

## Platform-Specific Functions and Behavior

The macOS verification pass must collect evidence for:

- tray/menu bar icon presence and Settings access
- global hotkey registration and manual translation path
- Keychain-backed API key storage
- native notifications for hidden completion, retry, and failure
- transparent and borderless translation/settings windows
- always-on-top pinned behavior
- unsupported-feature guardrails:
  - Aura automatic clipboard mode remains disabled
  - paste-back controls remain hidden or unavailable

Host evidence is required. CI success alone is insufficient.

## Implementation Tasks

1. Branch from `codex/macos-v023-integration` into `codex/macos-v023-macos-verification`.
2. Check out the branch on a real macOS host using the integrated product code after the Windows fix is merged into the integration branch.
3. Run the automated macOS preflight if the integration branch SHA changed after the previously reported 2026-06-07 pass:
   - `npm ci`
   - `npm run check`
   - `npm test`
   - `cargo test --manifest-path ./src-tauri/Cargo.toml`
   - `npm run tauri build`
4. Execute the checklist in [docs/macOS-Adaptation-Checklist.md](/E:/GitHub/Aura-Translation/docs/macOS-Adaptation-Checklist.md) and record the result of every item in this file with:
   - date and local time,
   - host info,
   - pass/fail,
   - evidence note or screenshot/log path.
5. If any blocking product defect is found, stop the verification-only workflow, document the reproduction here, and request a plan amendment before changing code.

## Automated Verification

- Observe or rerun the `macOS Adaptation Gate` on the integration branch.
- Record the branch SHA and run URL associated with the verified result.
- If no code changed on macOS-sensitive paths after the 2026-06-07 success, note that the earlier passing run is being reused and why.

## macOS Manual Verification

Record one line per item:

- tray/menu bar icon appears after launch
- Settings opens from the tray/menu bar action
- `Cmd+Shift+J` registers and can trigger manual translation from copied text
- API key save/read works through macOS Keychain when system storage is selected
- hidden completion, retry, and failure produce native notifications
- translation and settings windows render transparent and borderless
- pinned mode keeps the translation window above normal app windows
- Aura mode remains disabled on macOS
- paste-back remains hidden or unavailable on macOS

Minimum evidence format:

- `Date:`
- `Host:`
- `Build SHA:`
- `Observed:`
- `Evidence:`

## Completion Evidence

- Completed checklist with pass/fail state for every required macOS runtime behavior.
- macOS host metadata and branch SHA.
- Any screenshots, logs, or reproduction notes referenced from this file.

## Deviations and Remaining Risks

- If Accessibility or Automation permissions block verification, record the exact denial and whether it prevents product sign-off or only scripted inspection.
- Any product bug discovered here becomes a new blocking issue for the integration review.
