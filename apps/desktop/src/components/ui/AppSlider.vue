<script setup lang="ts">
import { computed } from "vue";

import Slider from "primevue/slider";

const props = withDefaults(
  defineProps<{
    modelValue: number;
    min?: number;
    max?: number;
    step?: number;
    disabled?: boolean;
    trackless?: boolean;
    accessibleLabel: string;
  }>(),
  {
    min: 0,
    max: 100,
    step: 1,
    disabled: false,
    trackless: false,
  },
);

defineEmits<{
  "update:modelValue": [value: number];
}>();

const sliderParts = computed(() => ({
  root: props.trackless
    ? "relative h-[42px] w-full cursor-pointer bg-transparent"
    : "relative h-1 w-full cursor-pointer rounded-full bg-line-strong",
  range: props.trackless
    ? "absolute h-full bg-transparent"
    : "absolute h-full rounded-[inherit] bg-accent",
  handle: props.trackless
    ? "absolute top-0 h-full w-5 -translate-x-1/2 cursor-ew-resize before:absolute before:inset-y-0 before:left-1/2 before:w-0.5 before:-translate-x-1/2 before:bg-accent before:shadow-[0_0_0_1px_var(--surface-raised)] focus-visible:before:w-1"
    : "absolute top-1/2 size-3.5 -translate-x-1/2 -translate-y-1/2 rounded-full border-[3px] border-surface-raised bg-accent shadow-[0_0_0_1px_var(--border-strong)] focus-visible:outline-3 focus-visible:outline-accent focus-visible:outline-offset-2",
}));
</script>

<template>
  <Slider
    unstyled
    class="app-slider flex min-w-[120px] flex-1 items-center aria-disabled:cursor-default aria-disabled:opacity-[var(--opacity-disabled)]"
    :model-value="modelValue"
    :min="min"
    :max="max"
    :step="step"
    :disabled="disabled"
    :aria-label="accessibleLabel"
    :pt="sliderParts"
    @update:model-value="
      $emit('update:modelValue', Array.isArray($event) ? $event[0] : $event)
    "
  />
</template>
