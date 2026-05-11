# Aura Translation

A lightweight, cross-platform floating desktop translation utility. Runs silently as a background daemon and is invoked via a global hotkey — it reads your clipboard, streams a translation token-by-token, and dismisses itself when you're done.

## Features

- **Zero-friction trigger** — Global hotkey (`Ctrl+T`) reads the clipboard instantly
- **Streaming output** — Token-by-token translation via the DeepSeek API
- **Bidirectional language pairs** — 11 languages with one-click swap
- **Glassmorphic UI** — Transparent, always-on-top floating popup near the system tray
- **System tray daemon** — Runs silently in the background; no dock/taskbar footprint
- **Auto-dismiss** — Closes on `Esc` or focus loss

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop shell | [Tauri 2](https://v2.tauri.app/) + Rust |
| Frontend | [SvelteKit](https://kit.svelte.dev/) + Svelte 5 |
| Styling | [Tailwind CSS v4](https://tailwindcss.com/) |
| Translation API | [DeepSeek](https://platform.deepseek.com/) (streaming SSE) |

## Project Structure

```
Aura-Translation/
├── ui/                  # SvelteKit frontend
│   ├── app.css          # Tailwind + design tokens
│   ├── app.html         # HTML shell
│   ├── lib/             # Svelte components
│   │   ├── LanguageSelector.svelte
│   │   ├── SettingsPanel.svelte
│   │   ├── SkeletonLoader.svelte
│   │   └── TranslationPopup.svelte
│   └── routes/          # SvelteKit pages
│       └── +page.svelte
├── src-tauri/           # Rust backend (Tauri convention — name is fixed)
│   ├── src/
│   │   ├── config.rs    # Plaintext JSON config persistence
│   │   ├── lib.rs       # App entry: tray, hotkey, window management
│   │   ├── main.rs
│   │   └── translate.rs # DeepSeek streaming client
│   ├── capabilities/    # Tauri permission grants
│   ├── icons/
│   └── tauri.conf.json
└── static/
    └── favicon.png
```

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) (stable toolchain)

### Development

```bash
npm install
npm run tauri dev
```

### First-time Setup

1. After launching, right-click the **system tray icon → Settings**
2. Enter your [DeepSeek API key](https://platform.deepseek.com/api_keys)
3. Save — the key is stored in `aura-config.json` in the app data directory

### Usage

1. Select any text and copy it (`Ctrl+C`)
2. Press `Ctrl+T` — the popup springs open near the tray
3. Watch the translation stream in
4. Press `Esc` or click elsewhere to dismiss

## Build for Production

```bash
npm run tauri build
```

The installer will be output to `src-tauri/target/release/bundle/`.
