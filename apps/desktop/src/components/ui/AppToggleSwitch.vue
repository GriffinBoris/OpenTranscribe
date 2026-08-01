<script setup lang="ts">
import ToggleSwitch from "primevue/toggleswitch";

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
}>();

const toggleParts = {
  root: "app-toggle relative inline-flex h-[22px] w-[38px] justify-self-end rounded-full bg-line-strong transition-colors duration-[var(--duration-standard)] ease-[var(--easing-standard)] data-[p-checked=true]:bg-lichen data-[p-disabled=true]:cursor-not-allowed data-[p-disabled=true]:opacity-[var(--opacity-disabled)]",
  input:
    "app-toggle__input absolute inset-0 z-1 m-0 size-full cursor-inherit opacity-0",
  slider:
    "app-toggle__slider pointer-events-none absolute top-[3px] left-[3px] size-4 rounded-full bg-surface-raised shadow-[0_1px_3px_color-mix(in_srgb,var(--text)_24%,transparent)] transition-transform duration-[var(--duration-standard)] ease-[var(--easing-standard)]",
};
</script>

<template>
  <ToggleSwitch
    unstyled
    :model-value="modelValue"
    :aria-label="accessibleLabel"
    :disabled="disabled"
    :pt="toggleParts"
    @update:model-value="$emit('update:modelValue', $event)"
  />
</template>
