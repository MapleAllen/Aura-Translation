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

describe('HistoryList operator log view', () => {
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

  it('shows summary rows first and keeps details collapsed by default', () => {
    render(HistoryList, {
      entries: [sampleEntry],
    });

    expect(screen.getByTestId('history-list')).toHaveTextContent('DeepSeek');
    expect(screen.getByTestId('history-list')).toHaveTextContent('英语');
    expect(screen.queryByTestId('history-details-entry-1')).not.toBeInTheDocument();
  });

  it('reveals and hides details through the row toggle', async () => {
    render(HistoryList, {
      entries: [sampleEntry],
    });

    await fireEvent.click(screen.getByTestId('history-toggle-entry-1'));
    expect(screen.getByTestId('history-details-entry-1')).toHaveTextContent('Hello world');
    expect(screen.getByTestId('history-details-entry-1')).toHaveTextContent('你好世界');

    await fireEvent.click(screen.getByTestId('history-toggle-entry-1'));
    expect(screen.queryByTestId('history-details-entry-1')).not.toBeInTheDocument();
  });

  it('keeps row actions available from the summary line', async () => {
    const onretry = vi.fn();
    const oncopy = vi.fn();
    const ondelete = vi.fn();

    render(HistoryList, {
      entries: [sampleEntry],
      onretry,
      oncopy,
      ondelete,
    });

    await fireEvent.click(screen.getByTestId('history-retry-entry-1'));
    await fireEvent.click(screen.getByTestId('history-copy-entry-1'));
    await fireEvent.click(screen.getByTestId('history-delete-entry-1'));

    expect(onretry).toHaveBeenCalledWith('entry-1');
    expect(oncopy).toHaveBeenCalledWith('你好世界');
    expect(ondelete).toHaveBeenCalledWith('entry-1');
  });

  it('calls onclear only when the user confirms', async () => {
    const onclear = vi.fn();
    render(HistoryList, {
      entries: [sampleEntry],
      onclear,
    });

    await fireEvent.click(screen.getByTestId('clear-all-button'));
    expect(onclear).not.toHaveBeenCalled();

    await fireEvent.click(screen.getByTestId('clear-confirm-commit'));
    expect(onclear).toHaveBeenCalledTimes(1);
  });

  it('cancels clear confirmation and restores the default action', async () => {
    const onclear = vi.fn();
    render(HistoryList, {
      entries: [sampleEntry],
      onclear,
    });

    await fireEvent.click(screen.getByTestId('clear-all-button'));
    await fireEvent.click(screen.getByTestId('clear-confirm-cancel'));

    expect(onclear).not.toHaveBeenCalled();
    expect(screen.getByTestId('clear-all-button')).toBeInTheDocument();
  });

  it('auto-resets clear confirmation after the timeout', async () => {
    render(HistoryList, {
      entries: [sampleEntry],
      onclear: vi.fn(),
    });

    await fireEvent.click(screen.getByTestId('clear-all-button'));
    vi.advanceTimersByTime(5000);

    await waitFor(() => {
      expect(screen.queryByTestId('clear-confirm-group')).not.toBeInTheDocument();
    });
    expect(screen.getByTestId('clear-all-button')).toBeInTheDocument();
  });
});
