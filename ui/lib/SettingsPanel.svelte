<script lang="ts">
  /**
   * SettingsPanel - Configuration UI for API key, model, provider, and hotkey.
   * Glassmorphic styling consistent with the translation popup.
   */
  import { Spring } from 'svelte/motion';
  import { invoke } from '@tauri-apps/api/core';
  import { shouldShowPlaintextApiKeyWarning } from './notifications';

  type Provider = 'deepseek' | 'openrouter' | 'ollama';

  const PROVIDER_OPTIONS: { value: Provider; label: string }[] = [
    { value: 'deepseek', label: 'DeepSeek (api.deepseek.com)' },
    { value: 'openrouter', label: 'OpenRouter (openrouter.ai)' },
    { value: 'ollama', label: 'Ollama (local, no auth)' },
  ];

  type ProviderDefaults = {
    base_url: string;
    models: string[];
  };

  type AppConfig = {
    api_key: string;
    model: string;
    source_lang: string;
    target_lang: string;
    hotkey: string;
    provider: Provider;
    api_base_url: string;
    available_models: string[];
  };

  type Props = {
    visible: boolean;
    onclose: () => void;
    hotkeyConflictMessage?: string;
  };

  let { visible, onclose, hotkeyConflictMessage = '' }: Props = $props();

  let config: AppConfig = $state({
    api_key: '',
    model: 'deepseek-chat',
    source_lang: 'auto',
    target_lang: 'Chinese',
    hotkey: 'CmdOrCtrl+T',
    provider: 'deepseek',
    api_base_url: 'https://api.deepseek.com',
    available_models: ['deepseek-chat', 'deepseek-reasoner'],
  });

  let showApiKey = $state(false);
  let saving = $state(false);
  let saveMessage = $state('');
  let saveErrorMessage = $state('');
  let panelErrorMessage = $state('');
  let hotkeyInputMessage = $state('');
  let isCapturingHotkey = $state(false);

  const saveScale = new Spring(1, { stiffness: 0.4, damping: 0.5 });

  $effect(() => {
    if (visible) {
      void loadConfig();
    }
  });

  async function loadConfig() {
    panelErrorMessage = '';
    hotkeyInputMessage = '';
    try {
      const loaded = await invoke<AppConfig>('get_config');
      config = loaded;
    } catch (e) {
      panelErrorMessage = 'Failed to load settings from the local Aura config.';
      console.error('Failed to load config:', { error: e });
    }
  }

  async function handleProviderChange(newProvider: Provider) {
    config.provider = newProvider;
    panelErrorMessage = '';
    saveErrorMessage = '';
    try {
      const defaults = await invoke<ProviderDefaults>('get_provider_defaults', {
        provider: newProvider,
      });
      config.api_base_url = defaults.base_url;
      config.available_models = defaults.models;
      if (!defaults.models.includes(config.model)) {
        config.model = defaults.models[0];
      }
    } catch (e) {
      panelErrorMessage = 'Failed to load provider defaults from the desktop backend.';
      console.error('Failed to fetch provider defaults:', e);
    }
  }

  function handleHotkeyKeydown(e: KeyboardEvent) {
    e.preventDefault();
    e.stopImmediatePropagation();

    if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) return;

    const parts: string[] = [];
    if (e.ctrlKey || e.metaKey) parts.push('CmdOrCtrl');
    if (e.altKey) parts.push('Alt');
    if (e.shiftKey) parts.push('Shift');

    if (parts.length === 0) {
      hotkeyInputMessage = 'Include at least one modifier key.';
      return;
    }

    const key = e.key.length === 1 && /^[a-z0-9]$/i.test(e.key) ? e.key.toUpperCase() : null;
    if (!key) {
      hotkeyInputMessage = 'Use a letter or digit key with at least one modifier.';
      return;
    }

    parts.push(key);
    config.hotkey = parts.join('+');
    hotkeyInputMessage = '';
  }

  async function saveConfig() {
    saving = true;
    saveMessage = '';
    saveErrorMessage = '';
    panelErrorMessage = '';
    saveScale.target = 0.95;

    try {
      await invoke('save_config', { config });
      saveMessage = 'Settings saved';
      hotkeyInputMessage = '';
      saveScale.target = 1.05;
      setTimeout(() => {
        saveScale.target = 1;
        saveMessage = '';
      }, 1200);
    } catch (e) {
      const message = String(e ?? '');
      saveErrorMessage = message.startsWith('Hotkey save rejected:')
        ? 'Could not save settings. Fix the hotkey and try again.'
        : 'Failed to save settings to the local Aura config.';
      saveScale.target = 1;
      console.error('Failed to save config:', { error: e });
    }

    saving = false;
  }
