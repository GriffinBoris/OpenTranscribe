<script setup lang="ts">
import ProgressBar from "primevue/progressbar";
import { computed } from "vue";

const props = defineProps<{
  value?: number;
  accessibleLabel: string;
}>();

const normalizedValue = computed(() =>
  props.value === undefined
    ? undefined
    : Math.min(100, Math.max(0, props.value)),
);
const progressStyle = computed(() =>
  normalizedValue.value === undefined
    ? undefined
    : {
        "--app-progress-value": String(normalizedValue.value / 100),
      },
);
</script>

<template>
  <ProgressBar
    unstyled
    class="app-progress bg-canvas-subtle relative h-2.5 overflow-hidden rounded-full"
    :class="{ 'app-progress--indeterminate': normalizedValue === undefined }"
    :value="normalizedValue"
    :mode="normalizedValue === undefined ? 'indeterminate' : 'determinate'"
    :aria-label="accessibleLabel"
    :show-value="false"
    :style="progressStyle"
    :pt="{
      value:
        normalizedValue === undefined
          ? 'h-full !w-[42%] rounded-[inherit] bg-lichen origin-left will-change-transform animate-[progress-indeterminate_850ms_linear_infinite]'
          : 'app-progress__value h-full !w-full rounded-[inherit] bg-lichen origin-left [transform:scaleX(var(--app-progress-value,0))]',
    }"
  />
</template>
