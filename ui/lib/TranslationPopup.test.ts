import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import TranslationPopup from './TranslationPopup.svelte';

describe('TranslationPopup', () => {
  it('shows the minimal idle hint', () => {
    render(TranslationPopup, {
      viewState: 'idle',
      translatedText: '',
      errorMessage: '',
      windowPinned: false,
      onTogglePinned: vi.fn(),
      oncancel: vi.fn(),
      ondismiss: vi.fn(),
      oncopy: vi.fn(),
    });

    expect(screen.getByText(/copy text to translate/i)).toBeInTheDocument();
  });

  it('shows pin state and notifies when the user toggles it', async () => {
    const onTogglePinned = vi.fn();

    render(TranslationPopup, {
      viewState: 'result',
      translatedText: '浣犲ソ',
      errorMessage: '',
      windowPinned: false,
      onTogglePinned,
      oncancel: vi.fn(),
      ondismiss: vi.fn(),
      oncopy: vi.fn(),
    });

    const pinButton = screen.getByRole('button', { name: /pin/i });
    expect(pinButton).toHaveAttribute('aria-pressed', 'false');

    await fireEvent.click(pinButton);
    expect(onTogglePinned).toHaveBeenCalledWith(true);
  });
});
