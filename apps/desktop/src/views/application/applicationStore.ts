import { ref } from "vue";
import { defineStore } from "pinia";

import { native } from "@/core/native";
import type { AppSnapshot, TranscriptionPreviewUpdate } from "@/types/domain";
import { createApplicationJobs } from "@/views/application/applicationJobs";
import { createApplicationLibrary } from "@/views/application/applicationLibrary";
import { createApplicationSettings } from "@/views/application/applicationSettings";

export const useApplicationStore = defineStore("application", () => {
  const snapshot = ref<AppSnapshot | null>(null);
  const isLoading = ref(true);
  const error = ref<string | null>(null);
  const operationError = ref<string | null>(null);
  const libraryRevision = ref(0);
  const importRequestRevision = ref(0);
  const transcriptionPreviews = ref<Record<string, string>>({});
  const settings = createApplicationSettings(snapshot, operationError);
  const library = createApplicationLibrary(
    snapshot,
    operationError,
    settings.saveSettings,
  );
  const jobs = createApplicationJobs(snapshot, operationError);

  async function bootstrap() {
    isLoading.value = true;
    error.value = null;

    try {
      snapshot.value = await native.bootstrap();
      settings.applyAppearance(snapshot.value.settings);
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason);
    } finally {
      isLoading.value = false;
    }
  }

  async function refreshLibrary() {
    operationError.value = null;

    try {
      snapshot.value = await native.bootstrap();
      libraryRevision.value += 1;
    } catch (reason) {
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  function requestImport() {
    importRequestRevision.value += 1;
  }

  function updateTranscriptionPreview(update: TranscriptionPreviewUpdate) {
    transcriptionPreviews.value[update.session_id] = update.text;
  }

  function clearTranscriptionPreview(sessionId: string | null) {
    if (sessionId) {
      delete transcriptionPreviews.value[sessionId];
    }
  }

  return {
    snapshot,
    isLoading,
    error,
    operationError,
    libraryRevision,
    importRequestRevision,
    transcriptionPreviews,
    bootstrap,
    refreshLibrary,
    requestImport,
    updateTranscriptionPreview,
    clearTranscriptionPreview,
    ...settings,
    ...library,
    ...jobs,
  };
});
