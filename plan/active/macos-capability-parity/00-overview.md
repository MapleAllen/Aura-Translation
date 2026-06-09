# macos-capability-parity Overview

Created: 2026-06-09
Status: ACTIVE - MACOS IMPLEMENTATION ASSIGNED
Coordination owner: Codex coordinator
Working branch: `main`

## Goal

Bring the remaining macOS-blocked desktop capabilities to a supported, reviewable state on the shared `main` branch: automatic Aura-mode clipboard translation, source-app paste-back, and the capability/permission UX needed to expose both features truthfully without regressing Windows behavior.

## Scope

- Replace the current Windows-only frontend gating with a backend-driven capability model that can represent macOS support, availability, and permission state.
- Implement macOS Aura-mode clipboard monitoring so copied text can auto-trigger translation when the feature is enabled.
- Implement a macOS paste-back path that can capture the source app, focus it again, paste the translated text, and surface actionable errors when permission or focus preconditions fail.
- Update Settings and translation-window UI so macOS no longer hardcodes Aura mode as unsupported once the backend supports it.
- Add or update shared, Windows, and macOS verification evidence plus current-state docs.

## Non-Goals

- Do not add Linux parity in this plan.
- Do not redesign tray UX, translation history access, or provider flows.
- Do not broaden paste-back beyond text clipboard payloads in this plan.
- Do not publish a new release or change tagging/release automation as part of this plan.
- Do not replace the clipboard polling strategy unless macOS implementation proves polling is insufficient.

## Current-State Evidence

- `docs/UI-Shell/UI-Shell-Description.md` states Aura mode is Windows-only and macOS users are forced onto the manual hotkey flow.
- `docs/Daemon-Core/Daemon-Core-Description.md` states paste-back is Windows-only and non-Windows builds reject the command as unsupported.
- `src-tauri/src/lib.rs` currently compiles `paste_translation_back(...)` only for Windows behavior and makes `spawn_clipboard_monitor(...)` a no-op on non-Windows targets.
- `ui/lib/SettingsPanel.svelte` currently disables Aura mode on macOS and normalizes `aura_mode_enabled` back to `false`.

## Invariants

- All implementation and review handoffs remain sequential on `main`.
- Windows hotkey, Aura-mode, and paste-back behavior must not regress while macOS support is added.
- Manual hotkey translation on macOS must continue to work whether Aura mode is enabled or disabled.
- Backend capability truth must own feature availability; the frontend must not reintroduce hardcoded platform assumptions once capability reporting exists.
- `docs/` becomes the final source of truth before this plan can close.

## Dependencies

- A tested macOS Accessibility and/or Automation path for source-app focus and paste operations.
- A compile-safe macOS implementation strategy that does not break Windows or Linux builds.
- macOS host evidence for permission prompts, denied-permission behavior, and successful paste-back into a real source app.
- Shared `main` CI for Windows and macOS after implementation lands.

## Risks and Unknowns

- macOS permission prompts may be app-specific, timing-sensitive, or difficult to automate reliably.
- Source-app refocus and paste injection may behave differently across native and browser apps.
- Clipboard polling may be sufficient for parity, but duplicate suppression and self-generated clipboard writes must be revalidated on macOS.
- CI can prove build/test health but not interactive permission or cross-app behavior, so local host evidence remains mandatory.

## Acceptance Criteria

- macOS Settings can expose Aura mode as a supported feature rather than force-disabling it.
- With Aura mode enabled on macOS, copying fresh text auto-triggers translation and still respects duplicate suppression plus the sensitive clipboard guard.
- macOS paste-back is reported through a truthful capability/status API and works against at least one real source app during manual verification.
- When macOS permissions are denied or unavailable, the UI and backend surface actionable, non-misleading failure states.
- Windows Aura mode and paste-back still pass their existing checks on the final shared SHA.
- `npm run check`, `npm test`, and `cargo test --manifest-path src-tauri/Cargo.toml` pass on the final shared SHA.
- Windows and macOS CI pass on the same final `main` SHA.
- `docs/UI-Shell/UI-Shell-Description.md`, `docs/Daemon-Core/Daemon-Core-Description.md`, and `docs/macOS-Adaptation-Checklist.md` reflect the implemented current state.

## Main-Branch Handoff Sequence

1. Freeze shared contracts and push the plan to `main`.
2. Assign the macOS implementation task lock; the implementer pulls `main`, records the starting SHA, and implements only the approved scope.
3. Codex reviews the macOS implementation, commits, and pushes approved work to `main`.
4. Assign the Windows verification task lock on the new shared SHA and resolve any Windows-only regressions if they appear.
5. Assign the macOS verification task lock on the latest shared SHA and capture target-host permission/runtime evidence.
6. Close only after docs, reviews, and both CI gates align on the same final `main` SHA.
