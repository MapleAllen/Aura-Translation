# UI Shell Module Description

## Module Name

UI Shell

## Purpose

The UI Shell is the visual layer of Aura Translation. It renders a compact desktop translation window that opens on demand, streams translated text in real time, and can either dismiss on blur or remain pinned for side-by-side reading. The shell owns the full user-facing workflow: showing copied source text, loading and streaming states, inline translation errors, transient daemon warnings, language selection, copy-to-clipboard, and settings management for provider, model, hotkey, API key, and window behavior.

## Current Implementation

The shell is a SvelteKit application with a single route (`ui/routes/+page.svelte`) acting as the lifecycle orchestrator. The Tauri backend creates the `main` window lazily; once the frontend mounts it calls `mark_ui_ready`, loads config via `get_config`, and registers request-scoped listeners for translation and daemon events.

The runtime lifecycle is:

1. **Hidden**: the desktop window is not visible and waits for a hotkey or tray action.
2. **Show**: `trigger-translate` or `show-settings` arrives; the shell springs to full opacity and scale.
3. **Translate**: `startTranslation()` moves `appState` from `idle` to `loading`, starts a 20-second timeout, and invokes `translate_text`.
4. **Stream**: current-request `translation-chunk` events move the UI into `streaming` and append text incrementally.
5. **Settle**: `translation-done` moves the UI to `result`; `translation-error` moves it to `error`.
6. **Dismiss or stay visible**: blur hides the window only when Settings is closed and `config.window_pinned === false`; otherwise the window stays visible for side-by-side comparison.

The shell uses two `Spring` instances for structural motion: `popupScale` starts at `0.92` and `popupOpacity` starts at `0`. Dismiss animates back to those values, waits 220 ms, resets transient state, and calls `getCurrentWindow().hide()`.

State is kept locally in the route component with Svelte 5 runes: `appState`, `sourceText`, `translatedText`, `errorMessage`, `showSettings`, `notifications`, `hotkeyConflictMessage`, language pair state, the current request ID, and the in-memory `AppConfig` copy. There are no Svelte stores.

### Capabilities

**Popup lifecycle**
- Spring-driven show/hide animation with scale (`stiffness: 0.14`, `damping: 0.68`) and opacity (`stiffness: 0.18`, `damping: 0.82`)
- Dismiss via `Esc` or `window-blur` when `shouldDismissOnBlur(showSettings, windowPinned)` returns `true`
- 220 ms delay before `appWindow.hide()` so the close animation can complete
- `show-settings` opens the same shell window in-place instead of launching a separate settings surface

**Translation states**
- `idle`: shows a quick-capture hint using the configured hotkey label
- `loading`: renders `SkeletonLoader` and starts a 20-second timeout
- `streaming`: appends current-request chunks to `translatedText` and shows a blinking cursor
- `result`: shows the final text and keeps copy actions available
- `error`: shows a centered error panel and also pushes a notification

**Request scoping and cancellation**
- Every translation uses a monotonically increasing `currentRequestId`
- Stale `translation-chunk`, `translation-done`, `translation-error`, and `translation-retry` payloads are ignored unless `request_id === currentRequestId`
- Starting a new translation, cancelling manually, dismissing the shell, or switching languages mid-stream all invoke `cancel_translate`
- Manual cancel falls back to `result` when partial text exists, otherwise `idle`

**Language controls**
- 11 selectable languages including `auto` for source detection
- Swap button is disabled when the source language is `auto`
- Changing language during `loading` or `streaming` cancels the active request and immediately restarts translation with the new pair

**Pinned window mode**
- Header pin button toggles `config.window_pinned`
- Pin state is saved immediately through `save_config`
- When pinned, the shell remains visible on blur and the backend applies `always_on_top`
- Settings exposes the same preference as a switch in the "Window behavior" section

**Settings panel**
- Overlay panel mounted above the translation window when `visible === true`
- Sections for window behavior, provider, plaintext API-key warning, API key entry, dynamic model list, and live hotkey capture
- Provider switch invokes `get_provider_defaults` and updates `api_base_url`, `available_models`, and `model`
- Save action invokes `save_config`, emits `onsaved`, shows transient success state, and preserves inline hotkey validation errors

**Notifications and daemon feedback**
- `NotificationCenter` stacks up to 3 active notifications
- `hotkey-conflict` becomes both an inline settings warning and a dismissible toast
- `daemon-error` events surface backend problems such as tray or window failures
- Translation failures are normalized with `formatTranslationError()` before being shown

**Resize affordances**
- Eight invisible edge and corner handles are rendered from `RESIZE_HANDLES`
- Mouse down on a handle calls `getCurrentWindow().startResizeDragging(direction)`
- Window resizing is enabled by Tauri config rather than by in-app layout controls

