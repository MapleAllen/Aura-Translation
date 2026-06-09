# Shared Contracts

## Ownership

Owner: Codex coordinator
Dependencies: Current `docs/` and the completed `macos-v023-integration-hardening` archive reviewed

## Active Task Lock

Owner: Codex macOS implementer
Starting main SHA: `fec18774b84f57fbdd9292464f10c7403fc33144`
Task: Implement the backend-driven capability model plus macOS Aura-mode and paste-back support within the approved scope.
Status: ASSIGNED

## Allowed Files

- `src-tauri/src/lib.rs`
- `src-tauri/src/config.rs`
- `src-tauri/src/readiness.rs`
- new macOS-specific backend helpers under `src-tauri/src/` when needed for permissions or automation
- `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock` only if new macOS-scoped dependencies are required
- `ui/lib/SettingsPanel.svelte`
- `ui/lib/TranslationWindowView.svelte`
- `ui/lib/TranslationPopup.svelte`
- `ui/lib/runtimeStatus.ts`
- directly corresponding frontend/backend test files for the touched behavior
- `docs/UI-Shell/UI-Shell-Description.md`
- `docs/Daemon-Core/Daemon-Core-Description.md`
- `docs/macOS-Adaptation-Checklist.md`
- this plan folder

## Do Not Modify

- Translation-provider request/response semantics unrelated to Aura mode or paste-back
- Keychain/plaintext credential-storage policy
- Release assets, tagging, or prerelease workflow
- Linux feature scope beyond compile-safe fallbacks/stubs required by touched shared code
- Completed plan archives except for reference while drafting this plan

## Public Interfaces and Critical Functions

- `get_desktop_platform()`
  - Current responsibility: return the backend OS label.
  - Contract for this plan: frontend platform checks must not remain the final source of truth for Aura/paste-back support once capability reporting exists.

- `get_paste_back_status(...)`
  - Current responsibility: report support plus availability of a captured source window.
  - Contract for this plan: backend remains the source of truth for whether paste-back is supported, available now, or blocked by missing permission/preconditions.

- `paste_translation_back(app, runtime, text)`
  - Responsibility: validate requested text, reuse captured source-app context, perform platform paste-back, and emit actionable failure messages.
  - Compatibility rule: Windows behavior must remain intact; macOS support may add new permission/focus branches but must not weaken existing Windows branches.

- `spawn_clipboard_monitor(app)`
  - Responsibility: watch clipboard changes when Aura mode is enabled, avoid self-trigger loops, respect duplicate suppression, and dispatch translations.
  - Compatibility rule: shared suppression logic must remain consistent across Windows and macOS implementations.

- `save_config(app, state, profiles_state, config)`
  - Responsibility: validate/persist user settings and synchronize runtime state.
  - Contract for this plan: once macOS Aura mode is supported, saving config must not silently force `aura_mode_enabled` back to `false` on macOS.

- `AppConfig.aura_mode_enabled`
  - Responsibility: persist user intent for automatic clipboard translation.
  - Contract for this plan: backend/frontend normalization must reflect actual platform capability, not a stale Windows-only assumption.

## Tasks

1. Define the backend-driven capability surface needed for Aura mode and paste-back on macOS.
2. Implement macOS Aura-mode clipboard monitoring with shared suppression and guard behavior.
3. Implement macOS paste-back plus permission/error handling.
4. Update Settings and translation-window UI to consume capability truth from the backend.
5. Update tests and current-state docs, then verify shared behavior on Windows and macOS.

## Automated Verification

- `npm run check`
- `npm test`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- macOS host build verification when implementation is ready: `npm run tauri build`
- Windows Trial Gate on the final shared SHA
- macOS CI gate on the final shared SHA

## Completion Evidence

- Shared contracts remained stable during implementation, or any required change was recorded before more source work continued.
- One task lock was active at a time.
- Capability reporting, Aura mode, paste-back, tests, and docs all reflect the same final behavior.

## Deviations

- None yet.
