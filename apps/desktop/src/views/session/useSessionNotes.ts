import { onBeforeUnmount, ref, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import { useSessionStore } from "@/views/session/sessionStore";

export function useSessionNotes(
  sessionId: Ref<string>,
  markerPosition: Ref<number>,
  isTrashing: Ref<boolean>,
) {
  const { t } = useI18n();
  const sessionStore = useSessionStore();
  const notes = ref("");
  const notesLoaded = ref(false);
  const notesState = ref(t("session.saved"));
  let notesTimer: number | undefined;

  async function persistNotes() {
    notesState.value = t("session.saving");
    notesState.value = (await sessionStore.saveNotes(
      sessionId.value,
      notes.value,
    ))
      ? t("session.saved")
      : t("session.unsaved");
  }

  function initialize(markdown: string, contentHash: string) {
    notes.value = markdown;
    sessionStore.startNotesEditing(sessionId.value, contentHash);
    notesLoaded.value = true;
  }

  function finishLoading() {
    notesLoaded.value = true;
  }

  function formatTimestamp(milliseconds: number) {
    const seconds = Math.floor(milliseconds / 1_000);
    return `${Math.floor(seconds / 60)}:${(seconds % 60).toString().padStart(2, "0")}`;
  }

  function insertTimestamp() {
    const prefix = notes.value.trimEnd();
    notes.value = `${prefix}${prefix ? "\n" : ""}- [${formatTimestamp(markerPosition.value)}] `;
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

  onBeforeUnmount(() => {
    if (notesTimer === undefined) {
      return;
    }

    window.clearTimeout(notesTimer);

    if (!isTrashing.value) {
      void persistNotes();
    }
  });

  return {
    notes,
    notesState,
    initialize,
    finishLoading,
    insertTimestamp,
  };
}
