use opentranscribe_domain::TranscriptSource;
use opentranscribe_transcriber_protocol::{Command, FileTranscription, ModelDescriptor};

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
        diarization_model_id: Option<String>,
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
        let definition =
            find(&model_id).ok_or_else(|| AppError::Model("unknown model".to_owned()))?;
        if matches!(definition.preset, "cleanup" | "diarization") {
            return Err(AppError::Model(
                "Select a speech model for transcription.".to_owned(),
            ));
        }
        let model_path = installed_path(app, &model_id)?;
        let diarization_model_path = diarization_model_id
            .as_deref()
            .map(|id| super::diarizer::model_path(app, id))
            .transpose()?
            .map(|path| path.to_string_lossy().into_owned());
        let job_id = opentranscribe_domain::new_id();
        let request = FileTranscription {
            job_id: job_id.clone(),
            path: audio_path.to_string_lossy().into_owned(),
            language_hint: input.session.language_hint.clone(),
            prompt: (!input.session.glossary.is_empty()).then(|| input.session.glossary.join(", ")),
            diarization_model_path,
        };
        let output = run_sidecar(
            app,
            Some(ModelDescriptor {
                model_id: model_id.clone(),
                path: model_path.to_string_lossy().into_owned(),
                use_gpu: cfg!(target_os = "macos"),
            }),
            &job_id,
            Command::TranscribeFile(request),
            |completed, total| {
                on_progress(completed, total);
            },
            should_cancel,
        )
        .await?;
        let segments = output.segments;
        let provider_response = serde_json::json!({
            "runtime": "whisper.cpp",
            "speech_model_id": model_id,
            "diarization_model_id": diarization_model_id,
            "diarization_sha256": diarization_model_id.as_deref().and_then(find).map(|model| model.sha256),
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
