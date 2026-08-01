import { ref } from "vue";
import { defineStore } from "pinia";

import { native } from "@/core/native";
import type { Session } from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";

export const useTrashStore = defineStore("trash", () => {
  const application = useApplicationStore();
  const sessions = ref<Session[]>([]);
  const isLoading = ref(false);
  const restoringSessionId = ref<string | null>(null);
  const error = ref<string | null>(null);

  async function load() {
    isLoading.value = true;
    error.value = null;

    try {
      sessions.value = await native.trashedSessions();
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason);
    } finally {
      isLoading.value = false;
    }
  }

  async function restore(sessionId: string) {
    restoringSessionId.value = sessionId;
    error.value = null;

    try {
      await native.restoreSession(sessionId);
      sessions.value = sessions.value.filter(
        (session) => session.id !== sessionId,
      );
      await application.refreshLibrary();
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason);
    } finally {
      restoringSessionId.value = null;
    }
  }

  return {
    sessions,
    isLoading,
    restoringSessionId,
    error,
    load,
    restore,
  };
});
