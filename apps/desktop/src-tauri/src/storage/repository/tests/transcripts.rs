use super::*;

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
