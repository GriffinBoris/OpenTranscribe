<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useRouter } from "vue-router";
import { HardDrive, Mic, SlidersHorizontal, Upload } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppSplitButton from "@/components/ui/AppSplitButton.vue";
import type { RecordingMode } from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useOpenAiStore } from "@/views/application/openAiStore";
import {
  type RecordingSessionOptions,
  useRecordingStore,
} from "@/views/application/recordingStore";
import FirstRunSetup from "@/views/home/components/FirstRunSetup.vue";
import HomeProcessingSummary from "@/views/home/components/HomeProcessingSummary.vue";
import HomeRecentSessions from "@/views/home/components/HomeRecentSessions.vue";
import NewRecordingDialog from "@/views/home/components/NewRecordingDialog.vue";
import { useLocalModelsStore } from "@/views/application/localModelsStore";

const application = useApplicationStore();
const openAi = useOpenAiStore();
const recording = useRecordingStore();
const localModels = useLocalModelsStore();
const router = useRouter();
const { t } = useI18n();
const recordingDialogOpen = ref(false);
const isStartingConfiguredRecording = ref(false);
const isDraggingMedia = ref(false);
let stopListeningForMediaDrops: (() => void) | null = null;

const recordingOptions = computed(() => [
  {
    label: t("home.recordLocal"),
    value: "local_after_recording",
    disabled: localModels.installedModels.length === 0,
  },
  {
    label: t("home.recordOpenAi"),
    value: "open_ai_live",
    disabled: !openAi.credential?.configured,
  },
]);

const defaultMicrophone = computed(
  () =>
    application.audioDevices?.microphones.find((device) => device.is_default)
      ?.label ??
    application.audioDevices?.microphones[0]?.label ??
    t("home.noMicrophone"),
);
const systemAudioState = computed(() => {
  if (!application.audioDevices?.system_audio_available) {
    return t("home.unavailable");
  }

  if (
    application.settings?.capture_system_audio &&
    application.audioDevices.system_audio_permission_granted === false
  ) {
    return t("home.permissionRequired");
  }

  return application.settings?.capture_system_audio
    ? t("home.on")
    : t("home.off");
});

async function startRecording(
  mode: RecordingMode = application.settings?.recording_mode ?? "record_only",
  options?: RecordingSessionOptions,
) {
  const session = await recording.createSession(mode, options);

  if (session) {
    await router.push(`/sessions/${session.id}`);
  }
}

async function importMedia(path?: string) {
  const session = await application.importMedia(path);

  if (session) {
    await router.push(`/sessions/${session.id}`);
  }
}

async function listenForMediaDrops() {
  if (!("__TAURI_INTERNALS__" in window)) {
    return;
  }

  stopListeningForMediaDrops = await getCurrentWindow().onDragDropEvent(
    (event) => {
      if (event.payload.type === "leave") {
        isDraggingMedia.value = false;
        return;
      }

      if (event.payload.type !== "drop") {
        isDraggingMedia.value = true;
        return;
      }

      isDraggingMedia.value = false;
      const [path] = event.payload.paths;

      if (path) {
        void importMedia(path);
      }
    },
  );
}

function startRecordingOption(value: string) {
  void startRecording(value as RecordingMode);
}

async function startConfiguredRecording(
  mode: RecordingMode,
  options: RecordingSessionOptions,
) {
  isStartingConfiguredRecording.value = true;

  try {
    const session = await recording.createSession(mode, options);

    if (session) {
      recordingDialogOpen.value = false;
      await router.push(`/sessions/${session.id}`);
    }
  } finally {
    isStartingConfiguredRecording.value = false;
  }
}

onMounted(() => {
  void Promise.all([
    application.loadAudioDevices(),
    openAi.loadCredential(),
    localModels.load(),
  ]);
  void listenForMediaDrops();
});

onBeforeUnmount(() => {
  stopListeningForMediaDrops?.();
});
</script>

