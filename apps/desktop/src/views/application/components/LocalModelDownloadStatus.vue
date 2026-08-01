<script setup lang="ts">
import { computed } from "vue";
import { Download } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppProgressBar from "@/components/ui/AppProgressBar.vue";
import { useLocalModelsStore } from "@/views/application/localModelsStore";

const localModels = useLocalModelsStore();
const { t } = useI18n();

const downloadingModel = computed(() =>
  localModels.models.find(
    (model) => model.id === localModels.downloadingModelId,
  ),
);
const progressValue = computed(() => {
  if (!localModels.progress?.total_units) {
    return undefined;
  }

  return (
    (localModels.progress.completed_units / localModels.progress.total_units) *
    100
  );
});
const progressLabel = computed(() =>
  progressValue.value === undefined
    ? t("models.preparingDownload")
    : t("models.downloadPercent", {
        percent: Math.round(progressValue.value),
      }),
);
</script>

<template>
  <Transition name="workspace-fade">
    <section
      v-if="downloadingModel"
      class="rounded-app-md shadow-app-pane mx-5 mt-3.5 flex items-center gap-3 border border-[color-mix(in_srgb,var(--lichen)_24%,var(--border))] bg-[color-mix(in_srgb,var(--lichen-soft)_62%,var(--surface-raised))] px-3.5 py-3"
      role="status"
      aria-live="polite"
      aria-atomic="true"
      data-testid="global-model-download"
    >
      <span
        class="bg-lichen-soft text-lichen rounded-app-sm grid size-8 shrink-0 place-items-center"
        aria-hidden="true"
      >
        <Download :size="17" />
      </span>

      <div class="grid min-w-0 flex-1 gap-2">
        <div class="flex min-w-0 items-center justify-between gap-3">
          <strong class="truncate text-sm">
            {{
              t("models.downloadProgress", { model: downloadingModel.label })
            }}
          </strong>
          <small class="text-ink-muted shrink-0 tabular-nums">
            {{ progressLabel }}
          </small>
        </div>
        <AppProgressBar
          :value="progressValue"
          :accessible-label="
            t('models.downloadProgress', { model: downloadingModel.label })
          "
        />
      </div>
    </section>
  </Transition>
</template>
