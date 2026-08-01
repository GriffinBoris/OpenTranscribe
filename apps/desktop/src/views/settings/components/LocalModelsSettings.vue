<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppDialog from "@/components/ui/AppDialog.vue";
import AppProgressBar from "@/components/ui/AppProgressBar.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import { native } from "@/core/native";
import { useLocalModelsStore } from "@/views/application/localModelsStore";

const localModels = useLocalModelsStore();
const { t } = useI18n();
const selectedStoragePath = ref<string | null>(null);
const moveDialogOpen = ref(false);

const hasInstalledModels = computed(
  () => localModels.installedModels.length > 0,
);

function modelSize(byteCount: number) {
  return `${Math.round(byteCount / 1_000_000)} MB`;
}

onMounted(() => {
  void localModels.load();
});

async function chooseStoragePath() {
  const path = await native.chooseLocalModelStorage();

  if (!path || path === localModels.storagePath) {
    return;
  }

  selectedStoragePath.value = path;
  moveDialogOpen.value = true;
}

async function moveStorage() {
  if (
    !selectedStoragePath.value ||
    !(await localModels.moveStorage(selectedStoragePath.value))
  ) {
    return;
  }

  moveDialogOpen.value = false;
  selectedStoragePath.value = null;
}
</script>

<template>
  <AppSurface id="models">
    <div
      class="flex items-start justify-between gap-4 border-b border-[var(--divider)] pb-3.5"
    >
      <div>
        <h2 class="mb-[5px] text-2xl">{{ t("models.title") }}</h2>
        <p class="text-ink mt-[3px] leading-[var(--line-height-body)]">
          {{ t("models.description") }}
        </p>
      </div>
      <StatusPill tone="local">{{ t("models.private") }}</StatusPill>
    </div>
    <div
      class="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
    >
      <span class="grid min-w-0 gap-1">
        <strong>{{ t("models.storage.title") }}</strong>
        <small class="text-ink-muted wrap-anywhere">
          {{ localModels.storagePath ?? t("models.storage.loading") }}
        </small>
      </span>
      <AppButton
        variant="secondary"
        :disabled="localModels.isMovingStorage"
        @click="chooseStoragePath"
      >
        {{ t("models.storage.change") }}
      </AppButton>
    </div>
    <p
      class="text-ink-muted m-0 border-t border-[var(--divider)] py-3 text-sm leading-[var(--line-height-body)]"
    >
      {{ t("models.source") }}
    </p>
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
          :disabled="localModels.downloadingModelId !== null"
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
    <AppDialog
      :open="moveDialogOpen"
      :title="t('models.storage.dialogTitle')"
      @update:open="moveDialogOpen = $event"
    >
      <div class="grid gap-3">
        <p class="m-0">
          {{
            t(
              hasInstalledModels
                ? "models.storage.dialogWithModels"
                : "models.storage.dialogWithoutModels",
            )
          }}
        </p>
        <p class="text-ink-muted m-0 text-sm wrap-anywhere">
          {{ selectedStoragePath }}
        </p>
      </div>
      <template #footer>
        <AppButton
          variant="ghost"
          :disabled="localModels.isMovingStorage"
          @click="moveDialogOpen = false"
        >
          {{ t("models.storage.cancel") }}
        </AppButton>
        <AppButton :loading="localModels.isMovingStorage" @click="moveStorage">
          {{ t("models.storage.confirm") }}
        </AppButton>
      </template>
    </AppDialog>
  </AppSurface>
</template>
