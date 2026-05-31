<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';
  import SettingsWindowView from '$lib/SettingsWindowView.svelte';
  import TranslationWindowView from '$lib/TranslationWindowView.svelte';

  let windowLabel = $state<'translation' | 'settings' | ''>('');

  onMount(() => {
    windowLabel = getCurrentWindow().label as 'translation' | 'settings';
  });
</script>

{#if windowLabel === 'settings'}
  <SettingsWindowView />
{:else}
  <TranslationWindowView />
{/if}
