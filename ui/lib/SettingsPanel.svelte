<script lang="ts">
  import { Spring } from 'svelte/motion';
  import { invoke } from '@tauri-apps/api/core';
  import HistoryList from './HistoryList.svelte';
  import LanguageSelector from './LanguageSelector.svelte';
  import ProfileManager from './ProfileManager.svelte';
  import SetupStatusCard from './SetupStatusCard.svelte';
  import type { ApiKeyStorage, AppConfig, Provider } from './appConfig';
  import { cloneAppConfig, createDefaultAppConfig } from './appConfig';
  import {
    shouldShowPlaintextApiKeyWarning,
    usesSystemCredentialStorage,
  } from './notifications';
  import type { ProviderProbeResult, RuntimeStatus } from './runtimeStatus';
  import type { TranslationHistoryEntry } from './translationHistory';
  import type { TranslationProfilesStore } from './translationProfiles';

  const PROVIDER_OPTIONS: { value: Provider; label: string }[] = [
    { value: 'deepseek', label: 'DeepSeek (api.deepseek.com)' },
    { value: 'openrouter', label: 'OpenRouter (openrouter.ai)' },
    { value: 'ollama', label: 'Ollama（本地，无需认证）' },
  ];

  type ProviderDefaults = {
    base_url: string;
    models: string[];
  };

  type SettingsSection = 'general' | 'provider' | 'behavior' | 'profiles' | 'history';

  const SETTINGS_SECTIONS: Array<{ id: SettingsSection; label: string; hint: string }> = [
    { id: 'general', label: '常规', hint: '语言与状态' },
    { id: 'provider', label: '服务商', hint: '模型与凭据' },
    { id: 'behavior', label: '快捷键与行为', hint: '触发与窗口' },
    { id: 'profiles', label: '配置方案', hint: '工作流切换' },
    { id: 'history', label: '历史记录', hint: '最近请求日志' },
  ];

  type Props = {
    visible: boolean;
    onclose: () => void;
    onsaved?: (config: AppConfig) => void;
    hotkeyConflictMessage?: string;
  };

  let { visible, onclose, onsaved, hotkeyConflictMessage = '' }: Props = $props();

  let config: AppConfig = $state(createDefaultAppConfig());
  let activeSection = $state<SettingsSection>('general');
  let showApiKey = $state(false);
  let saving = $state(false);
  let testingProvider = $state(false);
  let saveMessage = $state('');
  let saveErrorMessage = $state('');
  let panelErrorMessage = $state('');
  let hotkeyInputMessage = $state('');
  let isCapturingHotkey = $state(false);
  let historyEntries = $state<TranslationHistoryEntry[]>([]);
  let profileStore = $state<TranslationProfilesStore | null>(null);
  let profileDraftName = $state('');
  let runtimeStatus = $state<RuntimeStatus | null>(null);
  let probeResult = $state<ProviderProbeResult | null>(null);

  const saveScale = new Spring(1, { stiffness: 0.4, damping: 0.5 });

  const activeSectionMeta = $derived(
    SETTINGS_SECTIONS.find((section) => section.id === activeSection) ?? SETTINGS_SECTIONS[0],
  );

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
      const [loaded, nextRuntimeStatus, nextHistoryEntries, nextProfileStore] = await Promise.all([
        invoke<AppConfig>('get_config'),
        invoke<RuntimeStatus>('get_runtime_status'),
        invoke<TranslationHistoryEntry[]>('get_translation_history'),
        invoke<TranslationProfilesStore>('get_translation_profiles'),
      ]);
      config = cloneAppConfig(loaded);
      if (config.api_key_storage === 'legacy_plaintext') {
        config.api_key_storage = 'plaintext_fallback';
      }
      runtimeStatus = nextRuntimeStatus;
      historyEntries = nextHistoryEntries;
      profileStore = nextProfileStore;
      profileDraftName = getActiveProfileName(nextProfileStore);
    } catch (e) {
      panelErrorMessage = '无法从桌面后端加载设置、配置方案或翻译历史。';
      console.error('Failed to load config:', { error: e });
    }
  }

  function getActiveProfileName(store: TranslationProfilesStore | null): string {
    if (!store) {
      return '';
    }

    return store.profiles.find((profile) => profile.id === store.active_profile_id)?.name ?? '';
  }

  async function handleProviderChange(newProvider: Provider) {
    config.provider = newProvider;
    panelErrorMessage = '';
    saveErrorMessage = '';
    probeResult = null;
    try {
      const [defaults, storedApiKey] = await Promise.all([
        invoke<ProviderDefaults>('get_provider_defaults', {
          provider: newProvider,
        }),
        newProvider === 'ollama'
          ? Promise.resolve('')
          : invoke<string>('load_provider_api_key', { provider: newProvider }),
      ]);
      config.api_base_url = defaults.base_url;
      config.available_models = defaults.models;
      config.api_key = storedApiKey;
      if (!defaults.models.includes(config.model)) {
        config.model = defaults.models[0];
      }
    } catch (e) {
      panelErrorMessage = '无法从桌面后端加载该服务商的默认配置。';
      console.error('Failed to fetch provider defaults:', e);
    }
  }

  async function copyHistoryResult(translatedText: string) {
    try {
      await invoke('copy_result_to_clipboard', { text: translatedText });
    } catch (e) {
      panelErrorMessage = '无法复制保存的译文。';
      console.error('Failed to copy history result:', { error: e });
    }
  }

  async function retryHistoryEntry(entryId: string) {
    try {
      await invoke('replay_translation_history_entry', { entryId });
    } catch (e) {
      panelErrorMessage = '无法重试选中的历史记录。';
      console.error('Failed to replay history entry:', { error: e });
    }
  }

  async function deleteHistoryEntry(entryId: string) {
    try {
      historyEntries = await invoke<TranslationHistoryEntry[]>(
        'delete_translation_history_entry',
        { entryId },
      );
    } catch (e) {
      panelErrorMessage = '无法删除选中的历史记录。';
      console.error('Failed to delete history entry:', { error: e });
    }
  }

  async function clearHistoryEntries() {
    try {
      historyEntries = await invoke<TranslationHistoryEntry[]>('clear_translation_history');
    } catch (e) {
      panelErrorMessage = '无法清空翻译历史。';
      console.error('Failed to clear history:', { error: e });
    }
  }

  async function createProfile() {
    if (!profileDraftName.trim()) {
      panelErrorMessage = '请先输入配置方案名称，再保存新方案。';
      return;
    }

    try {
      profileStore = await invoke<TranslationProfilesStore>('create_translation_profile', {
        config: cloneAppConfig(config),
        name: profileDraftName.trim(),
      });
      const refreshedConfig = await invoke<AppConfig>('get_config');
      config = cloneAppConfig(refreshedConfig);
      profileDraftName = getActiveProfileName(profileStore);
    } catch (e) {
      panelErrorMessage = '无法创建翻译配置方案。';
      console.error('Failed to create profile:', { error: e });
    }
  }

  async function renameActiveProfile(profileId: string) {
    if (!profileDraftName.trim()) {
      panelErrorMessage = '请先输入配置方案名称，再重命名当前方案。';
      return;
    }

    try {
      profileStore = await invoke<TranslationProfilesStore>('rename_translation_profile', {
        profileId,
        name: profileDraftName.trim(),
      });
      profileDraftName = getActiveProfileName(profileStore);
    } catch (e) {
      panelErrorMessage = '无法重命名当前翻译配置方案。';
      console.error('Failed to rename profile:', { error: e });
    }
  }

  async function activateProfile(profileId: string) {
    try {
      profileStore = await invoke<TranslationProfilesStore>('activate_translation_profile', {
        profileId,
      });
      const refreshedConfig = await invoke<AppConfig>('get_config');
      config = cloneAppConfig(refreshedConfig);
      profileDraftName = getActiveProfileName(profileStore);
    } catch (e) {
      panelErrorMessage = '无法启用选中的翻译配置方案。';
      console.error('Failed to activate profile:', { error: e });
    }
  }

  async function deleteProfile(profileId: string) {
    try {
      profileStore = await invoke<TranslationProfilesStore>('delete_translation_profile', {
        profileId,
      });
      const refreshedConfig = await invoke<AppConfig>('get_config');
      config = cloneAppConfig(refreshedConfig);
      profileDraftName = getActiveProfileName(profileStore);
    } catch (e) {
      panelErrorMessage = '无法删除选中的翻译配置方案。';
      console.error('Failed to delete profile:', { error: e });
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
      hotkeyInputMessage = '快捷键至少需要包含一个修饰键。';
      return;
    }

    const key = e.key.length === 1 && /^[a-z0-9]$/i.test(e.key) ? e.key.toUpperCase() : null;
    if (!key) {
      hotkeyInputMessage = '请使用字母或数字键，并至少搭配一个修饰键。';
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
      panelErrorMessage = '无法从桌面后端刷新 Aura 就绪状态。';
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
        message: '无法从桌面后端运行服务商测试。',
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
      saveMessage = '设置已保存';
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
        ? '无法保存设置，请修正快捷键后重试。'
        : '无法保存到本地 Aura 配置。';
      saveScale.target = 1;
      console.error('Failed to save config:', { error: e });
    }

    saving = false;
  }
