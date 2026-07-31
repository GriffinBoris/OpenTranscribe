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
  <div
    class="page home-page relative h-full min-h-0 w-full overflow-auto px-[var(--layout-page-gutter)] pt-[var(--space-13)] pb-[var(--space-14)]"
  >
    <div
      v-if="isDraggingMedia"
      class="media-drop-overlay rounded-app-2xl pointer-events-none absolute inset-3 z-[var(--layer-drag-overlay)] grid place-items-center border-2 border-dashed border-[color-mix(in_srgb,var(--accent)_70%,var(--border))] bg-[color-mix(in_srgb,var(--surface)_88%,var(--accent-soft))]"
      aria-hidden="true"
    >
      <div class="text-ink grid justify-items-center gap-1.5">
        <Upload class="text-accent" :size="24" />
        <strong>{{ t("home.dropMedia") }}</strong>
        <span class="text-md text-ink-muted">{{
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
      <AppButton variant="secondary" @click="importMedia()">
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
          <div
            class="source-summary text-ink-muted flex flex-wrap gap-3.5 text-sm"
          >
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
        <section class="home-section recent-card min-w-0">
          <div
            class="section-heading mb-0 flex items-center justify-between gap-4 border-b border-[var(--divider)] pb-[11px]"
          >
            <h2 class="m-0 text-xl tracking-[-0.01em]">
              {{ t("home.recentSessions") }}
            </h2>
            <RouterLink
              to="/inbox"
              class="text-link text-lichen inline-flex items-center gap-1.5 text-sm font-bold"
            >
              {{ t("home.viewAll") }} <ArrowRight :size="14" />
            </RouterLink>
          </div>
          <div class="session-list grid">
            <p
              v-if="application.recentSessions.length === 0"
              class="quiet-state text-md text-ink-muted m-0 py-[18px]"
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

        <section
          class="home-section processing-summary min-w-0 border-l border-[var(--divider)] pl-6 max-[1100px]:border-l-0 max-[1100px]:pl-0"
        >
          <div
            class="section-heading mb-0 flex items-center justify-between gap-4 border-b border-[var(--divider)] pb-[11px]"
          >
            <div>
              <h2 class="m-0 text-xl tracking-[-0.01em]">
                {{ t("home.processing") }}
              </h2>
              <p class="text-ink-muted mt-[3px] text-sm">
                {{
                  t("home.activeJobs", {
                    count: application.runningJobs.length,
                  })
                }}
              </p>
            </div>
          </div>
          <p
            v-if="application.runningJobs.length === 0"
            class="quiet-state text-md text-ink-muted m-0 py-[18px]"
          >
            {{ t("home.nothingRunning") }}
          </p>
          <div
            v-for="job in application.runningJobs"
            :key="job.id"
            class="compact-job grid gap-2.5 pt-3 pb-[18px]"
          >
            <div>
              <strong>{{ job.progress?.message }}</strong>
              <small class="text-ink-muted block">{{ jobProvider(job) }}</small>
            </div>
            <AppProgressBar
              :value="jobProgress(job)"
              :accessible-label="
                job.progress?.message ?? t('home.processingSession')
              "
            />
          </div>
          <RouterLink
            to="/processing"
            class="text-link text-lichen inline-flex items-center gap-1.5 text-sm font-bold"
          >
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
