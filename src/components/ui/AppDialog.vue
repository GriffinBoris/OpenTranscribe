<script setup lang="ts">
import Dialog from "primevue/dialog";

withDefaults(
  defineProps<{
    open: boolean;
    title: string;
  }>(),
  {},
);

defineEmits<{
  "update:open": [value: boolean];
}>();

const dialogParts = {
  mask: "app-dialog__mask fixed inset-0 z-[var(--layer-dialog)] flex animate-[dialog-mask-enter_var(--duration-slow)_var(--easing-standard)] items-center justify-center bg-[var(--dialog-mask)] p-6 backdrop-blur-sm",
  root: "app-dialog w-full max-w-[420px] overflow-hidden rounded-app-lg border border-line bg-surface-raised shadow-app-dialog",
  header:
    "app-dialog__header flex items-center justify-between gap-4 px-5 pt-[var(--space-4-5)] pb-2.5",
  title: "app-dialog__title text-2xl font-bold tracking-[-0.015em]",
  headerActions: "app-dialog__header-actions flex",
  pcCloseButton: {
    root: {
      class:
        "app-dialog__close grid size-[30px] place-items-center rounded-app-xs border-0 bg-transparent p-0 text-ink-muted hover:bg-canvas-subtle hover:text-ink",
    },
    icon: { class: "app-dialog__close-icon size-[13px]" },
  },
  content: "app-dialog__content px-5 pt-2.5 pb-5",
  footer:
    "app-dialog__footer flex justify-end gap-2 border-t border-[var(--divider)] bg-canvas-subtle px-5 py-3",
};
</script>

<template>
  <Dialog
    unstyled
    modal
    :visible="open"
    :header="title"
    :draggable="false"
    :dismissable-mask="true"
    :pt="dialogParts"
    @update:visible="$emit('update:open', $event)"
  >
    <slot />
    <template v-if="$slots.footer" #footer>
      <slot name="footer" />
    </template>
  </Dialog>
</template>
