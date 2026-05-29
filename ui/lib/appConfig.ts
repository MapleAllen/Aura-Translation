export type Provider = 'deepseek' | 'openrouter' | 'ollama';

export type AppConfig = {
  api_key: string;
  model: string;
  source_lang: string;
  target_lang: string;
  hotkey: string;
  window_pinned: boolean;
  provider: Provider;
  api_base_url: string;
  available_models: string[];
};

export function createDefaultAppConfig(): AppConfig {
  return {
    api_key: '',
    model: 'deepseek-chat',
    source_lang: 'auto',
    target_lang: 'Chinese',
    hotkey: 'CmdOrCtrl+T',
    window_pinned: false,
    provider: 'deepseek',
    api_base_url: 'https://api.deepseek.com',
    available_models: ['deepseek-chat', 'deepseek-reasoner'],
  };
}

export function cloneAppConfig(config: AppConfig): AppConfig {
  return {
    ...config,
    available_models: [...config.available_models],
  };
}
