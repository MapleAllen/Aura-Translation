<script lang="ts">
  /**
   * SkeletonLoader — Pulsating loading state with shimmer bars.
   * Shows 4 bars with staggered widths and animation delays.
   */
</script>

<div class="flex flex-col gap-2.5 py-1">
  <div class="skeleton-bar h-4 w-full" style="animation-delay: 0ms;"></div>
  <div class="skeleton-bar h-4 w-[88%]" style="animation-delay: 120ms;"></div>
  <div class="skeleton-bar h-4 w-[72%]" style="animation-delay: 240ms;"></div>
  <div class="skeleton-bar h-3.5 w-[55%]" style="animation-delay: 360ms;"></div>
</div>
