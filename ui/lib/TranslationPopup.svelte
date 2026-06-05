<script lang="ts">
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

  let sourceExpanded = $state(false);

  const isBusy = $derived(viewState === 'loading' || viewState === 'streaming');

  const canPasteBackDerived = $derived(
    canPasteBack ?? (Boolean(pasteBackSupported) && Boolean(pasteBackAvailable)),
  );

  const statusLabel = $derived.by(() => {
    if (viewState === 'loading') return '连接中';
    if (retryAttempt !== null) return `重试 ${retryAttempt}/3`;
    if (viewState === 'streaming') return '翻译中';
    if (viewState === 'error') return '需要处理';
    if (viewState === 'result') return '已就绪';
    return 'Aura';
  });

  const contextSummary = $derived.by(() => {
    const items: string[] = [];

    if (sourceLangLabel || targetLangLabel) {
      items.push(`${sourceLangLabel || '自动'} → ${targetLangLabel || '目标语言'}`);
    }

    if (providerLabel || modelLabel) {
      items.push(`${providerLabel || 'Provider'} · ${modelLabel || 'Model'}`);
    }

    if (usage) {
      items.push(`${usage.total_tokens.toLocaleString()} token`);
    }

    return items;
  });

  const sourcePreview = $derived.by(() => {
    const text = sourceText.trim().replace(/\s+/g, ' ');
    if (text.length <= 96) return text;
    return `${text.slice(0, 96)}...`;
  });

  const showContextBar = $derived(
    contextSummary.length > 0 || Boolean(sourceText) || showComposer,
  );

  const showSourceToggle = $derived(Boolean(sourceText) && !showComposer);

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

  $effect(() => {
    if (showComposer) {
      sourceExpanded = true;
      return;
    }

    if (!sourceText || viewState === 'idle') {
      sourceExpanded = false;
    }
  });
</script>

