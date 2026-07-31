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
  root: "app-toggle",
  input: "app-toggle__input",
  slider: "app-toggle__slider",
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
