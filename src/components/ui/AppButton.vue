<script setup lang="ts">
import { LoaderCircle } from "@lucide/vue";
import Button from "primevue/button";

withDefaults(
  defineProps<{
    variant?: "primary" | "secondary" | "ghost" | "danger";
    size?: "small" | "medium" | "large";
    type?: "button" | "submit";
    disabled?: boolean;
    loading?: boolean;
  }>(),
  {
    variant: "secondary",
    size: "medium",
    type: "button",
    disabled: false,
    loading: false,
  },
);

defineEmits<{
  click: [event: MouseEvent];
}>();
</script>

<template>
  <Button
    unstyled
    class="app-button"
    :class="[`app-button--${variant}`, `app-button--${size}`]"
    :type="type"
    :disabled="disabled"
    :loading="loading"
    @click="$emit('click', $event)"
  >
    <LoaderCircle v-if="loading" class="spin" :size="15" aria-hidden="true" />
    <slot />
  </Button>
</template>
