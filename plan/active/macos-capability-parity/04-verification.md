# Verification Matrix

## Automated Checks

| Check | Windows | macOS | Evidence |
|---|---|---|---|
| `npm run check` | NOT RUN | NOT RUN | |
| `npm test` | NOT RUN | NOT RUN | |
| `cargo test --manifest-path src-tauri/Cargo.toml` | NOT RUN | NOT RUN | |
| `npm run tauri build` | OPTIONAL / NOT RUN | NOT RUN | |
| Windows Trial Gate | NOT RUN | N/A | |
| macOS CI gate | N/A | NOT RUN | |

## Manual Platform Checks

| Behavior | Required Host | Status | Evidence |
|---|---|---|---|
| Aura mode can be enabled in Settings | macOS | NOT RUN | |
| Aura mode auto-translates fresh clipboard text | macOS | NOT RUN | |
| Duplicate clipboard suppression still works | macOS | NOT RUN | |
| Sensitive clipboard guard still blocks credential-like text | macOS | NOT RUN | |
| Paste-back succeeds in a real source app after permission grant | macOS | NOT RUN | |
| Permission-denied or unavailable state is actionable | macOS | NOT RUN | |
| Manual hotkey translation still works | macOS | NOT RUN | |
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
