<script lang="ts">
  import { LANGUAGES } from './languages';
  import {
    formatHistoryTimestamp,
    formatTokenCount,
    type TranslationHistoryEntry,
  } from './translationHistory';

  type Props = {
    entries: TranslationHistoryEntry[];
    oncopy?: (translatedText: string) => void;
    onretry?: (entryId: string) => void;
    ondelete?: (entryId: string) => void;
    onclear?: () => void;
  };

  let { entries, oncopy, onretry, ondelete, onclear }: Props = $props();

  const CLEAR_CONFIRM_TIMEOUT_MS = 5000;
  let confirmingClear = $state(false);
  let clearConfirmTimer: ReturnType<typeof setTimeout> | null = null;

  function clearClearConfirmTimer() {
    if (clearConfirmTimer !== null) {
      clearTimeout(clearConfirmTimer);
      clearConfirmTimer = null;
    }
  }

  function enterClearConfirm() {
    clearClearConfirmTimer();
    confirmingClear = true;
    clearConfirmTimer = setTimeout(() => {
      confirmingClear = false;
      clearConfirmTimer = null;
    }, CLEAR_CONFIRM_TIMEOUT_MS);
  }

  function cancelClearConfirm() {
    clearClearConfirmTimer();
    confirmingClear = false;
  }

  function commitClear() {
    clearClearConfirmTimer();
    confirmingClear = false;
    onclear?.();
  }

  function providerLabel(provider: string) {
    switch (provider) {
      case 'deepseek':
        return 'DeepSeek';
      case 'openrouter':
        return 'OpenRouter';
      case 'ollama':
        return 'Ollama';
      default:
        return provider;
    }
  }

  function languageLabel(code: string) {
    return LANGUAGES.find((language) => language.code === code)?.label ?? code;
  }

  $effect(() => {
    return () => clearClearConfirmTimer();
  });
</script>

