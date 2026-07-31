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

const microphoneLevel = computed(() => ({
  "--audio-level": String(Math.min(recording.status?.microphone_peak ?? 0, 1)),
}));
const systemLevel = computed(() => ({
  "--audio-level": String(Math.min(recording.status?.system_peak ?? 0, 1)),
}));
const recordingOutcome = computed(() => {
  if (recording.activeMode === "local_after_recording") {
    return t("recordingDock.localAfterStop");
  }

  if (recording.activeMode === "open_ai_live") {
    return t("recordingDock.openAiLive");
  }

  return t("recordingDock.saveToDisk");
});
</script>

<template>
  <div
    class="rounded-app-xl shadow-app-dock fixed right-[26px] bottom-[22px] left-[270px] z-[var(--layer-dock)] grid grid-cols-[auto_minmax(160px,1fr)_auto] items-center gap-5 border border-[color-mix(in_srgb,var(--accent)_35%,var(--border))] bg-[color-mix(in_srgb,var(--surface-raised)_94%,transparent)] py-2.5 pr-3 pl-4 backdrop-blur-[16px] max-[900px]:left-[100px]"
    role="status"
    aria-live="polite"
  >
    <div class="flex items-center gap-2.5">
      <span
        class="bg-accent size-[9px] animate-[pulse_1.5s_ease-in-out_infinite] rounded-full shadow-[0_0_0_4px_var(--accent-soft)]"
      ></span>
      <strong>{{
        recording.status?.is_paused
          ? t("recordingDock.paused")
          : t("recordingDock.recording")
      }}</strong>
      <span class="text-ink-muted tabular-nums">{{ elapsed }}</span>
      <StatusPill tone="local">{{ recordingOutcome }}</StatusPill>
    </div>

    <div class="grid gap-1" :aria-label="t('recordingDock.inputLevels')">
      <span class="bg-canvas-subtle block h-[3px] overflow-hidden rounded-full"
        ><span
          class="bg-lichen block size-full origin-left [transform:scaleX(var(--audio-level,0))] rounded-[inherit] transition-transform duration-[var(--duration-meter)] ease-[var(--easing-linear)] will-change-transform"
          :style="microphoneLevel"
        ></span
      ></span>
      <span
        v-if="recording.status?.captures_system_audio"
        class="bg-canvas-subtle block h-[3px] overflow-hidden rounded-full"
      >
        <span
          class="bg-lichen block size-full origin-left [transform:scaleX(var(--audio-level,0))] rounded-[inherit] transition-transform duration-[var(--duration-meter)] ease-[var(--easing-linear)] will-change-transform"
          :style="systemLevel"
        ></span>
      </span>
      <small v-if="recording.status?.dropped_packets">
        {{
          t("recordingDock.droppedPackets", recording.status.dropped_packets)
        }}
      </small>
    </div>

    <div class="flex items-center gap-2.5">
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
