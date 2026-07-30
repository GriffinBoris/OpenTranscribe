use std::fs;
use std::path::Path;

use opentranscribe_domain::{
    Artifact, ArtifactKind, AudioSource, CaptureDevice, Codec, JobState, RecoveryState,
    SCHEMA_VERSION, SearchFilters, SessionLifecycle, SessionSource, Speaker, SpeakerSource,
    Transcript, TranscriptSegment, new_id, now,
};
use tempfile::tempdir;

use crate::audio::RecordingCapture;
use crate::error::AppError;

use super::{LibraryRepository, read_directories, read_files, relative_path};
use crate::storage::JobRequest;

#[test]
fn initializes_a_readable_library() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let project = repository
        .create_project("Product".to_owned())
        .expect("project should be created");
    let session = repository
        .create_session(
            "Weekly sync".to_owned(),
            Some(project.id),
            SessionSource::Recording,
        )
        .expect("session should be created");

    assert_eq!(repository.descriptor().unwrap().project_count, 1);
    assert_eq!(repository.descriptor().unwrap().session_count, 1);
    assert_eq!(session.title, "Weekly sync");
}

#[test]
fn discards_only_a_new_empty_draft_session() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session(
            "Untitled recording".to_owned(),
            None,
            SessionSource::Recording,
        )
        .expect("session should be created");

    repository
        .discard_draft_session(&session.id)
        .expect("empty draft should be discarded");

    assert!(
        repository
            .sessions()
            .expect("sessions should remain readable")
            .is_empty()
    );
}

#[test]
fn stores_a_per_session_language_hint_for_recordings() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");

    let session = repository
        .create_recording_session("Customer interview".to_owned(), None, Some("es".to_owned()))
        .expect("recording session should be created");

    assert_eq!(session.language_hint.as_deref(), Some("es"));
}

#[test]
fn preserves_and_marks_an_interrupted_job_for_retry() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session("Meeting".to_owned(), None, SessionSource::Recording)
        .expect("session should be created");
    let record = repository
        .create_job(
            session.id,
            JobRequest::TranscribeLocal {
                model_id: "whisper".to_owned(),
            },
        )
        .expect("job should be created");
    repository
        .update_job(&record.job.id, |job| job.state = JobState::Running)
        .expect("job should start");
    drop(repository);

    let recovered = LibraryRepository::initialize(directory.path()).expect("library should reopen");
    let jobs = recovered.jobs().expect("jobs should remain readable");

    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].state, JobState::Failed);
    assert!(jobs[0].error_message.is_some());
}

#[test]
fn marks_a_queued_job_as_interrupted_when_the_library_reopens() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session("Meeting".to_owned(), None, SessionSource::Recording)
        .expect("session should be created");
    repository
        .create_job(
            session.id,
            JobRequest::TranscribeLocal {
                model_id: "whisper".to_owned(),
            },
        )
        .expect("job should be queued");
    drop(repository);

    let recovered = LibraryRepository::initialize(directory.path()).expect("library should reopen");
    let jobs = recovered.jobs().expect("jobs should remain readable");

    assert_eq!(jobs[0].state, JobState::Failed);
}

#[test]
fn prevents_parallel_transcriptions_for_one_session() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session("Meeting".to_owned(), None, SessionSource::Recording)
        .expect("session should be created");
    repository
        .create_job(
            session.id.clone(),
            JobRequest::TranscribeLocal {
                model_id: "whisper".to_owned(),
            },
        )
        .expect("first job should be queued");

    let error = repository
        .create_job(
            session.id,
            JobRequest::TranscribeOpenAi {
                model_id: "gpt-transcribe".to_owned(),
            },
        )
        .expect_err("second job should be rejected");

    assert_eq!(
        error.to_string(),
        "a transcription is already running for this session"
    );
}

