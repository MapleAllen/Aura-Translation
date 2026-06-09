# macOS Implementation and Verification

Owner: macOS implementer
Dependencies: Shared contracts frozen
Working branch: `main`
Starting SHA: RESOLVE AND RECORD BEFORE SOURCE EDITS

## Allowed Files

- `src-tauri/src/lib.rs`
- `src-tauri/src/config.rs`
- `src-tauri/src/readiness.rs`
- new macOS-specific backend helpers under `src-tauri/src/` when needed for permissions or automation
- `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock` only if macOS-scoped dependencies are required
- `ui/lib/SettingsPanel.svelte`
- `ui/lib/TranslationWindowView.svelte`
- `ui/lib/TranslationPopup.svelte`
- `ui/lib/runtimeStatus.ts`
- directly corresponding frontend/backend test files
- `docs/UI-Shell/UI-Shell-Description.md`
- `docs/Daemon-Core/Daemon-Core-Description.md`
- `docs/macOS-Adaptation-Checklist.md`
- this file
- `plan/active/macos-capability-parity/04-verification.md`
- `plan/active/macos-capability-parity/05-review-and-integration.md`

## Do Not Modify

- Provider transport semantics unrelated to Aura mode or paste-back
- Release/publish workflow
- Linux parity scope beyond compile-safe fallbacks required by touched shared code
- Completed archive plans except by reference

## Platform-Specific Functions and Behavior

- macOS Aura mode must auto-trigger only when the feature is enabled and the runtime is ready to translate.
- Clipboard monitoring must respect duplicate suppression and `aura_guard::detect_sensitive_clipboard(...)`.
- macOS paste-back must report truthful capability state, reuse captured source-app context, and fail with actionable messages when permission, focus, or source-window preconditions are missing.
- Manual hotkey translation and the existing tray/Settings flow must continue to work.
- If a permission-denied state exists, the UI must not silently claim the feature is unavailable forever; it should reflect that authorization is required or currently denied.

## Implementation Tasks

1. Pull the latest shared `main`, verify a clean worktree, and record the starting SHA before source edits.
2. Replace Windows-only Aura-mode gating with a backend capability model or equivalent backend-driven status surface.
3. Implement macOS clipboard monitoring for Aura mode using the shared suppression and guard logic.
4. Implement macOS paste-back support, including permission checks, source-app targeting, error reporting, and text-clipboard restore behavior.
5. Update Settings and translation-window UI to reflect the new capability state on macOS.
6. Add or update tests plus current-state docs, then leave reviewable evidence for Codex review.

## Automated Verification

```bash
npm run check
npm test
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

- macOS CI gate on the resulting `main` SHA

## macOS Manual Verification

- Settings shows Aura mode as a supported capability once the implementation is in place.
- Saving config on macOS can persist `aura_mode_enabled: true` when the feature is enabled.
- Copying fresh text with Aura mode enabled auto-triggers translation.
- Re-copying identical text does not retrigger until text changes or the user uses the hotkey fallback.
- Sensitive clipboard guard still blocks obvious credential-like clipboard content.
- Paste-back works into at least one real macOS source app after the required permission path is granted.
- When permissions are denied or revoked, the UI surfaces actionable state/error feedback instead of a misleading success state.
- Manual hotkey translation, tray/menu bar behavior, Settings access, and keychain-backed secrets still work.

## Completion Evidence

- [ ] Starting SHA recorded before source edits.
- [ ] macOS automated checks pass.
- [ ] macOS host manual verification evidence recorded.
- [ ] macOS CI gate passes on the shared final SHA.
- [ ] Docs updated to match the final shipped behavior and known limitations.

## Deviations and Remaining Risks

- None yet.
