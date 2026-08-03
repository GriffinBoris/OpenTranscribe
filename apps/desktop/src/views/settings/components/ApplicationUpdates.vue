<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { Download, RefreshCw } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppDialog from "@/components/ui/AppDialog.vue";
import AppProgressBar from "@/components/ui/AppProgressBar.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import { useUpdateStore } from "@/views/application/updateStore";

const props = defineProps<{
  hasActiveWork: boolean;
}>();

const { t } = useI18n();
const updater = useUpdateStore();
const updateDialogOpen = ref(false);
const downloadPercent = computed(() => {
  const progress = updater.downloadProgress;

  if (!progress?.total_bytes) {
    return 0;
  }

  return Math.min(
    100,
    Math.round((progress.completed_bytes / progress.total_bytes) * 100),
  );
});

onMounted(() => void updater.initialize());

async function checkForUpdate() {
  await updater.checkForUpdate();

  if (updater.availableUpdate) {
    updateDialogOpen.value = true;
  }
}

async function installUpdate() {
  if (props.hasActiveWork) {
    return;
  }

  await updater.installUpdate();
}
</script>

<template>
  <div
    class="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
  >
    <span class="grid min-w-0 gap-1">
      <strong class="flex items-center gap-2">
        <Download :size="16" />{{ t("settings.application.updates.title") }}
      </strong>
      <small class="text-ink-muted">
        {{ t("settings.application.updates.description") }}
      </small>
      <small
        v-if="updater.initialized && !updater.configured"
        class="text-ink-muted"
      >
        {{ t("settings.application.updates.unavailable") }}
      </small>
      <small v-else-if="updater.isChecking" class="text-ink-muted">
        {{ t("settings.application.updates.checking") }}
      </small>
      <small v-else-if="updater.error" class="text-accent">
        {{ updater.error }}
      </small>
      <small v-else-if="updater.availableUpdate" class="text-ink-muted">
        {{
          t("settings.application.updates.available", {
            version: updater.availableUpdate.version,
          })
        }}
      </small>
      <small v-else-if="updater.configured" class="text-ink-muted">
        {{ t("settings.application.updates.upToDate") }}
      </small>
    </span>
    <AppButton
      variant="secondary"
      :loading="updater.isChecking"
      :disabled="!updater.configured || updater.isInstalling"
      @click="checkForUpdate"
    >
      <RefreshCw :size="15" />{{ t("settings.application.updates.check") }}
    </AppButton>
  </div>

  <AppDialog
    :open="updateDialogOpen"
    :title="t('settings.application.updates.dialogTitle')"
    @update:open="updateDialogOpen = $event"
  >
    <div class="grid gap-4">
      <StatusPill tone="success">
        {{
          t("settings.application.updates.available", {
            version: updater.availableUpdate?.version,
          })
        }}
      </StatusPill>
      <p class="m-0">
        {{
          updater.availableUpdate?.notes ||
          t("settings.application.updates.noNotes")
        }}
      </p>
      <p v-if="hasActiveWork" class="text-warning m-0 text-sm">
        {{ t("settings.application.updates.busy") }}
      </p>
      <div v-if="updater.isInstalling" class="grid gap-2">
        <AppProgressBar
          :value="downloadPercent"
          :accessible-label="t('settings.application.updates.downloading')"
        />
        <small class="text-ink-muted">{{
          t("settings.application.updates.downloading", {
            percent: downloadPercent,
          })
        }}</small>
      </div>
      <p v-if="updater.error" class="text-accent m-0 text-sm">
        {{ updater.error }}
      </p>
    </div>
    <template #footer>
      <AppButton
        variant="ghost"
        :disabled="updater.isInstalling"
        @click="updateDialogOpen = false"
      >
        {{ t("settings.application.reset.cancel") }}
      </AppButton>
      <AppButton
        variant="primary"
        :loading="updater.isInstalling"
        :disabled="hasActiveWork || !updater.availableUpdate"
        @click="installUpdate"
      >
        <Download :size="15" />{{ t("settings.application.updates.install") }}
      </AppButton>
    </template>
  </AppDialog>
</template>
