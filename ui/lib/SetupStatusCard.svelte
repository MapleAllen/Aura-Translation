<script lang="ts">
  import type { ProviderProbeResult, RuntimeStatus } from './runtimeStatus';

  type Props = {
    status: RuntimeStatus | null;
    testing: boolean;
    probeResult: ProviderProbeResult | null;
    ontest: () => void;
  };

  let { status, testing, probeResult, ontest }: Props = $props();
</script>

<section
  class="space-y-3 rounded-xl border border-aura-border bg-white/84 px-5 py-5"
  data-testid="runtime-status-card"
>
  <div class="flex items-start justify-between gap-3">
    <div>
      <p class="aura-section-title">
        就绪状态
      </p>
      <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
        {status?.summary ?? '正在检查 Aura 是否已准备好翻译...'}
      </p>
    </div>

    <span
      class={`shrink-0 rounded-full px-2.5 py-1 text-[10px] font-display font-semibold uppercase tracking-[0.14em] ${
        status?.level === 'ready'
          ? 'bg-[#eaf8f2] text-aura-success'
          : 'bg-[#fff4f6] text-aura-error'
      }`}
    >
      {status?.level === 'ready' ? '已就绪' : '需要配置'}
    </span>
  </div>

  {#if status}
    <div class="grid gap-2 text-xs text-aura-text-dim sm:grid-cols-2">
      {#each status.checklist as item}
        <div
          class={`rounded-md border px-3 py-2 ${
            item.ok
              ? 'border-[#d9efe4] bg-[#f5fbf8] text-aura-text'
              : 'border-[#f4d4da] bg-[#fff8f9] text-aura-text'
          }`}
        >
          <span class={`mr-2 inline-flex h-4 w-4 items-center justify-center rounded-full text-[10px] font-semibold ${item.ok ? 'bg-[#dff3e8] text-aura-success' : 'bg-[#fde6ea] text-aura-error'}`}>
            {item.ok ? '✓' : '!'}
          </span>
          {item.label}
        </div>
      {/each}
    </div>
  {/if}

  <div class="flex items-center justify-between gap-4 rounded-lg border border-aura-border/80 bg-aura-surface-soft px-4 py-4">
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
      class="shrink-0 rounded-lg border border-aura-border bg-white px-4 py-2.5 text-xs font-display font-medium text-aura-text transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-accent disabled:cursor-not-allowed disabled:opacity-55"
      onclick={ontest}
      disabled={testing}
      data-testid="probe-provider-button"
      type="button"
    >
      {testing ? '测试中...' : '测试服务商'}
    </button>
  </div>
</section>
