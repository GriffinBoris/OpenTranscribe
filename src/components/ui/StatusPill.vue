<script setup lang="ts">
withDefaults(
  defineProps<{
    tone?: "neutral" | "local" | "cloud" | "success" | "warning" | "recording";
  }>(),
  {
    tone: "neutral",
  },
);
</script>

<template>
  <span class="status-pill" :class="`status-pill--${tone}`">
    <slot />
  </span>
</template>
