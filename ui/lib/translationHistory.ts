import type { Provider } from './appConfig';

export type TranslationHistoryStatus = 'success' | 'error';

export type TranslationUsage = {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
};

export type TranslationHistoryEntry = {
  id: string;
  source_text: string;
  translated_text: string;
  error_message: string | null;
  source_lang: string;
  target_lang: string;
  provider: Provider;
  model: string;
  usage: TranslationUsage | null;
  status: TranslationHistoryStatus;
  created_at_ms: number;
};

export function formatHistoryTimestamp(timestampMs: number) {
  return new Date(timestampMs).toLocaleString();
}

export function formatTokenCount(tokenCount: number) {
  return tokenCount.toLocaleString();
}
