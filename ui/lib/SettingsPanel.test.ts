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
  summary: 'Aura is ready to translate with DeepSeek and model deepseek-chat.',
  can_translate_now: true,
  checklist: [
    { code: 'provider', label: 'Provider: DeepSeek', ok: true },
    { code: 'base_url', label: 'API base URL configured', ok: true },
    { code: 'model', label: 'Default model selected', ok: true },
    { code: 'api_key', label: 'API key saved', ok: true },
  ],
};

const needsSetupStatus = {
  level: 'needs_setup',
  summary: 'Save an API key, or switch to Ollama if you want a local provider.',
  can_translate_now: false,
  checklist: [
    { code: 'provider', label: 'Provider: DeepSeek', ok: true },
    { code: 'base_url', label: 'API base URL configured', ok: true },
    { code: 'model', label: 'Default model selected', ok: true },
    { code: 'api_key', label: 'API key saved', ok: false },
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
    status: 'success',
    created_at_ms: 1_717_171_717_000,
  },
];

describe('SettingsPanel', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockImplementation((command: string, payload?: { provider?: string; config?: typeof baseConfig }) => {
      switch (command) {
        case 'get_config':
          return Promise.resolve(baseConfig);
        case 'get_runtime_status':
          return Promise.resolve(readyStatus);
        case 'get_translation_history':
          return Promise.resolve(baseHistoryEntries);
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
            message: `Provider test succeeded for ${payload?.config?.provider ?? 'deepseek'}.`,
          });
        case 'load_provider_api_key':
          return Promise.resolve('sk-loaded');
        case 'replay_translation_history_entry':
          return Promise.resolve(undefined);
        case 'delete_translation_history_entry':
          return Promise.resolve([]);
        case 'clear_translation_history':
          return Promise.resolve([]);
        default:
          return Promise.resolve(undefined);
      }
    });
  });

  it('shows secure API key storage note and inline hotkey conflict for authenticated providers', async () => {
    render(SettingsPanel, {
      visible: true,
      onclose: () => {},
      hotkeyConflictMessage: 'Could not register "Alt+Shift+T": already in use.',
    });

    expect(await screen.findByTestId('system-api-key-storage-note')).toHaveTextContent(
      'OS credential store',
    );
    expect(screen.getByTestId('runtime-status-card')).toHaveTextContent('Aura is ready to translate');
    expect(screen.getByTestId('hotkey-conflict-inline')).toHaveTextContent(
      'Could not register "Alt+Shift+T"',
    );
    expect(screen.getByTestId('history-list')).toHaveTextContent('Hello world');
    expect(invokeMock).toHaveBeenCalledWith('get_config');
    expect(invokeMock).toHaveBeenCalledWith('get_runtime_status');
    expect(invokeMock).toHaveBeenCalledWith('get_translation_history');
  });

  it('hides API key storage messaging for ollama', async () => {
    invokeMock.mockImplementation((command: string, payload?: { provider?: string }) => {
      switch (command) {
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
            summary: 'Aura is ready to translate with Ollama and model qwen2.5.',
          });
        case 'get_translation_history':
          return Promise.resolve(baseHistoryEntries);
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

    render(SettingsPanel, {
      visible: true,
      onclose: () => {},
      hotkeyConflictMessage: '',
    });

    await waitFor(() => {
      expect(screen.queryByTestId('plaintext-api-key-warning')).not.toBeInTheDocument();
      expect(screen.queryByTestId('system-api-key-storage-note')).not.toBeInTheDocument();
    });
  });

  it('switches to plaintext fallback storage and saves the selection', async () => {
    render(SettingsPanel, {
      visible: true,
      onclose: () => {},
      hotkeyConflictMessage: '',
    });

    const storageSelect = await screen.findByLabelText(/api key storage/i);
    await fireEvent.change(storageSelect, { target: { value: 'plaintext_fallback' } });

    expect(screen.getByTestId('plaintext-api-key-warning')).toHaveTextContent(
      'written to the local Aura config',
    );

    await fireEvent.click(screen.getByRole('button', { name: /save settings/i }));

    expect(invokeMock).toHaveBeenCalledWith(
      'save_config',
      expect.objectContaining({
        config: expect.objectContaining({ api_key_storage: 'plaintext_fallback' }),
      }),
    );
  });

  it('rejects bare single-key hotkeys during capture', async () => {
    render(SettingsPanel, {
      visible: true,
      onclose: () => {},
      hotkeyConflictMessage: '',
    });

    const input = await screen.findByDisplayValue('CmdOrCtrl+T');
    await fireEvent.focus(input);
    await fireEvent.keyDown(input, { key: 't' });

    expect(screen.getByTestId('hotkey-input-message')).toHaveTextContent(
      'Include at least one modifier key.',
    );
    expect(input).toHaveValue('CmdOrCtrl+T');
  });

  it('captures modifier-based hotkeys and clears the inline validation message', async () => {
    render(SettingsPanel, {
      visible: true,
      onclose: () => {},
      hotkeyConflictMessage: '',
    });

    const input = await screen.findByDisplayValue('CmdOrCtrl+T');
    await fireEvent.focus(input);
    await fireEvent.keyDown(input, { key: 't' });
    await fireEvent.keyDown(input, { key: 'k', ctrlKey: true });

    expect(screen.queryByTestId('hotkey-input-message')).not.toBeInTheDocument();
    expect(input).toHaveValue('CmdOrCtrl+K');
  });

  it('loads and saves the window pin preference', async () => {
    const onsaved = vi.fn();

    render(SettingsPanel, {
      visible: true,
      onclose: () => {},
      onsaved,
      hotkeyConflictMessage: '',
    });

    const pinSwitch = await screen.findByRole('switch', { name: /pin window/i });
    expect(pinSwitch).toHaveAttribute('aria-checked', 'false');

    await fireEvent.click(pinSwitch);
    expect(pinSwitch).toHaveAttribute('aria-checked', 'true');

    await fireEvent.click(screen.getByRole('button', { name: /save settings/i }));

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

  it('loads and saves the aura mode preference', async () => {
    render(SettingsPanel, {
      visible: true,
      onclose: () => {},
      hotkeyConflictMessage: '',
    });

    const auraSwitch = await screen.findByRole('switch', { name: /aura mode/i });
    expect(auraSwitch).toHaveAttribute('aria-checked', 'false');

    await fireEvent.click(auraSwitch);
    expect(auraSwitch).toHaveAttribute('aria-checked', 'true');

    await fireEvent.click(screen.getByRole('button', { name: /save settings/i }));

    expect(invokeMock).toHaveBeenCalledWith(
      'save_config',
      expect.objectContaining({
        config: expect.objectContaining({ aura_mode_enabled: true }),
      }),
    );
  });

  it('loads and saves the sensitive clipboard guard preference', async () => {
    render(SettingsPanel, {
      visible: true,
      onclose: () => {},
      hotkeyConflictMessage: '',
    });

    const guardSwitch = await screen.findByRole('switch', { name: /sensitive clipboard guard/i });
    expect(guardSwitch).toHaveAttribute('aria-checked', 'true');

    await fireEvent.click(guardSwitch);
    expect(guardSwitch).toHaveAttribute('aria-checked', 'false');

    await fireEvent.click(screen.getByRole('button', { name: /save settings/i }));

    expect(invokeMock).toHaveBeenCalledWith(
      'save_config',
      expect.objectContaining({
        config: expect.objectContaining({ aura_guard_enabled: false }),
      }),
    );
  });

  it('shows setup guidance when the saved config is not ready', async () => {
    invokeMock.mockImplementation((command: string) => {
      switch (command) {
        case 'get_config':
          return Promise.resolve({
            ...baseConfig,
            api_key: '',
          });
        case 'get_runtime_status':
          return Promise.resolve(needsSetupStatus);
        case 'get_translation_history':
          return Promise.resolve(baseHistoryEntries);
        default:
          return Promise.resolve(undefined);
      }
    });

    render(SettingsPanel, {
      visible: true,
      onclose: () => {},
      hotkeyConflictMessage: '',
    });

    expect(await screen.findByTestId('runtime-status-card')).toHaveTextContent('Action needed');
    expect(screen.getByTestId('runtime-status-card')).toHaveTextContent(
      'Save an API key, or switch to Ollama if you want a local provider.',
    );
  });

  it('runs a provider test using the current unsaved settings', async () => {
    render(SettingsPanel, {
      visible: true,
      onclose: () => {},
      hotkeyConflictMessage: '',
    });

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
    expect(await screen.findByTestId('provider-probe-message')).toHaveTextContent(
      'Provider test succeeded',
    );
  });

  it('loads the stored provider key when switching providers', async () => {
    render(SettingsPanel, {
      visible: true,
      onclose: () => {},
      hotkeyConflictMessage: '',
    });

    const providerSelect = await screen.findByDisplayValue('DeepSeek (api.deepseek.com)');
    await fireEvent.change(providerSelect, { target: { value: 'openrouter' } });

    expect(invokeMock).toHaveBeenCalledWith('load_provider_api_key', {
      provider: 'openrouter',
    });
    await waitFor(() => {
      expect(screen.getByDisplayValue('sk-loaded')).toBeInTheDocument();
    });
  });

  it('retries, copies, deletes, and clears history entries', async () => {
    render(SettingsPanel, {
      visible: true,
      onclose: () => {},
      hotkeyConflictMessage: '',
    });

    expect(await screen.findByTestId('history-list')).toHaveTextContent('Hello world');

    await fireEvent.click(screen.getByRole('button', { name: 'Retry' }));
    expect(invokeMock).toHaveBeenCalledWith('replay_translation_history_entry', {
      entryId: 'history-1',
    });

    await fireEvent.click(screen.getByRole('button', { name: 'Copy' }));
    expect(invokeMock).toHaveBeenCalledWith('copy_result_to_clipboard', {
      text: '你好，世界',
    });

    await fireEvent.click(screen.getByRole('button', { name: 'Delete' }));
    expect(invokeMock).toHaveBeenCalledWith('delete_translation_history_entry', {
      entryId: 'history-1',
    });

    invokeMock.mockImplementation((command: string) => {
      switch (command) {
        case 'get_config':
          return Promise.resolve(baseConfig);
        case 'get_runtime_status':
          return Promise.resolve(readyStatus);
        case 'get_translation_history':
          return Promise.resolve(baseHistoryEntries);
        case 'clear_translation_history':
          return Promise.resolve([]);
        default:
          return Promise.resolve(undefined);
      }
    });

    render(SettingsPanel, {
      visible: true,
      onclose: () => {},
      hotkeyConflictMessage: '',
    });

    await fireEvent.click(await screen.findByRole('button', { name: /clear all/i }));
    expect(invokeMock).toHaveBeenCalledWith('clear_translation_history');
  });
});
