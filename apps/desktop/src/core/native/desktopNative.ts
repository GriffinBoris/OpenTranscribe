import { Channel, convertFileSrc, invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { register, unregister } from "@tauri-apps/plugin-global-shortcut";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";

import type { NativeBridge } from "@/core/native/NativeBridge";
import { i18n } from "@/i18n";
import type {
  AppEvent,
  AppUpdate,
  AppSettings,
  AppSnapshot,
  AudioDevices,
  ConnectionTestResult,
  CreateRecordingRequest,
  CredentialStatus,
  DictationHistoryEntry,
  DictationStatus,
  ExportFormat,
  ExportResult,
  Job,
  JobProgress,
  LocalModel,
  Project,
  RecordingStatus,
  SavedDocument,
  Session,
  SessionAudioSource,
  SessionWorkspace,
  SearchFilters,
  SearchPage,
  Transcript,
  UpdateDownloadProgress,
} from "@/types/domain";

const { t } = i18n.global;
const registeredGlobalShortcuts = new Map<"recording" | "dictation", string>();
let pendingUpdate: Update | null = null;

const exportExtensions: Record<ExportFormat, string> = {
  markdown: "md",
  text: "txt",
  json: "json",
  srt: "srt",
  vtt: "vtt",
};

export const desktopNative: NativeBridge = {
  bootstrap: () => invoke<AppSnapshot>("bootstrap"),

  saveSettings: (updates: Partial<AppSettings>) =>
    invoke<AppSettings>("save_settings", { request: { updates } }),

  resetApplicationSettings: () =>
    invoke<AppSettings>("reset_application_settings"),

  deleteAllApplicationData: () => invoke("delete_all_application_data"),

  updaterConfigured: () => invoke<boolean>("updater_configured"),

  async checkForUpdate(): Promise<AppUpdate | null> {
    if (pendingUpdate) {
      await pendingUpdate.close();
    }

    pendingUpdate = await check();

    return pendingUpdate
      ? {
          version: pendingUpdate.version,
          notes: pendingUpdate.body ?? null,
        }
      : null;
  },

  async installUpdate(onProgress) {
    if (!pendingUpdate) {
      throw new Error("No update is ready to install.");
    }

    let completedBytes = 0;
    let totalBytes: number | null = null;

    await pendingUpdate.downloadAndInstall((event) => {
      if (event.event === "Started") {
        totalBytes = event.data.contentLength ?? null;
      }

      if (event.event === "Progress") {
        completedBytes += event.data.chunkLength;
      }

      onProgress({
        completed_bytes: completedBytes,
        total_bytes: totalBytes,
      } satisfies UpdateDownloadProgress);
    });

    pendingUpdate = null;
    await relaunch();
  },

  initializeLibrary: (path: string) =>
    invoke<AppSnapshot>("initialize_library", { path }),

  async chooseLibrary() {
    return open({
      directory: true,
      multiple: false,
      title: t("native.chooseLibrary"),
    });
  },

  async chooseLocalModelStorage() {
    return open({
      directory: true,
      multiple: false,
      title: t("native.chooseLocalModelStorage"),
    });
  },

  createProject: (name: string) =>
    invoke<Project>("create_project", { request: { name } }),

  searchLibrary: (query: string, filters: SearchFilters) =>
    invoke<SearchPage>("search_library", { request: { query, filters } }),

  createRecording(request: CreateRecordingRequest) {
    return invoke<Session>("create_recording", {
      request: {
        title: request.title,
        project_id: request.projectId,
        microphone_device_id: request.microphoneDeviceId,
        capture_system_audio: request.captureSystemAudio,
        language_hint: request.languageHint,
        recording_mode: request.recordingMode,
        openai_model: request.openAiModel,
      },
    });
  },

  async importMedia(path?: string) {
    const selected =
      path ??
      (await open({
        multiple: false,
        title: t("native.importMedia"),
        filters: [
          {
            name: t("native.audioAndVideo"),
            extensions: ["mp3", "mp4", "mpeg", "mpga", "m4a", "wav", "webm"],
          },
        ],
      }));

    return selected
      ? invoke<Session>("import_media", { path: selected })
      : null;
  },

  audioDevices: () => invoke<AudioDevices>("audio_devices"),

  openSystemAudioPermissionSettings: () =>
    invoke("open_system_audio_permission_settings"),

  async configureGlobalShortcut(shortcutId, shortcut, onTrigger) {
    const registeredShortcut = registeredGlobalShortcuts.get(shortcutId);

    if (!shortcut) {
      if (registeredShortcut) {
        await unregister(registeredShortcut);
        registeredGlobalShortcuts.delete(shortcutId);
      }

      return;
    }

    if (shortcut === registeredShortcut) {
      return;
    }

    await register(shortcut, (event) => {
      if (event.state === "Pressed") {
        onTrigger();
      }
    });

    if (registeredShortcut) {
      try {
        await unregister(registeredShortcut);
      } catch (reason) {
        await unregister(shortcut);
        throw reason;
      }
    }

    registeredGlobalShortcuts.set(shortcutId, shortcut);
  },

  toggleDictation: () => invoke<DictationStatus>("toggle_dictation"),

  dictationStatus: () => invoke<DictationStatus>("dictation_status"),

  dictationShortcut: () => invoke<string>("dictation_shortcut"),

  dictationHistory: () => invoke<DictationHistoryEntry[]>("dictation_history"),

  clearDictationHistory: () => invoke<void>("clear_dictation_history"),

  cancelDictation: () => invoke<DictationStatus>("cancel_dictation"),

  dismissDictation: () => invoke("dismiss_dictation"),

  subscribeDictationStatus(onStatus) {
    const channel = new Channel<DictationStatus>();
    channel.onmessage = onStatus;
    return invoke("subscribe_dictation_status", { channel });
  },

  pauseRecording: (paused: boolean) =>
    invoke<RecordingStatus>("pause_recording", { paused }),

  recordingStatus: () => invoke<RecordingStatus | null>("recording_status"),

  stopRecording: () => invoke<Session>("stop_recording"),

  openAiCredentialStatus: () =>
    invoke<CredentialStatus>("openai_credential_status"),

  saveOpenAiApiKey: (apiKey: string) =>
    invoke<CredentialStatus>("save_openai_api_key", {
      request: { api_key: apiKey },
    }),

  removeOpenAiApiKey: () => invoke<CredentialStatus>("remove_openai_api_key"),

  testOpenAiConnection: () =>
    invoke<ConnectionTestResult>("test_openai_connection"),

  enqueueTranscription(
    sessionId: string,
    provider: "local" | "open_ai",
    modelId: string,
  ) {
    return invoke<Job>("enqueue_transcription", {
      request: {
        session_id: sessionId,
        provider,
        model_id: modelId,
      },
    });
  },

  retryJob: (jobId: string) => invoke<Job>("retry_job", { jobId }),

  cancelJob: (jobId: string) => invoke<Job>("cancel_job", { jobId }),

  updateTranscriptSegment(
    sessionId: string,
    segmentId: string,
    text: string,
    expectedRevision: number,
  ) {
    return invoke<Transcript>("update_transcript_segment", {
      request: {
        session_id: sessionId,
        segment_id: segmentId,
        text,
        expected_revision: expectedRevision,
      },
    });
  },

  renameSpeaker(
    sessionId: string,
    speakerId: string,
    displayName: string,
    expectedRevision: number,
  ) {
    return invoke<Transcript>("rename_speaker", {
      request: {
        session_id: sessionId,
        speaker_id: speakerId,
        display_name: displayName,
        expected_revision: expectedRevision,
      },
    });
  },

  mergeSpeakers(
    sessionId: string,
    sourceSpeakerId: string,
    targetSpeakerId: string,
    expectedRevision: number,
  ) {
    return invoke<Transcript>("merge_speakers", {
      request: {
        session_id: sessionId,
        source_speaker_id: sourceSpeakerId,
        target_speaker_id: targetSpeakerId,
        expected_revision: expectedRevision,
      },
    });
  },

  localModelStatuses: () => invoke<LocalModel[]>("local_model_statuses"),

  localModelStoragePath: () => invoke<string>("local_model_storage_path"),

  moveLocalModels: (path: string) =>
    invoke<string>("move_local_models", { request: { path } }),

  downloadLocalModel(
    modelId: string,
    onProgress: (progress: JobProgress) => void,
  ) {
    const progressChannel = new Channel<JobProgress>();
    progressChannel.onmessage = onProgress;
    return invoke<LocalModel>("download_local_model", {
      modelId,
      progressChannel,
    });
  },

  removeLocalModel: (modelId: string) =>
    invoke<LocalModel>("remove_local_model", { modelId }),

  async exportSession(
    sessionId: string,
    format: ExportFormat,
    sessionTitle: string,
  ) {
    const extension = exportExtensions[format];
    const fileName =
      sessionTitle
        .replace(/[<>:"/\\|?*]/g, "-")
        .trim()
        .slice(0, 80) || t("session.untitledRecording");
    const destinationPath = await save({
      defaultPath: `${fileName}.${extension}`,
      filters: [
        {
          name: t("native.transcriptFile"),
          extensions: [extension],
        },
      ],
      title: t("native.exportSession"),
    });

    return destinationPath
      ? invoke<ExportResult>("export_session", {
          request: {
            session_id: sessionId,
            format,
            destination_path: destinationPath,
          },
        })
      : null;
  },

  sessionWorkspace: (sessionId: string) =>
    invoke<SessionWorkspace>("session_workspace", { sessionId }),

  async sessionAudioSources(sessionId: string) {
    const sources = await invoke<
      Array<{
        kind: SessionAudioSource["kind"];
        path: string;
        duration_ms: number | null;
      }>
    >("session_audio_sources", { sessionId });

    return sources.map((source) => ({
      kind: source.kind,
      url: convertFileSrc(source.path),
      duration_ms: source.duration_ms,
    }));
  },

  sessionWaveform: (sessionId: string) =>
    invoke<number[]>("session_waveform", { sessionId }),

  recoverRecording: (sessionId: string) =>
    invoke<Session>("recover_recording", { sessionId }),

  async revealSession(sessionId: string) {
    await invoke("reveal_session", { sessionId });
  },

  renameSession: (sessionId: string, title: string) =>
    invoke<Session>("rename_session", {
      request: {
        session_id: sessionId,
        title,
      },
    }),

  moveSession: (sessionId: string, projectId: string | null) =>
    invoke<Session>("move_session", {
      request: {
        session_id: sessionId,
        project_id: projectId,
      },
    }),

  trashedSessions: () => invoke<Session[]>("trashed_sessions"),

  trashSession: (sessionId: string) =>
    invoke<Session>("trash_session", { sessionId }),

  restoreSession: (sessionId: string) =>
    invoke<Session>("restore_session", { sessionId }),

  emptyTrash: () => invoke<void>("empty_trash"),

  saveNotes(sessionId: string, markdown: string, expectedHash: string) {
    return invoke<SavedDocument>("save_notes", {
      request: {
        session_id: sessionId,
        markdown,
        expected_hash: expectedHash,
      },
    });
  },

  async subscribe(onEvent: (event: AppEvent) => void) {
    const channel = new Channel<AppEvent>();
    channel.onmessage = onEvent;
    await invoke("subscribe", { channel });
  },
};
