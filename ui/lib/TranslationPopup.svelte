<script lang="ts">
  /**
   * TranslationPopup - Minimal floating translation bubble.
   */
  type Props = {
    viewState: 'idle' | 'loading' | 'streaming' | 'result' | 'error';
    translatedText: string;
    errorMessage: string;
    windowPinned: boolean;
    onTogglePinned?: (windowPinned: boolean) => void;
    oncancel?: () => void;
    ondismiss?: () => void;
    oncopy?: () => void;
  };

  let {
    viewState,
    translatedText,
    errorMessage,
    windowPinned,
    onTogglePinned,
    oncancel,
    ondismiss,
    oncopy,
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
    {#if viewState === 'idle'}
      <div
        class="flex min-h-[96px] w-full items-center justify-center rounded-[14px] border border-dashed border-aura-border bg-white/62 px-5 text-center"
        data-tauri-drag-region
      >
        <p class="max-w-[250px] text-sm leading-relaxed text-aura-text-dim" data-tauri-drag-region>
          Copy text to translate. Use the hotkey to recall the last result anytime.
        </p>
      </div>
    {:else if viewState === 'loading'}
      <div class="flex min-h-[96px] w-full items-center justify-center" data-tauri-drag-region>
        <div class="flex items-center gap-3 rounded-full border border-aura-border bg-white/78 px-4 py-2.5 text-sm text-aura-text-dim">
          <span class="h-2.5 w-2.5 animate-pulse rounded-full bg-aura-accent"></span>
          <span>Translating…</span>
        </div>
      </div>
    {:else if viewState === 'streaming' || viewState === 'result'}
      <div class="w-full overflow-y-auto pr-1">
        <p class="select-text whitespace-pre-wrap text-[15px] leading-7 text-aura-text">
          {translatedText}{#if viewState === 'streaming'}<span class="ml-0.5 inline-block h-5 w-0.5 animate-pulse bg-aura-accent align-text-bottom"></span>{/if}
        </p>
      </div>
    {:else}
      <div class="flex min-h-[96px] w-full items-center justify-center text-center">
        <p class="max-w-[260px] text-sm leading-relaxed text-aura-error/90">
          {errorMessage || 'Translation failed.'}
        </p>
      </div>
    {/if}
  </div>
</div>
