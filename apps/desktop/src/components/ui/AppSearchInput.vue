<script setup lang="ts">
import { Search } from "@lucide/vue";
import IconField from "primevue/iconfield";
import InputIcon from "primevue/inputicon";
import { computed, useAttrs } from "vue";

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
const inputAttrs = computed(() => {
  const attributes = { ...attrs };

  delete attributes.class;
  delete attributes.style;

  return attributes;
});
</script>

<template>
  <IconField
    unstyled
    class="app-search-input relative block w-full"
    :class="[props.fullWidth ? 'max-w-none' : 'max-w-[460px]', attrs.class]"
    :style="attrs.style"
  >
    <InputIcon
      unstyled
      class="app-search-input__icon text-ink-muted pointer-events-none absolute inset-y-0 left-3 z-1 flex w-4 items-center justify-center"
    >
      <Search class="block size-4" :stroke-width="2" />
    </InputIcon>
    <AppInputText
      v-bind="inputAttrs"
      class="pl-[calc(var(--space-3)+16px+var(--space-2))]"
      type="search"
      :model-value="props.modelValue"
      :disabled="props.disabled"
      @update:model-value="$emit('update:modelValue', $event)"
    />
  </IconField>
</template>
