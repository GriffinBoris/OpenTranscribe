import { computed, ref } from "vue";
import { defineStore } from "pinia";

import { native } from "@/core/native";
import { i18n } from "@/i18n";
import type {
  AppSnapshot,
  AudioDevices,
  RecordingProjectSelection,
  Session,
} from "@/types/domain";
import { useRecordingStore } from "@/views/application/recordingStore";

export const useApplicationStore = defineStore("application", () => {
  const { t } = i18n.global;
  const snapshot = ref<AppSnapshot | null>(null);
  const isLoading = ref(true);
  const error = ref<string | null>(null);
  const operationError = ref<string | null>(null);
  const audioDevices = ref<AudioDevices | null>(null);
  const libraryRevision = ref(0);
  const importRequestRevision = ref(0);
  let subscribed = false;

  const projects = computed(() => snapshot.value?.projects ?? []);
  const recentSessions = computed(() => snapshot.value?.recent_sessions ?? []);
  const projectSessionCounts = computed(() => {
    const counts = new Map<string, number>();

    for (const session of recentSessions.value) {
      if (!session.project_id) {
        continue;
      }

      counts.set(session.project_id, (counts.get(session.project_id) ?? 0) + 1);
    }

    return counts;
  });
  const activeJobs = computed(() => snapshot.value?.active_jobs ?? []);
  const runningJobs = computed(() =>
    activeJobs.value.filter((job) =>
      ["queued", "preparing", "running"].includes(job.state),
    ),
  );
  const settings = computed(() => snapshot.value?.settings);

  async function bootstrap() {
    isLoading.value = true;
    error.value = null;

    try {
      snapshot.value = await native.bootstrap();
      applyTheme(snapshot.value.settings.appearance.theme);
      applyReducedMotion(snapshot.value.settings.appearance.reduced_motion);

      if (!subscribed) {
        await native.subscribe(handleAppEvent);
        subscribed = true;
      }
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason);
    } finally {
      isLoading.value = false;
    }
  }

  function handleAppEvent(event: import("@/types/domain").AppEvent) {
    if (event.type === "recording_levels") {
      useRecordingStore().handleEvent(event);
      return;
    }

    if (event.type === "recording_state_changed") {
      useRecordingStore().handleEvent(event);
      return;
    }

    if (event.type === "job_state_changed") {
      upsertJob(event.payload);
      return;
    }

    if (event.type === "library_changed") {
      void refreshSnapshot();
      return;
    }

    if (event.type === "import_requested") {
      importRequestRevision.value += 1;
      return;
    }

    if (event.type === "attention_required") {
      operationError.value = event.payload;
      return;
    }

    if (event.type !== "job_progress") {
      return;
    }

    const jobs = snapshot.value?.active_jobs;

    if (!jobs) {
      return;
    }

    const existing = jobs.find((job) => job.id === event.payload.job_id);

    if (existing) {
      existing.progress = event.payload.progress;
      existing.updated_at = new Date().toISOString();
    }
  }

  async function refreshSnapshot() {
    operationError.value = null;

    try {
      snapshot.value = await native.bootstrap();
      libraryRevision.value += 1;
    } catch (reason) {
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  function upsertJob(job: import("@/types/domain").Job) {
    const jobs = snapshot.value?.active_jobs;

    if (!jobs) {
      return;
    }

    const index = jobs.findIndex((candidate) => candidate.id === job.id);

    if (["completed", "canceled"].includes(job.state)) {
      if (index >= 0) {
        jobs.splice(index, 1);
      }
      return;
    }

    if (index >= 0) {
      jobs[index] = job;
      return;
    }

    jobs.unshift(job);
  }

  async function retryJob(jobId: string) {
    operationError.value = null;

    try {
      upsertJob(await native.retryJob(jobId));
    } catch (reason) {
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function cancelJob(jobId: string) {
    operationError.value = null;

    try {
      upsertJob(await native.cancelJob(jobId));
    } catch (reason) {
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function chooseLibrary() {
    operationError.value = null;

    try {
      const path = await native.chooseLibrary();

      if (path) {
        snapshot.value = await native.initializeLibrary(path);
      }
    } catch (reason) {
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function createProject(name: string) {
    if (!snapshot.value?.library) {
      await chooseLibrary();
    }

    if (!snapshot.value?.library) {
      throw new Error(t("errors.chooseLibraryBeforeProject"));
    }

    const project = await native.createProject(name);
    snapshot.value?.projects.push(project);
    snapshot.value?.projects.sort((left, right) =>
      left.name.localeCompare(right.name),
    );
    return project;
  }

  function defaultRecordingProjectId() {
    const selection = settings.value?.recording_project_selection;

    if (selection?.kind === "inbox") {
      return null;
    }

    if (
      selection?.kind === "project" &&
      projects.value.some((project) => project.id === selection.project_id)
    ) {
      return selection.project_id;
    }

    return projects.value[0]?.id ?? null;
  }

  async function saveRecordingProjectSelection(projectId: string | null) {
    const recordingProjectSelection: RecordingProjectSelection = projectId
      ? { kind: "project", project_id: projectId }
      : { kind: "inbox" };

    return saveSettings({
      recording_project_selection: recordingProjectSelection,
    });
  }

  async function importMedia(path?: string) {
    operationError.value = null;

    if (!snapshot.value?.library) {
      await chooseLibrary();
    }

    if (!snapshot.value?.library) {
      return null;
    }

    try {
      const session = await native.importMedia(path);

      if (session) {
        snapshot.value.recent_sessions.unshift(session);
      }

      return session;
    } catch (reason) {
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
      return null;
    }
  }

  async function loadAudioDevices() {
    operationError.value = null;

    try {
      audioDevices.value = await native.audioDevices();
    } catch (reason) {
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function openSystemAudioPermissionSettings() {
    operationError.value = null;

    try {
      await native.openSystemAudioPermissionSettings();
    } catch (reason) {
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  function replaceRecentSession(session: Session) {
    const index = snapshot.value?.recent_sessions.findIndex(
      (candidate) => candidate.id === session.id,
    );

    if (snapshot.value && index !== undefined && index >= 0) {
      snapshot.value.recent_sessions[index] = session;
    }
  }

  async function renameSession(sessionId: string, title: string) {
    operationError.value = null;

    try {
      const session = await native.renameSession(sessionId, title);
      replaceRecentSession(session);
      return session;
    } catch (reason) {
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
      return null;
    }
  }

  async function moveSession(sessionId: string, projectId: string | null) {
    operationError.value = null;

    try {
      const session = await native.moveSession(sessionId, projectId);
      replaceRecentSession(session);
      return session;
    } catch (reason) {
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
      return null;
    }
  }

  function applyTheme(theme: "system" | "light" | "dark") {
    document.documentElement.dataset.theme = theme;
  }

  function applyReducedMotion(reducedMotion: boolean) {
    document.documentElement.dataset.reducedMotion = String(reducedMotion);
  }

  async function saveSettings(
    updates: Partial<NonNullable<typeof settings.value>>,
  ) {
    if (!snapshot.value) {
      return false;
    }

    operationError.value = null;
    const previousSettings = snapshot.value.settings;
    const nextSettings = {
      ...previousSettings,
      ...updates,
    };
    snapshot.value.settings = nextSettings;

    try {
      snapshot.value.settings = await native.saveSettings(nextSettings);
      return true;
    } catch (reason) {
      snapshot.value.settings = previousSettings;
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
      return false;
    }
  }

  return {
    snapshot,
    isLoading,
    error,
    operationError,
    audioDevices,
    libraryRevision,
    importRequestRevision,
    projects,
    recentSessions,
    projectSessionCounts,
    activeJobs,
    runningJobs,
    settings,
    bootstrap,
    chooseLibrary,
    createProject,
    defaultRecordingProjectId,
    saveRecordingProjectSelection,
    importMedia,
    loadAudioDevices,
    openSystemAudioPermissionSettings,
    replaceRecentSession,
    renameSession,
    moveSession,
    upsertJob,
    retryJob,
    cancelJob,
    applyTheme,
    applyReducedMotion,
    saveSettings,
    refreshLibrary: refreshSnapshot,
  };
});
