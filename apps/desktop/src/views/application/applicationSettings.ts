import { computed, ref, type Ref } from "vue";

import { native } from "@/core/native";
import type { AppSettings, AppSnapshot, AudioDevices } from "@/types/domain";

export function createApplicationSettings(
  snapshot: Ref<AppSnapshot | null>,
  operationError: Ref<string | null>,
) {
  const audioDevices = ref<AudioDevices | null>(null);
  const settings = computed(() => snapshot.value?.settings);
  let settingsSaveSequence = 0;

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

  async function isAudioCaptureActive() {
    const [recordingStatus, dictationStatus] = await Promise.all([
      native.recordingStatus(),
      native.dictationStatus(),
    ]);

    return (
      recordingStatus !== null ||
      ["recording", "transcribing"].includes(dictationStatus.phase)
    );
  }

  async function saveSettings(updates: Partial<AppSettings>) {
    if (!snapshot.value) {
      return false;
    }

    operationError.value = null;
    const saveSequence = ++settingsSaveSequence;
    const previousSettings = snapshot.value.settings;
    const nextSettings = { ...previousSettings, ...updates };
    snapshot.value.settings = nextSettings;

    try {
      const savedSettings = await native.saveSettings(updates);

      if (saveSequence === settingsSaveSequence) {
        snapshot.value.settings = savedSettings;
        applyAppearance(savedSettings);
      }
      return true;
    } catch (reason) {
      if (saveSequence === settingsSaveSequence) {
        snapshot.value.settings = previousSettings;
        operationError.value =
          reason instanceof Error ? reason.message : String(reason);
      }
      return false;
    }
  }

  async function loadAudioDevices() {
    operationError.value = null;

    try {
      if (await isAudioCaptureActive()) {
        return;
      }

      audioDevices.value = await native.audioDevices();
    } catch (reason) {
      if (await isAudioCaptureActive()) {
        return;
      }

      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function resetApplicationSettings() {
    operationError.value = null;

    try {
      if (!snapshot.value) {
        return false;
      }

      snapshot.value.settings = await native.resetApplicationSettings();
      applyAppearance(snapshot.value.settings);
      return true;
    } catch (reason) {
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
      return false;
    }
  }

  async function deleteAllApplicationData() {
    operationError.value = null;

    try {
      await native.deleteAllApplicationData();
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
    resetApplicationSettings,
    deleteAllApplicationData,
    loadAudioDevices,
    openSystemAudioPermissionSettings,
  };
}
