import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import TranslationPopup from './TranslationPopup.svelte';

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  writeText: vi.fn(),
}));

describe('TranslationPopup', () => {
  it('shows the configured hotkey in the idle hint', () => {
    render(TranslationPopup, {
      viewState: 'idle',
      sourceText: '',
      translatedText: '',
      errorMessage: '',
      hotkeyLabel: 'Alt+Shift+T',
      sourceLang: 'auto',
      targetLang: 'Chinese',
      windowPinned: false,
      onLanguageChange: vi.fn(),
      onTogglePinned: vi.fn(),
      oncancel: vi.fn(),
      ondismiss: vi.fn(),
    });

    expect(screen.getByText(/copy text, press alt\+shift\+t/i)).toBeInTheDocument();
  });

  it('shows pin state and notifies when the user toggles it', async () => {
    const onTogglePinned = vi.fn();

    render(TranslationPopup, {
      viewState: 'result',
      sourceText: 'Hello',
      translatedText: '你好',
      errorMessage: '',
      hotkeyLabel: 'Alt+Shift+T',
      sourceLang: 'English',
      targetLang: 'Chinese',
      windowPinned: false,
      onLanguageChange: vi.fn(),
      onTogglePinned,
      oncancel: vi.fn(),
      ondismiss: vi.fn(),
    });

    const pinButton = screen.getByRole('button', { name: /pin/i });
    expect(pinButton).toHaveAttribute('aria-pressed', 'false');

    await fireEvent.click(pinButton);
    expect(onTogglePinned).toHaveBeenCalledWith(true);
  });
});
