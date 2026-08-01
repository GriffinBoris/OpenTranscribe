import { computed, ref, type Ref } from "vue";

import { native } from "@/core/native";
import type { AppSettings, AppSnapshot, AudioDevices } from "@/types/domain";

export function createApplicationSettings(
  snapshot: Ref<AppSnapshot | null>,
  operationError: Ref<string | null>,
) {
  const audioDevices = ref<AudioDevices | null>(null);
  const settings = computed(() => snapshot.value?.settings);

  function applyTheme(theme: "system" | "light" | "dark") {
    document.documentElement.dataset.theme = theme;
  }

  function applyReducedMotion(reducedMotion: boolean) {
    document.documentElement.dataset.reducedMotion = String(reducedMotion);
  }

  function applyAppearance(nextSettings: AppSettings) {
    applyTheme(nextSettings.appearance.theme);
    applyReducedMotion(nextSettings.appearance.reduced_motion);
  }

  async function saveSettings(updates: Partial<AppSettings>) {
    if (!snapshot.value) {
      return false;
    }

    operationError.value = null;
    const previousSettings = snapshot.value.settings;
    const nextSettings = { ...previousSettings, ...updates };
    snapshot.value.settings = nextSettings;

    try {
      snapshot.value.settings = await native.saveSettings(nextSettings);
      applyAppearance(snapshot.value.settings);
      return true;
    } catch (reason) {
      snapshot.value.settings = previousSettings;
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
      return false;
    }
  }

  async function loadAudioDevices() {
    operationError.value = null;

    try {
      audioDevices.value = await native.audioDevices();
    } catch (reason) {
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function resetApplicationData() {
    operationError.value = null;

    try {
      await native.resetApplicationData();
      return true;
    } catch (reason) {
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
      return false;
    }
  }

  async function openSystemAudioPermissionSettings() {
    operationError.value = null;

    try {
      await native.openSystemAudioPermissionSettings();
    } catch (reason) {
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  return {
    audioDevices,
    settings,
    applyAppearance,
    applyTheme,
    applyReducedMotion,
    saveSettings,
    resetApplicationData,
    loadAudioDevices,
    openSystemAudioPermissionSettings,
  };
}
