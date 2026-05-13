<script lang="ts">
  import type { AppNotification } from './notifications';

  type Props = {
    notifications: AppNotification[];
    ondismiss: (id: string) => void;
  };

  let { notifications, ondismiss }: Props = $props();

  const accentByKind = {
    error: 'border-aura-error/60 bg-[#2a151d]/94 text-aura-text',
    warning: 'border-[#d6a743]/60 bg-[#2a2216]/94 text-aura-text',
    info: 'border-aura-accent/55 bg-[#1a1830]/94 text-aura-text',
  } as const;
  const iconByKind = {
    error: '!',
    warning: '!',
    info: 'i',
  } as const;
</script>

{#if notifications.length > 0}
  <div class="absolute inset-x-0 top-0 z-60 pointer-events-none px-3 pt-3">
    <div class="flex flex-col gap-2 items-end">
      {#each notifications as notification (notification.id)}
        <section
          class={`pointer-events-auto w-full max-w-[360px] rounded-lg border shadow-lg ${accentByKind[notification.kind]}`}
          role="alert"
          aria-live="assertive"
          data-testid="notification-card"
        >
          <div class="flex items-start gap-3 px-3 py-2.5">
            <div class="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-full border border-white/12 bg-white/6 text-[11px] font-semibold">
              {iconByKind[notification.kind]}
            </div>
            <div class="min-w-0 flex-1">
              <p class="text-xs font-display font-semibold tracking-wide">{notification.title}</p>
              <p class="mt-1 text-xs leading-relaxed text-aura-text-dim break-words">
                {notification.message}
              </p>
            </div>
            <button
              class="flex h-6 w-6 shrink-0 items-center justify-center rounded-md text-aura-text-dim hover:bg-white/6 hover:text-aura-text transition-colors"
              type="button"
              aria-label={`Dismiss ${notification.title}`}
              data-testid={`dismiss-${notification.id}`}
              onclick={() => ondismiss(notification.id)}
            >
              <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </section>
      {/each}
    </div>
  </div>
{/if}
