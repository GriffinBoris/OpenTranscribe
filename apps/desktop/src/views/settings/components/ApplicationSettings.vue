<script setup lang="ts">
import { computed, ref } from "vue";
import { RotateCcw, ShieldCheck } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import AppButton from "@/components/ui/AppButton.vue";
import AppDialog from "@/components/ui/AppDialog.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useLocalModelsStore } from "@/views/application/localModelsStore";
import { useRecordingStore } from "@/views/application/recordingStore";

const application = useApplicationStore();
const localModels = useLocalModelsStore();
const recording = useRecordingStore();
const router = useRouter();
const { t } = useI18n();
const resetDialogOpen = ref(false);
const isResetting = ref(false);

const hasActiveWork = computed(
  () =>
    Boolean(recording.activeRecording) ||
    application.runningJobs.length > 0 ||
    localModels.downloadingModelId !== null,
);

async function runSetup() {
  if (await application.saveSettings({ setup_completed: false })) {
    await router.push("/");
  }
}

async function resetApplication() {
  isResetting.value = true;

  if (!(await application.resetApplicationData())) {
    isResetting.value = false;
  }
}
</script>

<template>
  <AppSurface id="application">
    <div class="border-b border-[var(--divider)] pb-3.5">
      <h2 class="mb-[5px] text-2xl">
        {{ t("settings.application.title") }}
      </h2>
      <p class="text-ink-muted mt-[3px] leading-[var(--line-height-body)]">
        {{ t("settings.application.description") }}
      </p>
    </div>

    <div
      class="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
    >
      <span class="grid min-w-0 gap-1">
        <strong class="flex items-center gap-2">
          <ShieldCheck :size="16" />{{ t("settings.application.setup.title") }}
        </strong>
        <small class="text-ink-muted">
          {{ t("settings.application.setup.description") }}
        </small>
      </span>
      <AppButton
        variant="secondary"
        :disabled="Boolean(recording.activeRecording)"
        @click="runSetup"
      >
        {{ t("settings.application.setup.action") }}
      </AppButton>
    </div>

    <div
      class="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
    >
      <span class="grid min-w-0 gap-1">
        <strong class="flex items-center gap-2">
          <RotateCcw :size="16" />{{ t("settings.application.reset.title") }}
        </strong>
        <small class="text-ink-muted">
          {{ t("settings.application.reset.description") }}
        </small>
        <small v-if="hasActiveWork" class="text-warning">
          {{ t("settings.application.reset.busy") }}
        </small>
      </span>
      <AppButton
        variant="danger"
        :disabled="hasActiveWork"
        @click="resetDialogOpen = true"
      >
        {{ t("settings.application.reset.action") }}
      </AppButton>
    </div>

    <AppDialog
      :open="resetDialogOpen"
      :title="t('settings.application.reset.dialogTitle')"
      @update:open="resetDialogOpen = $event"
    >
      <div class="grid gap-3">
        <p class="m-0">{{ t("settings.application.reset.dialogBody") }}</p>
        <p class="text-ink-muted m-0 text-sm">
          {{ t("settings.application.reset.permissions") }}
        </p>
      </div>
      <template #footer>
        <AppButton
          variant="ghost"
          :disabled="isResetting"
          @click="resetDialogOpen = false"
        >
          {{ t("settings.application.reset.cancel") }}
        </AppButton>
        <AppButton
          variant="danger"
          :loading="isResetting"
          @click="resetApplication"
        >
          {{ t("settings.application.reset.confirm") }}
        </AppButton>
      </template>
    </AppDialog>
  </AppSurface>
</template>
