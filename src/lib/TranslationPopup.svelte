<script lang="ts">
  /**
   * TranslationPopup — The glassmorphic floating card.
   * Shows source text, loading skeleton, streaming result, or error state.
   */
  import { Spring } from 'svelte/motion';
  import SkeletonLoader from './SkeletonLoader.svelte';
  import LanguageSelector from './LanguageSelector.svelte';
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';

  type Props = {
    viewState: 'idle' | 'loading' | 'streaming' | 'result' | 'error';
    sourceText: string;
    translatedText: string;
    errorMessage: string;
    sourceLang: string;
    targetLang: string;
    onLanguageChange: (source: string, target: string) => void;
  };

  let {
    viewState,
    sourceText,
    translatedText,
    errorMessage,
    sourceLang,
    targetLang,
    onLanguageChange,
  }: Props = $props();

  // Copy button spring animation
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
  class="relative flex flex-col h-full rounded-[18px] overflow-hidden
         border border-aura-border"
  style="
    background: rgba(12, 12, 20, 0.78);
    backdrop-filter: blur(28px) saturate(1.4);
    -webkit-backdrop-filter: blur(28px) saturate(1.4);
    box-shadow:
      0 0 0 1px rgba(255,255,255,0.04) inset,
      0 8px 32px rgba(0,0,0,0.5),
      0 2px 8px rgba(0,0,0,0.3),
      0 0 60px rgba(124,106,239,0.06);
  "
>
  <!-- Drag Region (top bar) -->
  <div
    class="flex items-center justify-between px-4 py-2.5 shrink-0"
    data-tauri-drag-region
  >
    <div class="flex items-center gap-2" data-tauri-drag-region>
      <div class="w-2 h-2 rounded-full bg-aura-accent opacity-70"></div>
      <span class="text-xs font-display font-medium text-aura-text-dim tracking-wide uppercase" data-tauri-drag-region>
        Aura
      </span>
    </div>

    <!-- Status indicator -->
    {#if viewState === 'loading' || viewState === 'streaming'}
      <div class="flex items-center gap-1.5">
        <div class="w-1.5 h-1.5 rounded-full bg-aura-accent animate-pulse"></div>
        <span class="text-[10px] text-aura-text-muted font-display">
          {viewState === 'loading' ? 'connecting...' : 'translating...'}
        </span>
      </div>
    {/if}
  </div>

  <!-- Language Selector -->
  <div class="px-4 pb-2 shrink-0">
    <LanguageSelector
      {sourceLang}
      {targetLang}
      onchange={onLanguageChange}
    />
  </div>

  <!-- Source Text -->
  {#if sourceText}
    <div class="px-4 pb-2 shrink-0">
      <div class="px-3 py-2 rounded-lg bg-aura-glass border border-aura-border">
        <p class="text-xs text-aura-text-dim font-body leading-relaxed line-clamp-2 select-text">
          {sourceText}
        </p>
      </div>
    </div>
  {/if}

  <!-- Divider -->
  <div class="mx-4 h-px bg-gradient-to-r from-transparent via-aura-border to-transparent shrink-0"></div>

  <!-- Content Area -->
  <div class="flex-1 px-4 py-3 overflow-y-auto min-h-0">
    {#if viewState === 'idle'}
      <div class="flex items-center justify-center h-full">
        <p class="text-sm text-aura-text-muted font-display italic">
          Copy text and press Ctrl+T
        </p>
      </div>
    {:else if viewState === 'loading'}
      <SkeletonLoader />
    {:else if viewState === 'streaming' || viewState === 'result'}
      <p
        class="text-sm text-aura-text font-body leading-relaxed select-text whitespace-pre-wrap"
        style="animation: fade-in-up 0.3s ease-out both;"
      >
        {translatedText}{#if viewState === 'streaming'}<span class="inline-block w-0.5 h-4 bg-aura-accent ml-0.5 animate-pulse align-text-bottom"></span>{/if}
      </p>
    {:else if viewState === 'error'}
      <div class="flex flex-col items-center justify-center h-full gap-2">
        <svg class="w-6 h-6 text-aura-error opacity-70" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
        </svg>
        <p class="text-xs text-aura-error/80 font-body text-center max-w-[280px]">
          {errorMessage || 'Translation failed'}
        </p>
      </div>
    {/if}
  </div>

  <!-- Footer (copy button) -->
  {#if (viewState === 'result' || viewState === 'streaming') && translatedText}
    <div class="flex items-center justify-end px-4 py-2 shrink-0 border-t border-aura-border/50">
      <button
        class="flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-display
               text-aura-text-dim hover:text-aura-accent
               bg-aura-glass hover:bg-aura-accent-soft
               border border-transparent hover:border-aura-border-accent
               transition-all duration-200"
        style:transform="scale({copyScale.current})"
        onclick={copyResult}
      >
        {#if copied}
          <svg class="w-3.5 h-3.5 text-aura-success" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
          </svg>
          <span class="text-aura-success">Copied!</span>
        {:else}
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M15.666 3.888A2.25 2.25 0 0 0 13.5 2.25h-3c-1.03 0-1.9.693-2.166 1.638m7.332 0c.055.194.084.4.084.612v0a.75.75 0 0 1-.75.75H9.75a.75.75 0 0 1-.75-.75v0c0-.212.03-.418.084-.612m7.332 0c.646.049 1.288.11 1.927.184 1.1.128 1.907 1.077 1.907 2.185V19.5a2.25 2.25 0 0 1-2.25 2.25H6.75A2.25 2.25 0 0 1 4.5 19.5V6.257c0-1.108.806-2.057 1.907-2.185a48.208 48.208 0 0 1 1.927-.184" />
          </svg>
          <span>Copy</span>
        {/if}
      </button>
    </div>
  {/if}
</div>