</script>

{#if visible}
  <div
    class="aura-glass-panel z-50 flex min-h-0 flex-col"
    style="animation: fade-in-up 0.25s ease-out both;"
  >
    <div class="flex items-start justify-between border-b border-aura-border px-5 py-4">
      <div class="flex-1 pr-4" data-tauri-drag-region>
        <h2 class="text-base font-display font-semibold text-aura-text" data-tauri-drag-region>
          设置
        </h2>
        <p class="mt-1 text-xs leading-relaxed text-aura-text-dim" data-tauri-drag-region>
          配置语言、触发方式、服务商访问和窗口记忆。
        </p>
      </div>
      <button
        class="aura-console-icon-button"
        onclick={onclose}
        aria-label="关闭设置"
        type="button"
      >
        <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>

    <div class="min-h-0 flex-1 px-4 py-4">
      <div
        class="flex h-full min-h-0 overflow-hidden border border-aura-border bg-aura-surface-strong"
        data-testid="settings-workspace"
      >
        <aside
          class="w-[190px] shrink-0 border-r border-aura-border bg-aura-surface-soft/80 p-2"
          data-testid="settings-nav"
        >
          <div class="space-y-1">
            {#each SETTINGS_SECTIONS as section}
              <button
                class={`w-full border px-3 py-3 text-left transition-colors duration-150 ${
                  activeSection === section.id
                    ? 'border-aura-accent bg-aura-accent-soft text-aura-text'
                    : 'border-transparent text-aura-text-dim hover:border-aura-border hover:bg-white'
                }`}
                type="button"
                data-testid={`settings-nav-${section.id}`}
                aria-pressed={activeSection === section.id}
                onclick={() => (activeSection = section.id)}
              >
                <span class="block text-sm font-medium">{section.label}</span>
                <span class="mt-1 block text-[11px] text-aura-text-muted">{section.hint}</span>
              </button>
            {/each}
          </div>
        </aside>

        <div class="min-w-0 flex flex-1 min-h-0 flex-col">
          <div class="border-b border-aura-border px-5 py-4">
            <p class="aura-section-title" data-testid="settings-active-section">{activeSectionMeta.label}</p>
            <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">{activeSectionMeta.hint}</p>
          </div>

          <div class="min-h-0 flex-1 overflow-y-auto px-5 py-5">
            {#if panelErrorMessage}
              <div class="mb-5 border border-aura-error/25 bg-[#fff8f9] px-4 py-3 text-sm leading-relaxed text-aura-error">
                {panelErrorMessage}
              </div>
            {/if}

            {#if activeSection === 'general'}
              <div class="space-y-6">
                <SetupStatusCard
                  status={runtimeStatus}
                  testing={testingProvider}
                  {probeResult}
                  ontest={handleProviderProbe}
                />

                <section class="space-y-4 border-t border-aura-border pt-5">
                  <div>
                    <p class="aura-section-title">翻译语言</p>
                    <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
                      设置每次翻译请求默认使用的语言方向。
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
              </div>
            {:else if activeSection === 'provider'}
              <div class="space-y-6">
                <SetupStatusCard
                  status={runtimeStatus}
                  testing={testingProvider}
                  {probeResult}
                  ontest={handleProviderProbe}
                />

                <section class="space-y-5 border-t border-aura-border pt-5">
                  <div class="grid gap-3 lg:grid-cols-[220px_minmax(0,1fr)]">
                    <div>
                      <label class="aura-section-title" for="provider">服务商</label>
                      <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
                        Aura 可使用任意 OpenAI 兼容接口，也支持本地 Ollama。
                      </p>
                    </div>
                    <select
                      id="provider"
                      value={config.provider}
                      onchange={(e) => handleProviderChange((e.currentTarget as HTMLSelectElement).value as Provider)}
                      class="aura-console-select"
                    >
                      {#each PROVIDER_OPTIONS as opt}
                        <option value={opt.value}>{opt.label}</option>
                      {/each}
                    </select>
                  </div>

                  {#if config.provider !== 'ollama'}
                    <div class="grid gap-3 lg:grid-cols-[220px_minmax(0,1fr)]">
                      <div>
                        <label class="aura-section-title" for="api-key-storage">API Key 存储</label>
                        <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
                          选择将服务商密钥存入系统凭据库，或作为明文备用写入 `config.json`。
                        </p>
                      </div>
                      <div class="space-y-3">
                        <select
                          id="api-key-storage"
                          bind:value={config.api_key_storage}
                          class="aura-console-select"
                        >
                          <option value="system">系统凭据存储（推荐）</option>
                          <option value="plaintext_fallback">明文配置备用</option>
                        </select>

                        {#if shouldShowPlaintextApiKeyWarning(config.provider, config.api_key_storage)}
                          <div
                            class="border border-[#e6c683] bg-[#fff7e4] px-4 py-3 text-xs leading-relaxed text-[#8a6226]"
                            data-testid="plaintext-api-key-warning"
                          >
                            明文备用模式会把 API Key 写入本机 Aura 配置。尽量使用试用或低权限密钥。
                          </div>
                        {:else if usesSystemCredentialStorage(config.api_key_storage)}
                          <div
                            class="border border-aura-border bg-aura-surface-soft px-4 py-3 text-xs leading-relaxed text-aura-text-dim"
                            data-testid="system-api-key-storage-note"
                          >
                            系统模式会把 API Key 存入操作系统凭据库，而不是 `config.json`。
                          </div>
                        {/if}
                      </div>
                    </div>

                    <div class="grid gap-3 lg:grid-cols-[220px_minmax(0,1fr)]">
                      <div>
                        <label class="aura-section-title" for="api-key">API Key</label>
                        <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
                          {#if usesSystemCredentialStorage(config.api_key_storage)}
                            当前服务商密钥会保存在系统凭据库中。
                          {:else}
                            当前服务商密钥会保存在本地 Aura 配置中。
                          {/if}
                        </p>
                      </div>
                      <div class="relative">
                        <input
                          id="api-key"
                          type={showApiKey ? 'text' : 'password'}
                          bind:value={config.api_key}
                          placeholder="sk-..."
                          class="aura-console-input pr-12"
                        />
                        <button
                          class="absolute right-2 top-1/2 flex h-8 w-8 -translate-y-1/2 items-center justify-center text-aura-text-muted transition-colors hover:text-aura-accent"
                          onclick={() => (showApiKey = !showApiKey)}
                          type="button"
                          aria-label={showApiKey ? '隐藏 API Key' : '显示 API Key'}
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
                    </div>
                  {/if}

                  <div class="grid gap-3 lg:grid-cols-[220px_minmax(0,1fr)]">
                    <div>
                      <label class="aura-section-title" for="model">模型</label>
                      <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
                        切换服务商后，模型列表会自动更新。
                      </p>
                    </div>
                    <select
                      id="model"
                      bind:value={config.model}
                      class="aura-console-select"
                    >
                      {#each config.available_models as m}
                        <option value={m}>{m}</option>
                      {/each}
                    </select>
                  </div>
                </section>
              </div>
            {:else if activeSection === 'behavior'}
              <div class="space-y-5">
                <section class="space-y-2">
                  <div class="grid gap-3 border border-aura-border px-3 py-3 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-center">
                    <div>
                      <p class="text-sm font-medium text-aura-text">Aura 模式</p>
                      <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
                        在 Windows 上，剪贴板文本变化后自动发起翻译。
                      </p>
                    </div>
                    <button
                      class={`aura-console-switch ${config.aura_mode_enabled ? 'is-on' : ''}`}
                      role="switch"
                      aria-checked={config.aura_mode_enabled}
                      aria-label="Aura 模式"
                      type="button"
                      onclick={() => (config.aura_mode_enabled = !config.aura_mode_enabled)}
                    >
                      <span class="aura-console-switch-thumb"></span>
                    </button>
                  </div>
                  <p class="text-xs leading-relaxed text-aura-text-dim">
                    关闭 Aura 模式后，仍可继续使用“复制文本后按快捷键”的手动流程。
                  </p>
                </section>

                <section class="space-y-2">
                  <div class="grid gap-3 border border-aura-border px-3 py-3 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-center">
                    <div>
                      <p class="text-sm font-medium text-aura-text">敏感剪贴板保护</p>
                      <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
                        Aura 模式发送内容前，会跳过疑似密码、令牌和长凭据字符串。
                      </p>
                    </div>
                    <button
                      class={`aura-console-switch ${config.aura_guard_enabled ? 'is-on' : ''}`}
                      role="switch"
                      aria-checked={config.aura_guard_enabled}
                      aria-label="敏感剪贴板保护"
                      type="button"
                      onclick={() => (config.aura_guard_enabled = !config.aura_guard_enabled)}
                    >
                      <span class="aura-console-switch-thumb"></span>
                    </button>
                  </div>
                  <p class="text-xs leading-relaxed text-aura-text-dim">
                    这项保护只影响 Aura 模式的自动剪贴板翻译；手动快捷键翻译仍会使用你主动触发的文本。
                  </p>
                </section>

                <section class="space-y-2">
                  <div class="grid gap-3 border border-aura-border px-3 py-3 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-center">
                    <div>
                      <p class="text-sm font-medium text-aura-text">固定窗口</p>
                      <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
                        需要翻译窗保持可见、可移动并置顶时，可以固定它。
                      </p>
                    </div>
                    <button
                      class={`aura-console-switch ${config.window_pinned ? 'is-on' : ''}`}
                      role="switch"
                      aria-checked={config.window_pinned}
                      aria-label="固定窗口"
                      type="button"
                      onclick={() => (config.window_pinned = !config.window_pinned)}
                    >
                      <span class="aura-console-switch-thumb"></span>
                    </button>
                  </div>
                  <div class="grid gap-2 text-xs text-aura-text-dim sm:grid-cols-2">
                    <div class="border border-aura-border px-3 py-3">
                      未固定：在光标附近唤起，失焦后自动隐藏。
                    </div>
                    <div class="border border-aura-border px-3 py-3">
                      已固定：记住上次拖动后的位置和大小。
                    </div>
                  </div>
                </section>

                <section class="space-y-3 border-t border-aura-border pt-5">
                  <div class="grid gap-3 lg:grid-cols-[220px_minmax(0,1fr)]">
                    <div>
                      <label class="aura-section-title" for="hotkey-capture">快捷键</label>
                      <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
                        使用快捷键手动翻译，也可唤起或隐藏悬浮翻译窗。
                      </p>
                    </div>
                    <div class="space-y-3">
                      <div class="relative">
                        <input
                          id="hotkey-capture"
                          type="text"
                          value={config.hotkey}
                          placeholder="按下快捷键组合"
                          readonly
                          onfocus={() => (isCapturingHotkey = true)}
                          onblur={() => (isCapturingHotkey = false)}
                          onkeydown={handleHotkeyKeydown}
                          class={`aura-console-input cursor-pointer select-none pr-24 font-medium ${
                            isCapturingHotkey ? 'border-aura-accent shadow-[0_0_0_3px_var(--color-aura-focus-ring)]' : ''
                          }`}
                        />
                        <span class={`absolute right-3 top-1/2 -translate-y-1/2 text-[11px] font-medium ${
                          isCapturingHotkey ? 'text-aura-accent' : 'text-aura-text-muted'
                        }`}>
                          {isCapturingHotkey ? '记录中...' : '点击编辑'}
                        </span>
                      </div>

                      <p class="text-xs text-aura-text-muted">
                        快捷键必须包含至少一个修饰键，并搭配一个字母或数字键。
                      </p>

                      {#if hotkeyInputMessage}
                        <p class="text-xs text-[#8a6226]" data-testid="hotkey-input-message">
                          {hotkeyInputMessage}
                        </p>
                      {/if}

                      {#if hotkeyConflictMessage}
                        <div
                          class="border border-[#e6c683] bg-[#fff7e4] px-4 py-3 text-xs leading-relaxed text-[#8a6226]"
                          data-testid="hotkey-conflict-inline"
                        >
                          {hotkeyConflictMessage}
                        </div>
                      {/if}
                    </div>
                  </div>
                </section>
              </div>
            {:else if activeSection === 'profiles'}
              <ProfileManager
                store={profileStore}
                draftName={profileDraftName}
                ondraftnamechange={(value) => {
                  profileDraftName = value;
                }}
                oncreate={createProfile}
                onrename={renameActiveProfile}
                onactivate={activateProfile}
                ondelete={deleteProfile}
              />
            {:else}
              <HistoryList
                entries={historyEntries}
                oncopy={copyHistoryResult}
                onretry={retryHistoryEntry}
                ondelete={deleteHistoryEntry}
                onclear={clearHistoryEntries}
              />
            {/if}
          </div>
        </div>
      </div>
    </div>

    <div class="border-t border-aura-border px-4 py-3">
      <div class="flex flex-wrap items-center justify-between gap-3">
        <div class="min-h-[1.25rem] text-sm">
          {#if saveErrorMessage}
            <p class="text-aura-error" data-testid="save-error-message">{saveErrorMessage}</p>
          {:else if saveMessage}
            <p class="text-aura-success">{saveMessage}</p>
          {/if}
        </div>

        <button
          class="aura-console-button min-w-[150px]"
          data-variant="primary"
          style:transform="scale({saveScale.current})"
          onclick={saveConfig}
          disabled={saving}
          type="button"
        >
          {#if saving}
            保存中...
          {:else}
            保存设置
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}
