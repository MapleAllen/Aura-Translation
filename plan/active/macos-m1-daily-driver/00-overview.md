# macos-m1-daily-driver Overview

Created: 2026-09-13
Status: ACTIVE
Coordination owner: implementation owner (single active lock)
Working branch: `main`
Starting SHA: `a6d1d168567bbb1a605fc3ddabafcbda76f9f20c`

## Goal

Deliver milestone **M1: macOS daily-driver build**. Shift the project from feature coverage to
predictable operation, quiet residency, simple first use, and measurable performance.

Product promise: copy text, press the hotkey once to see a translation, press Esc to collapse,
keep working; open, pause automatic translation, or quit from the menu bar at any time.

## Scope

Source of truth: `docs/macOS-M1-Proposal.md`. This plan implements its P0 and P1 items.

- **P0 — predictable toggle and hotkey**: one shared interaction state machine for hotkey,
  visibility, and recall, covering both automatic and manual modes.
- **P0 — menu bar direct actions**: open translation, automatic-translation toggle, settings, quit.
- **P0 — genuinely paused monitoring**: no clipboard access while automatic translation is off.
- **P0 — simple first run**: three-step onboarding (choose service, verify credential, trial translation).
- **P1 — translation-first popup**: translation and copy as the visual subject; context, source,
  and token usage demoted to details.
- **P1 — macOS look and feel**: system font priority, light/dark semantic colors, no hardcoded
  white surfaces, clear focus, reduced-motion support.
- **P1 — macOS acceptance loop**: Mac-executable resource, request-count, and startup measurements.

## Non-Goals

- No OCR, text-selection capture, screenshot translation, paste-back, account sync, bundled model
  downloader, additional providers, or complex workflows.
- No SwiftUI rewrite; Tauri 2 + Rust + Svelte 5 stays.
- No Windows feature expansion. Windows keeps its existing regression checks.
- No dependency-audit remediation in this plan (see Current-State Evidence).
- No new parallel architecture; existing modules are reused and only narrowly extracted.

## Current-State Evidence

- Baseline `a6d1d16` is v0.3.1. Re-verified on this workstation: `npm test` 49/49 passing,
  `cargo test --manifest-path src-tauri/Cargo.toml --locked` 57/57 passing, `npm run check`
  passing, `npm run build` passing. Node v24.18.0, npm 11.16.0.
- `lib.rs::handle_hotkey_pressed` applies visibility toggling only when `aura_mode_enabled` is
  true; manual mode calls `trigger_translation` directly, and `trigger_translation` has no
  same-text guard. Repeating the hotkey in manual mode therefore re-issues a paid request.
- `lib.rs::build_tray_menu` builds four entries (Profiles submenu, Settings, separator, Quit).
  There is no "open translation" item and no automatic-translation toggle.
- `lib.rs::spawn_clipboard_monitor` reads `NSPasteboard::changeCount()` and writes
  `last_sequence` *before* checking `aura_mode_enabled`, so the loop and state writes continue
  while automatic translation is off.
- `lib.rs::should_show_settings_on_startup` infers first run from
  `settings_window_placement.is_none()`. There is no explicit onboarding field anywhere in the
  repository (`onboard`, `first_run`, `firstRun` have zero matches).
- Escape handling exists only in `SettingsWindowView.svelte`; the translation window has none.
- `lib.rs` builds the HTTP client with `reqwest::Client::new()`, so no timeout is configured. The
  frontend's 20s timer only covers the `loading` state.
- `ui/app.css` sets Microsoft YaHei first in `--font-body`, uses fixed light colors, and sets
  `.aura-glass-panel { border-radius: 0 }`. `prefers-color-scheme` has zero matches. 45 hardcoded
  `bg-white/*` occurrences remain across components.
- `TranslationWindowView.svelte` passes `showComposer={config.window_pinned}`, so direct source
  editing is coupled to pinning.
- `lib.rs` has 3 unit tests (`startup_tests`, `p2_tests`), none of which cover the hotkey,
  monitoring, or menu decision paths. This is the concrete blocker for the proposal's
  "verify with a mock service counter" acceptance method.
