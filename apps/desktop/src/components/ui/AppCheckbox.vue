<script setup lang="ts">
import Checkbox from "primevue/checkbox";

withDefaults(
  defineProps<{
    modelValue: boolean;
    accessibleLabel: string;
    disabled?: boolean;
  }>(),
  {
    disabled: false,
  },
);

defineEmits<{
  "update:modelValue": [value: boolean];
  change: [event: Event];
  mousedown: [event: MouseEvent];
}>();

const checkboxParts = {
  root: "app-checkbox group relative inline-grid size-5 shrink-0 place-items-center",
  input:
    "app-checkbox__input peer absolute inset-0 z-1 size-full cursor-pointer opacity-0",
  box: "app-checkbox__box grid size-5 place-items-center rounded-app-xs border border-line bg-surface-raised text-accent-contrast transition-[background,border-color,box-shadow] duration-[var(--duration-standard)] ease-[var(--easing-standard)] peer-focus-visible:shadow-[var(--shadow-focus)] group-data-[p-checked=true]:border-accent group-data-[p-checked=true]:bg-accent group-data-[p-disabled=true]:cursor-not-allowed group-data-[p-disabled=true]:opacity-[var(--opacity-disabled)]",
  icon: "app-checkbox__icon size-3",
};
</script>

<template>
  <span class="inline-flex" @mousedown="$emit('mousedown', $event)">
    <Checkbox
      unstyled
      binary
      :model-value="modelValue"
      :aria-label="accessibleLabel"
      :disabled="disabled"
      :pt="checkboxParts"
      @change="$emit('change', $event)"
      @update:model-value="$emit('update:modelValue', $event)"
    />
  </span>
</template>
