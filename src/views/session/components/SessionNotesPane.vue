<script setup lang="ts">
import { Clock3, Eye, Pencil } from "@lucide/vue";
import DOMPurify from "dompurify";
import { marked } from "marked";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppTextarea from "@/components/ui/AppTextarea.vue";

const props = defineProps<{
  active: boolean;
  notes: string;
  notesState: string;
}>();

const emit = defineEmits<{
  "update:notes": [value: string];
  "add-timestamp": [];
}>();

const { t } = useI18n();
const notesMode = ref<"edit" | "preview">("edit");
const renderedNotes = computed(() =>
  DOMPurify.sanitize(marked.parse(props.notes, { async: false }), {
    USE_PROFILES: { html: true },
  }),
);
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
          :aria-label="t('session.addTimestamp')"
          :title="t('session.addTimestampShortcut')"
          @click="emit('add-timestamp')"
        >
          <Clock3 :size="14" aria-hidden="true" />
          <span class="notes-toolbar__label">
            {{ t("session.addTimestamp") }}
          </span>
        </AppButton>
        <div
          class="notes-mode-switch"
          role="group"
          :aria-label="t('session.notesMode.label')"
        >
          <AppButton
            size="small"
            :aria-label="t('session.notesMode.edit')"
            :title="t('session.notesMode.edit')"
            :variant="notesMode === 'edit' ? 'secondary' : 'ghost'"
            @click="notesMode = 'edit'"
          >
            <Pencil :size="14" aria-hidden="true" />
            <span class="notes-toolbar__label">
              {{ t("session.notesMode.edit") }}
            </span>
          </AppButton>
          <AppButton
            size="small"
            :aria-label="t('session.notesMode.preview')"
            :title="t('session.notesMode.preview')"
            :variant="notesMode === 'preview' ? 'secondary' : 'ghost'"
            @click="notesMode = 'preview'"
          >
            <Eye :size="14" aria-hidden="true" />
            <span class="notes-toolbar__label">
              {{ t("session.notesMode.preview") }}
            </span>
          </AppButton>
        </div>
      </div>
    </div>
    <AppTextarea
      v-if="notesMode === 'edit'"
      :model-value="notes"
      class="notes-editor"
      :aria-label="t('session.meetingNotes')"
      @update:model-value="emit('update:notes', $event)"
    />
    <article
      v-else
      class="notes-preview"
      :aria-label="t('session.notesMode.preview')"
    >
      <p v-if="!notes.trim()" class="notes-preview__empty">
        {{ t("session.notesMode.empty") }}
      </p>
      <!-- eslint-disable-next-line vue/no-v-html -- renderedNotes is sanitized with DOMPurify. -->
      <div v-else class="notes-preview__content" v-html="renderedNotes"></div>
    </article>
  </section>
</template>
