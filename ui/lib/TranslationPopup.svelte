<script lang="ts">
  /**
   * TranslationPopup - Dense desktop translation panel with pinned window controls.
   */
  import { Spring } from 'svelte/motion';
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';
  import LanguageSelector from './LanguageSelector.svelte';
  import SkeletonLoader from './SkeletonLoader.svelte';

  type Props = {
    viewState: 'idle' | 'loading' | 'streaming' | 'result' | 'error';
    sourceText: string;
    translatedText: string;
    errorMessage: string;
    hotkeyLabel: string;
    sourceLang: string;
    targetLang: string;
    windowPinned: boolean;
    onLanguageChange: (source: string, target: string) => void;
    onTogglePinned?: (windowPinned: boolean) => void;
    oncancel?: () => void;
    ondismiss?: () => void;
  };

  let {
    viewState,
    sourceText,
    translatedText,
    errorMessage,
    hotkeyLabel,
    sourceLang,
    targetLang,
    windowPinned,
    onLanguageChange,
    onTogglePinned,
    oncancel,
    ondismiss,
  }: Props = $props();

  const copyScale = new Spring(1, { stiffness: 0.4, damping: 0.5 });
  let copied = $state(false);

  async function copyResult() {
    if (!translatedText) return;
    try {
      await writeText(translatedText);
      copied = true;
      copyScale.target = 1.2;
      setTimeout(() => {
        copyScale.target = 1;
      }, 150);
      setTimeout(() => {
        copied = false;
      }, 1500);
    } catch (e) {
      console.error('Failed to copy:', e);
    }
  }
</script>

<div
  class="relative flex h-full flex-col overflow-hidden rounded-[10px] border border-aura-border bg-[rgba(248,250,252,0.96)] text-aura-text shadow-[0_16px_36px_rgba(89,104,129,0.14)]"
  style="
    backdrop-filter: blur(18px) saturate(1.02);
    -webkit-backdrop-filter: blur(18px) saturate(1.02);
  "
