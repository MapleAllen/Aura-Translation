<script lang="ts">
  /**
   * +page.svelte — Root page orchestrating the popup lifecycle.
   *
   * Lifecycle:
   * 1. Daemon: hidden, waiting for hotkey
   * 2. Trigger: hotkey → read clipboard → spring popup in (overshoot)
   * 3. Loading: skeleton loader pulsates
   * 4. Streaming: tokens flow in one-by-one
   * 5. Result: full text displayed
   * 6. Dismiss: Esc/blur → shrink+fade → hide window
   */
  import { Spring } from 'svelte/motion';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import TranslationPopup from '$lib/TranslationPopup.svelte';
  import SettingsPanel from '$lib/SettingsPanel.svelte';
  import { onMount } from 'svelte';

  // ── State ──────────────────────────────────────────────────────
  let appState: 'idle' | 'loading' | 'streaming' | 'result' | 'error' = $state('idle');
  let sourceText = $state('');
  let translatedText = $state('');
  let errorMessage = $state('');
  let showSettings = $state(false);

  // Language pair (persisted via config)
  let sourceLang = $state('auto');
  let targetLang = $state('Chinese');

  // ── Spring Animations ─────────────────────────────────────────
  // Haptic popup: overshoot past 1.0 then settle
  const popupScale = new Spring(0.92, { stiffness: 0.14, damping: 0.68 });
  const popupOpacity = new Spring(0, { stiffness: 0.18, damping: 0.82 });

  // ── Config ────────────────────────────────────────────────────
  type AppConfig = {
    api_key: string;
    model: string;
    source_lang: string;
    target_lang: string;
    hotkey: string;
  };

  let config: AppConfig = $state({
    api_key: '',
    model: 'deepseek-v4-flash',
    source_lang: 'auto',
    target_lang: 'Chinese',
    hotkey: 'CmdOrCtrl+T',
  });

  async function loadConfig() {
    try {
      const loaded = await invoke<AppConfig>('get_config');
      config = loaded;
      sourceLang = config.source_lang;
      targetLang = config.target_lang;
    } catch (e) {
      console.error('Failed to load config:', e);
    }
  }

  // ── Event Handlers ────────────────────────────────────────────
  function handleLanguageChange(source: string, target: string) {
    sourceLang = source;
    targetLang = target;
  }

  async function dismiss() {
    // Animate out
    popupScale.target = 0.92;
    popupOpacity.target = 0;

    // Wait for animation, then hide window
    setTimeout(async () => {
      appState = 'idle';
      sourceText = '';
      translatedText = '';
      errorMessage = '';
      showSettings = false;
      try {
        const appWindow = getCurrentWindow();
        await appWindow.hide();
      } catch (e) {
        console.error('Failed to hide window:', e);
      }
    }, 220);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (showSettings) {
        showSettings = false;
      } else {
        dismiss();
      }
    }
  }

  // ── Lifecycle ─────────────────────────────────────────────────
  onMount(() => {
    loadConfig();

    // Listen for translation trigger from Rust
    listen<string>('trigger-translate', async (event) => {
      const text = event.payload;
      if (!text || !text.trim()) return;

      // Reset state
      sourceText = text;
      translatedText = '';
      errorMessage = '';
      appState = 'loading';
      showSettings = false;

      // Spring popup in (haptic overshoot effect)
      popupScale.target = 1;
      popupOpacity.target = 1;

      // Reload config to get latest API key
      await loadConfig();

      if (!config.api_key) {
        appState = 'error';
        errorMessage = 'No API key configured. Right-click the tray icon → Settings.';
        return;
      }

      // Start streaming translation
      try {
        await invoke('translate_text', {
          text,
          sourceLang: sourceLang,
          targetLang: targetLang,
          apiKey: config.api_key,
          model: config.model,
        });
      } catch (e) {
        if (appState !== 'error') {
          appState = 'error';
          errorMessage = String(e);
        }
      }
    });

    // Listen for streaming chunks
    listen<string>('translation-chunk', (event) => {
      if (appState === 'loading') {
        appState = 'streaming';
      }
      translatedText += event.payload;
    });

    // Listen for stream completion
    listen('translation-done', () => {
      appState = 'result';
    });

    // Listen for errors
    listen<string>('translation-error', (event) => {
      appState = 'error';
      errorMessage = event.payload;
    });

    // Listen for window blur (focus loss) → dismiss
    listen('window-blur', () => {
      if (!showSettings) {
        dismiss();
      }
    });

    // Listen for settings request from tray
    listen('show-settings', () => {
      showSettings = true;
      popupScale.target = 1;
      popupOpacity.target = 1;
    });
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="w-screen h-screen p-2">
  <div
    class="w-full h-full relative"
    style:transform="scale({popupScale.current})"
    style:opacity={popupOpacity.current}
    style="transform-origin: bottom right; will-change: transform, opacity;"
  >
    <TranslationPopup
      viewState={appState}
      {sourceText}
      {translatedText}
      {errorMessage}
      {sourceLang}
      {targetLang}
      onLanguageChange={handleLanguageChange}
    />

    <SettingsPanel
      visible={showSettings}
      onclose={() => showSettings = false}
    />
  </div>
</div>
