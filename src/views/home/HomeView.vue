<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useRouter } from "vue-router";
import {
  ArrowRight,
  HardDrive,
  Mic,
  SlidersHorizontal,
  Upload,
} from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppProgressBar from "@/components/ui/AppProgressBar.vue";
import AppSplitButton from "@/components/ui/AppSplitButton.vue";
import type { Job, RecordingMode } from "@/types/domain";
import MovableSessionRow from "@/views/application/components/MovableSessionRow.vue";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useOpenAiStore } from "@/views/application/openAiStore";
import {
  type RecordingSessionOptions,
  useRecordingStore,
} from "@/views/application/recordingStore";
import FirstRunSetup from "@/views/home/components/FirstRunSetup.vue";
import NewRecordingDialog from "@/views/home/components/NewRecordingDialog.vue";
import { useLocalModelsStore } from "@/views/models/localModelsStore";

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
    value: "local_live",
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
  mode: RecordingMode = "record_only",
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

function jobProgress(job: Job) {
  if (!job.progress?.total_units) {
    return undefined;
  }

  return (job.progress.completed_units / job.progress.total_units) * 100;
}

function jobProvider(job: Job) {
  return job.kind.includes("open_ai")
    ? t("processing.openAi")
    : t("home.localProvider");
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
  <div class="page home-page">
    <div v-if="isDraggingMedia" class="media-drop-overlay" aria-hidden="true">
      <div>
        <Upload :size="24" />
        <strong>{{ t("home.dropMedia") }}</strong>
        <span>{{ t("home.dropMediaDescription") }}</span>
      </div>
    </div>

    <header class="page-header page-header--compact">
      <h1>{{ t("home.title") }}</h1>
      <AppButton variant="secondary" @click="importMedia()">
        <Upload :size="17" />{{ t("home.importMedia") }}
      </AppButton>
    </header>

    <section
      v-if="
        application.settings?.setup_completed && !application.snapshot?.library
      "
      class="library-setup"
    >
      <HardDrive :size="20" />
      <div>
        <strong>{{ t("home.chooseLibraryPrompt") }}</strong>
      </div>
      <AppButton variant="secondary" @click="application.chooseLibrary">
        {{ t("home.chooseLibrary") }}
      </AppButton>
    </section>

    <FirstRunSetup v-if="!application.settings?.setup_completed" />

    <template v-else>
      <section class="record-card">
        <div class="record-card__icon"><Mic :size="19" /></div>
        <div class="record-card__copy">
          <h2>{{ t("home.newRecording") }}</h2>
          <div class="source-summary">
            <span>
              <span class="source-dot source-dot--mic"></span>
              {{ defaultMicrophone }}
            </span>
            <span>
              <span class="source-dot source-dot--system"></span>
              {{ t("home.systemOutput") }} ·
              {{ systemAudioState }}
            </span>
          </div>
        </div>
        <div class="record-card__controls">
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
            <span class="button-record-dot"></span>
            {{ t("recording.record") }}
          </AppSplitButton>
        </div>
      </section>

      <div class="home-grid">
        <section class="home-section recent-card">
          <div class="section-heading">
            <h2>{{ t("home.recentSessions") }}</h2>
            <RouterLink to="/inbox" class="text-link">
              {{ t("home.viewAll") }} <ArrowRight :size="14" />
            </RouterLink>
          </div>
          <div class="session-list">
            <p
              v-if="application.recentSessions.length === 0"
              class="quiet-state"
            >
              {{ t("home.noSessions") }}
            </p>
            <MovableSessionRow
              v-for="session in application.recentSessions.slice(0, 4)"
              :key="session.id"
              :session="session"
            />
          </div>
        </section>

        <section class="home-section processing-summary">
          <div class="section-heading">
            <div>
              <h2>{{ t("home.processing") }}</h2>
              <p>
                {{
                  t("home.activeJobs", {
                    count: application.runningJobs.length,
                  })
                }}
              </p>
            </div>
          </div>
          <p v-if="application.runningJobs.length === 0" class="quiet-state">
            {{ t("home.nothingRunning") }}
          </p>
          <div
            v-for="job in application.runningJobs"
            :key="job.id"
            class="compact-job"
          >
            <div>
              <strong>{{ job.progress?.message }}</strong>
              <small>{{ jobProvider(job) }}</small>
            </div>
            <AppProgressBar
              :value="jobProgress(job)"
              :accessible-label="
                job.progress?.message ?? t('home.processingSession')
              "
            />
          </div>
          <RouterLink to="/processing" class="text-link">
            {{ t("home.openProcessing") }} <ArrowRight :size="14" />
          </RouterLink>
        </section>
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
