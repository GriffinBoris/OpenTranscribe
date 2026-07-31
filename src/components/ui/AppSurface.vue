<script setup lang="ts">
withDefaults(
  defineProps<{
    interactive?: boolean;
    padded?: boolean;
  }>(),
  {
    interactive: false,
    padded: true,
  },
);
</script>

<template>
  <section
    class="app-surface rounded-app-2xl border-line bg-surface overflow-hidden border shadow-[var(--shadow-surface)]"
    :class="[
      padded && 'p-[var(--space-5-5)]',
      interactive &&
        'hover:border-line-strong transition-[border-color,transform] duration-[var(--duration-slow)] ease-[var(--easing-standard)] hover:-translate-y-px',
    ]"
  >
    <slot />
  </section>
</template>
