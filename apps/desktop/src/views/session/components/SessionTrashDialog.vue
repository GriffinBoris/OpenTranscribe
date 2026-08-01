<script setup lang="ts">
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppDialog from "@/components/ui/AppDialog.vue";

defineProps<{
  open: boolean;
  trashing: boolean;
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
  confirm: [];
}>();
const { t } = useI18n();
</script>

<template>
  <AppDialog
    :open="open"
    :title="t('session.trash.title')"
    @update:open="emit('update:open', $event)"
  >
    <p>{{ t("session.trash.description") }}</p>
    <template #footer>
      <AppButton
        variant="ghost"
        :disabled="trashing"
        @click="emit('update:open', false)"
      >
        {{ t("session.cancel") }}
      </AppButton>
      <AppButton variant="danger" :loading="trashing" @click="emit('confirm')">
        {{ t("session.trash.confirm") }}
      </AppButton>
    </template>
  </AppDialog>
</template>
