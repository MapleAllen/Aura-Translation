# Windows Implementation and Verification

Owner: TBD
Dependencies: Shared contracts frozen
Working branch: `main`
Starting SHA: RESOLVE AND RECORD BEFORE SOURCE EDITS

## Allowed Files

- Shared-contract allowed files from `01-shared-contracts.md`
- Windows-specific tray/icon assets if required
- Windows evidence rows in this plan

## Do Not Modify

- macOS-only clipboard automation and paste-back logic
- unrelated release packaging workflow logic
- `docs/MacOS-Adaptation/*` unless a reviewed regression requires documentation correction

## Platform-Specific Functions and Behavior

- System key storage on Windows must preserve access to existing provider-scoped secrets while enabling profile-scoped secret isolation for new writes.
- Readiness tray state must remain legible with Windows tray/icon constraints.
- Retry-with-original-provider must not silently switch the active profile or alter saved provider credentials.
- Structured startup/load failures must surface as recoverable user-visible notifications without preventing the daemon from booting.

## Implementation Tasks

- Pull latest `main`, confirm a clean worktree, and record the starting SHA before any source edit.
- Implement or verify the Windows side of `P1`:
  - config/profile/history load errors route through `daemon-error`
  - readiness probe cache/invalidation works
  - tray readiness state updates after config changes
- Implement or verify the Windows side of `P2`:
  - profile-scoped secret naming and migration
  - history search/filter UI behavior
  - one-shot retry with original provider/model/base URL
- Leave reviewable source changes and target-host evidence for Codex review.

## Automated Verification

- `npm run check`
- `npm test`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- Windows CI workflow(s) on the final shared SHA

## Windows Manual Verification

- Corrupt `config.json`, `profiles.json`, and `history.json` one at a time; confirm Aura still starts and shows a recoverable structured error for each case.
- Save config changes repeatedly; confirm provider probe button returns a cached result inside the TTL window and re-probes after relevant config changes.
- Toggle between ready and incomplete setup states; confirm tray icon/tooltip reflect readiness changes.
- Start from a machine state that still has provider-scoped secrets; confirm activating/saving a profile can still resolve the old key and persist the new profile-scoped entry.
- Filter history by text/status/language pair; confirm retry with original provider uses the stored entry metadata without switching the active profile.

## Completion Evidence

- Record the implementation handoff SHA.
- Record local command results and any Windows-only screenshots/log notes needed to explain behavior.
- Backfill `04-verification.md` and `05-review-and-integration.md`.

## Deviations and Remaining Risks

- None yet.
