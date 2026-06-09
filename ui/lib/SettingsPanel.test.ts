import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import SettingsPanel from './SettingsPanel.svelte';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

const baseConfig = {
  api_key: 'sk-test',
  api_key_storage: 'system',
  active_profile_id: 'default',
  model: 'deepseek-chat',
  source_lang: 'auto',
  target_lang: 'Chinese',
  hotkey: 'CmdOrCtrl+T',
  aura_mode_enabled: false,
  aura_guard_enabled: true,
  window_pinned: false,
  provider: 'deepseek',
  api_base_url: 'https://api.deepseek.com',
  available_models: ['deepseek-chat', 'deepseek-reasoner'],
  settings_window_placement: null,
  pinned_translation_placement: null,
};

const readyStatus = {
  level: 'ready',
  summary: 'Aura 已准备好使用 DeepSeek 和模型 deepseek-chat 翻译。',
  can_translate_now: true,
  checklist: [
    { code: 'provider', label: '服务商：DeepSeek', ok: true },
    { code: 'base_url', label: 'API 地址已配置', ok: true },
    { code: 'model', label: '默认模型已选择', ok: true },
    { code: 'api_key', label: 'API Key 已保存', ok: true },
  ],
};

const needsSetupStatus = {
  level: 'needs_setup',
  summary: '请保存 API Key；如果想使用本地服务商，可以切换到 Ollama。',
  can_translate_now: false,
  checklist: [
    { code: 'provider', label: '服务商：DeepSeek', ok: true },
    { code: 'base_url', label: 'API 地址已配置', ok: true },
    { code: 'model', label: '默认模型已选择', ok: true },
    { code: 'api_key', label: 'API Key 已保存', ok: false },
  ],
};

const baseHistoryEntries = [
  {
    id: 'history-1',
    source_text: 'Hello world',
    translated_text: '你好，世界',
    error_message: null,
    source_lang: 'English',
    target_lang: 'Chinese',
    provider: 'deepseek',
    model: 'deepseek-chat',
    usage: {
      prompt_tokens: 9,
      completion_tokens: 4,
      total_tokens: 13,
    },
    status: 'success',
    created_at_ms: 1_717_171_717_000,
  },
];

const baseProfileStore = {
  active_profile_id: 'default',
  profiles: [
    {
      id: 'default',
      name: 'Default',
      api_key: '',
      api_key_storage: 'system',
      model: 'deepseek-chat',
      source_lang: 'auto',
      target_lang: 'Chinese',
      provider: 'deepseek',
      api_base_url: 'https://api.deepseek.com',
      available_models: ['deepseek-chat', 'deepseek-reasoner'],
    },
  ],
};

function renderPanel(props: { hotkeyConflictMessage?: string; onsaved?: (config: unknown) => void } = {}) {
  return render(SettingsPanel, {
    visible: true,
    onclose: () => {},
    hotkeyConflictMessage: props.hotkeyConflictMessage ?? '',
    onsaved: props.onsaved,
  });
}

async function openSection(section: 'overview' | 'general' | 'provider' | 'behavior' | 'profiles' | 'history') {
  await fireEvent.click(await screen.findByTestId(`settings-nav-${section}`));
}

