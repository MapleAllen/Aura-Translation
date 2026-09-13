import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import {
  DARK_THEME,
  FONT_STACKS,
  LIGHT_THEME,
  SURFACE_UTILITY_REPLACEMENTS,
  toSemanticSurfaceClasses,
} from './theme';

function readAppCss(): string {
  return readFileSync(resolve(process.cwd(), 'ui/app.css'), 'utf8');
}

describe('theme palette', () => {
  it('defines the same tokens for both appearances', () => {
    // A token present only in the light palette would silently fall back to a light value in
    // dark mode, which is exactly the class of bug this check exists to prevent.
    expect(Object.keys(DARK_THEME).sort()).toEqual(Object.keys(LIGHT_THEME).sort());
  });

  it('never reuses a light value in the dark palette', () => {
    for (const [token, lightValue] of Object.entries(LIGHT_THEME)) {
      const darkValue = DARK_THEME[token as keyof typeof DARK_THEME];
      expect(darkValue, `${token} must differ between appearances`).not.toBe(lightValue);
    }
  });

  it('prefers the system font over bundled or Windows-first families', () => {
    for (const stack of [FONT_STACKS.body, FONT_STACKS.display]) {
      expect(stack.startsWith('-apple-system')).toBe(true);
      // A Microsoft-first stack was the previous default and must not lead any stack.
      expect(stack.indexOf('-apple-system')).toBeLessThan(stack.indexOf('Microsoft YaHei UI'));
    }
  });
});

describe('surface utility rewriting', () => {
  it('maps the longest matching utility before its shorter prefix', () => {
    expect(toSemanticSurfaceClasses('bg-white/68')).toBe('bg-aura-glass-hover');
    expect(toSemanticSurfaceClasses('hover:bg-white/60')).toBe('hover:bg-aura-glass-hover');
    expect(toSemanticSurfaceClasses('bg-white')).toBe('bg-aura-surface-strong');
  });

  it('leaves unrelated classes untouched', () => {
    expect(toSemanticSurfaceClasses('rounded-lg border border-aura-border')).toBe(
      'rounded-lg border border-aura-border',
    );
  });

  it('declares a replacement for every literal it is expected to remove', () => {
    for (const literal of Object.keys(SURFACE_UTILITY_REPLACEMENTS)) {
      expect(literal.startsWith('bg-white')).toBe(true);
    }
  });
});

describe('app.css', () => {
  it('declares dark appearance tokens', () => {
    const css = readAppCss();
    expect(css).toContain('prefers-color-scheme: dark');
    for (const token of Object.keys(DARK_THEME)) {
      expect(css, `${token} must appear in app.css`).toContain(token);
    }
  });

  it('uses the system font stacks', () => {
    const css = readAppCss();
    expect(css).toContain('-apple-system');
  });

  it('honours the reduced-motion preference', () => {
    expect(readAppCss()).toContain('prefers-reduced-motion');
  });
});

describe('component styling', () => {
  const components = [
    'SettingsPanel.svelte',
    'TranslationPopup.svelte',
    'NotificationCenter.svelte',
    'SetupStatusCard.svelte',
    'ProfileManager.svelte',
    'HistoryList.svelte',
    'TranslationWindowView.svelte',
    'SettingsWindowView.svelte',
    'LanguageSelector.svelte',
    'SkeletonLoader.svelte',
  ];

  it('has no hardcoded light surfaces left in components', () => {
    for (const component of components) {
      const source = readFileSync(resolve(process.cwd(), 'ui/lib', component), 'utf8');
      expect(source, `${component} still uses a literal white surface`).not.toMatch(
        /bg-white(\/\d+)?/,
      );
    }
  });

  it('uses no arbitrary color literals in any component', () => {
    // The previous version of this test checked a hand-maintained list of literals, so a new
    // literal slipped through and shipped a light-only warning background that was unreadable in
    // dark mode. Scan for the whole category instead.
    const arbitraryColor = /(?:bg|text|border|from|to|via|ring|shadow|fill|stroke)-\[#[0-9a-fA-F]{3,8}\]/g;

    for (const component of components) {
      const source = readFileSync(resolve(process.cwd(), 'ui/lib', component), 'utf8');
      const found = source.match(arbitraryColor) ?? [];
      expect(
        found,
        `${component} must use semantic tokens, not arbitrary colors: ${found.join(', ')}`,
      ).toEqual([]);
    }
  });

  it('declares every token that replaces a former literal', () => {
    // The literals this change removed, and the tokens that must exist to stand in for them.
    const replacements = [
      '--color-aura-danger-surface',
      '--color-aura-warning',
      '--color-aura-warning-surface',
      '--color-aura-warning-border',
    ];
    for (const token of replacements) {
      expect(Object.keys(LIGHT_THEME)).toContain(token);
      expect(Object.keys(DARK_THEME)).toContain(token);
    }
  });
});
