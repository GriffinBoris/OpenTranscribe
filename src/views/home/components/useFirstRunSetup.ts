import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import { native } from "@/core/native";
import type {
  RecordingMode,
  Session,
  SessionAudioSource,
} from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useOpenAiStore } from "@/views/application/openAiStore";
import { useRecordingStore } from "@/views/application/recordingStore";
import { useLocalModelsStore } from "@/views/models/localModelsStore";

type SourceTestPhase = "configure" | "running" | "stopping" | "review";

const SOURCE_TEST_DURATION_MS = 10_000;

export function useFirstRunSetup() {
  const application = useApplicationStore();
  const openAi = useOpenAiStore();
  const recording = useRecordingStore();
  const localModels = useLocalModelsStore();
  const router = useRouter();
  const { t } = useI18n();

  const phase = ref<SourceTestPhase>("configure");
  const testSessionId = ref<string | null>(null);
  const audioSources = ref<SessionAudioSource[]>([]);
  const waveform = ref<number[]>([]);
  const sourceTestError = ref<string | null>(null);
  const isChoosingLibrary = ref(false);
  const isFinishing = ref(false);
  const selectedMicrophone = ref(
    application.settings?.microphone_device_id ?? "",
  );
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
  const systemPermissionDescription = computed(() => {
    if (!application.audioDevices?.system_audio_available) {
      return t("settings.recording.systemUnsupported");
    }

    if (application.audioDevices.system_audio_permission_granted === true) {
      return t("firstRun.systemPermissionGranted");
    }

    if (application.audioDevices.system_audio_permission_granted === false) {
      return t("firstRun.systemPermissionRequired");
    }

    return t("settings.recording.systemSupportedDescription");
  });
  const recordingModeOptions = computed(() => [
    { label: t("firstRun.decideLater"), value: "record_only" },
    { label: t("firstRun.transcribeLocal"), value: "local_live" },
    { label: t("firstRun.transcribeOpenAi"), value: "open_ai_live" },
  ]);
  const providerReady = computed(() => {
    if (recordingMode.value === "local_live") {
      return localModels.installedModels.length > 0;
    }

    if (recordingMode.value === "open_ai_live") {
      return Boolean(openAi.credential?.configured);
    }

    return true;
  });
  const sourceTestProgress = computed(() =>
    Math.min(
      Math.round(
        ((recording.status?.elapsed_ms ?? 0) / SOURCE_TEST_DURATION_MS) * 100,
      ),
      100,
    ),
  );
  const secondsRemaining = computed(() =>
    Math.max(
      0,
      Math.ceil(
        (SOURCE_TEST_DURATION_MS - (recording.status?.elapsed_ms ?? 0)) / 1_000,
      ),
    ),
  );
  const canStartTest = computed(
    () =>
      Boolean(application.snapshot?.library) &&
      microphoneOptions.value.length > 0 &&
      providerReady.value &&
      !recording.activeRecording,
  );

  async function chooseLibrary() {
    isChoosingLibrary.value = true;
    application.operationError = null;

    try {
      await application.chooseLibrary();
    } catch (reason) {
      application.operationError =
        reason instanceof Error ? reason.message : String(reason);
    } finally {
      isChoosingLibrary.value = false;
    }
  }

  async function updateMicrophone(value: string) {
    selectedMicrophone.value = value;
    await application.saveSettings({ microphone_device_id: value || null });
  }

  async function updateSystemCapture(value: boolean) {
    captureSystemAudio.value = value;
    await application.saveSettings({ capture_system_audio: value });
  }

  async function updateRecordingMode(value: string) {
    recordingMode.value = value as RecordingMode;
    await application.saveSettings({ recording_mode: recordingMode.value });
  }

  async function startSourceTest() {
    sourceTestError.value = null;
    const session = await recording.createSession("record_only", {
      title: t("firstRun.sourceTestTitle"),
    });

    if (!session) {
      return;
    }

    testSessionId.value = session.id;
    phase.value = "running";
  }

  async function stopSourceTest() {
    if (phase.value !== "running") {
      return;
    }

    phase.value = "stopping";

    try {
      await prepareReview(await recording.stop());
    } catch (reason) {
      sourceTestError.value =
        reason instanceof Error ? reason.message : String(reason);
      phase.value = "configure";
    }
  }

  async function prepareReview(session: Session | null) {
    const sessionId = session?.id ?? testSessionId.value;

    if (!sessionId) {
      phase.value = "configure";
      return;
    }

    testSessionId.value = sessionId;

    try {
      audioSources.value = await native.sessionAudioSources(sessionId);
      waveform.value = await native.sessionWaveform(sessionId).catch(() => []);
    } catch (reason) {
      sourceTestError.value =
        reason instanceof Error ? reason.message : String(reason);
    }

    phase.value = "review";
  }

  async function finishSetup() {
    isFinishing.value = true;
    sourceTestError.value = null;

    try {
      if (!(await application.saveSettings({ setup_completed: true }))) {
        sourceTestError.value = application.operationError;
      }
    } catch (reason) {
      sourceTestError.value =
        reason instanceof Error ? reason.message : String(reason);
    } finally {
      isFinishing.value = false;
    }
  }

  async function openTestSession() {
    if (testSessionId.value) {
      await router.push(`/sessions/${testSessionId.value}`);
    }
  }

  async function revealTestSession() {
    if (!testSessionId.value) {
      return;
    }

    try {
      await native.revealSession(testSessionId.value);
    } catch (reason) {
      sourceTestError.value =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function openSettings(section: "models" | "openai") {
    await router.push(`/settings#${section}`);
  }

  watch(
    () => recording.status?.elapsed_ms,
    (elapsedMs) => {
      if (
        phase.value === "running" &&
        (elapsedMs ?? 0) >= SOURCE_TEST_DURATION_MS
      ) {
        void stopSourceTest();
      }
    },
  );

  watch(
    () => recording.activeRecording?.id,
    (activeSessionId) => {
      if (
        phase.value === "running" &&
        testSessionId.value &&
        !activeSessionId
      ) {
        phase.value = "stopping";
        const session =
          application.recentSessions.find(
            (candidate) => candidate.id === testSessionId.value,
          ) ?? null;
        void prepareReview(session);
      }
    },
  );

  watch(
    () => application.audioDevices,
    (devices) => {
      if (selectedMicrophone.value || !devices) {
        return;
      }

      selectedMicrophone.value =
        devices.microphones.find((device) => device.is_default)?.id ??
        devices.microphones[0]?.id ??
        "";
    },
    { immediate: true },
  );

  return {
    application,
    phase,
    audioSources,
    waveform,
    sourceTestError,
    isChoosingLibrary,
    isFinishing,
    selectedMicrophone,
    recordingMode,
    captureSystemAudio,
    microphoneOptions,
    systemPermissionDescription,
    recordingModeOptions,
    providerReady,
    sourceTestProgress,
    secondsRemaining,
    canStartTest,
    chooseLibrary,
    updateMicrophone,
    updateSystemCapture,
    updateRecordingMode,
    startSourceTest,
    finishSetup,
    openTestSession,
    revealTestSession,
    openSettings,
  };
}
