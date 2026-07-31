use std::path::Path;

use opentranscribe_transcriber_protocol::FileTranscription;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::audio::decode_for_whisper;

pub struct Segment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

#[derive(Default)]
pub struct TranscriberRuntime {
    context: Option<WhisperContext>,
    model_id: Option<String>,
}

impl TranscriberRuntime {
    pub fn load_model(&mut self, model_id: String, path: &Path) -> Result<(), String> {
        let context = WhisperContext::new_with_params(
            path.to_string_lossy().as_ref(),
            WhisperContextParameters::default(),
        )
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

        Ok((segments, audio.duration_ms))
    }
}
