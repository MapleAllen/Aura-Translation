<script lang="ts">
  import type { ProviderProbeResult, RuntimeStatus } from './runtimeStatus';

  type Props = {
    status: RuntimeStatus | null;
    testing: boolean;
    probeResult: ProviderProbeResult | null;
    ontest: () => void;
    /** True while the user still has to finish first-run onboarding. */
    onboarding?: boolean;
    /** Whether a service has been chosen and its endpoint/model are present. */
    serviceChosen?: boolean;
    /** Whether the credential for the chosen service is present. */
    credentialPresent?: boolean;
    /** Whether a trial translation has already succeeded. */
    trialDone?: boolean;
    trialRunning?: boolean;
    trialMessage?: string;
    hotkeyLabel?: string;
    ontrial?: () => void;
    ongoadvanced?: () => void;
  };

  let {
    status,
    testing,
    probeResult,
    ontest,
    onboarding = false,
    serviceChosen = false,
    credentialPresent = false,
    trialDone = false,
    trialRunning = false,
    trialMessage = '',
    hotkeyLabel = '',
    ontrial,
    ongoadvanced,
  }: Props = $props();

  const trialReady = $derived(serviceChosen && credentialPresent && !trialRunning);

  function stepState(index: number) {
    if (index === 1) return serviceChosen ? 'done' : 'active';
    if (index === 2) {
      if (!serviceChosen) return 'todo';
      return credentialPresent ? 'done' : 'active';
    }
    if (!credentialPresent) return 'todo';
    return trialDone ? 'done' : 'active';
  }

  const stepClass: Record<string, string> = {
    done: 'border-aura-success/40 bg-aura-success/5',
    active: 'border-aura-border-accent bg-aura-accent-soft',
    todo: 'border-aura-border bg-aura-surface-soft/60',
  };

  const stepBadge: Record<string, string> = {
    done: 'border-aura-success/40 text-aura-success',
    active: 'border-aura-border-accent text-aura-accent',
    todo: 'border-aura-border text-aura-text-muted',
  };
</script>

