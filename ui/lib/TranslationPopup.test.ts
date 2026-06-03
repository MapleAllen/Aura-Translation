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

    expect(screen.getByText(/复制要翻译的文本/)).toBeInTheDocument();
  });

  it('shows pin state and notifies when the user toggles it', async () => {
    const onTogglePinned = vi.fn();

    render(TranslationPopup, {
      viewState: 'result',
      sourceText: 'Hello world',
      draftSourceText: 'Hello world',
      sourceLangLabel: '英语',
      targetLangLabel: '中文',
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

    const pinButton = screen.getByRole('button', { name: '固定' });
    expect(pinButton).toHaveAttribute('aria-pressed', 'false');
    expect(screen.getByText(/15 token/i)).toBeInTheDocument();
    expect(screen.getByTestId('window-drag-handle')).toBeInTheDocument();

    await fireEvent.click(pinButton);
    expect(onTogglePinned).toHaveBeenCalledWith(true);
  });

  it('renders the translated result text in the result state', () => {
    render(TranslationPopup, {
      viewState: 'result',
      sourceText: 'Render this result',
      draftSourceText: 'Render this result',
      sourceLangLabel: '英语',
      targetLangLabel: '中文',
      providerLabel: 'DeepSeek',
      modelLabel: 'deepseek-chat',
      retryAttempt: null,
      translatedText: '请把这段结果显示出来。',
      errorMessage: '',
      showComposer: false,
      hasDraftChanges: false,
      usage: {
        prompt_tokens: 11,
        completion_tokens: 8,
        total_tokens: 19,
      },
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
      onpasteback: vi.fn(),
    });

    expect(screen.getByText('请把这段结果显示出来。')).toBeInTheDocument();
    expect(screen.getByText(/总计 19/i)).toBeInTheDocument();
  });

  it('shows request context and retries when asked', async () => {
    const onretry = vi.fn();

    render(TranslationPopup, {
      viewState: 'error',
      sourceText: 'Retry this translation',
      draftSourceText: 'Retry this translation',
      sourceLangLabel: '英语',
      targetLangLabel: '中文',
      providerLabel: 'OpenRouter',
      modelLabel: 'mistralai/mistral-7b-instruct',
      retryAttempt: 2,
      translatedText: '',
      errorMessage: '翻译失败。',
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

    expect(screen.getByText(/原文/)).toBeInTheDocument();
    expect(screen.getByText(/英语 → 中文/)).toBeInTheDocument();
    expect(screen.getByText(/第 2\/3 次重试/)).toBeInTheDocument();

    await fireEvent.click(screen.getByRole('button', { name: /重新翻译/ }));
    expect(onretry).toHaveBeenCalledTimes(1);
  });

  it('renders a prominent Try again button in the error state', async () => {
    const onretry = vi.fn();

    render(TranslationPopup, {
      viewState: 'error',
      sourceText: 'Hello world',
      draftSourceText: 'Hello world',
      sourceLangLabel: '英语',
      targetLangLabel: '中文',
      providerLabel: 'DeepSeek',
      modelLabel: 'deepseek-chat',
      retryAttempt: null,
      translatedText: '',
      errorMessage: '网络错误：连接被拒绝',
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
    expect(tryAgain).toHaveTextContent('重试');
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
      errorMessage: '翻译失败。',
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
      sourceLangLabel: '英语',
      targetLangLabel: '中文',
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

    await fireEvent.click(screen.getByRole('button', { name: /回填译文/ }));
    expect(onpasteback).toHaveBeenCalledTimes(1);
  });

  it('copies the translated result when the copy action is clicked', async () => {
    const oncopy = vi.fn();

    render(TranslationPopup, {
      viewState: 'result',
      sourceText: 'Copy this result',
      draftSourceText: 'Copy this result',
      sourceLangLabel: '英语',
      targetLangLabel: '中文',
      providerLabel: 'DeepSeek',
      modelLabel: 'deepseek-chat',
      retryAttempt: null,
      translatedText: '请复制这条翻译结果。',
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
      oncopy,
      onpasteback: vi.fn(),
    });

    await fireEvent.click(screen.getByRole('button', { name: /复制译文/ }));
    expect(oncopy).toHaveBeenCalledTimes(1);
  });

  it('disables paste-back on macOS/Linux with an explanatory tooltip', () => {
    render(TranslationPopup, {
      viewState: 'result',
      sourceText: 'Paste this back',
      draftSourceText: 'Paste this back',
      sourceLangLabel: '英语',
      targetLangLabel: '中文',
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

    const button = screen.getByRole('button', { name: /回填译文/ });
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute('title', '回填功能目前仅支持 Windows。');
  });

  it('disables paste-back on Windows when no foreground source app is captured', () => {
    render(TranslationPopup, {
      viewState: 'result',
      sourceText: 'Paste this back',
      draftSourceText: 'Paste this back',
      sourceLangLabel: '英语',
      targetLangLabel: '中文',
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

    const button = screen.getByRole('button', { name: /回填译文/ });
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute(
      'title',
      '请先从前台应用复制文本，Aura 才能回填。',
    );
  });

  it('disables paste-back while no translated text is ready', () => {
    render(TranslationPopup, {
      viewState: 'streaming',
      sourceText: 'Hello world',
      draftSourceText: 'Hello world',
      sourceLangLabel: '英语',
      targetLangLabel: '中文',
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
      screen.queryByRole('button', { name: /回填译文/ }),
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
      screen.queryByRole('button', { name: /回填译文/ }),
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
      sourceLangLabel: '英语',
      targetLangLabel: '中文',
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

    await fireEvent.input(screen.getByPlaceholderText(/在这里输入或修改原文/), {
      target: { value: 'Edited again' },
    });
    expect(ondraftsourcechange).toHaveBeenCalledWith('Edited again');

    expect(
      screen.getByText(/按 Ctrl\+Enter 可直接重新翻译全文/),
    ).toBeInTheDocument();

    await fireEvent.click(screen.getByRole('button', { name: /重新翻译全文/ }));
    expect(ontranslatedraft).toHaveBeenCalledTimes(1);

    await fireEvent.click(screen.getByRole('button', { name: /还原/ }));
    expect(onresetdraft).toHaveBeenCalledTimes(1);
  });
});
