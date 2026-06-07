# UI Shell Plan

## Objective

Ship a Windows-first two-window desktop shell where translation feels immediate and lightweight:

- the translation surface behaves like a movable floating bubble near the cursor
- settings live in a dedicated tool window
- copied text can trigger translation automatically through Aura mode
- the last translation stays recallable until the next translation begins

## Current Status

### Phase 1: Translation Bubble Split - DONE

- Replaced the single `main` window model with `translation` and `settings` windows.
- Moved Settings out of the translation overlay flow.
- Routed the frontend by current Tauri window label.

### Phase 2: Cursor-Anchored Translation UX - DONE

- Translation bubble now opens near the cursor instead of forcing a tray-adjacent bottom-right position.
- Bubble content is minimal and focused on translated output plus pin/copy/close actions.
- Unpinned bubble auto-sizes with content and re-aligns after size changes.

### Phase 3: Pinned Placement Memory - DONE

- Pinned translation window now persists its last dragged size and position.
- Settings window now persists its last dragged size and position.
- Restored placements are clamped back into visible monitor work areas.

### Phase 4: Aura Mode Triggering - DONE

- Added `aura_mode_enabled` to config.
- Added Windows clipboard polling for automatic translation of fresh copied text.
- Added self-copy suppression so copying Aura’s own result does not retrigger translation.
- Kept the global hotkey as a fallback trigger plus recall/hide control.

### Phase 5: Request Persistence And Recall - DONE

- Hiding the translation bubble no longer clears the last result.
- The next translation start is the only event that overwrites the visible result state.
- Hotkey recall reopens the bubble without forcing a new translation when the clipboard text is unchanged.

### Phase 6: Named Translation Profiles - DONE

- Settings now supports creating, renaming, activating, and deleting named translation profiles.
- The tray menu now mirrors those profiles so users can switch provider, model, and language defaults without opening Settings.
- Profile switching intentionally leaves global hotkeys and window placement untouched.

### Phase 7: Source App Paste-Back - DONE

- The translation bubble now exposes a paste-back action when Aura has a captured source window.
- Paste-back temporarily swaps the text clipboard, sends `Ctrl+V` to the original Windows app, and then restores the previous text clipboard when possible.
- This flow stays opt-in from the translation bubble and does not change the existing copy-first workflow.

### Phase 8: Pinned Draft Composer - DONE

- Pinned mode now upgrades the source preview into a lightweight draft composer.
- Users can edit source text inline and press `Ctrl+Enter` or click `Translate edits` to reuse the current translation session flow.
- The draft stays inside the translation window instead of pushing this workflow into Settings.

### Phase 9: Platform-Aware Desktop Controls - DONE

- Settings now asks the backend for `get_desktop_platform`.
- macOS and Linux builds disable the Aura mode switch and save `aura_mode_enabled: false` even if an older config had it enabled.
- The translation popup hides the paste-back action when the backend reports paste-back is unsupported.
- Added UI regression coverage for macOS Aura mode disablement and unsupported paste-back hiding.

### Phase 10: Remaining Enhancements - NOT STARTED

- Add explicit retry state UI instead of console-only retry logging.
- Add more keyboard-only controls inside both windows.
- Evaluate replacing clipboard polling with a native message-based clipboard listener.

## Implementation Rules

- Keep translation result state request-scoped and ignore stale stream events.
- Do not clear the retained result when the bubble is merely hidden.
- Keep Aura mode Windows-only unless another platform gains a tested equivalent trigger path.
- Do not let non-Windows Settings persist `aura_mode_enabled: true`.
- Do not render paste-back controls when the backend reports paste-back is unsupported.
- Treat pinned placement persistence and settings placement persistence as config-backed behavior, not transient UI-only state.

## Validation Baseline

- `npm test`
- `npm run check`
- `cargo check`
- `cargo test`
