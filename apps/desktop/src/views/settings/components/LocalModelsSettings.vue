<script setup lang="ts">
import { onMounted } from "vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppProgressBar from "@/components/ui/AppProgressBar.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import { useLocalModelsStore } from "@/views/application/localModelsStore";

const localModels = useLocalModelsStore();
const { t } = useI18n();

function modelSize(byteCount: number) {
  return `${Math.round(byteCount / 1_000_000)} MB`;
}

onMounted(() => {
  void localModels.load();
});
</script>

<template>
  <AppSurface id="models">
    <div
      class="flex items-start justify-between gap-4 border-b border-[var(--divider)] pb-3.5"
    >
      <div>
        <h2 class="mb-[5px] text-2xl">{{ t("models.title") }}</h2>
        <p class="text-ink-muted mt-[3px] leading-[var(--line-height-body)]">
          {{ t("models.description") }}
        </p>
      </div>
      <StatusPill tone="local">{{ t("models.private") }}</StatusPill>
    </div>
    <div
      v-if="localModels.isLoading && !localModels.models.length"
      class="text-ink-muted grid grid-cols-[minmax(160px,1fr)_auto] items-center gap-3 border-t border-[var(--divider)] py-3"
      role="status"
    >
      <AppProgressBar :accessible-label="t('models.loading')" />
      <small>{{ t("models.loadingModels") }}</small>
    </div>
    <div
      v-for="model in localModels.models"
      :key="model.id"
      class="border-t border-[var(--divider)]"
      role="group"
      :aria-label="model.label"
    >
      <div
        class="grid grid-cols-[minmax(220px,1fr)_auto_auto] items-center gap-5 py-3 max-[700px]:grid-cols-1"
      >
        <span class="grid gap-1">
          <strong>{{ model.label }}</strong>
          <small>
            {{ model.description }} · {{ modelSize(model.byte_count) }}
          </small>
        </span>
        <StatusPill :tone="model.installed ? 'success' : 'neutral'">
          {{
            model.installed ? t("models.installed") : t("models.notInstalled")
          }}
        </StatusPill>
        <AppButton
          v-if="model.installed"
          size="small"
          variant="ghost"
          @click="localModels.remove(model.id)"
        >
          {{ t("models.remove") }}
        </AppButton>
        <AppButton
          v-else
          size="small"
          :loading="localModels.downloadingModelId === model.id"
          :disabled="
            localModels.downloadingModelId !== null &&
            localModels.downloadingModelId !== model.id
          "
          @click="localModels.download(model.id)"
        >
          {{
            localModels.downloadingModelId === model.id
              ? t("models.downloading")
              : t("models.download")
          }}
        </AppButton>
      </div>
    </div>
    <div
      v-if="localModels.error"
      class="text-accent grid grid-cols-[minmax(160px,1fr)_auto] items-center gap-3 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
      role="alert"
    >
      <span>{{ localModels.error }}</span>
      <AppButton size="small" variant="secondary" @click="localModels.load">
        {{ t("models.retry") }}
      </AppButton>
    </div>
  </AppSurface>
</template>
