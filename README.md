# Aura Translation

A lightweight cross-platform desktop translator that runs as a tray daemon. Trigger it with a global hotkey, stream results token by token, and pin the window only when you need side-by-side comparison.

## Features

- **Aura mode**: Optional Windows clipboard watcher translates fresh copied text automatically.
- **Hotkey recall**: Configurable global hotkey (default `CmdOrCtrl+T`) still works as a manual trigger and result recall toggle.
- **Streaming output**: Token-by-token translation via OpenAI-compatible providers.
- **Bidirectional language pairs**: 11 languages with one-click swap.
- **Floating translation bubble**: Minimal translator appears near the cursor and auto-sizes to the translated text.
- **Pinned comparison mode**: Pin the bubble to keep it visible, draggable, and position-persistent.
- **Separate settings tool window**: Provider, language pair, hotkey, Aura mode, and pin behavior live in a dedicated movable window.
- **Recent translation history**: Settings keeps the latest 50 successful or failed requests with copy, retry, delete, and clear actions.
- **System tray daemon**: Runs silently in the background with no taskbar footprint.
- **Tray recall and background alerts**: Left-click the tray icon to reopen the latest result or setup window, and get native notifications when hidden requests retry, fail, or finish.
- **Adaptive dismiss**: Press `Esc` to close; blur hides the translation bubble by default, while pinned mode stays visible.

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop shell | [Tauri 2](https://v2.tauri.app/) + Rust |
| Frontend | [SvelteKit](https://kit.svelte.dev/) + Svelte 5 |
| Styling | [Tailwind CSS v4](https://tailwindcss.com/) |
| Translation providers | DeepSeek, OpenRouter, Ollama, or another OpenAI-compatible endpoint |

## Project Structure

```text
Aura-Translation/
|-- ui/                  # SvelteKit frontend
|   |-- app.css          # Tailwind + design tokens
|   |-- app.html         # HTML shell
|   |-- lib/             # Svelte components and UI helpers
|   |   |-- LanguageSelector.svelte
|   |   |-- SettingsPanel.svelte
|   |   |-- SettingsWindowView.svelte
|   |   |-- SkeletonLoader.svelte
|   |   |-- TranslationWindowView.svelte
|   |   `-- TranslationPopup.svelte
|   `-- routes/
|       `-- +page.svelte
|-- src-tauri/           # Rust backend (Tauri convention, folder name is fixed)
|   |-- src/
|   |   |-- config.rs    # Plaintext JSON config persistence and provider defaults
|   |   |-- hotkey.rs    # Configurable hotkey parser
|   |   |-- lib.rs       # App entry: tray, hotkey, window management
|   |   |-- main.rs
|   |   `-- translate.rs # OpenAI-compatible streaming client
|   |-- capabilities/    # Tauri permission grants
|   |-- icons/
|   `-- tauri.conf.json
`-- static/
    `-- favicon.png
```

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) stable

### Development

```bash
npm install
npm run tauri dev
```

### First-time Setup

1. Launch the app. If Aura is not ready yet it opens `Settings` automatically; you can also right-click the tray icon and open `Settings` manually.
2. Choose a provider: DeepSeek, OpenRouter, or local Ollama.
3. Enter an API key for DeepSeek or OpenRouter. Ollama does not require one.
4. Pick your default language pair, hotkey, and whether Aura mode should auto-translate copied text.
5. Save the settings. Aura stores preferences in `{OS config dir}/aura-translation/config.json`, and keeps API keys in the system credential store by default with an explicit plaintext fallback if you choose it.

### Usage

1. Select any text and copy it with `Ctrl+C`.
2. If Aura mode is enabled on Windows, the translation bubble appears near the cursor automatically.
3. If Aura mode is disabled, press your configured hotkey (default `CmdOrCtrl+T`) to translate the current clipboard text.
4. Press the hotkey again to recall or hide the last translation bubble, or left-click the tray icon to reopen the latest result when Aura is in the background.
5. Pin the bubble to keep it visible while you read other pages.

## Build for Production

```bash
npm run tauri build
```

The installer is emitted to `src-tauri/target/release/bundle/`.

For the Windows trial release gate, use [docs/Windows-Trial-Checklist.md](docs/Windows-Trial-Checklist.md). The repo also includes a Windows GitHub Actions workflow at [.github/workflows/windows-trial.yml](.github/workflows/windows-trial.yml) for repeatable build, install, and resource checks. Live provider smoke and installed-build sign-off remain separate release steps.
