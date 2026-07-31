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
  root: "app-split-button__menu",
  list: "app-split-button__menu-list",
  item: "app-split-button__menu-item",
  itemContent: "app-split-button__menu-item-content",
  itemLink: "app-split-button__menu-item-link",
  itemLabel: "app-split-button__menu-item-label",
};

function toggleMenu(event: Event) {
  menu.value?.toggle(event);
}
</script>

<template>
  <div class="app-split-button">
    <Button
      unstyled
      class="app-split-button__main"
      :disabled="disabled"
      :aria-label="accessibleLabel"
      @click="$emit('click', $event)"
    >
      <slot />
    </Button>
    <Button
      unstyled
      class="app-split-button__dropdown"
      :disabled="disabled"
      :aria-label="$t('controls.moreRecordingOptions')"
      aria-haspopup="menu"
      @click="toggleMenu"
    >
      <ChevronDown :size="15" aria-hidden="true" />
    </Button>
    <Menu
      ref="menu"
      popup
      unstyled
      class="app-split-button__menu"
      :model="menuItems"
      :pt="menuParts"
    />
  </div>
</template>
