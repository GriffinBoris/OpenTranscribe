import { computed, ref } from "vue";
import { defineStore } from "pinia";

import { native } from "@/core/native";
import type { JobProgress, LocalModel } from "@/types/domain";

export const useLocalModelsStore = defineStore("local-models", () => {
  const models = ref<LocalModel[]>([]);
  const isLoading = ref(false);
  const storagePath = ref<string | null>(null);
  const isMovingStorage = ref(false);
  const downloadingModelId = ref<string | null>(null);
  const progress = ref<JobProgress | null>(null);
  const error = ref<string | null>(null);

  const installedModels = computed(() =>
    models.value.filter(
      (model) => model.preset !== "cleanup" && isReady(model),
    ),
  );
  const dictationModels = computed(() =>
    installedModels.value.filter((model) => model.preset !== "diarization"),
  );

  function isReady(model: LocalModel) {
    return (
      model.installed &&
      (!model.required_model_id ||
        models.value.some(
          (dependency) =>
            dependency.id === model.required_model_id && dependency.installed,
        ))
    );
  }

  async function load() {
    isLoading.value = true;
    error.value = null;

    try {
      const [nextModels, nextStoragePath] = await Promise.all([
        native.localModelStatuses(),
        native.localModelStoragePath(),
      ]);
      models.value = nextModels;
      storagePath.value = nextStoragePath;
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason);
    } finally {
      isLoading.value = false;
    }
  }

  async function download(modelId: string) {
    if (downloadingModelId.value) {
      return;
    }

    downloadingModelId.value = modelId;
    progress.value = null;
    error.value = null;

    try {
      await native.downloadLocalModel(modelId, (nextProgress) => {
        progress.value = nextProgress;
      });
      await load();
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason);
    } finally {
      downloadingModelId.value = null;
      progress.value = null;
    }
  }

  async function remove(modelId: string) {
    error.value = null;

    try {
      await native.removeLocalModel(modelId);
      await load();
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function moveStorage(path: string) {
    isMovingStorage.value = true;
    error.value = null;

    try {
      storagePath.value = await native.moveLocalModels(path);
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason);
      return false;
    } finally {
      isMovingStorage.value = false;
    }

    return true;
  }

  return {
    models,
    storagePath,
    installedModels,
    dictationModels,
    isReady,
    isLoading,
    isMovingStorage,
    downloadingModelId,
    progress,
    error,
    load,
    download,
    remove,
    moveStorage,
  };
});
