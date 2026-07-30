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
    class="app-input"
    :model-value="modelValue"
    :type="type"
    :disabled="disabled"
    @update:model-value="$emit('update:modelValue', $event)"
  />
</template>
