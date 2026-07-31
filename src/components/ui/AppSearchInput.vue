<script setup lang="ts">
import { Search } from "@lucide/vue";
import IconField from "primevue/iconfield";
import InputIcon from "primevue/inputicon";
import { useAttrs } from "vue";

import AppInputText from "@/components/ui/AppInputText.vue";

defineOptions({ inheritAttrs: false });

const props = withDefaults(
  defineProps<{
    modelValue?: string;
    disabled?: boolean;
    fullWidth?: boolean;
  }>(),
  {
    modelValue: "",
    disabled: false,
    fullWidth: false,
  },
);

defineEmits<{
  "update:modelValue": [value: string | undefined];
}>();

const attrs = useAttrs();
</script>

<template>
  <IconField
    unstyled
    class="app-search-input relative block w-full"
    :class="props.fullWidth ? 'max-w-none' : 'max-w-[460px]'"
  >
    <InputIcon
      unstyled
      class="app-search-input__icon text-ink-muted pointer-events-none absolute top-1/2 left-3 z-1 grid size-4 shrink-0 -translate-y-1/2 place-items-center"
    >
      <Search class="block size-4" :stroke-width="2" />
    </InputIcon>
    <AppInputText
      v-bind="attrs"
      class="pl-[calc(var(--space-3)+16px+var(--space-2))]"
      type="search"
      :model-value="props.modelValue"
      :disabled="props.disabled"
      @update:model-value="$emit('update:modelValue', $event)"
    />
  </IconField>
</template>
