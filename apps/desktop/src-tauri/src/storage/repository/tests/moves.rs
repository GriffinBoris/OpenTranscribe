use super::*;

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
