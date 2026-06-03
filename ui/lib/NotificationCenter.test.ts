import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import NotificationCenter from './NotificationCenter.svelte';
import type { AppNotification } from './notifications';

describe('NotificationCenter', () => {
  it('renders thin notification strips and dismisses them through the callback', async () => {
    const ondismiss = vi.fn();
    const notifications: AppNotification[] = [
      {
        id: 'daemon-1',
        kind: 'error',
        title: '后台错误',
        message: '无法创建托盘图标。',
        scope: 'global',
      },
    ];

    render(NotificationCenter, { notifications, ondismiss });

    expect(screen.getByTestId('notification-strip')).toBeInTheDocument();
    expect(screen.getByRole('alert')).toHaveTextContent('后台错误');
    expect(screen.getByRole('alert')).toHaveTextContent('无法创建托盘图标。');

    await fireEvent.click(screen.getByRole('button', { name: /关闭通知：后台错误/ }));
    expect(ondismiss).toHaveBeenCalledWith('daemon-1');
  });
});
