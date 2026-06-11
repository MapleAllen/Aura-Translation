# Runtime Readiness Plan

## Objective

Runtime Readiness will become a proactive, internationalised, and efficient health-check system. When all phases are complete, the module will cache probe results to avoid rate-limit exhaustion, surface readiness changes in the tray icon without requiring the user to open Settings, and expose fully internationalised status messages. A periodic background health check will detect connectivity degradation during long sessions.

## Design Principles

- **Readiness checks are always synchronous**: `is_translation_ready` and `build_runtime_status` must never perform I/O; they inspect the in-memory config only.
- **Probe is async and guarded**: `probe_provider` must always validate preconditions before making a network call; it must never initiate a request when any required field is missing.
- **Labels belong to the frontend**: status labels and error messages should move to the TypeScript layer over time to support internationalisation without recompiling the Rust backend.
- **Results are not stored in app config**: probe results and runtime status are transient; they must not be persisted to disk.

## Phase 1: Probe Result Caching and Debounce — NOT STARTED

Status: **Not Started**

Goals:

- Prevent rapid successive probe calls from exhausting provider rate limits.
- Return a cached result within a short TTL window.

Remaining features:

- Add a `last_probe_result: Option<(Instant, ProviderProbeResult)>` field to a new `ReadinessState` managed state.
- Return the cached result if it is less than 30 seconds old and the config has not changed since the last probe.
- Cache invalidation: invalidate the cached result whenever `save_config` is called (config change invalidates the probe).
- Expose `ReadinessState` as a Tauri managed state so it is accessible from `probe_provider_connection`.

## Phase 2: Tray Icon Readiness Badge — NOT STARTED

Status: **Not Started**

Goals:

- Reflect the current readiness level in the tray icon so users can see Aura's status at a glance without opening Settings.
- Update the tray icon when readiness changes (after settings save or startup probe).

Remaining features:

- Add two tray icon variants: a ready icon and a warning/needs-setup icon.
- Call `readiness::build_runtime_status` after every `save_config` and update the tray icon based on `level`.
- Update the tray tooltip to include the `summary` string from `RuntimeStatus`.

## Phase 3: Internationalised Status Labels — NOT STARTED

Status: **Not Started**

Goals:

- Move all human-readable strings from `readiness.rs` to the TypeScript frontend layer.
- Replace the Chinese-only checklist labels and summary strings with structured codes that the frontend maps to localised strings.

Remaining features:

- Replace `label` strings in `RuntimeChecklistItem` with the `code` field (already present); the frontend derives the display label from `code`.
- Replace `summary` in `RuntimeStatus` with a structured `reason_code: String` (e.g. `"missing_api_key"`, `"missing_model"`) and let the frontend format the message.
- Remove the `provider_label()` and `missing_setup_message()` helper functions from `readiness.rs` after the frontend takes over label formatting.
- Update `SetupStatusCard.svelte` and `SettingsPanel.svelte` to map `code` values to display strings.

## Phase 4: Periodic Background Health Check — NOT STARTED

Status: **Not Started**

Goals:

- Detect provider connectivity degradation during long Aura sessions without requiring the user to open Settings.
- Notify the user via `daemon-error` or a tray notification when the provider becomes unreachable.

Remaining features:

- Add a background interval (default: every 5 minutes when the translation window is not visible) that calls `probe_provider` and compares the result to the last known state.
- Emit a `daemon-error` event when the probe transitions from `ok: true` to `ok: false`.
- Emit a `provider-recovered` event (or update the tray icon) when the probe transitions from `ok: false` to `ok: true`.
- Make the polling interval configurable via `AppConfig` (range: 1–60 minutes, default: 5).
- Pause polling while a translation is in progress to avoid interference.

## Implementation Rules

- Do not perform network I/O inside `is_translation_ready`, `should_prompt_for_setup`, or `build_runtime_status`; these functions must remain synchronous and side-effect free.
- Do not store probe results in `AppConfig` or `profiles.json`; they are transient and must not survive app restarts.
- Do not change `max_tokens: 4` in the probe request body; the goal is minimal quota consumption, not a real translation.

## Open Questions

- **Should the probe cache be invalidated on provider change only, or on any config field change?** Invalidating on any `save_config` is conservative but safe. A smarter approach that only invalidates on provider/base_url/api_key/model changes would avoid unnecessary re-probes when unrelated settings (e.g., language pair) change.
- **What is the right polling interval for Phase 4?** 5 minutes is a reasonable default, but the correct value depends on provider rate limits and user sensitivity to "unnoticed failure" scenarios.
- **Should tray readiness badges use separate icon files or a programmatic overlay?** Separate bundled icons are simpler and avoid runtime compositing; an overlay would allow more states without bundling every combination.
