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

  // Request lifecycle — monotonically incrementing ID for cancellation
  let currentRequestId = $state(0);
  let loadingTimeoutId: ReturnType<typeof setTimeout> | null = null;

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
    provider: 'deepseek' | 'openrouter' | 'ollama';
    api_base_url: string;
    available_models: string[];
  };

  let config: AppConfig = $state({
    api_key: '',
    model: 'deepseek-chat',
    source_lang: 'auto',
    target_lang: 'Chinese',
    hotkey: 'CmdOrCtrl+T',
    provider: 'deepseek',
    api_base_url: 'https://api.deepseek.com',
    available_models: ['deepseek-chat', 'deepseek-reasoner'],
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

  // ── Loading Timeout ────────────────────────────────────────────
  function startLoadingTimeout() {
    clearLoadingTimeout();
    loadingTimeoutId = setTimeout(() => {
      if (appState === 'loading') {
        appState = 'error';
        errorMessage = 'No response from API (timeout)';
      }
    }, 20_000);
  }

  function clearLoadingTimeout() {
    if (loadingTimeoutId !== null) {
      clearTimeout(loadingTimeoutId);
      loadingTimeoutId = null;
    }
  }

  // ── Cancellation ──────────────────────────────────────────────
  async function cancelCurrentTranslation() {
    if (currentRequestId > 0) {
      try {
        await invoke('cancel_translate', { requestId: currentRequestId });
      } catch (e) {
        console.error('Failed to cancel translation:', e);
      }
    }
    clearLoadingTimeout();
  }

  /** Start a new translation with the current sourceText and language pair. */
  async function startTranslation() {
    if (!config.api_key) {
      appState = 'error';
      errorMessage = 'No API key configured. Right-click the tray icon → Settings.';
      return;
    }

    currentRequestId++;
    const requestId = currentRequestId;
    translatedText = '';
    errorMessage = '';
    appState = 'loading';

    startLoadingTimeout();

    try {
      await invoke('translate_text', {
        text: sourceText,
        sourceLang: sourceLang,
        targetLang: targetLang,
        apiKey: config.api_key,
        model: config.model,
        requestId: requestId,
        apiBaseUrl: config.api_base_url,
        provider: config.provider,
      });
    } catch (e) {
      // appState may have been changed to 'error' by a translation-error event
      // during the await — avoid overwriting with a less specific message
      if ((appState as string) !== 'error') {
        appState = 'error';
        errorMessage = String(e);
      }
    }
  }

  // ── Event Handlers ────────────────────────────────────────────
  async function handleLanguageChange(source: string, target: string) {
    sourceLang = source;
    targetLang = target;

    // Mid-stream language switch: cancel current and retranslate immediately
    if ((appState === 'loading' || appState === 'streaming') && sourceText) {
      await cancelCurrentTranslation();
      await startTranslation();
    }
  }

  async function handleCancel() {
    await cancelCurrentTranslation();
    // Transition to result state — partial text is preserved
    appState = translatedText ? 'result' : 'idle';
  }

  async function dismiss() {
    // Cancel any in-flight translation before hiding the window
    await cancelCurrentTranslation();

    // Animate out
    popupScale.target = 0.92;
    popupOpacity.target = 0;

    clearLoadingTimeout();

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

      // Cancel any in-flight translation
      await cancelCurrentTranslation();

      // Reset state
      sourceText = text;
      showSettings = false;

      // Spring popup in (haptic overshoot effect)
      popupScale.target = 1;
      popupOpacity.target = 1;

      // Reload config to get latest API key
      await loadConfig();

      // Start streaming translation
      await startTranslation();
    });

    // Listen for streaming chunks
    listen<string>('translation-chunk', (event) => {
      if (appState === 'loading') {
        appState = 'streaming';
        clearLoadingTimeout(); // First token arrived — no longer at risk of timeout
      }
      if (appState === 'streaming') {
        translatedText += event.payload;
      }
    });

    // Listen for stream completion
    listen('translation-done', () => {
      clearLoadingTimeout();
      if (appState === 'loading' || appState === 'streaming') {
        appState = 'result';
      }
    });

    // Listen for errors
    listen<string>('translation-error', (event) => {
      clearLoadingTimeout();
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

    // Listen for hotkey conflict notification from daemon
    listen<{ hotkey: string; error: string }>('hotkey-conflict', (event) => {
      console.error('Hotkey conflict:', {
        hotkey: event.payload.hotkey,
        error: event.payload.error,
      });
      // TODO: surface as a user-visible warning overlay in Daemon-Core Phase 5
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
      oncancel={handleCancel}
    />

    <SettingsPanel
      visible={showSettings}
      onclose={() => showSettings = false}
    />
  </div>
</div>

