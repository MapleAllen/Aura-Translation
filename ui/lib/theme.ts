/**
 * Appearance tokens for the Aura shell.
 *
 * The proposal requires system-font priority and light/dark semantic colors. Keeping the palette
 * in one auditable place — rather than scattered literal colors — is what makes "no hardcoded
 * light surface survives a dark appearance" a checkable property.
 */

/** System-first font stacks: macOS resolves to SF Pro and PingFang. */
export const FONT_STACKS = {
  body: "-apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Helvetica Neue', 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei UI', 'Noto Sans SC', sans-serif",
  display:
    "-apple-system, BlinkMacSystemFont, 'SF Pro Display', 'Helvetica Neue', 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei UI', 'Noto Sans SC', sans-serif",
  mono: "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace",
} as const;

export type ThemeToken =
  | '--color-aura-bg'
  | '--color-aura-glass'
  | '--color-aura-glass-hover'
  | '--color-aura-border'
  | '--color-aura-border-accent'
  | '--color-aura-accent'
  | '--color-aura-accent-soft'
  | '--color-aura-accent-glow'
  | '--color-aura-text'
  | '--color-aura-text-dim'
  | '--color-aura-text-muted'
  | '--color-aura-surface-strong'
  | '--color-aura-surface-soft'
  | '--color-aura-shadow'
  | '--color-aura-focus-ring'
  | '--color-aura-success'
  | '--color-aura-error'
  | '--color-aura-danger-surface'
  | '--color-aura-warning'
  | '--color-aura-warning-surface'
  | '--color-aura-warning-border';

export type ThemePalette = Record<ThemeToken, string>;

export const LIGHT_THEME: ThemePalette = {
  '--color-aura-bg': '#f4f7fb',
  '--color-aura-glass': '#fcfdff',
  '--color-aura-glass-hover': '#ffffff',
  '--color-aura-border': 'rgba(92, 108, 128, 0.16)',
  '--color-aura-border-accent': 'rgba(37, 111, 216, 0.32)',
  '--color-aura-accent': '#256fd8',
  '--color-aura-accent-soft': 'rgba(37, 111, 216, 0.09)',
  '--color-aura-accent-glow': 'rgba(37, 111, 216, 0.13)',
  '--color-aura-text': '#17202a',
  '--color-aura-text-dim': '#526173',
  '--color-aura-text-muted': '#8590a0',
  '--color-aura-surface-strong': '#ffffff',
  '--color-aura-surface-soft': '#f6f8fb',
  '--color-aura-shadow': 'rgba(24, 39, 56, 0.075)',
  '--color-aura-focus-ring': 'rgba(37, 111, 216, 0.14)',
  '--color-aura-success': '#1a7e60',
  '--color-aura-error': '#c95362',
  '--color-aura-danger-surface': '#fff8f9',
  '--color-aura-warning': '#8a6226',
  '--color-aura-warning-surface': '#fff7e4',
  '--color-aura-warning-border': '#e6c683',
};

/**
 * Dark appearance. macOS system colors are used as the reference so the window sits naturally
 * next to native panels instead of looking like an inverted light theme.
 */
export const DARK_THEME: ThemePalette = {
  '--color-aura-bg': '#1c1c1e',
  '--color-aura-glass': '#232326',
  '--color-aura-glass-hover': '#2b2b2f',
  '--color-aura-border': 'rgba(235, 240, 248, 0.14)',
  '--color-aura-border-accent': 'rgba(88, 158, 246, 0.42)',
  '--color-aura-accent': '#5aa2f8',
  '--color-aura-accent-soft': 'rgba(90, 162, 248, 0.16)',
  '--color-aura-accent-glow': 'rgba(90, 162, 248, 0.2)',
  '--color-aura-text': '#f2f4f8',
  '--color-aura-text-dim': '#b6bdc9',
  '--color-aura-text-muted': '#8c94a3',
  '--color-aura-surface-strong': '#2a2a2e',
  '--color-aura-surface-soft': '#242427',
  '--color-aura-shadow': 'rgba(0, 0, 0, 0.45)',
  '--color-aura-focus-ring': 'rgba(90, 162, 248, 0.3)',
  '--color-aura-success': '#4fc79b',
  '--color-aura-error': '#ff8a95',
  '--color-aura-danger-surface': '#3a2529',
  '--color-aura-warning': '#e8bd72',
  '--color-aura-warning-surface': '#3a3122',
  '--color-aura-warning-border': '#6b5730',
};

/**
 * Tailwind's `bg-white/<opacity>` utilities are literal light surfaces: they render as white
 * regardless of appearance. Each is mapped to the semantic token that carries the same role so
 * the dark palette can actually take effect.
 */
export const SURFACE_UTILITY_REPLACEMENTS: Record<string, string> = {
  'bg-white': 'bg-aura-surface-strong',
  'bg-white/95': 'bg-aura-glass',
  'bg-white/82': 'bg-aura-glass',
  'bg-white/80': 'bg-aura-glass',
  'bg-white/78': 'bg-aura-glass',
  'bg-white/75': 'bg-aura-glass',
  'bg-white/72': 'bg-aura-glass',
  'bg-white/70': 'bg-aura-glass',
  'bg-white/68': 'bg-aura-glass-hover',
  'bg-white/60': 'bg-aura-glass-hover',
};

/** Rewrites hardcoded light-surface utilities in a class string to semantic tokens. */
export function toSemanticSurfaceClasses(classNames: string): string {
  // Longest literal first, otherwise the `bg-white` rule matches the prefix of `bg-white/75`
  // before its own rule can apply.
  const ordered = Object.entries(SURFACE_UTILITY_REPLACEMENTS).sort(
    ([a], [b]) => b.length - a.length,
  );
  return ordered.reduce(
    (acc, [literal, token]) => acc.split(literal).join(token),
    classNames,
  );
}
