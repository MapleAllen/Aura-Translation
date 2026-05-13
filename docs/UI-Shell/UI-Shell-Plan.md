# UI Shell Plan

## Objective

Evolve the UI Shell from a single-state popup into a polished, production-grade desktop widget that feels like a natural extension of the host operating system — not a browser tab. The end state is a popup that: (1) springs open with a physically-grounded haptic overshoot, (2) supports in-flight cancellation and live language switching, (3) offers a configurable hotkey binding, (4) maintains a session-level history panel, and (5) is fully navigable by keyboard — all while remaining sub-second in perceived response time, consuming ≤20 MB idle RAM, and never adding a Taskbar or Dock footprint. Every structural animation must be driven by spring physics, not linear easing.

## Design Principles

- **State machine is the source of truth.** `appState` drives all conditional rendering; no component may alter its own visibility based on internal flags that conflict with `appState`.
- **Spring physics preferred.** Prefer `svelte/motion.Spring` for the popup entry/exit so the damping can be tuned low enough to allow a brief overshoot past `scale(1.0)` — this haptic "pop" is a core product requirement, not an aesthetic preference. CSS `transition` is acceptable for structural animations where overshoot is not needed (springs run JS per frame and are not inherently lighter than GPU-composited CSS transitions); CSS `transition` is the correct choice for hover micro-interactions (color, border).
- **Props flow down, events flow up.** Components receive state as props and emit changes via callback props (`onLanguageChange`, `onclose`); no component imports or writes to global state.
- **Config round-trips are avoided where obvious.** Cache `get_config` results in orchestrator state for the duration of a trigger cycle; do not call it in a loop. Re-reads for post-save verification are acceptable — a single IPC call to a local JSON file costs < 1 ms and is not a bottleneck.
- **Text selectability is intentional.** Only source text and result text carry `select-text`; the rest of the UI is `user-select: none`.
- **Errors are never silent.** Every `catch` block that handles a Tauri API call must either surface an `errorMessage` state or emit a `console.error` with a structured context object.
- **RTL-safe layout from Phase 3.** All flex layouts must use `gap` + `align-items` rather than margin hacks so that RTL language support can be added without layout surgery.

---

## Phase 1: MVP Polish — DONE

Status: **Done**

Goals:

- Ship the complete glassmorphic popup with spring animation, streaming text, skeleton loader, language selector, copy button, and settings panel.

Completed work:

- Implemented `+page.svelte` as the lifecycle orchestrator with eight Tauri event listeners.
- Implemented `Spring`-based scale/opacity popup animation (scale `0.92→1`, opacity `0→1`), anchored bottom-right.
- Implemented `appState` union (`idle | loading | streaming | result | error`) driving conditional rendering.
- Implemented `TranslationPopup.svelte` with drag region, status indicator, source text box, divider, content area, and copy footer.
- Implemented `SkeletonLoader.svelte` with 4 shimmer bars and staggered animation delays.
- Implemented `LanguageSelector.svelte` with 11 languages, swap button with rotation spring, and `auto` guard.
- Implemented `SettingsPanel.svelte` with API key (password + toggle), model dropdown, hotkey display, and save button with spring bounce.
- Defined Aura design token system in `app.css` via Tailwind v4 `@theme`.
- Wired `Esc` dismiss, `window-blur` auto-dismiss (suppressed during Settings), and `show-settings` tray event.

---

## Phase 2: Cancellation & Mid-Stream Controls — DONE

Status: **Done**

Goals:

- Allow the user to cancel an in-progress translation and start a new one.

Completed work:

- Added a cancel button (× icon) in the `TranslationPopup` header area, visible in `loading` and `streaming` states; styled with `hover:text-aura-error` transition.
- Added `oncancel` callback prop to `TranslationPopup`; wired to `handleCancel()` in the orchestrator.
- On cancel click: invokes `cancel_translate({ requestId })` (Rust Phase 2), then transitions `appState` to `result` if partial text exists, or `idle` if no text arrived.
- Added a monotonically incrementing `currentRequestId` counter in the orchestrator; passed to `invoke('translate_text', …)` on every new translation.
- Added a 20-second loading timeout in `+page.svelte`: if `appState === 'loading'` after 20 s with no `translation-chunk` event, sets `appState = 'error'` and `errorMessage = 'No response from API (timeout)'`.
- Timeout is cleared on first current-request chunk, on current-request `translation-done`, on current-request `translation-error`, and on dismiss.
- On language change while `loading` or `streaming`: invokes `cancelCurrentTranslation()`, resets state, then calls `startTranslation()` immediately with the new language pair.
- Extracted `startTranslation()` as a reusable function for both trigger and mid-stream language switch flows.
- Guarded `translation-chunk`, `translation-done`, `translation-error`, and `translation-retry` listeners by `request_id === currentRequestId` to prevent stale events from cancelled requests from corrupting the display.
- New triggers cancel any in-flight translation before starting a new one.

---

## Phase 3: Dynamic Config & Model List — DONE

