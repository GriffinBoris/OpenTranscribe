<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    tone?: "neutral" | "local" | "cloud" | "success" | "warning" | "recording";
  }>(),
  {
    tone: "neutral",
  },
);

const toneClasses = {
  neutral: "bg-canvas-subtle text-ink",
  local: "bg-lichen-soft text-ink",
  cloud: "bg-cloud-soft text-cloud",
  success: "bg-lichen-soft text-ink",
  warning: "bg-warning-soft text-ink",
  recording: "bg-accent-soft text-accent",
};
</script>

<template>
  <span
    class="status-pill inline-flex w-fit items-center gap-1.5 rounded-full px-2 py-1 text-xs leading-none font-semibold whitespace-nowrap"
    :class="toneClasses[props.tone]"
  >
    <slot />
  </span>
</template>
