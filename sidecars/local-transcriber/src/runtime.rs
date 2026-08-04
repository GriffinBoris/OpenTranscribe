use std::path::Path;

use opentranscribe_transcriber_protocol::FileTranscription;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::audio::decode_for_whisper;

pub struct Segment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

const MAX_SENTENCE_GAP_MS: u64 = 1_000;
const MAX_SENTENCE_DURATION_MS: u64 = 30_000;

#[derive(Default)]
pub struct TranscriberRuntime {
    context: Option<WhisperContext>,
    model_id: Option<String>,
}

impl TranscriberRuntime {
    pub fn load_model(
        &mut self,
        model_id: String,
        path: &Path,
        use_gpu: bool,
    ) -> Result<(), String> {
        let mut parameters = WhisperContextParameters::default();
        parameters.use_gpu(use_gpu);
        let context = WhisperContext::new_with_params(path.to_string_lossy().as_ref(), parameters)
            .map_err(|error| error.to_string())?;
        self.context = Some(context);
        self.model_id = Some(model_id);
        Ok(())
    }

    pub fn unload_model(&mut self) {
        self.context = None;
        self.model_id = None;
    }

    pub fn model_id(&self) -> Option<String> {
        self.model_id.clone()
    }

    pub fn transcribe<F>(
        &self,
        request: &FileTranscription,
        on_progress: F,
    ) -> Result<(Vec<Segment>, u64), String>
    where
        F: FnMut(u64, u64) + 'static,
    {
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| "load a model before transcribing".to_owned())?;
        let audio = decode_for_whisper(Path::new(&request.path))?;
        let mut state = context.create_state().map_err(|error| error.to_string())?;
        let mut parameters = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        let duration_ms = audio.duration_ms;
        let mut progress_callback = on_progress;
        let thread_count = std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1)
            .min(8) as i32;

        parameters.set_n_threads(thread_count);
        parameters.set_translate(false);
        parameters.set_language(request.language_hint.as_deref());
        parameters.set_print_special(false);
        parameters.set_print_progress(false);
        parameters.set_print_realtime(false);
        parameters.set_print_timestamps(false);
        let whisper_progress: Box<dyn FnMut(i32)> = Box::new(move |percent: i32| {
            progress_callback(
                duration_ms * percent.clamp(0, 100) as u64 / 100,
                duration_ms,
            );
        });
        parameters.set_progress_callback_safe::<_, Box<dyn FnMut(i32)>>(Some(whisper_progress));

        if let Some(prompt) = &request.prompt {
            parameters.set_initial_prompt(prompt);
        }

        state
            .full(parameters, &audio.samples)
            .map_err(|error| error.to_string())?;
        let mut segments = Vec::new();

        for segment in state.as_iter() {
            let text = segment
                .to_str_lossy()
                .map_err(|error| error.to_string())?
                .trim()
                .to_owned();

            if !text.is_empty() {
                segments.push(Segment {
                    start_ms: segment.start_timestamp().max(0) as u64 * 10,
                    end_ms: segment.end_timestamp().max(0) as u64 * 10,
                    text,
                });
            }
        }

        Ok((merge_sentence_fragments(segments), audio.duration_ms))
    }
}

fn merge_sentence_fragments(segments: Vec<Segment>) -> Vec<Segment> {
    let mut merged = Vec::new();

    for segment in segments {
        let should_merge = merged.last().is_some_and(|previous: &Segment| {
            !ends_sentence(&previous.text)
                && segment.start_ms.saturating_sub(previous.end_ms) <= MAX_SENTENCE_GAP_MS
                && segment.end_ms.saturating_sub(previous.start_ms) <= MAX_SENTENCE_DURATION_MS
        });

        if should_merge {
            let previous = merged.last_mut().expect("previous segment should exist");
            previous.text.push(' ');
            previous.text.push_str(&segment.text);
            previous.end_ms = segment.end_ms;
        } else {
            merged.push(segment);
        }
    }

    merged
}

fn ends_sentence(text: &str) -> bool {
    matches!(
        text.trim_end()
            .trim_end_matches(['"', '”', '’', ')', ']', '}'])
            .chars()
            .last(),
        Some('.' | '!' | '?' | '…' | '。' | '！' | '？')
    )
}

#[cfg(test)]
mod tests {
    use super::{Segment, merge_sentence_fragments};

    #[test]
    fn joins_adjacent_whisper_fragments_into_a_sentence() {
        let merged = merge_sentence_fragments(vec![
            Segment {
                start_ms: 0,
                end_ms: 700,
                text: "We should ship".to_owned(),
            },
            Segment {
                start_ms: 750,
                end_ms: 1_500,
                text: "the playback bar.".to_owned(),
            },
        ]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].start_ms, 0);
        assert_eq!(merged[0].end_ms, 1_500);
        assert_eq!(merged[0].text, "We should ship the playback bar.");
    }

    #[test]
    fn keeps_completed_sentences_and_distant_fragments_separate() {
        let merged = merge_sentence_fragments(vec![
            Segment {
                start_ms: 0,
                end_ms: 700,
                text: "First sentence.".to_owned(),
            },
            Segment {
                start_ms: 750,
                end_ms: 1_500,
                text: "Second sentence".to_owned(),
            },
            Segment {
                start_ms: 3_000,
                end_ms: 3_500,
                text: "after a pause.".to_owned(),
            },
        ]);

        assert_eq!(merged.len(), 3);
    }

    #[test]
    fn preserves_completed_quoted_and_multilingual_sentences() {
        let merged = merge_sentence_fragments(vec![
            Segment {
                start_ms: 0,
                end_ms: 700,
                text: "They said, \"ship it.\"".to_owned(),
            },
            Segment {
                start_ms: 750,
                end_ms: 1_500,
                text: "次の文です。".to_owned(),
            },
        ]);

        assert_eq!(merged.len(), 2);
    }
}
