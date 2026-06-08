# macOS Implementation and Verification

Owner: macOS verifier
Working branch: `main`
Status: COMPLETE - FINAL SHARED SHA GATE PENDING

## Verified Behavior

Date: 2026-06-08
Host: local macOS desktop (`aarch64-apple-darwin`)
Verified product baseline: v0.2.3 behavior merged into `main`
Evidence source: `docs/macOS-Adaptation-Checklist.md`

- PASS: app launches and stays resident in the menu bar.
- PASS: Settings auto-opens for first launch/incomplete setup.
- PASS: tray/menu bar interaction and Settings access.
- PASS: `Cmd+Shift+J` global hotkey and manual translation flow.
- PASS: Keychain save/read/delete.
- PASS: native notifications for completion, retry, and failure.
- PASS: transparent/borderless windows and close/reopen behavior.
- PASS: pinned always-on-top behavior.
- PASS: Aura mode remains disabled.
- PASS: paste-back remains unavailable.

Known host limitation:

- Scripted `System Events` inspection remains blocked by Accessibility denial `-25211`; direct manual verification completed the required checks.

## CI Evidence

- macOS Adaptation Gate passed on current `main` SHA `7ba8c40`:
  - https://github.com/MapleAllen/Aura-Translation/actions/runs/27127293378

## Remaining Work

- After the Windows fix reaches `main`, rerun or confirm the macOS Adaptation Gate on the new SHA.
- If macOS-sensitive source changes are introduced, repeat affected manual checks before sign-off.
- Synchronize final gate SHA into `04-verification.md`.

## Completion Evidence

- [x] Required macOS runtime behaviors manually verified.
- [x] Current `main` macOS gate passed.
- [ ] Final shared `main` SHA passes both Windows and macOS gates.
