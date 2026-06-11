# Translation Profiles Plan

## Objective

Translation Profiles will become a first-class multi-credential, fully portable switching system. When all phases are complete, users can hold independent API keys per profile (even for the same provider), reorder profiles as they see fit, switch profiles directly from the translation bubble, and export or import their entire profile set as a portable JSON document. The system will surface all I/O errors consistently and enforce no silent fallbacks.

## Design Principles

- **At least one profile always exists**: deletion must never leave the user without a profile; the minimum-of-one guard must be enforced in both the backend and the UI.
- **Active profile always reflects AppConfig**: after any profile activation or settings save, the in-memory config and the stored profile snapshot must be consistent.
- **Profile IDs are stable and unique**: IDs must not change after creation; the `profile-{ms}-{counter}` scheme guarantees uniqueness across rapid creates.
- **Writes are always atomic**: `profiles.json` must only be updated via the write-then-rename pattern.
- **Errors are surfaced, not swallowed**: parse and write failures must route through `daemon-error`, not `eprintln!`.
- **Provider-independent key isolation is a goal**: two profiles for the same provider should eventually be able to carry separate credentials.

## Phase 1: Structured Error Routing — DONE

Status: **Done**

Goals:

- Replace `eprintln!` in `TranslationProfilesStore::load()` with structured `daemon-error` events.
- Ensure all profile I/O failures are visible to the user in the notification center.

Completed work:

- `TranslationProfilesStore::load_with_issues()` now returns startup issues instead of logging with `eprintln!`.
- `lib.rs` emits those issues through `daemon-error` during `setup()` while preserving the existing empty-store fallback.
- When no Aura window is visible, startup profile issues also surface via background OS notifications.

## Phase 2: Per-Profile Independent API Keys — NOT STARTED

Status: **Not Started**

Goals:

- Allow two profiles for the same provider to hold independent API keys in the system keychain.
- Change the keychain account key from `"provider:deepseek"` to `"profile:{profile_id}:provider:deepseek"` for new system-stored profiles.

Remaining features:

- Define a new keychain account naming scheme that includes the profile ID.
- Write a migration that upgrades existing `"provider:{name}"` keychain entries on first activation of a profile that uses `api_key_storage: System`.
- Update `secrets::persist_api_key` and `secrets::load_provider_api_key` to accept an optional `profile_id` argument.
- Update `TranslationProfile::from_config` and `apply_to_config` to use the new account naming scheme.
- Expose the per-profile key in the Settings profile editor so users can edit the key associated with a non-active profile without activating it first.

### Key Naming Migration

Existing single-provider keychain entries (`"provider:deepseek"`) become the fallback for profiles that pre-date this change. On `activate()`, if a per-profile keychain entry is not found, fall back to the legacy provider-scoped entry and migrate it to the per-profile key on next save.

## Phase 3: Profile Ordering — NOT STARTED

Status: **Not Started**

Goals:

- Allow users to reorder profiles via drag-to-reorder in the Settings panel.
- Persist the user-defined order in `profiles.json`.

Remaining features:

- Add a drag-to-reorder interaction to `ProfileManager.svelte`.
- Expose a `reorder_translation_profiles(ordered_ids: Vec<String>)` Tauri command that reorders the `profiles` array and saves.
- Validate that all provided IDs match existing profiles before reordering; reject with an error if any ID is unknown.

## Phase 4: Profile Export and Import — NOT STARTED

Status: **Not Started**

Goals:

- Allow users to export all profiles (excluding system-stored keys) as a portable JSON file.
- Allow users to import a profiles JSON file, merging with or replacing the current profiles.

Remaining features:

- Add `export_translation_profiles()` Tauri command: serialises the profiles list, stripping `api_key` for `System`-stored profiles, and opens a save-file dialog.
- Add `import_translation_profiles(mode: "merge" | "replace")` Tauri command: reads a user-selected file, validates the schema, and either merges (adding new profiles with new generated IDs) or replaces the profiles list.
- Show a confirmation dialog in the Settings panel before replacing the profiles list.
- Document the export schema so users can manually edit the JSON.

## Phase 5: Quick-Switch in the Translation Bubble — NOT STARTED

Status: **Not Started**

Goals:

- Surface profile switching directly in the translation bubble so users can change profiles without opening Settings.
- Keep the tray menu as a secondary entry point.

Remaining features:

- Add a compact profile selector dropdown to `TranslationWindowView.svelte` (visible only when more than one profile exists).
- Emit an `activate_translation_profile` invoke from the bubble when the user selects a different profile.
- Re-trigger translation with the new profile immediately after activation if the bubble is currently showing a result.
- Persist the activated profile to `profiles.json` (reuse the existing `activate_translation_profile` Tauri command).

## Implementation Rules

- Do not allow deletion of the last remaining profile; the guard is in `TranslationProfilesStore::delete()` and must not be bypassed.
- Do not call `profiles.save()` except through the public mutation methods (`create_and_activate`, `rename`, `activate`, `delete`, `sync_active_profile_from_config`); raw saves from `lib.rs` are not allowed.
- Do not insert new profiles at the end of the list; always insert at index 0 so the newest profile appears first.
- Do not silently ignore write failures from `TranslationProfilesStore::save()`; propagate `Err(String)` to the calling Tauri command.

## Open Questions

- **Should the export format include `api_key_storage` mode?** If yes, importing a `plaintext_fallback` profile on a machine that supports system storage should prompt the user to migrate the key. If no, all imported profiles default to `system` storage.
- **What is the right merge strategy for import?** Should duplicate profile names be auto-renamed or rejected? Should the `active_profile_id` be preserved from the import file or kept as the current one?
- **Should profile switching from the translation bubble retranslate automatically?** Immediate retranslation is convenient but may be surprising; a manual "translate now" button after switching might be safer.
