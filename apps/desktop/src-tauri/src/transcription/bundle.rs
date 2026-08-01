use std::collections::HashMap;

use opentranscribe_domain::{
    ArtifactKind, AudioSource, SessionSource, Speaker, SpeakerSource, Transcript, TranscriptRun,
    TranscriptRunStatus, TranscriptSegment, TranscriptSource,
};

use crate::storage::TranscriptionInput;

pub struct TranscriptionSegmentInput {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
    pub speaker_label: Option<String>,
}

pub struct TranscriptionBundle {
    pub transcript: Transcript,
    pub run: TranscriptRun,
    pub provider_response: serde_json::Value,
}

pub fn build_bundle(
    input: TranscriptionInput,
    source: TranscriptSource,
    model_id: String,
    segments: Vec<TranscriptionSegmentInput>,
    detected_language: Option<String>,
    provider_response: serde_json::Value,
) -> TranscriptionBundle {
    let run_id = opentranscribe_domain::new_id();
    let is_import = input.session.source == SessionSource::Import;
    let (audio_source, default_speaker_name, default_speaker_source) = if is_import {
        (
            AudioSource::Imported,
            "Imported audio",
            SpeakerSource::Imported,
        )
    } else {
        match input.artifact_kind {
            Some(ArtifactKind::Mixed) => {
                (AudioSource::Mixed, "Meeting audio", SpeakerSource::Mixed)
            }
            Some(ArtifactKind::System) => {
                (AudioSource::System, "System audio", SpeakerSource::System)
            }
            _ => (AudioSource::Microphone, "Me", SpeakerSource::Microphone),
        }
    };
    let mut speakers = Vec::new();
    let mut speaker_ids: HashMap<String, String> = HashMap::new();
    let transcript_segments = segments
        .into_iter()
        .map(|segment| {
            let (speaker_key, speaker_name, speaker_source) =
                if let Some(label) = segment.speaker_label {
                    (
                        format!("diarized:{label}"),
                        format!("Speaker {label}"),
                        SpeakerSource::Diarized,
                    )
                } else {
                    (
                        "default".to_owned(),
                        default_speaker_name.to_owned(),
                        default_speaker_source.clone(),
                    )
                };
            let speaker_id = if let Some(speaker_id) = speaker_ids.get(&speaker_key) {
                speaker_id.clone()
            } else {
                let speaker_id = opentranscribe_domain::new_id();
                speaker_ids.insert(speaker_key, speaker_id.clone());
                speakers.push(Speaker {
                    id: speaker_id.clone(),
                    display_name: speaker_name,
                    source: speaker_source,
                });
                speaker_id
            };

            TranscriptSegment {
                id: opentranscribe_domain::new_id(),
                start_ms: segment.start_ms,
                end_ms: segment.end_ms,
                text: segment.text,
                speaker_id,
                source: audio_source.clone(),
                edited: false,
                word_timings: None,
            }
        })
        .collect();
    let transcript = Transcript {
        schema_version: opentranscribe_domain::SCHEMA_VERSION,
        id: opentranscribe_domain::new_id(),
        session_id: input.session.id.clone(),
        revision: 1,
        source_run_id: run_id.clone(),
        detected_language,
        language_hint: input.session.language_hint.clone(),
        speakers,
        segments: transcript_segments,
        updated_at: opentranscribe_domain::now(),
    };
    let run = TranscriptRun {
        schema_version: opentranscribe_domain::SCHEMA_VERSION,
        id: run_id,
        session_id: input.session.id,
        source,
        model_id,
        status: TranscriptRunStatus::Completed,
        input_artifact_ids: input
            .session
            .artifacts
            .iter()
            .filter(|artifact| Some(&artifact.kind) == input.artifact_kind.as_ref())
            .map(|artifact| artifact.id.clone())
            .collect(),
        language_hint: input.session.language_hint,
        glossary: input.session.glossary,
        started_at: Some(opentranscribe_domain::now()),
        completed_at: Some(opentranscribe_domain::now()),
        usage: None,
        approximate_cost_usd: None,
        promoted_at: Some(opentranscribe_domain::now()),
    };

    TranscriptionBundle {
        transcript,
        run,
        provider_response,
    }
}

#[cfg(test)]
mod tests {
    use opentranscribe_domain::{
        ArtifactKind, Session, SessionSource, SpeakerSource, TranscriptSource,
    };

    use crate::storage::TranscriptionInput;

    use super::{TranscriptionSegmentInput, build_bundle};

    #[test]
    fn keeps_diarized_speakers_distinct() {
        let input = TranscriptionInput {
            session: Session::new(
                "Planning meeting".to_owned(),
                None,
                SessionSource::Recording,
            ),
            audio_files: Vec::new(),
            artifact_kind: Some(ArtifactKind::Mixed),
        };
        let bundle = build_bundle(
            input,
            TranscriptSource::OpenAi,
            "gpt-4o-transcribe-diarize".to_owned(),
            vec![
                TranscriptionSegmentInput {
                    start_ms: 100,
                    end_ms: 500,
                    text: "First".to_owned(),
                    speaker_label: Some("A".to_owned()),
                },
                TranscriptionSegmentInput {
                    start_ms: 600,
                    end_ms: 900,
                    text: "Second".to_owned(),
                    speaker_label: Some("B".to_owned()),
                },
                TranscriptionSegmentInput {
                    start_ms: 1_000,
                    end_ms: 1_400,
                    text: "Third".to_owned(),
                    speaker_label: Some("A".to_owned()),
                },
            ],
            None,
            serde_json::Value::Null,
        );

        assert_eq!(bundle.transcript.speakers.len(), 2);
        assert_eq!(bundle.transcript.speakers[0].display_name, "Speaker A");
        assert_eq!(
            bundle.transcript.speakers[0].source,
            SpeakerSource::Diarized
        );
        assert_eq!(
            bundle.transcript.segments[0].speaker_id,
            bundle.transcript.segments[2].speaker_id
        );
        assert_ne!(
            bundle.transcript.segments[0].speaker_id,
            bundle.transcript.segments[1].speaker_id
        );
    }

    #[test]
    fn labels_system_only_transcripts_as_system_audio() {
        let input = TranscriptionInput {
            session: Session::new(
                "Playback capture".to_owned(),
                None,
                SessionSource::Recording,
            ),
            audio_files: Vec::new(),
            artifact_kind: Some(ArtifactKind::System),
        };
        let bundle = build_bundle(
            input,
            TranscriptSource::OpenAi,
            "gpt-transcribe".to_owned(),
            vec![TranscriptionSegmentInput {
                start_ms: 0,
                end_ms: 1_000,
                text: "System playback".to_owned(),
                speaker_label: None,
            }],
            None,
            serde_json::Value::Null,
        );

        assert_eq!(bundle.transcript.speakers[0].display_name, "System audio");
        assert_eq!(bundle.transcript.speakers[0].source, SpeakerSource::System);
        assert_eq!(
            bundle.transcript.segments[0].source,
            opentranscribe_domain::AudioSource::System
        );
    }
}
