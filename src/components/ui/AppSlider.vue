<script setup lang="ts">
import Slider from "primevue/slider";

withDefaults(
  defineProps<{
    modelValue: number;
    min?: number;
    max?: number;
    step?: number;
    disabled?: boolean;
    accessibleLabel: string;
  }>(),
  {
    min: 0,
    max: 100,
    step: 1,
    disabled: false,
  },
);

defineEmits<{
  "update:modelValue": [value: number];
}>();

const sliderParts = {
  root: "app-slider__root",
  range: "app-slider__range",
  handle: "app-slider__handle",
};
</script>

<template>
  <Slider
    unstyled
    class="app-slider"
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
