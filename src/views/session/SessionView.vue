<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppDialog from "@/components/ui/AppDialog.vue";
import AppInputText from "@/components/ui/AppInputText.vue";
import AppTabs from "@/components/ui/AppTabs.vue";
import { native } from "@/core/native";
import type { ExportFormat, SessionAudioSource } from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useOpenAiStore } from "@/views/application/openAiStore";
import { useRecordingStore } from "@/views/application/recordingStore";
import SessionHeader from "@/views/session/components/SessionHeader.vue";
import SessionNotesPane from "@/views/session/components/SessionNotesPane.vue";
import SessionSearchDialog from "@/views/session/components/SessionSearchDialog.vue";
import SessionTranscriptionProgress from "@/views/session/components/SessionTranscriptionProgress.vue";
import SessionTranscriptPane from "@/views/session/components/SessionTranscriptPane.vue";
import SessionPlayback from "@/views/session/SessionPlayback.vue";
import { useSessionStore } from "@/views/session/sessionStore";

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const application = useApplicationStore();
const openAi = useOpenAiStore();
const recording = useRecordingStore();
const sessionStore = useSessionStore();
const sessionId = computed(() => String(route.params.sessionId));
const session = computed(
  () =>
    application.recentSessions.find((item) => item.id === sessionId.value) ??
    recording.activeRecording,
);
const notes = ref("");
const activeTab = ref<"transcript" | "notes">("transcript");
const notesLoaded = ref(false);
const notesState = ref(t("session.saved"));
const exportFormat = ref<ExportFormat>("markdown");
const exportMessage = ref<string | null>(null);
const isExporting = ref(false);
const searchOpen = ref(false);
const renameOpen = ref(false);
const trashConfirmOpen = ref(false);
const isTrashing = ref(false);
const isRecovering = ref(false);
const isMovingSession = ref(false);
const isRenamingSession = ref(false);
const titleDraft = ref("");
const audioSources = ref<SessionAudioSource[]>([]);
const waveform = ref<number[]>([]);
const sessionError = ref<string | null>(null);
const playback = ref<{ seek: (seconds: number) => void } | null>(null);
const playbackPosition = ref(0);
let notesTimer: number | undefined;

const exportOptions = computed(() => [
  { label: t("session.exportFormats.markdown"), value: "markdown" },
  { label: t("session.exportFormats.text"), value: "text" },
  { label: t("session.exportFormats.json"), value: "json" },
  { label: t("session.exportFormats.srt"), value: "srt" },
  { label: t("session.exportFormats.vtt"), value: "vtt" },
]);
const workspaceTabs = computed(() => [
  { label: t("session.transcript"), value: "transcript" },
  { label: t("session.notes"), value: "notes" },
]);
const projectOptions = computed(() => [
  { label: t("navigation.inbox"), value: "" },
  ...application.projects.map((project) => ({
    label: project.name,
    value: project.id,
  })),
]);
const canTranscribe = computed(
  () =>
    !recording.activeRecording &&
    !savedTranscript.value &&
    Boolean(session.value?.duration_ms || session.value?.source === "import"),
);
const savedTranscript = computed(
  () => sessionStore.transcripts[sessionId.value],
);
const speakers = computed(() => savedTranscript.value?.speakers ?? []);
const segments = computed(() => savedTranscript.value?.segments ?? []);
const markerPosition = computed(
  () =>
    recording.status?.elapsed_ms ?? Math.round(playbackPosition.value * 1_000),
);
const recoverable = computed(
  () =>
    session.value?.recovery_state === "recoverable" &&
    recording.activeRecording?.id !== sessionId.value,
);

async function persistNotes() {
  notesState.value = t("session.saving");

  if (await sessionStore.saveNotes(sessionId.value, notes.value)) {
    notesState.value = t("session.saved");
  } else {
    notesState.value = t("session.unsaved");
  }
}

async function exportTranscript() {
  exportMessage.value = null;
  isExporting.value = true;
  const result = await sessionStore.exportSession(
    sessionId.value,
    exportFormat.value,
  );
  isExporting.value = false;

  if (result) {
    exportMessage.value = t("session.exportSaved", {
      filename: result.path.split(/[\\/]/).pop(),
    });
  }
}

