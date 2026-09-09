<script setup lang="ts">
withDefaults(
  defineProps<{
    title: string;
    message?: string;
  }>(),
  {
    message: "",
  },
);
</script>

<template>
  <div
    class="empty-state text-ink grid min-h-[190px] place-content-center justify-items-center gap-2 px-6 py-9 text-center leading-[var(--line-height-body)]"
  >
    <span
      v-if="$slots.icon"
      class="empty-state__icon rounded-app-lg bg-lichen-soft text-lichen mb-0.5 grid size-[42px] place-items-center"
      aria-hidden="true"
    >
      <slot name="icon" />
    </span>
    <strong class="text-ink text-base">{{ title }}</strong>
    <span v-if="message" class="text-ink-muted max-w-[360px] text-sm">{{
      message
    }}</span>
    <slot />
  </div>
</template>
