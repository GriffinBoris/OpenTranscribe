import { ref, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import type { Session } from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useRecordingStore } from "@/views/application/recordingStore";
import { useSessionStore } from "@/views/session/sessionStore";

export function useSessionActions(
  sessionId: Ref<string>,
  session: Ref<Session | null | undefined>,
  sessionError: Ref<string | null>,
) {
  const { t } = useI18n();
  const router = useRouter();
  const application = useApplicationStore();
  const recording = useRecordingStore();
  const sessionStore = useSessionStore();
  const renameOpen = ref(false);
  const trashConfirmOpen = ref(false);
  const isTrashing = ref(false);
  const isMovingSession = ref(false);
  const isRenamingSession = ref(false);
  const titleDraft = ref("");

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

  return {
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
  };
}
