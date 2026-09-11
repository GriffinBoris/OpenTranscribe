<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { Mic, Square, X, RotateCcw } from "@lucide/vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18n } from "vue-i18n";
import AppAudioWaveform from "@/components/ui/AppAudioWaveform.vue";
import AppButton from "@/components/ui/AppButton.vue";
import AppCopyButton from "@/components/ui/AppCopyButton.vue";
import { native } from "@/core/native";
import {
  emptyDictationStatus,
  isDictationActive,
} from "@/core/dictationStatus";
import type { DictationStatus } from "@/types/domain";
import { AdaptiveAudioLevelNormalizer } from "@/views/application/audioLevels";
import { applyAppearance } from "@/views/application/appearance";
import { formatGlobalShortcut } from "@/views/application/globalShortcutPresets";
import { playSoundCue } from "@/views/application/soundCues";

const { t } = useI18n();
const status = ref(emptyDictationStatus());
const isActing = ref(false);
const errorMessage = ref<string | null>(null);
const waveformSamples = ref<number[]>(Array.from({ length: 34 }, () => 0));
const shortcut = ref<string | null>(null);
const microphoneMeter = new AdaptiveAudioLevelNormalizer();
const isRecording = computed(() => status.value.phase === "recording");
const isProcessing = computed(() =>
  ["transcribing", "cleaning"].includes(status.value.phase),
);
const processingSamples = Array.from(
  { length: 34 },
  (_, index) => 0.2 + Math.sin((index / 33) * Math.PI) * 0.5,
);
const isActive = computed(() => isDictationActive(status.value.phase));
const resultText = computed(() => (!isActive.value ? status.value.text : null));
const elapsed = computed(() => {
  const seconds = Math.floor(status.value.elapsed_ms / 1_000);
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
});
const statusMessage = computed(() => {
  if (isRecording.value) return t("dictationPanel.recording");
  if (status.value.phase === "cleaning") return t("dictationPanel.cleaning");
  if (isProcessing.value) {
    const message = t(
      status.value.provider === "open_ai"
        ? "dictationPanel.transcribingOpenAi"
        : "dictationPanel.transcribingLocal",
    );
    return status.value.progress_percent === null
      ? message
      : `${message} ${status.value.progress_percent}%`;
  }
  if (status.value.phase === "completed") {
    if (!status.value.text) return t("dictationPanel.noSpeech");
    return t(
      status.value.auto_pasted
        ? "dictationPanel.pasted"
        : "dictationPanel.copied",
    );
  }
  return t("dictationPanel.ready");
});

async function refreshPreferences() {
  try {
    const settings = await native.dictationSettings();
    shortcut.value = settings.dictation_shortcut_enabled
      ? settings.dictation_shortcut
      : null;
    applyAppearance(settings);
  } catch (reason) {
    errorMessage.value =
      reason instanceof Error ? reason.message : String(reason);
  }
}

function applyStatus(next: DictationStatus) {
  const changed = status.value.phase !== next.phase;
  if (status.value.id !== next.id || changed) errorMessage.value = null;
  status.value = next;
  if (changed) {
    void refreshPreferences();
    if (next.phase === "recording") playSoundCue("dictation-start");
    if (next.phase === "completed") playSoundCue("dictation-complete");
  }
  if (next.phase !== "recording") {
    microphoneMeter.reset();
    waveformSamples.value = Array.from({ length: 34 }, () => 0);
    return;
  }
  waveformSamples.value = [
    ...waveformSamples.value.slice(1),
    microphoneMeter.normalize(next.microphone_peak),
  ];
}

async function perform(action: "toggle" | "cancel" | "retry" | "dismiss") {
  if (isActing.value) return;
  isActing.value = true;
  errorMessage.value = null;
  try {
    if (action === "dismiss") {
      await native.dismissDictation();
      applyStatus(emptyDictationStatus());
    } else if (action === "retry") applyStatus(await native.retryDictation());
    else if (action === "cancel") {
      applyStatus(await native.cancelDictation());
      playSoundCue("dictation-cancel");
    } else applyStatus(await native.toggleDictation());
  } catch (reason) {
    errorMessage.value =
      reason instanceof Error ? reason.message : String(reason);
  } finally {
    isActing.value = false;
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Escape" && !event.repeat) {
    event.preventDefault();
    void perform(isActive.value ? "cancel" : "dismiss");
  }
}

