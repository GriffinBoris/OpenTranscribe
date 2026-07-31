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
  root: "app-select relative flex min-h-[var(--control-height-medium)] min-w-0 items-center rounded-app-sm border border-line bg-surface-raised text-ink transition-[border-color,box-shadow] duration-[var(--duration-standard)] ease-[var(--easing-standard)] focus-within:border-accent focus-within:shadow-[var(--shadow-focus)] data-[p-disabled=true]:cursor-not-allowed data-[p-disabled=true]:opacity-[var(--opacity-disabled)]",
  label:
    "app-select__label min-w-0 flex-1 overflow-hidden px-3 py-2 pr-1.5 text-ellipsis whitespace-nowrap outline-none",
  dropdown:
    "app-select__trigger grid w-[34px] shrink-0 self-stretch place-items-center text-ink-muted",
  dropdownIcon: "app-select__icon size-[13px]",
  overlay:
    "app-select__overlay z-[var(--layer-popover)] mt-[5px] min-w-[150px] overflow-hidden rounded-app-md border border-line bg-surface-raised shadow-app",
  listContainer: "app-select__list-container max-h-[260px] overflow-auto",
  list: "app-select__list grid list-none gap-0.5 p-[5px]",
  option:
    "app-select__option cursor-pointer rounded-app-xs px-2.5 py-2 text-md text-ink data-[p-focused=true]:bg-canvas-subtle data-[p-selected=true]:bg-canvas-subtle data-[p-selected=true]:font-bold data-[p-selected=true]:text-lichen data-[p-disabled=true]:cursor-not-allowed data-[p-disabled=true]:text-ink-faint",
  optionLabel: "app-select__option-label",
  emptyMessage: "app-select__empty p-2.5 text-ink-muted",
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
