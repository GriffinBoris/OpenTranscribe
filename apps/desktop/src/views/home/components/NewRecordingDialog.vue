<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Mic } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppDialog from "@/components/ui/AppDialog.vue";
import AppInputText from "@/components/ui/AppInputText.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import AppToggleSwitch from "@/components/ui/AppToggleSwitch.vue";
import type { OpenAiTranscriptionModel, RecordingMode } from "@/types/domain";
import { openAiModelOptions } from "@/views/application/openAiModels";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useOpenAiStore } from "@/views/application/openAiStore";
import type { RecordingSessionOptions } from "@/views/application/recordingStore";
import { useLocalModelsStore } from "@/views/application/localModelsStore";

const props = defineProps<{
  open: boolean;
  starting: boolean;
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
  start: [mode: RecordingMode, options: RecordingSessionOptions];
}>();

const application = useApplicationStore();
const openAi = useOpenAiStore();
const localModels = useLocalModelsStore();
const { t } = useI18n();

const title = ref("");
const projectId = ref("");
const microphoneDeviceId = ref("");
const captureSystemAudio = ref(false);
const recordingMode = ref<RecordingMode>("record_only");
const languageHint = ref("");
const openAiModel = ref<OpenAiTranscriptionModel>("gpt_transcribe");

const projectOptions = computed(() => [
  { label: t("navigation.inbox"), value: "" },
  ...application.projects.map((project) => ({
    label: project.name,
    value: project.id,
  })),
]);
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
  {
    label: t("home.recordLocal"),
    value: "local_after_recording",
    disabled: localModels.installedModels.length === 0,
  },
  {
    label: t("home.recordOpenAi"),
    value: "open_ai_live",
    disabled: !openAi.credential?.configured,
  },
]);
const languageOptions = computed(() => [
  { label: t("recordingOptions.languages.auto"), value: "" },
  { label: t("recordingOptions.languages.english"), value: "en" },
  { label: t("recordingOptions.languages.spanish"), value: "es" },
  { label: t("recordingOptions.languages.french"), value: "fr" },
  { label: t("recordingOptions.languages.german"), value: "de" },
  { label: t("recordingOptions.languages.portuguese"), value: "pt" },
  { label: t("recordingOptions.languages.italian"), value: "it" },
  { label: t("recordingOptions.languages.japanese"), value: "ja" },
  { label: t("recordingOptions.languages.korean"), value: "ko" },
  { label: t("recordingOptions.languages.chinese"), value: "zh" },
]);
const cloudModelOptions = computed(() => openAiModelOptions(t));
const canStart = computed(
  () => microphoneOptions.value.length > 0 && !props.starting,
);

function reset() {
  title.value = "";
  projectId.value = application.defaultRecordingProjectId() ?? "";
  microphoneDeviceId.value =
    application.settings?.microphone_device_id ??
    application.audioDevices?.microphones.find((device) => device.is_default)
      ?.id ??
    application.audioDevices?.microphones[0]?.id ??
    "";
  captureSystemAudio.value =
    application.settings?.capture_system_audio ?? false;
  recordingMode.value = application.settings?.recording_mode ?? "record_only";
  languageHint.value = "";
  openAiModel.value =
    application.settings?.openai_transcription_model ?? "gpt_transcribe";
}

function updateProject(value: string) {
  projectId.value = value;
  void application.saveRecordingProjectSelection(value || null);
}

function startRecording() {
  emit("start", recordingMode.value, {
    title: title.value.trim() || undefined,
    projectId: projectId.value || null,
    microphoneDeviceId: microphoneDeviceId.value || null,
    captureSystemAudio: captureSystemAudio.value,
    languageHint: languageHint.value || null,
    openAiModel: openAiModel.value,
  });
}

watch(
  () => props.open,
  (open) => {
    if (open) {
      reset();
    }
  },
);
</script>

<template>
  <AppDialog
    class="max-w-[600px]"
    :open="open"
    :title="t('recordingOptions.title')"
    @update:open="emit('update:open', $event)"
  >
    <form
      id="new-recording-form"
      class="grid grid-cols-2 gap-4 max-[600px]:grid-cols-1"
      @submit.prevent="startRecording"
    >
      <label
        class="dialog-field text-ink-muted col-span-full grid gap-2 text-sm font-semibold"
      >
        <span>{{ t("recordingOptions.meetingTitle") }}</span>
        <AppInputText
          v-model="title"
          autocomplete="off"
          :placeholder="t('recordingOptions.meetingTitlePlaceholder')"
        />
      </label>
      <label
        class="dialog-field text-ink-muted grid gap-2 text-sm font-semibold"
      >
        <span>{{ t("recordingOptions.project") }}</span>
        <AppSelect
          :model-value="projectId"
          :options="projectOptions"
          :accessible-label="t('recordingOptions.project')"
          :placeholder="t('navigation.inbox')"
          @update:model-value="updateProject"
        />
      </label>
      <label
        class="dialog-field text-ink-muted grid gap-2 text-sm font-semibold"
      >
        <span>{{ t("recordingOptions.microphone") }}</span>
        <AppSelect
          v-model="microphoneDeviceId"
          :options="microphoneOptions"
          :accessible-label="t('recordingOptions.microphone')"
          :placeholder="t('settings.recording.noMicrophone')"
        />
      </label>
      <div
        class="dialog-field text-ink-muted col-span-full flex items-center justify-between gap-4 text-sm font-semibold"
      >
        <span class="grid gap-1">
          <strong>{{ t("recordingOptions.systemOutput") }}</strong>
          <small class="text-ink-muted font-normal">{{
            t("settings.recording.systemDescription")
          }}</small>
        </span>
        <AppToggleSwitch
          v-model="captureSystemAudio"
          :accessible-label="t('recordingOptions.systemOutput')"
          :disabled="!application.audioDevices?.system_audio_available"
        />
      </div>
      <label
        class="dialog-field text-ink-muted grid gap-2 text-sm font-semibold"
      >
        <span>{{ t("recordingOptions.transcription") }}</span>
        <AppSelect
          v-model="recordingMode"
          :options="recordingModeOptions"
          :accessible-label="t('recordingOptions.transcription')"
        />
      </label>
      <label
        v-if="recordingMode === 'open_ai_live'"
        class="dialog-field text-ink-muted grid gap-2 text-sm font-semibold"
      >
        <span>{{ t("recordingOptions.openAiModel") }}</span>
        <AppSelect
          v-model="openAiModel"
          :options="cloudModelOptions"
          :accessible-label="t('recordingOptions.openAiModel')"
        />
      </label>
      <label
        class="dialog-field text-ink-muted grid gap-2 text-sm font-semibold"
      >
        <span>{{ t("recordingOptions.language") }}</span>
        <AppSelect
          v-model="languageHint"
          :options="languageOptions"
          :accessible-label="t('recordingOptions.language')"
          :placeholder="t('recordingOptions.languages.auto')"
        />
      </label>
    </form>

    <template #footer>
      <AppButton
        variant="ghost"
        :disabled="starting"
        @click="emit('update:open', false)"
      >
        {{ t("navigation.cancel") }}
      </AppButton>
      <AppButton
        type="submit"
        form="new-recording-form"
        variant="primary"
        :disabled="!canStart"
        :loading="starting"
      >
        <Mic :size="16" />{{ t("recordingOptions.start") }}
      </AppButton>
    </template>
  </AppDialog>
</template>