</script>

{#if visible}
  <div
    class="absolute inset-0 z-50 flex flex-col overflow-hidden rounded-[18px] border border-aura-border"
    style="
      background: rgba(12, 12, 20, 0.92);
      backdrop-filter: blur(32px) saturate(1.5);
      -webkit-backdrop-filter: blur(32px) saturate(1.5);
      animation: fade-in-up 0.25s ease-out both;
    "
  >
    <div class="flex items-center justify-between border-b border-aura-border/50 px-4 py-3" data-tauri-drag-region>
      <h2 class="text-sm font-display font-semibold tracking-wide text-aura-text" data-tauri-drag-region>
        Settings
      </h2>
      <button
        class="flex h-6 w-6 items-center justify-center rounded-md hover:bg-aura-glass-hover transition-colors duration-150"
        onclick={onclose}
        aria-label="Close settings"
      >
        <svg class="h-4 w-4 text-aura-text-dim" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>

    <div class="flex-1 space-y-4 overflow-y-auto px-4 py-3">
      {#if panelErrorMessage}
        <div class="rounded-lg border border-aura-error/45 bg-aura-error/8 px-3 py-2 text-xs text-aura-error">
          {panelErrorMessage}
        </div>
      {/if}

      <div class="space-y-1.5">
        <label class="text-xs font-display font-medium uppercase tracking-wider text-aura-text-dim" for="provider">
          Provider
        </label>
        <select
          id="provider"
          value={config.provider}
          onchange={(e) => handleProviderChange((e.currentTarget as HTMLSelectElement).value as Provider)}
          class="w-full cursor-pointer appearance-none rounded-lg border border-aura-border bg-aura-glass px-3 py-2 text-sm text-aura-text transition-all duration-200 hover:border-aura-border-accent focus:border-aura-accent focus:outline-none"
        >
          {#each PROVIDER_OPTIONS as opt}
            <option value={opt.value} class="bg-[#1a1a2e]">{opt.label}</option>
          {/each}
        </select>
      </div>

      {#if shouldShowPlaintextApiKeyWarning(config.provider)}
        <div
          class="rounded-lg border border-[#d6a743]/35 bg-[#2a2216]/88 px-3 py-2"
          data-testid="plaintext-api-key-warning"
        >
          <p class="text-[11px] leading-relaxed text-[#e7c980]">
            API keys are stored in plaintext in the local Aura config on this machine. Use a trial or low-permission key when possible.
          </p>
        </div>
      {/if}

      {#if config.provider !== 'ollama'}
        <div class="space-y-1.5">
          <label class="text-xs font-display font-medium uppercase tracking-wider text-aura-text-dim" for="api-key">
            API Key
          </label>
          <div class="relative">
            <input
              id="api-key"
              type={showApiKey ? 'text' : 'password'}
              bind:value={config.api_key}
              placeholder="sk-..."
              class="w-full rounded-lg border border-aura-border bg-aura-glass px-3 py-2 text-sm text-aura-text placeholder:text-aura-text-muted transition-all duration-200 hover:border-aura-border-accent focus:border-aura-accent focus:outline-none focus:ring-1 focus:ring-aura-accent/30"
            />
            <button
              class="absolute right-2 top-1/2 -translate-y-1/2 rounded p-1 hover:bg-aura-glass-hover transition-colors"
              onclick={() => (showApiKey = !showApiKey)}
              type="button"
              aria-label={showApiKey ? 'Hide API key' : 'Show API key'}
            >
              <svg class="h-4 w-4 text-aura-text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                {#if showApiKey}
                  <path stroke-linecap="round" stroke-linejoin="round" d="M3.98 8.223A10.477 10.477 0 0 1.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0 1 12 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 0 1-4.293 5.774M6.228 6.228 3 3m3.228 3.228 3.65 3.65m7.894 7.894L21 21m-3.228-3.228-3.65-3.65m0 0a3 3 0 1 0-4.243-4.243m4.242 4.242L9.88 9.88" />
                {:else}
                  <path stroke-linecap="round" stroke-linejoin="round" d="M2.036 12.322a1.012 1.012 0 0 1 0-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178z" />
                  <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0z" />
                {/if}
              </svg>
            </button>
          </div>
        </div>
      {/if}

      <div class="space-y-1.5">
        <label class="text-xs font-display font-medium uppercase tracking-wider text-aura-text-dim" for="model">
          Model
        </label>
        <select
          id="model"
          bind:value={config.model}
          class="w-full cursor-pointer appearance-none rounded-lg border border-aura-border bg-aura-glass px-3 py-2 text-sm text-aura-text transition-all duration-200 hover:border-aura-border-accent focus:border-aura-accent focus:outline-none"
        >
          {#each config.available_models as m}
            <option value={m} class="bg-[#1a1a2e]">{m}</option>
          {/each}
        </select>
      </div>

      <div class="space-y-1.5">
        <label class="text-xs font-display font-medium uppercase tracking-wider text-aura-text-dim" for="hotkey-capture">
          Hotkey
        </label>
        <div class="relative">
          <input
            id="hotkey-capture"
            type="text"
            value={config.hotkey}
            placeholder="Press a key combination"
            readonly
            onfocus={() => (isCapturingHotkey = true)}
            onblur={() => (isCapturingHotkey = false)}
            onkeydown={handleHotkeyKeydown}
            class={`w-full cursor-pointer select-none caret-transparent rounded-lg border bg-aura-glass px-3 py-2 text-sm font-medium text-aura-text transition-all duration-200 ${
              isCapturingHotkey
                ? 'border-aura-accent ring-1 ring-aura-accent/30'
                : 'border-aura-border hover:border-aura-border-accent'
            }`}
          />
          {#if isCapturingHotkey}
            <span class="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-aura-accent animate-pulse">
              recording...
            </span>
          {:else}
            <span class="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-aura-text-muted">
              click to change
            </span>
          {/if}
        </div>
        <p class="pl-0.5 text-xs text-aura-text-muted">
          Hotkeys must include at least one modifier and a letter or digit key.
        </p>

        {#if hotkeyInputMessage}
          <p class="pl-0.5 text-xs text-[#e7c980]" data-testid="hotkey-input-message">
            {hotkeyInputMessage}
          </p>
        {/if}

        {#if hotkeyConflictMessage}
          <div
            class="rounded-lg border border-[#d6a743]/35 bg-[#2a2216]/88 px-3 py-2 text-xs leading-relaxed text-[#e7c980]"
            data-testid="hotkey-conflict-inline"
          >
            {hotkeyConflictMessage}
          </div>
        {/if}
      </div>
    </div>

    <div class="border-t border-aura-border/50 px-4 py-3">
      {#if saveErrorMessage}
        <p class="mb-2 text-xs text-aura-error" data-testid="save-error-message">{saveErrorMessage}</p>
      {:else if saveMessage}
        <p class="mb-2 text-xs text-aura-success">{saveMessage}</p>
      {/if}

      <button
        class="w-full rounded-lg bg-aura-accent py-2 text-sm font-display font-medium text-white transition-all duration-200 hover:brightness-110 active:brightness-90 disabled:cursor-not-allowed disabled:opacity-50"
        style:transform="scale({saveScale.current})"
        onclick={saveConfig}
        disabled={saving}
      >
        {#if saving}
          Saving...
        {:else}
          Save Settings
        {/if}
      </button>
    </div>
  </div>
{/if}