<section class="space-y-3 rounded-xl border border-aura-border bg-white/80 px-5 py-5">
  <div class="flex items-start justify-between gap-4">
    <div>
      <p class="aura-section-title">
        最近历史
      </p>
      <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
        Aura 会在本机保留最近 50 条成功或失败的翻译请求。
      </p>
    </div>

    {#if entries.length > 0}
      {#if confirmingClear}
        <div class="flex flex-wrap items-center gap-2" data-testid="clear-confirm-group">
          <button
            class="rounded-md border border-aura-error bg-aura-error px-3 py-1.5 text-[11px] font-display font-semibold text-white transition-colors duration-150 hover:brightness-110"
            type="button"
            data-testid="clear-confirm-commit"
            onclick={commitClear}
          >
            确认清空
          </button>
          <button
            class="rounded-md border border-aura-border bg-white px-3 py-1.5 text-[11px] font-medium text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-text"
            type="button"
            data-testid="clear-confirm-cancel"
            onclick={cancelClearConfirm}
          >
            取消
          </button>
        </div>
      {:else}
        <button
          class="rounded-md border border-aura-border bg-white px-3 py-1.5 text-[11px] font-medium text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-text"
          type="button"
          data-testid="clear-all-button"
          onclick={enterClearConfirm}
        >
          清空全部
        </button>
      {/if}
    {/if}
  </div>

  {#if entries.length === 0}
    <div
      class="rounded-md border border-dashed border-aura-border bg-aura-surface-soft px-3 py-3 text-xs leading-relaxed text-aura-text-dim"
      data-testid="history-empty"
    >
      暂无翻译历史。开始翻译后，最近的成功和失败记录会显示在这里。
    </div>
  {:else}
    <div class="space-y-3" data-testid="history-list">
      {#each entries as entry (entry.id)}
        <article class="rounded-[14px] border border-aura-border bg-white px-3 py-3 shadow-[0_10px_22px_rgba(89,104,129,0.06)]">
          <div class="flex flex-wrap items-center gap-2">
            <span class={`rounded-full px-2.5 py-1 text-[11px] font-display font-medium ${
              entry.status === 'success'
                ? 'bg-aura-accent-soft text-aura-accent'
                : 'bg-[#fff4f6] text-aura-error'
            }`}>
              {entry.status === 'success' ? '成功' : '失败'}
            </span>
            <span class="rounded-full border border-aura-border px-2.5 py-1 text-[11px] font-display font-medium text-aura-text-dim">
              {languageLabel(entry.source_lang)} → {languageLabel(entry.target_lang)}
            </span>
            <span class="rounded-full border border-aura-border px-2.5 py-1 text-[11px] font-display font-medium text-aura-text-dim">
              {providerLabel(entry.provider)} · {entry.model}
            </span>
            <span class="text-[11px] text-aura-text-muted">{formatHistoryTimestamp(entry.created_at_ms)}</span>
            {#if entry.usage}
              <span class="rounded-full border border-aura-accent/25 bg-aura-accent-soft px-2.5 py-1 text-[11px] font-display font-medium text-aura-accent">
                {formatTokenCount(entry.usage.total_tokens)} token
              </span>
            {/if}
          </div>

          <div class="mt-3 grid gap-3 md:grid-cols-2">
            <div class="rounded-md border border-aura-border/80 bg-aura-surface-soft px-3 py-3">
              <p class="aura-section-title">
                原文
              </p>
              <p class="mt-2 whitespace-pre-wrap break-words text-xs leading-6 text-aura-text-dim">
                {entry.source_text}
              </p>
            </div>

            <div class={`rounded-md border px-3 py-3 ${
              entry.status === 'success'
                ? 'border-aura-border/80 bg-white'
                : 'border-aura-error/20 bg-[#fff8f9]'
            }`}>
              <p class="aura-section-title">
                {entry.status === 'success' ? '译文' : '错误'}
              </p>
              <p class={`mt-2 whitespace-pre-wrap break-words text-xs leading-6 ${
                entry.status === 'success' ? 'text-aura-text' : 'text-aura-error/90'
              }`}>
                {entry.status === 'success' ? entry.translated_text : (entry.error_message ?? '翻译失败。')}
              </p>
            </div>
          </div>

          {#if entry.usage}
            <div class="mt-3 flex flex-wrap gap-2 text-[11px] text-aura-text-muted">
              <span class="rounded-full border border-aura-border bg-aura-surface-soft px-2.5 py-1">
                输入 {formatTokenCount(entry.usage.prompt_tokens)}
              </span>
              <span class="rounded-full border border-aura-border bg-aura-surface-soft px-2.5 py-1">
                输出 {formatTokenCount(entry.usage.completion_tokens)}
              </span>
              <span class="rounded-full border border-aura-border bg-aura-surface-soft px-2.5 py-1">
                总计 {formatTokenCount(entry.usage.total_tokens)}
              </span>
            </div>
          {/if}

          <div class="mt-3 flex flex-wrap gap-2">
            <button
              class="rounded-md border border-aura-border bg-white px-3 py-1.5 text-[11px] font-medium text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-text"
              type="button"
              onclick={() => onretry?.(entry.id)}
            >
              重试
            </button>

            {#if entry.translated_text}
              <button
                class="rounded-md border border-aura-border bg-white px-3 py-1.5 text-[11px] font-medium text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-text"
                type="button"
                onclick={() => oncopy?.(entry.translated_text)}
              >
                复制
              </button>
            {/if}

            <button
              class="rounded-md border border-aura-border bg-white px-3 py-1.5 text-[11px] font-medium text-aura-text-dim transition-colors duration-150 hover:border-aura-error/30 hover:text-aura-error"
              type="button"
              onclick={() => ondelete?.(entry.id)}
            >
              删除
            </button>
          </div>
        </article>
      {/each}
    </div>
  {/if}
</section>
