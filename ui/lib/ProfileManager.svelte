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

<section class="space-y-3 rounded-xl border border-aura-border bg-white/80 px-5 py-5">
  <div>
    <p class="aura-section-title">
      配置方案
    </p>
    <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
      保存常用翻译设置，可在设置页或托盘中快速切换。
    </p>
  </div>

  <div class="grid gap-3 md:grid-cols-[1fr_auto_auto]">
    <input
      value={draftName}
      oninput={(event) => ondraftnamechange?.((event.currentTarget as HTMLInputElement).value)}
      placeholder="配置方案名称"
      class="w-full rounded-lg border border-aura-border bg-white px-3 py-2.5 text-sm text-aura-text outline-none transition-all duration-200 placeholder:text-aura-text-muted hover:border-aura-border-accent focus:border-aura-accent focus:ring-2 focus:ring-aura-accent/15"
    />
    <button
      class="rounded-lg border border-aura-border bg-white px-3 py-2.5 text-sm font-medium text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-text"
      type="button"
      onclick={() => oncreate?.()}
    >
      另存为新方案
    </button>
    <button
      class="rounded-lg border border-aura-border bg-white px-3 py-2.5 text-sm font-medium text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-text disabled:cursor-not-allowed disabled:opacity-50"
      type="button"
      onclick={() => activeProfile && onrename?.(activeProfile.id)}
      disabled={!activeProfile}
    >
      重命名当前方案
    </button>
  </div>

  {#if !store || store.profiles.length === 0}
    <div class="rounded-md border border-dashed border-aura-border bg-aura-surface-soft px-3 py-3 text-xs leading-relaxed text-aura-text-dim">
      Aura 会自动创建一个默认翻译配置方案。
    </div>
  {:else}
    <div class="space-y-2" data-testid="profile-list">
      {#each store.profiles as profile (profile.id)}
        <div class="flex flex-wrap items-center justify-between gap-3 rounded-[14px] border border-aura-border bg-white px-3 py-3">
          <div class="min-w-0">
            <div class="flex flex-wrap items-center gap-2">
              <p class="truncate text-sm font-medium text-aura-text">{profile.name}</p>
              {#if profile.id === store.active_profile_id}
                <span class="rounded-full bg-aura-accent-soft px-2.5 py-1 text-[11px] font-display font-medium text-aura-accent">
                  当前
                </span>
              {/if}
            </div>
            <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
              {providerLabel(profile.provider)} · {profile.model} · {languageLabel(profile.source_lang)} → {languageLabel(profile.target_lang)}
            </p>
          </div>

          <div class="flex flex-wrap gap-2">
            {#if profile.id !== store.active_profile_id}
              <button
                class="rounded-md border border-aura-border bg-white px-3 py-1.5 text-[11px] font-medium text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-text"
                type="button"
                onclick={() => onactivate?.(profile.id)}
              >
                启用
              </button>
            {/if}

            {#if confirmingDeleteId === profile.id}
              <div class="flex flex-wrap items-center gap-2" data-testid="delete-confirm-group">
                <button
                  class="rounded-md border border-aura-error bg-aura-error px-3 py-1.5 text-[11px] font-display font-semibold text-white transition-colors duration-150 hover:brightness-110"
                  type="button"
                  data-testid="delete-confirm-commit"
                  onclick={() => commitDelete(profile.id)}
                >
                  确认删除
                </button>
                <button
                  class="rounded-md border border-aura-border bg-white px-3 py-1.5 text-[11px] font-medium text-aura-text-dim transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-text"
                  type="button"
                  data-testid="delete-confirm-cancel"
                  onclick={cancelDeleteConfirm}
                >
                  取消
                </button>
              </div>
            {:else}
              <button
                class="rounded-md border border-aura-border bg-white px-3 py-1.5 text-[11px] font-medium text-aura-text-dim transition-colors duration-150 hover:border-aura-error/30 hover:text-aura-error"
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
