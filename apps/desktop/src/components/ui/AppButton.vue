<script setup lang="ts">
import { LoaderCircle } from "@lucide/vue";
import Button from "primevue/button";

const props = withDefaults(
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

const sizeClasses = {
  small:
    "h-[var(--control-height-small)] min-h-[var(--control-height-small)] px-2.5 py-1.5 text-sm",
  medium:
    "h-[var(--control-height-medium)] min-h-[var(--control-height-medium)] px-3.5 py-2",
  large:
    "h-[var(--control-height-large)] min-h-[var(--control-height-large)] px-5 py-2.5 text-lg",
};

const variantClasses = {
  primary:
    "bg-accent text-accent-contrast shadow-app-action hover:bg-accent-hover",
  secondary: "border-line bg-surface-raised hover:border-line-strong",
  ghost: "border-transparent bg-transparent hover:bg-canvas-subtle",
  danger: "bg-accent text-accent-contrast hover:bg-accent-hover",
};

defineEmits<{
  click: [event: MouseEvent];
}>();
</script>

<template>
  <Button
    unstyled
    class="app-button rounded-app-sm inline-flex items-center justify-center gap-2 border border-transparent font-semibold whitespace-nowrap transition-[background,border-color,transform] duration-[var(--duration-standard)] ease-[var(--easing-standard)] active:translate-y-px disabled:translate-y-0 disabled:cursor-not-allowed disabled:opacity-[var(--opacity-disabled)]"
    :class="[variantClasses[props.variant], sizeClasses[props.size]]"
    :type="props.type"
    :disabled="props.disabled"
    :loading="props.loading"
    @click="$emit('click', $event)"
  >
    <LoaderCircle
      v-if="props.loading"
      class="animate-[spin_1.4s_linear_infinite]"
      :size="15"
      aria-hidden="true"
    />
    <slot />
  </Button>
</template>