#[test]
fn canceled_jobs_leave_the_session_available_for_another_transcription() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session("Meeting".to_owned(), None, SessionSource::Recording)
        .expect("session should be created");
    let first = repository
        .create_job(
            session.id.clone(),
            JobRequest::TranscribeLocal {
                model_id: "whisper".to_owned(),
            },
        )
        .expect("job should be queued");
    repository
        .update_job(&first.job.id, |job| job.state = JobState::Canceled)
        .expect("job should cancel");
    let second = repository
        .create_job(
            session.id,
            JobRequest::TranscribeOpenAi {
                model_id: "gpt-transcribe".to_owned(),
            },
        )
        .expect("a replacement job should be queued");

    let jobs = repository.jobs().expect("jobs should load");
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, second.job.id);
}

#[test]
fn imports_media_into_an_inbox_session() {
    let directory = tempdir().expect("temporary directory should exist");
    let source = directory.path().join("interview.wav");
    let mut writer = hound::WavWriter::create(
        &source,
        hound::WavSpec {
            channels: 1,
            sample_rate: 48_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .expect("fixture should be created");

    for _ in 0..480 {
        writer
            .write_sample(0_i16)
            .expect("fixture sample should be written");
    }
    writer.finalize().expect("fixture should finalize");
    let library = directory.path().join("library");
    let repository = LibraryRepository::initialize(&library).expect("library should initialize");

    let session = repository
        .import_media(&source)
        .expect("media should import");

    assert_eq!(session.title, "interview");
    assert_eq!(session.source, SessionSource::Import);
    assert_eq!(session.duration_ms, 10);
    assert_eq!(session.artifacts.len(), 3);
    assert!(
        session
            .artifacts
            .iter()
            .all(|artifact| library.join(&artifact.relative_path).exists())
    );
    assert!(
        session
            .artifacts
            .iter()
            .any(|artifact| artifact.kind == ArtifactKind::ImportedAudio)
    );
}

#[test]
fn leaves_no_session_when_media_import_fails() {
    let directory = tempdir().expect("temporary directory should exist");
    let source = directory.path().join("not-a-recording.wav");
    fs::write(&source, b"not audio").expect("invalid fixture should be written");
    let library = directory.path().join("library");
    let repository = LibraryRepository::initialize(&library).expect("library should initialize");

    assert!(repository.import_media(&source).is_err());
    assert!(
        repository
            .sessions()
            .expect("sessions should remain readable")
            .is_empty()
    );
    assert!(
        read_directories(&library.join("Inbox"))
            .expect("inbox should remain readable")
            .is_empty()
    );
}

#[test]
fn moves_sessions_to_trash_and_restores_them() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session("Weekly sync".to_owned(), None, SessionSource::Recording)
        .expect("session should be created");

    let trashed = repository
        .trash_session(&session.id)
        .expect("session should move to trash");

    assert_eq!(trashed.lifecycle, SessionLifecycle::Trashed);
    assert!(
        repository
            .sessions()
            .expect("sessions should load")
            .is_empty()
    );
    assert_eq!(
        repository
            .trashed_sessions()
            .expect("trash should load")
            .len(),
        1
    );

    let restored = repository
        .restore_session(&session.id)
        .expect("session should restore");

    assert_eq!(restored.lifecycle, SessionLifecycle::Ready);
    assert_eq!(
        repository.sessions().expect("sessions should load").len(),
        1
    );
    assert!(
        repository
            .trashed_sessions()
            .expect("trash should load")
            .is_empty()
    );
}

#[test]
fn moves_sessions_between_inbox_and_projects_without_breaking_artifacts() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let project = repository
        .create_project("Product".to_owned())
        .expect("project should be created");
    let session = repository
        .create_session("Weekly sync".to_owned(), None, SessionSource::Recording)
        .expect("session should be created");
    let original_directory = repository
        .session_directory_path(&session.id)
        .expect("session directory should resolve");
    let audio_path = original_directory.join("audio/meeting.wav");
    fs::write(original_directory.join("notes.md"), b"Agenda\n").expect("notes should be written");
    fs::write(&audio_path, b"audio").expect("audio should be written");
    let session = repository
        .update_session(&session.id, |session| {
            session.artifacts.push(Artifact {
                id: new_id(),
                kind: ArtifactKind::ImportedOriginal,
                relative_path: relative_path(directory.path(), &audio_path),
                codec: Codec::Original,
                sample_rate_hz: None,
                channels: None,
                duration_ms: None,
                byte_count: 5,
                sha256: "fixture".to_owned(),
            });
        })
        .expect("artifact should be indexed");

    let moved = repository
        .move_session(&session.id, Some(project.id.clone()))
        .expect("session should move to the project");
    let project_directory = repository
        .project_directory(&project.id)
        .expect("project directory should resolve");
    let moved_directory = repository
        .session_directory_path(&session.id)
        .expect("moved session should resolve");

    assert_eq!(moved.project_id.as_deref(), Some(project.id.as_str()));
    assert_eq!(moved.revision, session.revision + 1);
    assert_eq!(moved_directory.parent(), Some(project_directory.as_path()));
    assert_eq!(
        fs::read_to_string(moved_directory.join("notes.md")).expect("notes should remain readable"),
        "Agenda\n"
    );
    assert!(
        directory
            .path()
            .join(&moved.artifacts[0].relative_path)
            .exists()
    );
    assert!(!original_directory.exists());
    assert_eq!(
        repository
            .search(
                "Weekly",
                &SearchFilters {
                    project_id: Some(project.id),
                    content_kinds: vec!["session".to_owned()],
                },
            )
            .expect("project search should succeed")
            .results
            .len(),
        1
    );

    let returned = repository
        .move_session(&session.id, None)
        .expect("session should return to the inbox");
    let returned_directory = repository
        .session_directory_path(&session.id)
        .expect("returned session should resolve");

    assert_eq!(returned.project_id, None);
    assert_eq!(
        returned_directory.parent(),
        Some(directory.path().join("Inbox").as_path())
    );
    assert!(
        directory
            .path()
            .join(&returned.artifacts[0].relative_path)
            .exists()
    );
}

