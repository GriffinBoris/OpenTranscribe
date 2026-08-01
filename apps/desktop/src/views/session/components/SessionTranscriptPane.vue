<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { Pencil, Users } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppTextarea from "@/components/ui/AppTextarea.vue";
import type {
  LiveTranscriptUpdate,
  Speaker,
  TranscriptSegment,
} from "@/types/domain";
import SessionSpeakerManager from "@/views/session/components/SessionSpeakerManager.vue";
import { useSessionStore } from "@/views/session/sessionStore";

const props = defineProps<{
  sessionId: string;
  segments: TranscriptSegment[];
  speakers: Speaker[];
  recording: boolean;
  playbackMs: number;
  liveTranscript: LiveTranscriptUpdate[];
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
const transcriptScroller = ref<HTMLElement | null>(null);
const followingLiveTranscript = ref(true);
const liveTranscriptRevision = computed(() =>
  props.liveTranscript
    .map(
      (item) => `${item.source}:${item.item_id}:${item.completed}:${item.text}`,
    )
    .join("|"),
);

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

function liveSpeaker(source: LiveTranscriptUpdate["source"]) {
  return source === "system"
    ? t("session.systemAudio")
    : t("session.microphone");
}

function updateLiveFollowing() {
  const scroller = transcriptScroller.value;

  if (!scroller) {
    return;
  }

  const distanceFromBottom =
    scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight;
  followingLiveTranscript.value = distanceFromBottom <= 48;
}

function stopFollowingLiveTranscript() {
  if (props.recording) {
    followingLiveTranscript.value = false;
  }
}

watch(
  [() => props.recording, liveTranscriptRevision],
  async ([recording], [wasRecording]) => {
    if (!recording) {
      return;
    }

    if (!wasRecording) {
      followingLiveTranscript.value = true;
    }

    if (!followingLiveTranscript.value) {
      return;
    }

    await nextTick();
    const scroller = transcriptScroller.value;

    if (scroller && followingLiveTranscript.value) {
      scroller.scrollTop = scroller.scrollHeight;
    }
  },
);

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
  <section
    class="grid min-h-0 grid-rows-[var(--layout-pane-toolbar-height)_minmax(0,1fr)]"
  >
    <div
      class="session-pane-toolbar text-ink-muted flex h-[var(--layout-pane-toolbar-height)] min-h-0 items-center justify-between gap-3 border-b border-[var(--divider)] px-[18px] py-[9px] text-xs"
    >
      <div>
        <strong>{{ t("session.transcript") }}</strong>
      </div>
      <div class="flex items-center justify-end gap-2.5">
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
    <div
      ref="transcriptScroller"
      class="min-h-0 overflow-auto px-6 pt-[18px] pb-20"
      @scroll.passive="updateLiveFollowing"
      @touchstart.passive="stopFollowingLiveTranscript"
      @wheel.passive="stopFollowingLiveTranscript"
    >
      <div
        v-if="
          segments.length === 0 && liveTranscript.length === 0 && !recording
        "
        class="rounded-app-md border-line-strong text-ink-muted flex items-center gap-3 border border-dashed p-[18px]"
      >
        <span class="grid gap-1">
          <strong>{{ t("session.noTranscript") }}</strong>
          <small>{{ t("session.noTranscriptDescription") }}</small>
        </span>
      </div>
      <article
        v-for="segment in segments"
        :key="segment.id"
        class="group rounded-app-md hover:bg-canvas-subtle mt-1 grid grid-cols-[44px_1fr] gap-2.5 px-2.5 py-3"
        :class="[
          `transcript-segment--${segment.source}`,
          {
            'bg-[color-mix(in_srgb,var(--accent)_7%,transparent)] hover:bg-[color-mix(in_srgb,var(--accent)_7%,transparent)]':
              playbackMs >= segment.start_ms && playbackMs < segment.end_ms,
          },
        ]"
      >
        <AppButton
          class="text-2xs text-ink-faint block min-h-6 p-0 pt-[19px] text-left tabular-nums"
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
          <div
            class="text-lichen flex items-center gap-[var(--space-1-5)] text-xs font-bold"
          >
            <span class="size-1.5 rounded-full bg-current"></span>
            {{ speakerFor(segment)?.display_name ?? t("session.speaker") }}
            <AppButton
              v-if="!recording && editingSegmentId !== segment.id"
              class="text-ink-muted ml-auto min-h-6 px-1.5 py-0.5 font-medium opacity-[var(--opacity-muted)] group-hover:opacity-100 focus-visible:opacity-100"
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
              class="min-h-[92px] resize-y"
              :aria-label="t('session.segmentText')"
            />
            <div class="mt-2 flex justify-end gap-[var(--space-1-5)]">
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
          <p
            v-else
            class="mt-[5px] text-lg leading-[var(--line-height-reading)]"
          >
            {{ segment.text }}
          </p>
        </div>
      </article>
      <article
        v-for="item in liveTranscript"
        :key="`${item.source}:${item.item_id}`"
        class="rounded-app-md mt-1 grid grid-cols-[44px_1fr] gap-2.5 px-2.5 py-3 opacity-[0.58]"
        :class="{ 'opacity-100': item.completed }"
      >
        <span
          class="text-2xs text-ink-faint block pt-[19px] text-left tabular-nums"
          >{{ timestamp(item.started_at_ms) }}</span
        >
        <div>
          <div
            class="text-cloud flex items-center gap-[var(--space-1-5)] text-xs font-bold"
          >
            <span class="size-1.5 rounded-full bg-current"></span
            >{{ liveSpeaker(item.source) }}
          </div>
          <p class="mt-[5px] text-lg leading-[var(--line-height-reading)]">
            {{ item.text }}
          </p>
        </div>
      </article>
      <article
        v-if="recording && liveTranscript.length === 0"
        class="rounded-app-md mt-1 grid grid-cols-[44px_1fr] gap-2.5 px-2.5 py-3 opacity-[0.58]"
      >
        <span class="text-2xs text-ink-faint block pt-[19px] text-left">{{
          t("session.live")
        }}</span>
        <div>
          <div class="text-cloud text-xs font-bold">
            {{ t("session.liveTranscription") }}
          </div>
          <p class="mt-[5px] text-lg leading-[var(--line-height-reading)]">
            {{ t("session.listening") }}
          </p>
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
