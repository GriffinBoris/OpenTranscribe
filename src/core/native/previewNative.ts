import type { NativeBridge } from "@/core/native/NativeBridge";
import { createPreviewSnapshot } from "@/core/previewData";
import { i18n } from "@/i18n";
import type {
  AppEvent,
  AppSettings,
  AudioDevices,
  CreateRecordingRequest,
  CredentialStatus,
  ExportFormat,
  Job,
  JobProgress,
  LocalModel,
  RecordingMode,
  RecordingStatus,
  SearchFilters,
  Session,
} from "@/types/domain";

const { t } = i18n.global;
let recordingStartedAt = 0;
let recordingSessionId = "";
let recordingMode: RecordingMode = "record_only";
let paused = false;
let pausedStartedAt = 0;
let pausedDurationMs = 0;
let previewGlobalShortcutListener: (() => void) | null = null;
let previewAppEventListener: ((event: AppEvent) => void) | null = null;
let previewSystemAudioPermission: "granted" | "required" | null = null;
const previewSessionMoves = new Map<
  string,
  { projectId: string | null; revision: number }
>();
const previewSessionTitles = new Map<
  string,
  { title: string; revision: number }
>();
const previewTrashedSessions = new Map<string, Session>();
let credential: CredentialStatus = {
  configured: false,
  masked_key: null,
};
const localModels: LocalModel[] = [
  {
    id: "whisper-small-q5_1",
    preset: "fast",
    label: t("models.fast"),
    description: t("models.fastDescription"),
    byte_count: 190_085_487,
    installed: false,
  },
  {
    id: "whisper-medium-q5_0",
    preset: "balanced",
    label: t("models.balanced"),
    description: t("models.balancedDescription"),
    byte_count: 539_212_467,
    installed: false,
  },
  {
    id: "whisper-large-v3-turbo-q5_0",
    preset: "best",
    label: t("models.best"),
    description: t("models.bestDescription"),
    byte_count: 574_041_195,
    installed: false,
  },
];

function recordingStatus(): RecordingStatus | null {
  if (recordingStartedAt === 0) {
    return null;
  }

  const now = Date.now();

  return {
    session_id: recordingSessionId,
    is_paused: paused,
    captures_system_audio: false,
    elapsed_ms:
      now -
      recordingStartedAt -
      pausedDurationMs -
      (paused ? now - pausedStartedAt : 0),
    microphone_peak: paused ? 0 : 0.58,
    system_peak: 0,
    dropped_packets: 0,
  };
}

function desktopOnly(message: string): never {
  throw new Error(message);
}

function setupCompleted() {
  return !new URLSearchParams(window.location.search).has("firstRun");
}

function emitPostRecordingJob(sessionId: string, mode: RecordingMode) {
  if (mode === "record_only") {
    return;
  }

  if (mode === "local_live" && !localModels.some((model) => model.installed)) {
    previewAppEventListener?.({
      type: "attention_required",
      payload: t("shell.installLocalModel"),
    });
    return;
  }

  if (mode === "open_ai_live" && !credential.configured) {
    previewAppEventListener?.({
      type: "attention_required",
      payload: t("native.addApiKey"),
    });
    return;
  }

  const timestamp = new Date().toISOString();
  previewAppEventListener?.({
    type: "job_state_changed",
    payload: {
      id: crypto.randomUUID(),
      session_id: sessionId,
      kind: mode === "open_ai_live" ? "transcribe_open_ai" : "transcribe_local",
      state: "queued",
      progress: null,
      attempt: 1,
      error_message: null,
      created_at: timestamp,
      updated_at: timestamp,
    },
  });
}

function previewSnapshot(path?: string) {
  const snapshot = createPreviewSnapshot(path, setupCompleted());
  const recordingMode = new URLSearchParams(window.location.search).get(
    "recordingMode",
  );

  if (recordingMode === "local_live" || recordingMode === "open_ai_live") {
    snapshot.settings.recording_mode = recordingMode;
  }

  if (new URLSearchParams(window.location.search).has("recovery")) {
    snapshot.recent_sessions[0] = {
      ...snapshot.recent_sessions[0],
      lifecycle: "recovered",
      recovery_state: "recoverable",
      duration_ms: 0,
    };
  }

  snapshot.recent_sessions = snapshot.recent_sessions.map((session) => {
    const moved = previewSessionMoves.get(session.id);
    const renamed = previewSessionTitles.get(session.id);

    if (!moved && !renamed) {
      return session;
    }

    return {
      ...session,
      ...(moved ? { project_id: moved.projectId } : {}),
      ...(renamed ? { title: renamed.title } : {}),
      revision: Math.max(
        session.revision,
        moved?.revision ?? 0,
        renamed?.revision ?? 0,
      ),
    };
  });
  snapshot.recent_sessions = snapshot.recent_sessions.filter(
    (session) => !previewTrashedSessions.has(session.id),
  );

  return snapshot;
}

