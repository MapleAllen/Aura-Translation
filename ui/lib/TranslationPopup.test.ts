import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import TranslationPopup from './TranslationPopup.svelte';

function renderPopup(overrides: Record<string, unknown> = {}) {
  return render(TranslationPopup, {
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
    windowPinned: false,
    ondraftsourcechange: vi.fn(),
    ontranslatedraft: vi.fn(),
    onresetdraft: vi.fn(),
    onTogglePinned: vi.fn(),
    onretry: vi.fn(),
    oncancel: vi.fn(),
    ondismiss: vi.fn(),
    oncopy: vi.fn(),
    ...overrides,
  });
}

describe('TranslationPopup', () => {
  it('renders the idle hint inside the shared operator surface', () => {
    renderPopup();

    expect(screen.getByTestId('popup-status-bar')).toBeInTheDocument();
    expect(screen.getByTestId('popup-main-surface')).toBeInTheDocument();
    expect(screen.getByTestId('popup-empty-state')).toBeInTheDocument();
  });

  it('keeps source text collapsed by default in result mode and reveals it on demand', async () => {
    renderPopup({
      viewState: 'result',
      sourceText: 'Hello world',
      draftSourceText: 'Hello world',
      sourceLangLabel: '英语',
      targetLangLabel: '中文',
      providerLabel: 'DeepSeek',
      modelLabel: 'deepseek-chat',
      translatedText: '你好，世界',
      usage: {
        prompt_tokens: 9,
        completion_tokens: 6,
        total_tokens: 15,
      },
    });

    // The translation is the visual subject: language, model, and token usage start collapsed
    // behind the details toggle rather than occupying the header.
    expect(screen.getByText('你好，世界')).toBeInTheDocument();
    expect(screen.queryByTestId('popup-context-details')).not.toBeInTheDocument();
    expect(screen.queryByTestId('popup-source-panel')).not.toBeInTheDocument();

    await fireEvent.click(screen.getByTestId('details-toggle-button'));
    expect(screen.getByTestId('popup-context-details')).toHaveTextContent('英语');
    expect(screen.getByTestId('popup-context-details')).toHaveTextContent('15 token');

    await fireEvent.click(screen.getByTestId('source-toggle-button'));

    expect(screen.getByTestId('popup-source-panel')).toHaveTextContent('Hello world');
    expect(screen.getByText('你好，世界')).toBeInTheDocument();
  });

  it('shows pin state and exposes result actions from the action rail', async () => {
    const onTogglePinned = vi.fn();
    const oncopy = vi.fn();

    renderPopup({
      viewState: 'result',
      sourceText: 'Copy this result',
      draftSourceText: 'Copy this result',
      sourceLangLabel: '英语',
      targetLangLabel: '中文',
      providerLabel: 'DeepSeek',
      modelLabel: 'deepseek-chat',
      translatedText: '请复制这条翻译结果',
      windowPinned: false,
      onTogglePinned,
      oncopy,
    });

    const pinButton = screen.getByRole('button', { name: '固定' });
    expect(pinButton).toHaveAttribute('aria-pressed', 'false');

    await fireEvent.click(pinButton);
    expect(onTogglePinned).toHaveBeenCalledWith(true);

    expect(screen.getByTestId('popup-result-actions')).toBeInTheDocument();
    await fireEvent.click(screen.getByRole('button', { name: /复制译文/ }));
    expect(oncopy).toHaveBeenCalledTimes(1);
  });

  it('does not render a paste-back action in result mode', () => {
    renderPopup({
      viewState: 'result',
      sourceText: 'Copy this result',
      draftSourceText: 'Copy this result',
      sourceLangLabel: '鑻辫',
      targetLangLabel: '涓枃',
      providerLabel: 'DeepSeek',
      modelLabel: 'deepseek-chat',
      translatedText: 'Translated result',
    });

    expect(screen.queryByRole('button', { name: /回填/ })).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: /复制译文/ })).toBeInTheDocument();
  });

  it('shows the cross-platform composer shortcut hint', () => {
    renderPopup({
      viewState: 'result',
      sourceText: 'Original text',
      draftSourceText: 'Original text',
      translatedText: '翻译结果',
      showComposer: true,
      hasDraftChanges: false,
      windowPinned: true,
    });

    expect(screen.getByText(/Cmd\/Ctrl\+Enter/)).toBeInTheDocument();
  });

  it('reuses the shared surface for loading and streaming states', () => {
    const { rerender } = renderPopup({
      viewState: 'loading',
      sourceText: 'Loading text',
      draftSourceText: 'Loading text',
    });

    expect(screen.getByTestId('popup-main-surface')).toBeInTheDocument();
    expect(screen.getByTestId('popup-loading-state')).toBeInTheDocument();

    rerender({
      viewState: 'streaming',
      sourceText: 'Loading text',
      draftSourceText: 'Loading text',
      sourceLangLabel: '英语',
      targetLangLabel: '中文',
      providerLabel: 'DeepSeek',
      modelLabel: 'deepseek-chat',
      retryAttempt: null,
      translatedText: '正在输出',
      errorMessage: '',
      showComposer: false,
      hasDraftChanges: false,
      usage: null,
      windowPinned: false,
      ondraftsourcechange: vi.fn(),
      ontranslatedraft: vi.fn(),
      onresetdraft: vi.fn(),
      onTogglePinned: vi.fn(),
      onretry: vi.fn(),
      oncancel: vi.fn(),
      ondismiss: vi.fn(),
      oncopy: vi.fn(),
    });

    expect(screen.getByTestId('popup-main-surface')).toBeInTheDocument();
    expect(screen.getByText('正在输出')).toBeInTheDocument();
  });

  it('shows a pinned draft composer with translate and reset actions', async () => {
    const ondraftsourcechange = vi.fn();
    const ontranslatedraft = vi.fn();
    const onresetdraft = vi.fn();

    renderPopup({
      viewState: 'result',
      sourceText: 'Original text',
      draftSourceText: 'Edited text',
      sourceLangLabel: '英语',
      targetLangLabel: '中文',
      providerLabel: 'DeepSeek',
      modelLabel: 'deepseek-chat',
      translatedText: '编辑后的翻译',
      showComposer: true,
      hasDraftChanges: true,
      windowPinned: true,
      ondraftsourcechange,
      ontranslatedraft,
      onresetdraft,
    });

    await fireEvent.input(screen.getByPlaceholderText(/在这里输入或修改原文/), {
      target: { value: 'Edited again' },
    });
    expect(ondraftsourcechange).toHaveBeenCalledWith('Edited again');

    await fireEvent.click(screen.getByRole('button', { name: /重新翻译全文/ }));
    expect(ontranslatedraft).toHaveBeenCalledTimes(1);

    await fireEvent.click(screen.getByRole('button', { name: /还原/ }));
    expect(onresetdraft).toHaveBeenCalledTimes(1);
  });
});
