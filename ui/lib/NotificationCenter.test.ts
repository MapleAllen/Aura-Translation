import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import NotificationCenter from './NotificationCenter.svelte';
import type { AppNotification } from './notifications';

describe('NotificationCenter', () => {
  it('renders notifications and dismisses them through the callback', async () => {
    const ondismiss = vi.fn();
    const notifications: AppNotification[] = [
      {
        id: 'daemon-1',
        kind: 'error',
        title: 'Daemon error',
        message: 'Tray icon could not be created.',
        scope: 'global',
      },
    ];

    render(NotificationCenter, { notifications, ondismiss });

    expect(screen.getByRole('alert')).toHaveTextContent('Daemon error');
    expect(screen.getByRole('alert')).toHaveTextContent('Tray icon could not be created.');

    await fireEvent.click(screen.getByRole('button', { name: /dismiss daemon error/i }));
    expect(ondismiss).toHaveBeenCalledWith('daemon-1');
  });
});
