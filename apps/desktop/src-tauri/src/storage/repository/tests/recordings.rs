use super::*;

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
fn refreshes_a_legacy_waveform_cache() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session("Weekly sync".to_owned(), None, SessionSource::Recording)
        .expect("session should be created");
    let recovery_directory = repository
        .recording_directory(&session.id)
        .expect("recovery directory should resolve");
    write_recovery_chunk(&recovery_directory.join("microphone-000001.wav"), 1, 10, 25);
    let finalized = repository
        .finish_recording(&session.id)
        .expect("recording should finalize");
    let waveform_path = directory.path().join(
        finalized
            .artifacts
            .iter()
            .find(|artifact| artifact.kind == ArtifactKind::Waveform)
            .expect("finalized recording should contain a waveform")
            .relative_path
            .clone(),
    );
    std::fs::write(
        &waveform_path,
        serde_json::to_vec(&vec![1.0_f32; 160]).expect("legacy waveform should serialize"),
    )
    .expect("legacy waveform should replace the cache");

    let waveform = repository
        .session_waveform(&session.id)
        .expect("legacy waveform should refresh");
    let refreshed = repository
        .session_workspace(&session.id)
        .expect("refreshed session should be readable")
        .session;
    let artifact = refreshed
        .artifacts
        .iter()
        .find(|artifact| artifact.kind == ArtifactKind::Waveform)
        .expect("refreshed session should retain its waveform");

    assert_eq!(waveform.len(), WAVEFORM_BUCKET_COUNT);
    assert!(artifact.relative_path.ends_with("audio/waveform-rms.json"));
    assert_eq!(
        artifact.byte_count,
        std::fs::metadata(directory.path().join(&artifact.relative_path))
            .unwrap()
            .len()
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
fn marks_a_stopped_recording_as_finalizing_before_background_work_starts() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session(
            "Finalizing recording".to_owned(),
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

    let finalizing = repository
        .mark_recording_finalizing(&session.id)
        .expect("recording should be marked finalizing");

    assert_eq!(finalizing.lifecycle, SessionLifecycle::Finalizing);
    assert_eq!(finalizing.recovery_state, RecoveryState::Recoverable);
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
fn selects_system_audio_for_transcription_when_it_is_the_only_recorded_track() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session(
            "Playback capture".to_owned(),
            None,
            SessionSource::Recording,
        )
        .expect("session should be created");
    let session_directory = repository
        .session_directory_path(&session.id)
        .expect("session directory should resolve");
    let system_path = session_directory.join("audio/system.wav");
    write_recovery_chunk(&system_path, 2, 10, 20);
    let system_relative_path = relative_path(directory.path(), &system_path);
    repository
        .update_session(&session.id, |session| {
            session.artifacts.push(Artifact {
                id: new_id(),
                kind: ArtifactKind::System,
                relative_path: system_relative_path,
                codec: Codec::WavPcm24,
                sample_rate_hz: Some(10),
                channels: Some(2),
                duration_ms: Some(1_000),
                byte_count: 0,
                sha256: String::new(),
            });
        })
        .expect("system artifact should be saved");

    let input = repository
        .transcription_input(&session.id)
        .expect("transcription input should resolve");

    assert_eq!(input.artifact_kind, Some(ArtifactKind::System));
    assert_eq!(input.audio_files, vec![system_path]);
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
