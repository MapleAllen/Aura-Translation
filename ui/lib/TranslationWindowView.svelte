<script lang="ts">
  import { Spring } from 'svelte/motion';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
  import { onMount, tick } from 'svelte';
  import type { AppConfig } from './appConfig';
  import { cloneAppConfig, createDefaultAppConfig } from './appConfig';
  import { LANGUAGES } from './languages';
  import NotificationCenter from './NotificationCenter.svelte';
  import TranslationPopup from './TranslationPopup.svelte';
  import type { TranslationUsage } from './translationHistory';
  import {
    createAuraGuardNotification,
    createDaemonErrorNotification,
    createTranslationErrorNotification,
    formatTranslationError,
    type AppNotification,
    type AuraGuardBlockedPayload,
    type DaemonErrorPayload,
  } from './notifications';
  import { RESIZE_HANDLES, shouldDismissOnBlur, type ResizeDirection } from './windowBehavior';
  import { persistCurrentWindowPlacement } from './windowPlacement';

  let appState: 'idle' | 'loading' | 'streaming' | 'result' | 'error' = $state('idle');
  let translatedText = $state('');
  let errorMessage = $state('');
  let notifications = $state<AppNotification[]>([]);
  let currentRequestId = $state(0);
  let loadingTimeoutId: ReturnType<typeof setTimeout> | null = null;
  let placementSaveTimeoutId: ReturnType<typeof setTimeout> | null = null;
  let autoSizeTimeoutId: ReturnType<typeof setTimeout> | null = null;
  let emptyHintTimerId: ReturnType<typeof setTimeout> | null = null;
  let visible = $state(false);
  let config: AppConfig = $state(createDefaultAppConfig());
  let popupElement = $state<HTMLDivElement | null>(null);

  const popupScale = new Spring(0.94, { stiffness: 0.16, damping: 0.72 });
  const popupOpacity = new Spring(0, { stiffness: 0.18, damping: 0.82 });

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

  type TranslationUsagePayload = {
    request_id: number;
    usage: TranslationUsage;
  };

  type PasteBackStatus = {
    supported: boolean;
    available: boolean;
  };

  let lastRequestConfig = $state<AppConfig | null>(null);
  let pasteBackStatus = $state<PasteBackStatus>({ supported: false, available: false });
  let retryAttempt = $state<number | null>(null);
  let translationUsage = $state<TranslationUsage | null>(null);
  let draftSourceText = $state('');

  function pushNotification(notification: AppNotification) {
    notifications = [notification, ...notifications.filter((item) => item.scope !== notification.scope)]
      .slice(0, 3);
  }

  function dismissNotification(id: string) {
    notifications = notifications.filter((item) => item.id !== id);
  }

  function languageLabel(code: string) {
    return LANGUAGES.find((language) => language.code === code)?.label ?? code;
  }

  function providerLabel(provider: AppConfig['provider']) {
    switch (provider) {
      case 'deepseek':
        return 'DeepSeek';
      case 'openrouter':
        return 'OpenRouter';
      case 'ollama':
        return 'Ollama';
    }
  }

  async function loadConfig() {
    try {
      const loaded = await invoke<AppConfig>('get_config');
      config = cloneAppConfig(loaded);
      return config;
    } catch (e) {
      console.error('Failed to load config:', e);
      pushNotification(
        createDaemonErrorNotification({
          code: 'config-load-failed',
          message: 'Failed to load the local Aura config from the desktop backend.',
          recoverable: true,
        }),
      );
      return null;
    }
  }

  async function refreshPasteBackStatus() {
    try {
      pasteBackStatus = await invoke<PasteBackStatus>('get_paste_back_status');
    } catch (e) {
      console.error('Failed to load paste-back status:', e);
      pasteBackStatus = { supported: false, available: false };
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

  function clearEmptyHintTimer() {
    if (emptyHintTimerId !== null) {
      clearTimeout(emptyHintTimerId);
      emptyHintTimerId = null;
    }
  }

  function showEmptyClipboardHint() {
    if (visible) {
      pushNotification(
        createDaemonErrorNotification({
          code: 'hotkey-empty-clipboard',
          message: 'Copy text to translate first, then press the hotkey.',
          recoverable: true,
        }),
      );
      return;
    }

    clearEmptyHintTimer();
    appState = 'idle';
    errorMessage = '';
    translatedText = '';
    translatedTextTriggerText = '';
    draftSourceText = '';
    showBubble();

    emptyHintTimerId = setTimeout(() => {
      emptyHintTimerId = null;
      void dismiss();
    }, 3000);
  }

  function isCurrentRequest(requestId: number) {
    return requestId === currentRequestId;
  }

  function showTranslationFailure(rawMessage: unknown) {
    const message = formatTranslationError(rawMessage);
    clearLoadingTimeout();
    retryAttempt = null;
    translationUsage = null;
    appState = 'error';
    errorMessage = message;
    pushNotification(createTranslationErrorNotification(message));
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

  async function startTranslation(
    requestConfig: AppConfig | null = lastRequestConfig ?? config,
    textOverride?: string,
  ) {
    const requestText = (textOverride ?? draftSourceText ?? translatedTextTriggerText).trim();

    if (!requestConfig || !requestText) {
      showTranslationFailure('No translation request is available to retry yet.');
      return;
    }

    if (requestConfig.provider !== 'ollama' && !requestConfig.api_key) {
      showTranslationFailure('No API key configured. Right-click the tray icon -> Settings.');
      return;
    }

    clearEmptyHintTimer();
    lastRequestConfig = cloneAppConfig(requestConfig);
    currentRequestId += 1;
    const requestId = currentRequestId;
    translatedTextTriggerText = requestText;
    draftSourceText = requestText;
    translatedText = '';
    translationUsage = null;
    errorMessage = '';
    retryAttempt = null;
    appState = 'loading';

    startLoadingTimeout();
    scheduleAutoSize();

    try {
      await invoke('translate_text', {
        text: requestText,
        sourceLang: requestConfig.source_lang,
        targetLang: requestConfig.target_lang,
        apiKey: requestConfig.api_key,
        model: requestConfig.model,
        requestId,
        apiBaseUrl: requestConfig.api_base_url,
        provider: requestConfig.provider,
      });
    } catch (e) {
      if (isCurrentRequest(requestId) && errorMessage === '') {
        showTranslationFailure(e);
      }
    }
  }

  let translatedTextTriggerText = $state('');

  async function handleCancel() {
    await cancelCurrentTranslation();
    currentRequestId += 1;
    retryAttempt = null;
    translationUsage = null;
    appState = translatedText ? 'result' : 'idle';
    scheduleAutoSize();
  }

  async function retryTranslation() {
    if (!draftSourceText.trim()) return;

    await cancelCurrentTranslation();
    currentRequestId += 1;
    showBubble();
    await startTranslation(cloneAppConfig(lastRequestConfig ?? config), draftSourceText);
  }

  function resetDraftSource() {
    draftSourceText = translatedTextTriggerText;
  }

  async function handlePinnedChange(nextPinned: boolean) {
    if (config.window_pinned === nextPinned) return;

    const previousPinned = config.window_pinned;
    config.window_pinned = nextPinned;

    try {
      await invoke('save_config', { config: cloneAppConfig(config) });
      if (nextPinned) {
        await persistCurrentWindowPlacement('pinned_translation');
      }
    } catch (e) {
      config.window_pinned = previousPinned;
      console.error('Failed to save window pin state:', e);
      pushNotification(
        createDaemonErrorNotification({
          code: 'window-pin-save-failed',
          message: 'Failed to update window behavior. Try again from Settings.',
          recoverable: true,
        }),
      );
    }
  }

  async function copyResult() {
    if (!translatedText) return;
    try {
      await invoke('copy_result_to_clipboard', { text: translatedText });
    } catch (e) {
      console.error('Failed to copy:', e);
      pushNotification(
        createDaemonErrorNotification({
          code: 'clipboard-copy-failed',
          message: 'Failed to copy the translated text to the clipboard.',
          recoverable: true,
        }),
      );
    }
  }

  async function pasteBackResult() {
    if (!translatedText) return;

    try {
      await invoke('paste_translation_back', { text: translatedText });
    } catch (e) {
      console.error('Failed to paste back:', e);
      await refreshPasteBackStatus();
      pushNotification(
        createDaemonErrorNotification({
          code: 'paste-back-failed',
          message: String(e ?? 'Failed to paste the translated text back into the source app.'),
          recoverable: true,
        }),
      );
    }
  }

  function showBubble() {
    visible = true;
    popupScale.target = 1;
    popupOpacity.target = 1;
  }

  async function dismiss() {
    const appWindow = getCurrentWindow();
    void cancelCurrentTranslation();
    currentRequestId += 1;

    if (appState === 'streaming') {
      appState = translatedText ? 'result' : 'idle';
    } else if (appState === 'loading') {
      appState = translatedText ? 'result' : 'idle';
    }

    popupScale.target = 0.94;
    popupOpacity.target = 0;

    clearLoadingTimeout();

    setTimeout(async () => {
      visible = false;
      try {
        await appWindow.hide();
      } catch (e) {
        console.error('Failed to hide window:', e);
      }
    }, 160);
  }

  async function startResize(direction: ResizeDirection) {
    try {
      await getCurrentWindow().startResizeDragging(direction);
    } catch (e) {
      console.error('Failed to start resize drag:', e);
    }
  }

  function schedulePinnedPlacementSave() {
    if (!config.window_pinned) return;
    if (placementSaveTimeoutId !== null) {
      clearTimeout(placementSaveTimeoutId);
    }
    placementSaveTimeoutId = setTimeout(() => {
      void persistCurrentWindowPlacement('pinned_translation').catch((e) => {
        console.error('Failed to persist translation placement:', e);
      });
    }, 180);
  }

  async function applyAutoSize() {
    if (!visible || config.window_pinned || !popupElement) return;

    await tick();

    const desiredHeight = Math.min(
      Math.max(Math.ceil(popupElement.scrollHeight) + 2, 124),
      460,
    );

    try {
      await getCurrentWindow().setSize(new LogicalSize(360, desiredHeight));
      await invoke('realign_translation_window');
    } catch (e) {
      console.error('Failed to auto-size translation window:', e);
    }
  }

  function scheduleAutoSize() {
    if (autoSizeTimeoutId !== null) {
      clearTimeout(autoSizeTimeoutId);
    }
    autoSizeTimeoutId = setTimeout(() => {
      void applyAutoSize();
    }, 50);
  }

  $effect(() => {
    visible;
    appState;
    translatedText;
    errorMessage;
    config.window_pinned;
    scheduleAutoSize();
  });

  onMount(() => {
    void loadConfig();
    void refreshPasteBackStatus();

    const appWindow = getCurrentWindow();
    const unlisteners: Array<() => void> = [];

    const register = async () => {
      unlisteners.push(
        await listen<string>('trigger-translate', async (event) => {
          const text = event.payload?.trim();
          if (!text) return;

          clearEmptyHintTimer();
          await cancelCurrentTranslation();
          currentRequestId += 1;

          translatedTextTriggerText = text;
          draftSourceText = text;
          showBubble();
          await refreshPasteBackStatus();
          const loadedConfig = await loadConfig();
          if (!loadedConfig) return;
          lastRequestConfig = cloneAppConfig(loadedConfig);
          await startTranslation(loadedConfig);
        }),
      );

      unlisteners.push(
        await listen('show-existing-translation', () => {
          showBubble();
          void refreshPasteBackStatus();
          scheduleAutoSize();
        }),
      );

      unlisteners.push(
        await listen<TranslationChunkPayload>('translation-chunk', (event) => {
          if (!isCurrentRequest(event.payload.request_id)) return;

          if (appState === 'loading') {
            appState = 'streaming';
            clearLoadingTimeout();
          }
          retryAttempt = null;
          if (appState === 'streaming') {
            translatedText += event.payload.content;
            scheduleAutoSize();
          }
        }),
      );

      unlisteners.push(
        await listen<TranslationDonePayload>('translation-done', (event) => {
          if (!isCurrentRequest(event.payload.request_id)) return;

          clearLoadingTimeout();
          retryAttempt = null;
          if (appState === 'loading' || appState === 'streaming') {
            appState = translatedText ? 'result' : 'idle';
            scheduleAutoSize();
          }
        }),
      );

      unlisteners.push(
        await listen<TranslationErrorPayload>('translation-error', (event) => {
          if (!isCurrentRequest(event.payload.request_id)) return;
          showTranslationFailure(event.payload.message);
          scheduleAutoSize();
        }),
      );

      unlisteners.push(
        await listen<TranslationRetryPayload>('translation-retry', (event) => {
          if (!isCurrentRequest(event.payload.request_id)) return;
          retryAttempt = event.payload.attempt;
          scheduleAutoSize();
        }),
      );

      unlisteners.push(
        await listen<TranslationUsagePayload>('translation-usage', (event) => {
          if (!isCurrentRequest(event.payload.request_id)) return;
          translationUsage = event.payload.usage;
        }),
      );

      unlisteners.push(
        await listen('window-blur', () => {
          if (shouldDismissOnBlur(false, config.window_pinned)) {
            void dismiss();
          }
        }),
      );

      unlisteners.push(
        await listen<DaemonErrorPayload>('daemon-error', (event) => {
          if (event.payload.code === 'hotkey-empty-clipboard') {
            showEmptyClipboardHint();
            return;
          }
          pushNotification(createDaemonErrorNotification(event.payload));
          console.error('Daemon error:', event.payload);
        }),
      );

      unlisteners.push(
        await listen<AuraGuardBlockedPayload>('aura-guard-blocked', (event) => {
          pushNotification(createAuraGuardNotification(event.payload.reason));
        }),
      );

      unlisteners.push(
        await listen<AppConfig>('config-updated', (event) => {
          config = cloneAppConfig(event.payload);
        }),
      );

      unlisteners.push(
        await appWindow.onMoved(() => {
          schedulePinnedPlacementSave();
        }),
      );

      unlisteners.push(
        await appWindow.onResized(() => {
          if (config.window_pinned) {
            schedulePinnedPlacementSave();
          }
        }),
      );
    };

    void register();
    void invoke('mark_ui_ready').catch((e) => {
      console.error('Failed to mark UI as ready:', e);
    });

    return () => {
      if (placementSaveTimeoutId !== null) clearTimeout(placementSaveTimeoutId);
      if (autoSizeTimeoutId !== null) clearTimeout(autoSizeTimeoutId);
      if (emptyHintTimerId !== null) clearTimeout(emptyHintTimerId);
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  });
</script>

<div class="relative h-screen w-screen overflow-hidden">
  <div class="pointer-events-none absolute inset-0 z-[80]">
    {#each RESIZE_HANDLES as handle}
      <button
        class={`pointer-events-auto absolute ${handle.className}`}
        style={`cursor: ${handle.cursor};`}
        onmousedown={(event) => {
          event.preventDefault();
          void startResize(handle.direction);
        }}
        aria-hidden="true"
        tabindex="-1"
        type="button"
      ></button>
    {/each}
  </div>

  <div
    class="relative h-full w-full"
    style:transform="scale({popupScale.current})"
    style:opacity={popupOpacity.current}
    style="transform-origin: center bottom; will-change: transform, opacity;"
  >
    <NotificationCenter {notifications} ondismiss={dismissNotification} />

    <div bind:this={popupElement} class="h-full w-full">
      <TranslationPopup
        viewState={appState}
        sourceText={translatedTextTriggerText}
        draftSourceText={draftSourceText}
        sourceLangLabel={lastRequestConfig ? languageLabel(lastRequestConfig.source_lang) : ''}
        targetLangLabel={lastRequestConfig ? languageLabel(lastRequestConfig.target_lang) : ''}
        providerLabel={lastRequestConfig ? providerLabel(lastRequestConfig.provider) : ''}
        modelLabel={lastRequestConfig?.model ?? ''}
        {retryAttempt}
        {translatedText}
        {errorMessage}
        showComposer={config.window_pinned}
        hasDraftChanges={draftSourceText.trim() !== translatedTextTriggerText.trim()}
        usage={translationUsage}
        canPasteBack={pasteBackStatus.supported && pasteBackStatus.available}
        pasteBackSupported={pasteBackStatus.supported}
        pasteBackAvailable={pasteBackStatus.available}
        windowPinned={config.window_pinned}
        ondraftsourcechange={(value) => {
          draftSourceText = value;
        }}
        ontranslatedraft={retryTranslation}
        onresetdraft={resetDraftSource}
        onTogglePinned={handlePinnedChange}
        onretry={retryTranslation}
        oncancel={handleCancel}
        ondismiss={dismiss}
        oncopy={copyResult}
        onpasteback={pasteBackResult}
      />
    </div>
  </div>
</div>
