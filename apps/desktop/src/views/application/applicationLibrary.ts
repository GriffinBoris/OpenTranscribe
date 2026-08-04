import { computed, ref, type Ref } from "vue";

import { native } from "@/core/native";
import { i18n } from "@/i18n";
import type {
  AppSettings,
  AppSnapshot,
  RecordingProjectSelection,
  Session,
} from "@/types/domain";

export function createApplicationLibrary(
  snapshot: Ref<AppSnapshot | null>,
  operationError: Ref<string | null>,
  saveSettings: (updates: Partial<AppSettings>) => Promise<boolean>,
) {
  const { t } = i18n.global;
  const projects = computed(() => snapshot.value?.projects ?? []);
  const recentSessions = computed(() => snapshot.value?.recent_sessions ?? []);
  const isImporting = ref(false);
  const projectSessionCounts = computed(() => {
    const counts = new Map<string, number>();

    for (const session of recentSessions.value) {
      if (session.project_id) {
        counts.set(
          session.project_id,
          (counts.get(session.project_id) ?? 0) + 1,
        );
      }
    }

    return counts;
  });

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
    snapshot.value = {
      ...snapshot.value,
      projects: [...snapshot.value.projects, project].sort((left, right) =>
        left.name.localeCompare(right.name),
      ),
    };
    return project;
  }

  function defaultRecordingProjectId() {
    const selection = snapshot.value?.settings.recording_project_selection;

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
    if (isImporting.value) {
      return null;
    }

    operationError.value = null;
    isImporting.value = true;

    try {
      if (!snapshot.value?.library) {
        await chooseLibrary();
      }

      if (!snapshot.value?.library) {
        return null;
      }

      const session = await native.importMedia(path);

      if (session) {
        snapshot.value.recent_sessions.unshift(session);
      }

      return session;
    } catch (reason) {
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
      return null;
    } finally {
      isImporting.value = false;
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

  return {
    projects,
    recentSessions,
    isImporting,
    projectSessionCounts,
    chooseLibrary,
    createProject,
    defaultRecordingProjectId,
    saveRecordingProjectSelection,
    importMedia,
    replaceRecentSession,
    renameSession,
    moveSession,
  };
}
