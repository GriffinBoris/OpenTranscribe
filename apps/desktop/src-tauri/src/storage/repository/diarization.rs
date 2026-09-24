use std::collections::BTreeMap;
use std::fs;

use opentranscribe_domain::{Speaker, SpeakerSource, Transcript, new_id};
use opentranscribe_transcriber_protocol::SpeakerTurn;

use super::{atomic_file, repository::LibraryRepository};
use crate::error::{AppError, AppResult};

impl LibraryRepository {
    pub(crate) fn diarization_input(
        &self,
        session_id: &str,
        transcript_id: &str,
        expected_revision: u64,
    ) -> AppResult<Transcript> {
        let (_, transcript) = self.editable_transcript(session_id)?;
        if transcript.id != transcript_id || transcript.revision != expected_revision {
            return Err(AppError::RevisionConflict);
        }
        if transcript.segments.is_empty() {
            return Err(AppError::Application(
                "Transcribe this session before identifying speakers.".to_owned(),
            ));
        }
        Ok(transcript)
    }

    pub(crate) fn save_diarization(
        &self,
        input: &Transcript,
        model_id: &str,
        model_sha256: &str,
        turns: &[SpeakerTurn],
    ) -> AppResult<Transcript> {
        let (directory, current) = self.editable_transcript(&input.session_id)?;
        // Compare the full snapshot, including external edits that did not advance revision.
        if current != *input {
            return Err(AppError::RevisionConflict);
        }
        if turns.is_empty() {
            return Err(AppError::Model(
                "No speakers were detected. The transcript was left unchanged.".to_owned(),
            ));
        }
        let mut transcript = input.clone();
        assign_speakers(&mut transcript, turns);
        transcript.revision += 1;
        transcript.updated_at = opentranscribe_domain::now();
        let run_id = new_id();
        let history = directory.join("diarization-runs");
        fs::create_dir_all(&history)?;
        let run_directory = history.join(&run_id);
        fs::create_dir(&run_directory)?;
        atomic_file::write_json(&run_directory.join("input-transcript.json"), input)?;
        atomic_file::write_json(&run_directory.join("provider-response.json"), &turns)?;
        atomic_file::write_json(
            &run_directory.join("run.json"),
            &serde_json::json!({
                "schema_version": 1,
                "id": run_id,
                "session_id": input.session_id,
                "source_run_id": input.source_run_id,
                "input_revision": input.revision,
                "output_revision": input.revision + 1,
                "model_id": model_id,
                "model_sha256": model_sha256,
                "created_at": opentranscribe_domain::now(),
            }),
        )?;
        atomic_file::write_json(&run_directory.join("transcript.json"), &transcript)?;
        self.persist_transcript(&directory, &transcript)?;
        Ok(transcript)
    }
}

fn assign_speakers(transcript: &mut Transcript, turns: &[SpeakerTurn]) {
    transcript.speakers.clear();
    let mut speaker_ids = BTreeMap::new();
    for segment in &mut transcript.segments {
        let mut overlap = BTreeMap::<&str, u64>::new();
        let end = segment.end_ms.max(segment.start_ms + 1);
        for turn in turns {
            let duration = end
                .min(turn.end_ms)
                .saturating_sub(segment.start_ms.max(turn.start_ms));
            if duration > 0 {
                *overlap.entry(&turn.speaker_label).or_default() += duration;
            }
        }
        // Stable label ordering resolves ties; silence never inherits stale speaker identities.
        let label = overlap
            .into_iter()
            .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(left.0)))
            .map(|(label, _)| label);
        let id = speaker_ids.entry(label).or_insert_with(|| {
            let id = new_id();
            transcript.speakers.push(Speaker {
                id: id.clone(),
                display_name: label
                    .map(|label| format!("Speaker {label}"))
                    .unwrap_or_else(|| "Unassigned".to_owned()),
                source: SpeakerSource::Diarized,
            });
            id
        });
        segment.speaker_id = id.clone();
    }
}
