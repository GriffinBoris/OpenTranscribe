use std::fs;
use std::path::PathBuf;

use opentranscribe_domain::{ArtifactKind, Session, SpeakerSource, Transcript, TranscriptRun};

use super::atomic_file;
use super::repository::LibraryRepository;
use crate::error::{AppError, AppResult};

pub struct TranscriptionInput {
    pub session: Session,
    pub audio_files: Vec<PathBuf>,
    pub artifact_kind: Option<ArtifactKind>,
}

impl LibraryRepository {
    pub fn transcription_input(&self, session_id: &str) -> AppResult<TranscriptionInput> {
        let directory = self.session_directory(session_id)?;
        let session: Session = serde_json::from_slice(&fs::read(directory.join("session.json"))?)?;
        let artifact = [
            ArtifactKind::Mixed,
            ArtifactKind::Microphone,
            ArtifactKind::System,
            ArtifactKind::ImportedAudio,
            ArtifactKind::ImportedOriginal,
        ]
        .iter()
        .find_map(|kind| {
            session
                .artifacts
                .iter()
                .find(|artifact| artifact.kind == *kind)
        });
        let selected_artifact = artifact.map(|artifact| {
            (
                artifact.kind.clone(),
                self.root.join(&artifact.relative_path),
            )
        });
        let audio_files = selected_artifact
            .as_ref()
            .map(|(_, path)| vec![path.clone()])
            .unwrap_or_default();

        Ok(TranscriptionInput {
            session,
            audio_files,
            artifact_kind: selected_artifact.map(|(kind, _)| kind),
        })
    }

    pub fn save_transcript(
        &self,
        transcript: &Transcript,
        run: &TranscriptRun,
        provider_response: &serde_json::Value,
    ) -> AppResult<Session> {
        let directory = self.session_directory(&transcript.session_id)?;
        let transcript_directory = directory.join("transcripts");
        let run_directory = transcript_directory.join("runs").join(&run.id);
        fs::create_dir_all(&run_directory)?;
        atomic_file::write_json(&run_directory.join("run.json"), run)?;
        atomic_file::write_json(
            &run_directory.join("provider-response.json"),
            provider_response,
        )?;
        atomic_file::write_json(&transcript_directory.join("transcript.json"), transcript)?;
        atomic_file::write(
            &transcript_directory.join("transcript.md"),
            transcript_markdown(transcript).as_bytes(),
        )?;
        self.index.replace_transcript(transcript)?;

        self.update_session(&transcript.session_id, |session| {
            session.current_transcript_id = Some(transcript.id.clone());
            session.transcript_run_ids.push(run.id.clone());
        })
    }

    pub fn update_transcript_segment(
        &self,
        session_id: &str,
        segment_id: &str,
        text: String,
        expected_revision: u64,
    ) -> AppResult<Transcript> {
        let (transcript_directory, mut transcript) = self.editable_transcript(session_id)?;

        if transcript.revision != expected_revision {
            return Err(AppError::RevisionConflict);
        }

        let segment = transcript
            .segments
            .iter_mut()
            .find(|segment| segment.id == segment_id)
            .ok_or(AppError::NotFound)?;
        segment.text = text;
        segment.edited = true;
        self.save_transcript_edits(&transcript_directory, transcript)
    }

    pub fn rename_speaker(
        &self,
        session_id: &str,
        speaker_id: &str,
        display_name: String,
        expected_revision: u64,
    ) -> AppResult<Transcript> {
        let (transcript_directory, mut transcript) = self.editable_transcript(session_id)?;

        if transcript.revision != expected_revision {
            return Err(AppError::RevisionConflict);
        }

        let speaker = transcript
            .speakers
            .iter_mut()
            .find(|speaker| speaker.id == speaker_id)
            .ok_or(AppError::NotFound)?;
        speaker.display_name = display_name;
        speaker.source = SpeakerSource::Manual;
        self.save_transcript_edits(&transcript_directory, transcript)
    }

    pub fn merge_speakers(
        &self,
        session_id: &str,
        source_speaker_id: &str,
        target_speaker_id: &str,
        expected_revision: u64,
    ) -> AppResult<Transcript> {
        if source_speaker_id == target_speaker_id {
            return Err(AppError::Application(
                "choose two different speakers to merge".to_owned(),
            ));
        }

        let (transcript_directory, mut transcript) = self.editable_transcript(session_id)?;

        if transcript.revision != expected_revision {
            return Err(AppError::RevisionConflict);
        }

        if !transcript
            .speakers
            .iter()
            .any(|speaker| speaker.id == target_speaker_id)
        {
            return Err(AppError::NotFound);
        }

        let source_index = transcript
            .speakers
            .iter()
            .position(|speaker| speaker.id == source_speaker_id)
            .ok_or(AppError::NotFound)?;
        transcript.speakers.remove(source_index);

        for segment in &mut transcript.segments {
            if segment.speaker_id == source_speaker_id {
                segment.speaker_id = target_speaker_id.to_owned();
                segment.edited = true;
            }
        }

        self.save_transcript_edits(&transcript_directory, transcript)
    }

    fn editable_transcript(&self, session_id: &str) -> AppResult<(PathBuf, Transcript)> {
        let transcript_directory = self.session_directory(session_id)?.join("transcripts");
        let transcript =
            serde_json::from_slice(&fs::read(transcript_directory.join("transcript.json"))?)?;
        Ok((transcript_directory, transcript))
    }

    fn save_transcript_edits(
        &self,
        transcript_directory: &std::path::Path,
        mut transcript: Transcript,
    ) -> AppResult<Transcript> {
        transcript.revision += 1;
        transcript.updated_at = opentranscribe_domain::now();

        atomic_file::write_json(&transcript_directory.join("transcript.json"), &transcript)?;
        atomic_file::write(
            &transcript_directory.join("transcript.md"),
            transcript_markdown(&transcript).as_bytes(),
        )?;
        self.index.replace_transcript(&transcript)?;
        Ok(transcript)
    }
}

fn transcript_markdown(transcript: &Transcript) -> String {
    let mut markdown = String::from("# Transcript\n\n");

    for segment in &transcript.segments {
        let total_seconds = segment.start_ms / 1_000;
        let speaker_name = transcript
            .speakers
            .iter()
            .find(|speaker| speaker.id == segment.speaker_id)
            .map(|speaker| speaker.display_name.as_str())
            .unwrap_or("Speaker");
        markdown.push_str(&format!(
            "**{:02}:{:02} · {speaker_name}** {}\n\n",
            total_seconds / 60,
            total_seconds % 60,
            segment.text
        ));
    }

    markdown
}
