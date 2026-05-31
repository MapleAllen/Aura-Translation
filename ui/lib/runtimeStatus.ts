export type RuntimeStatusLevel = 'ready' | 'needs_setup';

export type RuntimeChecklistItem = {
  code: string;
  label: string;
  ok: boolean;
};

export type RuntimeStatus = {
  level: RuntimeStatusLevel;
  summary: string;
  can_translate_now: boolean;
  checklist: RuntimeChecklistItem[];
};

export type ProviderProbeResult = {
  ok: boolean;
  message: string;
};
