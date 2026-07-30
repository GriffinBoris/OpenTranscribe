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
  mask: "app-dialog__mask",
  root: "app-dialog",
  header: "app-dialog__header",
  title: "app-dialog__title",
  headerActions: "app-dialog__header-actions",
  pcCloseButton: {
    root: { class: "app-dialog__close" },
    icon: { class: "app-dialog__close-icon" },
  },
  content: "app-dialog__content",
  footer: "app-dialog__footer",
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
