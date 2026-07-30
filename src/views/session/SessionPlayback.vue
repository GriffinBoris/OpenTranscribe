<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { FolderOpen, Pause, Play } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import AppSlider from "@/components/ui/AppSlider.vue";
import type { SessionAudioSource } from "@/types/domain";

const props = defineProps<{
  sources: SessionAudioSource[];
  waveform: number[];
}>();

const emit = defineEmits<{
  reveal: [];
  timeUpdate: [seconds: number];
}>();

const { t } = useI18n();
const selectedKind = ref(props.sources[0]?.kind ?? "mixed");
const source = computed(
  () =>
    props.sources.find((candidate) => candidate.kind === selectedKind.value) ??
    props.sources[0] ??
    null,
);
const sourceOptions = computed(() =>
  props.sources.map((candidate) => ({
    label: t(`playback.tracks.${candidate.kind}`),
    value: candidate.kind,
  })),
);
const audio = ref<HTMLAudioElement | null>(null);
const currentTime = ref(0);
const mediaDuration = ref((source.value?.duration_ms ?? 0) / 1_000);
const isPlaying = ref(false);
const playbackError = ref<string | null>(null);
const duration = computed(() => Math.max(mediaDuration.value, 0));
const progress = computed(() =>
  duration.value > 0 ? currentTime.value / duration.value : 0,
);

async function togglePlayback() {
  if (!audio.value) {
    return;
  }

  if (isPlaying.value) {
    audio.value.pause();
    return;
  }

  playbackError.value = null;

  try {
    await audio.value.play();
  } catch {
    playbackError.value = t("playback.unavailable");
  }
}

function seek(seconds: number) {
  if (!audio.value) {
    return;
  }

  audio.value.currentTime = seconds;
  currentTime.value = seconds;
}

function updateTime() {
  currentTime.value = audio.value?.currentTime ?? 0;
  emit("timeUpdate", currentTime.value);
}

function loadMetadata() {
  const loadedDuration = audio.value?.duration;

  if (loadedDuration && Number.isFinite(loadedDuration)) {
    mediaDuration.value = loadedDuration;
  }
}

function formatTime(seconds: number) {
  const wholeSeconds = Math.max(0, Math.floor(seconds));
  const minutes = Math.floor(wholeSeconds / 60);
  return `${minutes}:${(wholeSeconds % 60).toString().padStart(2, "0")}`;
}

watch(
  () => source.value?.url,
  () => {
    currentTime.value = 0;
    isPlaying.value = false;
    playbackError.value = null;
    mediaDuration.value = (source.value?.duration_ms ?? 0) / 1_000;
  },
);

watch(
  () => props.sources,
  (sources) => {
    if (sources.length === 0) {
      return;
    }

    if (!sources.some((candidate) => candidate.kind === selectedKind.value)) {
      selectedKind.value = sources[0].kind;
    }
  },
);

defineExpose({ seek });
</script>

<template>
  <footer v-if="source" class="playback-bar">
    <audio
      ref="audio"
      class="playback-media"
      :src="source.url"
      preload="metadata"
      @durationchange="loadMetadata"
      @ended="isPlaying = false"
      @error="playbackError = t('playback.unavailable')"
      @pause="isPlaying = false"
      @play="isPlaying = true"
      @timeupdate="updateTime"
    ></audio>
    <AppSelect
      v-if="sources.length > 1"
      v-model="selectedKind"
      class="playback-source-select"
      :options="sourceOptions"
      :accessible-label="t('playback.source')"
    />
    <AppButton
      variant="secondary"
      size="small"
      :aria-label="isPlaying ? t('playback.pause') : t('playback.play')"
      @click="togglePlayback"
    >
      <Pause v-if="isPlaying" :size="15" aria-hidden="true" />
      <Play v-else :size="15" aria-hidden="true" />
    </AppButton>
    <span class="playback-time">{{ formatTime(currentTime) }}</span>
    <div class="playback-overview">
      <div v-if="waveform.length" class="playback-waveform" aria-hidden="true">
        <span
          v-for="(peak, index) in waveform"
          :key="index"
          :class="{ active: index / waveform.length <= progress }"
          :style="{ height: `${Math.max(3, peak * 28)}px` }"
        ></span>
      </div>
      <AppSlider
        :model-value="currentTime"
        :max="duration"
        :step="0.1"
        :disabled="duration === 0"
        :accessible-label="t('playback.seek')"
        @update:model-value="seek"
      />
    </div>
    <span class="playback-time">{{ formatTime(duration) }}</span>
    <AppButton variant="ghost" size="small" @click="$emit('reveal')">
      <FolderOpen :size="15" aria-hidden="true" />
      {{ t("playback.reveal") }}
    </AppButton>
    <span v-if="playbackError" class="playback-error" role="alert">
      {{ playbackError }}
    </span>
  </footer>
</template>
