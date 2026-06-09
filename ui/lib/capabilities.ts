export type FeatureCapability = 'ready' | 'needs_permission' | 'unsupported';

export interface SystemCapabilities {
  aura_mode: FeatureCapability;
  paste_back: FeatureCapability;
}
