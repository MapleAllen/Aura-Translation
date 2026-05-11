# UI Shell Module Description

## Module Name

UI Shell

## Purpose

The UI Shell is the visual layer of Aura Translation. It renders a 440×360 px glassmorphic floating popup that animates into view when the hotkey fires and dismisses on `Esc` or focus loss. The core UX goal is for the widget to feel like a **natural extension of the operating system** rather than a browser tab: physically-based spring animations create a haptic "pop-up" effect — the popup briefly overshoots past 100% scale before settling — giving every interaction a tangible, alive quality. The shell handles the complete translation lifecycle from the user's perspective: displaying the source text, pulsating while waiting for the first token, streaming translated text token-by-token, surfacing error states, and letting the user copy the result or change the language pair. A settings overlay is also embedded in the shell for API key and model configuration.

## Current Implementation

The shell implements a five-stage lifecycle:

1. **Daemon** — Window is hidden (`visible: false`), waiting for the hotkey.
2. **Trigger** — The `trigger-translate` Tauri event fires; Svelte springs the popup in with a scale overshoot (`0.92 → 1.0`, tuned to briefly exceed 1.0 via low damping) and opacity fade, creating a haptic "pop" feel.
3. **Loading** — `appState` enters `loading`; `SkeletonLoader` renders 4 pulsating shimmer bars to signal activity without feeling mechanical.
4. **Streaming** — First `translation-chunk` event arrives; `appState` enters `streaming`; tokens append to `translatedText` with a blinking cursor; translated text fades in and pans upward via the `fade-in-up` keyframe animation.
5. **Dismiss** — On `Esc` or focus loss, the popup springs back to scale `0.92` / opacity `0` over ~220 ms, then `appWindow.hide()` returns the process to the daemon state.

The shell is a SvelteKit application with a single route (`ui/routes/+page.svelte`) that acts as the lifecycle orchestrator. On mount, it loads config from the Rust backend, registers five Tauri event listeners, and binds keyboard events for `Esc`. Two `Spring` instances (`popupScale` starting at `0.92`, `popupOpacity` starting at `0`) drive the enter/exit animation: on trigger the targets jump to `1`; on dismiss they return to `0.92` / `0`. A 220 ms delay after the spring settles hides the Tauri window.

The orchestrator maintains an `appState` union (`idle | loading | streaming | result | error`) that is driven by incoming Tauri events. Each translation is assigned a monotonically incrementing `currentRequestId` (used for cancellation targeting). A `startTranslation()` function encapsulates the API call setup — incrementing the request ID, clearing previous text, setting `appState = 'loading'`, starting a 20-second loading timeout, and invoking `translate_text` with the current source text, language pair, and request ID. The `TranslationPopup` component renders different content for each state and exposes a cancel button (× icon) during `loading` and `streaming` states. `SettingsPanel` is layered absolutely above the popup and toggled via a `showSettings` boolean.

The design system is defined in `ui/app.css` as Tailwind CSS v4 `@theme` tokens under the `aura-` namespace (accent color `#7c6aef`, glassmorphic background `rgba(12, 12, 20, 0.78)`, two font families: Outfit and DM Sans).

### Capabilities

**Popup lifecycle**
- Spring-animated scale (`stiffness: 0.14, damping: 0.68`) and opacity (`stiffness: 0.18, damping: 0.82`) entry/exit, anchored to `transform-origin: bottom right`
- Low damping on the scale spring is intentional: it allows the popup to momentarily overshoot past `scale(1.0)` before settling, producing the haptic "pop" feel described in the architecture goals
- 220 ms post-animation delay before `appWindow.hide()` to allow the spring to settle
- Auto-dismiss on `Esc` key or `window-blur` Tauri event (suppressed when Settings is open)

**Translation states**
- `idle`: shows placeholder hint text "Copy text and press Ctrl+T"
- `loading`: renders `SkeletonLoader` (4 shimmer bars with staggered 120 ms animation delays: widths 100%, 88%, 72%, 55%); starts a 20-second loading timeout
- `streaming`: renders translated text with an animated blinking cursor appended; chunks are only appended when `appState === 'streaming'` (guarded against stale events)
- `result`: same as streaming but cursor removed; copy button appears in footer
- `error`: error icon + message text (e.g., "No API key configured…", "No response from API (timeout)")

**Language selector**
- 11 language options including `auto` (source only): Chinese, English, Japanese, Korean, French, German, Spanish, Russian, Arabic, Portuguese
- Animated swap button (`Spring { stiffness: 0.3, damping: 0.65 }`, continuous rotation accumulation) — disabled when source is `auto`
- Language changes propagate up to the orchestrator via `onLanguageChange(source, target)`
- **Mid-stream language switch:** if a language change occurs while `appState === 'loading'` or `'streaming'`, the orchestrator cancels the current translation (via `cancel_translate`) and immediately starts a new one with the updated language pair

