import { describe, expect, it } from 'vitest';
import { shouldDismissOnBlur } from './windowBehavior';

describe('shouldDismissOnBlur', () => {
  it('dismisses when the window is not pinned and settings are closed', () => {
    expect(shouldDismissOnBlur(false, false)).toBe(true);
  });

  it('keeps the window visible when pinned', () => {
    expect(shouldDismissOnBlur(false, true)).toBe(false);
  });

  it('keeps the settings panel visible on blur', () => {
    expect(shouldDismissOnBlur(true, false)).toBe(false);
  });
});
