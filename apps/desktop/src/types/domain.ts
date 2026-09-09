import type { ArtifactKind } from "@/types/generated/ArtifactKind";
import type { OpenAiTranscriptionModel } from "@/types/generated/OpenAiTranscriptionModel";
import type { RecordingMode } from "@/types/generated/RecordingMode";
import type { Session } from "@/types/generated/Session";
import type { Transcript } from "@/types/generated/Transcript";
import type { TranscriptRun } from "@/types/generated/TranscriptRun";

export type { AppEvent } from "@/types/generated/AppEvent";
export type { AppSettings } from "@/types/generated/AppSettings";
export type { AppSnapshot } from "@/types/generated/AppSnapshot";
export type { Appearance } from "@/types/generated/Appearance";
export type { Artifact } from "@/types/generated/Artifact";
export type { ArtifactKind } from "@/types/generated/ArtifactKind";
export type { AudioSource } from "@/types/generated/AudioSource";
export type { CaptureDevice } from "@/types/generated/CaptureDevice";
export type { Codec } from "@/types/generated/Codec";
export type { DictationHistoryEntry } from "@/types/generated/DictationHistoryEntry";
export type { DictationPhase } from "@/types/generated/DictationPhase";
export type { DictationProvider } from "@/types/generated/DictationProvider";
export type { DictationStatus } from "@/types/generated/DictationStatus";
export type { GapEvent } from "@/types/generated/GapEvent";
export type { Job } from "@/types/generated/Job";
export type { JobKind } from "@/types/generated/JobKind";
export type { JobProgress } from "@/types/generated/JobProgress";
export type { JobStage } from "@/types/generated/JobStage";
export type { JobState } from "@/types/generated/JobState";
export type { LevelSnapshot } from "@/types/generated/LevelSnapshot";
export type { LibraryDescriptor } from "@/types/generated/LibraryDescriptor";
export type { LibraryManifest } from "@/types/generated/LibraryManifest";
export type { LiveTranscriptUpdate } from "@/types/generated/LiveTranscriptUpdate";
export type { OpenAiTranscriptionModel } from "@/types/generated/OpenAiTranscriptionModel";
export type { PauseEvent } from "@/types/generated/PauseEvent";
export type { ProgressUnit } from "@/types/generated/ProgressUnit";
export type { Project } from "@/types/generated/Project";
export type { RecordingMode } from "@/types/generated/RecordingMode";
export type { RecordingProjectSelection } from "@/types/generated/RecordingProjectSelection";
export type { RecoveryState } from "@/types/generated/RecoveryState";
export type { SearchFilters } from "@/types/generated/SearchFilters";
export type { SearchPage } from "@/types/generated/SearchPage";
export type { SearchResult } from "@/types/generated/SearchResult";
export type { Session } from "@/types/generated/Session";
export type { SessionLifecycle } from "@/types/generated/SessionLifecycle";
export type { SessionSource } from "@/types/generated/SessionSource";
export type { Speaker } from "@/types/generated/Speaker";
export type { SpeakerSource } from "@/types/generated/SpeakerSource";
export type { ThemePreference } from "@/types/generated/ThemePreference";
export type { Transcript } from "@/types/generated/Transcript";
export type { TranscriptRun } from "@/types/generated/TranscriptRun";
export type { TranscriptRunStatus } from "@/types/generated/TranscriptRunStatus";
export type { TranscriptSegment } from "@/types/generated/TranscriptSegment";
export type { TranscriptSource } from "@/types/generated/TranscriptSource";
export type { TranscriptionPreviewUpdate } from "@/types/generated/TranscriptionPreviewUpdate";

export interface CreateRecordingRequest {
  title: string;
  projectId: string | null;
  microphoneDeviceId: string | null;
  captureSystemAudio: boolean;
  microphoneEchoCancellation: boolean;
  languageHint: string | null;
  recordingMode: RecordingMode;
  openAiModel: OpenAiTranscriptionModel;
}

export interface AudioDevice {
  id: string;
  label: string;
  is_default: boolean;
}

export interface AudioDevices {
  microphones: AudioDevice[];
  system_audio_available: boolean;
  system_audio_permission_granted: boolean | null;
  system_audio_permission_settings_available: boolean;
}

export interface RecordingStatus {
  session_id: string;
  is_paused: boolean;
  captures_system_audio: boolean;
  elapsed_ms: number;
  microphone_peak: number;
  system_peak: number;
  dropped_packets: number;
}

export interface CredentialStatus {
  configured: boolean;
  masked_key: string | null;
}

export interface ConnectionTestResult {
  connected: boolean;
  message: string;
}

export interface AppUpdate {
  version: string;
  notes: string | null;
}

export type UpdaterInstallationStatus =
  "ready" | "macos_read_only" | "linux_not_writable";

export interface UpdateDownloadProgress {
  completed_bytes: number;
  total_bytes: number | null;
}

export interface LocalModel {
  id: string;
  preset: "fast" | "balanced" | "best" | "cleanup";
  label: string;
  description: string;
  byte_count: number;
  installed: boolean;
}

export type ExportFormat = "markdown" | "text" | "json" | "srt" | "vtt";

export interface ExportResult {
  format: ExportFormat;
  path: string;
}

export interface SessionWorkspace {
  session: Session;
  notes: string;
  notes_hash: string;
  transcript: Transcript | null;
  transcript_run: TranscriptRun | null;
}

export interface SavedDocument {
  content_hash: string;
}

export interface SessionAudioSource {
  kind: ArtifactKind;
  url: string;
  duration_ms: number | null;
}
