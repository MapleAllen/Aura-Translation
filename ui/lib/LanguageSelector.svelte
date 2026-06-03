<script lang="ts">
  import { Spring } from 'svelte/motion';
  import { LANGUAGES } from './languages';

  type Props = {
    sourceLang: string;
    targetLang: string;
    onchange: (source: string, target: string) => void;
  };

  let { sourceLang, targetLang, onchange }: Props = $props();

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

<div class="grid gap-2 md:grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)]">
  <label class="space-y-1.5">
    <span class="aura-section-title">源语言</span>
    <select
      class="aura-console-select"
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
    class="aura-console-icon-button self-end"
    onclick={swap}
    disabled={sourceLang === 'auto'}
    title="交换语言"
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

  <label class="space-y-1.5">
    <span class="aura-section-title">目标语言</span>
    <select
      class="aura-console-select"
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
