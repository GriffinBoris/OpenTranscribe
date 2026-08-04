import { computed, ref } from "vue";
import { defineStore } from "pinia";

import { native } from "@/core/native";
import { i18n } from "@/i18n";
import type { AppUpdate, UpdateDownloadProgress } from "@/types/domain";

const { t } = i18n.global;

export const useUpdateStore = defineStore("updater", () => {
  const configured = ref(false);
  const initialized = ref(false);
  const availableUpdate = ref<AppUpdate | null>(null);
  const error = ref<string | null>(null);
  const isChecking = ref(false);
  const isInstalling = ref(false);
  const downloadProgress = ref<UpdateDownloadProgress | null>(null);
  const isBusy = computed(() => isChecking.value || isInstalling.value);

  async function initialize() {
    if (initialized.value) {
      return;
    }

    initialized.value = true;
    configured.value = await native.updaterConfigured();

    if (configured.value) {
      await checkForUpdate();
    }
  }

  async function checkForUpdate() {
    if (!configured.value || isBusy.value) {
      return;
    }

    isChecking.value = true;
    error.value = null;

    try {
      availableUpdate.value = await native.checkForUpdate();
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason);
    } finally {
      isChecking.value = false;
    }
  }

  async function installUpdate() {
    if (!availableUpdate.value || isBusy.value) {
      return;
    }

    isInstalling.value = true;
    error.value = null;
    downloadProgress.value = null;

    try {
      switch (await native.updaterInstallationStatus()) {
        case "macos_read_only":
          error.value = t(
            "settings.application.updates.macosReadOnlyInstallation",
          );
          return;
        case "linux_not_writable":
          error.value = t(
            "settings.application.updates.linuxNotWritableInstallation",
          );
          return;
        case "ready":
          break;
      }

      await native.installUpdate((progress) => {
        downloadProgress.value = progress;
      });
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason);
    } finally {
      isInstalling.value = false;
    }
  }

  return {
    configured,
    initialized,
    availableUpdate,
    error,
    isChecking,
    isInstalling,
    downloadProgress,
    isBusy,
    initialize,
    checkForUpdate,
    installUpdate,
  };
});
