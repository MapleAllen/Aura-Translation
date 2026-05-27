<script lang="ts">
  /**
   * +page.svelte - Root page orchestrating the popup lifecycle.
   */
  import { Spring } from 'svelte/motion';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import TranslationPopup from '$lib/TranslationPopup.svelte';
  import SettingsPanel from '$lib/SettingsPanel.svelte';
  import NotificationCenter from '$lib/NotificationCenter.svelte';
  import {
    createDaemonErrorNotification,
    createHotkeyConflictNotification,
    createTranslationErrorNotification,
    formatTranslationError,
    type AppNotification,
    type DaemonErrorPayload,
    type HotkeyConflictPayload,
  } from '$lib/notifications';
  import { onMount } from 'svelte';

  let appState: 'idle' | 'loading' | 'streaming' | 'result' | 'error' = $state('idle');
  let sourceText = $state('');
  let translatedText = $state('');
  let errorMessage = $state('');
  let showSettings = $state(false);
  let notifications = $state<AppNotification[]>([]);
  let hotkeyConflictMessage = $state('');

  let sourceLang = $state('auto');
  let targetLang = $state('Chinese');

  let currentRequestId = $state(0);
  let loadingTimeoutId: ReturnType<typeof setTimeout> | null = null;

  const popupScale = new Spring(0.92, { stiffness: 0.14, damping: 0.68 });
  const popupOpacity = new Spring(0, { stiffness: 0.18, damping: 0.82 });

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

  type TranslationChunkPayload = {
    request_id: number;
    content: string;
  };

  type TranslationDonePayload = {
    request_id: number;
  };

  type TranslationErrorPayload = {
    request_id: number;
    message: string;
  };

  type TranslationRetryPayload = {
    request_id: number;
    attempt: number;
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

  function pushNotification(notification: AppNotification) {
    notifications = [notification, ...notifications.filter((item) => item.scope !== notification.scope)]
      .slice(0, 3);
  }

  function dismissNotification(id: string) {
    notifications = notifications.filter((item) => item.id !== id);
  }

  function clearSettingsWarnings() {
    hotkeyConflictMessage = '';
    notifications = notifications.filter((item) => item.scope !== 'settings');
  }

  function showTranslationFailure(rawMessage: unknown) {
    const message = formatTranslationError(rawMessage);
    clearLoadingTimeout();
    appState = 'error';
    errorMessage = message;
    pushNotification(createTranslationErrorNotification(message));
  }

  async function loadConfig() {
    try {
      const loaded = await invoke<AppConfig>('get_config');
      config = loaded;
      sourceLang = config.source_lang;
      targetLang = config.target_lang;
    } catch (e) {
      console.error('Failed to load config:', e);
      pushNotification(
        createDaemonErrorNotification({
          code: 'config-load-failed',
          message: 'Failed to load the local Aura config from the desktop backend.',
          recoverable: true,
        }),
      );
    }
  }

  function startLoadingTimeout() {
    clearLoadingTimeout();
    loadingTimeoutId = setTimeout(() => {
      if (appState === 'loading') {
        showTranslationFailure('No response from API (timeout)');
      }
    }, 20_000);
  }

  function clearLoadingTimeout() {
    if (loadingTimeoutId !== null) {
      clearTimeout(loadingTimeoutId);
      loadingTimeoutId = null;
    }
  }

  function isCurrentRequest(requestId: number) {
    return requestId === currentRequestId;
  }

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

  async function startTranslation() {
    if (config.provider !== 'ollama' && !config.api_key) {
      showTranslationFailure('No API key configured. Right-click the tray icon -> Settings.');
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
        sourceLang,
        targetLang,
        apiKey: config.api_key,
        model: config.model,
        requestId,
        apiBaseUrl: config.api_base_url,
        provider: config.provider,
      });
    } catch (e) {
      if (isCurrentRequest(requestId) && errorMessage === '') {
        showTranslationFailure(e);
      }
    }
  }

  async function handleLanguageChange(source: string, target: string) {
    sourceLang = source;
    targetLang = target;

    if ((appState === 'loading' || appState === 'streaming') && sourceText) {
      await cancelCurrentTranslation();
      currentRequestId++;
      await startTranslation();
    }
  }

  async function handleCancel() {
    await cancelCurrentTranslation();
    currentRequestId++;
    appState = translatedText ? 'result' : 'idle';
  }

  async function dismiss() {
    void cancelCurrentTranslation();
    currentRequestId++;

    popupScale.target = 0.92;
    popupOpacity.target = 0;

    clearLoadingTimeout();

    setTimeout(async () => {
      await cancelCurrentTranslation();
      currentRequestId++;
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
        void dismiss();
      }
    }
  }

  onMount(() => {
    void loadConfig();

    const unlisteners: Array<() => void> = [];

    const register = async () => {
      unlisteners.push(
        await listen<string>('trigger-translate', async (event) => {
          const text = event.payload;
          if (!text || !text.trim()) return;

          await cancelCurrentTranslation();
          currentRequestId++;

          sourceText = text;
          showSettings = false;

          popupScale.target = 1;
          popupOpacity.target = 1;

          await loadConfig();
          await startTranslation();
        }),
      );

      unlisteners.push(
        await listen<TranslationChunkPayload>('translation-chunk', (event) => {
          if (!isCurrentRequest(event.payload.request_id)) return;

          if (appState === 'loading') {
            appState = 'streaming';
            clearLoadingTimeout();
          }
          if (appState === 'streaming') {
            translatedText += event.payload.content;
          }
        }),
      );

      unlisteners.push(
        await listen<TranslationDonePayload>('translation-done', (event) => {
          if (!isCurrentRequest(event.payload.request_id)) return;

          clearLoadingTimeout();
          if (appState === 'loading' || appState === 'streaming') {
            appState = 'result';
          }
        }),
      );

      unlisteners.push(
        await listen<TranslationErrorPayload>('translation-error', (event) => {
          if (!isCurrentRequest(event.payload.request_id)) return;
          showTranslationFailure(event.payload.message);
        }),
      );

      unlisteners.push(
        await listen<TranslationRetryPayload>('translation-retry', (event) => {
          if (!isCurrentRequest(event.payload.request_id)) return;
          console.warn('Retrying translation request', event.payload);
        }),
      );

      unlisteners.push(
        await listen('window-blur', () => {
          if (!showSettings) {
            void dismiss();
          }
        }),
      );

      unlisteners.push(
        await listen('show-settings', () => {
          showSettings = true;
          popupScale.target = 1;
          popupOpacity.target = 1;
        }),
      );

      unlisteners.push(
        await listen<HotkeyConflictPayload>('hotkey-conflict', (event) => {
          const notification = createHotkeyConflictNotification(event.payload);
          hotkeyConflictMessage = notification.message;
          pushNotification(notification);
          console.error('Hotkey conflict:', event.payload);
        }),
      );

      unlisteners.push(
        await listen<string>('hotkey-registered', () => {
          clearSettingsWarnings();
        }),
      );

      unlisteners.push(
        await listen<DaemonErrorPayload>('daemon-error', (event) => {
          pushNotification(createDaemonErrorNotification(event.payload));
          console.error('Daemon error:', event.payload);
        }),
      );
    };

    void register();
    void invoke('mark_ui_ready').catch((e) => {
      console.error('Failed to mark UI as ready:', e);
    });

    return () => {
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="h-screen w-screen p-2">
  <div
    class="relative h-full w-full"
    style:transform="scale({popupScale.current})"
    style:opacity={popupOpacity.current}
    style="transform-origin: bottom right; will-change: transform, opacity;"
  >
    <NotificationCenter {notifications} ondismiss={dismissNotification} />

    <TranslationPopup
      viewState={appState}
      {sourceText}
      {translatedText}
      {errorMessage}
      hotkeyLabel={config.hotkey || 'CmdOrCtrl+T'}
      {sourceLang}
      {targetLang}
      onLanguageChange={handleLanguageChange}
      oncancel={handleCancel}
    />

    <SettingsPanel
      visible={showSettings}
      onclose={() => (showSettings = false)}
      {hotkeyConflictMessage}
    />
  </div>
</div>
