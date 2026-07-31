<script setup lang="ts">
import { Search } from "@lucide/vue";
import IconField from "primevue/iconfield";
import InputIcon from "primevue/inputicon";
import { useAttrs } from "vue";

import AppInputText from "@/components/ui/AppInputText.vue";

defineOptions({ inheritAttrs: false });

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
  "update:modelValue": [value: string | undefined];
}>();

const attrs = useAttrs();
</script>

<template>
  <IconField unstyled class="app-search-input">
    <InputIcon unstyled class="app-search-input__icon">
      <Search :size="16" />
    </InputIcon>
    <AppInputText
      v-bind="attrs"
      type="search"
      :model-value="modelValue"
      :disabled="disabled"
      @update:model-value="$emit('update:modelValue', $event)"
    />
  </IconField>
</template>
