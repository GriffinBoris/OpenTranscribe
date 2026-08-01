use super::*;

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
fn renames_a_session_and_updates_the_search_index() {
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

    let renamed = repository
        .rename_session(&session.id, "Weekly design review".to_owned())
        .expect("session should be renamed");

    assert_eq!(renamed.title, "Weekly design review");
    assert_eq!(renamed.revision, session.revision + 1);
    assert_eq!(
        repository
            .session_workspace(&session.id)
            .expect("workspace should remain readable")
            .session
            .title,
        "Weekly design review"
    );
    assert_eq!(
        repository
            .search("design", &SearchFilters::default())
            .expect("renamed title should be searchable")
            .results[0]
            .session_title,
        "Weekly design review"
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