<div class="aura-glass-panel flex min-h-0 flex-col">
  <div
    class="flex shrink-0 items-center gap-3 border-b border-aura-border bg-white/75 px-4 py-2.5"
    data-testid="popup-status-bar"
  >
    <div class="flex shrink-0 items-center gap-2 text-[11px] font-display font-medium text-aura-text-dim">
      <span class="h-2 w-2 rounded-full bg-aura-accent shadow-[0_0_0_4px_var(--color-aura-accent-soft)]"></span>
      <span>{statusLabel}</span>
    </div>

    <div
      class="min-w-[92px] flex-1 cursor-move select-none text-center text-[11px] font-medium uppercase tracking-[0.14em] text-aura-text-muted"
      data-tauri-drag-region
      data-testid="window-drag-handle"
      aria-label="拖动窗口"
      title="拖动窗口"
    >
      Aura
    </div>

    <div class="flex shrink-0 items-center gap-1.5">
      <button
        class={`flex h-[2.15rem] items-center gap-1.5 rounded-md border px-2.5 text-[11px] font-display transition-colors duration-150 ${
          windowPinned
            ? 'border-aura-border-accent bg-aura-accent-soft text-aura-accent'
            : 'border-aura-border bg-aura-surface-strong text-aura-text-dim hover:border-aura-border-accent hover:bg-white hover:text-aura-text'
        }`}
        type="button"
        aria-pressed={windowPinned}
        onclick={() => onTogglePinned?.(!windowPinned)}
      >
        <span class="h-2 w-2 rounded-full bg-current"></span>
        <span>{windowPinned ? '已固定' : '固定'}</span>
      </button>

      <button
        class="aura-console-icon-button rounded-md"
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

  <div class="min-h-0 flex-1">
    <div
      class="flex h-full min-h-0 flex-col overflow-hidden bg-white/72"
      data-testid="popup-main-surface"
    >
      {#if showContextBar}
        <div class="border-b border-aura-border bg-aura-surface-soft/38 px-4 py-2.5">
          <div
            class="flex flex-wrap items-center gap-x-3 gap-y-2 text-[11px] text-aura-text-muted"
            data-testid="popup-context-row"
          >
            {#each contextSummary as item}
              <span class="font-mono tracking-[0.02em]">{item}</span>
            {/each}

            {#if showSourceToggle}
              <button
                class="ml-auto text-[11px] font-medium text-aura-text-dim transition-colors duration-150 hover:text-aura-accent"
                type="button"
                data-testid="source-toggle-button"
                onclick={() => (sourceExpanded = !sourceExpanded)}
              >
                {sourceExpanded ? '收起原文' : '查看原文'}
              </button>
            {/if}
          </div>

          {#if showComposer}
            <div class="mt-2.5 border-t border-aura-border pt-3">
              <div class="flex flex-wrap items-center justify-between gap-3">
                <div>
                  <p class="aura-section-title">原文草稿</p>
                  <p class="mt-1 text-xs leading-relaxed text-aura-text-muted">
                    在这里修改原文，按 Ctrl+Enter 直接重译。
                  </p>
                </div>

                <div class="flex flex-wrap items-center gap-2">
                  {#if hasDraftChanges}
                    <button
                      class="aura-console-button"
                      type="button"
                      onclick={() => onresetdraft?.()}
                    >
                      还原
                    </button>
                  {/if}

                  <button
                    class="aura-console-button"
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
                class="aura-console-textarea mt-3 min-h-[104px] bg-white/80"
              ></textarea>
            </div>
          {:else if showSourceToggle}
            <div
              class="mt-3 border-t border-aura-border pt-3"
              data-testid={sourceExpanded ? 'popup-source-panel' : 'popup-source-preview'}
            >
              <div class="flex flex-wrap items-center justify-between gap-2">
                <p class="aura-section-title">原文</p>
                <button
                  class="text-[11px] font-medium text-aura-text-dim transition-colors duration-150 hover:text-aura-accent"
                  type="button"
                  onclick={() => onretry?.()}
                  disabled={isBusy}
                >
                  重新翻译
                </button>
              </div>
              {#if sourceExpanded}
                <p class="mt-2.5 max-h-[108px] overflow-y-auto pr-1 text-sm leading-7 text-aura-text-dim">
                  {sourceText}
                </p>
              {:else}
                <p class="mt-2 line-clamp-2 select-text break-words text-xs leading-5 text-aura-text-muted">
                  {sourcePreview}
                </p>
              {/if}
            </div>
          {/if}
        </div>
      {/if}

      <div class="flex-1 min-h-0 px-5 py-4">
        {#if viewState === 'idle'}
          <div
            class="flex h-full items-center justify-center text-center"
            data-testid="popup-empty-state"
          >
            <p class="max-w-[280px] text-sm leading-7 text-aura-text-dim">
              {showComposer
                ? '在草稿区输入或粘贴原文，然后按 Ctrl+Enter 重新翻译全文。'
                : '复制要翻译的文本，按快捷键即可唤起 Aura。'}
            </p>
          </div>
        {:else if viewState === 'loading'}
          <div
            class="flex h-full items-center justify-center"
            data-testid="popup-loading-state"
          >
            <div class="flex flex-wrap items-center justify-center gap-3 border-y border-aura-border bg-aura-surface-soft px-4 py-3 text-sm text-aura-text-dim">
              <div class="flex items-center gap-3">
                <span class="h-2 w-2 animate-pulse rounded-full bg-aura-accent shadow-[0_0_0_4px_var(--color-aura-accent-soft)]"></span>
                <span>{retryAttempt !== null ? `正在重试请求 ${retryAttempt}/3...` : '正在翻译...'}</span>
              </div>
              <button
                class="aura-console-button h-8 min-h-8 px-2.5 py-1 text-xs hover:border-aura-error/40 hover:text-aura-error"
                onclick={() => oncancel?.()}
                type="button"
              >
                取消
              </button>
            </div>
          </div>
        {:else if viewState === 'streaming' || viewState === 'result'}
          <div class="flex h-full min-h-0 flex-col">
            <div class="min-h-0 flex-1 overflow-y-auto pr-1">
              <p class="select-text whitespace-pre-wrap break-words text-[15px] leading-[1.9rem] text-aura-text">
                {translatedText}{#if viewState === 'streaming'}<span class="ml-0.5 inline-block h-5 w-0.5 animate-pulse bg-aura-accent align-text-bottom"></span>{/if}
              </p>
            </div>

            {#if viewState === 'result'}
              <div
                class="mt-4 flex flex-wrap items-center justify-between gap-3 border-t border-aura-border pt-3"
                data-testid="popup-result-actions"
              >
                {#if usage}
                  <div class="flex min-w-0 flex-wrap gap-x-3 gap-y-1 text-[11px] text-aura-text-muted">
                    <span class="font-mono">输入 {usage.prompt_tokens.toLocaleString()}</span>
                    <span class="font-mono">输出 {usage.completion_tokens.toLocaleString()}</span>
                    <span class="font-mono">总计 {usage.total_tokens.toLocaleString()}</span>
                  </div>
                {:else}
                  <span class="min-h-[1rem]"></span>
                {/if}

                <div class="flex flex-wrap items-center justify-end gap-2">
                  <button
                    class="aura-console-button disabled:cursor-not-allowed disabled:opacity-40"
                    onclick={() => onpasteback?.()}
                    title={pasteBackTooltip}
                    type="button"
                    disabled={!canPasteBackDerived}
                  >
                    回填
                  </button>

                  <button
                    class="aura-console-button"
                    data-variant="primary"
                    onclick={() => oncopy?.()}
                    type="button"
                  >
                    复制译文
                  </button>
                </div>
              </div>
            {/if}
          </div>
        {:else}
          <div class="flex h-full flex-col items-start justify-center gap-4 border-y border-aura-error/20 bg-[#fff8f9] px-4 py-4">
            <div>
              <p class="aura-section-title text-aura-error">翻译失败</p>
              <p class="mt-2 max-w-[320px] text-sm leading-7 text-aura-error/90">
                {errorMessage || '翻译失败。'}
              </p>
            </div>

            <button
              type="button"
              data-testid="try-again-button"
              class="aura-console-button"
              data-variant="primary"
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
</div>
