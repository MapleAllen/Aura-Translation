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
    canPasteBack: boolean;
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
</script>

<div
  class="group relative flex h-full flex-col overflow-hidden rounded-[18px] border border-aura-border bg-[rgba(248,250,252,0.96)] text-aura-text shadow-[0_20px_42px_rgba(89,104,129,0.18)]"
  style="
    backdrop-filter: blur(18px) saturate(1.04);
    -webkit-backdrop-filter: blur(18px) saturate(1.04);
  "
>
  <div
    class="absolute inset-x-4 top-2 z-[1] h-5 rounded-full"
    data-tauri-drag-region
    aria-hidden="true"
  ></div>

  <div
    class="pointer-events-none absolute inset-x-0 top-0 flex items-center justify-between gap-3 px-3 py-3 opacity-0 transition-opacity duration-150 group-hover:opacity-100 group-focus-within:opacity-100"
    data-tauri-drag-region
  >
    <div
      class="rounded-full border border-aura-border/70 bg-white/78 px-2.5 py-1 text-[10px] font-display font-semibold uppercase tracking-[0.16em] text-aura-text-dim"
      data-tauri-drag-region
    >
      {#if viewState === 'loading'}
        Connecting
      {:else if retryAttempt !== null}
        Retry {retryAttempt}
      {:else if viewState === 'streaming'}
        Streaming
      {:else if viewState === 'error'}
        Attention
      {:else if viewState === 'result'}
        Ready
      {:else}
        Aura
      {/if}
    </div>

    <div class="pointer-events-auto flex items-center gap-2">
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
        <span>{windowPinned ? 'Pinned' : 'Pin'}</span>
      </button>

      {#if sourceText && !isBusy && !showComposer}
        <button
          class="flex h-8 w-8 items-center justify-center rounded-full border border-aura-border bg-white/84 text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-accent"
          onclick={() => onretry?.()}
          aria-label="Retry translation"
          type="button"
        >
          <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.9">
            <path stroke-linecap="round" stroke-linejoin="round" d="M16.023 9.348h4.992V4.356m-.937 4.992A9 9 0 1 0 6.75 18.75" />
          </svg>
        </button>
      {/if}

      {#if translatedText && !isBusy && canPasteBack}
        <button
          class="flex h-8 w-8 items-center justify-center rounded-full border border-aura-border bg-white/84 text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-accent"
          onclick={() => onpasteback?.()}
          aria-label="Paste translation back"
          type="button"
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
          aria-label="Copy translation"
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
          aria-label="Cancel translation"
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
        aria-label="Close translator"
        type="button"
      >
        <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.1">
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  </div>

  <div class="flex min-h-0 flex-1 items-stretch px-5 py-5" data-tauri-drag-region>
    <div class="flex w-full min-h-0 flex-col">
      {#if sourceText || showComposer}
        <div class="mb-4 space-y-2">
          <div class="flex flex-wrap gap-2">
            <span class="rounded-full border border-aura-border/80 bg-white/84 px-2.5 py-1 text-[10px] font-display font-semibold uppercase tracking-[0.14em] text-aura-text-dim">
              {sourceLangLabel} to {targetLangLabel}
            </span>
            <span class="rounded-full border border-aura-border/80 bg-white/84 px-2.5 py-1 text-[10px] font-display font-semibold uppercase tracking-[0.14em] text-aura-text-dim">
              {providerLabel} · {modelLabel}
            </span>
            {#if retryAttempt !== null}
              <span class="rounded-full border border-aura-accent/25 bg-aura-accent-soft px-2.5 py-1 text-[10px] font-display font-semibold uppercase tracking-[0.14em] text-aura-accent">
                Retry {retryAttempt}/3
              </span>
            {/if}
            {#if usage}
              <span class="rounded-full border border-aura-accent/25 bg-aura-accent-soft px-2.5 py-1 text-[10px] font-display font-semibold uppercase tracking-[0.14em] text-aura-accent">
                {usage.total_tokens.toLocaleString()} tokens
              </span>
            {/if}
          </div>

          {#if showComposer}
            <div class="rounded-[14px] border border-aura-border bg-white/68 px-4 py-3">
              <div class="flex flex-wrap items-center justify-between gap-3">
                <div>
                  <p class="text-[10px] font-display font-semibold uppercase tracking-[0.18em] text-aura-text-muted">
                    Draft
                  </p>
                  <p class="mt-1 text-[11px] leading-relaxed text-aura-text-muted">
                    Edit the source here and press Ctrl+Enter to re-translate without leaving Aura.
                  </p>
                </div>

                <div class="flex flex-wrap items-center gap-2">
                  {#if hasDraftChanges}
                    <button
                      class="rounded-full border border-aura-border bg-white px-3 py-1.5 text-[11px] font-medium text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-text"
                      type="button"
                      onclick={() => onresetdraft?.()}
                    >
                      Reset
                    </button>
                  {/if}

                  <button
                    class="rounded-full border border-aura-accent/30 bg-aura-accent-soft px-3 py-1.5 text-[11px] font-medium text-aura-accent transition-colors duration-150 hover:border-aura-accent hover:bg-aura-accent hover:text-white disabled:cursor-not-allowed disabled:opacity-50"
                    type="button"
                    onclick={() => ontranslatedraft?.()}
                    disabled={!draftSourceText.trim() || isBusy}
                  >
                    Translate edits
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
                placeholder="Type or revise source text here."
                class="mt-3 min-h-[104px] w-full resize-none rounded-[12px] border border-aura-border bg-white px-3 py-3 text-[13px] leading-6 text-aura-text outline-none transition-colors duration-150 placeholder:text-aura-text-muted focus:border-aura-accent focus:ring-2 focus:ring-aura-accent/15"
              ></textarea>
            </div>
          {:else}
            <div class="rounded-[14px] border border-aura-border bg-white/68 px-4 py-3">
              <p class="text-[10px] font-display font-semibold uppercase tracking-[0.18em] text-aura-text-muted">
                Source
              </p>
              <p
                class="mt-2 text-[13px] leading-6 text-aura-text-dim"
                style="display: -webkit-box; overflow: hidden; -webkit-box-orient: vertical; -webkit-line-clamp: 2;"
              >
                {sourceText}
              </p>
            </div>
          {/if}
        </div>
      {/if}

      {#if viewState === 'idle'}
        <div
          class="flex min-h-[96px] w-full items-center justify-center rounded-[14px] border border-dashed border-aura-border bg-white/62 px-5 text-center"
          data-tauri-drag-region
        >
          <p class="max-w-[250px] text-sm leading-relaxed text-aura-text-dim" data-tauri-drag-region>
            {showComposer
              ? 'Type or paste source text into the draft area, then press Ctrl+Enter to translate.'
              : 'Copy text to translate. Use the hotkey to recall the last result anytime.'}
          </p>
        </div>
      {:else if viewState === 'loading'}
        <div class="flex min-h-[96px] w-full flex-1 items-center justify-center" data-tauri-drag-region>
          <div class="flex items-center gap-3 rounded-full border border-aura-border bg-white/78 px-4 py-2.5 text-sm text-aura-text-dim">
            <span class="h-2.5 w-2.5 animate-pulse rounded-full bg-aura-accent"></span>
            <span>{retryAttempt !== null ? `Retrying request (${retryAttempt}/3)...` : 'Translating...'}</span>
          </div>
        </div>
      {:else if viewState === 'streaming' || viewState === 'result'}
        <div class="w-full min-h-0 flex-1 overflow-y-auto pr-1">
          <p class="select-text whitespace-pre-wrap text-[15px] leading-7 text-aura-text">
            {translatedText}{#if viewState === 'streaming'}<span class="ml-0.5 inline-block h-5 w-0.5 animate-pulse bg-aura-accent align-text-bottom"></span>{/if}
          </p>
          {#if usage}
            <div class="mt-4 flex flex-wrap gap-2 text-[11px] text-aura-text-muted">
              <span class="rounded-full border border-aura-border bg-aura-surface-soft px-2.5 py-1">
                Input {usage.prompt_tokens.toLocaleString()}
              </span>
              <span class="rounded-full border border-aura-border bg-aura-surface-soft px-2.5 py-1">
                Output {usage.completion_tokens.toLocaleString()}
              </span>
              <span class="rounded-full border border-aura-border bg-aura-surface-soft px-2.5 py-1">
                Total {usage.total_tokens.toLocaleString()}
              </span>
            </div>
          {/if}
        </div>
      {:else}
        <div class="flex min-h-[96px] w-full flex-1 items-center justify-center text-center">
          <p class="max-w-[260px] text-sm leading-relaxed text-aura-error/90">
            {errorMessage || 'Translation failed.'}
          </p>
        </div>
      {/if}
    </div>
  </div>
</div>