>
  <div
    class="flex items-start justify-between gap-3 border-b border-aura-border px-4 py-3 shrink-0"
    data-tauri-drag-region
  >
    <div class="min-w-0 flex-1 space-y-1" data-tauri-drag-region>
      <div class="flex items-center gap-2" data-tauri-drag-region>
        <div class="h-2 w-2 rounded-full bg-aura-accent opacity-80"></div>
        <span class="text-[11px] font-display font-semibold tracking-[0.2em] text-aura-text uppercase" data-tauri-drag-region>
          Aura
        </span>
        <span
          class={`rounded-full px-2 py-0.5 text-[10px] font-display tracking-[0.12em] uppercase ${
            viewState === 'loading' || viewState === 'streaming'
              ? 'bg-aura-accent-soft text-aura-accent'
              : viewState === 'error'
                ? 'bg-aura-error/10 text-aura-error'
                : 'bg-aura-surface-soft text-aura-text-dim'
          }`}
          data-tauri-drag-region
        >
          {viewState === 'loading'
            ? 'Connecting'
            : viewState === 'streaming'
              ? 'Streaming'
              : viewState === 'error'
                ? 'Attention'
                : translatedText
                  ? 'Ready'
                  : 'Waiting'}
        </span>
      </div>
      <p class="text-xs leading-relaxed text-aura-text-dim" data-tauri-drag-region>
        {#if windowPinned}
          Pinned to stay visible while you compare other pages.
        {:else}
          Copy text, then press {hotkeyLabel}. Blur hides the window until you pin it.
        {/if}
      </p>
    </div>

    <div class="flex items-center gap-2">
      <button
        class={`flex h-8 items-center gap-1.5 rounded-md border px-2.5 text-[11px] font-display font-medium transition-all duration-150 ${
          windowPinned
            ? 'border-aura-accent bg-aura-accent text-white'
            : 'border-aura-border bg-white text-aura-text-dim hover:border-aura-border-accent hover:text-aura-text'
        }`}
        type="button"
        aria-pressed={windowPinned}
        onclick={() => onTogglePinned?.(!windowPinned)}
      >
        <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 17.25v4.5m0-4.5 3.75-3.75m-3.75 3.75-3.75-3.75m7.5-6.75V4.5A2.25 2.25 0 0 0 13.5 2.25h-3A2.25 2.25 0 0 0 8.25 4.5v2.25m7.5 0H8.25m7.5 0 3 5.25H5.25l3-5.25" />
        </svg>
        <span>{windowPinned ? 'Pinned' : 'Pin'}</span>
      </button>

      {#if viewState === 'loading' || viewState === 'streaming'}
        <button
          id="cancel-translate-btn"
          class="flex h-8 w-8 items-center justify-center rounded-md border border-aura-border bg-white text-aura-text-muted transition-all duration-150 hover:border-aura-error/30 hover:text-aura-error"
          onclick={() => oncancel?.()}
          aria-label="Cancel translation"
          type="button"
        >
          <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      {/if}

      <button
        class="flex h-8 w-8 items-center justify-center rounded-md border border-aura-border bg-white text-aura-text-muted transition-all duration-150 hover:border-aura-border-accent hover:text-aura-text"
        onclick={() => ondismiss?.()}
        aria-label="Close translator"
        type="button"
      >
        <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  </div>

  <div class="border-b border-aura-border px-4 py-2.5 shrink-0">
    <LanguageSelector
      {sourceLang}
      {targetLang}
      onchange={onLanguageChange}
    />
  </div>

  {#if sourceText}
    <div class="shrink-0 border-b border-aura-border bg-aura-surface-soft px-4 py-2.5">
      <div class="flex items-start gap-3">
        <span class="pt-0.5 text-[10px] font-display font-semibold uppercase tracking-[0.18em] text-aura-text-muted">
          Source
        </span>
        <p class="flex-1 text-xs text-aura-text-dim font-body leading-relaxed line-clamp-3 select-text">
          {sourceText}
        </p>
      </div>
    </div>
  {/if}

  <div class="flex-1 min-h-0 overflow-y-auto px-4 py-4">
    {#if viewState === 'idle'}
      <div class="flex h-full flex-col items-center justify-center gap-3 text-center">
        <div class="rounded-full border border-aura-border bg-white px-3 py-1 text-[10px] font-display font-semibold uppercase tracking-[0.18em] text-aura-text-muted">
          Quick capture
        </div>
        <p class="max-w-[320px] text-sm leading-relaxed text-aura-text-dim">
          Copy text, press {hotkeyLabel}, and pin the window when you need side-by-side comparison.
        </p>
      </div>
    {:else if viewState === 'loading'}
      <SkeletonLoader />
    {:else if viewState === 'streaming' || viewState === 'result'}
      <p
        class="text-[15px] text-aura-text font-body leading-7 select-text whitespace-pre-wrap"
        style="animation: fade-in-up 0.3s ease-out both;"
      >
        {translatedText}{#if viewState === 'streaming'}<span class="ml-0.5 inline-block h-5 w-0.5 animate-pulse bg-aura-accent align-text-bottom"></span>{/if}
      </p>
    {:else if viewState === 'error'}
      <div class="flex h-full flex-col items-center justify-center gap-3 text-center">
        <svg class="h-6 w-6 text-aura-error opacity-70" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
        </svg>
        <p class="max-w-[320px] text-sm leading-relaxed text-aura-error/85">
          {errorMessage || 'Translation failed'}
        </p>
      </div>
    {/if}
  </div>

  <div class="flex items-center justify-between gap-3 border-t border-aura-border bg-white/72 px-4 py-2.5 shrink-0">
    <p class="min-w-0 text-[11px] leading-relaxed text-aura-text-dim">
      {#if viewState === 'result' || viewState === 'streaming'}
        {windowPinned ? 'Pinned window stays visible while you compare source material.' : 'Pin the window if you want to keep it on screen while browsing.'}
      {:else}
        Press Esc to close, or keep this panel pinned for side-by-side reading.
      {/if}
    </p>

    {#if (viewState === 'result' || viewState === 'streaming') && translatedText}
      <button
        class="flex shrink-0 items-center gap-1.5 rounded-md border border-aura-border bg-white px-3 py-1.5 text-xs font-display text-aura-text-dim transition-all duration-200 hover:border-aura-border-accent hover:text-aura-accent"
        style:transform="scale({copyScale.current})"
        onclick={copyResult}
        type="button"
      >
        {#if copied}
          <svg class="h-3.5 w-3.5 text-aura-success" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
          </svg>
          <span class="text-aura-success">Copied!</span>
        {:else}
          <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M15.666 3.888A2.25 2.25 0 0 0 13.5 2.25h-3c-1.03 0-1.9.693-2.166 1.638m7.332 0c.055.194.084.4.084.612v0a.75.75 0 0 1-.75.75H9.75a.75.75 0 0 1-.75-.75v0c0-.212.03-.418.084-.612m7.332 0c.646.049 1.288.11 1.927.184 1.1.128 1.907 1.077 1.907 2.185V19.5a2.25 2.25 0 0 1-2.25 2.25H6.75A2.25 2.25 0 0 1 4.5 19.5V6.257c0-1.108.806-2.057 1.907-2.185a48.208 48.208 0 0 1 1.927-.184" />
          </svg>
          <span>Copy</span>
        {/if}
      </button>
    {/if}
  </div>
</div>
