use super::*;

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
                live_stream_count: 0,
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
                live_stream_count: 0,
            },
        )
        .expect("a replacement job should be queued");

    let jobs = repository.jobs().expect("jobs should load");
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, second.job.id);
}
