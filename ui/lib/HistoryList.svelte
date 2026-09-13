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
    onretry?: (entryId: string, retryWithOriginal: boolean) => void;
    ondelete?: (entryId: string) => void;
    onclear?: () => void;
  };

  let { entries, oncopy, onretry, ondelete, onclear }: Props = $props();

  const CLEAR_CONFIRM_TIMEOUT_MS = 5000;
  let confirmingClear = $state(false);
  let expandedEntryId = $state<string | null>(null);
  let searchQuery = $state('');
  let statusFilter = $state<'all' | 'success' | 'error'>('all');
  let languagePairFilter = $state('all');
  let clearConfirmTimer: ReturnType<typeof setTimeout> | null = null;

  const languagePairs = $derived.by(() => {
    const pairs = new Map<string, string>();
    for (const entry of entries) {
      const key = JSON.stringify([entry.source_lang, entry.target_lang]);
      pairs.set(key, `${languageLabel(entry.source_lang)} → ${languageLabel(entry.target_lang)}`);
    }
    return Array.from(pairs.entries());
  });

  const filteredEntries = $derived.by(() => {
    const query = searchQuery.trim().toLocaleLowerCase();
    return entries.filter((entry) => {
      const pairKey = JSON.stringify([entry.source_lang, entry.target_lang]);
      const matchesText =
        !query ||
        entry.source_text.toLocaleLowerCase().includes(query) ||
        entry.translated_text.toLocaleLowerCase().includes(query);
      const matchesStatus = statusFilter === 'all' || entry.status === statusFilter;
      const matchesPair = languagePairFilter === 'all' || pairKey === languagePairFilter;
      return matchesText && matchesStatus && matchesPair;
    });
  });

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
    searchQuery = '';
    statusFilter = 'all';
    languagePairFilter = 'all';
    onclear?.();
  }

  function toggleDetails(entryId: string) {
    expandedEntryId = expandedEntryId === entryId ? null : entryId;
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
    if (entries.length === 0) {
      searchQuery = '';
      statusFilter = 'all';
      languagePairFilter = 'all';
    }
  });

  $effect(() => {
    return () => clearClearConfirmTimer();
  });
</script>

