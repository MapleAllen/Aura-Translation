import type { ApiKeyStorage, Provider } from './appConfig';

export type TranslationProfile = {
  id: string;
  name: string;
  api_key: string;
  api_key_storage: ApiKeyStorage;
  model: string;
  source_lang: string;
  target_lang: string;
  provider: Provider;
  api_base_url: string;
  available_models: string[];
};

export type TranslationProfilesStore = {
  active_profile_id: string;
  profiles: TranslationProfile[];
};