**Copy result**
- Available in `streaming` and `result` states when `translatedText` is non-empty
- Writes to the system clipboard via `@tauri-apps/plugin-clipboard-manager`
- Copy button uses a `Spring { stiffness: 0.4, damping: 0.5 }` scale bounce on click; shows "Copied!" for 1500 ms

**Settings panel**
- Glassmorphic overlay rendered absolutely over the popup (`z-50`)
- Fields: DeepSeek API Key (password input with show/hide toggle), Model (dropdown: `deepseek-v4-flash` / `deepseek-v4-pro`), Hotkey (read-only display of `Ctrl + T`)
- Save button with spring bounce animation; shows "Saved!" for 1200 ms on success
- Panel opened via tray `show-settings` event or programmatically; closed by `Esc` or the close button

**Request cancellation**
- Cancel button (× icon, 20×20 px) appears in the `TranslationPopup` header next to the status indicator during `loading` and `streaming` states
- Styled with `hover:text-aura-error` and `hover:bg-aura-error/10` transitions for visual feedback
- On click: invokes `cancel_translate` with the current request ID; transitions to `result` (if partial text exists) or `idle` (if no text arrived)
- New translations also cancel any in-flight request before starting

**Loading timeout**
- A 20-second `setTimeout` starts when `appState` enters `'loading'`
- If no `translation-chunk` event arrives within 20 seconds, `appState` transitions to `'error'` with message "No response from API (timeout)"
- The timeout is cleared when the first chunk arrives, when `translation-done` fires, when `translation-error` fires, or on dismiss

**Design system**
- Color tokens: `aura-accent (#7c6aef)`, `aura-glass (rgba(255,255,255,0.05))`, `aura-border (rgba(255,255,255,0.07))`, `aura-text (#e8e6f0)`, `aura-error (#f87171)`, `aura-success (#4ade80)`
- Fonts: `Outfit` (display/headings), `DM Sans` (body text), both via Google Fonts
- Keyframe animations: `shimmer` (skeleton), `pulse-glow` (skeleton), `fade-in-up` (settings panel, streaming text)
- Scrollbar: 4 px wide, transparent track, `rgba(255,255,255,0.1)` thumb
- Window is `overflow: hidden` with `user-select: none` globally; `select-text` class re-enables selection on source/result text

## Architecture

Single-route SvelteKit application using Svelte 5 runes API (`$state`, `$props`, `$effect`). No stores or reactive contexts — state is passed as props from the orchestrator page downward to components.

### Orchestrator (`ui/routes/`)

- `+page.svelte`
  - Owns all `$state` variables: `appState`, `sourceText`, `translatedText`, `errorMessage`, `showSettings`, `sourceLang`, `targetLang`, `config`, `currentRequestId`, `loadingTimeoutId`.
  - Owns the two `Spring` instances for popup animation.
  - `loadConfig()`: invokes `get_config` and syncs `config`, `sourceLang`, `targetLang`.
  - `startTranslation()`: increments `currentRequestId`, resets text/error state, sets `appState = 'loading'`, starts loading timeout, invokes `translate_text` with all six arguments including `requestId`.
  - `cancelCurrentTranslation()`: invokes `cancel_translate` with the current request ID and clears the loading timeout.
  - `handleCancel()`: calls `cancelCurrentTranslation()`, then transitions to `result` (if partial text) or `idle`.
  - `dismiss()`: clears loading timeout, springs out → 220 ms timeout → resets state → `appWindow.hide()`.
  - `handleKeydown(e)`: routes `Esc` to close Settings or dismiss.
  - `handleLanguageChange(source, target)`: updates state; if currently loading or streaming, cancels and retranslates immediately.
  - Registers Tauri event listeners in `onMount`: `trigger-translate`, `translation-chunk`, `translation-done`, `translation-error`, `window-blur`, `show-settings`.
  - `translation-chunk` listener: only appends to `translatedText` when `appState === 'streaming'` (guards against stale events from cancelled requests).

### Components (`ui/lib/`)

- `TranslationPopup.svelte`
  - Props: `viewState`, `sourceText`, `translatedText`, `errorMessage`, `sourceLang`, `targetLang`, `onLanguageChange`, `oncancel`.
  - Renders the glassmorphic card with backdrop blur (`28px`), box-shadow stack, and drag region.
  - Delegates loading state to `SkeletonLoader` and language UI to `LanguageSelector`.
  - Renders a cancel button (× icon) in the header bar during `loading` and `streaming` states; fires `oncancel` on click.
  - `copyResult()`: calls `writeText(translatedText)` from `@tauri-apps/plugin-clipboard-manager`; spring-bounces the copy button.