function updateExportFormat(value: string) {
  exportFormat.value = value as ExportFormat;
}

function updateActiveTab(value: string) {
  activeTab.value = value as "transcript" | "notes";
}

function seekPlayback(milliseconds: number) {
  activeTab.value = "transcript";
  playback.value?.seek(milliseconds / 1_000);
}

function formatTimestamp(milliseconds: number) {
  const seconds = Math.floor(milliseconds / 1_000);
  return `${Math.floor(seconds / 60)}:${(seconds % 60).toString().padStart(2, "0")}`;
}

function insertTimestamp() {
  const prefix = notes.value.trimEnd();
  notes.value = `${prefix}${prefix ? "\n" : ""}- [${formatTimestamp(markerPosition.value)}] `;
}

function handleShortcut(event: KeyboardEvent) {
  if (
    (event.metaKey || event.ctrlKey) &&
    !event.shiftKey &&
    event.key.toLocaleLowerCase() === "f"
  ) {
    event.preventDefault();
    searchOpen.value = true;
    return;
  }

  if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key === "M") {
    event.preventDefault();
    insertTimestamp();
  }
}

async function revealSession() {
  sessionError.value = null;

  try {
    await native.revealSession(sessionId.value);
  } catch (reason) {
    sessionError.value =
      reason instanceof Error ? reason.message : String(reason);
  }
}

async function moveToTrash() {
  isTrashing.value = true;

  try {
    if (await sessionStore.trashSession(sessionId.value)) {
      await router.push("/inbox");
    }
  } finally {
    isTrashing.value = false;
  }
}

async function moveToProject(projectId: string) {
  if (projectId === (session.value?.project_id ?? "")) {
    return;
  }

  isMovingSession.value = true;
  sessionError.value = null;
  const moved = await sessionStore.moveSession(
    sessionId.value,
    projectId || null,
  );

  if (!moved) {
    sessionError.value =
      application.operationError ?? t("session.moveProject.failed");
  }

  isMovingSession.value = false;
}

function openRenameDialog() {
  titleDraft.value = session.value?.title ?? "";
  renameOpen.value = true;
}

async function renameSession() {
  const title = titleDraft.value.trim();

  if (!title) {
    return;
  }

  isRenamingSession.value = true;
  sessionError.value = null;
  const renamed = await sessionStore.renameSession(sessionId.value, title);

  if (!renamed) {
    sessionError.value =
      application.operationError ?? t("session.rename.failed");
  } else {
    if (recording.activeRecording?.id === renamed.id) {
      recording.activeRecording = renamed;
    }

    renameOpen.value = false;
  }

  isRenamingSession.value = false;
}

async function recoverRecording() {
  isRecovering.value = true;
  sessionError.value = null;

  try {
    const recovered = await sessionStore.recoverRecording(sessionId.value);

    if (!recovered) {
      return;
    }

    audioSources.value = await native.sessionAudioSources(sessionId.value);
    waveform.value = await native
      .sessionWaveform(sessionId.value)
      .catch(() => []);
  } catch (reason) {
    sessionError.value =
      reason instanceof Error ? reason.message : String(reason);
  } finally {
    isRecovering.value = false;
  }
}

watch(notes, () => {
  if (!notesLoaded.value) {
    return;
  }

  notesState.value = t("session.unsaved");

  if (notesTimer !== undefined) {
    window.clearTimeout(notesTimer);
  }

  notesTimer = window.setTimeout(() => {
    notesTimer = undefined;
    void persistNotes();
  }, 700);
});

watch(
  () => application.libraryRevision,
  () => {
    if (isTrashing.value) {
      return;
    }

    void sessionStore.loadWorkspace(sessionId.value);
  },
);

onMounted(async () => {
  window.addEventListener("keydown", handleShortcut);
  void openAi.loadCredential();
  const workspace = await sessionStore.loadWorkspace(sessionId.value);

  try {
    [audioSources.value, waveform.value] = await Promise.all([
      native.sessionAudioSources(sessionId.value),
      native.sessionWaveform(sessionId.value).catch(() => []),
    ]);
  } catch (reason) {
    sessionError.value =
      reason instanceof Error ? reason.message : String(reason);
  }

  if (workspace) {
    notes.value = workspace.notes;
    sessionStore.startNotesEditing(sessionId.value, workspace.notes_hash);
  }

  notesLoaded.value = true;
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleShortcut);
  if (notesTimer !== undefined) {
    window.clearTimeout(notesTimer);

    if (!isTrashing.value) {
      void persistNotes();
    }
  }
});
</script>

