# post-v030-p1-p2-foundation-and-workflows Overview

Created: 2026-06-11
Status: COMPLETE - P1/P2 CLOSED ON SHARED `main`; WINDOWS INTERACTIVE VERIFICATION DEFERRED BY FOLLOW-UP
Coordination owner: Codex coordinator
Working branch: `main`

## Goal

Execute the first two post-`v0.3.0` priorities sequentially on `main` with one active owner at a time.

- `P1`: trust, diagnosability, and readiness hardening across config, profiles, history, and runtime-status surfaces.
- `P2`: operator workflow depth across profile-scoped credentials and translation-history recall flows.

## Scope

- `docs/Config-And-Secrets/Config-And-Secrets-Plan.md`
  - Phase 1: structured error routing
- `docs/Runtime-Readiness/Runtime-Readiness-Plan.md`
  - Phase 1: probe result caching and debounce
  - Phase 2: tray icon readiness badge
- `docs/Translation-Profiles/Translation-Profiles-Plan.md`
  - Phase 1: structured error routing
  - Phase 2: per-profile independent API keys
- `docs/Translation-History/Translation-History-Plan.md`
  - Phase 1: structured error routing
  - Phase 2: search and filter
  - Phase 4: retry with original provider

## Non-Goals

- Do not implement schema versioning, window-placement validation, or Linux keyring support in this plan.
- Do not implement profile reorder, profile import/export, translation-bubble quick-switch, or history export in this plan.
- Do not implement Aura Guard notifications, allow-lists, entropy scoring, or token-registry expansion in this plan.
- Do not change translation-engine streaming semantics unless required for the retry-override API contract.
- Do not create implementation branches or parallel host edits.

## Current-State Evidence

- `docs/*-Description.md` and `docs/*-Plan.md` for Config And Secrets, Runtime Readiness, Translation Profiles, and Translation History were added on `4fe5f24`.
- `00d91ec` established the cross-platform execution contract and locked `P1A` to sequential main-branch implementation.
- `P1A` now routes config/profile/history startup load failures through `daemon-error` plus background notifications while preserving fallback startup behavior.
- `P1A` also adds a 30-second readiness probe cache keyed by provider/base URL/model/hydrated API key and keeps the tray tooltip summary aligned with the current readiness state.
- `569d229` is the pushed `P1A` runtime handoff commit, and `89a7bf2` records the handoff status without further source changes.
- `19a1bad` is the reviewed `P2` shared-source handoff commit, and `98c20a2` is the pushed plan-sync baseline on shared `main`.
- Shared CI passed on the same final SHA `98c20a2` for both Windows and macOS workflows.
- Windows interactive verification is explicitly deferred because no Windows host is currently available.
- macOS target-host verification has confirmed first-launch Settings behavior plus isolated history search, status, and language-pair filters on the built app bundle.
- Existing regression coverage already touches the target surface:
  - `ui/lib/SettingsPanel.test.ts`
  - `ui/lib/ProfileManager.test.ts`
  - `ui/lib/HistoryList.test.ts`
  - `src-tauri/src/history.rs` unit tests

## Invariants

- `docs/` remains current-state truth; `plan/` records temporary execution intent only.
- All work proceeds sequentially on `main`.
- No plaintext API key may be written to `config.json` while storage mode is `system`.
- `TranslationProfilesStore::delete()` must continue to enforce the minimum-of-one-profile rule.
- Translation history remains newest-first, backward-compatible, and atomically persisted.
- Readiness inspection functions remain synchronous and side-effect free.
- Existing Windows and macOS hotkey, tray, and Aura behavior must not regress while P1/P2 are implemented.

## Dependencies

- Existing daemon event surface in `src-tauri/src/lib.rs`, especially `emit_daemon_error()` and `emit_config_updated()`
- Existing keyring-backed secret persistence in `src-tauri/src/secrets.rs`
- Existing tray/menu refresh flow in `src-tauri/src/lib.rs`
- Current Settings, Profile Manager, History List, and Setup Status Card UI contracts in `ui/lib/`
- Windows and macOS CI workflows plus target-host manual checks

## Risks and Unknowns

- The repository has no literal `P1`/`P2` labels; this plan interprets them per ADR-0001 and must be amended before source edits if that interpretation is wrong.
- Per-profile key migration crosses OS credential stores and must preserve access to legacy provider-scoped entries on both Windows and macOS.
- Tray readiness badges may need separate bundled assets per host if programmatic overlays prove inconsistent.
- Retry-with-original-provider needs a narrow override contract that does not accidentally mutate the active profile.

## Acceptance Criteria

- Config, profile, and history load failures surface through the structured `daemon-error` path without changing fallback behavior.
- Provider probe requests are cached/debounced and invalidated when relevant config fields change.
- Tray readiness tooltip reflects ready vs needs-setup state on both Windows and macOS, and icon-badge follow-up work is tracked explicitly.
- Profiles can resolve provider secrets through a profile-scoped system-store naming contract while preserving legacy-entry fallback during migration.
- History UI supports client-side search and filters, and users can retry an entry with its original provider/model/base URL without switching the active profile.
- Shared checks and both CI gates pass on the same final `main` SHA.
- `docs/` is updated to reflect the implemented current state before archiving this plan.
- Required macOS target-host evidence is recorded for closure, and the deferred Windows/manual follow-up is called out explicitly before archive.

## Main-Branch Handoff Sequence

1. Freeze shared contracts and record the active lock in `01-shared-contracts.md`.
2. One implementation owner pulls latest `main`, records the starting SHA, and implements the approved `P1` shared-source slice.
3. Codex reviews, commits, and pushes the approved `P1` stage to `main`.
4. Windows and macOS verification owners pull the new `main`, run assigned target-host checks, and record evidence.
5. One implementation owner pulls latest `main`, records the new starting SHA, and implements the approved `P2` shared-source slice.
6. Codex reviews, commits, and pushes the approved `P2` stage to `main`.
7. Windows and macOS verification owners confirm final shared checks and target-host behavior on the same SHA.
8. Codex completes review, updates `docs/`, and moves this folder to `plan/completed/`.
