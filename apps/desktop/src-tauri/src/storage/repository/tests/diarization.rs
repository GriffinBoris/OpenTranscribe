use opentranscribe_transcriber_protocol::SpeakerTurn;

use super::*;

fn fixture(repository: &LibraryRepository) -> (std::path::PathBuf, Transcript) {
    let session = repository
        .create_session("Speakers".to_owned(), None, SessionSource::Import)
        .unwrap();
    let transcript = Transcript {
        schema_version: SCHEMA_VERSION,
        id: new_id(),
        session_id: session.id.clone(),
        revision: 3,
        source_run_id: new_id(),
        detected_language: Some("en".to_owned()),
        language_hint: None,
        speakers: vec![Speaker {
            id: "old".to_owned(),
            display_name: "Alice".to_owned(),
            source: SpeakerSource::Manual,
        }],
        segments: [(0, 1000), (1000, 2000), (3000, 4000)]
            .into_iter()
            .map(|(start_ms, end_ms)| TranscriptSegment {
                id: new_id(),
                start_ms,
                end_ms,
                text: "Manually corrected words.".to_owned(),
                speaker_id: "old".to_owned(),
                source: AudioSource::Imported,
                edited: true,
                word_timings: Some(
                    serde_json::json!([{ "word": "Manually", "start_ms": start_ms }]),
                ),
            })
            .collect(),
        updated_at: now(),
    };
    let path = repository
        .session_directory_path(&session.id)
        .unwrap()
        .join("transcripts");
    fs::create_dir_all(&path).unwrap();
    fs::write(
        path.join("transcript.json"),
        serde_json::to_vec(&transcript).unwrap(),
    )
    .unwrap();
    (path, transcript)
}

fn turns() -> Vec<SpeakerTurn> {
    vec![
        SpeakerTurn {
            start_ms: 0,
            end_ms: 600,
            speaker_label: "1".to_owned(),
        },
        SpeakerTurn {
            start_ms: 600,
            end_ms: 2000,
            speaker_label: "2".to_owned(),
        },
    ]
}

