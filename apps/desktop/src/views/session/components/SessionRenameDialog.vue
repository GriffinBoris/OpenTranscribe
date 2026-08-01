<script setup lang="ts">
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppDialog from "@/components/ui/AppDialog.vue";
import AppInputText from "@/components/ui/AppInputText.vue";

defineProps<{
  open: boolean;
  title: string;
  saving: boolean;
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
  "update:title": [value: string];
  save: [];
}>();
const { t } = useI18n();
</script>

<template>
  <AppDialog
    :open="open"
    :title="t('session.rename.title')"
    @update:open="emit('update:open', $event)"
  >
    <form id="rename-session-form" @submit.prevent="emit('save')">
      <label
        class="dialog-field text-ink-muted grid gap-2 text-sm font-semibold"
      >
        <span>{{ t("session.rename.label") }}</span>
        <AppInputText
          :model-value="title"
          autocomplete="off"
          :disabled="saving"
          @update:model-value="emit('update:title', $event ?? '')"
        />
      </label>
    </form>
    <template #footer>
      <AppButton
        variant="ghost"
        :disabled="saving"
        @click="emit('update:open', false)"
      >
        {{ t("session.cancel") }}
      </AppButton>
      <AppButton
        form="rename-session-form"
        type="submit"
        variant="primary"
        :disabled="!title.trim()"
        :loading="saving"
      >
        {{ t("session.rename.save") }}
      </AppButton>
    </template>
  </AppDialog>
</template>