describe('SettingsPanel operator console layout', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockImplementation((command: string, payload?: { provider?: string; config?: typeof baseConfig; name?: string }) => {
      switch (command) {
        case 'get_config':
          return Promise.resolve(baseConfig);
        case 'get_system_capabilities':
          return Promise.resolve({
            aura_mode: 'ready',
            paste_back: 'ready',
          });
        case 'get_runtime_status':
          return Promise.resolve(readyStatus);
        case 'get_translation_history':
          return Promise.resolve(baseHistoryEntries);
        case 'get_translation_profiles':
          return Promise.resolve(baseProfileStore);
        case 'get_provider_defaults':
          if (payload?.provider === 'ollama') {
            return Promise.resolve({
              base_url: 'http://localhost:11434',
              models: ['qwen2.5'],
            });
          }
          return Promise.resolve({
            base_url: 'https://api.deepseek.com',
            models: ['deepseek-chat', 'deepseek-reasoner'],
          });
        case 'save_config':
          return Promise.resolve(undefined);
        case 'probe_provider':
          return Promise.resolve({
            ok: true,
            message: `${payload?.config?.provider ?? 'deepseek'} 的服务商测试成功。`,
          });
        case 'load_provider_api_key':
          return Promise.resolve('sk-loaded');
        case 'replay_translation_history_entry':
          return Promise.resolve(undefined);
        case 'copy_result_to_clipboard':
          return Promise.resolve(undefined);
        case 'delete_translation_history_entry':
          return Promise.resolve([]);
        case 'clear_translation_history':
          return Promise.resolve([]);
        case 'create_translation_profile':
        case 'rename_translation_profile':
        case 'activate_translation_profile':
        case 'delete_translation_profile':
          return Promise.resolve(baseProfileStore);
        default:
          return Promise.resolve(undefined);
      }
    });
  });

  it('loads the operator console shell with section navigation', async () => {
    renderPanel({
      hotkeyConflictMessage: '无法注册 "Alt+Shift+T"：已被占用。',
    });

    expect(await screen.findByTestId('settings-nav')).toBeInTheDocument();
    expect(screen.getByTestId('settings-nav-overview')).toHaveAttribute('aria-pressed', 'true');
    for (const section of ['overview', 'general', 'provider', 'behavior', 'profiles', 'history']) {
      expect(screen.getByTestId(`settings-nav-${section}`).querySelector('svg[aria-hidden="true"]')).not.toBeNull();
    }
    expect(screen.getByTestId('settings-overview')).toHaveTextContent('DeepSeek');
    await waitFor(() => {
      expect(screen.getByTestId('runtime-status-card')).toHaveTextContent('Aura 已准备好');
    });
    expect(screen.getByTestId('overview-general-card')).toHaveTextContent('自动');
    expect(screen.getByTestId('runtime-status-card')).toHaveTextContent('Aura 已准备好');
    expect(invokeMock).toHaveBeenCalledWith('get_config');
    expect(invokeMock).toHaveBeenCalledWith('get_runtime_status');
    expect(invokeMock).toHaveBeenCalledWith('get_translation_history');
    expect(invokeMock).toHaveBeenCalledWith('get_translation_profiles');

    await fireEvent.click(screen.getByTestId('overview-provider-card'));
    expect(screen.getByTestId('settings-nav-provider')).toHaveAttribute('aria-pressed', 'true');

    await openSection('history');
    expect(await screen.findByTestId('history-list')).toHaveTextContent('Hello world');

    await openSection('profiles');
    expect(await screen.findByTestId('profile-list')).toHaveTextContent('Default');
  });

  it('keeps the close button outside the draggable title region', async () => {
    const { container } = renderPanel();

    const closeButton = container.querySelector('button[aria-label="关闭设置"]');
    expect(closeButton).not.toBeNull();
    if (!closeButton) {
      throw new Error('Expected the settings close button to be rendered.');
    }
    expect(closeButton.closest('[data-tauri-drag-region]')).toBeNull();
  });

  it('hides API key storage messaging for ollama', async () => {
    invokeMock.mockImplementation((command: string, payload?: { provider?: string }) => {
      switch (command) {
        case 'get_system_capabilities':
          return Promise.resolve({
            aura_mode: 'ready',
            paste_back: 'ready',
          });
        case 'get_config':
          return Promise.resolve({
            ...baseConfig,
            api_key: '',
            api_key_storage: 'system',
            model: 'qwen2.5',
            provider: 'ollama',
            api_base_url: 'http://localhost:11434',
            available_models: ['qwen2.5'],
          });
        case 'get_runtime_status':
          return Promise.resolve({
            ...readyStatus,
            summary: 'Aura 已准备好使用 Ollama 和模型 qwen2.5 翻译。',
          });
        case 'get_translation_history':
          return Promise.resolve(baseHistoryEntries);
        case 'get_translation_profiles':
          return Promise.resolve(baseProfileStore);
        case 'get_provider_defaults':
          if (payload?.provider === 'ollama') {
            return Promise.resolve({
              base_url: 'http://localhost:11434',
              models: ['qwen2.5'],
            });
          }
          return Promise.resolve({
            base_url: 'https://api.deepseek.com',
            models: ['deepseek-chat', 'deepseek-reasoner'],
          });
        default:
          return Promise.resolve(undefined);
      }
    });

    renderPanel();
    await openSection('provider');

    await waitFor(() => {
      expect(screen.queryByTestId('plaintext-api-key-warning')).not.toBeInTheDocument();
      expect(screen.queryByTestId('system-api-key-storage-note')).not.toBeInTheDocument();
    });
  });

  it('switches to plaintext fallback storage and saves the selection', async () => {
    renderPanel();
    await openSection('provider');

    const storageSelect = await screen.findByLabelText(/API Key 存储/);
    await fireEvent.change(storageSelect, { target: { value: 'plaintext_fallback' } });

    expect(screen.getByTestId('plaintext-api-key-warning')).toHaveTextContent('写入本机 Aura 配置');

    await fireEvent.click(screen.getByRole('button', { name: /保存设置/ }));

    expect(invokeMock).toHaveBeenCalledWith(
      'save_config',
      expect.objectContaining({
        config: expect.objectContaining({ api_key_storage: 'plaintext_fallback' }),
      }),
    );
  });

  it('rejects bare single-key hotkeys during capture', async () => {
    renderPanel({
      hotkeyConflictMessage: '无法注册 "Alt+Shift+T"：已被占用。',
    });
    await openSection('behavior');

    const input = await screen.findByDisplayValue('CmdOrCtrl+T');
    await fireEvent.focus(input);
    await fireEvent.keyDown(input, { key: 't' });

    expect(screen.getByTestId('hotkey-input-message')).toHaveTextContent('快捷键至少需要包含一个修饰键');
    expect(screen.getByTestId('hotkey-conflict-inline')).toHaveTextContent('无法注册 "Alt+Shift+T"');
    expect(input).toHaveValue('CmdOrCtrl+T');
  });

  it('captures modifier-based hotkeys and clears the inline validation message', async () => {
    renderPanel();
    await openSection('behavior');

    const input = await screen.findByDisplayValue('CmdOrCtrl+T');
    await fireEvent.focus(input);
    await fireEvent.keyDown(input, { key: 't' });
    await fireEvent.keyDown(input, { key: 'k', ctrlKey: true });

    expect(screen.queryByTestId('hotkey-input-message')).not.toBeInTheDocument();
    expect(input).toHaveValue('CmdOrCtrl+K');
  });

  it('loads and saves the aura mode preference', async () => {
    renderPanel();
    await openSection('behavior');

    const auraSwitch = await screen.findByRole('switch', { name: /Aura 模式/ });
    expect(auraSwitch).toHaveAttribute('aria-checked', 'false');

    await fireEvent.click(auraSwitch);
    await fireEvent.click(screen.getByRole('button', { name: /保存设置/ }));

    expect(invokeMock).toHaveBeenCalledWith(
      'save_config',
      expect.objectContaining({
        config: expect.objectContaining({ aura_mode_enabled: true }),
      }),
    );
  });

  it('disables aura mode toggle when capability is unsupported', async () => {
    invokeMock.mockImplementation((command: string) => {
      switch (command) {
        case 'get_config':
          return Promise.resolve({
            ...baseConfig,
            aura_mode_enabled: true,
          });
        case 'get_system_capabilities':
          return Promise.resolve({
            aura_mode: 'unsupported',
            paste_back: 'unsupported',
          });
        case 'get_runtime_status':
          return Promise.resolve(readyStatus);
        case 'get_translation_history':
          return Promise.resolve(baseHistoryEntries);
        case 'get_translation_profiles':
          return Promise.resolve(baseProfileStore);
        case 'save_config':
          return Promise.resolve(undefined);
        default:
          return Promise.resolve(undefined);
      }
    });

    renderPanel();
    await openSection('behavior');

    const auraSwitch = await screen.findByRole('switch', { name: /Aura/ });
    expect(auraSwitch).toHaveAttribute('aria-checked', 'false');
    expect(auraSwitch).toBeDisabled();
    expect(screen.getByTestId('aura-mode-platform-note')).toHaveTextContent('不支持');

    await fireEvent.click(auraSwitch);
    await fireEvent.click(screen.getByRole('button', { name: /保存设置/ }));

    expect(invokeMock).toHaveBeenCalledWith(
      'save_config',
      expect.objectContaining({
        config: expect.objectContaining({ aura_mode_enabled: false }),
      }),
    );
  });

  it('loads and saves the sensitive clipboard guard preference', async () => {
    renderPanel();
    await openSection('behavior');

    const guardSwitch = await screen.findByRole('switch', { name: /敏感剪贴板保护/ });
    expect(guardSwitch).toHaveAttribute('aria-checked', 'true');

    await fireEvent.click(guardSwitch);
    await fireEvent.click(screen.getByRole('button', { name: /保存设置/ }));

    expect(invokeMock).toHaveBeenCalledWith(
      'save_config',
      expect.objectContaining({
        config: expect.objectContaining({ aura_guard_enabled: false }),
      }),
    );
  });

  it('loads and saves the window pin preference', async () => {
    const onsaved = vi.fn();

    renderPanel({ onsaved });
    await openSection('behavior');

    const pinSwitch = await screen.findByRole('switch', { name: /固定窗口/ });
    expect(pinSwitch).toHaveAttribute('aria-checked', 'false');

    await fireEvent.click(pinSwitch);
    await fireEvent.click(screen.getByRole('button', { name: /保存设置/ }));

    expect(invokeMock).toHaveBeenCalledWith(
      'save_config',
      expect.objectContaining({
        config: expect.objectContaining({ window_pinned: true }),
      }),
    );
    await waitFor(() => {
      expect(onsaved).toHaveBeenCalledWith(expect.objectContaining({ window_pinned: true }));
    });
  });

  it('shows setup guidance when the saved config is not ready', async () => {
    invokeMock.mockImplementation((command: string) => {
      switch (command) {
        case 'get_config':
          return Promise.resolve({
            ...baseConfig,
            api_key: '',
          });
        case 'get_system_capabilities':
          return Promise.resolve({
            aura_mode: 'ready',
            paste_back: 'ready',
          });
        case 'get_runtime_status':
          return Promise.resolve(needsSetupStatus);
        case 'get_translation_history':
          return Promise.resolve(baseHistoryEntries);
        case 'get_translation_profiles':
          return Promise.resolve(baseProfileStore);
        default:
          return Promise.resolve(undefined);
      }
    });

    renderPanel();

    expect(await screen.findByTestId('runtime-status-card')).toHaveTextContent('需要配置');
    await waitFor(() => {
      expect(screen.getByTestId('runtime-status-card')).toHaveTextContent('请保存 API Key');
    });
  });

  it('runs a provider test using the current unsaved settings', async () => {
    renderPanel();
    await openSection('provider');

    await fireEvent.input(await screen.findByPlaceholderText('sk-...'), {
      target: { value: 'sk-updated' },
    });
    await fireEvent.click(screen.getByTestId('probe-provider-button'));

    expect(invokeMock).toHaveBeenCalledWith(
      'probe_provider',
      expect.objectContaining({
        config: expect.objectContaining({ api_key: 'sk-updated' }),
      }),
    );
    expect(await screen.findByTestId('provider-probe-message')).toHaveTextContent('服务商测试成功');
  });

  it('loads the stored provider key when switching providers', async () => {
    renderPanel();
    await openSection('provider');

    const providerSelect = await screen.findByDisplayValue('DeepSeek (api.deepseek.com)');
    await fireEvent.change(providerSelect, { target: { value: 'openrouter' } });

    expect(invokeMock).toHaveBeenCalledWith('load_provider_api_key', {
      provider: 'openrouter',
    });
    await waitFor(() => {
      expect(screen.getByDisplayValue('sk-loaded')).toBeInTheDocument();
    });
  });

  it('creates and renames translation profiles', async () => {
    invokeMock.mockImplementation((command: string, payload?: { config?: typeof baseConfig; name?: string }) => {
      switch (command) {
        case 'get_system_capabilities':
          return Promise.resolve({
            aura_mode: 'ready',
            paste_back: 'ready',
          });
        case 'get_config':
          return Promise.resolve(baseConfig);
        case 'get_runtime_status':
          return Promise.resolve(readyStatus);
        case 'get_translation_history':
          return Promise.resolve(baseHistoryEntries);
        case 'get_translation_profiles':
          return Promise.resolve(baseProfileStore);
        case 'create_translation_profile':
          return Promise.resolve({
            active_profile_id: 'focus-jp',
            profiles: [
              {
                ...baseProfileStore.profiles[0],
                id: 'focus-jp',
                name: payload?.name ?? 'Focus JP',
                target_lang: payload?.config?.target_lang ?? 'Chinese',
              },
              ...baseProfileStore.profiles,
            ],
          });
        case 'rename_translation_profile':
          return Promise.resolve({
            active_profile_id: 'focus-jp',
            profiles: [
              { ...baseProfileStore.profiles[0], id: 'focus-jp', name: payload?.name ?? 'Renamed' },
              ...baseProfileStore.profiles,
            ],
          });
        default:
          return Promise.resolve(undefined);
      }
    });

    renderPanel();
    await openSection('profiles');

    await fireEvent.input(await screen.findByPlaceholderText('配置方案名称'), {
      target: { value: 'Focus JP' },
    });
    await fireEvent.click(screen.getByRole('button', { name: /另存为新方案/ }));
    expect(invokeMock).toHaveBeenCalledWith(
      'create_translation_profile',
      expect.objectContaining({
        name: 'Focus JP',
        config: expect.objectContaining({ active_profile_id: 'default' }),
      }),
    );

    await fireEvent.input(screen.getByPlaceholderText('配置方案名称'), {
      target: { value: 'Renamed' },
    });
    await fireEvent.click(screen.getByRole('button', { name: /重命名当前方案/ }));
    expect(invokeMock).toHaveBeenCalledWith('rename_translation_profile', {
      profileId: 'focus-jp',
      name: 'Renamed',
    });
  });

  it('retries, copies, deletes, and clears history entries from the history section', async () => {
    const firstView = renderPanel();
    await openSection('history');

    expect(await screen.findByTestId('history-list')).toHaveTextContent('Hello world');

    await fireEvent.click(screen.getByTestId('history-retry-history-1'));
    expect(invokeMock).toHaveBeenCalledWith('replay_translation_history_entry', {
      entryId: 'history-1',
    });

    await fireEvent.click(screen.getByTestId('history-copy-history-1'));
    expect(invokeMock).toHaveBeenCalledWith('copy_result_to_clipboard', {
      text: '你好，世界',
    });

    await fireEvent.click(screen.getByTestId('history-delete-history-1'));
    expect(invokeMock).toHaveBeenCalledWith('delete_translation_history_entry', {
      entryId: 'history-1',
    });

    firstView.unmount();

    renderPanel();
    await openSection('history');

    await fireEvent.click(screen.getByTestId('clear-all-button'));
    expect(invokeMock).not.toHaveBeenCalledWith('clear_translation_history');

    await fireEvent.click(screen.getByTestId('clear-confirm-commit'));
    expect(invokeMock).toHaveBeenCalledWith('clear_translation_history');
  });
});
