import type { NativeBridge } from "@/core/native/NativeBridge";
import {
  currentRecordingStatus,
  emitPostRecordingJob,
  previewState,
} from "@/core/native/preview/previewState";
import { i18n } from "@/i18n";
import type { CreateRecordingRequest } from "@/types/domain";

const { t } = i18n.global;

type RecordingBridge = Pick<
  NativeBridge,
  "createRecording" | "pauseRecording" | "recordingStatus" | "stopRecording"
>;

export const previewRecordingBridge = {
  async createRecording(request: CreateRecordingRequest) {
    const id = crypto.randomUUID();
    previewState.recordingStartedAt = Date.now();
    previewState.recordingSessionId = id;
    previewState.recordingMode = request.recordingMode;
    previewState.recordingWithoutLocalModels =
      new URLSearchParams(window.location.search).get("localModels") === "none";
    previewState.paused = false;
    previewState.pausedStartedAt = 0;
    previewState.pausedDurationMs = 0;
    return {
      schema_version: 1,
      id,
      title: request.title,
      project_id: request.projectId,
      revision: 1,
      source: "recording" as const,
      lifecycle: "recording" as const,
      recovery_state: "recoverable" as const,
      created_at: new Date().toISOString(),
      started_at: new Date().toISOString(),
      stopped_at: null,
      duration_ms: 0,
      microphone: {
        stable_id: request.microphoneDeviceId ?? "preview-microphone",
        label: t("native.defaultMicrophone"),
        sample_rate_hz: 48_000,
        channels: 1,
      },
      system_output: null,
      artifacts: [],
      pauses: [],
      gaps: [],
      language_hint: request.languageHint,
      glossary: [],
      openai_profile_id: null,
      current_transcript_id: null,
      transcript_run_ids: [],
    };
  },

  async pauseRecording(nextPaused: boolean) {
    if (nextPaused && !previewState.paused) {
      previewState.pausedStartedAt = Date.now();
    }

    if (!nextPaused && previewState.paused) {
      previewState.pausedDurationMs +=
        Date.now() - previewState.pausedStartedAt;
      previewState.pausedStartedAt = 0;
    }

    previewState.paused = nextPaused;
    return currentRecordingStatus();
  },

  async recordingStatus() {
    return currentRecordingStatus();
  },

  async stopRecording() {
    const completedSessionId = previewState.recordingSessionId;
    const completedRecordingMode = previewState.recordingMode;
    const recordingWithoutLocalModels =
      previewState.recordingWithoutLocalModels;
    previewState.recordingStartedAt = 0;
    previewState.recordingSessionId = "";
    previewState.recordingMode = "record_only";
    previewState.recordingWithoutLocalModels = false;
    previewState.paused = false;
    previewState.pausedStartedAt = 0;
    previewState.pausedDurationMs = 0;
    emitPostRecordingJob(
      completedSessionId,
      completedRecordingMode,
      recordingWithoutLocalModels,
    );
    return null;
  },
} satisfies RecordingBridge;
