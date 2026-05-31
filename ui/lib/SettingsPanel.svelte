<script lang="ts">
  /**
   * SettingsPanel - Configuration UI for API key, model, provider, and hotkey.
   */
  import { Spring } from 'svelte/motion';
  import { invoke } from '@tauri-apps/api/core';
  import LanguageSelector from './LanguageSelector.svelte';
  import SetupStatusCard from './SetupStatusCard.svelte';
  import type { AppConfig, Provider } from './appConfig';
  import { cloneAppConfig, createDefaultAppConfig } from './appConfig';
  import { shouldShowPlaintextApiKeyWarning } from './notifications';
  import type { ProviderProbeResult, RuntimeStatus } from './runtimeStatus';

  const PROVIDER_OPTIONS: { value: Provider; label: string }[] = [
    { value: 'deepseek', label: 'DeepSeek (api.deepseek.com)' },
    { value: 'openrouter', label: 'OpenRouter (openrouter.ai)' },
    { value: 'ollama', label: 'Ollama (local, no auth)' },
  ];

  type ProviderDefaults = {
    base_url: string;
    models: string[];
  };

  type Props = {
    visible: boolean;
    onclose: () => void;
    onsaved?: (config: AppConfig) => void;
    hotkeyConflictMessage?: string;
  };

  let { visible, onclose, onsaved, hotkeyConflictMessage = '' }: Props = $props();

  let config: AppConfig = $state(createDefaultAppConfig());

  let showApiKey = $state(false);
  let saving = $state(false);
  let testingProvider = $state(false);
  let saveMessage = $state('');
  let saveErrorMessage = $state('');
  let panelErrorMessage = $state('');
  let hotkeyInputMessage = $state('');
  let isCapturingHotkey = $state(false);
  let runtimeStatus = $state<RuntimeStatus | null>(null);
  let probeResult = $state<ProviderProbeResult | null>(null);

  const saveScale = new Spring(1, { stiffness: 0.4, damping: 0.5 });

  $effect(() => {
    if (visible) {
      void loadConfig();
    }
  });

  async function loadConfig() {
    panelErrorMessage = '';
    hotkeyInputMessage = '';
    saveMessage = '';
    saveErrorMessage = '';
    probeResult = null;
    try {
      const [loaded, nextRuntimeStatus] = await Promise.all([
        invoke<AppConfig>('get_config'),
        invoke<RuntimeStatus>('get_runtime_status'),
      ]);
      config = cloneAppConfig(loaded);
      runtimeStatus = nextRuntimeStatus;
    } catch (e) {
      panelErrorMessage = 'Failed to load settings from the local Aura config.';
      console.error('Failed to load config:', { error: e });
    }
  }

  async function handleProviderChange(newProvider: Provider) {
    config.provider = newProvider;
    panelErrorMessage = '';
    saveErrorMessage = '';
    probeResult = null;
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

  async function refreshRuntimeStatus() {
    try {
      runtimeStatus = await invoke<RuntimeStatus>('get_runtime_status');
    } catch (e) {
      panelErrorMessage = 'Failed to refresh Aura readiness from the desktop backend.';
      console.error('Failed to refresh runtime status:', { error: e });
    }
  }

  async function handleProviderProbe() {
    testingProvider = true;
    probeResult = null;

    try {
      probeResult = await invoke<ProviderProbeResult>('probe_provider', {
        config: cloneAppConfig(config),
      });
    } catch (e) {
      probeResult = {
        ok: false,
        message: 'Failed to run the provider test from the desktop backend.',
      };
      console.error('Failed to probe provider:', { error: e });
    }

    testingProvider = false;
  }

  async function saveConfig() {
    saving = true;
    saveMessage = '';
    saveErrorMessage = '';
    panelErrorMessage = '';
    saveScale.target = 0.97;

    try {
      await invoke('save_config', { config: cloneAppConfig(config) });
      await refreshRuntimeStatus();
      saveMessage = 'Settings saved';
      onsaved?.(cloneAppConfig(config));
      hotkeyInputMessage = '';
      probeResult = null;
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
    class="absolute inset-0 z-50 flex flex-col overflow-hidden rounded-[10px] border border-aura-border bg-[rgba(248,250,252,0.96)] text-aura-text shadow-[0_18px_40px_rgba(89,104,129,0.16)]"
    style="
      backdrop-filter: blur(18px) saturate(1.02);
      -webkit-backdrop-filter: blur(18px) saturate(1.02);
      animation: fade-in-up 0.25s ease-out both;
    "
  >
    <div class="flex items-start justify-between border-b border-aura-border px-5 py-4" data-tauri-drag-region>
      <div data-tauri-drag-region>
        <h2 class="text-sm font-display font-semibold tracking-[0.08em] text-aura-text uppercase" data-tauri-drag-region>
          Settings
        </h2>
        <p class="mt-1 text-xs leading-relaxed text-aura-text-dim" data-tauri-drag-region>
          Configure languages, trigger behavior, provider access, and window memory.
        </p>
      </div>
      <button
        class="flex h-8 w-8 items-center justify-center rounded-md border border-aura-border bg-white/84 text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-text"
        onclick={onclose}
        aria-label="Close settings"
        type="button"
      >
        <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>

    <div class="flex-1 space-y-4 overflow-y-auto px-5 py-4">
      {#if panelErrorMessage}
        <div class="rounded-lg border border-aura-error/25 bg-[#fff4f6] px-4 py-3 text-sm leading-relaxed text-aura-error">
          {panelErrorMessage}
        </div>
      {/if}

      <SetupStatusCard
        status={runtimeStatus}
        testing={testingProvider}
        {probeResult}
        ontest={handleProviderProbe}
      />

      <section class="space-y-3 rounded-lg border border-aura-border bg-white/80 px-4 py-4">
        <div>
          <p class="text-[10px] font-display font-semibold uppercase tracking-[0.22em] text-aura-text-muted">
            Translation
          </p>
          <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
            Choose the default language pair used for every new translation request.
          </p>
        </div>
        <LanguageSelector
          sourceLang={config.source_lang}
          targetLang={config.target_lang}
          onchange={(source, target) => {
            config.source_lang = source;
            config.target_lang = target;
          }}
        />
      </section>

      <section class="space-y-3 rounded-lg border border-aura-border bg-white/80 px-4 py-4">
        <div class="flex items-center justify-between gap-4">
          <div>
            <p class="text-[10px] font-display font-semibold uppercase tracking-[0.22em] text-aura-text-muted">
              Aura mode
            </p>
            <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
              On Windows, automatically translate new clipboard text as soon as it changes.
            </p>
          </div>
          <button
            class={`flex h-7 w-12 items-center rounded-full border px-1 transition-all duration-150 ${
              config.aura_mode_enabled
                ? 'border-aura-accent bg-aura-accent text-white'
                : 'border-aura-border bg-white text-aura-text-muted'
            }`}
            role="switch"
            aria-checked={config.aura_mode_enabled}
            aria-label="Aura mode"
            type="button"
            onclick={() => (config.aura_mode_enabled = !config.aura_mode_enabled)}
          >
            <span
              class={`h-5 w-5 rounded-full bg-current transition-transform duration-150 ${
                config.aura_mode_enabled ? 'translate-x-5' : 'translate-x-0'
              }`}
              style={config.aura_mode_enabled ? 'color: white;' : 'color: rgba(138, 150, 166, 0.75);'}
            ></span>
          </button>
        </div>

        <div class="rounded-md border border-aura-border/80 bg-aura-surface-soft px-3 py-2 text-xs leading-relaxed text-aura-text-dim">
          With Aura mode off, Aura keeps the existing copy-then-hotkey workflow.
        </div>
      </section>

      <section class="space-y-3 rounded-lg border border-aura-border bg-white/80 px-4 py-4">
        <div class="flex items-center justify-between gap-4">
          <div>
            <p class="text-[10px] font-display font-semibold uppercase tracking-[0.22em] text-aura-text-muted">
              Sensitive clipboard guard
            </p>
            <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
              Skip likely passwords, tokens, and long credential-like strings before Aura mode sends anything to a provider.
            </p>
          </div>
          <button
            class={`flex h-7 w-12 items-center rounded-full border px-1 transition-all duration-150 ${
              config.aura_guard_enabled
                ? 'border-aura-accent bg-aura-accent text-white'
                : 'border-aura-border bg-white text-aura-text-muted'
            }`}
            role="switch"
            aria-checked={config.aura_guard_enabled}
            aria-label="Sensitive clipboard guard"
            type="button"
            onclick={() => (config.aura_guard_enabled = !config.aura_guard_enabled)}
          >
            <span
              class={`h-5 w-5 rounded-full bg-current transition-transform duration-150 ${
                config.aura_guard_enabled ? 'translate-x-5' : 'translate-x-0'
              }`}
              style={config.aura_guard_enabled ? 'color: white;' : 'color: rgba(138, 150, 166, 0.75);'}
            ></span>
          </button>
        </div>

        <div class="rounded-md border border-aura-border/80 bg-aura-surface-soft px-3 py-2 text-xs leading-relaxed text-aura-text-dim">
          This guard only affects automatic Aura mode clipboard translations. Manual hotkey translations still use the text you explicitly trigger.
        </div>
      </section>

      <section class="space-y-3 rounded-lg border border-aura-border bg-white/80 px-4 py-4">
        <div class="flex items-center justify-between gap-4">
          <div>
            <p class="text-[10px] font-display font-semibold uppercase tracking-[0.22em] text-aura-text-muted">
              Window behavior
            </p>
            <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
              Pin the translator when you want it to stay visible, movable, and always on top.
            </p>
          </div>
          <button
            class={`flex h-7 w-12 items-center rounded-full border px-1 transition-all duration-150 ${
              config.window_pinned
                ? 'border-aura-accent bg-aura-accent text-white'
                : 'border-aura-border bg-white text-aura-text-muted'
            }`}
            role="switch"
            aria-checked={config.window_pinned}
            aria-label="Pin window"
            type="button"
            onclick={() => (config.window_pinned = !config.window_pinned)}
          >
            <span
              class={`h-5 w-5 rounded-full bg-current transition-transform duration-150 ${
                config.window_pinned ? 'translate-x-5' : 'translate-x-0'
              }`}
              style={config.window_pinned ? 'color: white;' : 'color: rgba(138, 150, 166, 0.75);'}
            ></span>
          </button>
        </div>

        <div class="grid gap-2 text-xs text-aura-text-dim sm:grid-cols-2">
          <div class="rounded-md border border-aura-border/80 bg-aura-surface-soft px-3 py-2">
            Unpinned: recalls near the cursor and hides on blur.
          </div>
          <div class="rounded-md border border-aura-border/80 bg-aura-surface-soft px-3 py-2">
            Pinned: remembers its last dragged position and size.
          </div>
        </div>
      </section>

      <section class="space-y-3 rounded-lg border border-aura-border bg-white/80 px-4 py-4">
        <div>
          <p class="text-[10px] font-display font-semibold uppercase tracking-[0.22em] text-aura-text-muted">
            Provider
          </p>
          <p class="mt-1 text-xs text-aura-text-dim">
            Aura uses any OpenAI-compatible endpoint, including local Ollama.
          </p>
        </div>
        <select
          id="provider"
          value={config.provider}
          onchange={(e) => handleProviderChange((e.currentTarget as HTMLSelectElement).value as Provider)}
          class="w-full cursor-pointer rounded-lg border border-aura-border bg-white px-3 py-2.5 text-sm text-aura-text outline-none transition-all duration-200 hover:border-aura-border-accent focus:border-aura-accent focus:ring-2 focus:ring-aura-accent/15"
        >
          {#each PROVIDER_OPTIONS as opt}
            <option value={opt.value}>{opt.label}</option>
          {/each}
        </select>
      </section>

      {#if shouldShowPlaintextApiKeyWarning(config.provider)}
        <div
          class="rounded-lg border border-[#e6c683] bg-[#fff7e4] px-4 py-3"
          data-testid="plaintext-api-key-warning"
        >
          <p class="text-xs leading-relaxed text-[#8a6226]">
            API keys are stored in plaintext in the local Aura config on this machine. Use a trial or low-permission key when possible.
          </p>
        </div>
      {/if}

      {#if config.provider !== 'ollama'}
        <section class="space-y-3 rounded-lg border border-aura-border bg-white/80 px-4 py-4">
          <div>
            <label class="text-[10px] font-display font-semibold uppercase tracking-[0.22em] text-aura-text-muted" for="api-key">
              API Key
            </label>
            <p class="mt-1 text-xs text-aura-text-dim">
              Stored locally for the selected provider.
            </p>
          </div>
          <div class="relative">
            <input
              id="api-key"
              type={showApiKey ? 'text' : 'password'}
              bind:value={config.api_key}
              placeholder="sk-..."
              class="w-full rounded-lg border border-aura-border bg-white px-3 py-2.5 pr-12 text-sm text-aura-text outline-none transition-all duration-200 placeholder:text-aura-text-muted hover:border-aura-border-accent focus:border-aura-accent focus:ring-2 focus:ring-aura-accent/15"
            />
            <button
              class="absolute right-2 top-1/2 flex h-8 w-8 -translate-y-1/2 items-center justify-center rounded-md text-aura-text-muted transition-colors hover:bg-aura-accent-soft hover:text-aura-accent"
              onclick={() => (showApiKey = !showApiKey)}
              type="button"
              aria-label={showApiKey ? 'Hide API key' : 'Show API key'}
            >
              <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                {#if showApiKey}
                  <path stroke-linecap="round" stroke-linejoin="round" d="M3.98 8.223A10.477 10.477 0 0 1.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0 1 12 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 0 1-4.293 5.774M6.228 6.228 3 3m3.228 3.228 3.65 3.65m7.894 7.894L21 21m-3.228-3.228-3.65-3.65m0 0a3 3 0 1 0-4.243-4.243m4.242 4.242L9.88 9.88" />
                {:else}
                  <path stroke-linecap="round" stroke-linejoin="round" d="M2.036 12.322a1.012 1.012 0 0 1 0-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178z" />
                  <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0z" />
                {/if}
              </svg>
            </button>
          </div>
        </section>
      {/if}

      <section class="space-y-3 rounded-lg border border-aura-border bg-white/80 px-4 py-4">
        <div>
          <label class="text-[10px] font-display font-semibold uppercase tracking-[0.22em] text-aura-text-muted" for="model">
            Model
          </label>
          <p class="mt-1 text-xs text-aura-text-dim">
            The list updates when you switch providers.
          </p>
        </div>
        <select
          id="model"
          bind:value={config.model}
          class="w-full cursor-pointer rounded-lg border border-aura-border bg-white px-3 py-2.5 text-sm text-aura-text outline-none transition-all duration-200 hover:border-aura-border-accent focus:border-aura-accent focus:ring-2 focus:ring-aura-accent/15"
        >
          {#each config.available_models as m}
            <option value={m}>{m}</option>
          {/each}
        </select>
      </section>

      <section class="space-y-3 rounded-lg border border-aura-border bg-white/80 px-4 py-4">
        <div>
          <label class="text-[10px] font-display font-semibold uppercase tracking-[0.22em] text-aura-text-muted" for="hotkey-capture">
            Hotkey
          </label>
          <p class="mt-1 text-xs text-aura-text-dim">
            Use the hotkey to translate manually, or to recall and hide the floating translation bubble.
          </p>
        </div>
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
            class={`w-full cursor-pointer select-none rounded-lg border bg-white px-3 py-2.5 pr-28 text-sm font-medium text-aura-text outline-none transition-all duration-200 ${
              isCapturingHotkey
                ? 'border-aura-accent ring-2 ring-aura-accent/15'
                : 'border-aura-border hover:border-aura-border-accent'
            }`}
          />
          <span class={`absolute right-3 top-1/2 -translate-y-1/2 text-[11px] font-medium ${isCapturingHotkey ? 'text-aura-accent' : 'text-aura-text-muted'}`}>
            {isCapturingHotkey ? 'recording...' : 'click to edit'}
          </span>
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
            class="rounded-lg border border-[#e6c683] bg-[#fff7e4] px-4 py-3 text-xs leading-relaxed text-[#8a6226]"
            data-testid="hotkey-conflict-inline"
          >
            {hotkeyConflictMessage}
          </div>
        {/if}
      </section>
    </div>

    <div class="border-t border-aura-border px-5 py-4">
      {#if saveErrorMessage}
        <p class="mb-3 text-sm text-aura-error" data-testid="save-error-message">{saveErrorMessage}</p>
      {:else if saveMessage}
        <p class="mb-3 text-sm text-aura-success">{saveMessage}</p>
      {/if}

      <button
        class="w-full rounded-lg bg-aura-accent py-3 text-sm font-display font-medium text-white transition-all duration-200 hover:brightness-105 active:brightness-95 disabled:cursor-not-allowed disabled:opacity-50"
        style:transform="scale({saveScale.current})"
        onclick={saveConfig}
        disabled={saving}
        type="button"
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