- **Dependency audit measured on this workstation**: `npm audit --json` reports
  `metadata.vulnerabilities = { low: 1, moderate: 8, high: 5, critical: 2, total: 16 }`.
  All 16 entries report `No fix available`. 8 of them are direct dependencies
  (`@sveltejs/adapter-static`, `@sveltejs/kit`, `@sveltejs/vite-plugin-svelte`,
  `@tailwindcss/vite`, `@testing-library/svelte`, `jsdom`, `vite`, `vitest`); the other 8 are
  transitive. Every entry sits in devDependencies or the build toolchain, not in the shipped
  application. The proposal's "8 affected packages (1 critical, 4 high, 2 moderate, 1 low)" does
  not match any measurement basis and is corrected in the proposal document by this plan.
  Remediation requires a major-version migration with no automatic path, so it is accepted for
  M1 and tracked as an independent maintenance item.

## Invariants

- `docs/` remains current-state truth; `plan/` records temporary execution intent only.
- All work proceeds sequentially on `main`; no implementation branches or worktrees.
- Windows behaviour must not regress: hotkey, tray, Aura mode, paste-back, and installer checks
  stay intact.
- No plaintext API key may be written to `config.json` while storage mode is `system`.
- Existing configuration, profile, and history files must keep loading without migration steps.
- Every new interaction rule must be assertable by a test that does not construct a Tauri
  `AppHandle`.
- Existing streaming, cancellation, and retry semantics in `translate.rs` are reused, not rewritten.

## Dependencies

- Existing request/streaming/cancellation layer in `src-tauri/src/translate.rs`
- Existing tray, hotkey, window, and monitor wiring in `src-tauri/src/lib.rs`
- Existing config persistence in `src-tauri/src/config.rs`
- Existing readiness probe and cache in `src-tauri/src/readiness.rs`
- Existing UI contracts in `ui/lib/` and semantic tokens in `ui/app.css`
- macOS CI workflow `.github/workflows/macos-adaptation.yml`

## Risks and Unknowns

- Extracting the interaction state machine from `lib.rs` touches the hottest path in the app.
  Mitigation: the extraction is additive and pure; behaviour changes land only after its unit
  tests exist.
- Switching the tray to show its menu on left click changes an established interaction. The
  previous "left click recalls the translation" behaviour is preserved as an explicit menu item.
- `NSPasteboard` has no general-purpose change notification, so the enabled path keeps polling.
  Only the disabled path is required to be free of clipboard access.
- Dark mode is a broad visual change across 45 hardcoded surfaces. Mitigation: token substitution
  only, no layout changes, verified by component tests and a real-host pass.
- The proposal's "5 consecutive working days" and "3 unread first-time users" acceptance rows are
  not automatable. They are recorded as trial evidence, never as a CI gate.
- Intel Macs and the minimum supported macOS version remain unverified; they must be settled
  before any public distribution.

## Acceptance Criteria

- Same text with the window visible collapses the window and issues no new request; same text with
  the window hidden recalls the existing result and issues no new request; new text issues exactly
  one request. Verified by pure-function unit tests in CI.
- Request reuse is keyed by text, language direction, provider, model, base URL, and profile
  identity, so changing translation settings never reuses a stale result.
- The menu bar exposes open-translation, automatic-translation toggle, settings, and quit, and the
  toggle reflects live configuration state.
- With automatic translation off, zero `NSPasteboard` calls occur and re-enabling does not translate
  text copied while it was off.
- A three-step first run ends with a persisted `setup_completed` flag, and existing users are never
  re-onboarded.
- Escape collapses the translation window once and does not interrupt IME composition.
- The popup presents translation and copy as the primary affordances with context and source
  available as details.
- The UI uses system font priority and follows the system light/dark appearance.
- Mac-executable acceptance scripts produce resource, request-count, and startup evidence.
- `npm run check`, `npm test`, and `cargo test` pass on the final SHA, and macOS CI is green.

## Main-Branch Handoff Sequence

1. Record shared contracts and allowed files in `01-shared-contracts.md`.
2. Implement the interaction state machine and shell changes (`02-state-machine-and-shell.md`).
3. Implement first-run and experience changes (`03-first-run-and-experience.md`).
4. Produce macOS acceptance tooling and target-host evidence (`04-macos-acceptance.md`).
5. Backfill `05-verification.md`, update `docs/` to the implemented state, and archive this folder.
