import type {
  AppEvent,
  AppSettings,
  AppSnapshot,
  AudioDevices,
  ConnectionTestResult,
  CreateRecordingRequest,
  CredentialStatus,
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
} from "@/types/domain";

export interface NativeBridge {
  bootstrap(): Promise<AppSnapshot>;
  saveSettings(settings: AppSettings): Promise<AppSettings>;
  initializeLibrary(path: string): Promise<AppSnapshot>;
  chooseLibrary(): Promise<string | null>;
  createProject(name: string): Promise<Project>;
  searchLibrary(query: string, filters: SearchFilters): Promise<SearchPage>;
  createRecording(request: CreateRecordingRequest): Promise<Session>;
  importMedia(path?: string): Promise<Session | null>;
  audioDevices(): Promise<AudioDevices>;
  openSystemAudioPermissionSettings(): Promise<void>;
  configureGlobalShortcut(
    shortcut: string | null,
    onTrigger: () => void,
  ): Promise<void>;
  pauseRecording(paused: boolean): Promise<RecordingStatus | null>;
  recordingStatus(): Promise<RecordingStatus | null>;
  stopRecording(): Promise<Session | null>;
  openAiCredentialStatus(): Promise<CredentialStatus>;
  saveOpenAiApiKey(apiKey: string): Promise<CredentialStatus>;
  removeOpenAiApiKey(): Promise<CredentialStatus>;
  testOpenAiConnection(): Promise<ConnectionTestResult>;
  enqueueTranscription(
    sessionId: string,
    provider: "local" | "open_ai",
    modelId: string,
  ): Promise<Job>;
  retryJob(jobId: string): Promise<Job>;
  cancelJob(jobId: string): Promise<Job>;
  updateTranscriptSegment(
    sessionId: string,
    segmentId: string,
    text: string,
    expectedRevision: number,
  ): Promise<Transcript>;
  renameSpeaker(
    sessionId: string,
    speakerId: string,
    displayName: string,
    expectedRevision: number,
  ): Promise<Transcript>;
  mergeSpeakers(
    sessionId: string,
    sourceSpeakerId: string,
    targetSpeakerId: string,
    expectedRevision: number,
  ): Promise<Transcript>;
  localModelStatuses(): Promise<LocalModel[]>;
  downloadLocalModel(
    modelId: string,
    onProgress: (progress: JobProgress) => void,
  ): Promise<LocalModel>;
  removeLocalModel(modelId: string): Promise<LocalModel>;
  exportSession(sessionId: string, format: ExportFormat): Promise<ExportResult>;
  sessionWorkspace(sessionId: string): Promise<SessionWorkspace | null>;
  sessionAudioSources(sessionId: string): Promise<SessionAudioSource[]>;
  sessionWaveform(sessionId: string): Promise<number[]>;
  recoverRecording(sessionId: string): Promise<Session>;
  revealSession(sessionId: string): Promise<void>;
  moveSession(sessionId: string, projectId: string | null): Promise<Session>;
  trashedSessions(): Promise<Session[]>;
  trashSession(sessionId: string): Promise<Session>;
  restoreSession(sessionId: string): Promise<Session>;
  saveNotes(
    sessionId: string,
    markdown: string,
    expectedHash: string,
  ): Promise<SavedDocument | null>;
  subscribe(onEvent: (event: AppEvent) => void): Promise<void>;
}
