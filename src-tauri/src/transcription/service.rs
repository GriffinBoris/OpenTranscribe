use opentranscribe_domain::TranscriptSource;

use crate::error::{AppError, AppResult};
use crate::storage::TranscriptionInput;

use super::bundle::{TranscriptionBundle, TranscriptionSegmentInput, build_bundle};
use super::openai::OpenAiChunkTranscript;
use super::{OpenAiFileTranscriber, estimate_openai_cost, openai_usage};

pub struct OpenAiTranscriptionService;

impl OpenAiTranscriptionService {
    pub fn run(
        input: TranscriptionInput,
        api_key: &str,
        model: String,
        live_stream_count: u8,
        on_progress: impl FnMut(usize, usize),
        should_cancel: impl Fn() -> bool,
    ) -> AppResult<TranscriptionBundle> {
        let language_hint = input.session.language_hint.clone();
        let transcripts = OpenAiFileTranscriber::new().transcribe(
            api_key,
            &model,
            &input.audio_files,
            language_hint.as_deref(),
            on_progress,
            should_cancel,
        )?;
        let segments = transcription_segments(&transcripts)?;
        let detected_language = transcripts
            .iter()
            .flat_map(|chunk| chunk.languages.iter())
            .next()
            .map(|language| language.code.clone());
        let duration_ms = transcripts.iter().map(|chunk| chunk.duration_ms).sum();
        let usage = openai_usage(
            &model,
            duration_ms,
            live_stream_count,
            transcripts
                .iter()
                .filter_map(|chunk| chunk.usage.clone())
                .collect(),
        );
        let approximate_cost_usd = estimate_openai_cost(&model, duration_ms, live_stream_count);
        let provider_response = serde_json::to_value(&transcripts)?;

        let mut bundle = build_bundle(
            input,
            TranscriptSource::OpenAi,
            model,
            segments,
            detected_language,
            provider_response,
        );
        bundle.run.usage = Some(usage);
        bundle.run.approximate_cost_usd = approximate_cost_usd;
        Ok(bundle)
    }
}

fn transcription_segments(
    transcripts: &[OpenAiChunkTranscript],
) -> AppResult<Vec<TranscriptionSegmentInput>> {
    let mut chunk_start_ms = 0_u64;
    let mut segments = Vec::new();

    for (chunk_index, chunk) in transcripts.iter().enumerate() {
        if chunk.segments.is_empty() {
            if !chunk.text.trim().is_empty() {
                segments.push(TranscriptionSegmentInput {
                    start_ms: chunk_start_ms,
                    end_ms: chunk_start_ms + chunk.duration_ms,
                    text: chunk.text.clone(),
                    speaker_label: None,
                });
            }
        } else {
            for segment in &chunk.segments {
                if segment.text.trim().is_empty() {
                    continue;
                }

                let speaker_label = if transcripts.len() == 1 {
                    segment.speaker.clone()
                } else {
                    format!("{} · part {}", segment.speaker, chunk_index + 1)
                };
                segments.push(TranscriptionSegmentInput {
                    start_ms: chunk_start_ms + seconds_to_milliseconds(segment.start),
                    end_ms: chunk_start_ms + seconds_to_milliseconds(segment.end),
                    text: segment.text.clone(),
                    speaker_label: Some(speaker_label),
                });
            }
        }

        chunk_start_ms += chunk.duration_ms;
    }

    if segments.is_empty() {
        return Err(AppError::Provider(
            "OpenAI returned an empty transcript".to_owned(),
        ));
    }

    Ok(segments)
}

fn seconds_to_milliseconds(seconds: f64) -> u64 {
    (seconds * 1_000.0).round() as u64
}

#[cfg(test)]
mod tests {
    use super::super::openai::{OpenAiChunkTranscript, OpenAiDiarizedSegment};
    use super::transcription_segments;

    #[test]
    fn offsets_diarized_segments_across_upload_chunks() {
        let transcripts = vec![
            OpenAiChunkTranscript {
                text: "First speaker".to_owned(),
                usage: None,
                languages: Vec::new(),
                segments: vec![OpenAiDiarizedSegment {
                    start: 0.1,
                    end: 0.6,
                    text: "First speaker".to_owned(),
                    speaker: "A".to_owned(),
                }],
                duration_ms: 1_000,
            },
            OpenAiChunkTranscript {
                text: "Same provider label, separate upload".to_owned(),
                usage: None,
                languages: Vec::new(),
                segments: vec![OpenAiDiarizedSegment {
                    start: 0.2,
                    end: 0.8,
                    text: "Same provider label, separate upload".to_owned(),
                    speaker: "A".to_owned(),
                }],
                duration_ms: 1_200,
            },
        ];

        let segments = transcription_segments(&transcripts)
            .expect("diarized segments should produce a transcript");

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].start_ms, 100);
        assert_eq!(segments[0].end_ms, 600);
        assert_eq!(segments[0].speaker_label.as_deref(), Some("A · part 1"));
        assert_eq!(segments[1].start_ms, 1_200);
        assert_eq!(segments[1].end_ms, 1_800);
        assert_eq!(segments[1].speaker_label.as_deref(), Some("A · part 2"));
    }

    #[test]
    fn keeps_text_only_responses_as_chunk_timed_fallbacks() {
        let transcripts = vec![OpenAiChunkTranscript {
            text: "Transcript text".to_owned(),
            usage: None,
            languages: Vec::new(),
            segments: Vec::new(),
            duration_ms: 2_500,
        }];

        let segments = transcription_segments(&transcripts)
            .expect("text-only responses should produce a transcript");

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].start_ms, 0);
        assert_eq!(segments[0].end_ms, 2_500);
        assert_eq!(segments[0].speaker_label, None);
    }

    #[test]
    fn rejects_empty_provider_responses() {
        let transcripts = vec![OpenAiChunkTranscript {
            text: String::new(),
            usage: None,
            languages: Vec::new(),
            segments: Vec::new(),
            duration_ms: 2_500,
        }];

        let error = match transcription_segments(&transcripts) {
            Ok(_) => panic!("empty responses should fail the transcription job"),
            Err(error) => error,
        };

        assert_eq!(
            error.to_string(),
            "provider request failed: OpenAI returned an empty transcript"
        );
    }
}
