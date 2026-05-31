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
    title: payload.recoverable ? 'Daemon warning' : 'Daemon error',
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
    title: 'Hotkey conflict',
    message: `Could not register "${payload.hotkey}": ${payload.error}`,
    scope: 'settings',
  };
}

export function createTranslationErrorNotification(message: string): AppNotification {
  return {
    id: nextNotificationId('translation'),
    kind: 'error',
    title: 'Translation failed',
    message,
    scope: 'global',
  };
}

export function createAuraGuardNotification(reason: string): AppNotification {
  return {
    id: nextNotificationId('aura-guard'),
    kind: 'warning',
    title: 'Sensitive clipboard skipped',
    message: reason,
    scope: 'global',
  };
}

export function formatTranslationError(input: unknown): string {
  const message = String(input ?? 'Translation failed.');

  if (message.startsWith('No API key configured')) {
    return 'No API key configured. Open Settings from the tray and save a key for the selected provider.';
  }

  if (message.startsWith('No response from API (timeout)')) {
    return 'The provider did not return a first token within 20 seconds. Try again or switch provider or model.';
  }

  if (message.includes('API error (401')) {
    return 'Authentication failed. Check the API key for the selected provider.';
  }

  if (message.includes('API error (403')) {
    return 'The provider rejected this request. Check account permissions and model access.';
  }

  if (message.includes('API error (429')) {
    return 'The provider rate-limited this request. Wait a moment and try again.';
  }

  if (message.startsWith('Network error:')) {
    return 'Could not reach the provider. Check the network connection and API base URL.';
  }

  if (message.startsWith('Malformed SSE data:')) {
    return 'The provider returned an invalid streaming response. Try again or switch provider.';
  }

  if (message.startsWith('Stream error:')) {
    return 'The translation stream ended unexpectedly. Try again.';
  }

  if (message.startsWith('Invalid API key header:')) {
    return 'The API key contains invalid characters and could not be sent.';
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
