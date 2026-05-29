<script lang="ts">
  import type { AppNotification } from './notifications';

  type Props = {
    notifications: AppNotification[];
    ondismiss: (id: string) => void;
  };

  let { notifications, ondismiss }: Props = $props();

  const accentByKind = {
    error: 'border-l-aura-error bg-white/92 text-aura-text',
    warning: 'border-l-[#d39d2f] bg-[#fffaf0] text-aura-text',
    info: 'border-l-aura-accent bg-white/92 text-aura-text',
  } as const;
  const iconByKind = {
    error: '!',
    warning: '!',
    info: 'i',
  } as const;
</script>

{#if notifications.length > 0}
  <div class="pointer-events-none absolute inset-x-0 top-0 z-60 px-3 pt-3">
    <div class="flex flex-col items-end gap-2">
      {#each notifications as notification (notification.id)}
        <section
          class={`pointer-events-auto w-full max-w-[388px] rounded-lg border border-aura-border border-l-4 shadow-[0_10px_22px_rgba(89,104,129,0.12)] ${accentByKind[notification.kind]}`}
          role="alert"
          aria-live="assertive"
          data-testid="notification-card"
        >
          <div class="flex items-start gap-3 px-4 py-3">
            <div class="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-md bg-black/[0.04] text-[11px] font-semibold text-aura-text-dim">
              {iconByKind[notification.kind]}
            </div>
            <div class="min-w-0 flex-1">
              <p class="text-xs font-display font-semibold tracking-wide">{notification.title}</p>
              <p class="mt-1 break-words text-xs leading-relaxed text-aura-text-dim">
                {notification.message}
              </p>
            </div>
            <button
              class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-aura-text-muted transition-colors hover:bg-black/[0.04] hover:text-aura-text"
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
