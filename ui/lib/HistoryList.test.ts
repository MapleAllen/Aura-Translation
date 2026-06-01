import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import HistoryList from './HistoryList.svelte';
import type { TranslationHistoryEntry } from './translationHistory';

const sampleEntry: TranslationHistoryEntry = {
  id: 'entry-1',
  source_text: 'Hello world',
  translated_text: '你好世界',
  error_message: null,
  source_lang: 'English',
  target_lang: 'Chinese',
  provider: 'deepseek',
  model: 'deepseek-chat',
  usage: { prompt_tokens: 5, completion_tokens: 4, total_tokens: 9 },
  status: 'success',
  created_at_ms: Date.UTC(2026, 0, 1, 12, 0, 0),
};

describe('HistoryList clear all inline confirm', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('hides the Clear all button when there are no entries', () => {
    render(HistoryList, {
      entries: [],
    });

    expect(screen.queryByTestId('clear-all-button')).not.toBeInTheDocument();
    expect(screen.queryByTestId('clear-confirm-group')).not.toBeInTheDocument();
  });

  it('does not call onclear on the first click of Clear all', async () => {
    const onclear = vi.fn();
    render(HistoryList, {
      entries: [sampleEntry],
      onclear,
    });

    await fireEvent.click(screen.getByTestId('clear-all-button'));
    expect(onclear).not.toHaveBeenCalled();
  });

  it('shows the confirm group after the first click and hides the original button', async () => {
    render(HistoryList, {
      entries: [sampleEntry],
    });

    await fireEvent.click(screen.getByTestId('clear-all-button'));

    expect(screen.queryByTestId('clear-all-button')).not.toBeInTheDocument();
    expect(screen.getByTestId('clear-confirm-group')).toBeInTheDocument();
    expect(screen.getByTestId('clear-confirm-commit')).toHaveTextContent(/confirm clear all/i);
    expect(screen.getByTestId('clear-confirm-cancel')).toHaveTextContent(/cancel/i);
  });

  it('calls onclear only when the user confirms', async () => {
    const onclear = vi.fn();
    render(HistoryList, {
      entries: [sampleEntry],
      onclear,
    });

    await fireEvent.click(screen.getByTestId('clear-all-button'));
    await fireEvent.click(screen.getByTestId('clear-confirm-commit'));

    expect(onclear).toHaveBeenCalledTimes(1);
    expect(screen.queryByTestId('clear-confirm-group')).not.toBeInTheDocument();
  });

  it('cancels without calling onclear when Cancel is pressed', async () => {
    const onclear = vi.fn();
    render(HistoryList, {
      entries: [sampleEntry],
      onclear,
    });

    await fireEvent.click(screen.getByTestId('clear-all-button'));
    await fireEvent.click(screen.getByTestId('clear-confirm-cancel'));

    expect(onclear).not.toHaveBeenCalled();
    expect(screen.getByTestId('clear-all-button')).toBeInTheDocument();
    expect(screen.queryByTestId('clear-confirm-group')).not.toBeInTheDocument();
  });

  it('auto-resets to the initial state after the 5s timeout', async () => {
    const onclear = vi.fn();
    render(HistoryList, {
      entries: [sampleEntry],
      onclear,
    });

    await fireEvent.click(screen.getByTestId('clear-all-button'));
    expect(screen.getByTestId('clear-confirm-group')).toBeInTheDocument();

    vi.advanceTimersByTime(5000);

    await waitFor(() => {
      expect(screen.queryByTestId('clear-confirm-group')).not.toBeInTheDocument();
    });
    expect(screen.getByTestId('clear-all-button')).toBeInTheDocument();
    expect(onclear).not.toHaveBeenCalled();
  });

  it('does not call onclear before the timeout elapses', async () => {
    const onclear = vi.fn();
    render(HistoryList, {
      entries: [sampleEntry],
      onclear,
    });

    await fireEvent.click(screen.getByTestId('clear-all-button'));
    vi.advanceTimersByTime(4999);
    expect(onclear).not.toHaveBeenCalled();
    expect(screen.getByTestId('clear-confirm-group')).toBeInTheDocument();
  });
});
