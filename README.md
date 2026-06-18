# Aura Translation

Aura Translation is a local-first desktop translator built with Tauri, Rust, and Svelte. It runs from the system tray, translates copied text through OpenAI-compatible providers, and keeps the translation bubble available only when you need it.

The app is designed for desktop reading and writing workflows: copy text, translate it with a hotkey or Aura mode, pin the result for comparison, revise the source draft inline, and keep recent translation history locally.

## Features

- **Aura mode**: Optional clipboard watcher translates fresh copied text automatically on supported Windows and macOS builds.
- **Hotkey recall**: Configurable global hotkey translates the current clipboard text and toggles the latest result.
- **Streaming output**: Token-by-token translation through DeepSeek, OpenRouter, Ollama, or another OpenAI-compatible endpoint.
- **Bidirectional language pairs**: 11 languages with one-click source/target swap.
- **Floating translation bubble**: Minimal result window appears near the cursor and auto-sizes to translated text.
- **Pinned comparison mode**: Keep the bubble visible, draggable, and position-persistent while you compare source and output.
- **Pinned draft composer**: In pinned mode, revise the source text inline and press `Cmd/Ctrl+Enter` to translate again.
- **Separate settings window**: Provider, language pair, hotkey, Aura mode, pin behavior, profiles, and history live outside the translation bubble.
- **Named translation profiles**: Save multiple provider/language setups and switch from Settings or the tray menu.
- **Token usage visibility**: Show prompt, completion, and total token counts when the provider streams usage metadata.
- **Recent translation history**: Keep the latest 50 local results with copy, retry, delete, and clear actions.
- **System tray daemon**: Runs quietly in the background with tray recall and native notifications.
- **Aura Guard**: Optionally skips clipboard text that looks like credentials before an Aura-mode request is sent.

Source-app paste-back was intentionally removed from the supported product scope. Result actions now rely on explicit copy.

## Platform Status

| Platform | Status | Notes |
|---|---|---|
| macOS | Supported for current development | Aura mode uses native pasteboard polling; API keys use Keychain when system storage is selected. |
| Windows | Supported by project workflows | Aura mode, hotkey translation, credential storage, and installer checks are covered by Windows workflow scripts. |
| Linux | Not supported yet | Tauri may build in parts, but credential storage and desktop behavior are not productized. |

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop shell | [Tauri 2](https://v2.tauri.app/) + Rust |
| Frontend | [SvelteKit](https://kit.svelte.dev/) + Svelte 5 |
| Styling | [Tailwind CSS v4](https://tailwindcss.com/) |
| Translation providers | DeepSeek, OpenRouter, Ollama, or another OpenAI-compatible endpoint |

## Repository Layout

```text
Aura-Translation/
|-- ui/                         # SvelteKit frontend
|   |-- app.css                 # Tailwind + design tokens
|   |-- app.html                # HTML shell
|   |-- lib/                    # Svelte components and UI helpers
|   `-- routes/                 # SvelteKit route entry
|-- src-tauri/                  # Rust backend and Tauri app shell
|   |-- src/
|   |   |-- aura_guard.rs       # Clipboard credential screening for Aura mode
|   |   |-- capabilities.rs     # Backend-reported desktop capability state
|   |   |-- config.rs           # Preferences and provider defaults
|   |   |-- history.rs          # Local translation history
|   |   |-- hotkey.rs           # Configurable hotkey parser
|   |   |-- profiles.rs         # Named translation profiles
|   |   |-- readiness.rs        # Setup checklist and provider probe
|   |   |-- secrets.rs          # OS credential-store integration
|   |   |-- translate.rs        # OpenAI-compatible streaming client
|   |   `-- lib.rs              # Tray, hotkey, window, and command wiring
|   |-- capabilities/           # Tauri permission grants
|   |-- icons/
|   `-- tauri.conf.json
|-- docs/                       # Current module docs and release checklists
|-- plan/                       # Planning records and completed work archives
|-- scripts/release/            # Windows release-gate helper scripts
|-- .github/workflows/          # macOS and Windows CI workflows
`-- static/
    `-- favicon.png
```

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) stable
- Platform prerequisites from the [Tauri setup guide](https://v2.tauri.app/start/prerequisites/)

### Development

```bash
npm install
npm run tauri dev
```

### First-Time Setup

1. Launch the app. If Aura is not ready yet, it opens Settings automatically. You can also open Settings from the tray menu.
2. Choose a provider: DeepSeek, OpenRouter, Ollama, or another OpenAI-compatible endpoint.
3. Enter an API key for DeepSeek/OpenRouter-style providers. Ollama does not require one.
4. Pick the default language pair, hotkey, Aura mode preference, pin behavior, and optional named profiles.
5. Save settings. Aura stores preferences in the OS config directory and stores API keys in the system credential store by default.

### Usage

1. Copy text with `Ctrl+C` or `Cmd+C`.
2. If Aura mode is enabled, fresh clipboard text is translated automatically when supported.
3. If Aura mode is disabled, press the configured hotkey to translate the current clipboard text.
4. Copy the translated result from the bubble when you want to use it elsewhere.
5. Press the hotkey again to recall or hide the latest bubble, or left-click the tray icon to reopen the latest result.
6. Pin the bubble to keep it visible while reading other pages, or revise the source draft inline and re-translate with `Cmd/Ctrl+Enter`.

## Privacy and Security

- Aura has no hosted backend in this repository. Translation requests are sent from your machine to the provider endpoint you configure.
- API keys default to the OS credential store on supported desktop builds. Plaintext config storage exists only as an explicit fallback.
- Preferences and recent translation history are stored locally on the user's machine.
- Aura Guard can skip clipboard text that looks like passwords, API keys, JWTs, or other credentials before an automatic Aura-mode translation starts.
- Do not commit real API keys, provider tokens, generated config files, or release artifacts.

## Scripts

```bash
npm run check
npm test
npm run build
npm run tauri build
```

The production installer is emitted under `src-tauri/target/release/bundle/`.

Windows release-gate helpers live in `scripts/release/`:

```bash
npm run release:accept-install
npm run release:measure-resources
npm run release:smoke-providers
```

Use [docs/Windows-Trial-Checklist.md](docs/Windows-Trial-Checklist.md) for Windows trial release sign-off and [docs/macOS-Adaptation-Checklist.md](docs/macOS-Adaptation-Checklist.md) for macOS behavior checks.

## Documentation

- [docs/README.md](docs/README.md) lists the current module descriptions, plans, and release checklists.
- [plan/README.md](plan/README.md) explains the planning archive used during cross-platform development.
- GitHub Releases contain user-facing builds and release notes.

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request, and use [SECURITY.md](SECURITY.md) for vulnerability reporting.

## License

Aura Translation is released under the [MIT License](LICENSE).
