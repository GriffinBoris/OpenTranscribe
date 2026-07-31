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
  <section
    class="@container grid min-h-0 grid-rows-[var(--layout-pane-toolbar-height)_minmax(0,1fr)]"
    :class="{ 'max-[900px]:hidden': !active }"
    :aria-label="t('session.notes')"
  >
    <div
      class="notes-toolbar session-pane-toolbar text-ink-muted flex h-[var(--layout-pane-toolbar-height)] min-h-0 items-center justify-between gap-3 border-b border-[var(--divider)] px-[18px] py-[9px] text-xs"
    >
      <div class="flex items-center gap-1.5">
        <strong class="text-md text-ink">{{ t("session.notes") }}</strong>
        <span class="text-success">{{ notesState }}</span>
      </div>
      <div
        class="notes-toolbar__actions ml-auto flex shrink-0 items-center gap-2"
      >
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
          class="flex items-center gap-1"
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
      class="bg-surface font-inherit min-h-0 resize-none border-0 p-6 text-base leading-[var(--line-height-editor)] focus:!border-0 focus:!bg-[color-mix(in_srgb,var(--surface)_97%,var(--accent))] focus:![box-shadow:none]"
      :aria-label="t('session.meetingNotes')"
      @update:model-value="emit('update:notes', $event)"
    />
    <article
      v-else
      class="bg-surface [&_a]:text-lichen [&_blockquote]:border-line-strong [&_blockquote]:text-ink-muted [&_code]:rounded-app-xs [&_code]:bg-canvas-subtle [&_pre]:rounded-app-md [&_pre]:border-line [&_pre]:bg-canvas-subtle min-h-0 overflow-auto p-6 text-base leading-[var(--line-height-reading)] [&_>_:first-child]:mt-0 [&_>_:last-child]:mb-0 [&_a]:underline [&_blockquote]:mb-4 [&_blockquote]:border-l-2 [&_blockquote]:pl-3 [&_code]:px-1 [&_code]:py-px [&_code]:font-mono [&_h1]:my-5 [&_h1]:text-2xl [&_h1]:leading-[var(--line-height-heading)] [&_h2]:my-5 [&_h2]:text-xl [&_h2]:leading-[var(--line-height-heading)] [&_h3]:my-5 [&_h3]:text-lg [&_h3]:leading-[var(--line-height-heading)] [&_li+li]:mt-1 [&_ol]:mb-4 [&_ol]:pl-5 [&_p]:mb-4 [&_pre]:mb-4 [&_pre]:overflow-auto [&_pre]:p-3 [&_pre_code]:bg-transparent [&_pre_code]:p-0 [&_ul]:mb-4 [&_ul]:pl-5"
      :aria-label="t('session.notesMode.preview')"
    >
      <p v-if="!notes.trim()" class="text-ink-muted m-0">
        {{ t("session.notesMode.empty") }}
      </p>
      <!-- eslint-disable-next-line vue/no-v-html -- renderedNotes is sanitized with DOMPurify. -->
      <div v-else class="notes-preview__content" v-html="renderedNotes"></div>
    </article>
  </section>
</template>
