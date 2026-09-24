use opentranscribe_domain::TranscriptSource;
use opentranscribe_transcriber_protocol::FileTranscription;

use crate::error::{AppError, AppResult};
use crate::storage::TranscriptionInput;
use crate::transcription::{TranscriptionBundle, build_bundle};

use super::catalog::find;
use super::installed_path;
use super::sidecar::run_sidecar;

pub struct LocalTranscriptionService;

impl LocalTranscriptionService {
    pub async fn run(
        app: &tauri::AppHandle,
        input: TranscriptionInput,
        model_id: String,
        mut on_progress: impl FnMut(u64, u64),
        should_cancel: impl Fn() -> bool,
    ) -> AppResult<TranscriptionBundle> {
        let audio_path = input
            .audio_files
            .first()
            .ok_or_else(|| {
                AppError::Model("record or import audio before transcribing".to_owned())
            })?
            .clone();
        if model_id == "s1-mini-q4_k_m" {
            return Err(AppError::Model(
                "S1-mini cleans text; select a speech model for transcription.".to_owned(),
            ));
        }
        let definition =
            find(&model_id).ok_or_else(|| AppError::Model("unknown model".to_owned()))?;
        let installed = installed_path(app, &model_id)?;
        let (speech_model_id, model_path, diarization_model_path) = match definition.speech_model_id
        {
            Some(speech_model_id) => (
                speech_model_id,
                installed_path(app, speech_model_id)?,
                Some(installed.to_string_lossy().into_owned()),
            ),
            None => (model_id.as_str(), installed, None),
        };
        let job_id = opentranscribe_domain::new_id();
        let request = FileTranscription {
            job_id,
            path: audio_path.to_string_lossy().into_owned(),
            language_hint: input.session.language_hint.clone(),
            prompt: (!input.session.glossary.is_empty()).then(|| input.session.glossary.join(", ")),
            diarization_model_path,
        };
        let segments = run_sidecar(
            app,
            speech_model_id,
            &model_path,
            request,
            |completed, total| {
                on_progress(completed, total);
            },
            should_cancel,
        )
        .await?;
        let provider_response = serde_json::json!({
            "runtime": "whisper.cpp",
            "speech_model_id": speech_model_id,
            "diarization_model_id": definition.speech_model_id.map(|_| "nvidia/Nemotron-3-Diarization"),
            "diarization_sha256": definition.speech_model_id.map(|_| definition.sha256),
            "segments": segments.iter().map(|segment| serde_json::json!({
                "start_ms": segment.start_ms,
                "end_ms": segment.end_ms,
                "text": segment.text,
                "speaker_label": segment.speaker_label,
            })).collect::<Vec<_>>(),
        });

        Ok(build_bundle(
            input,
            TranscriptSource::Local,
            model_id,
            segments,
            None,
            provider_response,
        ))
    }
}
