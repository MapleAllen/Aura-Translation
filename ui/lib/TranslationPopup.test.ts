import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import TranslationPopup from './TranslationPopup.svelte';

describe('TranslationPopup', () => {
  it('shows the minimal idle hint', () => {
    render(TranslationPopup, {
      viewState: 'idle',
      sourceText: '',
      sourceLangLabel: '',
      targetLangLabel: '',
      providerLabel: '',
      modelLabel: '',
      retryAttempt: null,
      translatedText: '',
      errorMessage: '',
      canPasteBack: false,
      windowPinned: false,
      onTogglePinned: vi.fn(),
      onretry: vi.fn(),
      oncancel: vi.fn(),
      ondismiss: vi.fn(),
      oncopy: vi.fn(),
      onpasteback: vi.fn(),
    });

    expect(screen.getByText(/copy text to translate/i)).toBeInTheDocument();
  });

  it('shows pin state and notifies when the user toggles it', async () => {
    const onTogglePinned = vi.fn();

    render(TranslationPopup, {
      viewState: 'result',
      sourceText: 'Hello world',
      sourceLangLabel: 'English',
      targetLangLabel: 'Chinese',
      providerLabel: 'DeepSeek',
      modelLabel: 'deepseek-chat',
      retryAttempt: null,
      translatedText: 'Hello world translated',
      errorMessage: '',
      canPasteBack: true,
      windowPinned: false,
      onTogglePinned,
      onretry: vi.fn(),
      oncancel: vi.fn(),
      ondismiss: vi.fn(),
      oncopy: vi.fn(),
      onpasteback: vi.fn(),
    });

    const pinButton = screen.getByRole('button', { name: /pin/i });
    expect(pinButton).toHaveAttribute('aria-pressed', 'false');

    await fireEvent.click(pinButton);
    expect(onTogglePinned).toHaveBeenCalledWith(true);
  });

  it('shows request context and retries when asked', async () => {
    const onretry = vi.fn();

    render(TranslationPopup, {
      viewState: 'error',
      sourceText: 'Retry this translation',
      sourceLangLabel: 'English',
      targetLangLabel: 'Chinese',
      providerLabel: 'OpenRouter',
      modelLabel: 'mistralai/mistral-7b-instruct',
      retryAttempt: 2,
      translatedText: '',
      errorMessage: 'Translation failed.',
      canPasteBack: false,
      windowPinned: false,
      onTogglePinned: vi.fn(),
      onretry,
      oncancel: vi.fn(),
      ondismiss: vi.fn(),
      oncopy: vi.fn(),
      onpasteback: vi.fn(),
    });

    expect(screen.getByText(/source/i)).toBeInTheDocument();
    expect(screen.getByText(/english to chinese/i)).toBeInTheDocument();
    expect(screen.getByText(/retry 2\/3/i)).toBeInTheDocument();

    await fireEvent.click(screen.getByRole('button', { name: /retry translation/i }));
    expect(onretry).toHaveBeenCalledTimes(1);
  });

  it('offers a paste-back action when the source app is available', async () => {
    const onpasteback = vi.fn();

    render(TranslationPopup, {
      viewState: 'result',
      sourceText: 'Paste this back',
      sourceLangLabel: 'English',
      targetLangLabel: 'Chinese',
      providerLabel: 'DeepSeek',
      modelLabel: 'deepseek-chat',
      retryAttempt: null,
      translatedText: '把这个贴回去',
      errorMessage: '',
      canPasteBack: true,
      windowPinned: false,
      onTogglePinned: vi.fn(),
      onretry: vi.fn(),
      oncancel: vi.fn(),
      ondismiss: vi.fn(),
      oncopy: vi.fn(),
      onpasteback,
    });

    await fireEvent.click(screen.getByRole('button', { name: /paste translation back/i }));
    expect(onpasteback).toHaveBeenCalledTimes(1);
  });
});
