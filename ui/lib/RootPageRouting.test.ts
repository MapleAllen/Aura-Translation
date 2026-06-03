import { render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import Page from '../routes/+page.svelte';

let windowLabel: 'translation' | 'settings' = 'translation';

const invokeMock = vi.fn();
const listenMock = vi.fn();
const onMovedMock = vi.fn();
const onResizedMock = vi.fn();

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({
    label: windowLabel,
    close: vi.fn(),
    hide: vi.fn(),
    onMoved: onMovedMock,
    onResized: onResizedMock,
    outerPosition: vi.fn(),
    outerSize: vi.fn(),
    scaleFactor: vi.fn(),
    setSize: vi.fn(),
    startResizeDragging: vi.fn(),
  }),
  LogicalSize: class LogicalSize {
    constructor(
      public width: number,
      public height: number,
    ) {}
  },
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: (...args: unknown[]) => listenMock(...args),
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
  checklist: [],
};

const profileStore = {
  active_profile_id: 'default',
  profiles: [
    {
      id: 'default',
      name: 'Default',
      provider: 'deepseek',
      model: 'deepseek-chat',
      source_lang: 'auto',
      target_lang: 'Chinese',
      api_key_storage: 'system',
      api_base_url: 'https://api.deepseek.com',
      available_models: ['deepseek-chat', 'deepseek-reasoner'],
      api_key: '',
    },
  ],
};

describe('root page window routing', () => {
  beforeEach(() => {
    windowLabel = 'translation';
    invokeMock.mockReset();
    listenMock.mockReset();
    onMovedMock.mockReset();
    onResizedMock.mockReset();

    listenMock.mockResolvedValue(() => {});
    onMovedMock.mockResolvedValue(() => {});
    onResizedMock.mockResolvedValue(() => {});
    invokeMock.mockImplementation((command: string) => {
      switch (command) {
        case 'get_config':
          return Promise.resolve(baseConfig);
        case 'get_paste_back_status':
          return Promise.resolve({ supported: true, available: false });
        case 'get_runtime_status':
          return Promise.resolve(readyStatus);
        case 'get_translation_history':
          return Promise.resolve([]);
        case 'get_translation_profiles':
          return Promise.resolve(profileStore);
        case 'mark_ui_ready':
          return Promise.resolve(undefined);
        default:
          return Promise.resolve(undefined);
      }
    });
  });

  it('does not mount the translation popup while bootstrapping the settings window', async () => {
    windowLabel = 'settings';

    render(Page);

    expect(screen.queryByTestId('popup-status-bar')).not.toBeInTheDocument();
    expect(await screen.findByTestId('settings-workspace')).toBeInTheDocument();

    await waitFor(() => {
      const readyCalls = invokeMock.mock.calls.filter(([command]) => command === 'mark_ui_ready');
      expect(readyCalls).toHaveLength(1);
    });
  });
});