#[test]
fn relabels_without_changing_words_timing_edits_or_speech_provenance_and_archives_each_run() {
    let temp = tempdir().unwrap();
    let repository = LibraryRepository::initialize(temp.path()).unwrap();
    let (path, original) = fixture(&repository);
    let updated = repository
        .save_diarization(&original, "nemotron-3-diarization", "sha", &turns())
        .unwrap();
    assert_eq!(updated.id, original.id);
    assert_eq!(updated.source_run_id, original.source_run_id);
    assert_eq!(updated.revision, 4);
    assert_eq!(
        updated
            .speakers
            .iter()
            .map(|speaker| speaker.display_name.as_str())
            .collect::<Vec<_>>(),
        ["Speaker 1", "Speaker 2", "Unassigned"]
    );
    for (before, after) in original.segments.iter().zip(&updated.segments) {
        let mut expected = before.clone();
        expected.speaker_id = after.speaker_id.clone();
        assert_eq!(*after, expected);
    }
    let run = fs::read_dir(path.join("diarization-runs"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let archived: Transcript =
        serde_json::from_slice(&fs::read(run.join("input-transcript.json")).unwrap()).unwrap();
    assert_eq!(archived, original);
    let result: Transcript =
        serde_json::from_slice(&fs::read(run.join("transcript.json")).unwrap()).unwrap();
    assert_eq!(result, updated);
    assert!(
        fs::read_to_string(path.join("transcript.md"))
            .unwrap()
            .contains("Speaker 2")
    );
    repository
        .save_diarization(&updated, "nemotron-3-diarization", "sha", &turns())
        .unwrap();
    assert_eq!(
        fs::read_dir(path.join("diarization-runs")).unwrap().count(),
        2
    );
    assert_eq!(
        fs::read(run.join("input-transcript.json")).unwrap(),
        serde_json::to_vec_pretty(&original).unwrap()
    );
}

#[test]
fn rejects_stale_identity_revision_and_external_edits_before_promotion() {
    let temp = tempdir().unwrap();
    let repository = LibraryRepository::initialize(temp.path()).unwrap();
    let (path, original) = fixture(&repository);
    assert!(matches!(
        repository.diarization_input(&original.session_id, "other", original.revision),
        Err(AppError::RevisionConflict)
    ));
    assert!(matches!(
        repository.diarization_input(&original.session_id, &original.id, 1),
        Err(AppError::RevisionConflict)
    ));
    for change in ["revision", "identity", "text", "speaker"] {
        let mut current = original.clone();
        match change {
            "revision" => current.revision += 1,
            "identity" => current.id = new_id(),
            "text" => current.segments[0].text = "External edit".to_owned(),
            "speaker" => current.speakers[0].display_name = "Renamed".to_owned(),
            _ => unreachable!(),
        }
        let bytes = serde_json::to_vec(&current).unwrap();
        fs::write(path.join("transcript.json"), &bytes).unwrap();
        assert!(matches!(
            repository.save_diarization(&original, "nemotron", "sha", &turns()),
            Err(AppError::RevisionConflict)
        ));
        assert_eq!(fs::read(path.join("transcript.json")).unwrap(), bytes);
        assert!(!path.join("diarization-runs").exists());
    }
}

#[test]
fn no_detected_speech_keeps_existing_assignments() {
    let temp = tempdir().unwrap();
    let repository = LibraryRepository::initialize(temp.path()).unwrap();
    let (path, original) = fixture(&repository);
    assert!(
        repository
            .save_diarization(&original, "nemotron", "sha", &[])
            .is_err()
    );
    let current: Transcript =
        serde_json::from_slice(&fs::read(path.join("transcript.json")).unwrap()).unwrap();
    assert_eq!(current, original);
}

#[test]
fn persists_diarization_snapshot_for_restart_and_rejects_duplicate_session_jobs() {
    let temp = tempdir().unwrap();
    let repository = LibraryRepository::initialize(temp.path()).unwrap();
    let (_, original) = fixture(&repository);
    let request = JobRequest::DiarizeLocal {
        model_id: "nemotron-3-diarization".to_owned(),
        transcript: Box::new(original.clone()),
    };
    let job = repository
        .create_job(original.session_id.clone(), request.clone())
        .unwrap();
    assert!(
        repository
            .create_job(original.session_id.clone(), request)
            .is_err()
    );
    let saved = repository.job_record(&job.job.id).unwrap();
    match saved.request {
        JobRequest::DiarizeLocal { transcript, .. } => assert_eq!(*transcript, original),
        _ => panic!("diarization job expected"),
    }
}

#[test]
fn old_local_jobs_default_to_no_diarizer() {
    let request: JobRequest = serde_json::from_value(
        serde_json::json!({ "kind": "transcribe_local", "model_id": "whisper-small-q5_1" }),
    )
    .unwrap();
    assert!(matches!(
        request,
        JobRequest::TranscribeLocal {
            diarization_model_id: None,
            ..
        }
    ));
}

#[test]
fn aggregates_speaker_activity_and_breaks_overlap_ties_deterministically() {
    let temp = tempdir().unwrap();
    let repository = LibraryRepository::initialize(temp.path()).unwrap();
    let (_, original) = fixture(&repository);
    let turns = vec![
        SpeakerTurn {
            start_ms: 0,
            end_ms: 300,
            speaker_label: "1".to_owned(),
        },
        SpeakerTurn {
            start_ms: 600,
            end_ms: 900,
            speaker_label: "1".to_owned(),
        },
        SpeakerTurn {
            start_ms: 0,
            end_ms: 500,
            speaker_label: "2".to_owned(),
        },
        SpeakerTurn {
            start_ms: 1000,
            end_ms: 2000,
            speaker_label: "2".to_owned(),
        },
        SpeakerTurn {
            start_ms: 1000,
            end_ms: 2000,
            speaker_label: "1".to_owned(),
        },
    ];
    let updated = repository
        .save_diarization(&original, "nemotron", "sha", &turns)
        .unwrap();
    assert_eq!(
        updated.segments[0].speaker_id,
        updated.segments[1].speaker_id
    );
    assert_eq!(updated.speakers[0].display_name, "Speaker 1");
}
