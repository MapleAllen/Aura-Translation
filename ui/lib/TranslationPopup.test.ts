import { render, screen } from '@testing-library/svelte';
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
      onLanguageChange: vi.fn(),
      oncancel: vi.fn(),
    });

    expect(screen.getByText('Copy text and press Alt+Shift+T')).toBeInTheDocument();
  });
});
