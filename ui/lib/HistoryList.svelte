<script lang="ts">
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

  $effect(() => {
    return () => clearClearConfirmTimer();
  });
</script>

<section class="space-y-3 rounded-lg border border-aura-border bg-white/80 px-4 py-4">
  <div class="flex items-start justify-between gap-4">
    <div>
      <p class="text-[10px] font-display font-semibold uppercase tracking-[0.22em] text-aura-text-muted">
        Recent History
      </p>
      <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
        Aura keeps the latest 50 successful or failed translation requests on this machine.
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
            Confirm clear all
          </button>
          <button
            class="rounded-md border border-aura-border bg-white px-3 py-1.5 text-[11px] font-medium text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-text"
            type="button"
            data-testid="clear-confirm-cancel"
            onclick={cancelClearConfirm}
          >
            Cancel
          </button>
        </div>
      {:else}
        <button
          class="rounded-md border border-aura-border bg-white px-3 py-1.5 text-[11px] font-medium text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-text"
          type="button"
          data-testid="clear-all-button"
          onclick={enterClearConfirm}
        >
          Clear all
        </button>
      {/if}
    {/if}
  </div>

  {#if entries.length === 0}
    <div
      class="rounded-md border border-dashed border-aura-border bg-aura-surface-soft px-3 py-3 text-xs leading-relaxed text-aura-text-dim"
      data-testid="history-empty"
    >
      No translation history yet. Aura will keep recent successes and failures here once you start translating.
    </div>
  {:else}
    <div class="space-y-3" data-testid="history-list">
      {#each entries as entry (entry.id)}
        <article class="rounded-[14px] border border-aura-border bg-white px-3 py-3 shadow-[0_10px_22px_rgba(89,104,129,0.06)]">
          <div class="flex flex-wrap items-center gap-2">
            <span class={`rounded-full px-2.5 py-1 text-[10px] font-display font-semibold uppercase tracking-[0.14em] ${
              entry.status === 'success'
                ? 'bg-aura-accent-soft text-aura-accent'
                : 'bg-[#fff4f6] text-aura-error'
            }`}>
              {entry.status === 'success' ? 'Success' : 'Failed'}
            </span>
            <span class="rounded-full border border-aura-border px-2.5 py-1 text-[10px] font-display font-semibold uppercase tracking-[0.14em] text-aura-text-dim">
              {entry.source_lang} to {entry.target_lang}
            </span>
            <span class="rounded-full border border-aura-border px-2.5 py-1 text-[10px] font-display font-semibold uppercase tracking-[0.14em] text-aura-text-dim">
              {entry.provider} 路 {entry.model}
            </span>
            <span class="text-[11px] text-aura-text-muted">{formatHistoryTimestamp(entry.created_at_ms)}</span>
            {#if entry.usage}
              <span class="rounded-full border border-aura-accent/25 bg-aura-accent-soft px-2.5 py-1 text-[10px] font-display font-semibold uppercase tracking-[0.14em] text-aura-accent">
                {formatTokenCount(entry.usage.total_tokens)} tokens
              </span>
            {/if}
          </div>

          <div class="mt-3 grid gap-3 md:grid-cols-2">
            <div class="rounded-md border border-aura-border/80 bg-aura-surface-soft px-3 py-3">
              <p class="text-[10px] font-display font-semibold uppercase tracking-[0.18em] text-aura-text-muted">
                Source
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
              <p class="text-[10px] font-display font-semibold uppercase tracking-[0.18em] text-aura-text-muted">
                {entry.status === 'success' ? 'Result' : 'Error'}
              </p>
              <p class={`mt-2 whitespace-pre-wrap break-words text-xs leading-6 ${
                entry.status === 'success' ? 'text-aura-text' : 'text-aura-error/90'
              }`}>
                {entry.status === 'success' ? entry.translated_text : (entry.error_message ?? 'Translation failed.')}
              </p>
            </div>
          </div>

          {#if entry.usage}
            <div class="mt-3 flex flex-wrap gap-2 text-[11px] text-aura-text-muted">
              <span class="rounded-full border border-aura-border bg-aura-surface-soft px-2.5 py-1">
                Input {formatTokenCount(entry.usage.prompt_tokens)}
              </span>
              <span class="rounded-full border border-aura-border bg-aura-surface-soft px-2.5 py-1">
                Output {formatTokenCount(entry.usage.completion_tokens)}
              </span>
              <span class="rounded-full border border-aura-border bg-aura-surface-soft px-2.5 py-1">
                Total {formatTokenCount(entry.usage.total_tokens)}
              </span>
            </div>
          {/if}

          <div class="mt-3 flex flex-wrap gap-2">
            <button
              class="rounded-md border border-aura-border bg-white px-3 py-1.5 text-[11px] font-medium text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-text"
              type="button"
              onclick={() => onretry?.(entry.id)}
            >
              Retry
            </button>

            {#if entry.translated_text}
              <button
                class="rounded-md border border-aura-border bg-white px-3 py-1.5 text-[11px] font-medium text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-text"
                type="button"
                onclick={() => oncopy?.(entry.translated_text)}
              >
                Copy
              </button>
            {/if}

            <button
              class="rounded-md border border-aura-border bg-white px-3 py-1.5 text-[11px] font-medium text-aura-text-dim transition-colors duration-150 hover:border-aura-error/30 hover:text-aura-error"
              type="button"
              onclick={() => ondelete?.(entry.id)}
            >
              Delete
            </button>
          </div>
        </article>
      {/each}
    </div>
  {/if}
</section>
