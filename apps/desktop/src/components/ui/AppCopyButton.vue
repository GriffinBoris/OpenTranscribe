<script setup lang="ts">
import { Check, Copy } from "@lucide/vue";
import { computed } from "vue";

import { useClipboard } from "@/composables/useClipboard";

import AppButton from "./AppButton.vue";

const props = withDefaults(
  defineProps<{
    text: string;
    label: string;
    copiedLabel: string;
    size?: "small" | "medium" | "large";
    variant?: "primary" | "secondary" | "ghost" | "danger";
    disabled?: boolean;
  }>(),
  {
    size: "small",
    variant: "ghost",
    disabled: false,
  },
);

const { copiedId, copy } = useClipboard();
const copied = computed(() => copiedId.value === props.text);

async function copyText() {
  await copy(props.text);
}
</script>

<template>
  <AppButton
    :size="size"
    :variant="variant"
    :disabled="disabled"
    :aria-label="copied ? copiedLabel : label"
    :title="copied ? copiedLabel : label"
    @click="copyText"
  >
    <Check v-if="copied" :size="15" aria-hidden="true" />
    <Copy v-else :size="15" aria-hidden="true" />
  </AppButton>
</template>
