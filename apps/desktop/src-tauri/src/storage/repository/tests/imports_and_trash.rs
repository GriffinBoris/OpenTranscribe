use super::*;

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
fn permanently_removes_trashed_sessions() {
    let directory = tempdir().expect("temporary directory should exist");
    let repository =
        LibraryRepository::initialize(directory.path()).expect("library should initialize");
    let session = repository
        .create_session("Weekly sync".to_owned(), None, SessionSource::Recording)
        .expect("session should be created");

    repository
        .trash_session(&session.id)
        .expect("session should move to trash");
    repository.empty_trash().expect("trash should empty");

    assert!(
        repository
            .trashed_sessions()
            .expect("trash should load")
            .is_empty()
    );
    assert!(
        !directory
            .path()
            .join("Trash/sessions")
            .join(session.id)
            .exists()
    );
}