export const previewNative: NativeBridge = {
  async bootstrap() {
    return previewSnapshot();
  },

  async saveSettings(settings: AppSettings) {
    return { ...settings, revision: settings.revision + 1 };
  },

  async initializeLibrary(path: string) {
    return previewSnapshot(path);
  },

  async chooseLibrary() {
    return "/Users/you/Documents/OpenTranscribe";
  },

  async createProject(name: string) {
    const timestamp = new Date().toISOString();
    return {
      schema_version: 1,
      id: crypto.randomUUID(),
      name,
      revision: 1,
      glossary: [],
      openai_profile_id: null,
      language_hint: null,
      created_at: timestamp,
      updated_at: timestamp,
    };
  },

  async searchLibrary(query: string, filters: SearchFilters) {
    const normalizedQuery = query.trim().toLocaleLowerCase();
    const sessions = previewSnapshot().recent_sessions;

    return {
      results: sessions
        .filter(
          (session) =>
            session.title.toLocaleLowerCase().includes(normalizedQuery) &&
            (!filters.project_id ||
              session.project_id === filters.project_id) &&
            (!filters.content_kinds.length ||
              filters.content_kinds.includes("session")),
        )
        .map((session) => ({
          session_id: session.id,
          session_title: session.title,
          kind: "session" as const,
          excerpt: session.title,
          timestamp_ms: null,
        })),
      next_cursor: null,
    };
  },

  async createRecording(request: CreateRecordingRequest) {
    const id = crypto.randomUUID();
    recordingStartedAt = Date.now();
    recordingSessionId = id;
    recordingMode = request.recordingMode;
    paused = false;
    pausedStartedAt = 0;
    pausedDurationMs = 0;
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
      current_transcript_id: null,
    };
  },

  async importMedia() {
    return null;
  },

  async audioDevices(): Promise<AudioDevices> {
    const requestedPermission = new URLSearchParams(window.location.search).get(
      "systemAudioPermission",
    );
    if (
      requestedPermission === "granted" ||
      requestedPermission === "required"
    ) {
      previewSystemAudioPermission = requestedPermission;
    }

    const permissionState = previewSystemAudioPermission;
    const permissionSettingsAvailable = permissionState !== null;

    return {
      microphones: [
        {
          id: "preview-microphone",
          label: t("native.defaultMicrophone"),
          is_default: true,
        },
      ],
      system_audio_available: permissionSettingsAvailable,
      system_audio_permission_granted:
        permissionState === "granted"
          ? true
          : permissionState === "required"
            ? false
            : null,
      system_audio_permission_settings_available: permissionSettingsAvailable,
    };
  },

  async openSystemAudioPermissionSettings() {
    return;
  },

  async configureGlobalShortcut(shortcut, onTrigger) {
    if (previewGlobalShortcutListener) {
      window.removeEventListener(
        "opentranscribe:preview-global-shortcut",
        previewGlobalShortcutListener,
      );
      previewGlobalShortcutListener = null;
    }

    if (!shortcut) {
      return;
    }

    if (
      new URLSearchParams(window.location.search).has("globalShortcutError")
    ) {
      throw new Error(t("native.previewGlobalShortcutUnavailable"));
    }

    previewGlobalShortcutListener = onTrigger;
    window.addEventListener(
      "opentranscribe:preview-global-shortcut",
      previewGlobalShortcutListener,
    );
  },

  async pauseRecording(nextPaused: boolean) {
    if (nextPaused && !paused) {
      pausedStartedAt = Date.now();
    }

    if (!nextPaused && paused) {
      pausedDurationMs += Date.now() - pausedStartedAt;
      pausedStartedAt = 0;
    }

    paused = nextPaused;
    return recordingStatus();
  },

  async recordingStatus() {
    return recordingStatus();
  },

  async stopRecording() {
    const completedSessionId = recordingSessionId;
    const completedRecordingMode = recordingMode;
    recordingStartedAt = 0;
    recordingSessionId = "";
    recordingMode = "record_only";
    paused = false;
    pausedStartedAt = 0;
    pausedDurationMs = 0;
    emitPostRecordingJob(completedSessionId, completedRecordingMode);
    return null;
  },

  async openAiCredentialStatus() {
    return credential;
  },

  async saveOpenAiApiKey(apiKey: string) {
    credential = {
      configured: true,
      masked_key: `•••• ${apiKey.slice(-4)}`,
    };
    return credential;
  },

  async removeOpenAiApiKey() {
    credential = { configured: false, masked_key: null };
    return credential;
  },

  async testOpenAiConnection() {
    return {
      connected: credential.configured,
      message: credential.configured
        ? t("native.previewConnected")
        : t("native.addApiKey"),
    };
  },

  async enqueueTranscription(
    sessionId: string,
    provider: "local" | "open_ai",
  ): Promise<Job> {
    const timestamp = new Date().toISOString();
    return {
      id: crypto.randomUUID(),
      session_id: sessionId,
      kind: provider === "open_ai" ? "transcribe_open_ai" : "transcribe_local",
      state: "queued",
      progress: null,
      attempt: 1,
      error_message: null,
      created_at: timestamp,
      updated_at: timestamp,
    };
  },

  async retryJob() {
    return desktopOnly(t("native.localDesktopOnly"));
  },

  async cancelJob() {
    return desktopOnly(t("native.localDesktopOnly"));
  },

  async updateTranscriptSegment() {
    return desktopOnly(t("native.editingDesktopOnly"));
  },

  async renameSpeaker() {
    return desktopOnly(t("native.editingDesktopOnly"));
  },

  async mergeSpeakers() {
    return desktopOnly(t("native.editingDesktopOnly"));
  },

  async localModelStatuses() {
    return structuredClone(localModels);
  },

  async downloadLocalModel(
    modelId: string,
    onProgress: (progress: JobProgress) => void,
  ) {
    const model = localModels.find((item) => item.id === modelId);
    if (!model) {
      return desktopOnly(t("native.unknownLocalModel"));
    }

    model.installed = true;
    onProgress({
      stage: "downloading",
      completed_units: model.byte_count,
      total_units: model.byte_count,
      unit: "bytes",
      message: t("native.downloadComplete"),
    });
    return structuredClone(model);
  },

  async removeLocalModel(modelId: string) {
    const model = localModels.find((item) => item.id === modelId);
    if (!model) {
      return desktopOnly(t("native.unknownLocalModel"));
    }

    model.installed = false;
    return structuredClone(model);
  },

  async exportSession(_sessionId: string, format: ExportFormat) {
    return {
      format,
      path: t("native.previewExportPath", {
        extension: format === "markdown" ? "md" : format,
      }),
    };
  },

  async sessionWorkspace(sessionId: string) {
    if (previewTrashedSessions.has(sessionId)) {
      throw new Error(t("native.requestedItemMissing"));
    }

    return null;
  },

  async sessionAudioSources() {
    return [];
  },

  async sessionWaveform() {
    return [];
  },

  async recoverRecording(sessionId: string) {
    const session = createPreviewSnapshot().recent_sessions.find(
      (candidate) => candidate.id === sessionId,
    );

    if (!session) {
      return desktopOnly(t("native.editingDesktopOnly"));
    }

    return {
      ...session,
      lifecycle: "recovered" as const,
      recovery_state: "recovered" as const,
    };
  },

  async revealSession() {},

  async renameSession(sessionId: string, title: string) {
    const session = previewSnapshot().recent_sessions.find(
      (candidate) => candidate.id === sessionId,
    );

    if (!session) {
      return desktopOnly(t("native.editingDesktopOnly"));
    }

    const normalizedTitle = title.trim();

    if (!normalizedTitle) {
      throw new Error("meeting title cannot be empty");
    }

    const renamed = {
      ...session,
      title: normalizedTitle,
      revision: session.revision + 1,
    };
    previewSessionTitles.set(sessionId, {
      title: renamed.title,
      revision: renamed.revision,
    });
    previewAppEventListener?.({ type: "library_changed" });
    return renamed;
  },

  async moveSession(sessionId: string, projectId: string | null) {
    const session = previewSnapshot().recent_sessions.find(
      (candidate) => candidate.id === sessionId,
    );

    if (!session) {
      return desktopOnly(t("native.editingDesktopOnly"));
    }

    const moved = {
      ...session,
      project_id: projectId,
      revision: session.revision + 1,
    };
    previewSessionMoves.set(sessionId, {
      projectId,
      revision: moved.revision,
    });
    previewAppEventListener?.({ type: "library_changed" });
    return moved;
  },

  async trashedSessions() {
    return [...previewTrashedSessions.values()];
  },

  async trashSession(sessionId: string) {
    const session = previewSnapshot().recent_sessions.find(
      (candidate) => candidate.id === sessionId,
    );

    if (!session) {
      throw new Error(t("native.requestedItemMissing"));
    }

    const trashed = {
      ...session,
      lifecycle: "trashed" as const,
      revision: session.revision + 1,
    };
    previewTrashedSessions.set(sessionId, trashed);
    previewAppEventListener?.({ type: "library_changed" });
    return trashed;
  },

  async restoreSession(sessionId: string) {
    const session = previewTrashedSessions.get(sessionId);

    if (!session) {
      throw new Error(t("native.requestedItemMissing"));
    }

    const restored = {
      ...session,
      lifecycle: "ready" as const,
      revision: session.revision + 1,
    };
    previewTrashedSessions.delete(sessionId);
    previewSessionMoves.set(sessionId, {
      projectId: restored.project_id,
      revision: restored.revision,
    });
    previewAppEventListener?.({ type: "library_changed" });
    return restored;
  },

  async saveNotes() {
    return null;
  },

  async subscribe(onEvent: (event: AppEvent) => void) {
    previewAppEventListener = onEvent;
    window.setInterval(() => {
      const status = recordingStatus();
      if (!status) {
        return;
      }

      onEvent({ type: "recording_levels", payload: status });
    }, 50);
  },
};
