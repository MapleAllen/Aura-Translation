<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';
  import SettingsWindowView from '$lib/SettingsWindowView.svelte';
  import TranslationWindowView from '$lib/TranslationWindowView.svelte';

  type AuraWindowLabel = 'translation' | 'settings';

  let windowLabel = $state<AuraWindowLabel | null>(null);

  function resolveWindowLabel(label: string): AuraWindowLabel {
    return label === 'settings' ? 'settings' : 'translation';
  }

  onMount(() => {
    try {
      windowLabel = resolveWindowLabel(getCurrentWindow().label);
    } catch (e) {
      console.error('Failed to resolve Aura window label:', e);
      windowLabel = 'translation';
    }
  });
</script>

{#if windowLabel === 'settings'}
  <SettingsWindowView />
{:else if windowLabel === 'translation'}
  <TranslationWindowView />
{/if}
