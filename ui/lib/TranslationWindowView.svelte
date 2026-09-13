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
  import { prefersReducedMotion } from './motion';
  import {
    RESIZE_HANDLES,
    shouldAutoFit,
    shouldDismissOnBlur,
    shouldDismissOnEscape,
    shouldRecordUserResize,
    type ResizeDirection,
  } from './windowBehavior';
  import { persistCurrentWindowPlacement } from './windowPlacement';

  let appState: 'idle' | 'loading' | 'streaming' | 'result' | 'error' = $state('idle');
  let translatedText = $state('');
  let errorMessage = $state('');
  let notifications = $state<AppNotification[]>([]);
  let currentRequestId = $state(0);
  let loadingTimeoutId: ReturnType<typeof setTimeout> | null = null;
  let placementSaveTimeoutId: ReturnType<typeof setTimeout> | null = null;
  let autoSizeTimeoutId: ReturnType<typeof setTimeout> | null = null;
  let visible = $state(false);
  let config: AppConfig = $state(createDefaultAppConfig());
  let popupElement = $state<HTMLDivElement | null>(null);
  /** A height the user chose by dragging; non-null disables auto-fitting. */
  let userResizedHeight = $state<number | null>(null);
  let suppressNextResizeObservation = false;

  const TRANSLATION_WINDOW_WIDTH = 392;
  const TRANSLATION_MIN_HEIGHT = 156;
  const TRANSLATION_MAX_HEIGHT = 492;
  const WINDOW_SAFE_AREA_HEIGHT = 2;

  const popupScale = new Spring(0.94, { stiffness: 0.16, damping: 0.72 });
  const popupOpacity = new Spring(0, { stiffness: 0.18, damping: 0.82 });

  /**
   * Applies a spring target, or jumps straight to it when the user has asked macOS to reduce
   * motion. The window still appears and disappears; it just stops animating.
   */
  function setSpringTarget(spring: Spring<number>, value: number) {
    if (prefersReducedMotion()) {
      void spring.set(value, { instant: true });
      return;
    }
    spring.target = value;
  }

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

  type HistoryReplayPayload = {
    text: string;
    config: AppConfig;
  };

  let lastRequestConfig = $state<AppConfig | null>(null);
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
          message: '无法从桌面后端加载本地 Aura 配置。',
          recoverable: true,
        }),
      );
      return null;
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

  function showTranslationFailure(rawMessage: unknown) {
    const message = formatTranslationError(rawMessage);
    clearLoadingTimeout();
    retryAttempt = null;
    translationUsage = null;
    appState = 'error';
    errorMessage = message;
    pushNotification(createTranslationErrorNotification(message));
  }

  /**
   * Releases a backend request reservation for a dispatch this window decided not to perform.
   *
   * A successful emit only proves the event reached the webview; the request is not real until
   * `translate_text` runs. A failure before that point must hand the reservation back, or the next
   * press for this text collapses instead of translating.
   */
  async function releaseReservation(text: string) {
    try {
      await invoke('release_request_reservation', { text });
    } catch (e) {
      console.error('Failed to release the request reservation:', e);
    }
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
      showTranslationFailure('还没有可重试的翻译请求。');
      return;
    }

    if (requestConfig.provider !== 'ollama' && !requestConfig.api_key) {
      showTranslationFailure('No API key configured. Right-click the tray icon -> Settings.');
      return;
    }

    lastRequestConfig = cloneAppConfig(requestConfig);
    currentRequestId += 1;
    const requestId = currentRequestId;
    translatedTextTriggerText = requestText;
    draftSourceText = requestText;
    // Cleared only once there is text for the composer to derive from, so an early failure cannot
    // leave the user without an input box.
    emptyEntry = false;
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
        profileId: requestConfig.active_profile_id,
      });
    } catch (e) {
      if (isCurrentRequest(requestId) && errorMessage === '') {
        showTranslationFailure(e);
      }
    }
  }

  let translatedTextTriggerText = $state('');

  /**
   * Set when the window is opened as an empty entry point because the clipboard had nothing.
   *
   * Deriving "can the user type?" from the presence of text is circular for this state: the whole
   * point is that there is no text yet, so the composer has to be requested explicitly.
   */
  let emptyEntry = $state(false);

  /**
   * Direct source editing is not tied to pinning. The composer shows when there is text to work
   * with or when the user explicitly opened an empty entry point; pinning only controls whether
   * the window survives losing focus.
   */
  const hasSourceText = $derived(
    emptyEntry || draftSourceText.trim().length > 0 || translatedTextTriggerText.trim().length > 0,
  );

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
          message: '无法更新窗口行为，请在设置中重试。',
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
          message: '无法将译文复制到剪贴板。',
          recoverable: true,
        }),
      );
    }
  }

  function showBubble() {
    visible = true;
    setSpringTarget(popupScale, 1);
    setSpringTarget(popupOpacity, 1);
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

    setSpringTarget(popupScale, 0.94);
    setSpringTarget(popupOpacity, 0);

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

  /**
   * Escape is a single, unconditional collapse. A pinned window is un-pinned first so that the
   * documented "one Escape closes it" contract also holds while pinned, instead of leaving a
   * window that immediately re-opens on the next focus change.
   */
  async function handleEscape() {
    if (config.window_pinned) {
      await handlePinnedChange(false);
    }
    await dismiss();
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.key !== 'Escape') return;
    // Never swallow Escape while an input method editor is composing, or typing Chinese would
    // close the window mid-word.
    if (!shouldDismissOnEscape(event.isComposing)) return;

    event.preventDefault();
    void handleEscape();
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
    if (!visible || !popupElement) return;
    if (!shouldAutoFit(config.window_pinned, userResizedHeight)) return;

    await tick();

    const desiredHeight = Math.min(
      Math.max(Math.ceil(popupElement.scrollHeight) + WINDOW_SAFE_AREA_HEIGHT, TRANSLATION_MIN_HEIGHT),
      TRANSLATION_MAX_HEIGHT,
    );

    try {
      suppressNextResizeObservation = true;
      await getCurrentWindow().setSize(new LogicalSize(TRANSLATION_WINDOW_WIDTH, desiredHeight));
      await invoke('realign_translation_window');
    } catch (e) {
      console.error('Failed to auto-size translation window:', e);
    } finally {
      suppressNextResizeObservation = false;
    }
  }

  /**
   * Records a user-initiated height so auto-fitting stops fighting them.
   *
   * `setSize` also fires `onResized`, so programmatic sizing is flagged and skipped; otherwise the
   * first auto-fit would immediately look like a manual resize and freeze the window height.
   */
  async function noteUserResize() {
    if (!shouldRecordUserResize(suppressNextResizeObservation, config.window_pinned)) return;

    try {
      const size = await getCurrentWindow().innerSize();
      const scale = await getCurrentWindow().scaleFactor();
      userResizedHeight = size.height / scale;
    } catch (e) {
      console.error('Failed to read the resized window height:', e);
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

    const appWindow = getCurrentWindow();
    const unlisteners: Array<() => void> = [];

    const register = async () => {
      unlisteners.push(
        await listen<string>('trigger-translate', async (event) => {
          const text = event.payload?.trim();
          if (!text) return;
          await cancelCurrentTranslation();
          currentRequestId += 1;

          // New text: auto-fit may size the window for this content again.
          userResizedHeight = null;
          translatedTextTriggerText = text;
          draftSourceText = text;
          showBubble();
          const loadedConfig = await loadConfig();
          if (!loadedConfig) {
            // The backend reserved this request before emitting it. Abandoning the dispatch here
            // must release that reservation.
            await releaseReservation(text);
            return;
          }
          lastRequestConfig = cloneAppConfig(loadedConfig);
          await startTranslation(loadedConfig);
        }),
      );

      unlisteners.push(
        await listen<HistoryReplayPayload>('trigger-translate-with-override', async (event) => {
          const text = event.payload.text?.trim();
          if (!text) return;

          await cancelCurrentTranslation();
          currentRequestId += 1;
          userResizedHeight = null;
          translatedTextTriggerText = text;
          draftSourceText = text;
          showBubble();
          const overrideConfig = event.payload.config
            ? cloneAppConfig(event.payload.config)
            : null;
          if (!overrideConfig) {
            await releaseReservation(text);
            return;
          }
          await startTranslation(overrideConfig, text);
        }),
      );

      unlisteners.push(
        await listen('show-existing-translation', () => {
          showBubble();
          scheduleAutoSize();
        }),
      );

      unlisteners.push(
        await listen('show-empty-translation', () => {
          // Empty clipboard: open a directly editable empty state instead of an error. This is a
          // new request boundary, so any previous result is cleared rather than left stale.
          void cancelCurrentTranslation();
          currentRequestId += 1;
          translatedTextTriggerText = '';
          draftSourceText = '';
          translatedText = '';
          translationUsage = null;
          retryAttempt = null;
          errorMessage = '';
          lastRequestConfig = null;
          appState = 'idle';
          // No text exists yet, so the composer must be requested rather than derived.
          emptyEntry = true;
          showBubble();
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
            return;
          }
          void noteUserResize();
        }),
      );
    };

    // Registration must complete before the backend is told the UI is ready. Reporting readiness
    // while listeners were still being attached let a trigger be emitted into a window with no
    // listener yet, which also left that request's reservation pending.
    void register()
      .catch((e) => {
        console.error('Failed to register translation window listeners:', e);
      })
      .finally(() => {
        void invoke('mark_ui_ready').catch((e) => {
          console.error('Failed to mark UI as ready:', e);
        });
      });

    return () => {
      if (placementSaveTimeoutId !== null) clearTimeout(placementSaveTimeoutId);
      if (autoSizeTimeoutId !== null) clearTimeout(autoSizeTimeoutId);
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  });
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div class="relative h-screen w-screen overflow-hidden">
  {#if config.window_pinned}
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
  {/if}

  <div
    class="aura-window-shell"
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
        showComposer={hasSourceText}
        hasDraftChanges={draftSourceText.trim() !== translatedTextTriggerText.trim()}
        usage={translationUsage}
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
      />
    </div>
  </div>
</div>