- `LanguageSelector.svelte`
  - Props: `sourceLang`, `targetLang`, `onchange(source, target)`.
  - `LANGUAGES` constant: array of 11 `{ code, label, flag }` objects.
  - `swap()`: blocked when `sourceLang === 'auto'`; accumulates rotation count for the spring animation.

- `SkeletonLoader.svelte`
  - Stateless. Renders 4 `div.skeleton-bar` elements with fixed widths and staggered `animation-delay` offsets (`0ms`, `120ms`, `240ms`, `360ms`).

- `SettingsPanel.svelte`
  - Props: `visible`, `onclose`.
  - `$effect`: calls `loadConfig()` whenever `visible` becomes `true`.
  - `saveConfig()`: invokes `save_config` with the local `config` object; spring-bounces the save button.
  - Rendered only when `visible === true` (Svelte `{#if}` block — full DOM teardown on hide).

### Design System (`ui/`)

- `app.css`: Tailwind v4 `@theme` block defining all `aura-*` color and font tokens; global reset; scrollbar styles; keyframe animations; `.skeleton-bar` utility class.
- `app.html`: HTML shell with `%sveltekit.head%` and `%sveltekit.body%` placeholders; no `<meta charset>` or viewport tag (Tauri window, not a browser tab).

### Integration Points

- `ui/routes/+page.svelte`
  - `invoke('get_config') -> AppConfig`: called in `loadConfig()`.
  - `invoke('save_config', { config })`: called in `SettingsPanel.saveConfig()` via the orchestrator.
  - `invoke('translate_text', { text, sourceLang, targetLang, apiKey, model, requestId })`: called in `startTranslation()` on every `trigger-translate` event and on mid-stream language switch.
  - `invoke('cancel_translate', { requestId })`: called in `cancelCurrentTranslation()` on user cancel, language switch mid-stream, or new trigger while streaming.
  - `listen('trigger-translate')`, `listen('translation-chunk')`, `listen('translation-done')`, `listen('translation-error')`, `listen('window-blur')`, `listen('show-settings')`: all registered in `onMount`.
  - `getCurrentWindow().hide()`: called in the dismiss timeout.

- `ui/lib/TranslationPopup.svelte`
  - `writeText(translatedText)` from `@tauri-apps/plugin-clipboard-manager`: clipboard write on copy button click.

## Current Limitations

- **Hotkey display is hardcoded** — the Settings panel shows `Ctrl + T` as static `<kbd>` elements; it does not reflect `config.hotkey` and cannot be interactively reconfigured.
- **Model list is hardcoded** — two `<option>` elements in `SettingsPanel.svelte` (`deepseek-v4-flash`, `deepseek-v4-pro`); adding a new model requires a frontend code change.
- **No translation history** — each trigger replaces the previous result; there is no session-level history panel.
- **`appWindow.hide()` can fail silently** — the `catch` block in `dismiss()` only logs to `console.error`; a failed hide is not surfaced to the user.
- **Settings panel is DOM-destroyed on close** — using `{#if visible}` means every open/close cycle re-mounts and re-loads config; a `visibility: hidden` approach would avoid the config round-trip.
- **`window-blur` dismiss is suppressed only via `showSettings`** — rapid state transitions (e.g., blur event arriving during the dismiss animation) can cause double-dismiss attempts.
- **Dismiss does not cancel the Rust task** — dismissing the popup via `Esc` or focus loss hides the window but does not invoke `cancel_translate`; the in-flight Rust stream continues until the SSE response completes naturally.

## Future Directions

- Replace the hardcoded hotkey `<kbd>` display with a live binding capture widget that reads and writes `config.hotkey`.
- Replace the hardcoded model `<option>` list with a dynamic list sourced from `AppConfig.available_models`.
- Add a collapsible translation history panel showing the last N source/result pairs within a session.
- Add a `translation-retry` listener to show a "retrying…" indicator badge in the status bar.
- Replace `{#if visible}` in `SettingsPanel` with `display: none` to preserve the mounted component across open/close cycles.
- Add keyboard navigation shortcuts within the popup (e.g., `Tab` to cycle focus, `Enter` to copy).
- Support right-to-left layout for Arabic and Hebrew target languages.
- Wire `cancel_translate` into the dismiss flow so hiding the popup also cancels the in-flight Rust task.
