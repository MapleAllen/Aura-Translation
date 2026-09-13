<script lang="ts">
  import { LANGUAGES } from './languages';
  import type { TranslationProfilesStore } from './translationProfiles';

  type Props = {
    store: TranslationProfilesStore | null;
    draftName: string;
    ondraftnamechange?: (value: string) => void;
    oncreate?: () => void;
    onrename?: (profileId: string) => void;
    onactivate?: (profileId: string) => void;
    ondelete?: (profileId: string) => void;
  };

  let {
    store,
    draftName,
    ondraftnamechange,
    oncreate,
    onrename,
    onactivate,
    ondelete,
  }: Props = $props();

  const activeProfile = $derived(
    store?.profiles.find((profile) => profile.id === store.active_profile_id) ?? null,
  );

  const DELETE_CONFIRM_TIMEOUT_MS = 5000;
  let confirmingDeleteId = $state<string | null>(null);
  let deleteConfirmTimer: ReturnType<typeof setTimeout> | null = null;

  function clearDeleteConfirmTimer() {
    if (deleteConfirmTimer !== null) {
      clearTimeout(deleteConfirmTimer);
      deleteConfirmTimer = null;
    }
  }

  function enterDeleteConfirm(profileId: string) {
    clearDeleteConfirmTimer();
    confirmingDeleteId = profileId;
    deleteConfirmTimer = setTimeout(() => {
      confirmingDeleteId = null;
      deleteConfirmTimer = null;
    }, DELETE_CONFIRM_TIMEOUT_MS);
  }

  function cancelDeleteConfirm() {
    clearDeleteConfirmTimer();
    confirmingDeleteId = null;
  }

  function commitDelete(profileId: string) {
    clearDeleteConfirmTimer();
    confirmingDeleteId = null;
    ondelete?.(profileId);
  }

  function providerLabel(provider: string) {
    switch (provider) {
      case 'deepseek':
        return 'DeepSeek';
      case 'openrouter':
        return 'OpenRouter';
      case 'ollama':
        return 'Ollama';
      default:
        return provider;
    }
  }

  function languageLabel(code: string) {
    return LANGUAGES.find((language) => language.code === code)?.label ?? code;
  }

  $effect(() => {
    return () => clearDeleteConfirmTimer();
  });
</script>

<section class="space-y-4">
  <div>
    <p class="aura-section-title">配置方案</p>
    <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
      保存常用翻译设置，在独立工作流之间快速切换。
    </p>
  </div>

  <div class="grid gap-2.5 md:grid-cols-[minmax(0,1fr)_auto_auto]">
    <input
      value={draftName}
      oninput={(event) => ondraftnamechange?.((event.currentTarget as HTMLInputElement).value)}
      placeholder="配置方案名称"
      class="aura-console-input"
    />
    <button
      class="aura-console-button"
      type="button"
      onclick={() => oncreate?.()}
    >
      另存为新方案
    </button>
    <button
      class="aura-console-button"
      type="button"
      onclick={() => activeProfile && onrename?.(activeProfile.id)}
      disabled={!activeProfile}
    >
      重命名当前方案
    </button>
  </div>

  {#if !store || store.profiles.length === 0}
    <div class="rounded-lg border border-dashed border-aura-border bg-aura-surface-soft px-3.5 py-3 text-xs leading-relaxed text-aura-text-dim">
      Aura 会自动创建一个默认翻译配置方案。
    </div>
  {:else}
    <div class="overflow-hidden rounded-lg border border-aura-border bg-aura-glass" data-testid="profile-list">
      {#each store.profiles as profile, index (profile.id)}
        <div
          class={`grid gap-3 px-4 py-3.5 md:grid-cols-[minmax(0,1fr)_auto] ${
            index === 0 ? '' : 'border-t border-aura-border'
          }`}
          data-testid="profile-row"
        >
          <div class="min-w-0">
            <div class="flex flex-wrap items-center gap-2">
              <p class="truncate text-sm font-medium text-aura-text">{profile.name}</p>
              {#if profile.id === store.active_profile_id}
                <span class="rounded-full border border-aura-border-accent bg-aura-accent-soft px-2.5 py-0.5 text-xs font-medium text-aura-accent">
                  当前
                </span>
              {/if}
            </div>
            <p class="mt-1 font-mono text-xs text-aura-text-dim">
              {providerLabel(profile.provider)} · {profile.model} · {languageLabel(profile.source_lang)} → {languageLabel(profile.target_lang)}
            </p>
          </div>

          <div class="flex flex-wrap items-center justify-end gap-2">
            {#if profile.id !== store.active_profile_id}
              <button
                class="aura-console-button"
                type="button"
                onclick={() => onactivate?.(profile.id)}
              >
                启用
              </button>
            {/if}

            {#if confirmingDeleteId === profile.id}
              <div class="flex flex-wrap items-center gap-2" data-testid="delete-confirm-group">
                <button
                  class="aura-console-button"
                  data-variant="primary"
                  type="button"
                  data-testid="delete-confirm-commit"
                  onclick={() => commitDelete(profile.id)}
                >
                  确认删除
                </button>
                <button
                  class="aura-console-button"
                  type="button"
                  data-testid="delete-confirm-cancel"
                  onclick={cancelDeleteConfirm}
                >
                  取消
                </button>
              </div>
            {:else}
              <button
                class="aura-console-button"
                type="button"
                aria-label={`删除配置方案 ${profile.name}`}
                data-testid="delete-profile-button"
                onclick={() => enterDeleteConfirm(profile.id)}
                disabled={store.profiles.length <= 1}
              >
                删除
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</section>
