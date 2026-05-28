<script lang="ts">
  /**
   * TranslationPopup - Floating translation surface with a dominant reading panel.
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
    hotkeyLabel: string;
    sourceLang: string;
    targetLang: string;
    onLanguageChange: (source: string, target: string) => void;
    oncancel?: () => void;
  };

  let {
    viewState,
    sourceText,
    translatedText,
    errorMessage,
    hotkeyLabel,
    sourceLang,
    targetLang,
    onLanguageChange,
    oncancel,
  }: Props = $props();

  const copyScale = new Spring(1, { stiffness: 0.4, damping: 0.5 });
  let copied = $state(false);

  async function copyResult() {
    if (!translatedText) return;
    try {
      await writeText(translatedText);
      copied = true;
      copyScale.target = 1.08;
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
  class="relative flex h-full flex-col overflow-hidden rounded-[24px] border border-aura-border/80"
  data-testid="popup-shell"
  style="
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.72) 0%, rgba(246, 242, 236, 0.92) 100%);
    backdrop-filter: blur(20px) saturate(1.1);
    -webkit-backdrop-filter: blur(20px) saturate(1.1);
    box-shadow: 0 20px 44px var(--color-aura-shadow);
  "
>
  <div
    class="flex shrink-0 items-center justify-between px-5 pt-4 pb-3"
    data-tauri-drag-region
  >
    <div class="flex items-center gap-3" data-tauri-drag-region>
      <div class="flex h-8 w-8 items-center justify-center rounded-full bg-aura-surface-strong text-[11px] font-display font-semibold tracking-[0.24em] text-aura-accent shadow-sm">
        AU
      </div>
      <div data-tauri-drag-region>
        <p class="text-[11px] font-display font-medium uppercase tracking-[0.22em] text-aura-text-dim" data-tauri-drag-region>
          Aura Translation
        </p>
        <p class="text-[11px] text-aura-text-muted" data-tauri-drag-region>
          Clipboard translation, kept close to the tray.
        </p>
      </div>
    </div>

    {#if viewState === 'loading' || viewState === 'streaming'}
      <div class="flex items-center gap-2 rounded-full bg-aura-surface-soft px-2.5 py-1 text-[11px] text-aura-text-dim">
        <span class="h-2 w-2 rounded-full bg-aura-accent shadow-[0_0_0_4px_rgba(43,134,255,0.12)] animate-pulse"></span>
        <span class="font-display font-medium">
          {viewState === 'loading' ? 'Connecting' : 'Translating'}
        </span>
        <button
          id="cancel-translate-btn"
          class="ml-0.5 flex h-5 w-5 items-center justify-center rounded-full text-aura-text-muted transition-colors duration-200 hover:bg-white/70 hover:text-aura-error"
          onclick={() => oncancel?.()}
          aria-label="Cancel translation"
          data-testid="cancel-translate-button"
          type="button"
        >
          <svg class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    {/if}
  </div>

  <div class="shrink-0 px-5 pb-3">
    <LanguageSelector
      {sourceLang}
      {targetLang}
      onchange={onLanguageChange}
    />
  </div>

  {#if sourceText}
    <div class="shrink-0 px-5 pb-3">
      <div class="rounded-2xl bg-white/48 px-4 py-3 text-sm text-aura-text-dim ring-1 ring-white/60" data-testid="source-context">
        <p class="mb-1 text-[10px] font-display font-semibold uppercase tracking-[0.2em] text-aura-text-muted">
          Source
        </p>
        <p class="max-h-11 overflow-hidden leading-relaxed select-text">
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
          Copy text and press {hotkeyLabel}
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
      </div>

      <div class="min-h-0 flex-1 overflow-y-auto px-4 py-4">
        {#if viewState === 'idle'}
          <div class="flex h-full items-center justify-center">
            <p class="max-w-[240px] text-center text-sm leading-relaxed text-aura-text-dim" data-testid="idle-hint">
              Copy text, press your hotkey, and Aura will keep the translation close without taking over your screen.
            </p>
          </div>
        {:else if viewState === 'loading'}
          <div data-testid="loading-state">
            <SkeletonLoader />
          </div>
        {:else if viewState === 'streaming' || viewState === 'result'}
          <p
            class="whitespace-pre-wrap break-words text-[15px] leading-7 text-aura-text select-text"
            data-testid="translation-output"
            style="animation: fade-in-up 0.3s ease-out both;"
          >
            {translatedText}{#if viewState === 'streaming'}<span class="ml-0.5 inline-block h-4 w-0.5 animate-pulse align-text-bottom bg-aura-accent"></span>{/if}
          </p>
        {:else if viewState === 'error'}
          <div class="flex h-full flex-col items-center justify-center gap-3 text-center" data-testid="error-state">
            <div class="flex h-10 w-10 items-center justify-center rounded-full bg-[#fce8eb] text-aura-error">
              <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8">
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
              </svg>
            </div>
            <p class="max-w-[280px] text-sm leading-relaxed text-aura-text-dim">
              {errorMessage || 'Translation failed'}
            </p>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>
