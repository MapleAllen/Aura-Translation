<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';
  import type { AppConfig } from './appConfig';
  import NotificationCenter from './NotificationCenter.svelte';
  import SettingsPanel from './SettingsPanel.svelte';
  import {
    createDaemonErrorNotification,
    createHotkeyConflictNotification,
    type AppNotification,
    type DaemonErrorPayload,
    type HotkeyConflictPayload,
  } from './notifications';
  import { persistCurrentWindowPlacement } from './windowPlacement';

  let notifications = $state<AppNotification[]>([]);
  let hotkeyConflictMessage = $state('');
  let placementSaveTimeoutId: ReturnType<typeof setTimeout> | null = null;

  function pushNotification(notification: AppNotification) {
    notifications = [notification, ...notifications.filter((item) => item.scope !== notification.scope)]
      .slice(0, 3);
  }

  function dismissNotification(id: string) {
    notifications = notifications.filter((item) => item.id !== id);
  }

  function clearSettingsWarnings() {
    hotkeyConflictMessage = '';
    notifications = notifications.filter((item) => item.scope !== 'settings');
  }

  function schedulePlacementSave() {
    if (placementSaveTimeoutId !== null) {
      clearTimeout(placementSaveTimeoutId);
    }
    placementSaveTimeoutId = setTimeout(() => {
      void persistCurrentWindowPlacement('settings').catch((e) => {
        console.error('Failed to persist settings placement:', e);
      });
    }, 180);
  }

  async function closeWindow() {
    try {
      await persistCurrentWindowPlacement('settings');
    } catch (e) {
      console.error('Failed to persist settings placement on close:', e);
    }

    try {
      await getCurrentWindow().hide();
    } catch (e) {
      console.error('Failed to hide settings window:', e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      void closeWindow();
    }
  }

  onMount(() => {
    const appWindow = getCurrentWindow();
    const unlisteners: Array<() => void> = [];

    const register = async () => {
      unlisteners.push(
        await listen<HotkeyConflictPayload>('hotkey-conflict', (event) => {
          const notification = createHotkeyConflictNotification(event.payload);
          hotkeyConflictMessage = notification.message;
          pushNotification(notification);
        }),
      );

      unlisteners.push(
        await listen('hotkey-registered', () => {
          clearSettingsWarnings();
        }),
      );

      unlisteners.push(
        await listen<DaemonErrorPayload>('daemon-error', (event) => {
          pushNotification(createDaemonErrorNotification(event.payload));
        }),
      );

      unlisteners.push(
        await appWindow.onMoved(() => {
          schedulePlacementSave();
        }),
      );

      unlisteners.push(
        await appWindow.onResized(() => {
          schedulePlacementSave();
        }),
      );
    };

    void register();
    void invoke('mark_ui_ready').catch((e) => {
      console.error('Failed to mark settings UI as ready:', e);
    });

    return () => {
      if (placementSaveTimeoutId !== null) clearTimeout(placementSaveTimeoutId);
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="relative h-screen w-screen overflow-hidden">
  <NotificationCenter {notifications} ondismiss={dismissNotification} />

  <SettingsPanel
    visible={true}
    onclose={() => {
      void closeWindow();
    }}
    onsaved={(_config: AppConfig) => {
      clearSettingsWarnings();
    }}
    {hotkeyConflictMessage}
  />
</div>
