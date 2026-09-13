/**
 * Motion preferences.
 *
 * Kept separate from the colour palette: the translation window applies the preference to its
 * springs while the palette drives styling, and neither should have to import the other.
 */

/**
 * Whether the user asked macOS to reduce motion.
 *
 * Guarded for non-browser contexts so tests and SSR do not need a `matchMedia` stub.
 */
export function prefersReducedMotion(): boolean {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return false;
  }
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}
