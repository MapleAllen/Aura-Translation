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

/**
 * Delivers a payload to a registered listener.
 *
 * The listener callbacks take a Tauri event envelope and read `event.payload`, so the value has to
 * be wrapped. Passing it raw made every handler bail at its own `payload?.trim()` guard, so these
 * tests passed without exercising anything past that line.
 */
function emit(event: string, payload?: unknown) {
  for (const entry of registered) {
    if (entry.event === event) entry.handler({ payload });
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

  it('keeps the empty entry point after a failed translation', async () => {    render(TranslationWindowView);
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
  it('releases the backend reservation when it abandons a trigger', async () => {
    // A successful emit only proves the event reached the webview. If this window cannot load a
    // configuration it never calls translate_text, so the reservation the backend took before
    // emitting has to be handed back; otherwise the next press for the same text collapses.
    render(TranslationWindowView);
    await screen.findByTestId('popup-status-bar');
    await vi.waitFor(() => {
      expect(registered.some((entry) => entry.event === 'trigger-translate')).toBe(true);
    });

    invokeMock.mockImplementation((command: string) => {
      if (command === 'get_config') return Promise.reject(new Error('config unreadable'));
      return Promise.resolve(undefined);
    });

    emit('trigger-translate', 'Hello world');

    await vi.waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('release_request_reservation', {
        text: 'Hello world',
      });
    });

    // And it must not have dispatched a translation.
    expect(invokeMock).not.toHaveBeenCalledWith('translate_text', expect.anything());
  });

  it('marks the UI ready only after every listener is registered', async () => {
    // Reporting readiness while listeners were still attaching let the backend emit a trigger into
    // a window with no listener yet, which left that request's reservation pending forever.
    // Held in an object so the closure assignment is not narrowed away by control-flow analysis.
    const gate: { release: null | (() => void) } = { release: null };

    listenMock.mockImplementation((event: string, handler: (payload: unknown) => void) => {
      const registration = new Promise<void>((resolve) => {
        if (event === 'daemon-error') {
          // Hold the final registration open so ordering can be observed.
          gate.release = () => {
            registered.push({ event, handler });
            resolve();
          };
          return;
        }
        registered.push({ event, handler });
        resolve();
      });
      return registration.then(() => () => {});
    });

    render(TranslationWindowView);
    await screen.findByTestId('popup-status-bar');

    await vi.waitFor(() => {
      expect(gate.release).not.toBeNull();
    });

    // The last listener is still pending, so readiness must not have been reported.
    expect(invokeMock).not.toHaveBeenCalledWith('mark_ui_ready');

    gate.release?.();

    await vi.waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('mark_ui_ready');
    });
  });
});
