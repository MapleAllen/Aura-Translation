# Interaction Contract Module Description

## Module Name

Interaction Contract

## Purpose

The Interaction Contract module owns the rules that decide what a user gesture means: whether a
global hotkey press translates, collapses, or recalls; whether a clipboard change may be
dispatched; and when two translation requests are the same request.

It exists as its own module for a testing reason rather than a stylistic one. These rules
previously lived inline in `lib.rs::handle_hotkey_pressed` and the clipboard monitor loop, reachable
only through a Tauri `AppHandle`. That made the product's most important promise — pressing the
hotkey again must not issue a second paid request — impossible to assert in CI, and left the
milestone's "count the requests against a stub provider" acceptance method unimplementable.

Everything here is free of Tauri types, so every branch is covered by plain unit tests.

## Current Implementation

`interaction.rs` contains three decision surfaces and two supporting types.

### Request identity

`RequestIdentity` is the complete identity of a translation request: text, source language, target
language, provider, model, base URL, and profile id. `RequestIdentity::from_config(text, config)`
builds one from the live configuration.

Reuse is allowed only on a full match. Comparing text alone — which is what the code did
previously, with a bare `last_requested_text: Option<String>` — let a stale result survive a
language, model, provider, base-URL, or profile change. The identity is persisted in
`TranslationRuntimeState::last_request_identity` and written by both `trigger_translation()` and
the clipboard monitor.

### Hotkey decision

`decide_hotkey(HotkeyContext) -> HotkeyOutcome` is the single decision point for both automatic and
manual modes. `HotkeyContext` carries `window_visible`, `clipboard_empty`, `ready`, and
`matches_last_request`.

| Context | Outcome | Effect |
|---|---|---|
| Configuration incomplete | `Unavailable` | Open Settings |
| Clipboard empty | `ShowEmptyState` | Open an editable empty state |
| Same identity, window visible | `Collapse` | Hide the window; in-flight work keeps running |
| Same identity, window hidden | `Recall` | Show the existing result |
| New text | `TranslateNew` | Dispatch a request |

The ordering is deliberate: readiness and an empty clipboard are resolved before identity, so an
unusable configuration never attempts a request and an empty clipboard never reports a spurious
"same request".

Only `TranslateNew` may lead to a network request. `Collapse` and `Recall` cannot reach
`trigger_translation()`, which is what makes the documented "press again to recall or hide"
behaviour true in both modes instead of only in automatic mode.

### Clipboard decision

`ClipboardResumeState::tick(mode_enabled) -> ClipboardRunState` owns the mode gate and the
pause/resume transition:

| Input | Outcome | Caller action |
|---|---|---|
| Automatic translation disabled | `Paused` | Perform no clipboard work at all |
| First enabled tick after a pause | `ResumeBaseline` | Record the sequence, translate nothing |
| Enabled and already baselined | `Running` | Evaluate the observed text |

`decide_clipboard_tick(suppressed, duplicate) -> MonitorAction` then answers whether the observed
text may be dispatched — `Skip` for Aura Guard suppressions, self-copies, and already-dispatched
text, `Dispatch` otherwise.

The resume baseline exists because of a defect the first implementation shipped. The monitor used
to clear its stored sequence while paused; since nothing samples the pasteboard in that state, the
stored number was unknown on resume, so the very next sample always looked like a fresh clipboard
change and the text copied during the pause was translated the moment automatic translation came
back — the opposite of the documented behaviour. Requiring one baseline sample makes the first
observed sequence a reference point rather than a change: text copied during the pause is never
sent, and text copied after the resume still is.

### Readiness

`is_ready(config)` wraps `readiness::is_translation_ready` so the hotkey path and the readiness
module cannot drift apart.

## Architecture

A pure decision layer with no managed state, no I/O, and no Tauri dependency. It sits between the
runtime shell and the translation engine:

```
hotkey / clipboard monitor          interaction.rs              translate.rs
  (lib.rs, I/O + Tauri)      ->   (pure decisions)   ->   (network + streaming)
```

Callers gather context, ask for a decision, and perform the effect. The module never performs the
effect itself, which is precisely what keeps it testable.

### Backend (`src-tauri/src/interaction.rs`)

- `RequestIdentity`, `RequestIdentity::from_config()`
- `HotkeyContext`, `HotkeyOutcome`, `decide_hotkey()`
- `MonitorAction`, `decide_clipboard_tick()`
- `is_ready()`
- `mod tests`: 9 unit tests covering every branch, the identity comparison across all seven fields,
  and the disabled-monitor guarantee

### Callers

- `lib.rs::handle_hotkey_pressed()` builds the context and dispatches on the outcome
- `lib.rs::spawn_clipboard_monitor()` builds the tick decision for both the macOS and Windows loops
- `lib.rs::trigger_translation()` and the monitor write `last_request_identity`
- `ui/lib/windowBehavior.ts` holds the frontend counterpart decisions (blur dismissal, Escape
  dismissal, user-resize recording, auto-fit), also pure and separately tested

### Integration Points

- `src-tauri/src/config.rs`: `Provider::as_str()` supplies the stable provider identifier used in
  the identity
- `src-tauri/src/readiness.rs`: `is_translation_ready` is the readiness gate
- `scripts/release/accept-interaction-macos.sh`: asserts the same rules at the HTTP level by
  counting requests against a stub provider

## Current Limitations

- The identity stores the resolved language, provider, model, and endpoint, not a hash, so it is
  compared field by field on each hotkey press. This is cheap and readable at current sizes.
- Reuse is scoped to a single process lifetime. Restarting Aura does not carry a cached result, so
  the first hotkey press after a restart with unchanged clipboard text issues a request.
- The module does not model window position, focus, or multi-display state; those remain the window
  layer's responsibility.
- `HotkeyOutcome::Collapse` intentionally leaves an in-flight request running. There is no separate
  outcome for "collapse and cancel" because an explicit close already provides that.

## Future Directions

- Extend the identity if a second request-affecting input is added (for example a temperature or
  system-prompt setting), rather than adding an ad-hoc comparison at the call site.
- Move the frontend window decisions into this contract if a third platform needs them, so all
  interaction rules live in one reviewable place.
- Consider surfacing the decision outcome in opt-in `AURA_TRACE` output, so a real-host session can
  show why a press translated instead of collapsing.
