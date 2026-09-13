import { describe, expect, it } from 'vitest';
import {
  shouldAutoFit,
  shouldDismissOnBlur,
  shouldDismissOnEscape,
  shouldRecordUserResize,
} from './windowBehavior';

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

describe('shouldDismissOnEscape', () => {
  it('collapses on a plain Escape press', () => {
    expect(shouldDismissOnEscape(false)).toBe(true);
  });

  it('ignores Escape while an IME composition is active', () => {
    // Otherwise typing Chinese would close the window mid-word.
    expect(shouldDismissOnEscape(true)).toBe(false);
  });
});

describe('shouldRecordUserResize', () => {
  it('records a resize the user performed', () => {
    expect(shouldRecordUserResize(false, false)).toBe(true);
  });

  it('ignores the window resize that auto-fitting itself caused', () => {
    // Without this, the first auto-fit would look like a manual resize and freeze the height.
    expect(shouldRecordUserResize(true, false)).toBe(false);
  });

  it('ignores resizes while the window is pinned', () => {
    expect(shouldRecordUserResize(false, true)).toBe(false);
  });
});

describe('shouldAutoFit', () => {
  it('fits content when the user has not resized the window', () => {
    expect(shouldAutoFit(false, null)).toBe(true);
  });

  it('stops fighting a height the user chose', () => {
    expect(shouldAutoFit(false, 320)).toBe(false);
  });

  it('never auto-fits a pinned window', () => {
    expect(shouldAutoFit(true, null)).toBe(false);
  });
});
