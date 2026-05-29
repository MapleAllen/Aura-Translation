<script lang="ts">
  /**
   * LanguageSelector - Compact source/target language toolbar.
   */
  import { Spring } from 'svelte/motion';

  type Props = {
    sourceLang: string;
    targetLang: string;
    onchange: (source: string, target: string) => void;
  };

  let { sourceLang, targetLang, onchange }: Props = $props();

  const LANGUAGES = [
    { code: 'auto', label: 'Auto detect' },
    { code: 'Chinese', label: 'Chinese' },
    { code: 'English', label: 'English' },
    { code: 'Japanese', label: 'Japanese' },
    { code: 'Korean', label: 'Korean' },
    { code: 'French', label: 'French' },
    { code: 'German', label: 'German' },
    { code: 'Spanish', label: 'Spanish' },
    { code: 'Russian', label: 'Russian' },
    { code: 'Arabic', label: 'Arabic' },
    { code: 'Portuguese', label: 'Portuguese' },
  ];

  const swapRotation = new Spring(0, { stiffness: 0.3, damping: 0.65 });
  let rotationCount = $state(0);

  function swap() {
    if (sourceLang === 'auto') return;
    rotationCount += 1;
    swapRotation.target = rotationCount * 180;
    const temp = sourceLang;
    onchange(targetLang, temp);
  }
</script>

<div class="flex items-center gap-2 rounded-lg border border-aura-border bg-white px-2 py-2">
  <label class="min-w-0 flex-1 rounded-md border border-aura-border/70 bg-aura-surface-soft px-3 py-2">
    <span class="mb-1 block text-[10px] font-display font-semibold uppercase tracking-[0.16em] text-aura-text-muted">
      From
    </span>
    <select
      class="w-full cursor-pointer appearance-none bg-transparent text-sm font-medium text-aura-text outline-none"
      value={sourceLang}
      onchange={(e) => onchange((e.target as HTMLSelectElement).value, targetLang)}
    >
      {#each LANGUAGES as lang}
        <option value={lang.code}>
          {lang.label}
        </option>
      {/each}
    </select>
  </label>

  <button
    class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md border border-aura-border bg-white text-aura-accent transition-all duration-200 hover:border-aura-border-accent hover:bg-aura-accent-soft active:scale-95 disabled:opacity-35 disabled:hover:border-aura-border disabled:hover:bg-white"
    onclick={swap}
    disabled={sourceLang === 'auto'}
    title="Swap languages"
    type="button"
  >
    <svg
      class="h-4 w-4"
      style:transform="rotate({swapRotation.current}deg)"
      fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"
    >
      <path stroke-linecap="round" stroke-linejoin="round" d="M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" />
    </svg>
  </button>

  <label class="min-w-0 flex-1 rounded-md border border-aura-border/70 bg-aura-surface-soft px-3 py-2">
    <span class="mb-1 block text-[10px] font-display font-semibold uppercase tracking-[0.16em] text-aura-text-muted">
      To
    </span>
    <select
      class="w-full cursor-pointer appearance-none bg-transparent text-sm font-medium text-aura-text outline-none"
      value={targetLang}
      onchange={(e) => onchange(sourceLang, (e.target as HTMLSelectElement).value)}
    >
      {#each LANGUAGES.filter((language) => language.code !== 'auto') as lang}
        <option value={lang.code}>
          {lang.label}
        </option>
      {/each}
    </select>
  </label>
</div>
