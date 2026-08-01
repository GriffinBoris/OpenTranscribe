<script setup lang="ts">
import InputText from "primevue/inputtext";

withDefaults(
  defineProps<{
    modelValue?: string;
    type?: "text" | "password" | "search";
    disabled?: boolean;
  }>(),
  {
    modelValue: "",
    type: "text",
    disabled: false,
  },
);

defineEmits<{
  "update:modelValue": [value: string | undefined];
}>();
</script>

<template>
  <InputText
    unstyled
    class="app-input rounded-app-sm border-line bg-surface-raised text-ink placeholder:text-ink-faint focus:border-accent h-[var(--control-height-medium)] min-h-[var(--control-height-medium)] w-full border px-3 py-2 transition-[border-color,box-shadow] duration-[var(--duration-standard)] ease-[var(--easing-standard)] focus:shadow-[var(--shadow-focus)] focus:outline-none disabled:cursor-not-allowed disabled:opacity-[var(--opacity-disabled)]"
    :model-value="modelValue"
    :type="type"
    :disabled="disabled"
    @update:model-value="$emit('update:modelValue', $event)"
  />
</template>