<section class="space-y-4" data-testid="runtime-status-card">
  {#if onboarding}
    <div class="flex flex-wrap items-start justify-between gap-3">
      <div>
        <p class="aura-section-title">开始使用</p>
        <p class="mt-1 text-sm leading-6 text-aura-text-dim">
          三步即可完成设置。服务地址和模型已经预填，通常只需要填入你自己的凭据。
        </p>
      </div>
      <span class="rounded-lg border border-aura-border-accent bg-aura-accent-soft px-2.5 py-1 text-xs font-display font-semibold tracking-[0.06em] text-aura-accent">
        首次设置
      </span>
    </div>

    <ol class="space-y-3" data-testid="onboarding-steps">
      <li
        class={`rounded-lg border px-4 py-3.5 ${stepClass[stepState(1)]}`}
        data-testid="onboarding-step-service"
        data-state={stepState(1)}
      >
        <div class="flex items-start gap-3">
          <span class={`mt-0.5 inline-flex h-5 w-5 shrink-0 items-center justify-center rounded-full border text-xs font-semibold ${stepBadge[stepState(1)]}`}>
            {stepState(1) === 'done' ? '✓' : '1'}
          </span>
          <div class="min-w-0">
            <p class="text-sm font-medium text-aura-text">选择服务</p>
            <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
              DeepSeek、OpenRouter，或本地 Ollama。默认地址与模型已填好，自定义地址在高级设置里。
            </p>
          </div>
        </div>
      </li>

      <li
        class={`rounded-lg border px-4 py-3.5 ${stepClass[stepState(2)]}`}
        data-testid="onboarding-step-credential"
        data-state={stepState(2)}
      >
        <div class="flex flex-col gap-3">
          <div class="flex items-start gap-3">
            <span class={`mt-0.5 inline-flex h-5 w-5 shrink-0 items-center justify-center rounded-full border text-xs font-semibold ${stepBadge[stepState(2)]}`}>
              {stepState(2) === 'done' ? '✓' : '2'}
            </span>
            <div class="min-w-0">
              <p class="text-sm font-medium text-aura-text">填入凭据并验证</p>
              <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
                需要你已有对应的服务账户。凭据按系统方式保存，不会写进明文配置。
              </p>
            </div>
          </div>

          <div class="flex flex-wrap items-center gap-3 pl-8">
            <button
              class="aura-console-button"
              onclick={ontest}
              disabled={testing || !serviceChosen}
              data-testid="onboarding-probe-button"
              type="button"
            >
              {testing ? '验证中...' : '验证凭据'}
            </button>
            {#if probeResult}
              <p
                class={`text-xs leading-relaxed ${probeResult.ok ? 'text-aura-success' : 'text-aura-error'}`}
                data-testid="provider-probe-message"
              >
                {probeResult.message}
              </p>
            {/if}
          </div>
        </div>
      </li>

      <li
        class={`rounded-lg border px-4 py-3.5 ${stepClass[stepState(3)]}`}
        data-testid="onboarding-step-trial"
        data-state={stepState(3)}
      >
        <div class="flex flex-col gap-3">
          <div class="flex items-start gap-3">
            <span class={`mt-0.5 inline-flex h-5 w-5 shrink-0 items-center justify-center rounded-full border text-xs font-semibold ${stepBadge[stepState(3)]}`}>
              {stepState(3) === 'done' ? '✓' : '3'}
            </span>
            <div class="min-w-0">
              <p class="text-sm font-medium text-aura-text">试译一条</p>
              <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
                真实发一次翻译请求，确认整条链路可用。
              </p>
            </div>
          </div>

          <div class="flex flex-wrap items-center gap-3 pl-8">
            <button
              class="aura-console-button"
              data-variant="primary"
              onclick={ontrial}
              disabled={!trialReady}
              data-testid="onboarding-trial-button"
              type="button"
            >
              {trialRunning ? '试译中...' : trialDone ? '再试一次' : '试译一条示例'}
            </button>
            {#if trialMessage}
              <p
                class={`text-xs leading-relaxed ${trialDone ? 'text-aura-success' : 'text-aura-error'}`}
                data-testid="onboarding-trial-message"
              >
                {trialMessage}
              </p>
            {/if}
          </div>
        </div>
      </li>
    </ol>

    {#if trialDone}
      <div
        class="rounded-lg border border-aura-border bg-aura-surface-soft px-4 py-3.5"
        data-testid="onboarding-success"
      >
        <p class="text-sm font-medium text-aura-text">可以开始用了</p>
        <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
          复制文字后按 <span class="font-mono text-aura-text">{hotkeyLabel || '快捷键'}</span> 就能看到译文，再按一次收起，Esc 也能收起。
          菜单栏图标可以直接打开翻译、暂停自动翻译或退出。
        </p>
      </div>
    {/if}

    {#if ongoadvanced}
      <button
        class="text-xs font-medium text-aura-text-dim transition-colors duration-150 hover:text-aura-accent"
        type="button"
        onclick={ongoadvanced}
      >
        需要自定义地址、模型列表或多套配置？前往高级设置
      </button>
    {/if}
  {:else}
    <div class="flex flex-wrap items-start justify-between gap-3">
      <div>
        <p class="aura-section-title">就绪状态</p>
        <p class="mt-1 text-sm leading-6 text-aura-text-dim">
          {status?.summary ?? '正在检查 Aura 是否已准备好翻译...'}
        </p>
      </div>

      <span class={`rounded-lg border px-2.5 py-1 text-xs font-display font-semibold tracking-[0.06em] ${
        status?.level === 'ready'
          ? 'border-aura-success/30 bg-aura-success/5 text-aura-success'
          : 'border-aura-error/30 bg-aura-error/5 text-aura-error'
      }`}>
        {status?.level === 'ready' ? '已就绪' : '需要配置'}
      </span>
    </div>

    {#if status}
      <div class="overflow-hidden rounded-lg border border-aura-border bg-aura-surface-soft/70">
        {#each status.checklist as item, index}
          <div
            class={`flex items-center gap-3 px-3.5 py-2.5 text-xs ${
              index === 0 ? '' : 'border-t border-aura-border'
            } ${item.ok ? 'text-aura-text' : 'text-aura-error/90'}`}
          >
            <span class={`inline-flex h-4 w-4 items-center justify-center text-xs font-semibold ${
              item.ok ? 'text-aura-success' : 'text-aura-error'
            }`}>
              {item.ok ? '✓' : '!'}
            </span>
            <span>{item.label}</span>
          </div>
        {/each}
      </div>
    {/if}

    <div class="flex flex-wrap items-center justify-between gap-4 rounded-lg border border-aura-border bg-aura-surface-soft px-4 py-3.5">
      <div class="min-w-0">
        <p class="text-sm font-medium text-aura-text">测试服务商连接</p>
        <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
          使用当前设置发起一次轻量请求，保存前也可以测试。
        </p>
        {#if probeResult}
          <p
            class={`mt-2 text-xs leading-relaxed ${probeResult.ok ? 'text-aura-success' : 'text-aura-error'}`}
            data-testid="provider-probe-message"
          >
            {probeResult.message}
          </p>
        {/if}
      </div>

      <button
        class="aura-console-button"
        onclick={ontest}
        disabled={testing}
        data-testid="probe-provider-button"
        type="button"
      >
        {testing ? '测试中...' : '测试服务商'}
      </button>
    </div>
  {/if}
</section>
