import { ref } from "vue";
import { defineStore } from "pinia";

import { native } from "@/core/native";
import { i18n } from "@/i18n";
import type {
  AppEvent,
  RecordingMode,
  RecordingStatus,
  Session,
} from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";

export interface RecordingSessionOptions {
  title?: string;
  projectId?: string | null;
  microphoneDeviceId?: string | null;
  captureSystemAudio?: boolean;
  languageHint?: string | null;
}

export const useRecordingStore = defineStore("recording", () => {
  const { t } = i18n.global;
  const activeRecording = ref<Session | null>(null);
  const activeMode = ref<RecordingMode>("record_only");
  const status = ref<RecordingStatus | null>(null);

  function handleEvent(event: AppEvent) {
    if (event.type === "recording_levels") {
      if (event.payload.session_id !== activeRecording.value?.id) {
        return;
      }

      status.value = event.payload;
      return;
    }

    if (event.type !== "recording_state_changed") {
      return;
    }

    if (activeRecording.value) {
      activeRecording.value.lifecycle = event.payload;
    }

    if (event.payload === "ready" || event.payload === "needs_attention") {
      clear();
    }
  }

  async function createSession(
    mode: RecordingMode = "record_only",
    options: RecordingSessionOptions = {},
  ): Promise<Session | null> {
    const application = useApplicationStore();
    application.operationError = null;

    try {
      if (!application.snapshot?.library) {
        await application.chooseLibrary();
      }

      if (!application.snapshot?.library) {
        throw new Error(t("errors.chooseLibraryBeforeRecording"));
      }

      const session = await native.createRecording({
        title:
          options.title ??
          t("errors.untitledRecording", {
            date: new Intl.DateTimeFormat(undefined, {
              dateStyle: "medium",
              timeStyle: "short",
            }).format(new Date()),
          }),
        projectId: options.projectId ?? null,
        microphoneDeviceId:
          options.microphoneDeviceId ??
          application.settings?.microphone_device_id ??
          null,
        captureSystemAudio:
          options.captureSystemAudio ??
          application.settings?.capture_system_audio ??
          false,
        languageHint: options.languageHint ?? null,
        recordingMode: mode,
      });
      activeRecording.value = session;
      activeMode.value = mode;
      application.snapshot.recent_sessions.unshift(session);
      await refreshStatus();
      return session;
    } catch (reason) {
      application.operationError =
        reason instanceof Error ? reason.message : String(reason);
      return null;
    }
  }

  async function togglePaused() {
    if (!status.value) {
      return;
    }

    const application = useApplicationStore();
    application.operationError = null;

    try {
      status.value = await native.pauseRecording(!status.value.is_paused);
      if (activeRecording.value) {
        activeRecording.value.lifecycle = status.value?.is_paused
          ? "paused"
          : "recording";
      }
    } catch (reason) {
      application.operationError =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function refreshStatus() {
    status.value = await native.recordingStatus();
  }

  async function stop() {
    const application = useApplicationStore();

    if (!activeRecording.value) {
      return null;
    }

    activeRecording.value.lifecycle = "finalizing";
    let stoppedSession: Session | null;

    try {
      stoppedSession = await native.stopRecording();
    } catch (reason) {
      status.value = await native.recordingStatus();

      if (status.value && activeRecording.value) {
        activeRecording.value.lifecycle = status.value.is_paused
          ? "paused"
          : "recording";
      } else {
        clear();
        await application.refreshLibrary();
      }

      throw reason;
    }

    const session = stoppedSession ?? {
      ...activeRecording.value,
      lifecycle: "ready" as const,
      stopped_at: new Date().toISOString(),
      duration_ms: status.value?.elapsed_ms ?? 0,
    };
    const index = application.snapshot?.recent_sessions.findIndex(
      (candidate) => candidate.id === session.id,
    );

    if (application.snapshot && index !== undefined && index >= 0) {
      application.snapshot.recent_sessions[index] = session;
    }

    clear();
    return session;
  }

  function clear() {
    activeRecording.value = null;
    status.value = null;
    activeMode.value = "record_only";
  }

  return {
    activeRecording,
    activeMode,
    status,
    handleEvent,
    createSession,
    togglePaused,
    refreshStatus,
    stop,
  };
});