#[test]
fn finalizes_recording_chunks_into_one_artifact() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session("Weekly sync".to_owned(), None, SessionSource::Recording)
        .expect("session should be created");
    let recovery_directory = repository
        .recording_directory(&session.id)
        .expect("recovery directory should resolve");
    let mut writer = hound::WavWriter::create(
        recovery_directory.join("microphone-000001.wav"),
        hound::WavSpec {
            channels: 1,
            sample_rate: 10,
            bits_per_sample: 24,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .expect("chunk should be created");

    for _ in 0..25 {
        writer
            .write_sample(0_i32)
            .expect("sample should be written");
    }

    writer.finalize().expect("chunk should finalize");
    let finalized = repository
        .finish_recording(&session.id)
        .expect("recording should finalize");

    assert_eq!(finalized.artifacts.len(), 2);
    assert_eq!(finalized.duration_ms, 2_500);
    assert!(
        directory
            .path()
            .join(&finalized.artifacts[0].relative_path)
            .exists()
    );
    assert!(
        read_files(&recovery_directory)
            .expect("recovery directory should be readable")
            .is_empty()
    );
}

#[test]
fn marks_a_stopped_recording_as_recoverable_when_finalization_needs_attention() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session(
            "Interrupted finalization".to_owned(),
            None,
            SessionSource::Recording,
        )
        .expect("session should be created");
    repository
        .begin_recording(
            &session.id,
            RecordingCapture {
                microphone: CaptureDevice {
                    stable_id: "microphone".to_owned(),
                    label: "Microphone".to_owned(),
                    sample_rate_hz: 48_000,
                    channels: 1,
                },
                system_output: None,
            },
        )
        .expect("recording should begin");

    let marked = repository
        .mark_recording_needs_attention(&session.id)
        .expect("recording should be marked for recovery");

    assert_eq!(marked.lifecycle, SessionLifecycle::NeedsAttention);
    assert_eq!(marked.recovery_state, RecoveryState::Recoverable);
}

