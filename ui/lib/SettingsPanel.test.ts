import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
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
  window_pinned: false,
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

  it('rejects bare single-key hotkeys during capture', async () => {
    invokeMock.mockResolvedValueOnce(baseConfig);

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
    invokeMock.mockResolvedValueOnce(baseConfig);

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
    invokeMock.mockResolvedValueOnce(baseConfig);
    invokeMock.mockResolvedValueOnce(undefined);
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

    expect(invokeMock).toHaveBeenLastCalledWith(
      'save_config',
      expect.objectContaining({
        config: expect.objectContaining({ window_pinned: true }),
      }),
    );
    expect(onsaved).toHaveBeenCalledWith(expect.objectContaining({ window_pinned: true }));
  });
});
