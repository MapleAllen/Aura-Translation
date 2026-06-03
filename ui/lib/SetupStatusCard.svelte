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

<section class="space-y-4" data-testid="runtime-status-card">
  <div class="flex flex-wrap items-start justify-between gap-3">
    <div>
      <p class="aura-section-title">就绪状态</p>
      <p class="mt-1 text-sm leading-6 text-aura-text-dim">
        {status?.summary ?? '正在检查 Aura 是否已准备好翻译...'}
      </p>
    </div>

    <span class={`border px-2 py-1 text-[11px] font-display font-semibold tracking-[0.08em] ${
      status?.level === 'ready'
        ? 'border-[#d9efe4] bg-[#f5fbf8] text-aura-success'
        : 'border-[#f4d4da] bg-[#fff8f9] text-aura-error'
    }`}>
      {status?.level === 'ready' ? '已就绪' : '需要配置'}
    </span>
  </div>

  {#if status}
    <div class="overflow-hidden border border-aura-border">
      {#each status.checklist as item, index}
        <div
          class={`flex items-center gap-3 px-3 py-2 text-xs ${
            index === 0 ? '' : 'border-t border-aura-border'
          } ${item.ok ? 'text-aura-text' : 'text-aura-error/90'}`}
        >
          <span class={`inline-flex h-4 w-4 items-center justify-center text-[10px] font-semibold ${
            item.ok ? 'text-aura-success' : 'text-aura-error'
          }`}>
            {item.ok ? '✓' : '!'}
          </span>
          <span>{item.label}</span>
        </div>
      {/each}
    </div>
  {/if}

  <div class="flex flex-wrap items-center justify-between gap-4 border border-aura-border bg-aura-surface-soft px-4 py-3">
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
</section>
