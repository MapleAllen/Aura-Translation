# Translation History Plan

## Objective

Translation History will become a searchable, configurable, and exportable record of the user's translation activity. When all phases are complete, users can filter and search history without opening a separate panel, configure the maximum number of stored entries, export their history to a portable file, and retry failed translations with the exact provider and model that was originally used. All I/O failures will surface through the structured event system rather than `eprintln!`.

## Design Principles

- **Newest-first ordering is invariant**: entries must always be stored and displayed newest-first; no sorting operation should change this contract.
- **Hard cap is enforced on every insert**: the store must never exceed `MAX_HISTORY_ENTRIES` after any insert; truncation happens inside `insert_entry`, not at read time.
- **Writes are always atomic**: `history.json` must only be written via the `history.json.tmp` rename pattern.
- **Usage is optional and backward-compatible**: the `usage` field on `TranslationHistoryEntry` must remain `Option` with `#[serde(default)]` so older entries without usage data deserialise cleanly.
- **Errors surface consistently**: history I/O failures must route through `daemon-error`, not `eprintln!`.
- **Retry faithfulness is a goal**: replaying a history entry should eventually be able to use the original provider and model, not only the current active config.

## Phase 1: Structured Error Routing — DONE

Status: **Done**

Goals:

- Replace `eprintln!` in `TranslationHistoryStore::load()` with structured `daemon-error` events.
- Ensure all history I/O failures are visible to the user.

Completed work:

- `TranslationHistoryStore::load_with_issues()` now returns startup issues instead of logging with `eprintln!`.
- `lib.rs` emits those issues through `daemon-error` during `setup()` while preserving the existing empty-store fallback.
- When no Aura window is visible, startup history issues also surface via background OS notifications.

## Phase 2: Search and Filter — DONE

Status: **Done**

Goals:

- Add text search and status/language filters to `HistoryList.svelte`.
- Allow users to find specific past translations without scrolling through the full list.

Completed work:

- Add a search input above the history list in `HistoryList.svelte`; filter entries client-side by matching `source_text` and `translated_text`.
- Add a status toggle (All / Success / Error) to the filter bar.
- Add a language-pair filter dropdown populated from the unique language pairs present in the history list.
- Display a "No results" state when filters produce an empty match set.
- Reset filters when the history list is cleared.

## Phase 3: Configurable History Cap — NOT STARTED

Status: **Not Started**

Goals:

- Allow users to configure the maximum number of stored history entries.
- Respect the configured cap on every insert without requiring a restart.

Remaining features:

- Add `history_max_entries: usize` to `AppConfig` with a default of `50`.
- Pass the cap from config into `TranslationHistoryStore` and use it inside `insert_entry` instead of the compile-time `MAX_HISTORY_ENTRIES` constant.
- Expose the setting in the Settings panel (a numeric input with min 10, max 500).
- On cap reduction, prune the existing list to the new cap and save immediately.

## Phase 4: Retry with Original Provider — DONE

Status: **Done**

Goals:

- Allow users to replay a history entry using the exact provider and model recorded in the entry, not the current active profile.
- Make the retry intent explicit in the UI.

Completed work:

- Add a `retry_with_original` flag to the `onretry` callback in `HistoryList.svelte`.
- Pass the `provider`, `model`, and `api_base_url` fields from the entry to the translation invocation when `retry_with_original` is true.
- `replay_translation_history_entry` emits an explicit one-shot request config containing the stored provider, model, base URL, language pair, and resolvable profile-scoped credential.
- Show the entry's provider and model in the retry confirmation tooltip so users can see what will be used.

## Phase 5: History Export — NOT STARTED

Status: **Not Started**

Goals:

- Allow users to export their translation history to a JSON or CSV file.
- Support exporting the full list or only the filtered result set.

Remaining features:

- Add an `export_translation_history(format: "json" | "csv")` Tauri command: serialises the history list and opens a save-file dialog.
- CSV format: columns for `created_at_ms` (ISO date), `status`, `source_lang`, `target_lang`, `provider`, `model`, `source_text`, `translated_text`, `error_message`, `total_tokens`.
- JSON format: the existing `TranslationHistoryEntry` serialisation with human-readable timestamp added.
- Expose an "Export history" button in the Settings panel history section.

## Implementation Rules

- Do not allow `entries` in `TranslationHistoryStore` to exceed `MAX_HISTORY_ENTRIES` (or the configured cap) at any point after an insert; truncation must happen inside `insert_entry` before `save()` is called.
- Do not remove `#[serde(default)]` from `TranslationHistoryEntry.usage`; older history files must continue to deserialise correctly.
- Do not expose a `record_*` method that takes a full `TranslationHistoryEntry` from the outside; callers must use `record_success` or `record_error` so the ID and timestamp are always generated internally.
- Do not reorder entries outside of `insert_entry`; the list must remain newest-first at all times.

## Open Questions

- **Should the search filter run on the backend or the frontend?** For 50 entries, frontend filtering is sufficient. If the cap becomes configurable up to 500, backend filtering with a dedicated command might be more responsive.
- **Should "Retry with original provider" switch the active profile temporarily or issue a one-off translation?** Switching the active profile has side effects (tray menu, config state); a one-off translation with an override is cleaner but requires API changes.
- **What CSV encoding should be used for export?** UTF-8 with BOM is recommended for Excel compatibility on Windows; plain UTF-8 is preferred on macOS. Should this be configurable or always one format?
