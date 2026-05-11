<script lang="ts">
  /**
   * SettingsPanel — Configuration UI for API key, model, and hotkey.
   * Glassmorphic styling consistent with the translation popup.
   */
  import { Spring } from 'svelte/motion';
  import { invoke } from '@tauri-apps/api/core';

  type AppConfig = {
    api_key: string;
    model: string;
    source_lang: string;
    target_lang: string;
    hotkey: string;
  };

  type Props = {
    visible: boolean;
    onclose: () => void;
  };

  let { visible, onclose }: Props = $props();

  let config: AppConfig = $state({
    api_key: '',
    model: 'deepseek-v4-flash',
    source_lang: 'auto',
    target_lang: 'Chinese',
    hotkey: 'CmdOrCtrl+T',
  });

  let showApiKey = $state(false);
  let saving = $state(false);
  let saveMessage = $state('');

  const saveScale = new Spring(1, { stiffness: 0.4, damping: 0.5 });

  // Load config when settings become visible
  $effect(() => {
    if (visible) {
      loadConfig();
    }
  });

  async function loadConfig() {
    try {
      const loaded = await invoke<AppConfig>('get_config');
      config = loaded;
    } catch (e) {
      console.error('Failed to load config:', e);
    }
  }

  async function saveConfig() {
    saving = true;
    saveScale.target = 0.95;
    try {
      await invoke('save_config', { config });
      saveMessage = 'Saved!';
      saveScale.target = 1.05;
      setTimeout(() => {
        saveScale.target = 1;
        saveMessage = '';
      }, 1200);
    } catch (e) {
      saveMessage = 'Error saving';
      saveScale.target = 1;
    }
    saving = false;
  }
</script>

{#if visible}
  <div
    class="absolute inset-0 z-50 flex flex-col rounded-[18px] overflow-hidden
           border border-aura-border"
    style="
      background: rgba(12, 12, 20, 0.92);
      backdrop-filter: blur(32px) saturate(1.5);
      -webkit-backdrop-filter: blur(32px) saturate(1.5);
      animation: fade-in-up 0.25s ease-out both;
    "
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-aura-border/50" data-tauri-drag-region>
      <h2 class="text-sm font-display font-semibold text-aura-text tracking-wide" data-tauri-drag-region>
        Settings
      </h2>
      <button
        class="flex items-center justify-center w-6 h-6 rounded-md
               hover:bg-aura-glass-hover transition-colors duration-150"
        onclick={onclose}
      >
        <svg class="w-4 h-4 text-aura-text-dim" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>

    <!-- Settings Form -->
    <div class="flex-1 overflow-y-auto px-4 py-3 space-y-4">
      <!-- API Key -->
      <div class="space-y-1.5">
        <label class="text-xs font-display font-medium text-aura-text-dim uppercase tracking-wider" for="api-key">
          DeepSeek API Key
        </label>
        <div class="relative">
          <input
            id="api-key"
            type={showApiKey ? 'text' : 'password'}
            bind:value={config.api_key}
            placeholder="sk-..."
            class="w-full bg-aura-glass border border-aura-border rounded-lg px-3 py-2
                   text-sm text-aura-text font-body placeholder:text-aura-text-muted
                   hover:border-aura-border-accent
                   focus:outline-none focus:border-aura-accent focus:ring-1 focus:ring-aura-accent/30
                   transition-all duration-200"
          />
          <button
            class="absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded
                   hover:bg-aura-glass-hover transition-colors"
            onclick={() => showApiKey = !showApiKey}
            type="button"
          >
            <svg class="w-4 h-4 text-aura-text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
              {#if showApiKey}
                <path stroke-linecap="round" stroke-linejoin="round" d="M3.98 8.223A10.477 10.477 0 001.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0112 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 01-4.293 5.774M6.228 6.228L3 3m3.228 3.228l3.65 3.65m7.894 7.894L21 21m-3.228-3.228l-3.65-3.65m0 0a3 3 0 10-4.243-4.243m4.242 4.242L9.88 9.88" />
              {:else}
                <path stroke-linecap="round" stroke-linejoin="round" d="M2.036 12.322a1.012 1.012 0 010-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178z" />
                <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              {/if}
            </svg>
          </button>
        </div>
      </div>

      <!-- Model Selection -->
      <div class="space-y-1.5">
        <label class="text-xs font-display font-medium text-aura-text-dim uppercase tracking-wider" for="model">
          Model
        </label>
        <select
          id="model"
          bind:value={config.model}
          class="w-full bg-aura-glass border border-aura-border rounded-lg px-3 py-2
                 text-sm text-aura-text font-body appearance-none cursor-pointer
                 hover:border-aura-border-accent
                 focus:outline-none focus:border-aura-accent
                 transition-all duration-200"
        >
          <option value="deepseek-v4-flash" class="bg-[#1a1a2e]">DeepSeek V4 Flash (Fast)</option>
          <option value="deepseek-v4-pro" class="bg-[#1a1a2e]">DeepSeek V4 Pro (Quality)</option>
        </select>
      </div>

      <!-- Hotkey Info -->
      <div class="space-y-1.5">
        <label class="text-xs font-display font-medium text-aura-text-dim uppercase tracking-wider">
          Hotkey
        </label>
        <div class="flex items-center gap-2 px-3 py-2 bg-aura-glass border border-aura-border rounded-lg">
          <kbd class="px-2 py-0.5 text-xs font-display font-medium text-aura-accent bg-aura-accent-soft rounded border border-aura-border-accent">
            Ctrl
          </kbd>
          <span class="text-aura-text-muted text-xs">+</span>
          <kbd class="px-2 py-0.5 text-xs font-display font-medium text-aura-accent bg-aura-accent-soft rounded border border-aura-border-accent">
            T
          </kbd>
          <span class="text-xs text-aura-text-muted ml-auto">Copy text first, then press</span>
        </div>
      </div>
    </div>

    <!-- Save Button -->
    <div class="px-4 py-3 border-t border-aura-border/50">
      <button
        class="w-full py-2 rounded-lg text-sm font-display font-medium
               bg-aura-accent text-white
               hover:brightness-110 active:brightness-90
               disabled:opacity-50 disabled:cursor-not-allowed
               transition-all duration-200"
        style:transform="scale({saveScale.current})"
        onclick={saveConfig}
        disabled={saving}
      >
        {#if saveMessage}
          {saveMessage}
        {:else if saving}
          Saving...
        {:else}
          Save Settings
        {/if}
      </button>
    </div>
  </div>
{/if}
