# Windows Trial Checklist

This checklist is the release gate for the first Windows user-trial MVP.

## Automated Gates

Run from the repo root:

```powershell
npm run check
cargo test --manifest-path .\src-tauri\Cargo.toml
npm test
npm run release:smoke-providers
npm run release:accept-install
npm run release:measure-resources
```

Generated artifacts are written to `artifacts/windows-trial/`:

- `smoke-providers.json`
- `accept-install.json`
- `resource-metrics.json`
- `ci-summary.json`

The repository also includes a Windows CI workflow at [.github/workflows/windows-trial.yml](/Users/maple/Github/Aura-Translation/.github/workflows/windows-trial.yml). It runs `check`, Rust tests, UI tests, bundle build, install acceptance, and resource measurement on `windows-latest`. Live provider smoke remains opt-in because it needs a real DeepSeek key and a reachable Ollama endpoint. For `codex/*` push branches, the workflow also commits `artifacts/windows-trial/ci-summary.json` back to the branch so the gate result can be fetched with plain Git even when GitHub Actions API access is unavailable.

Required CI gates:

- NSIS and MSI installer sizes are both `< 10 MB`
- Silent install, launch, config creation, uninstall all pass
- Idle max Working Set is `<= 20 MB`
- `check`, Rust tests, UI tests, bundle build, install acceptance, and resource measurement all pass in the Windows workflow summary

Release sign-off extras:

- DeepSeek live smoke passes with `AURA_SMOKE_DEEPSEEK_API_KEY`
- Ollama live smoke passes against `http://localhost:11434` or `AURA_SMOKE_OLLAMA_BASE_URL`

## Manual Verification

Complete these checks on the installed build:

1. Confirm the tray icon appears after launch.
2. Right-click the tray icon and open **Settings**.
3. Switch to `DeepSeek` or `OpenRouter` and confirm the plaintext API key warning is visible.
4. Enter an invalid API key and confirm the translation failure is visible in the popup and notification layer.
5. Enter a valid API key, copy source text, and trigger the hotkey.
6. Confirm the translation streams and finishes successfully.
7. Change the hotkey to an in-use shortcut and confirm the conflict appears both in the notification layer and inline in Settings.
8. Try capturing a bare single-key hotkey such as `T` and confirm the Settings UI rejects it before save.
9. Restore a valid hotkey and confirm the warning clears after save.

## Notes

- This trial gate is Windows-only.
- OpenRouter remains supported but is not a blocking smoke gate for this release.
- API keys are intentionally still stored in plaintext `config.json` for this trial; that is why the warning must remain visible in Settings.
- Hotkeys for this release must include at least one modifier key plus a letter or digit.
- Release builds rely on the Rust release profile in `src-tauri/Cargo.toml` (`lto`, `strip`, `opt-level = "s"`) to stay within the installer-size target.
