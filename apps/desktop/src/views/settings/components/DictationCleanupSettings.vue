<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import AppButton from "@/components/ui/AppButton.vue";
import AppToggleSwitch from "@/components/ui/AppToggleSwitch.vue";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useLocalModelsStore } from "@/views/application/localModelsStore";

const application = useApplicationStore();
const localModels = useLocalModelsStore();
const { t } = useI18n();
const model = computed(() =>
  localModels.models.find((model) => model.preset === "cleanup"),
);
const downloading = computed(
  () => localModels.downloadingModelId === model.value?.id,
);
</script>

<template>
  <div
    class="grid grid-cols-[minmax(160px,1fr)_minmax(220px,1.3fr)] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
  >
    <span class="grid gap-1">
      <strong>{{ t("settings.dictation.cleanupTitle") }}</strong>
      <small>{{ t("settings.dictation.cleanupDescription") }}</small>
    </span>
    <AppToggleSwitch
      v-if="model?.installed"
      :model-value="application.settings?.dictation_cleanup_enabled ?? false"
      :accessible-label="t('settings.dictation.cleanupTitle')"
      @update:model-value="
        application.saveSettings({ dictation_cleanup_enabled: $event })
      "
    />
    <AppButton
      v-else-if="model"
      variant="secondary"
      :loading="downloading"
      :disabled="localModels.downloadingModelId !== null"
      @click="localModels.download(model.id)"
    >
      {{
        t(
          downloading
            ? "settings.dictation.cleanupDownloading"
            : "settings.dictation.cleanupDownload",
        )
      }}
    </AppButton>
    <p
      v-if="localModels.error"
      class="text-accent col-span-full m-0 text-sm"
      role="alert"
    >
      {{ localModels.error }}
    </p>
  </div>
</template>
