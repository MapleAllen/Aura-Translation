import type { Provider } from './appConfig';

export type TranslationHistoryStatus = 'success' | 'error';

export type TranslationHistoryEntry = {
  id: string;
  source_text: string;
  translated_text: string;
  error_message: string | null;
  source_lang: string;
  target_lang: string;
  provider: Provider;
  model: string;
  status: TranslationHistoryStatus;
  created_at_ms: number;
};

export function formatHistoryTimestamp(timestampMs: number) {
  return new Date(timestampMs).toLocaleString();
}
