use std::fs;

use opentranscribe_domain::Transcript;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::storage::{ExportInput, atomic_file};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Markdown,
    Text,
    Json,
    Srt,
    Vtt,
}

impl ExportFormat {
    fn extension(self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Text => "txt",
            Self::Json => "json",
            Self::Srt => "srt",
            Self::Vtt => "vtt",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExportResult {
    pub format: ExportFormat,
    pub path: String,
}

pub struct ExportService;

impl ExportService {
    pub fn export(input: ExportInput, format: ExportFormat) -> AppResult<ExportResult> {
        fs::create_dir_all(&input.directory)?;
        validate_speakers(&input.transcript)?;
        let path = input.directory.join(format!(
            "{}.{}",
            export_file_name(&input.session.title),
            format.extension()
        ));

        match format {
            ExportFormat::Markdown => atomic_file::write(
                &path,
                transcript_markdown(&input.session.title, &input.transcript, &input.notes)
                    .as_bytes(),
            )?,
            ExportFormat::Text => atomic_file::write(
                &path,
                transcript_text(&input.transcript, &input.notes).as_bytes(),
            )?,
            ExportFormat::Json => atomic_file::write_json(&path, &input.transcript)?,
            ExportFormat::Srt => {
                atomic_file::write(&path, transcript_srt(&input.transcript)?.as_bytes())?
            }
            ExportFormat::Vtt => {
                atomic_file::write(&path, transcript_vtt(&input.transcript)?.as_bytes())?
            }
        }

        Ok(ExportResult {
            format,
            path: path.to_string_lossy().into_owned(),
        })
    }
}

fn validate_speakers(transcript: &Transcript) -> AppResult<()> {
    if transcript.segments.iter().any(|segment| {
        !transcript
            .speakers
            .iter()
            .any(|speaker| speaker.id == segment.speaker_id)
    }) {
        return Err(AppError::Export(
            "transcript segments must reference an existing speaker".to_owned(),
        ));
    }

    Ok(())
}

fn export_file_name(title: &str) -> String {
    let sanitized: String = title
        .chars()
        .map(|character| match character {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '-',
            _ => character,
        })
        .collect();
    sanitized.trim().chars().take(80).collect()
}

fn transcript_markdown(title: &str, transcript: &Transcript, notes: &str) -> String {
    let mut markdown = format!("# {title}\n\n## Transcript\n\n");

    for segment in &transcript.segments {
        markdown.push_str(&format!(
            "**{} — {}**\n\n{}\n\n",
            readable_timestamp(segment.start_ms),
            speaker_name(transcript, &segment.speaker_id),
            segment.text
        ));
    }

    if !notes.trim().is_empty() {
        markdown.push_str("## Notes\n\n");
        markdown.push_str(notes.trim());
        markdown.push('\n');
    }

    markdown
}

fn transcript_text(transcript: &Transcript, notes: &str) -> String {
    let mut text = String::new();

    for segment in &transcript.segments {
        text.push_str(&format!(
            "{} {}: {}\n",
            readable_timestamp(segment.start_ms),
            speaker_name(transcript, &segment.speaker_id),
            segment.text
        ));
    }

    if !notes.trim().is_empty() {
        text.push_str("\nNotes\n\n");
        text.push_str(notes.trim());
        text.push('\n');
    }

    text
}

fn transcript_srt(transcript: &Transcript) -> AppResult<String> {
    validate_subtitle_timing(transcript)?;
    let mut output = String::new();

    for (index, segment) in transcript.segments.iter().enumerate() {
        output.push_str(&format!(
            "{}\n{} --> {}\n{}: {}\n\n",
            index + 1,
            subtitle_timestamp(segment.start_ms, ','),
            subtitle_timestamp(segment.end_ms, ','),
            speaker_name(transcript, &segment.speaker_id),
            segment.text
        ));
    }

    Ok(output)
}

fn transcript_vtt(transcript: &Transcript) -> AppResult<String> {
    validate_subtitle_timing(transcript)?;
    let mut output = String::from("WEBVTT\n\n");

    for segment in &transcript.segments {
        output.push_str(&format!(
            "{} --> {}\n{}: {}\n\n",
            subtitle_timestamp(segment.start_ms, '.'),
            subtitle_timestamp(segment.end_ms, '.'),
            speaker_name(transcript, &segment.speaker_id),
            segment.text
        ));
    }

    Ok(output)
}

fn validate_subtitle_timing(transcript: &Transcript) -> AppResult<()> {
    if transcript
        .segments
        .iter()
        .any(|segment| segment.end_ms <= segment.start_ms)
    {
        return Err(AppError::Export(
            "SRT and VTT exports require timestamped transcript segments".to_owned(),
        ));
    }

    Ok(())
}

fn speaker_name<'a>(transcript: &'a Transcript, speaker_id: &str) -> &'a str {
    transcript
        .speakers
        .iter()
        .find(|speaker| speaker.id == speaker_id)
        .expect("transcript segments must reference an existing speaker")
        .display_name
        .as_str()
}

