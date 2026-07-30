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
  <div class="empty-state">
    <span v-if="$slots.icon" class="empty-state__icon" aria-hidden="true">
      <slot name="icon" />
    </span>
    <strong>{{ title }}</strong>
    <span v-if="message">{{ message }}</span>
    <slot />
  </div>
</template>
