<script setup lang="ts">
import Textarea from "primevue/textarea";

withDefaults(
  defineProps<{
    modelValue?: string;
    disabled?: boolean;
  }>(),
  {
    modelValue: "",
    disabled: false,
  },
);

defineEmits<{
  "update:modelValue": [value: string];
}>();
</script>

<template>
  <Textarea
    unstyled
    class="app-textarea rounded-app-sm border-line bg-surface-raised text-ink placeholder:text-ink-faint focus:border-accent w-full border p-3 transition-[border-color,box-shadow] duration-[var(--duration-standard)] ease-[var(--easing-standard)] focus:shadow-[var(--shadow-focus)] focus:outline-none disabled:cursor-not-allowed disabled:opacity-[var(--opacity-disabled)]"
    :model-value="modelValue"
    :disabled="disabled"
    @update:model-value="$emit('update:modelValue', $event)"
  />
</template>
