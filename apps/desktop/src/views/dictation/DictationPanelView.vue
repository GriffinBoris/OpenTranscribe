<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { Mic, Square, X } from "@lucide/vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18n } from "vue-i18n";

import AppAudioWaveform from "@/components/ui/AppAudioWaveform.vue";
import AppButton from "@/components/ui/AppButton.vue";
import { native } from "@/core/native";
import type { DictationStatus } from "@/types/domain";
import { AdaptiveAudioLevelNormalizer } from "@/views/application/audioLevels";
import { formatGlobalShortcut } from "@/views/application/globalShortcutPresets";
import { playSoundCue } from "@/views/application/soundCues";

const { t } = useI18n();
const status = ref<DictationStatus>({
  id: null,
  phase: "idle",
  provider: null,
  text: null,
  error_message: null,
  elapsed_ms: 0,
  microphone_peak: 0,
  auto_pasted: false,
  approximate_cost_usd: null,
});
const isToggling = ref(false);
const isCanceling = ref(false);
const errorMessage = ref<string | null>(null);
const waveformSamples = ref<number[]>(Array.from({ length: 34 }, () => 0));
const shortcut = ref("Alt+Space");
const microphoneMeter = new AdaptiveAudioLevelNormalizer();

const isRecording = computed(() => status.value.phase === "recording");
const isTranscribing = computed(() => status.value.phase === "transcribing");
const transcribingWaveform = Array.from({ length: 48 }, (_, index) => {
  const position = index / 47;
  return 0.2 + Math.sin(position * Math.PI) * 0.55;
});
const elapsed = computed(() => {
  const seconds = Math.floor(status.value.elapsed_ms / 1_000);
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
});
const waveform = computed(() =>
  isTranscribing.value ? transcribingWaveform : waveformSamples.value,
);
const statusMessage = computed(() => {
  if (isRecording.value) {
    return t("dictationPanel.recording");
  }

  if (isTranscribing.value) {
    return status.value.provider === "open_ai"
      ? t("dictationPanel.transcribingOpenAi")
      : t("dictationPanel.transcribingLocal");
  }

  return t("dictationPanel.ready");
});

async function refreshStatus() {
  try {
    applyStatus(await native.dictationStatus());
    errorMessage.value = null;
  } catch (reason) {
    errorMessage.value =
      reason instanceof Error ? reason.message : String(reason);
  }
}

async function toggle() {
  isToggling.value = true;

  try {
    applyStatus(await native.toggleDictation());
    errorMessage.value = null;
  } catch (reason) {
    errorMessage.value =
      reason instanceof Error ? reason.message : String(reason);
  } finally {
    isToggling.value = false;
  }
}

async function cancel() {
  isCanceling.value = true;

  try {
    applyStatus(await native.cancelDictation());
    errorMessage.value = null;
    playSoundCue("dictation-cancel");
  } catch (reason) {
    errorMessage.value =
      reason instanceof Error ? reason.message : String(reason);
  } finally {
    isCanceling.value = false;
  }
}

function applyStatus(nextStatus: DictationStatus) {
  const previousPhase = status.value.phase;
  status.value = nextStatus;

  if (previousPhase !== nextStatus.phase) {
    if (nextStatus.phase === "recording") {
      playSoundCue("dictation-start");
    }

    if (nextStatus.phase === "completed") {
      playSoundCue("dictation-complete");
    }
  }

  if (nextStatus.phase !== "recording") {
    microphoneMeter.reset();
    waveformSamples.value = Array.from({ length: 34 }, () => 0);
    return;
  }

  waveformSamples.value = [
    ...waveformSamples.value.slice(1),
    microphoneMeter.normalize(nextStatus.microphone_peak),
  ];
}

function startDragging(event: MouseEvent) {
  if (event.button !== 0) {
    return;
  }

  event.preventDefault();
  void getCurrentWindow().startDragging();
}

async function handleKeydown(event: KeyboardEvent) {
  if (
    event.key !== "Escape" ||
    event.repeat ||
    isToggling.value ||
    isCanceling.value
  ) {
    return;
  }

  event.preventDefault();

  if (isRecording.value) {
    await cancel();
    return;
  }

  await native.dismissDictation();
}

onMounted(async () => {
  window.addEventListener("keydown", handleKeydown);
  const [statusResult, shortcutResult] = await Promise.allSettled([
    refreshStatus(),
    native.dictationShortcut(),
  ]);

  if (shortcutResult.status === "fulfilled") {
    shortcut.value = shortcutResult.value;
  }

  if (statusResult.status === "rejected") {
    errorMessage.value = String(statusResult.reason);
  }

  try {
    await native.subscribeDictationStatus((nextStatus) => {
      applyStatus(nextStatus);
      errorMessage.value = null;
    });
  } catch (reason) {
    errorMessage.value =
      reason instanceof Error ? reason.message : String(reason);
  }
});

onBeforeUnmount(() => window.removeEventListener("keydown", handleKeydown));
</script>

<template>
  <main
    class="dictation-panel bg-sidebar rounded-app-xl h-screen overflow-hidden p-2"
  >
    <section
      class="rounded-app-lg border-line bg-surface-raised shadow-app grid h-full min-h-0 grid-rows-[minmax(0,1fr)_auto] gap-2 border px-5 pt-4 pb-3"
    >
      <div class="grid min-h-0 content-center gap-2">
        <div
          class="flex h-12 cursor-grab items-center justify-center gap-1.5 active:cursor-grabbing"
          aria-hidden="true"
          data-tauri-drag-region
          @mousedown.left="startDragging"
        >
          <AppAudioWaveform
            :samples="waveform"
            :active="isRecording"
            :loading="isTranscribing"
            :maximum-height="44"
            dense
          />
        </div>
        <p
          v-if="errorMessage || status.error_message"
          class="text-accent m-0 mb-1 text-center text-sm leading-5"
          role="alert"
        >
          {{ errorMessage ?? status.error_message }}
        </p>
        <p
          v-else
          class="text-ink-muted m-0 mb-1 text-center text-sm leading-5"
          aria-live="polite"
        >
          {{ statusMessage }}
        </p>
      </div>
      <div
        class="flex items-center justify-between gap-3 border-t border-[var(--divider)] pt-3"
      >
        <div class="flex items-center gap-2">
          <span class="text-ink-muted text-sm font-medium">{{ elapsed }}</span>
          <kbd class="text-ink-muted text-xs">{{
            formatGlobalShortcut(shortcut)
          }}</kbd>
        </div>
        <div class="flex items-center gap-2">
          <AppButton
            v-if="isRecording"
            size="small"
            variant="ghost"
            :disabled="isToggling"
            :loading="isCanceling"
            @click="cancel"
          >
            <X :size="15" />{{ t("dictationPanel.cancel") }}
          </AppButton>
          <AppButton
            size="small"
            :variant="isRecording ? 'danger' : 'primary'"
            :loading="isToggling || isTranscribing"
            :disabled="isTranscribing || isCanceling"
            @click="toggle"
          >
            <Square v-if="isRecording" :size="14" fill="currentColor" />
            <Mic v-else :size="15" />
            {{
              isRecording ? t("dictationPanel.stop") : t("dictationPanel.start")
            }}
          </AppButton>
        </div>
      </div>
    </section>
  </main>
</template>
