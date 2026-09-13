export type ResizeDirection =
  | 'East'
  | 'North'
  | 'NorthEast'
  | 'NorthWest'
  | 'South'
  | 'SouthEast'
  | 'SouthWest'
  | 'West';

export type ResizeHandle = {
  direction: ResizeDirection;
  className: string;
  cursor: string;
};

export const RESIZE_HANDLES: ResizeHandle[] = [
  { direction: 'South', className: 'bottom-0 left-4 right-4 h-1.5', cursor: 'ns-resize' },
  { direction: 'West', className: 'bottom-4 left-0 top-4 w-1.5', cursor: 'ew-resize' },
  { direction: 'East', className: 'bottom-4 right-0 top-4 w-1.5', cursor: 'ew-resize' },
  { direction: 'SouthWest', className: 'bottom-0 left-0 h-4 w-4', cursor: 'nesw-resize' },
  { direction: 'SouthEast', className: 'bottom-0 right-0 h-4 w-4', cursor: 'nwse-resize' },
];

export function shouldDismissOnBlur(showSettings: boolean, windowPinned: boolean) {
  return !showSettings && !windowPinned;
}

/**
 * Whether Escape should collapse the translation window.
 *
 * Escape must never interrupt an active IME composition, otherwise typing Chinese would close
 * the window mid-word. Pinning does not block Escape: the contract is that one Escape always
 * collapses, and the caller un-pins as part of that.
 */
export function shouldDismissOnEscape(isComposing: boolean): boolean {
  return !isComposing;
}

/**
 * Whether an observed resize should be recorded as a user-chosen height.
 *
 * `setSize` from the auto-fit path also fires `onResized`. Without this guard the very first
 * auto-fit would look like a manual resize and permanently freeze the window height. Pinned
 * windows have their own placement persistence and keep auto-fitting disabled anyway.
 */
export function shouldRecordUserResize(programmatic: boolean, windowPinned: boolean): boolean {
  return !programmatic && !windowPinned;
}

/** Whether content may resize the window automatically. */
export function shouldAutoFit(windowPinned: boolean, userResizedHeight: number | null): boolean {
  return !windowPinned && userResizedHeight === null;
}
