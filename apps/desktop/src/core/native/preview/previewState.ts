import { createPreviewSnapshot } from "@/core/native/preview/previewData";
import { i18n } from "@/i18n";
import type {
  AppEvent,
  CredentialStatus,
  LocalModel,
  RecordingMode,
  RecordingStatus,
  DictationStatus,
  Session,
} from "@/types/domain";

const { t } = i18n.global;

export const previewState = {
  recordingStartedAt: 0,
  recordingSessionId: "",
  recordingMode: "record_only" as RecordingMode,
  paused: false,
  pausedStartedAt: 0,
  pausedDurationMs: 0,
  globalShortcutListeners: new Map<string, () => void>(),
  dictationStatusListener: null as ((status: DictationStatus) => void) | null,
  dictationStatus: {
    id: null,
    phase: "idle",
    provider: null,
    text: null,
    error_message: null,
    elapsed_ms: 0,
    microphone_peak: 0,
    auto_pasted: false,
    approximate_cost_usd: null,
  } as DictationStatus,
  appEventListener: null as ((event: AppEvent) => void) | null,
  systemAudioPermission: null as "granted" | "required" | null,
  sessionMoves: new Map<
    string,
    { projectId: string | null; revision: number }
  >(),
  sessionTitles: new Map<string, { title: string; revision: number }>(),
  trashedSessions: new Map<string, Session>(),
  credential: {
    configured: false,
    masked_key: null,
  } as CredentialStatus,
  settings: {} as Partial<import("@/types/domain").AppSettings>,
  localModelStoragePath: "/Users/you/Documents/OpenTranscribe/models",
  localModels: [
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
  ] as LocalModel[],
};

export function currentRecordingStatus(): RecordingStatus | null {
  if (previewState.recordingStartedAt === 0) {
    return null;
  }

  const now = Date.now();

  return {
    session_id: previewState.recordingSessionId,
    is_paused: previewState.paused,
    captures_system_audio: false,
    elapsed_ms:
      now -
      previewState.recordingStartedAt -
      previewState.pausedDurationMs -
      (previewState.paused ? now - previewState.pausedStartedAt : 0),
    microphone_peak: previewState.paused ? 0 : 0.58,
    system_peak: 0,
    dropped_packets: 0,
  };
}

export function desktopOnly(message: string): never {
  throw new Error(message);
}

function setupCompleted() {
  return !new URLSearchParams(window.location.search).has("firstRun");
}

export function previewSnapshot(path?: string) {
  const snapshot = createPreviewSnapshot(path, setupCompleted());
  const searchParameters = new URLSearchParams(window.location.search);
  const recordingMode = searchParameters.get("recordingMode");
  const globalShortcut = searchParameters.get("globalShortcut");

  if (searchParameters.has("resetReady")) {
    snapshot.active_jobs = [];
  }

  if (
    recordingMode === "local_after_recording" ||
    recordingMode === "open_ai_live"
  ) {
    snapshot.settings.recording_mode = recordingMode;
  }

  const legacyGlobalShortcuts: Record<string, string> = {
    command_or_control_shift_r: "CommandOrControl+Shift+R",
    command_or_control_shift_space: "CommandOrControl+Shift+Space",
    alt_shift_r: "Alt+Shift+R",
  };
  const configuredGlobalShortcut = globalShortcut
    ? (legacyGlobalShortcuts[globalShortcut] ?? globalShortcut)
    : null;

  if (configuredGlobalShortcut) {
    snapshot.settings.global_shortcut_enabled = true;
    snapshot.settings.global_shortcut = configuredGlobalShortcut;
  }

  snapshot.settings = { ...snapshot.settings, ...previewState.settings };

  if (searchParameters.has("recovery")) {
    snapshot.recent_sessions[0] = {
      ...snapshot.recent_sessions[0],
      lifecycle: "recovered",
      recovery_state: "recoverable",
      duration_ms: 0,
    };
  }

  snapshot.recent_sessions = snapshot.recent_sessions
    .map((session) => {
      const moved = previewState.sessionMoves.get(session.id);
      const renamed = previewState.sessionTitles.get(session.id);

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
    })
    .filter((session) => !previewState.trashedSessions.has(session.id));

  return snapshot;
}

export function emitPostRecordingJob(sessionId: string, mode: RecordingMode) {
  if (mode === "record_only") {
    return;
  }

  if (
    mode === "local_after_recording" &&
    !previewState.localModels.some((model) => model.installed)
  ) {
    previewState.appEventListener?.({
      type: "attention_required",
      payload: t("shell.installLocalModel"),
    });
    return;
  }

  if (mode === "open_ai_live" && !previewState.credential.configured) {
    previewState.appEventListener?.({
      type: "attention_required",
      payload: t("native.addApiKey"),
    });
    return;
  }

  const timestamp = new Date().toISOString();
  previewState.appEventListener?.({
    type: "job_state_changed",
    payload: {
      id: crypto.randomUUID(),
      session_id: sessionId,
      kind: mode === "open_ai_live" ? "transcribe_open_ai" : "transcribe_local",
      state: "queued",
      progress: null,
      estimated_cost_usd: null,
      attempt: 1,
      error_message: null,
      created_at: timestamp,
      updated_at: timestamp,
    },
  });
}