<template>
  <div
    class="page home-page relative h-full min-h-0 w-full overflow-auto px-[var(--layout-page-gutter)] pt-[var(--space-13)] pb-[var(--space-14)]"
  >
    <div
      v-if="isDraggingMedia || application.isImporting"
      class="media-drop-overlay rounded-app-2xl pointer-events-none absolute inset-3 z-[var(--layer-drag-overlay)] grid place-items-center border-2 border-dashed border-[color-mix(in_srgb,var(--accent)_70%,var(--border))] bg-[color-mix(in_srgb,var(--surface)_88%,var(--accent-soft))]"
      aria-hidden="true"
    >
      <div class="text-ink grid justify-items-center gap-1.5">
        <Upload class="text-accent" :size="24" />
        <strong>{{
          application.isImporting
            ? t("home.importingMedia")
            : t("home.dropMedia")
        }}</strong>
        <span v-if="!application.isImporting" class="text-md text-ink-muted">{{
          t("home.dropMediaDescription")
        }}</span>
      </div>
    </div>

    <header
      class="page-header page-header--compact mb-[var(--space-5-5)] flex items-start justify-between gap-6"
    >
      <h1
        class="text-display m-0 max-w-[730px] leading-[var(--line-height-tight)] font-bold tracking-[-0.035em]"
      >
        {{ t("home.title") }}
      </h1>
      <AppButton
        variant="secondary"
        :disabled="application.isImporting"
        :loading="application.isImporting"
        @click="importMedia()"
      >
        <Upload :size="17" />{{ t("home.importMedia") }}
      </AppButton>
    </header>

    <section
      v-if="
        application.settings?.setup_completed && !application.snapshot?.library
      "
      class="library-setup rounded-app-md bg-lichen-soft text-lichen mb-4 flex items-center gap-3 px-4 py-3.5"
    >
      <HardDrive :size="20" />
      <div class="min-w-0 flex-1">
        <strong>{{ t("home.chooseLibraryPrompt") }}</strong>
      </div>
      <AppButton variant="secondary" @click="application.chooseLibrary">
        {{ t("home.chooseLibrary") }}
      </AppButton>
    </section>

    <FirstRunSetup v-if="!application.settings?.setup_completed" />

    <template v-else>
      <section
        class="record-card rounded-app-lg bg-canvas-subtle mb-7 grid min-h-[118px] grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3.5 p-[18px] max-[1100px]:grid-cols-[auto_minmax(0,1fr)]"
      >
        <div
          class="record-card__icon rounded-app-md bg-accent-soft text-accent grid size-10 place-items-center"
        >
          <Mic :size="19" />
        </div>
        <div class="record-card__copy">
          <h2 class="mb-1 text-2xl tracking-[-0.015em]">
            {{ t("home.newRecording") }}
          </h2>
          <div class="source-summary text-ink flex flex-wrap gap-3.5 text-sm">
            <span class="flex items-center gap-1.5">
              <span
                class="source-dot source-dot--mic bg-lichen size-[7px] rounded-full"
              ></span>
              {{ defaultMicrophone }}
            </span>
            <span class="flex items-center gap-1.5">
              <span
                class="source-dot source-dot--system bg-cloud size-[7px] rounded-full"
              ></span>
              {{ t("home.systemOutput") }} ·
              {{ systemAudioState }}
            </span>
          </div>
        </div>
        <div
          class="record-card__controls flex min-w-[220px] items-center justify-end gap-2 max-[1100px]:col-span-full max-[1100px]:justify-self-end"
        >
          <AppButton
            variant="ghost"
            size="large"
            :aria-label="t('recordingOptions.menuAction')"
            @click="recordingDialogOpen = true"
          >
            <SlidersHorizontal :size="16" />
            {{ t("recordingOptions.action") }}
          </AppButton>
          <AppSplitButton
            :options="recordingOptions"
            :accessible-label="t('home.startRecording')"
            @click="startRecording()"
            @select="startRecordingOption"
          >
            <span
              class="text-accent-contrast size-[9px] rounded-full bg-current"
            ></span>
            {{ t("recording.record") }}
          </AppSplitButton>
        </div>
      </section>

      <div
        class="home-grid grid grid-cols-[minmax(0,1.7fr)_minmax(250px,0.7fr)] gap-8 max-[1100px]:grid-cols-1"
      >
        <HomeRecentSessions />
        <HomeProcessingSummary />
      </div>
    </template>

    <NewRecordingDialog
      :open="recordingDialogOpen"
      :starting="isStartingConfiguredRecording"
      @update:open="recordingDialogOpen = $event"
      @start="startConfiguredRecording"
    />
  </div>
</template>
