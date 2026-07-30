<script setup lang="ts">
import { Clock3 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppTextarea from "@/components/ui/AppTextarea.vue";

defineProps<{
  active: boolean;
  notes: string;
  notesState: string;
}>();

const emit = defineEmits<{
  "update:notes": [value: string];
  "add-timestamp": [];
}>();

const { t } = useI18n();
</script>

<template>
  <section class="notes-pane" :class="{ 'mobile-hidden': !active }">
    <div class="pane-toolbar notes-toolbar">
      <div>
        <strong>{{ t("session.notes") }}</strong>
        <span class="saved-state">{{ notesState }}</span>
      </div>
      <div class="notes-toolbar__actions">
        <AppButton
          size="small"
          variant="ghost"
          :title="t('session.addTimestampShortcut')"
          @click="emit('add-timestamp')"
        >
          <Clock3 :size="14" aria-hidden="true" />
          {{ t("session.addTimestamp") }}
        </AppButton>
        <span class="notes-format">{{
          t("session.exportFormats.markdown")
        }}</span>
      </div>
    </div>
    <AppTextarea
      :model-value="notes"
      class="notes-editor"
      :aria-label="t('session.meetingNotes')"
      @update:model-value="emit('update:notes', $event)"
    />
  </section>
</template>