function startDragging(event: MouseEvent) {
  if (event.button === 0 && "__TAURI_INTERNALS__" in window) {
    event.preventDefault();
    void getCurrentWindow().startDragging();
  }
}

onMounted(async () => {
  window.addEventListener("keydown", handleKeydown);
  window.addEventListener("focus", refreshPreferences);
  try {
    await native.subscribeDictationStatus(applyStatus);
    applyStatus(await native.dictationStatus());
    await refreshPreferences();
  } catch (reason) {
    errorMessage.value =
      reason instanceof Error ? reason.message : String(reason);
  }
});
onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleKeydown);
  window.removeEventListener("focus", refreshPreferences);
});
</script>

<template>
  <main
    class="dictation-panel bg-sidebar rounded-app-xl h-screen overflow-hidden p-2"
  >
    <section
      class="rounded-app-lg border-line bg-surface-raised shadow-app flex h-full min-h-0 flex-col gap-2 border px-5 pt-3 pb-3"
    >
      <div
        class="flex min-h-0 flex-1 flex-col justify-center gap-2 overflow-auto"
      >
        <div
          v-if="!resultText && !status.error_message && !errorMessage"
          class="flex h-12 shrink-0 cursor-grab items-center justify-center active:cursor-grabbing"
          aria-hidden="true"
          data-tauri-drag-region
          @mousedown.left="startDragging"
        >
          <AppAudioWaveform
            :samples="isProcessing ? processingSamples : waveformSamples"
            :active="isRecording"
            :loading="isProcessing"
            :maximum-height="44"
            dense
          />
        </div>
        <p
          v-if="errorMessage || status.error_message"
          class="text-accent m-0 text-center text-sm leading-5"
          role="alert"
        >
          {{ errorMessage ?? status.error_message }}
        </p>
        <p v-else class="text-ink m-0 text-center text-sm" aria-live="polite">
          {{ statusMessage }}
        </p>
        <p
          v-if="status.can_retry"
          class="text-ink-muted m-0 text-center text-xs"
        >
          {{ t("dictationPanel.retained") }}
        </p>
        <p
          v-if="resultText"
          class="text-ink m-0 max-h-16 overflow-auto text-sm whitespace-pre-wrap"
        >
          {{ resultText }}
        </p>
      </div>
      <div
        class="flex shrink-0 items-center justify-between gap-3 border-t border-[var(--divider)] pt-3"
      >
        <div class="text-ink-muted flex items-center gap-2 text-xs">
          <span :title="t('dictationPanel.duration')">{{ elapsed }}</span>
          <kbd v-if="shortcut">{{ formatGlobalShortcut(shortcut) }}</kbd>
        </div>
        <div class="flex items-center gap-2">
          <AppCopyButton
            v-if="resultText"
            :text="resultText"
            :label="t('dictationHistory.copy')"
            :copied-label="t('common.copied')"
          />
          <AppButton
            size="small"
            variant="ghost"
            :disabled="isActing"
            @click="perform(isActive ? 'cancel' : 'dismiss')"
          >
            <X :size="15" />{{
              t(isActive ? "dictationPanel.cancel" : "dictationPanel.close")
            }}
          </AppButton>
          <AppButton
            v-if="status.can_retry"
            size="small"
            :loading="isActing"
            @click="perform('retry')"
            ><RotateCcw :size="15" />{{ t("dictationPanel.retry") }}</AppButton
          >
          <AppButton
            v-else-if="!isProcessing"
            size="small"
            :variant="isRecording ? 'danger' : 'primary'"
            :loading="isActing"
            @click="perform('toggle')"
          >
            <Square v-if="isRecording" :size="14" fill="currentColor" /><Mic
              v-else
              :size="15"
            />
            {{
              t(isRecording ? "dictationPanel.stop" : "dictationPanel.start")
            }}
          </AppButton>
        </div>
      </div>
    </section>
  </main>
</template>
