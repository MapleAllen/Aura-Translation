import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import TranslationPopup from './TranslationPopup.svelte';

describe('TranslationPopup', () => {
  it('shows the minimal idle hint', () => {
    render(TranslationPopup, {
      viewState: 'idle',
      sourceText: '',
      draftSourceText: '',
      sourceLangLabel: '',
      targetLangLabel: '',
      providerLabel: '',
      modelLabel: '',
      retryAttempt: null,
      translatedText: '',
      errorMessage: '',
      showComposer: false,
      hasDraftChanges: false,
      usage: null,
      canPasteBack: false,
      windowPinned: false,
      ondraftsourcechange: vi.fn(),
      ontranslatedraft: vi.fn(),
      onresetdraft: vi.fn(),
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
      draftSourceText: 'Hello world',
      sourceLangLabel: 'English',
      targetLangLabel: 'Chinese',
      providerLabel: 'DeepSeek',
      modelLabel: 'deepseek-chat',
      retryAttempt: null,
      translatedText: 'Hello world translated',
      errorMessage: '',
      showComposer: false,
      hasDraftChanges: false,
      usage: {
        prompt_tokens: 9,
        completion_tokens: 6,
        total_tokens: 15,
      },
      canPasteBack: true,
      windowPinned: false,
      ondraftsourcechange: vi.fn(),
      ontranslatedraft: vi.fn(),
      onresetdraft: vi.fn(),
      onTogglePinned,
      onretry: vi.fn(),
      oncancel: vi.fn(),
      ondismiss: vi.fn(),
      oncopy: vi.fn(),
      onpasteback: vi.fn(),
    });

    const pinButton = screen.getByRole('button', { name: /pin/i });
    expect(pinButton).toHaveAttribute('aria-pressed', 'false');
    expect(screen.getByText(/15 tokens/i)).toBeInTheDocument();

    await fireEvent.click(pinButton);
    expect(onTogglePinned).toHaveBeenCalledWith(true);
  });

  it('shows request context and retries when asked', async () => {
    const onretry = vi.fn();

    render(TranslationPopup, {
      viewState: 'error',
      sourceText: 'Retry this translation',
      draftSourceText: 'Retry this translation',
      sourceLangLabel: 'English',
      targetLangLabel: 'Chinese',
      providerLabel: 'OpenRouter',
      modelLabel: 'mistralai/mistral-7b-instruct',
      retryAttempt: 2,
      translatedText: '',
      errorMessage: 'Translation failed.',
      showComposer: false,
      hasDraftChanges: false,
      usage: null,
      canPasteBack: false,
      windowPinned: false,
      ondraftsourcechange: vi.fn(),
      ontranslatedraft: vi.fn(),
      onresetdraft: vi.fn(),
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

  it('renders a prominent Try again button in the error state', async () => {
    const onretry = vi.fn();

    render(TranslationPopup, {
      viewState: 'error',
      sourceText: 'Hello world',
      draftSourceText: 'Hello world',
      sourceLangLabel: 'English',
      targetLangLabel: 'Chinese',
      providerLabel: 'DeepSeek',
      modelLabel: 'deepseek-chat',
      retryAttempt: null,
      translatedText: '',
      errorMessage: 'Network error: connection refused',
      showComposer: false,
      hasDraftChanges: false,
      usage: null,
      canPasteBack: false,
      windowPinned: false,
      ondraftsourcechange: vi.fn(),
      ontranslatedraft: vi.fn(),
      onresetdraft: vi.fn(),
      onTogglePinned: vi.fn(),
      onretry,
      oncancel: vi.fn(),
      ondismiss: vi.fn(),
      oncopy: vi.fn(),
      onpasteback: vi.fn(),
    });

    const tryAgain = screen.getByTestId('try-again-button');
    expect(tryAgain).toBeInTheDocument();
    expect(tryAgain).toHaveTextContent(/try again/i);
    expect(tryAgain).not.toBeDisabled();

    await fireEvent.click(tryAgain);
    expect(onretry).toHaveBeenCalledTimes(1);
  });

  it('disables the Try again button when there is no source text to retry', () => {
    render(TranslationPopup, {
      viewState: 'error',
      sourceText: '',
      draftSourceText: '',
      sourceLangLabel: '',
      targetLangLabel: '',
      providerLabel: '',
      modelLabel: '',
      retryAttempt: null,
      translatedText: '',
      errorMessage: 'Translation failed.',
      showComposer: false,
      hasDraftChanges: false,
      usage: null,
      canPasteBack: false,
      windowPinned: false,
      ondraftsourcechange: vi.fn(),
      ontranslatedraft: vi.fn(),
      onresetdraft: vi.fn(),
      onTogglePinned: vi.fn(),
      onretry: vi.fn(),
      oncancel: vi.fn(),
      ondismiss: vi.fn(),
      oncopy: vi.fn(),
      onpasteback: vi.fn(),
    });

    expect(screen.getByTestId('try-again-button')).toBeDisabled();
  });

  it('offers a paste-back action when the source app is available', async () => {
    const onpasteback = vi.fn();

    render(TranslationPopup, {
      viewState: 'result',
      sourceText: 'Paste this back',
      draftSourceText: 'Paste this back',
      sourceLangLabel: 'English',
      targetLangLabel: 'Chinese',
      providerLabel: 'DeepSeek',
      modelLabel: 'deepseek-chat',
      retryAttempt: null,
      translatedText: '把这个贴回去',
      errorMessage: '',
      showComposer: false,
      hasDraftChanges: false,
      usage: null,
      canPasteBack: true,
      windowPinned: false,
      ondraftsourcechange: vi.fn(),
      ontranslatedraft: vi.fn(),
      onresetdraft: vi.fn(),
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

  it('disables paste-back on macOS/Linux with an explanatory tooltip', () => {
    render(TranslationPopup, {
      viewState: 'result',
      sourceText: 'Paste this back',
      draftSourceText: 'Paste this back',
      sourceLangLabel: 'English',
      targetLangLabel: 'Chinese',
      providerLabel: 'DeepSeek',
      modelLabel: 'deepseek-chat',
      retryAttempt: null,
      translatedText: '把这个贴回去',
      errorMessage: '',
      showComposer: false,
      hasDraftChanges: false,
      usage: null,
      canPasteBack: false,
      pasteBackSupported: false,
      pasteBackAvailable: false,
      windowPinned: false,
      ondraftsourcechange: vi.fn(),
      ontranslatedraft: vi.fn(),
      onresetdraft: vi.fn(),
      onTogglePinned: vi.fn(),
      onretry: vi.fn(),
      oncancel: vi.fn(),
      ondismiss: vi.fn(),
      oncopy: vi.fn(),
      onpasteback: vi.fn(),
    });

    const button = screen.getByRole('button', { name: /paste translation back/i });
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute('title', 'Paste-back is only available on Windows.');
  });

  it('disables paste-back on Windows when no foreground source app is captured', () => {
    render(TranslationPopup, {
      viewState: 'result',
      sourceText: 'Paste this back',
      draftSourceText: 'Paste this back',
      sourceLangLabel: 'English',
      targetLangLabel: 'Chinese',
      providerLabel: 'DeepSeek',
      modelLabel: 'deepseek-chat',
      retryAttempt: null,
      translatedText: '把这个贴回去',
      errorMessage: '',
      showComposer: false,
      hasDraftChanges: false,
      usage: null,
      canPasteBack: false,
      pasteBackSupported: true,
      pasteBackAvailable: false,
      windowPinned: false,
      ondraftsourcechange: vi.fn(),
      ontranslatedraft: vi.fn(),
      onresetdraft: vi.fn(),
      onTogglePinned: vi.fn(),
      onretry: vi.fn(),
      oncancel: vi.fn(),
      ondismiss: vi.fn(),
      oncopy: vi.fn(),
      onpasteback: vi.fn(),
    });

    const button = screen.getByRole('button', { name: /paste translation back/i });
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute(
      'title',
      'Copy text from a foreground app first, then paste-back will be available.',
    );
  });

  it('disables paste-back while no translated text is ready', () => {
    render(TranslationPopup, {
      viewState: 'streaming',
      sourceText: 'Hello world',
      draftSourceText: 'Hello world',
      sourceLangLabel: 'English',
      targetLangLabel: 'Chinese',
      providerLabel: 'DeepSeek',
      modelLabel: 'deepseek-chat',
      retryAttempt: null,
      translatedText: '',
      errorMessage: '',
      showComposer: false,
      hasDraftChanges: false,
      usage: null,
      canPasteBack: true,
      pasteBackSupported: true,
      pasteBackAvailable: true,
      windowPinned: false,
      ondraftsourcechange: vi.fn(),
      ontranslatedraft: vi.fn(),
      onresetdraft: vi.fn(),
      onTogglePinned: vi.fn(),
      onretry: vi.fn(),
      oncancel: vi.fn(),
      ondismiss: vi.fn(),
      oncopy: vi.fn(),
      onpasteback: vi.fn(),
    });

    expect(
      screen.queryByRole('button', { name: /paste translation back/i }),
    ).not.toBeInTheDocument();
  });

  it('hides paste-back when the result is empty (idle/idle-ready view)', () => {
    render(TranslationPopup, {
      viewState: 'idle',
      sourceText: '',
      draftSourceText: '',
      sourceLangLabel: '',
      targetLangLabel: '',
      providerLabel: '',
      modelLabel: '',
      retryAttempt: null,
      translatedText: '',
      errorMessage: '',
      showComposer: false,
      hasDraftChanges: false,
      usage: null,
      canPasteBack: true,
      pasteBackSupported: true,
      pasteBackAvailable: true,
      windowPinned: false,
      ondraftsourcechange: vi.fn(),
      ontranslatedraft: vi.fn(),
      onresetdraft: vi.fn(),
      onTogglePinned: vi.fn(),
      onretry: vi.fn(),
      oncancel: vi.fn(),
      ondismiss: vi.fn(),
      oncopy: vi.fn(),
      onpasteback: vi.fn(),
    });

    expect(
      screen.queryByRole('button', { name: /paste translation back/i }),
    ).not.toBeInTheDocument();
  });

  it('shows a pinned draft composer with translate and reset actions', async () => {
    const ondraftsourcechange = vi.fn();
    const ontranslatedraft = vi.fn();
    const onresetdraft = vi.fn();

    render(TranslationPopup, {
      viewState: 'result',
      sourceText: 'Original text',
      draftSourceText: 'Edited text',
      sourceLangLabel: 'English',
      targetLangLabel: 'Chinese',
      providerLabel: 'DeepSeek',
      modelLabel: 'deepseek-chat',
      retryAttempt: null,
      translatedText: '编辑后的翻译',
      errorMessage: '',
      showComposer: true,
      hasDraftChanges: true,
      usage: null,
      canPasteBack: false,
      windowPinned: true,
      ondraftsourcechange,
      ontranslatedraft,
      onresetdraft,
      onTogglePinned: vi.fn(),
      onretry: vi.fn(),
      oncancel: vi.fn(),
      ondismiss: vi.fn(),
      oncopy: vi.fn(),
      onpasteback: vi.fn(),
    });

    await fireEvent.input(screen.getByPlaceholderText(/type or revise source text here/i), {
      target: { value: 'Edited again' },
    });
    expect(ondraftsourcechange).toHaveBeenCalledWith('Edited again');

    await fireEvent.click(screen.getByRole('button', { name: /translate edits/i }));
    expect(ontranslatedraft).toHaveBeenCalledTimes(1);

    await fireEvent.click(screen.getByRole('button', { name: /reset/i }));
    expect(onresetdraft).toHaveBeenCalledTimes(1);
  });
});
