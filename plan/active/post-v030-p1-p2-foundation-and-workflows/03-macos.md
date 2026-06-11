# macOS Implementation and Verification

Owner: TBD
Dependencies: Shared contracts frozen
Working branch: `main`
Starting SHA: RESOLVE AND RECORD BEFORE SOURCE EDITS

## Allowed Files

- Shared-contract allowed files from `01-shared-contracts.md`
- macOS tray/icon assets if required
- macOS evidence rows in this plan

## Do Not Modify

- Windows-only acceptance scripts unless a reviewed parity issue requires it
- existing capability-model, NSPasteboard polling, or Accessibility/paste-back behavior outside a reviewed regression fix
- unrelated archived plan files

## Platform-Specific Functions and Behavior

- System key storage on macOS must preserve access to existing provider-scoped Keychain entries while enabling profile-scoped entries for new writes.
- Readiness tray icon/tooltip updates must stay consistent with the existing macOS tray UX.
- Structured load errors must remain recoverable and non-fatal during startup.
- One-shot retry overrides must not mutate the active profile or break current translation-window behavior.

## Implementation Tasks

- Pull latest `main`, confirm a clean worktree, and record the starting SHA before any source edit.
- Implement or verify the macOS side of `P1`:
  - config/profile/history load errors route through `daemon-error`
  - readiness probe cache/invalidation works
  - tray readiness state updates after config changes
- Implement or verify the macOS side of `P2`:
  - profile-scoped secret naming and migration
  - history search/filter UI behavior
  - one-shot retry with original provider/model/base URL
- Leave reviewable source changes and target-host evidence for Codex review.

## Automated Verification

- `npm run check`
- `npm test`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run tauri build`
- macOS CI/build workflow on the final shared SHA

## macOS Manual Verification

- Corrupt `config.json`, `profiles.json`, and `history.json` one at a time; confirm Aura still starts and shows a recoverable structured error for each case.
- Save config changes repeatedly; confirm provider probe button returns a cached result inside the TTL window and re-probes after relevant config changes.
- Toggle between ready and incomplete setup states; confirm tray icon/tooltip reflect readiness changes.
- Start from a Keychain state that still has provider-scoped secrets; confirm activating/saving a profile can still resolve the old key and persist the new profile-scoped entry.
- Filter history by text/status/language pair; confirm retry with original provider uses the stored entry metadata without switching the active profile.
- Confirm no regression in existing macOS Aura mode, paste-back capability, or settings startup flow while P1/P2 changes are present.

## Completion Evidence

- Record the implementation handoff SHA.
- Record local command results and any macOS-only screenshots/log notes needed to explain behavior.
- Backfill `04-verification.md` and `05-review-and-integration.md`.

## Deviations and Remaining Risks

- None yet.