**Design system**
- Tailwind CSS v4 theme tokens live in `ui/app.css`
- Accent palette is light-mode first: `aura-accent (#2b86ff)`, muted borders, slate text tones, white glass surfaces
- Fonts: `Outfit` for display text and `DM Sans` for body text
- Shared keyframes: `shimmer`, `pulse-glow`, and `fade-in-up`

## Architecture

Single-route SvelteKit application using Svelte 5 runes and callback props. State lives at the route level and flows downward into presentational components.

### Orchestrator (`ui/routes/`)

- `+page.svelte`
  - Owns all shell state, request IDs, notifications, and the in-memory `config`
  - `loadConfig()`: invokes `get_config` and applies the returned config
  - `startTranslation()`: validates API-key requirements, increments the request ID, clears prior output, sets `appState = 'loading'`, starts the timeout, and invokes `translate_text`
  - `cancelCurrentTranslation()`: invokes `cancel_translate` for the current request ID and clears the timeout
  - `handleLanguageChange()`: updates language state and retriggers translation when needed
  - `handlePinnedChange()`: persists `window_pinned` immediately and reverts local state if the save fails
  - `dismiss()`: cancels active work, animates out, resets shell state, and hides the Tauri window
  - Registers 10 listeners: `trigger-translate`, `translation-chunk`, `translation-done`, `translation-error`, `translation-retry`, `window-blur`, `show-settings`, `hotkey-conflict`, `hotkey-registered`, and `daemon-error`
  - Calls `mark_ui_ready` after mount so the Rust backend can wait for the frontend before emitting events into a newly created window

### Components (`ui/lib/`)

- `TranslationPopup.svelte`
  - Renders the main translation card, source preview, footer, pin button, cancel button, close button, copy button, and inline view-state messaging
  - Accepts `windowPinned`, `onTogglePinned`, `oncancel`, and `ondismiss` in addition to translation props
  - Uses a `Spring` for copy-button bounce feedback

- `LanguageSelector.svelte`
  - Owns the source/target toolbar and animated swap button
  - Emits `onchange(source, target)` back to the route

- `SkeletonLoader.svelte`
  - Stateless four-line shimmer placeholder

- `SettingsPanel.svelte`
  - Loads config on open with `get_config`
  - Shows provider-aware fields and a pin-window switch
  - Captures hotkeys with modifier enforcement and letter/digit restriction
  - Emits `onsaved(config)` after successful persistence

- `NotificationCenter.svelte`
  - Renders stacked alert cards with severity accents and dismiss actions

- `notifications.ts`
  - Defines `AppNotification`, notification factories, error formatting, and the plaintext API-key warning predicate

- `windowBehavior.ts`
  - Exports `RESIZE_HANDLES` and `shouldDismissOnBlur(showSettings, windowPinned)`

### Design System (`ui/`)

- `app.css`
  - Theme tokens, font imports, scrollbar treatment, shell gradients, and skeleton animation rules
- `app.html`
  - SvelteKit HTML shell used by the Tauri webview

## Integration Points

- `ui/routes/+page.svelte`
  - `invoke('get_config')`
  - `invoke('save_config', { config })`
  - `invoke('translate_text', { text, sourceLang, targetLang, apiKey, model, requestId, apiBaseUrl, provider })`
  - `invoke('cancel_translate', { requestId })`
  - `invoke('mark_ui_ready')`
  - `listen('trigger-translate')`
  - `listen('translation-chunk')`
  - `listen('translation-done')`
  - `listen('translation-error')`
  - `listen('translation-retry')`
  - `listen('window-blur')`
  - `listen('show-settings')`
  - `listen('hotkey-conflict')`
  - `listen('hotkey-registered')`
  - `listen('daemon-error')`
  - `getCurrentWindow().hide()`
  - `getCurrentWindow().startResizeDragging(direction)`

- `ui/lib/TranslationPopup.svelte`
  - `writeText(translatedText)` from `@tauri-apps/plugin-clipboard-manager`

## Current Limitations

- **Retry state is not visibly surfaced**: `translation-retry` is only logged with `console.warn`; the UI does not yet show a retry badge or progress state.
- **Settings still remount on every open**: `{#if visible}` causes the panel to reload config and reset local UI state on each open/close cycle.
- **Dismiss uses fixed animation timing**: the 220 ms hide delay is hardcoded rather than derived from spring completion.
- **Copy failures stay in developer logs**: clipboard write failures are only logged to `console.error`; the user does not get a notification.
- **No session history**: each new translation replaces the previous visible result.

## Future Directions

- Add a visible retry indicator sourced from `translation-retry`.
- Preserve `SettingsPanel` mount state while hidden instead of tearing it down with `{#if}`.
- Add session-level history and quick replay for recent translations.
- Add keyboard focus management for the full popup, not only `Esc` dismissal.
- Surface clipboard-copy failures and resize failures through the notification system.
