<script lang="ts">
  /**
   * TranslationPopup - Minimal floating translation bubble.
   */
  import type { TranslationUsage } from './translationHistory';

  type Props = {
    viewState: 'idle' | 'loading' | 'streaming' | 'result' | 'error';
    sourceText: string;
    draftSourceText: string;
    sourceLangLabel: string;
    targetLangLabel: string;
    providerLabel: string;
    modelLabel: string;
    retryAttempt: number | null;
    translatedText: string;
    errorMessage: string;
    showComposer: boolean;
    hasDraftChanges: boolean;
    usage: TranslationUsage | null;
    canPasteBack?: boolean;
    pasteBackSupported?: boolean;
    pasteBackAvailable?: boolean;
    windowPinned: boolean;
    ondraftsourcechange?: (value: string) => void;
    ontranslatedraft?: () => void;
    onresetdraft?: () => void;
    onTogglePinned?: (windowPinned: boolean) => void;
    onretry?: () => void;
    oncancel?: () => void;
    ondismiss?: () => void;
    oncopy?: () => void;
    onpasteback?: () => void;
  };

  let {
    viewState,
    sourceText,
    draftSourceText,
    sourceLangLabel,
    targetLangLabel,
    providerLabel,
    modelLabel,
    retryAttempt,
    translatedText,
    errorMessage,
    showComposer,
    hasDraftChanges,
    usage,
    canPasteBack,
    pasteBackSupported,
    pasteBackAvailable,
    windowPinned,
    ondraftsourcechange,
    ontranslatedraft,
    onresetdraft,
    onTogglePinned,
    onretry,
    oncancel,
    ondismiss,
    oncopy,
    onpasteback,
  }: Props = $props();

  const isBusy = $derived(viewState === 'loading' || viewState === 'streaming');

  const canPasteBackDerived = $derived(
    canPasteBack ?? (Boolean(pasteBackSupported) && Boolean(pasteBackAvailable)),
  );

  const pasteBackTooltip = $derived.by(() => {
    if (!translatedText || isBusy) {
      return '请先完成翻译再回填。';
    }
    if (!pasteBackSupported) {
      return '回填功能目前仅支持 Windows。';
    }
    if (!pasteBackAvailable) {
      return '请先从前台应用复制文本，Aura 才能回填。';
    }
    return '将译文回填到原应用。';
  });
</script>

<div
  class="aura-glass-panel flex flex-col"
