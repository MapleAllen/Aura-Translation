<script lang="ts">
  import type { AppNotification } from './notifications';

  type Props = {
    notifications: AppNotification[];
    ondismiss: (id: string) => void;
  };

  let { notifications, ondismiss }: Props = $props();

  const accentByKind = {
    error: 'border-aura-error bg-white text-aura-text',
    warning: 'border-[#d39d2f] bg-[#fffaf0] text-aura-text',
    info: 'border-aura-accent bg-white text-aura-text',
  } as const;

  const iconByKind = {
    error: '!',
    warning: '!',
    info: 'i',
  } as const;
</script>

{#if notifications.length > 0}
  <div class="pointer-events-none absolute inset-x-2 top-2 z-60 px-2 pt-2">
    <div class="flex flex-col items-end gap-2">
      {#each notifications as notification (notification.id)}
        <section
          class={`pointer-events-auto w-full max-w-[420px] border-l-2 border-y border-r px-3 py-2 shadow-[0_6px_14px_rgba(25,39,54,0.08)] ${accentByKind[notification.kind]}`}
          role="alert"
          aria-live="assertive"
          data-testid="notification-strip"
        >
          <div class="flex items-start gap-3">
            <div class="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center border border-current/10 text-[10px] font-semibold text-aura-text-dim">
              {iconByKind[notification.kind]}
            </div>
            <div class="min-w-0 flex-1">
              <p class="text-[11px] font-display font-semibold tracking-[0.08em]">
                {notification.title}
              </p>
              <p class="mt-0.5 break-words text-xs leading-relaxed text-aura-text-dim">
                {notification.message}
              </p>
            </div>
            <button
              class="flex h-6 w-6 shrink-0 items-center justify-center text-aura-text-muted transition-colors hover:text-aura-text"
              type="button"
              aria-label={`关闭通知：${notification.title}`}
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
