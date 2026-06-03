export type NotificationKind = 'error' | 'warning' | 'info';
export type NotificationScope = 'global' | 'settings';

export type AppNotification = {
  id: string;
  kind: NotificationKind;
  title: string;
  message: string;
  scope: NotificationScope;
};

export type DaemonErrorPayload = {
  code: string;
  message: string;
  recoverable: boolean;
};

export type HotkeyConflictPayload = {
  hotkey: string;
  error: string;
};

export type AuraGuardBlockedPayload = {
  reason: string;
};

type Provider = 'deepseek' | 'openrouter' | 'ollama';
type ApiKeyStorage = 'system' | 'plaintext_fallback' | 'legacy_plaintext';

let notificationCounter = 0;

function nextNotificationId(prefix: string) {
  notificationCounter += 1;
  return `${prefix}-${notificationCounter}`;
}

export function createDaemonErrorNotification(
  payload: DaemonErrorPayload,
): AppNotification {
  return {
    id: nextNotificationId('daemon'),
    kind: 'error',
    title: payload.recoverable ? '后台提醒' : '后台错误',
    message: payload.message,
    scope: 'global',
  };
}

export function createHotkeyConflictNotification(
  payload: HotkeyConflictPayload,
): AppNotification {
  return {
    id: nextNotificationId('hotkey'),
    kind: 'warning',
    title: '快捷键冲突',
    message: `无法注册 "${payload.hotkey}"：${payload.error}`,
    scope: 'settings',
  };
}

export function createTranslationErrorNotification(message: string): AppNotification {
  return {
    id: nextNotificationId('translation'),
    kind: 'error',
    title: '翻译失败',
    message,
    scope: 'global',
  };
}

export function createAuraGuardNotification(reason: string): AppNotification {
  return {
    id: nextNotificationId('aura-guard'),
    kind: 'warning',
    title: '已跳过敏感剪贴板内容',
    message: reason,
    scope: 'global',
  };
}

export function formatTranslationError(input: unknown): string {
  const message = String(input ?? '翻译失败。');

  if (message.startsWith('No API key configured')) {
    return '尚未配置 API Key。请从托盘打开设置，并为当前服务商保存密钥。';
  }

  if (message.startsWith('No response from API (timeout)')) {
    return '服务商在 20 秒内没有返回首个 token。请重试，或切换服务商/模型。';
  }

  if (message.includes('API error (401')) {
    return '认证失败。请检查当前服务商的 API Key。';
  }

  if (message.includes('API error (403')) {
    return '服务商拒绝了本次请求。请检查账号权限和模型访问权限。';
  }

  if (message.includes('API error (429')) {
    return '服务商触发了限流。请稍等后重试。';
  }

  if (message.startsWith('Network error:')) {
    return '无法连接服务商。请检查网络连接和 API 地址。';
  }

  if (message.startsWith('Malformed SSE data:')) {
    return '服务商返回了无效的流式响应。请重试或切换服务商。';
  }

  if (message.startsWith('Stream error:')) {
    return '翻译流意外结束。请重试。';
  }

  if (message.startsWith('Invalid API key header:') || message.startsWith('API Key 请求头无效：')) {
    return 'API Key 包含无效字符，无法发送。';
  }

  return message;
}

export function shouldShowPlaintextApiKeyWarning(
  provider: Provider,
  storage: ApiKeyStorage,
) {
  return provider !== 'ollama' && storage !== 'system';
}

export function usesSystemCredentialStorage(storage: ApiKeyStorage) {
  return storage === 'system';
}