>
  <div
    class="relative z-[2] flex shrink-0 items-center gap-3 px-4 pb-3 pt-4"
  >
    <div
      class="shrink-0 rounded-full border border-aura-border/70 bg-white/78 px-3 py-1.5 text-xs font-display font-medium text-aura-text-dim"
      data-tauri-drag-region
    >
      {#if viewState === 'loading'}
        连接中
      {:else if retryAttempt !== null}
        重试 {retryAttempt}
      {:else if viewState === 'streaming'}
        翻译中
      {:else if viewState === 'error'}
        需要处理
      {:else if viewState === 'result'}
        已就绪
      {:else}
        Aura
      {/if}
    </div>

    <div
      class="h-8 min-w-[88px] flex-1 cursor-move rounded-full"
      data-tauri-drag-region
      data-testid="window-drag-handle"
      aria-label="拖动窗口"
      title="拖动窗口"
    ></div>

    <div class="flex shrink-0 items-center gap-2">
      <button
        class={`flex h-8 items-center gap-1.5 rounded-full border px-3 text-[11px] font-display transition-colors duration-150 ${
          windowPinned
            ? 'border-aura-accent bg-aura-accent text-white'
            : 'border-aura-border bg-white/84 text-aura-text-dim hover:border-aura-border-accent hover:text-aura-text'
        }`}
        type="button"
        aria-pressed={windowPinned}
        onclick={() => onTogglePinned?.(!windowPinned)}
      >
        <span class="h-2.5 w-2.5 rounded-full bg-current"></span>
        <span>{windowPinned ? '已固定' : '固定'}</span>
      </button>

      {#if sourceText && !isBusy && !showComposer}
        <button
          class="flex h-8 w-8 items-center justify-center rounded-full border border-aura-border bg-white/84 text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-accent"
          onclick={() => onretry?.()}
          aria-label="重新翻译"
          type="button"
        >
          <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.9">
            <path stroke-linecap="round" stroke-linejoin="round" d="M16.023 9.348h4.992V4.356m-.937 4.992A9 9 0 1 0 6.75 18.75" />
          </svg>
        </button>
      {/if}

      {#if translatedText && !isBusy}
        <button
          class="flex h-8 w-8 items-center justify-center rounded-full border border-aura-border bg-white/84 text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-accent disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:border-aura-border disabled:hover:text-aura-text-dim"
          onclick={() => onpasteback?.()}
          aria-label="回填译文"
          title={pasteBackTooltip}
          type="button"
          disabled={!canPasteBackDerived}
        >
          <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.9">
            <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 8.25H5.625A2.625 2.625 0 0 0 3 10.875v7.5A2.625 2.625 0 0 0 5.625 21h7.5a2.625 2.625 0 0 0 2.625-2.625V15.75m-7.5-7.5L12 4.5m0 0 3.75 3.75M12 4.5v10.5" />
          </svg>
        </button>
      {/if}

      {#if translatedText && !isBusy}
        <button
          class="flex h-8 w-8 items-center justify-center rounded-full border border-aura-border bg-white/84 text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-accent"
          onclick={() => oncopy?.()}
          aria-label="复制译文"
          type="button"
        >
          <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8">
            <path stroke-linecap="round" stroke-linejoin="round" d="M15.666 3.888A2.25 2.25 0 0 0 13.5 2.25h-3c-1.03 0-1.9.693-2.166 1.638m7.332 0c.055.194.084.4.084.612v0a.75.75 0 0 1-.75.75H9.75a.75.75 0 0 1-.75-.75v0c0-.212.03-.418.084-.612m7.332 0c.646.049 1.288.11 1.927.184 1.1.128 1.907 1.077 1.907 2.185V19.5a2.25 2.25 0 0 1-2.25 2.25H6.75A2.25 2.25 0 0 1 4.5 19.5V6.257c0-1.108.806-2.057 1.907-2.185a48.208 48.208 0 0 1 1.927-.184" />
          </svg>
        </button>
      {/if}

      {#if isBusy}
        <button
          class="flex h-8 w-8 items-center justify-center rounded-full border border-aura-border bg-white/84 text-aura-text-dim transition-colors duration-150 hover:border-aura-error/30 hover:text-aura-error"
          onclick={() => oncancel?.()}
          aria-label="取消翻译"
          type="button"
        >
          <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.1">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      {/if}

      <button
        class="flex h-8 w-8 items-center justify-center rounded-full border border-aura-border bg-white/84 text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-text"
        onclick={() => ondismiss?.()}
        aria-label="关闭翻译窗"
        type="button"
      >
        <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.1">
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  </div>

  <div class="flex min-h-0 flex-1 items-stretch px-5 pb-5 pt-1">
    <div class="flex w-full min-h-0 flex-col">
      {#if sourceText || showComposer}
        <div class="mb-4 space-y-2">
          <div class="flex flex-wrap gap-2">
            <span class="rounded-full border border-aura-border/80 bg-white/84 px-3 py-1.5 text-[11px] font-display font-medium text-aura-text-dim">
              {sourceLangLabel} → {targetLangLabel}
            </span>
            <span class="rounded-full border border-aura-border/80 bg-white/84 px-3 py-1.5 text-[11px] font-display font-medium text-aura-text-dim">
              {providerLabel} · {modelLabel}
            </span>
            {#if retryAttempt !== null}
              <span class="rounded-full border border-aura-accent/25 bg-aura-accent-soft px-3 py-1.5 text-[11px] font-display font-medium text-aura-accent">
                第 {retryAttempt}/3 次重试
              </span>
            {/if}
            {#if usage}
              <span class="rounded-full border border-aura-accent/25 bg-aura-accent-soft px-3 py-1.5 text-[11px] font-display font-medium text-aura-accent">
                {usage.total_tokens.toLocaleString()} token
              </span>
            {/if}
          </div>

          {#if showComposer}
            <div class="rounded-[16px] border border-aura-border bg-white/68 px-5 py-4">
              <div class="flex flex-wrap items-center justify-between gap-3">
                <div>
                  <p class="aura-section-title">
                    原文草稿
                  </p>
                  <p class="mt-1 text-xs leading-relaxed text-aura-text-muted">
                    在这里修改原文，按 Ctrl+Enter 可直接重新翻译全文。
                  </p>
                </div>

                <div class="flex flex-wrap items-center gap-2">
                  {#if hasDraftChanges}
                    <button
                      class="rounded-full border border-aura-border bg-white px-3 py-1.5 text-[11px] font-medium text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-text"
                      type="button"
                      onclick={() => onresetdraft?.()}
                    >
                      还原
                    </button>
                  {/if}

                  <button
                    class="rounded-full border border-aura-accent/30 bg-aura-accent-soft px-3 py-1.5 text-[11px] font-medium text-aura-accent transition-colors duration-150 hover:border-aura-accent hover:bg-aura-accent hover:text-white disabled:cursor-not-allowed disabled:opacity-50"
                    type="button"
                    onclick={() => ontranslatedraft?.()}
                    disabled={!draftSourceText.trim() || isBusy}
                  >
                    重新翻译全文
                  </button>
                </div>
              </div>

              <textarea
                value={draftSourceText}
                oninput={(event) => ondraftsourcechange?.((event.currentTarget as HTMLTextAreaElement).value)}
                onkeydown={(event) => {
                  if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
                    event.preventDefault();
                    ontranslatedraft?.();
                  }
                }}
                placeholder="在这里输入或修改原文"
                class="mt-3 min-h-[112px] w-full resize-none rounded-[14px] border border-aura-border bg-white px-4 py-3 text-sm leading-7 text-aura-text outline-none transition-colors duration-150 placeholder:text-aura-text-muted focus:border-aura-accent focus:ring-2 focus:ring-aura-accent/15"
              ></textarea>
            </div>
          {:else}
            <div class="rounded-[16px] border border-aura-border bg-white/68 px-5 py-4">
              <p class="aura-section-title">
                原文
              </p>
              <p
                class="mt-2 max-h-[76px] overflow-y-auto pr-1 text-sm leading-7 text-aura-text-dim"
              >
                {sourceText}
              </p>
            </div>
          {/if}
        </div>
      {/if}

      {#if viewState === 'idle'}
        <div class="flex min-h-[88px] w-full items-center justify-center rounded-[16px] border border-dashed border-aura-border bg-white/62 px-6 text-center">
          <p class="max-w-[280px] text-sm leading-7 text-aura-text-dim">
            {showComposer
              ? '在草稿区输入或粘贴原文，然后按 Ctrl+Enter 重新翻译全文。'
              : '复制要翻译的文本，按快捷键即可唤起 Aura。'}
          </p>
        </div>
      {:else if viewState === 'loading'}
        <div class="flex min-h-[88px] w-full flex-1 items-center justify-center">
          <div class="flex items-center gap-3 rounded-full border border-aura-border bg-white/78 px-4 py-2.5 text-sm text-aura-text-dim">
            <span class="h-2.5 w-2.5 animate-pulse rounded-full bg-aura-accent"></span>
            <span>{retryAttempt !== null ? `正在重试请求（${retryAttempt}/3）...` : '正在翻译...'}</span>
          </div>
        </div>
      {:else if viewState === 'streaming' || viewState === 'result'}
        <div class="w-full min-h-0 flex-1 overflow-y-auto pb-2 pr-2">
          <p class="select-text whitespace-pre-wrap break-words text-[15px] leading-8 text-aura-text">
            {translatedText}{#if viewState === 'streaming'}<span class="ml-0.5 inline-block h-5 w-0.5 animate-pulse bg-aura-accent align-text-bottom"></span>{/if}
          </p>
          {#if usage}
            <div class="mt-4 flex flex-wrap gap-2 text-[11px] text-aura-text-muted">
              <span class="rounded-full border border-aura-border bg-aura-surface-soft px-2.5 py-1">
                输入 {usage.prompt_tokens.toLocaleString()}
              </span>
              <span class="rounded-full border border-aura-border bg-aura-surface-soft px-2.5 py-1">
                输出 {usage.completion_tokens.toLocaleString()}
              </span>
              <span class="rounded-full border border-aura-border bg-aura-surface-soft px-2.5 py-1">
                总计 {usage.total_tokens.toLocaleString()}
              </span>
            </div>
          {/if}
        </div>
      {:else}
        <div class="flex min-h-[88px] w-full flex-1 flex-col items-center justify-center gap-3 px-3 text-center">
          <p class="max-w-[280px] text-sm leading-7 text-aura-error/90">
            {errorMessage || '翻译失败。'}
          </p>
          <button
            type="button"
            data-testid="try-again-button"
            class="rounded-full bg-aura-accent px-5 py-2 text-[12px] font-display font-medium text-white transition-all duration-200 hover:brightness-105 active:brightness-95 disabled:cursor-not-allowed disabled:opacity-50"
            onclick={() => onretry?.()}
            disabled={!draftSourceText.trim()}
          >
            重试
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>
