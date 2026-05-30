# UI Shell Plan

## Objective

Evolve the UI Shell into a production-grade desktop translator that can fluidly switch between two usage modes: quick ephemeral lookup and pinned side-by-side reading. The end state is a shell that opens instantly, survives backend warnings gracefully, supports pinned always-on-top comparison, exposes clear configuration controls, and remains resilient under retries, cancellations, and repeated open/close cycles.

## Design Principles

- **Request-scoped state is mandatory.** Every backend event must be filtered by `request_id` before mutating visible UI state.
- **Pinned mode changes behavior, not structure.** The same window should serve transient translation, pinned comparison, and settings workflows.
- **Frontend config is a cached mirror.** The shell may cache config locally, but Rust remains the source of truth.
- **Notifications carry operational failures.** User-visible daemon and translation failures must surface through the notification layer, not only developer logs.
- **Structural motion stays spring-based.** Popup show/hide behavior should keep using springs; micro-interactions may use CSS transitions.
- **Desktop density beats decorative space.** The shell should prefer compact, legible controls and preserve room for source and translated text.

---

## Phase 1: Core Popup Lifecycle - DONE

Status: **Done**

Goals:

- Ship the main translation popup and its request lifecycle.

Completed work:

- Implemented `+page.svelte` as the route-level orchestrator.
- Implemented `idle | loading | streaming | result | error` state handling.
- Implemented spring-based popup show and hide transitions.
- Implemented `SkeletonLoader`, `TranslationPopup`, and `LanguageSelector`.
- Wired translation events, `Esc` dismissal, and loading timeout handling.

---

## Phase 2: Mid-Stream Control Surface - DONE

Status: **Done**

Goals:

- Let users interrupt or redirect an in-flight request without closing the shell.

Completed work:

- Added request-scoped `currentRequestId` handling.
- Added cancel action in the popup header.
- Added 20-second first-token timeout protection.
- Added mid-stream language switching by cancel-and-restart.
- Ignored stale translation events from cancelled requests.

---

## Phase 3: Dynamic Settings & Provider Sync - DONE

Status: **Done**

Goals:

- Make settings backend-driven instead of static frontend strings.

Completed work:

- Populated models from `config.available_models`.
- Added provider selector backed by `get_provider_defaults`.
- Added live hotkey capture with modifier and key validation.
- Added plaintext API-key warning for authenticated providers.
- Added save success and save failure feedback in the settings overlay.

---

## Phase 4: Pinned Window & Operational Feedback - DONE

Status: **Done**

Goals:

- Support side-by-side comparison and visible daemon feedback without leaving the main shell.

Completed work:

- Added `window_pinned` UI controls in both the popup header and Settings panel.
- Added immediate pin persistence through `save_config`.
- Suppressed blur dismissal when the shell is pinned.
- Added `NotificationCenter` with daemon, hotkey-conflict, and translation-failure notifications.
- Added `mark_ui_ready` startup handshake so lazily created windows can safely receive events.
- Added close action in the header separate from translation cancel.

---

## Phase 5: Resizable Desktop Shell - DONE

Status: **Done**

Goals:

- Allow the shell window to scale beyond the original fixed popup size.

Completed work:

- Enabled `resizable: true` in Tauri window config with min dimensions.
- Added eight invisible edge and corner resize handles in the frontend.
- Wired `startResizeDragging(direction)` to those handles.
- Expanded the shell layout to support larger source/result areas without breaking density.

---

## Phase 6: History & Extended Keyboard UX - NOT STARTED

Status: **Not Started**

Goals:

- Improve recall, keyboard-only operation, and repeat workflows.

Remaining features:

- Add in-memory translation history with restore/replay actions.
- Add keyboard traversal for icon buttons, pin toggle, language controls, and copy action.
- Add focus trapping or predictable tab order when Settings is open.
- Add optional shortcuts for copy, pin toggle, and language swap.

---

## Phase 7: Testing Strategy - PARTIAL

Status: **Partial**

Goals:

- Keep the shell stable across state, settings, and window-behavior changes.

Completed work:

- Added `SettingsPanel.svelte` tests for plaintext API-key warning, hotkey validation, hotkey capture, and window pin persistence.
- Added `TranslationPopup.svelte` tests for configured hotkey rendering and pin-button behavior.
- Added `windowBehavior.ts` tests for blur-dismiss logic.

Remaining features:

- Add tests for `+page.svelte` request lifecycle and notification behavior.
- Add tests for stale-event rejection across rapid request changes.
- Add integration coverage for resize handles and dismiss timing.
- Add end-to-end coverage for tray-triggered translation and settings opening.

## Implementation Rules

- Do not mutate visible translation state from unscoped backend events.
- Do not let pinned mode bypass config persistence; the backend must know the active preference.
- Do not add duplicate error surfaces for the same event; toast plus inline warning is acceptable only when the contexts differ.
- Do not tie resize behavior to CSS-only handles; native window dragging must remain the source of truth.
- Do not add new global state containers unless route-local state becomes demonstrably unmanageable.

## Open Questions

- **History semantics:** Should restoring a history item be read-only, or should it optionally re-run translation with the current provider and model?
- **Retry messaging:** Should the shell show a subtle background retry badge or a more explicit blocking state while the backend retries?
- **Pinned persistence:** Should pin state be global across launches, or should the app optionally remember it per session only?
