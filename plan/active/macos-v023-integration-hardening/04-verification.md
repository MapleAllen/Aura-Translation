# Verification Matrix

## Automated Checks

| Check | Windows | macOS | Evidence |
|---|---|---|---|
| Integration baseline ancestry | PASS | PASS | Local check on 2026-06-07: `git merge-base --is-ancestor ebe2162 6a826fb` succeeded, so `main -> v0.2.3` fast-forward is valid. |
| `npm run check` | PENDING ON INTEGRATION BRANCH | REPORTED PASS ON 2026-06-07 | Re-run on `codex/macos-v023-integration` after Windows fix lands if product SHA changes. |
| `npm test` | PENDING ON INTEGRATION BRANCH | REPORTED PASS ON 2026-06-07 | macOS docs currently mention passing UI tests; branch-level rerun still needs recording here. |
| `cargo test --manifest-path ./src-tauri/Cargo.toml` | BLOCKED BY KNOWN HOTKEY TEST ON `v0.2.3` UNTIL FIX | REPORTED PASS ON 2026-06-07 | Windows blocker: `platform_default_hotkey_is_parseable` asserts `KeyJ` everywhere in `v0.2.3`. |
| `npm run tauri build` | PENDING ON INTEGRATION BRANCH | REPORTED PASS ON 2026-06-07 | macOS build evidence belongs to the adaptation gate and checklist once updated. |
| Windows Trial Gate | FAILING BEFORE HARDENING | N/A | Must pass on `codex/macos-v023-windows-hardening` and later on `codex/macos-v023-integration`. |
| macOS Adaptation Gate | N/A | REPORTED PASS ON 2026-06-07 | Shared docs must be updated to reflect this reported state and the exact branch/SHA used. |

## Manual Platform Checks

| Behavior | Required Host | Status | Evidence |
|---|---|---|---|
| Tray/menu bar icon and Settings access | macOS | PENDING | To be recorded by the macOS implementer in `03-macos.md`. |
| Global hotkey manual translation path | macOS | PENDING | Must confirm `Cmd+Shift+J` behavior on a real host. |
| Keychain-backed credential storage | macOS | PENDING | Must use the Settings UI on a real host. |
| Native notifications for hidden flows | macOS | PENDING | Must verify completion, retry, and failure paths. |
| Transparent/borderless windows | macOS | PENDING | Requires visible runtime inspection. |
| Always-on-top pinned behavior | macOS | PENDING | Requires multi-window runtime inspection. |
| Aura mode disabled on macOS | macOS | PENDING | UI/state verification required. |
| Paste-back unavailable on macOS | macOS | PENDING | UI verification required. |
| Default hotkey remains `CmdOrCtrl+T` on Windows | Windows | PENDING / OPTIONAL | Only required if runtime hotkey logic changes outside the intended test fix. |

## Regression Coverage

- Existing Rust coverage in `src-tauri/src/hotkey.rs` and `src-tauri/src/config.rs` already covers default hotkey parsing and macOS legacy migration paths.
- This hardening effort must add or preserve coverage proving platform-specific default hotkey expectations.
- UI regressions remain covered by existing Svelte tests; rerun results must be captured on the hardening branch.

## Known Limitations

- Local repo state does not itself prove the 2026-06-07 macOS CI result; this plan currently treats that result as operator-reported evidence that still must be reflected in `docs/`.
- CI-summary-only commits on `codex/macos-adaptation-runtime` are evidence artifacts, not integration inputs.
