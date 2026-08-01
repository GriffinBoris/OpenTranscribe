<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { FileText, MessageSquareText } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppDialog from "@/components/ui/AppDialog.vue";
import AppSearchInput from "@/components/ui/AppSearchInput.vue";
import type { Speaker, TranscriptSegment } from "@/types/domain";

const props = defineProps<{
  open: boolean;
  notes: string;
  segments: TranscriptSegment[];
  speakers: Speaker[];
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
  seek: [milliseconds: number];
  "show-notes": [];
}>();

const { t } = useI18n();
const query = ref("");

const normalizedQuery = computed(() => query.value.trim().toLocaleLowerCase());
const transcriptResults = computed(() => {
  if (!normalizedQuery.value) {
    return [];
  }

  return props.segments.filter((segment) =>
    segment.text.toLocaleLowerCase().includes(normalizedQuery.value),
  );
});
const noteResults = computed(() => {
  if (!normalizedQuery.value) {
    return [];
  }

  return props.notes
    .split("\n")
    .map((line) => line.trim())
    .filter(
      (line) =>
        line.length > 0 &&
        line.toLocaleLowerCase().includes(normalizedQuery.value),
    )
    .map((line) => ({
      line,
      timestampMs: noteTimestamp(line),
    }));
});
const resultCount = computed(
  () => transcriptResults.value.length + noteResults.value.length,
);

watch(
  () => props.open,
  (open) => {
    if (open) {
      query.value = "";
    }
  },
);

function speakerName(segment: TranscriptSegment) {
  return (
    props.speakers.find((speaker) => speaker.id === segment.speaker_id)
      ?.display_name ?? t("session.speaker")
  );
}

function timestamp(milliseconds: number) {
  const seconds = Math.floor(milliseconds / 1_000);
  return `${Math.floor(seconds / 60)}:${(seconds % 60).toString().padStart(2, "0")}`;
}

function seek(milliseconds: number) {
  emit("seek", milliseconds);
  emit("update:open", false);
}

function noteTimestamp(line: string) {
  const match = line.match(/^- \[(\d+):(\d{2})\]/);

  if (!match) {
    return null;
  }

  return (Number(match[1]) * 60 + Number(match[2])) * 1_000;
}

function showNote(timestampMs: number | null) {
  if (timestampMs !== null) {
    emit("seek", timestampMs);
  }

  emit("show-notes");
  emit("update:open", false);
}
</script>

<template>
  <AppDialog
    :open="open"
    :title="t('session.search.title')"
    @update:open="emit('update:open', $event)"
  >
    <div class="grid gap-3">
      <AppSearchInput
        v-model="query"
        :placeholder="t('session.search.placeholder')"
        autofocus
      />

      <p v-if="normalizedQuery" class="text-ink-muted m-0 text-sm">
        {{ t("session.search.resultCount", { count: resultCount }) }}
      </p>

      <div
        v-if="resultCount"
        class="grid max-h-[min(480px,60vh)] gap-1 overflow-auto"
      >
        <AppButton
          v-for="segment in transcriptResults"
          :key="segment.id"
          class="w-full items-start justify-start p-2.5 text-left"
          variant="ghost"
          @click="seek(segment.start_ms)"
        >
          <FileText :size="15" />
          <span class="grid min-w-0 gap-1">
            <strong>
              {{ timestamp(segment.start_ms) }} · {{ speakerName(segment) }}
            </strong>
            <small
              class="text-ink-muted overflow-hidden font-normal text-ellipsis whitespace-nowrap"
              >{{ segment.text }}</small
            >
          </span>
        </AppButton>

        <AppButton
          v-for="(result, index) in noteResults"
          :key="`${index}-${result.line}`"
          class="w-full items-start justify-start p-2.5 text-left"
          variant="ghost"
          @click="showNote(result.timestampMs)"
        >
          <MessageSquareText :size="15" />
          <span class="grid min-w-0 gap-1">
            <strong>
              {{ t("session.notes") }}
              <template v-if="result.timestampMs !== null">
                · {{ timestamp(result.timestampMs) }}
              </template>
            </strong>
            <small
              class="text-ink-muted overflow-hidden font-normal text-ellipsis whitespace-nowrap"
              >{{ result.line }}</small
            >
          </span>
        </AppButton>
      </div>

      <div
        v-else-if="normalizedQuery"
        class="rounded-app-md border-line-strong text-ink-muted flex items-center gap-3 border border-dashed p-[18px]"
      >
        <span class="grid gap-1">
          <strong>{{ t("session.search.empty") }}</strong>
          <small>{{ t("session.search.emptyDescription") }}</small>
        </span>
      </div>
    </div>
  </AppDialog>
</template>
