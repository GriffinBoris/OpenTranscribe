import { computed, ref } from "vue";
import { defineStore } from "pinia";

import { native } from "@/core/native";
import type { JobProgress, LocalModel } from "@/types/domain";

export const useLocalModelsStore = defineStore("local-models", () => {
  const models = ref<LocalModel[]>([]);
  const isLoading = ref(false);
  const downloadingModelId = ref<string | null>(null);
  const progress = ref<JobProgress | null>(null);
  const error = ref<string | null>(null);

  const installedModels = computed(() =>
    models.value.filter((model) => model.installed),
  );

  async function load() {
    isLoading.value = true;
    error.value = null;

    try {
      models.value = await native.localModelStatuses();
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason);
    } finally {
      isLoading.value = false;
    }
  }

  async function download(modelId: string) {
    downloadingModelId.value = modelId;
    progress.value = null;
    error.value = null;

    try {
      const model = await native.downloadLocalModel(modelId, (nextProgress) => {
        progress.value = nextProgress;
      });
      replace(model);
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
      replace(await native.removeLocalModel(modelId));
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason);
    }
  }

  function replace(model: LocalModel) {
    const index = models.value.findIndex((item) => item.id === model.id);

    if (index >= 0) {
      models.value[index] = model;
    }
  }

  return {
    models,
    installedModels,
    isLoading,
    downloadingModelId,
    progress,
    error,
    load,
    download,
    remove,
  };
});
