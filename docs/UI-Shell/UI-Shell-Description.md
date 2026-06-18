# UI Shell Module Description

## Module Name

UI Shell

## Purpose

The UI Shell is the desktop surface for Aura Translation. It now exposes two coordinated windows:

1. **Translation window**: a minimal floating bubble that appears near the cursor, streams translated text, can be pinned, resized, dragged, hidden, and recalled without losing the last result.
2. **Settings window**: a separate movable tool window opened from the tray, used to configure provider access, named translation profiles, default language pair, hotkey, Aura mode, and pinned-window behavior.

The shell remains tray-driven and daemon-backed, but it no longer treats Settings as an overlay inside the translation surface.

## Current Implementation

The shell is a single SvelteKit route (`ui/routes/+page.svelte`) that branches by the current Tauri window label:

- `translation` renders `TranslationWindowView.svelte`
- `settings` renders `SettingsWindowView.svelte`

Tauri creates the hidden translation window during startup and lazily creates the settings window on demand. Both windows call `mark_ui_ready` after mount so the Rust backend can safely target them.

### Translation window behavior

- New translations are triggered by:
  - the global hotkey when Aura mode is off
  - Windows clipboard changes when Aura mode is on
  - the global hotkey as a fallback when Aura mode is on and the clipboard text is new
- Hotkey press in Aura mode hides the visible bubble, or recalls the last result when no new text is available.
- The bubble stays hidden between uses, but its last result remains mounted in memory until the next translation starts.
- The window auto-sizes to loading, streaming, result, and error states while unpinned.
- When unpinned, the backend repositions the bubble above the cursor and clamps it to the current monitor work area.
- When pinned, the bubble stops auto-hiding on blur and persists its dragged size and position.
- When pinned, the source preview becomes a lightweight draft composer so the user can edit source text in place and re-translate with `Cmd/Ctrl+Enter`.
- Result actions rely on explicit copy; source-app paste-back is no longer part of the UI shell.

### Settings window behavior

- Opened from the tray menu as an independent tool window.
- First open defaults to the bottom-right of the current work area.
- Later opens restore the last saved position and size, clamped back into a visible monitor region if display layout changes.
- Settings are loaded on open and saved through `save_config`.
- Settings also calls `get_system_capabilities` so features can be disabled or configured based on system/permission capabilities before the user saves preferences.
- On platforms where Aura mode is unsupported (e.g. Linux), the Aura mode switch is disabled, the config is normalized back to `aura_mode_enabled: false` before save, and the Settings copy explains the manual-hotkey workflow.

## Architecture

### Frontend

- `ui/routes/+page.svelte`
  - detects the current window label and renders the correct window view

- `ui/lib/TranslationWindowView.svelte`
  - owns translation request state, notifications, window auto-sizing, pin persistence, dismissal, and translation event listeners
  - persists pinned placement through `save_window_placement`
  - calls `realign_translation_window` after automatic size changes

- `ui/lib/SettingsWindowView.svelte`
  - owns settings-scoped notifications and window placement persistence
  - hides the settings window on close or `Esc`

- `ui/lib/TranslationPopup.svelte`
  - renders the minimal floating translation bubble
  - exposes pin, retry, copy, cancel, close, pinned-mode draft editing controls, and token usage badges when available

- `ui/lib/SettingsPanel.svelte`
  - renders the dedicated settings form
  - now includes named translation profiles, default language pair, Aura mode configuration, provider settings, API key storage, hotkey, pin behavior, and recent translation history actions
  - calls `get_system_capabilities`, disables Aura mode on platforms where it is unsupported, and saves a platform-normalized config

- `ui/lib/ProfileManager.svelte`
  - renders profile create, rename, activate, and delete controls inside Settings

- `ui/lib/translationProfiles.ts`
  - defines the typed profile store shape shared by the Settings shell and backend commands

- `ui/lib/HistoryList.svelte`
  - renders the recent translation history list inside Settings
  - exposes copy, retry, delete, clear, and token usage summaries for the latest 50 history entries

- `ui/lib/windowPlacement.ts`
  - converts current window physical geometry into logical coordinates and persists them through the backend

### Backend

- `src-tauri/src/lib.rs`
  - owns both window lifecycles, hotkey handling, cursor-anchored positioning, settings placement restore, and platform clipboard polling for Aura mode
  - persists window placement metadata into config
  - suppresses self-originated clipboard writes when the translation bubble copies its own result

- `src-tauri/src/config.rs`
  - stores `aura_mode_enabled`
  - stores `settings_window_placement`
  - stores `pinned_translation_placement`

## Integration Points

- `invoke('get_config')`
- `invoke('get_system_capabilities')`
- `invoke('save_config', { config })`
- `invoke('save_window_placement', { kind, placement })`
- `invoke('realign_translation_window')`
- `invoke('copy_result_to_clipboard', { text })`
- `invoke('translate_text', { ... })`
- `invoke('cancel_translate', { requestId })`
- `invoke('get_translation_history')`
- `invoke('delete_translation_history_entry', { entryId })`
- `invoke('clear_translation_history')`
- `invoke('replay_translation_history_entry', { entryId })`
- `invoke('get_translation_profiles')`
- `invoke('create_translation_profile', { config, name })`
- `invoke('rename_translation_profile', { profileId, name })`
- `invoke('activate_translation_profile', { profileId })`
- `invoke('delete_translation_profile', { profileId })`
- `invoke('mark_ui_ready')`
- `listen('trigger-translate')`
- `listen('show-existing-translation')`
- `listen('translation-chunk')`
- `listen('translation-done')`
- `listen('translation-error')`
- `listen('window-blur')`
- `listen('config-updated')`
- `listen('hotkey-conflict')`
- `listen('hotkey-registered')`
- `listen('daemon-error')`

## Current Limitations

- Aura mode is unsupported on Linux; Linux users still rely on the manual hotkey flow.
- Settings prevents users on unsupported platforms from enabling Aura mode, and normalizes their config accordingly.
- Clipboard auto-trigger intentionally ignores repeated copies of identical text until a different text arrives or the user uses the hotkey fallback.
- The cursor-near bubble uses cursor position rather than exact cross-application text selection bounds.
- Translation history is currently managed from Settings only; the tray still does not expose recent entries directly.
- Translation profiles intentionally scope only translation provider and language defaults; hotkey and window placement stay global.
- Source-app paste-back has been removed from the UI shell; result actions now rely on explicit copy.

## Future Directions

- Add recent history access directly from the tray experience.
- Surface retry state inside the translation bubble instead of logging retries only.