fn readable_timestamp(milliseconds: u64) -> String {
    let total_seconds = milliseconds / 1_000;
    format!(
        "{:02}:{:02}:{:02}",
        total_seconds / 3_600,
        total_seconds % 3_600 / 60,
        total_seconds % 60
    )
}

fn subtitle_timestamp(milliseconds: u64, separator: char) -> String {
    format!(
        "{}{separator}{:03}",
        readable_timestamp(milliseconds),
        milliseconds % 1_000
    )
}

#[cfg(test)]
mod tests {
    use opentranscribe_domain::{
        AudioSource, SCHEMA_VERSION, Session, SessionSource, Speaker, SpeakerSource, Transcript,
        TranscriptSegment,
    };
    use tempfile::tempdir;

    use crate::storage::ExportInput;

    use super::{ExportFormat, ExportService};

    #[test]
    fn exports_readable_transcript_formats() {
        let directory = tempdir().expect("temporary directory should exist");

        for format in [
            ExportFormat::Markdown,
            ExportFormat::Text,
            ExportFormat::Json,
            ExportFormat::Srt,
            ExportFormat::Vtt,
        ] {
            let result = ExportService::export(
                ExportInput {
                    session: session(),
                    transcript: transcript(2_500),
                    notes: "- Follow up".to_owned(),
                    directory: directory.path().to_owned(),
                },
                format,
            )
            .expect("export should succeed");
            assert!(std::path::Path::new(&result.path).exists());
        }

        let markdown = std::fs::read_to_string(directory.path().join("Weekly sync.md")).unwrap();
        assert!(markdown.contains("**00:00:01 — Me**"));
        assert!(markdown.contains("## Notes"));

        let subtitles = std::fs::read_to_string(directory.path().join("Weekly sync.srt")).unwrap();
        assert!(subtitles.contains("00:00:01,250 --> 00:00:02,500"));
    }

    #[test]
    fn rejects_subtitles_without_real_timing() {
        let directory = tempdir().expect("temporary directory should exist");
        let error = ExportService::export(
            ExportInput {
                session: session(),
                transcript: transcript(1_250),
                notes: String::new(),
                directory: directory.path().to_owned(),
            },
            ExportFormat::Vtt,
        )
        .expect_err("zero-length cues should fail");

        assert_eq!(
            error.to_string(),
            "export failed: SRT and VTT exports require timestamped transcript segments"
        );
    }

    fn session() -> Session {
        Session::new("Weekly sync".to_owned(), None, SessionSource::Recording)
    }

    fn transcript(end_ms: u64) -> Transcript {
        Transcript {
            schema_version: SCHEMA_VERSION,
            id: "transcript-1".to_owned(),
            session_id: "session-1".to_owned(),
            revision: 1,
            source_run_id: "run-1".to_owned(),
            detected_language: Some("en".to_owned()),
            language_hint: None,
            speakers: vec![Speaker {
                id: "speaker-1".to_owned(),
                display_name: "Me".to_owned(),
                source: SpeakerSource::Microphone,
            }],
            segments: vec![TranscriptSegment {
                id: "segment-1".to_owned(),
                start_ms: 1_250,
                end_ms,
                text: "Ship the export flow.".to_owned(),
                speaker_id: "speaker-1".to_owned(),
                source: AudioSource::Microphone,
                edited: false,
                word_timings: None,
            }],
            updated_at: opentranscribe_domain::now(),
        }
    }
}