<template>
  <div class="session-workspace">
    <SessionHeader
      :session="session ?? null"
      :session-id="sessionId"
      :project-options="projectOptions"
      :recording="Boolean(recording.activeRecording)"
      :moving-session="isMovingSession"
      :can-transcribe="canTranscribe"
      :has-transcript="Boolean(savedTranscript)"
      :export-format="exportFormat"
      :export-options="exportOptions"
      :is-exporting="isExporting"
      :recoverable="recoverable"
      :recovering="isRecovering"
      @recover="recoverRecording"
      @rename="openRenameDialog"
      @search="searchOpen = true"
      @trash="trashConfirmOpen = true"
      @move-to-project="moveToProject"
      @export="exportTranscript"
      @update:export-format="updateExportFormat"
    />

    <p v-if="exportMessage" class="session-notice" role="status">
      {{ exportMessage }}
    </p>

    <p v-if="recoverable" class="session-notice" role="status">
      {{ t("session.recovery.description") }}
    </p>

    <SessionTranscriptionProgress :session-id="sessionId" />

    <p
      v-if="sessionError"
      class="session-notice session-notice--error"
      role="alert"
    >
      {{ sessionError }}
    </p>

    <AppTabs
      class="workspace-tabs"
      :model-value="activeTab"
      :options="workspaceTabs"
      @update:model-value="updateActiveTab"
    />

    <div class="session-split">
      <SessionTranscriptPane
        :class="{ 'mobile-hidden': activeTab !== 'transcript' }"
        :session-id="sessionId"
        :segments="segments"
        :speakers="speakers"
        :recording="Boolean(recording.activeRecording)"
        :playback-ms="Math.round(playbackPosition * 1000)"
        @seek="seekPlayback"
      />

      <SessionNotesPane
        :active="activeTab === 'notes'"
        :notes="notes"
        :notes-state="notesState"
        @update:notes="notes = $event"
        @add-timestamp="insertTimestamp"
      />
    </div>

    <SessionPlayback
      v-if="audioSources.length"
      ref="playback"
      :sources="audioSources"
      :waveform="waveform"
      @reveal="revealSession"
      @time-update="playbackPosition = $event"
    />

    <SessionSearchDialog
      :open="searchOpen"
      :notes="notes"
      :segments="segments"
      :speakers="speakers"
      @update:open="searchOpen = $event"
      @seek="seekPlayback"
      @show-notes="activeTab = 'notes'"
    />

    <AppDialog
      :open="renameOpen"
      :title="t('session.rename.title')"
      @update:open="renameOpen = $event"
    >
      <form id="rename-session-form" @submit.prevent="renameSession">
        <label class="dialog-field">
          <span>{{ t("session.rename.label") }}</span>
          <AppInputText
            v-model="titleDraft"
            autocomplete="off"
            :disabled="isRenamingSession"
          />
        </label>
      </form>
      <template #footer>
        <AppButton
          variant="ghost"
          :disabled="isRenamingSession"
          @click="renameOpen = false"
        >
          {{ t("session.cancel") }}
        </AppButton>
        <AppButton
          form="rename-session-form"
          type="submit"
          variant="primary"
          :disabled="!titleDraft.trim()"
          :loading="isRenamingSession"
        >
          {{ t("session.rename.save") }}
        </AppButton>
      </template>
    </AppDialog>

    <AppDialog
      :open="trashConfirmOpen"
      :title="t('session.trash.title')"
      @update:open="trashConfirmOpen = $event"
    >
      <p>{{ t("session.trash.description") }}</p>
      <template #footer>
        <AppButton
          variant="ghost"
          :disabled="isTrashing"
          @click="trashConfirmOpen = false"
        >
          {{ t("session.cancel") }}
        </AppButton>
        <AppButton variant="danger" :loading="isTrashing" @click="moveToTrash">
          {{ t("session.trash.confirm") }}
        </AppButton>
      </template>
    </AppDialog>
  </div>
</template>
