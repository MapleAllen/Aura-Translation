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
