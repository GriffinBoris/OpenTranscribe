import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import type { Session } from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";

export function useMoveSession() {
  const application = useApplicationStore();
  const { t } = useI18n();
  const movingSessionIds = ref(new Set<string>());
  const projectOptions = computed(() => [
    { label: t("navigation.inbox"), value: "" },
    ...application.projects.map((project) => ({
      label: project.name,
      value: project.id,
    })),
  ]);

  function isMoving(sessionId: string) {
    return movingSessionIds.value.has(sessionId);
  }

  async function moveSession(session: Session, projectId: string) {
    if (projectId === (session.project_id ?? "")) {
      return;
    }

    movingSessionIds.value = new Set(movingSessionIds.value).add(session.id);
    await application.moveSession(session.id, projectId || null);
    const remainingSessionIds = new Set(movingSessionIds.value);
    remainingSessionIds.delete(session.id);
    movingSessionIds.value = remainingSessionIds;
  }

  return { isMoving, moveSession, projectOptions };
}
