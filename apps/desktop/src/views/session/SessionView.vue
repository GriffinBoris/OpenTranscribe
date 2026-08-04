<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";

import AppTabs from "@/components/ui/AppTabs.vue";
import type { ExportFormat } from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useOpenAiStore } from "@/views/application/openAiStore";
import { useRecordingStore } from "@/views/application/recordingStore";
import SessionHeader from "@/views/session/components/SessionHeader.vue";
import SessionNotesPane from "@/views/session/components/SessionNotesPane.vue";
import SessionRenameDialog from "@/views/session/components/SessionRenameDialog.vue";
import SessionSearchDialog from "@/views/session/components/SessionSearchDialog.vue";
import SessionTrashDialog from "@/views/session/components/SessionTrashDialog.vue";
import SessionTranscriptionProgress from "@/views/session/components/SessionTranscriptionProgress.vue";
import SessionTranscriptPane from "@/views/session/components/SessionTranscriptPane.vue";
import SessionPlayback from "@/views/session/components/SessionPlayback.vue";
import { useSessionStore } from "@/views/session/sessionStore";
import { useSessionActions } from "@/views/session/useSessionActions";
import { useSessionMedia } from "@/views/session/useSessionMedia";
import { useSessionNotes } from "@/views/session/useSessionNotes";

const route = useRoute();
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
const activeTab = ref<"transcript" | "notes">("transcript");
const transcriptPaneWidth = ref<number | null>(null);
const transcriptHidden = ref(false);
const workspacePanes = ref<HTMLElement | null>(null);
const exportFormat = ref<ExportFormat>("markdown");
const exportMessage = ref<string | null>(null);
const isExporting = ref(false);
const searchOpen = ref(false);
const {
  audioSources,
  waveform,
  playback,
  playbackPosition,
  sessionError,
  isRecovering,
  loadMedia,
  revealSession,
  recoverRecording,
} = useSessionMedia(sessionId);
const {
  renameOpen,
  trashConfirmOpen,
  isTrashing,
  isMovingSession,
  isRenamingSession,
  titleDraft,
  moveToTrash,
  moveToProject,
  openRenameDialog,
  renameSession,
} = useSessionActions(sessionId, session, sessionError);

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
const { notes, notesState, initialize, finishLoading, insertTimestamp } =
  useSessionNotes(sessionId, markerPosition, isTrashing);
const recoverable = computed(
  () =>
    session.value?.recovery_state === "recoverable" &&
    recording.activeRecording?.id !== sessionId.value,
);
const isCurrentRecording = computed(
  () => recording.activeRecording?.id === sessionId.value,
);
const transcriptPanePercent = computed(() => {
  const workspace = workspacePanes.value;

  if (!workspace) {
    return 60;
  }

  const usableWidth = Math.max(1, workspace.clientWidth - 9);
  const width = transcriptPaneWidth.value ?? usableWidth * 0.6;
  return Math.round((width / usableWidth) * 100);
});

