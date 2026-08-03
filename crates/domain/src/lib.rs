mod artifact;
mod dictation;
mod event;
mod job;
mod library;
mod project;
mod session;
mod settings;
mod transcript;

pub use artifact::{Artifact, ArtifactKind, AudioSource, Codec};
pub use dictation::{DictationHistoryEntry, DictationPhase, DictationStatus};
pub use event::{AppEvent, LevelSnapshot, LiveTranscriptUpdate, TranscriptionPreviewUpdate};
pub use job::{Job, JobKind, JobProgress, JobStage, JobState, ProgressUnit};
pub use library::{
    AppSnapshot, LibraryDescriptor, LibraryManifest, SearchFilters, SearchPage, SearchResult,
};
pub use project::Project;
pub use session::{
    CaptureDevice, GapEvent, PauseEvent, RecoveryState, Session, SessionLifecycle, SessionSource,
};
pub use settings::{
    APP_SETTINGS_SCHEMA_VERSION, AppSettings, Appearance, DictationProvider, GlobalShortcut,
    OpenAiTranscriptionModel, RecordingMode, RecordingProjectSelection, ThemePreference,
};
pub use transcript::{
    Speaker, SpeakerSource, Transcript, TranscriptRun, TranscriptRunStatus, TranscriptSegment,
    TranscriptSource,
};

pub const SCHEMA_VERSION: u32 = 1;

pub fn new_id() -> String {
    ulid::Ulid::generate().to_string()
}

pub fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}