#[test]
fn recovers_interrupted_recording_chunks_into_playable_audio() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session(
            "Interrupted meeting".to_owned(),
            None,
            SessionSource::Recording,
        )
        .expect("session should be created");
    repository
        .begin_recording(
            &session.id,
            RecordingCapture {
                microphone: CaptureDevice {
                    stable_id: "microphone".to_owned(),
                    label: "Microphone".to_owned(),
                    sample_rate_hz: 10,
                    channels: 1,
                },
                system_output: None,
            },
        )
        .expect("recording should begin");
    let recovery_directory = repository
        .recording_directory(&session.id)
        .expect("recovery directory should resolve");
    write_recovery_chunk(&recovery_directory.join("microphone-000001.wav"), 1, 10, 25);
    drop(repository);

    let reopened = LibraryRepository::initialize(directory.path()).expect("library should reopen");
    let interrupted = reopened
        .session_workspace(&session.id)
        .expect("interrupted session should remain readable")
        .session;
    assert_eq!(interrupted.lifecycle, SessionLifecycle::Recovered);
    assert_eq!(interrupted.recovery_state, RecoveryState::Recoverable);

    let recovered = reopened
        .recover_recording(&session.id)
        .expect("recording should recover");

    assert_eq!(recovered.lifecycle, SessionLifecycle::Recovered);
    assert_eq!(recovered.recovery_state, RecoveryState::Recovered);
    assert_eq!(recovered.duration_ms, 2_500);
    assert!(
        recovered
            .artifacts
            .iter()
            .any(|artifact| artifact.kind == ArtifactKind::Microphone)
    );
    assert!(
        read_files(&recovery_directory)
            .expect("recovery directory should remain readable")
            .is_empty()
    );
}

#[test]
fn finalizes_microphone_and_system_tracks_separately() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session("Weekly sync".to_owned(), None, SessionSource::Recording)
        .expect("session should be created");
    repository
        .begin_recording(
            &session.id,
            RecordingCapture {
                microphone: CaptureDevice {
                    stable_id: "microphone".to_owned(),
                    label: "Microphone".to_owned(),
                    sample_rate_hz: 10,
                    channels: 1,
                },
                system_output: Some(CaptureDevice {
                    stable_id: "system".to_owned(),
                    label: "System output".to_owned(),
                    sample_rate_hz: 10,
                    channels: 2,
                }),
            },
        )
        .expect("recording should begin");
    let recovery_directory = repository
        .recording_directory(&session.id)
        .expect("recovery directory should resolve");
    write_recovery_chunk(&recovery_directory.join("microphone-000001.wav"), 1, 10, 25);
    write_recovery_chunk(&recovery_directory.join("system-000001.wav"), 2, 10, 60);

    let finalized = repository
        .finish_recording(&session.id)
        .expect("recording should finalize");

    assert_eq!(finalized.artifacts.len(), 4);
    assert_eq!(finalized.duration_ms, 3_000);
    assert!(
        finalized
            .artifacts
            .iter()
            .any(|artifact| artifact.kind == ArtifactKind::Microphone)
    );
    assert!(
        finalized
            .artifacts
            .iter()
            .any(|artifact| artifact.kind == ArtifactKind::System)
    );
    assert!(
        finalized
            .artifacts
            .iter()
            .any(|artifact| artifact.kind == ArtifactKind::Mixed)
    );
    assert!(finalized.microphone.is_some());
    assert!(finalized.system_output.is_some());
}

