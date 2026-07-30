<script setup lang="ts">
import { computed } from "vue";
import { Pause, Play, Square } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import { useRecordingStore } from "@/views/application/recordingStore";

withDefaults(
  defineProps<{
    stopping?: boolean;
  }>(),
  {
    stopping: false,
  },
);

defineEmits<{
  stop: [];
}>();

const recording = useRecordingStore();
const { t } = useI18n();

const elapsed = computed(() => {
  const elapsedSeconds = Math.floor((recording.status?.elapsed_ms ?? 0) / 1000);
  const hours = Math.floor(elapsedSeconds / 3600);
  const minutes = Math.floor((elapsedSeconds % 3600) / 60);
  const seconds = elapsedSeconds % 60;
  return [hours, minutes, seconds]
    .map((part) => part.toString().padStart(2, "0"))
    .join(":");
});

const microphoneLevel = computed(
  () => `${Math.min((recording.status?.microphone_peak ?? 0) * 100, 100)}%`,
);
const recordingOutcome = computed(() => {
  if (recording.activeMode === "local_live") {
    return t("recordingDock.localAfterStop");
  }

  if (recording.activeMode === "open_ai_live") {
    return t("recordingDock.openAiAfterStop");
  }

  return t("recordingDock.saveToDisk");
});
</script>

<template>
  <div class="recording-dock" role="status" aria-live="polite">
    <div class="recording-dock__status">
      <span class="recording-dot"></span>
      <strong>{{
        recording.status?.is_paused
          ? t("recordingDock.paused")
          : t("recordingDock.recording")
      }}</strong>
      <span class="recording-dock__timer">{{ elapsed }}</span>
      <StatusPill tone="local">{{ recordingOutcome }}</StatusPill>
    </div>

    <div
      class="recording-dock__levels"
      :aria-label="t('recordingDock.inputLevels')"
    >
      <span class="level-meter"
        ><span :style="{ width: microphoneLevel }"></span
      ></span>
      <span v-if="recording.status?.captures_system_audio" class="level-meter">
        <span
          :style="{
            width: `${Math.min((recording.status?.system_peak ?? 0) * 100, 100)}%`,
          }"
        ></span>
      </span>
      <small v-if="recording.status?.dropped_packets">
        {{
          t("recordingDock.droppedPackets", recording.status.dropped_packets)
        }}
      </small>
    </div>

    <div class="recording-dock__actions">
      <AppButton size="small" variant="ghost" @click="recording.togglePaused">
        <Play v-if="recording.status?.is_paused" :size="16" />
        <Pause v-else :size="16" />
        {{
          recording.status?.is_paused
            ? t("recordingDock.resume")
            : t("recordingDock.pause")
        }}
      </AppButton>
      <AppButton
        size="small"
        variant="danger"
        :disabled="stopping"
        :loading="stopping"
        @click="$emit('stop')"
      >
        <Square :size="14" fill="currentColor" />{{ t("recordingDock.stop") }}
      </AppButton>
    </div>
  </div>
</template>
