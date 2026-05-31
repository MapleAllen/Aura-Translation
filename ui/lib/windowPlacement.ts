import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { WindowPlacement } from './appConfig';

export type WindowPlacementKind = 'settings' | 'pinned_translation';

export async function captureCurrentWindowPlacement(): Promise<WindowPlacement> {
  const appWindow = getCurrentWindow();
  const [position, size, scaleFactor] = await Promise.all([
    appWindow.outerPosition(),
    appWindow.outerSize(),
    appWindow.scaleFactor(),
  ]);

  return {
    x: position.x / scaleFactor,
    y: position.y / scaleFactor,
    width: size.width / scaleFactor,
    height: size.height / scaleFactor,
    monitor: null,
  };
}

export async function persistCurrentWindowPlacement(kind: WindowPlacementKind) {
  const placement = await captureCurrentWindowPlacement();
  await invoke('save_window_placement', { kind, placement });
}