#[test]
fn clears_stale_recovery_chunks_after_a_valid_finalize() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session("Weekly sync".to_owned(), None, SessionSource::Recording)
        .expect("session should be created");
    let recovery_directory = repository
        .recording_directory(&session.id)
        .expect("recovery directory should resolve");
    let mut writer = hound::WavWriter::create(
        recovery_directory.join("microphone-000001.wav"),
        hound::WavSpec {
            channels: 1,
            sample_rate: 10,
            bits_per_sample: 24,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .expect("chunk should be created");
    writer
        .write_sample(0_i32)
        .expect("sample should be written");
    writer.finalize().expect("chunk should finalize");
    let finalized = repository
        .finish_recording(&session.id)
        .expect("recording should finalize");
    let finalized_path = directory.path().join(&finalized.artifacts[0].relative_path);
    fs::copy(
        finalized_path,
        recovery_directory.join("microphone-000001.wav"),
    )
    .expect("stale recovery chunk should be restored");
    drop(repository);

    let reopened = LibraryRepository::initialize(directory.path()).expect("library should reopen");
    let workspace = reopened
        .session_workspace(&session.id)
        .expect("session should remain readable");

    assert_eq!(workspace.session.lifecycle, SessionLifecycle::Ready);
    assert!(
        read_files(&recovery_directory)
            .expect("recovery directory should be readable")
            .is_empty()
    );
}

#[test]
fn edits_a_transcript_segment_and_rewrites_readable_artifacts() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session("Weekly sync".to_owned(), None, SessionSource::Recording)
        .expect("session should be created");
    let speaker_id = new_id();
    let segment_id = new_id();
    let transcript = Transcript {
        schema_version: SCHEMA_VERSION,
        id: new_id(),
        session_id: session.id.clone(),
        revision: 1,
        source_run_id: new_id(),
        detected_language: Some("en".to_owned()),
        language_hint: None,
        speakers: vec![Speaker {
            id: speaker_id.clone(),
            display_name: "Me".to_owned(),
            source: SpeakerSource::Microphone,
        }],
        segments: vec![TranscriptSegment {
            id: segment_id.clone(),
            start_ms: 0,
            end_ms: 1_000,
            text: "Original text".to_owned(),
            speaker_id,
            source: AudioSource::Microphone,
            edited: false,
            word_timings: None,
        }],
        updated_at: now(),
    };
    let transcript_directory = repository
        .session_directory_path(&session.id)
        .expect("session directory should resolve")
        .join("transcripts");
    fs::create_dir_all(&transcript_directory).expect("transcript directory should be created");
    fs::write(
        transcript_directory.join("transcript.json"),
        serde_json::to_vec_pretty(&transcript).expect("transcript should serialize"),
    )
    .expect("transcript should be written");

    let updated = repository
        .update_transcript_segment(&session.id, &segment_id, "Edited text".to_owned(), 1)
        .expect("segment should update");

    assert_eq!(updated.revision, 2);
    assert_eq!(updated.segments[0].text, "Edited text");
    assert!(updated.segments[0].edited);
    assert!(
        fs::read_to_string(transcript_directory.join("transcript.md"))
            .expect("markdown should be readable")
            .contains("Edited text")
    );
}

#[test]
fn renames_and_merges_transcript_speakers() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session("Interview".to_owned(), None, SessionSource::Import)
        .expect("session should be created");
    let first_speaker_id = new_id();
    let second_speaker_id = new_id();
    let transcript = Transcript {
        schema_version: SCHEMA_VERSION,
        id: new_id(),
        session_id: session.id.clone(),
        revision: 1,
        source_run_id: new_id(),
        detected_language: Some("en".to_owned()),
        language_hint: None,
        speakers: vec![
            Speaker {
                id: first_speaker_id.clone(),
                display_name: "Speaker 1".to_owned(),
                source: SpeakerSource::Imported,
            },
            Speaker {
                id: second_speaker_id.clone(),
                display_name: "Speaker 2".to_owned(),
                source: SpeakerSource::Imported,
            },
        ],
        segments: vec![TranscriptSegment {
            id: new_id(),
            start_ms: 0,
            end_ms: 1_000,
            text: "Hello".to_owned(),
            speaker_id: first_speaker_id.clone(),
            source: AudioSource::Imported,
            edited: false,
            word_timings: None,
        }],
        updated_at: now(),
    };
    let transcript_directory = repository
        .session_directory_path(&session.id)
        .expect("session directory should resolve")
        .join("transcripts");
    fs::create_dir_all(&transcript_directory).expect("transcript directory should be created");
    fs::write(
        transcript_directory.join("transcript.json"),
        serde_json::to_vec_pretty(&transcript).expect("transcript should serialize"),
    )
    .expect("transcript should be written");

    let renamed = repository
        .rename_speaker(&session.id, &first_speaker_id, "Host".to_owned(), 1)
        .expect("speaker should rename");
    let merged = repository
        .merge_speakers(
            &session.id,
            &first_speaker_id,
            &second_speaker_id,
            renamed.revision,
        )
        .expect("speakers should merge");

    assert_eq!(renamed.speakers[0].display_name, "Host");
    assert_eq!(merged.speakers.len(), 1);
    assert_eq!(merged.segments[0].speaker_id, second_speaker_id);
    assert!(merged.segments[0].edited);
}

