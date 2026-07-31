export type RecordingMode = "record_only" | "local_live" | "open_ai_live";
export type OpenAiTranscriptionModel =
  "gpt_transcribe" | "gpt_4o_transcribe_diarize" | "gpt_4o_mini_transcribe";
export type RecordingProjectSelection =
  | { kind: "automatic" }
  | { kind: "inbox" }
  | { kind: "project"; project_id: string };

export interface CreateRecordingRequest {
  title: string;
  projectId: string | null;
  microphoneDeviceId: string | null;
  captureSystemAudio: boolean;
  languageHint: string | null;
  recordingMode: RecordingMode;
  openAiModel: OpenAiTranscriptionModel;
}

export type GlobalShortcutPreset =
  | "command_or_control_shift_r"
  | "command_or_control_shift_space"
  | "alt_shift_r";
export type ThemePreference = "system" | "light" | "dark";
export type SessionLifecycle =
  | "draft"
  | "recording"
  | "paused"
  | "finalizing"
  | "ready"
  | "needs_attention"
  | "recovered"
  | "trashed";

export interface LibraryDescriptor {
  id: string;
  path: string;
  project_count: number;
  session_count: number;
}

export interface Project {
  schema_version: number;
  id: string;
  name: string;
  revision: number;
  glossary: string[];
  openai_profile_id: string | null;
  language_hint: string | null;
  created_at: string;
  updated_at: string;
}

export interface Session {
  schema_version: number;
  id: string;
  title: string;
  project_id: string | null;
  revision: number;
  source: "recording" | "import";
  lifecycle: SessionLifecycle;
  recovery_state: "none" | "recoverable" | "recovered" | "incomplete";
  created_at: string;
  started_at: string | null;
  stopped_at: string | null;
  duration_ms: number;
  microphone: CaptureDevice | null;
  system_output: CaptureDevice | null;
  artifacts: Artifact[];
  current_transcript_id: string | null;
}

export interface CaptureDevice {
  stable_id: string;
  label: string;
  sample_rate_hz: number;
  channels: number;
}

export type ArtifactKind =
  | "microphone"
  | "system"
  | "mixed"
  | "imported_original"
  | "imported_audio"
  | "waveform";

export interface Artifact {
  id: string;
  kind: ArtifactKind;
  relative_path: string;
  duration_ms: number | null;
}

export interface JobProgress {
  stage: string;
  completed_units: number;
  total_units: number | null;
  unit: "audio_ms" | "bytes" | "items";
  message: string;
}

export interface Job {
  id: string;
  session_id: string | null;
  kind: string;
  state: string;
  progress: JobProgress | null;
  attempt: number;
  error_message: string | null;
  created_at: string;
  updated_at: string;
}

export interface AppSettings {
  revision: number;
  setup_completed: boolean;
  recording_mode: RecordingMode;
  openai_transcription_model: OpenAiTranscriptionModel;
  microphone_device_id: string | null;
  capture_system_audio: boolean;
  recording_project_selection: RecordingProjectSelection;
  global_shortcut_enabled: boolean;
  global_shortcut: GlobalShortcutPreset;
  appearance: {
    theme: ThemePreference;
    reduced_motion: boolean;
  };
}

export interface AppSnapshot {
  library: LibraryDescriptor | null;
  projects: Project[];
  recent_sessions: Session[];
  active_jobs: Job[];
  settings: AppSettings;
}

export interface SearchFilters {
  project_id: string | null;
  content_kinds: string[];
}

export interface SearchResult {
  session_id: string;
  session_title: string;
  kind: "session" | "transcript" | "notes";
  excerpt: string;
  timestamp_ms: number | null;
}

export interface SearchPage {
  results: SearchResult[];
  next_cursor: string | null;
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

export interface LocalModel {
  id: string;
  preset: "fast" | "balanced" | "best";
  label: string;
  description: string;
  byte_count: number;
  installed: boolean;
}

export interface Speaker {
  id: string;
  display_name: string;
  source: "microphone" | "system" | "diarized" | "manual" | "imported";
}

export interface TranscriptSegment {
  id: string;
  start_ms: number;
  end_ms: number;
  text: string;
  speaker_id: string;
  source: "microphone" | "system" | "mixed" | "imported";
  edited: boolean;
}

export interface Transcript {
  schema_version: number;
  id: string;
  session_id: string;
  revision: number;
  source_run_id: string;
  detected_language: string | null;
  language_hint: string | null;
  speakers: Speaker[];
  segments: TranscriptSegment[];
  updated_at: string;
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
}

export interface SavedDocument {
  content_hash: string;
}

export interface SessionAudioSource {
  kind: ArtifactKind;
  url: string;
  duration_ms: number | null;
}

export interface LiveTranscriptUpdate {
  session_id: string;
  item_id: string;
  source: "microphone" | "system" | "mixed" | "imported";
  text: string;
  completed: boolean;
  started_at_ms: number;
}

export type AppEvent =
  | {
      type: "job_progress";
      payload: {
        job_id: string;
        progress: JobProgress;
      };
    }
  | {
      type: "recording_levels";
      payload: {
        session_id: string;
        is_paused: boolean;
        captures_system_audio: boolean;
        microphone_peak: number;
        system_peak: number;
        elapsed_ms: number;
        dropped_packets: number;
      };
    }
  | {
      type: "live_transcript_changed";
      payload: LiveTranscriptUpdate;
    }
  | {
      type: "recording_state_changed";
      payload: SessionLifecycle;
    }
  | {
      type: "job_state_changed";
      payload: Job;
    }
  | {
      type: "library_changed";
    }
  | {
      type: "import_requested";
    }
  | {
      type: "attention_required";
      payload: string;
    };