<section class="space-y-4">
  <div class="flex items-start justify-between gap-4">
    <div>
      <p class="aura-section-title">最近历史</p>
      <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
        默认显示摘要日志，需要时再展开查看原文、译文和错误详情。
      </p>
    </div>

    {#if entries.length > 0}
      {#if confirmingClear}
        <div class="flex flex-wrap items-center gap-2" data-testid="clear-confirm-group">
          <button
            class="aura-console-button"
            data-variant="primary"
            type="button"
            data-testid="clear-confirm-commit"
            onclick={commitClear}
          >
            确认清空
          </button>
          <button
            class="aura-console-button"
            type="button"
            data-testid="clear-confirm-cancel"
            onclick={cancelClearConfirm}
          >
            取消
          </button>
        </div>
      {:else}
        <button
          class="aura-console-button"
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
      class="rounded-lg border border-dashed border-aura-border bg-aura-surface-soft px-3.5 py-3 text-xs leading-relaxed text-aura-text-dim"
      data-testid="history-empty"
    >
      暂无翻译历史。开始翻译后，最近的成功和失败记录会显示在这里。
    </div>
  {:else}
    <div class="grid gap-2 rounded-lg border border-aura-border bg-aura-surface-soft/60 p-3 md:grid-cols-[minmax(0,1fr)_auto_auto]">
      <input
        class="aura-input"
        type="search"
        placeholder="搜索原文或译文"
        data-testid="history-search"
        bind:value={searchQuery}
      />
      <select class="aura-input" data-testid="history-status-filter" bind:value={statusFilter}>
        <option value="all">全部状态</option>
        <option value="success">成功</option>
        <option value="error">失败</option>
      </select>
      <select class="aura-input" data-testid="history-language-filter" bind:value={languagePairFilter}>
        <option value="all">全部语言对</option>
        {#each languagePairs as [value, label]}
          <option {value}>{label}</option>
        {/each}
      </select>
    </div>

    {#if filteredEntries.length === 0}
      <div
        class="rounded-lg border border-dashed border-aura-border bg-aura-surface-soft px-3.5 py-3 text-xs leading-relaxed text-aura-text-dim"
        data-testid="history-no-results"
      >
        没有符合当前筛选条件的历史记录。
      </div>
    {:else}
    <div class="overflow-hidden rounded-lg border border-aura-border bg-aura-glass" data-testid="history-list">
      {#each filteredEntries as entry, index (entry.id)}
        <article class={index === 0 ? '' : 'border-t border-aura-border'}>
          <div class="grid gap-3 px-4 py-3.5 lg:grid-cols-[minmax(0,1fr)_auto]">
            <div class="min-w-0">
              <div class="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-aura-text-dim">
                <span class={entry.status === 'success' ? 'text-aura-accent' : 'text-aura-error'}>
                  {entry.status === 'success' ? '成功' : '失败'}
                </span>
                <span class="font-mono">{languageLabel(entry.source_lang)} → {languageLabel(entry.target_lang)}</span>
                <span class="font-mono">{providerLabel(entry.provider)} · {entry.model}</span>
                <span>{formatHistoryTimestamp(entry.created_at_ms)}</span>
                {#if entry.usage}
                  <span class="font-mono">{formatTokenCount(entry.usage.total_tokens)} token</span>
                {/if}
              </div>
              <p class="mt-2 truncate text-sm text-aura-text">
                {entry.source_text}
              </p>
            </div>

            <div class="flex flex-wrap items-start justify-end gap-1.5">
              <button
                class="aura-console-button"
                type="button"
                data-testid={`history-retry-${entry.id}`}
                title={`使用原始配置重试：${providerLabel(entry.provider)} · ${entry.model}`}
                onclick={() => onretry?.(entry.id, true)}
              >
                重试
              </button>

              {#if entry.translated_text}
                <button
                  class="aura-console-button"
                  type="button"
                  data-testid={`history-copy-${entry.id}`}
                  onclick={() => oncopy?.(entry.translated_text)}
                >
                  复制
                </button>
              {/if}

              <button
                class="aura-console-button"
                type="button"
                data-testid={`history-delete-${entry.id}`}
                onclick={() => ondelete?.(entry.id)}
              >
                删除
              </button>

              <button
                class="aura-console-button"
                type="button"
                data-testid={`history-toggle-${entry.id}`}
                aria-expanded={expandedEntryId === entry.id}
                onclick={() => toggleDetails(entry.id)}
              >
                {expandedEntryId === entry.id ? '收起' : '展开'}
              </button>
            </div>
          </div>

          {#if expandedEntryId === entry.id}
            <div
              class="grid gap-4 border-t border-aura-border bg-aura-surface-soft/50 px-4 py-4 md:grid-cols-2"
              data-testid={`history-details-${entry.id}`}
            >
              <div>
                <p class="aura-section-title">原文</p>
                <p class="mt-2 whitespace-pre-wrap break-words text-xs leading-6 text-aura-text-dim">
                  {entry.source_text}
                </p>
              </div>

              <div>
                <p class="aura-section-title">{entry.status === 'success' ? '译文' : '错误'}</p>
                <p class={`mt-2 whitespace-pre-wrap break-words text-xs leading-6 ${
                  entry.status === 'success' ? 'text-aura-text' : 'text-aura-error/90'
                }`}>
                  {entry.status === 'success' ? entry.translated_text : (entry.error_message ?? '翻译失败。')}
                </p>
              </div>

              {#if entry.usage}
                <div class="md:col-span-2 flex flex-wrap gap-x-4 gap-y-1 border-t border-aura-border pt-3 text-xs text-aura-text-muted">
                  <span class="font-mono">输入 {formatTokenCount(entry.usage.prompt_tokens)}</span>
                  <span class="font-mono">输出 {formatTokenCount(entry.usage.completion_tokens)}</span>
                  <span class="font-mono">总计 {formatTokenCount(entry.usage.total_tokens)}</span>
                </div>
              {/if}
            </div>
          {/if}
        </article>
      {/each}
    </div>
    {/if}
  {/if}
</section>
