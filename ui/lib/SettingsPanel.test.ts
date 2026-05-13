import { render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import SettingsPanel from './SettingsPanel.svelte';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

const baseConfig = {
  api_key: 'sk-test',
  model: 'deepseek-chat',
  source_lang: 'auto',
  target_lang: 'Chinese',
  hotkey: 'CmdOrCtrl+T',
  provider: 'deepseek',
  api_base_url: 'https://api.deepseek.com',
  available_models: ['deepseek-chat', 'deepseek-reasoner'],
};

describe('SettingsPanel', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('shows plaintext API key warning and inline hotkey conflict for authenticated providers', async () => {
    invokeMock.mockResolvedValueOnce(baseConfig);

    render(SettingsPanel, {
      visible: true,
      onclose: () => {},
      hotkeyConflictMessage: 'Could not register "Alt+Shift+T": already in use.',
    });

    expect(await screen.findByTestId('plaintext-api-key-warning')).toHaveTextContent(
      'stored in plaintext',
    );
    expect(screen.getByTestId('hotkey-conflict-inline')).toHaveTextContent(
      'Could not register "Alt+Shift+T"',
    );
    expect(invokeMock).toHaveBeenCalledWith('get_config');
  });

  it('hides plaintext API key warning for ollama', async () => {
    invokeMock.mockResolvedValueOnce({
      ...baseConfig,
      api_key: '',
      model: 'qwen2.5',
      provider: 'ollama',
      api_base_url: 'http://localhost:11434',
      available_models: ['qwen2.5'],
    });

    render(SettingsPanel, {
      visible: true,
      onclose: () => {},
      hotkeyConflictMessage: '',
    });

    await waitFor(() => {
      expect(screen.queryByTestId('plaintext-api-key-warning')).not.toBeInTheDocument();
    });
  });
});
