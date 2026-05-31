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
  class="space-y-3 rounded-lg border border-aura-border bg-white/84 px-4 py-4"
  data-testid="runtime-status-card"
>
  <div class="flex items-start justify-between gap-3">
    <div>
      <p class="text-[10px] font-display font-semibold uppercase tracking-[0.22em] text-aura-text-muted">
        Readiness
      </p>
      <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
        {status?.summary ?? 'Checking whether Aura is ready to translate...'}
      </p>
    </div>

    <span
      class={`shrink-0 rounded-full px-2.5 py-1 text-[10px] font-display font-semibold uppercase tracking-[0.14em] ${
        status?.level === 'ready'
          ? 'bg-[#eaf8f2] text-aura-success'
          : 'bg-[#fff4f6] text-aura-error'
      }`}
    >
      {status?.level === 'ready' ? 'Ready' : 'Action needed'}
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

  <div class="flex items-center justify-between gap-3 rounded-md border border-aura-border/80 bg-aura-surface-soft px-3 py-3">
    <div class="min-w-0">
      <p class="text-xs font-medium text-aura-text">Test provider access</p>
      <p class="mt-1 text-xs leading-relaxed text-aura-text-dim">
        Runs a lightweight request using the current settings, even before you save them.
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
      class="shrink-0 rounded-lg border border-aura-border bg-white px-3 py-2 text-xs font-display font-medium text-aura-text transition-colors duration-150 hover:border-aura-border-accent hover:text-aura-accent disabled:cursor-not-allowed disabled:opacity-55"
      onclick={ontest}
      disabled={testing}
      data-testid="probe-provider-button"
      type="button"
    >
      {testing ? 'Testing...' : 'Test provider'}
    </button>
  </div>
</section>
