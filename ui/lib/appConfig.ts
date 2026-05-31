export type Provider = 'deepseek' | 'openrouter' | 'ollama';

export type WindowPlacement = {
  x: number;
  y: number;
  width: number | null;
  height: number | null;
  monitor: string | null;
};

export type AppConfig = {
  api_key: string;
  model: string;
  source_lang: string;
  target_lang: string;
  hotkey: string;
  aura_mode_enabled: boolean;
  aura_guard_enabled: boolean;
  window_pinned: boolean;
  provider: Provider;
  api_base_url: string;
  available_models: string[];
  settings_window_placement: WindowPlacement | null;
  pinned_translation_placement: WindowPlacement | null;
};

export function createDefaultAppConfig(): AppConfig {
  return {
    api_key: '',
    model: 'deepseek-chat',
    source_lang: 'auto',
    target_lang: 'Chinese',
    hotkey: 'CmdOrCtrl+T',
    aura_mode_enabled: false,
    aura_guard_enabled: true,
    window_pinned: false,
    provider: 'deepseek',
    api_base_url: 'https://api.deepseek.com',
    available_models: ['deepseek-chat', 'deepseek-reasoner'],
    settings_window_placement: null,
    pinned_translation_placement: null,
  };
}

export function cloneAppConfig(config: AppConfig): AppConfig {
  return {
    ...config,
    available_models: [...config.available_models],
    settings_window_placement: config.settings_window_placement
      ? { ...config.settings_window_placement }
      : null,
    pinned_translation_placement: config.pinned_translation_placement
      ? { ...config.pinned_translation_placement }
      : null,
  };
}