async function exportTranscript() {
  exportMessage.value = null;
  isExporting.value = true;
  const result = await sessionStore.exportSession(
    sessionId.value,
    exportFormat.value,
    session.value?.title ?? t("session.untitledRecording"),
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

function setTranscriptPaneWidth(nextWidth: number) {
  const workspace = workspacePanes.value;

  if (!workspace) {
    return;
  }

  const splitterWidth = 9;
  const minimumTranscriptWidth = 280;
  const minimumNotesWidth = 320;
  const maximumTranscriptWidth = Math.max(
    minimumTranscriptWidth,
    workspace.clientWidth - splitterWidth - minimumNotesWidth,
  );

  transcriptPaneWidth.value = Math.min(
    maximumTranscriptWidth,
    Math.max(minimumTranscriptWidth, nextWidth),
  );
}

function startPaneResize(event: PointerEvent) {
  if (window.matchMedia("(max-width: 900px)").matches) {
    return;
  }

  const splitter = event.currentTarget as HTMLElement;
  const workspace = workspacePanes.value;

  if (!workspace) {
    return;
  }

  splitter.setPointerCapture(event.pointerId);
  setTranscriptPaneWidth(
    event.clientX - workspace.getBoundingClientRect().left,
  );
}

function resizePane(event: PointerEvent) {
  const workspace = workspacePanes.value;
  const splitter = event.currentTarget as HTMLElement;

  if (!workspace || !splitter.hasPointerCapture(event.pointerId)) {
    return;
  }

  setTranscriptPaneWidth(
    event.clientX - workspace.getBoundingClientRect().left,
  );
}

function finishPaneResize(event: PointerEvent) {
  const splitter = event.currentTarget as HTMLElement;

  if (splitter.hasPointerCapture(event.pointerId)) {
    splitter.releasePointerCapture(event.pointerId);
  }
}

function adjustPaneWidth(direction: number) {
  const workspace = workspacePanes.value;

  if (!workspace) {
    return;
  }

  const currentWidth =
    transcriptPaneWidth.value ?? (workspace.clientWidth - 9) * 0.6;
  setTranscriptPaneWidth(currentWidth + direction * 24);
}

function seekPlayback(milliseconds: number) {
  activeTab.value = "transcript";
  playback.value?.seek(milliseconds / 1_000);
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

watch(
  () => application.libraryRevision,
  () => {
    if (isTrashing.value) {
      return;
    }

    void sessionStore.loadWorkspace(sessionId.value);
    void loadMedia();
  },
);

onMounted(async () => {
  window.addEventListener("keydown", handleShortcut);
  void openAi.loadCredential();
  const workspace = await sessionStore.loadWorkspace(sessionId.value);

  await loadMedia();

  if (workspace) {
    initialize(workspace.notes, workspace.notes_hash);
  } else {
    finishLoading();
  }
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleShortcut);
});
</script>

<template>
  <div
    class="session-workspace bg-surface flex h-full min-h-0 flex-col overflow-hidden"
  >
    <SessionHeader
      :session="session ?? null"
      :session-id="sessionId"
      :project-options="projectOptions"
      :recording="isCurrentRecording"
      :moving-session="isMovingSession"
      :can-transcribe="canTranscribe"
      :has-transcript="Boolean(savedTranscript)"
      :export-format="exportFormat"
      :export-options="exportOptions"
      :is-exporting="isExporting"
      :recoverable="recoverable"
      :recovering="isRecovering"
      :transcript-run="sessionStore.transcriptRuns[sessionId] ?? null"
      @recover="recoverRecording"
      @rename="openRenameDialog"
      @search="searchOpen = true"
      @trash="trashConfirmOpen = true"
      @move-to-project="moveToProject"
      @export="exportTranscript"
      @update:export-format="updateExportFormat"
    />

    <p
      v-if="exportMessage"
      class="rounded-app-lg bg-lichen-soft text-success mx-6 my-2.5 border border-[color-mix(in_srgb,var(--success)_18%,transparent)] px-3.5 py-2 text-sm"
      role="status"
    >
      {{ exportMessage }}
    </p>

    <p
      v-if="recoverable"
      class="rounded-app-lg bg-lichen-soft text-success mx-6 my-2.5 border border-[color-mix(in_srgb,var(--success)_18%,transparent)] px-3.5 py-2 text-sm"
      role="status"
    >
      {{ t("session.recovery.description") }}
    </p>

    <SessionTranscriptionProgress :session-id="sessionId" />

    <p
      v-if="sessionError"
      class="rounded-app-lg bg-accent-soft text-accent mx-6 my-2.5 border border-[color-mix(in_srgb,var(--accent)_18%,transparent)] px-3.5 py-2 text-sm"
      role="alert"
    >
      {{ sessionError }}
    </p>

    <AppTabs
      class="hidden max-[900px]:block max-[900px]:border-b max-[900px]:border-[var(--divider)] max-[900px]:px-[18px] max-[900px]:py-[7px]"
      :model-value="activeTab"
      :options="workspaceTabs"
      @update:model-value="updateActiveTab"
    />

    <div
      ref="workspacePanes"
      class="session-workspace__panes grid min-h-0 flex-1 max-[900px]:grid-cols-1"
      :class="{
        'grid-cols-1': transcriptHidden,
        'grid-cols-[minmax(280px,3fr)_9px_minmax(320px,2fr)]':
          !transcriptHidden && transcriptPaneWidth === null,
        'grid-cols-[minmax(280px,var(--transcript-pane-width))_9px_minmax(320px,1fr)]':
          !transcriptHidden && transcriptPaneWidth !== null,
      }"
      :style="
        transcriptPaneWidth === null
          ? undefined
          : { '--transcript-pane-width': `${transcriptPaneWidth}px` }
      "
    >
      <SessionTranscriptPane
        v-if="!transcriptHidden"
        :class="{ 'max-[900px]:hidden': activeTab !== 'transcript' }"
        :session-id="sessionId"
        :segments="segments"
        :speakers="speakers"
        :recording="isCurrentRecording"
        :live-transcript="recording.liveTranscript"
        :playback-ms="Math.round(playbackPosition * 1000)"
        @seek="seekPlayback"
      />

      <div
        v-if="!transcriptHidden"
        class="group relative hidden h-full cursor-col-resize touch-none outline-none select-none max-[900px]:!hidden min-[901px]:block"
        role="separator"
        :aria-label="t('session.resizeTranscript')"
        aria-orientation="vertical"
        aria-valuemin="0"
        aria-valuemax="100"
        :aria-valuenow="transcriptPanePercent"
        tabindex="0"
        @keydown.left.prevent="adjustPaneWidth(-1)"
        @keydown.right.prevent="adjustPaneWidth(1)"
        @pointerdown="startPaneResize"
        @pointermove="resizePane"
        @pointerup="finishPaneResize"
        @pointercancel="finishPaneResize"
      >
        <span
          class="group-hover:bg-lichen group-focus-visible:bg-lichen absolute top-0 bottom-0 left-1/2 w-px -translate-x-1/2 bg-[var(--divider)] transition-colors"
          aria-hidden="true"
        ></span>
      </div>

      <SessionNotesPane
        :active="activeTab === 'notes'"
        :notes="notes"
        :notes-state="notesState"
        :transcript-hidden="transcriptHidden"
        @update:notes="notes = $event"
        @add-timestamp="insertTimestamp"
        @toggle-transcript="transcriptHidden = !transcriptHidden"
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

    <SessionRenameDialog
      v-model:open="renameOpen"
      v-model:title="titleDraft"
      :saving="isRenamingSession"
      @save="renameSession"
    />

    <SessionTrashDialog
      v-model:open="trashConfirmOpen"
      :trashing="isTrashing"
      @confirm="moveToTrash"
    />
  </div>
</template>