#[test]
fn searches_session_titles_notes_and_transcript_timestamps_after_reopening() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session(
            "Quarterly roadmap".to_owned(),
            None,
            SessionSource::Recording,
        )
        .expect("session should be created");
    let workspace = repository
        .session_workspace(&session.id)
        .expect("workspace should load");
    repository
        .save_notes(
            &session.id,
            "Follow up about lighthouse metrics.",
            &workspace.notes_hash,
        )
        .expect("notes should save");

    let speaker_id = new_id();
    let segment_id = new_id();
    let transcript = Transcript {
        schema_version: SCHEMA_VERSION,
        id: new_id(),
        session_id: session.id.clone(),
        revision: 1,
        source_run_id: new_id(),
        detected_language: Some("en".to_owned()),
        language_hint: None,
        speakers: vec![Speaker {
            id: speaker_id.clone(),
            display_name: "Speaker".to_owned(),
            source: SpeakerSource::Microphone,
        }],
        segments: vec![TranscriptSegment {
            id: segment_id,
            start_ms: 42_000,
            end_ms: 44_000,
            text: "The northstar metric is retention.".to_owned(),
            speaker_id,
            source: AudioSource::Microphone,
            edited: false,
            word_timings: None,
        }],
        updated_at: now(),
    };
    let transcript_path = repository
        .session_directory_path(&session.id)
        .expect("session directory should resolve")
        .join("transcripts/transcript.json");
    fs::write(
        transcript_path,
        serde_json::to_vec_pretty(&transcript).expect("transcript should serialize"),
    )
    .expect("transcript should be written");
    drop(repository);

    let reopened = LibraryRepository::initialize(directory.path()).expect("library should reopen");
    let filters = SearchFilters::default();

    assert_eq!(
        reopened
            .search("roadmap", &filters)
            .expect("title search should work")
            .results[0]
            .kind,
        "session"
    );
    assert_eq!(
        reopened
            .search("lighthouse", &filters)
            .expect("notes search should work")
            .results[0]
            .kind,
        "notes"
    );
    assert_eq!(
        reopened
            .search("northstar", &filters)
            .expect("transcript search should work")
            .results[0]
            .timestamp_ms,
        Some(42_000)
    );
}

#[test]
fn rejects_stale_note_saves_after_an_external_edit() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session(
            "External notes conflict".to_owned(),
            None,
            SessionSource::Recording,
        )
        .expect("session should be created");
    let workspace = repository
        .session_workspace(&session.id)
        .expect("workspace should load");
    let notes_path = repository
        .session_directory_path(&session.id)
        .expect("session directory should resolve")
        .join("notes.md");
    fs::write(&notes_path, "Edited outside OpenTranscribe")
        .expect("external notes edit should be written");

    let result = repository.save_notes(&session.id, "Stale editor content", &workspace.notes_hash);

    assert!(matches!(result, Err(AppError::RevisionConflict)));
    assert_eq!(
        fs::read_to_string(notes_path).expect("external edit should remain readable"),
        "Edited outside OpenTranscribe"
    );
}

#[test]
fn refreshes_external_library_edits_without_reopening() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session("Research interview".to_owned(), None, SessionSource::Import)
        .expect("session should be created");
    let session_directory = repository
        .session_directory_path(&session.id)
        .expect("session directory should resolve");
    fs::write(
        session_directory.join("notes.md"),
        "Externally edited follow-up",
    )
    .expect("external notes edit should be written");

    repository
        .refresh_search_index()
        .expect("external edit should reindex");
    assert_eq!(
        repository
            .search("externally", &SearchFilters::default())
            .expect("external notes should be searchable")
            .results[0]
            .kind,
        "notes"
    );

    fs::remove_dir_all(session_directory).expect("external deletion should succeed");
    repository
        .refresh_search_index()
        .expect("external deletion should reindex");
    assert!(
        repository
            .search("interview", &SearchFilters::default())
            .expect("deleted sessions should leave the index")
            .results
            .is_empty()
    );
}

fn write_recovery_chunk(path: &Path, channels: u16, sample_rate: u32, samples: usize) {
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 24,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .expect("chunk should be created");

    for _ in 0..samples {
        writer
            .write_sample(0_i32)
            .expect("sample should be written");
    }

    writer.finalize().expect("chunk should finalize");
}
