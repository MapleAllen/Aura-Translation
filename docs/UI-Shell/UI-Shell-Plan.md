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

### Phase 6: Remaining Enhancements - NOT STARTED

- Add explicit retry state UI instead of console-only retry logging.
- Add multi-entry translation history beyond the single retained result.
- Add more keyboard-only controls inside both windows.
- Evaluate replacing clipboard polling with a native message-based clipboard listener.

## Implementation Rules

- Keep translation result state request-scoped and ignore stale stream events.
- Do not clear the retained result when the bubble is merely hidden.
- Keep Aura mode Windows-only unless another platform gains a tested equivalent trigger path.
- Treat pinned placement persistence and settings placement persistence as config-backed behavior, not transient UI-only state.

## Validation Baseline

- `npm test`
- `npm run check`
- `cargo check`
- `cargo test`