Status: **Done**

Goals:

- Remove all hardcoded strings from the Settings panel and replace them with data from `AppConfig`.

Completed work:

- Replaced hardcoded `<option>` elements in `SettingsPanel.svelte` with a `{#each config.available_models}` loop.
- Replaced the static hotkey `<kbd>` display with a live binding capture widget (`<input>` with `keydown` listener that formats modifier+key into `CmdOrCtrl+T`-style string; uses `e.preventDefault()` + `e.stopImmediatePropagation()` to prevent global shortcuts while focused).
- Added a `Provider` dropdown that auto-populates `api_base_url` and `available_models` on change by invoking Rust `get_provider_defaults`, keeping provider defaults centralized in `config.rs`.
- API base URL is configurable via `config.api_base_url` (defaults populated from provider selection).

---

## Phase 4: History Panel — NOT STARTED

Status: **Not Started**

Goals:

- Let the user review and reuse translations from the current session.

Remaining features:

- Add a `history: Array<{ sourceText, translatedText, sourceLang, targetLang, timestamp }>` array to the orchestrator state, capped at 20 items.
- Append a history entry whenever `appState` transitions to `result`.
- Add a history toggle button in the `TranslationPopup` header (clock icon); toggles a `showHistory` boolean.
- Render the history as a scrollable list within the popup content area when `showHistory === true`, replacing the current content area.
- Each history item is clickable; clicking restores `sourceText` and `translatedText` to the displayed values (read-only replay, not a re-translation).
- Add a "clear history" button at the top of the history panel.
- History is in-memory only — it does not persist across restarts.

---

## Phase 5: Accessibility & RTL Support — NOT STARTED

Status: **Not Started**

Goals:

- Make the popup usable via keyboard alone and correctly render RTL target languages.

Remaining features:

- Add `tabIndex` and `aria-label` attributes to remaining icon-only interactive elements; the settings close button and API-key visibility toggle now have explicit `aria-label` attributes.
- Add keyboard navigation: `Tab` cycles through interactive elements within the popup; `Enter` activates the focused button.
- Detect RTL target languages (Arabic) and apply `dir="rtl"` to the result text container.
- Apply `text-align: right` and `font-size: 0.9em` (Arabic script optical size adjustment) to the result text when target is RTL.
- Replace `{#if visible}` in `SettingsPanel.svelte` with `display: none` (`class:hidden`) to avoid remounting the component on every open/close.

---

## Phase 6: Testing Strategy — NOT STARTED

Status: **Not Started**

Goals:

- Establish regression coverage for state transitions, event handling, and component rendering.

Remaining features:

- Add Vitest unit tests for `LanguageSelector.svelte`: verify swap is blocked when `sourceLang === 'auto'`; verify correct `onchange` calls for both selects.
- Add Vitest unit tests for `+page.svelte` state machine: mock Tauri event listeners and verify state transitions for all five events.
- Add a Playwright integration test (via webapp-testing skill) for the full translate flow: trigger → loading → streaming → result → copy.
- Add a Playwright test for dismiss: trigger → streaming → press Esc → window hidden.
- Add a Playwright test for settings: open via tray → change API key → save → reload and verify.

---

## Implementation Rules

- Prefer merging `{#if viewState === 'streaming'}` and `{#if viewState === 'result'}` into a single `{#if streaming || result}` branch to avoid template duplication — the compiled Svelte output is identical either way, so this is a code-style preference, not a performance constraint.
- Do not add CSS `transition` to elements that also use a `Spring` — double animation causes visual jitter.
- Do not call `invoke('get_config')` more than once per translation trigger — cache in orchestrator state.
- Do not leave verbose debug logging in production paths. Use `console.warn` / `console.error` for meaningful diagnostics. Guard development-only `console.log` calls behind a `DEV` flag or remove them before merging — `console.log` and `console.error` have identical runtime cost; the concern is noisy output, not resource usage.
- Do not mutate `translatedText` directly from multiple event handlers without guarding `appState` — check `appState !== 'error'` before appending chunks.

## Open Questions

- **History persistence:** Should translation history survive a window hide/show cycle (in-memory) or also survive a process restart (disk)? In-memory is simpler; disk requires a new Tauri command. Decide before Phase 4.
- **RTL font:** Arabic script renders poorly with DM Sans at small sizes. Should a separate Arabic-optimized font (e.g., Noto Sans Arabic) be loaded conditionally? Decide before Phase 5.
- **Settings remount cost:** Does the `{#if visible}` teardown + `get_config` round-trip cause a perceptible flash when opening Settings on slow machines? Measure before deciding to change to `display: none` in Phase 5.

## Resolved Questions

- **Cancel UX:** Resolved in Phase 2: cancelling preserves partial translated text and transitions to `result` state; if no text has arrived, transitions to `idle`.
- **Mid-stream language switch:** Resolved in Phase 2: changing language while streaming cancels the current request and immediately starts a new translation with the updated language pair.
