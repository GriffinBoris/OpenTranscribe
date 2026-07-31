<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import AppToggleSwitch from "@/components/ui/AppToggleSwitch.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import type { RecordingMode } from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";

const application = useApplicationStore();
const { t } = useI18n();
const selectedMicrophone = ref("");
const recordingMode = ref<RecordingMode>(
  application.settings?.recording_mode ?? "record_only",
);
const captureSystemAudio = ref(
  application.settings?.capture_system_audio ?? false,
);

const microphoneOptions = computed(
  () =>
    application.audioDevices?.microphones.map((device) => ({
      label: device.is_default
        ? `${device.label} (${t("home.defaultDevice")})`
        : device.label,
      value: device.id,
    })) ?? [],
);
const recordingModeOptions = computed(() => [
  { label: t("recording.recordOnly"), value: "record_only" },
  { label: t("home.recordLocal"), value: "local_live" },
  { label: t("home.recordOpenAi"), value: "open_ai_live" },
]);
const systemStatusDescription = computed(() => {
  if (!application.audioDevices?.system_audio_available) {
    return t("settings.recording.systemUnsupported");
  }

  if (application.audioDevices.system_audio_permission_granted === true) {
    return t("settings.recording.systemPermissionGranted");
  }

  if (application.audioDevices.system_audio_permission_granted === false) {
    return t("settings.recording.systemPermission");
  }

  return t("settings.recording.systemSupportedDescription");
});
const systemStatusTone = computed(() => {
  if (!application.audioDevices?.system_audio_available) {
    return "warning";
  }

  if (application.audioDevices.system_audio_permission_granted === true) {
    return "success";
  }

  if (application.audioDevices.system_audio_permission_granted === false) {
    return "warning";
  }

  return "neutral";
});
const systemStatusLabel = computed(() => {
  if (!application.audioDevices?.system_audio_available) {
    return t("settings.recording.systemUnavailable");
  }

  return application.audioDevices.system_audio_permission_granted === false
    ? t("settings.recording.systemPermissionRequired")
    : t("settings.recording.systemReady");
});

async function updateRecordingMode(value: string) {
  recordingMode.value = value as RecordingMode;
  await application.saveSettings({ recording_mode: recordingMode.value });
}

async function updateMicrophone(value: string) {
  selectedMicrophone.value = value;
  await application.saveSettings({
    microphone_device_id: value || null,
  });
}

async function updateSystemCapture(value: boolean) {
  captureSystemAudio.value = value;
  await application.saveSettings({
    capture_system_audio: value,
  });
}

onMounted(async () => {
  await application.loadAudioDevices();
  selectedMicrophone.value =
    application.settings?.microphone_device_id ??
    application.audioDevices?.microphones.find((device) => device.is_default)
      ?.id ??
    application.audioDevices?.microphones[0]?.id ??
    "";
  captureSystemAudio.value =
    application.settings?.capture_system_audio ?? false;
});
</script>

<template>
  <AppSurface id="recording">
    <div class="section-heading">
      <div>
        <h2>{{ t("settings.navigation.recording") }}</h2>
        <p class="setting-description">
          {{ t("settings.recording.description") }}
        </p>
      </div>
    </div>
    <label class="setting-field">
      <span>{{ t("settings.recording.defaultAction") }}</span>
      <AppSelect
        :model-value="recordingMode"
        :options="recordingModeOptions"
        :accessible-label="t('settings.recording.defaultAction')"
        @update:model-value="updateRecordingMode"
      />
    </label>
    <label class="setting-field">
      <span>{{ t("settings.recording.microphone") }}</span>
      <AppSelect
        :model-value="selectedMicrophone"
        :options="microphoneOptions"
        :accessible-label="t('settings.recording.microphone')"
        :placeholder="t('settings.recording.noMicrophone')"
        @update:model-value="updateMicrophone"
      />
    </label>
    <div class="setting-field setting-toggle-row">
      <span>
        <strong>{{ t("settings.recording.systemOutput") }}</strong>
        <small>{{ t("settings.recording.systemDescription") }}</small>
      </span>
      <AppToggleSwitch
        :model-value="captureSystemAudio"
        :accessible-label="t('settings.recording.captureSystemOutput')"
        :disabled="!application.audioDevices?.system_audio_available"
        @update:model-value="updateSystemCapture"
      />
    </div>
    <div class="permission-row">
      <span>
        <strong>{{ t("settings.recording.sourceStatus") }}</strong>
        <small>{{ systemStatusDescription }}</small>
      </span>
      <div class="permission-row__actions">
        <AppButton
          v-if="
            application.audioDevices
              ?.system_audio_permission_settings_available &&
            application.audioDevices.system_audio_permission_granted === false
          "
          variant="ghost"
          size="small"
          @click="application.openSystemAudioPermissionSettings"
        >
          {{ t("settings.recording.openSystemSettings") }}
        </AppButton>
        <div class="permission-row__statuses">
          <StatusPill
            :tone="
              application.audioDevices?.microphones.length
                ? 'success'
                : 'warning'
            "
          >
            {{
              application.audioDevices?.microphones.length
                ? t("settings.recording.microphoneReady")
                : t("settings.recording.noMicrophoneStatus")
            }}
          </StatusPill>
          <StatusPill :tone="systemStatusTone">
            {{ systemStatusLabel }}
          </StatusPill>
        </div>
      </div>
    </div>
  </AppSurface>
</template>
