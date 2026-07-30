<script setup lang="ts">
import Select from "primevue/select";

interface AppSelectOption {
  label: string;
  value: string;
  disabled?: boolean;
}

withDefaults(
  defineProps<{
    modelValue: string;
    options: AppSelectOption[];
    accessibleLabel: string;
    disabled?: boolean;
    placeholder?: string;
  }>(),
  {
    disabled: false,
    placeholder: undefined,
  },
);

defineEmits<{
  "update:modelValue": [value: string];
}>();

const selectParts = {
  root: "app-select",
  label: "app-select__label",
  dropdown: "app-select__trigger",
  dropdownIcon: "app-select__icon",
  overlay: "app-select__overlay",
  listContainer: "app-select__list-container",
  list: "app-select__list",
  option: "app-select__option",
  optionLabel: "app-select__option-label",
  emptyMessage: "app-select__empty",
};
</script>

<template>
  <Select
    unstyled
    :model-value="modelValue"
    :options="options"
    option-label="label"
    option-value="value"
    option-disabled="disabled"
    :aria-label="accessibleLabel"
    :disabled="disabled"
    :placeholder="placeholder"
    :pt="selectParts"
    @update:model-value="$emit('update:modelValue', $event)"
  />
</template>
