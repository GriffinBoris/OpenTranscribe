<script setup lang="ts">
import { ref } from "vue";
import { Pencil, Users } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppTextarea from "@/components/ui/AppTextarea.vue";
import type { Speaker, TranscriptSegment } from "@/types/domain";
import SessionSpeakerManager from "@/views/session/components/SessionSpeakerManager.vue";
import { useSessionStore } from "@/views/session/sessionStore";

const props = defineProps<{
  sessionId: string;
  segments: TranscriptSegment[];
  speakers: Speaker[];
  recording: boolean;
  playbackMs: number;
}>();

const emit = defineEmits<{
  seek: [milliseconds: number];
}>();

const { t } = useI18n();
const sessionStore = useSessionStore();
const editingSegmentId = ref<string | null>(null);
const draft = ref("");
const isSaving = ref(false);
const speakerManagerOpen = ref(false);

function speakerFor(segment: TranscriptSegment) {
  return (
    props.speakers.find((speaker) => speaker.id === segment.speaker_id) ??
    props.speakers[0]
  );
}

function timestamp(milliseconds: number) {
  const seconds = Math.floor(milliseconds / 1000);
  return `${Math.floor(seconds / 60)}:${(seconds % 60).toString().padStart(2, "0")}`;
}

function edit(segment: TranscriptSegment) {
  editingSegmentId.value = segment.id;
  draft.value = segment.text;
}

function cancel() {
  editingSegmentId.value = null;
  draft.value = "";
}

async function save(segment: TranscriptSegment) {
  isSaving.value = true;

  try {
    const saved = await sessionStore.updateTranscriptSegment(
      props.sessionId,
      segment.id,
      draft.value,
    );
    if (saved) {
      cancel();
    }
  } finally {
    isSaving.value = false;
  }
}
</script>

<template>
  <section class="transcript-pane">
    <div class="pane-toolbar">
      <div>
        <strong>{{ t("session.transcript") }}</strong>
      </div>
      <div class="pane-toolbar__actions">
        <span>{{ t("session.segmentCount", { count: segments.length }) }}</span>
        <AppButton
          v-if="!recording && speakers.length"
          size="small"
          variant="ghost"
          @click="speakerManagerOpen = true"
        >
          <Users :size="14" />
          {{ t("session.speakers.manage") }}
        </AppButton>
      </div>
    </div>
    <div class="transcript-feed">
      <div v-if="segments.length === 0" class="empty-setting">
        <span>
          <strong>{{ t("session.noTranscript") }}</strong>
          <small>{{ t("session.noTranscriptDescription") }}</small>
        </span>
      </div>
      <article
        v-for="segment in segments"
        :key="segment.id"
        class="transcript-segment"
        :class="[
          `transcript-segment--${segment.source}`,
          {
            'transcript-segment--active':
              playbackMs >= segment.start_ms && playbackMs < segment.end_ms,
          },
        ]"
      >
        <AppButton
          class="segment-time"
          size="small"
          variant="ghost"
          :aria-label="
            t('session.seekTo', { time: timestamp(segment.start_ms) })
          "
          @click="emit('seek', segment.start_ms)"
        >
          {{ timestamp(segment.start_ms) }}
        </AppButton>
        <div>
          <div class="segment-speaker">
            <span class="speaker-dot"></span>
            {{ speakerFor(segment)?.display_name ?? t("session.speaker") }}
            <AppButton
              v-if="!recording && editingSegmentId !== segment.id"
              class="segment-edit"
              size="small"
              variant="ghost"
              @click="edit(segment)"
            >
              <Pencil :size="13" />
              {{ t("session.edit") }}
            </AppButton>
          </div>
          <template v-if="editingSegmentId === segment.id">
            <AppTextarea
              v-model="draft"
              class="segment-editor"
              :aria-label="t('session.segmentText')"
            />
            <div class="segment-editor__actions">
              <AppButton
                size="small"
                variant="ghost"
                :disabled="isSaving"
                @click="cancel"
              >
                {{ t("session.cancel") }}
              </AppButton>
              <AppButton
                size="small"
                variant="primary"
                :disabled="!draft.trim()"
                :loading="isSaving"
                @click="save(segment)"
              >
                {{ t("session.save") }}
              </AppButton>
            </div>
          </template>
          <p v-else>{{ segment.text }}</p>
        </div>
      </article>
      <article
        v-if="recording"
        class="transcript-segment transcript-provisional"
      >
        <span class="segment-time">{{ t("session.live") }}</span>
        <div>
          <div class="segment-speaker">
            <span class="speaker-dot"></span>{{ t("session.system") }}
          </div>
          <p>{{ t("session.listening") }}</p>
        </div>
      </article>
    </div>
    <SessionSpeakerManager
      :open="speakerManagerOpen"
      :session-id="sessionId"
      :speakers="speakers"
      @update:open="speakerManagerOpen = $event"
    />
  </section>
</template>
