<script setup lang="ts">
import { computed, ref } from "vue";
import { ChevronDown } from "@lucide/vue";
import Button from "primevue/button";
import Menu from "primevue/menu";
import type { MenuItem } from "primevue/menuitem";

interface SplitButtonOption {
  label: string;
  value: string;
  disabled?: boolean;
}

const props = withDefaults(
  defineProps<{
    options: SplitButtonOption[];
    disabled?: boolean;
    accessibleLabel: string;
  }>(),
  {
    disabled: false,
  },
);

const emit = defineEmits<{
  click: [event: Event];
  select: [value: string];
}>();
const menu = ref<{ toggle: (event: Event) => void } | null>(null);

const menuItems = computed<MenuItem[]>(() =>
  props.options.map((option) => ({
    label: option.label,
    disabled: option.disabled,
    command: () => emit("select", option.value),
  })),
);

const menuParts = {
  root: "z-[var(--layer-popover)] mt-[5px] min-w-[255px] overflow-hidden rounded-app-md border border-line bg-surface-raised text-ink shadow-app",
  list: "grid list-none gap-0.5 p-[5px]",
  item: "data-[p-disabled=true]:cursor-not-allowed data-[p-disabled=true]:text-ink-faint",
  itemContent:
    "rounded-app-xs hover:bg-canvas-subtle data-[p-focused=true]:bg-canvas-subtle",
  itemLink: "flex p-2.5 text-inherit no-underline",
  itemLabel: "app-split-button__menu-item-label",
};

function toggleMenu(event: Event) {
  menu.value?.toggle(event);
}
</script>

<template>
  <div class="inline-flex min-h-[var(--control-height-large)]">
    <Button
      unstyled
      class="rounded-l-app-sm bg-accent text-accent-contrast shadow-app-action hover:bg-accent-hover flex flex-1 items-center justify-center gap-2 border-0 px-[var(--space-4-5)] py-2.5 text-lg font-semibold transition-colors duration-[var(--duration-standard)] ease-[var(--easing-standard)] active:translate-y-px disabled:cursor-not-allowed disabled:opacity-[var(--opacity-disabled)]"
      :disabled="disabled"
      :aria-label="accessibleLabel"
      @click="$emit('click', $event)"
    >
      <slot />
    </Button>
    <Button
      unstyled
      class="rounded-r-app-sm bg-accent text-accent-contrast shadow-app-action hover:bg-accent-hover flex w-10 items-center justify-center border-0 border-l border-[var(--control-divider)] transition-colors duration-[var(--duration-standard)] ease-[var(--easing-standard)] active:translate-y-px disabled:cursor-not-allowed disabled:opacity-[var(--opacity-disabled)]"
      :disabled="disabled"
      :aria-label="$t('controls.moreRecordingOptions')"
      aria-haspopup="menu"
      @click="toggleMenu"
    >
      <ChevronDown :size="15" aria-hidden="true" />
    </Button>
    <Menu ref="menu" popup unstyled :model="menuItems" :pt="menuParts" />
  </div>
</template>
