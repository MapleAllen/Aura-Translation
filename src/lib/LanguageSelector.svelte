<script lang="ts">
  /**
   * LanguageSelector — Bidirectional language pair selector with swap button.
   */
  import { Spring } from 'svelte/motion';

  type Props = {
    sourceLang: string;
    targetLang: string;
    onchange: (source: string, target: string) => void;
  };

  let { sourceLang, targetLang, onchange }: Props = $props();

  const LANGUAGES = [
    { code: 'auto', label: 'Auto Detect', flag: '🌐' },
    { code: 'Chinese', label: '中文', flag: '🇨🇳' },
    { code: 'English', label: 'English', flag: '🇬🇧' },
    { code: 'Japanese', label: '日本語', flag: '🇯🇵' },
    { code: 'Korean', label: '한국어', flag: '🇰🇷' },
    { code: 'French', label: 'Français', flag: '🇫🇷' },
    { code: 'German', label: 'Deutsch', flag: '🇩🇪' },
    { code: 'Spanish', label: 'Español', flag: '🇪🇸' },
    { code: 'Russian', label: 'Русский', flag: '🇷🇺' },
    { code: 'Arabic', label: 'العربية', flag: '🇸🇦' },
    { code: 'Portuguese', label: 'Português', flag: '🇵🇹' },
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

<div class="flex items-center gap-2">
  <!-- Source Language -->
  <select
    class="flex-1 bg-aura-glass border border-aura-border rounded-lg px-3 py-1.5 text-sm
           text-aura-text font-body appearance-none cursor-pointer
           hover:bg-aura-glass-hover hover:border-aura-border-accent
           focus:outline-none focus:border-aura-accent
           transition-all duration-200"
    value={sourceLang}
    onchange={(e) => onchange((e.target as HTMLSelectElement).value, targetLang)}
  >
    {#each LANGUAGES as lang}
      <option value={lang.code} class="bg-[#1a1a2e] text-aura-text">
        {lang.flag} {lang.label}
      </option>
    {/each}
  </select>

  <!-- Swap Button -->
  <button
    class="flex items-center justify-center w-8 h-8 rounded-full
           bg-aura-glass border border-aura-border
           hover:bg-aura-accent-soft hover:border-aura-border-accent
           active:scale-90 transition-all duration-200
           disabled:opacity-30 disabled:cursor-not-allowed"
    onclick={swap}
    disabled={sourceLang === 'auto'}
    title="Swap languages"
  >
    <svg
      class="w-4 h-4 text-aura-accent"
      style:transform="rotate({swapRotation.current}deg)"
      fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"
    >
      <path stroke-linecap="round" stroke-linejoin="round" d="M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" />
    </svg>
  </button>

  <!-- Target Language -->
  <select
    class="flex-1 bg-aura-glass border border-aura-border rounded-lg px-3 py-1.5 text-sm
           text-aura-text font-body appearance-none cursor-pointer
           hover:bg-aura-glass-hover hover:border-aura-border-accent
           focus:outline-none focus:border-aura-accent
           transition-all duration-200"
    value={targetLang}
    onchange={(e) => onchange(sourceLang, (e.target as HTMLSelectElement).value)}
  >
    {#each LANGUAGES.filter(l => l.code !== 'auto') as lang}
      <option value={lang.code} class="bg-[#1a1a2e] text-aura-text">
        {lang.flag} {lang.label}
      </option>
    {/each}
  </select>
</div>
