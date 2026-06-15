# macOS Implementation and Verification

Owner: macOS implementation agent
Dependencies: Shared contracts frozen
Working branch: `main`
Starting SHA: `d141ac27d493c26fa272b9ba5855a8574ff9d91f`

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

- `P1A` shared-source slice implemented locally from starting SHA `00d91ec20e5b20f04d26fd6fd2661bb4661d8d79` and pushed as `569d229`.
- Local shared checks passed on macOS during `P1A`:
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `npm run check`
  - `npm test`
- `04-verification.md` has been backfilled for the macOS automated checks completed so far.
- Complete `P2` working-tree implementation started from `d141ac27d493c26fa272b9ba5855a8574ff9d91f`.
- Local `P2` shared checks passed:
  - `npm run check`
  - `npm test` (49 tests)
  - `cargo test --manifest-path src-tauri/Cargo.toml` with proxy variables cleared for localhost WireMock (57 tests)
  - `npm run tauri build` (`src-tauri/target/release/bundle/macos/Aura Translation.app`)
- Unit and component coverage confirms profile-scoped secret isolation, legacy provider fallback, backward-compatible history metadata, client-side filters, and explicit retry intent.
- P2 review added regression coverage for plaintext-fallback replay using the original profile key without changing the active profile.

## Deviations and Remaining Risks

- Interactive Keychain migration and end-to-end history replay still require target-host manual verification after review.
