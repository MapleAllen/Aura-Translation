# Verification Matrix

## Automated Checks

| Check | Windows | macOS | Evidence |
|---|---|---|---|
| `npm run check` | NOT RUN | PASS | 2026-06-09 local run: `svelte-check found 0 errors and 0 warnings` |
| `npm test` | NOT RUN | PASS | 2026-06-09 local run: `9` test files / `47` tests passed |
| `cargo test --manifest-path src-tauri/Cargo.toml` | NOT RUN | PASS | 2026-06-09 local run: `53` Rust tests passed |
| `npm run tauri build` | OPTIONAL / NOT RUN | PASS | 2026-06-09 local run: produced `Aura Translation.app` bundle |
| Windows Trial Gate | NOT RUN | N/A | |
| macOS CI gate | N/A | NOT RUN | |

## Manual Platform Checks

| Behavior | Required Host | Status | Evidence |
|---|---|---|---|
| Aura mode can be enabled in Settings | macOS | PASS | 2026-06-09: Settings exposes Aura mode as a supported capability; saving persists `aura_mode_enabled: true` |
| Aura mode auto-translates fresh clipboard text | macOS | PASS | 2026-06-09: NSPasteboard polling detects clipboard changes and auto-triggers translation |
| Duplicate clipboard suppression still works | macOS | PASS | 2026-06-09: re-copying identical text does not retrigger until text changes |
| Sensitive clipboard guard still blocks credential-like text | macOS | PASS | 2026-06-09: `aura_guard::detect_sensitive_clipboard` blocks credential-like content |
| Paste-back succeeds in a real source app after permission grant | macOS | PASS | 2026-06-09: NSRunningApplication focus + CGEvent Cmd+V injection verified against a real source app |
| Permission-denied or unavailable state is actionable | macOS | PASS | 2026-06-09: UI shows permission tooltip when `needs_permission`; `request_accessibility_permission` triggers native prompt |
| Manual hotkey translation still works | macOS | PASS | 2026-06-09: Cmd+Shift+J triggers manual translation regardless of Aura mode state |
| Aura mode still works on Windows | Windows | NOT RUN | |
| Paste-back still works on Windows | Windows | NOT RUN | |

## Regression Coverage

- `ui/lib/SettingsPanel.test.ts` must stop encoding the stale assumption that macOS always forces `aura_mode_enabled` to `false`.
- Translation-window tests must still cover paste-back supported/unsupported status handling.
- Backend tests around config loading, hotkey defaults, and readiness must keep cross-platform builds healthy after capability changes.

## Known Limitations

- CI cannot prove cross-app macOS permission and paste-back behavior by itself.
- Non-text clipboard payload restore remains out of scope unless explicitly re-planned.
- Linux parity remains out of scope for this plan.
