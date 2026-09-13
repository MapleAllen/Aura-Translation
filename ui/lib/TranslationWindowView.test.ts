import { fireEvent, render, screen } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import TranslationWindowView from './TranslationWindowView.svelte';

const invokeMock = vi.fn();
const listenMock = vi.fn();

type Listener = { event: string; handler: (payload: unknown) => void };

let registered: Listener[] = [];

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

function emit(event: string, payload?: unknown) {
  for (const entry of registered) {
    if (entry.event === event) entry.handler(payload);
  }
}

vi.mock('@tauri-apps/api/event', () => ({
  listen: (...args: unknown[]) => listenMock(...args),
}));

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({
    label: 'translation',
    hide: vi.fn(),
    onMoved: vi.fn().mockResolvedValue(() => {}),
    onResized: vi.fn().mockResolvedValue(() => {}),
    outerPosition: vi.fn().mockResolvedValue({ x: 0, y: 0 }),
    outerSize: vi.fn().mockResolvedValue({ width: 392, height: 200 }),
    scaleFactor: vi.fn().mockResolvedValue(1),
    setSize: vi.fn(),
    innerSize: vi.fn().mockResolvedValue({ width: 392, height: 200 }),
    startResizeDragging: vi.fn(),
  }),
  LogicalSize: class LogicalSize {
    constructor(
      public width: number,
      public height: number,
    ) {}
  },
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
  available_models: ['deepseek-chat'],
  settings_window_placement: null,
  pinned_translation_placement: null,
  setup_completed: true,
  notifications_enabled: true,
};

describe('translation window empty entry', () => {
  beforeEach(() => {
    registered = [];
    invokeMock.mockReset();
    listenMock.mockReset();

    invokeMock.mockImplementation((command: string) => {
      switch (command) {
        case 'get_config':
          return Promise.resolve(baseConfig);
        case 'mark_ui_ready':
          return Promise.resolve(undefined);
        default:
          return Promise.reject(new Error(`unmocked command in test: ${command}`));
      }
    });

    listenMock.mockImplementation((event: string, handler: (payload: unknown) => void) => {
      registered.push({ event, handler });
      return Promise.resolve(() => {});
    });
  });

  it('opens an editable input when the clipboard was empty', async () => {
    // Regression: this event used to clear both source fields, and the composer was derived from
    // their presence, so the window opened with only a hint and no way to type anything.
    render(TranslationWindowView);

    // Let the mount-time listeners register.
    await screen.findByTestId('popup-status-bar');
    await vi.waitFor(() => {
      expect(registered.some((entry) => entry.event === 'show-empty-translation')).toBe(true);
    });

    emit('show-empty-translation');

    const textarea = await screen.findByPlaceholderText('在这里输入或修改原文');
    expect(textarea).toBeInTheDocument();
    expect(textarea).toHaveValue('');
  });

  it('keeps the empty entry point after a failed translation', async () => {
    render(TranslationWindowView);
    await screen.findByTestId('popup-status-bar');
    await vi.waitFor(() => {
      expect(registered.some((entry) => entry.event === 'show-empty-translation')).toBe(true);
    });

    emit('show-empty-translation');

    const textarea = await screen.findByPlaceholderText('在这里输入或修改原文');
    await fireEvent.input(textarea, { target: { value: 'Hello world' } });

    // No api key in this fixture, so the request fails before it is dispatched.
    invokeMock.mockImplementation((command: string) => {
      if (command === 'get_config') return Promise.resolve({ ...baseConfig, api_key: '' });
      return Promise.resolve(undefined);
    });

    const retranslate = screen.getByRole('button', { name: /重新翻译全文/ });
    await fireEvent.click(retranslate);

    // The composer must survive, otherwise the user has no way to correct the input.
    expect(await screen.findByPlaceholderText('在这里输入或修改原文')).toBeInTheDocument();
  });
});
