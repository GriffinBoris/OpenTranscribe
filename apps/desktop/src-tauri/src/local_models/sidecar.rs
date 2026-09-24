use opentranscribe_transcriber_protocol::{
    Command, Envelope, Event, FileTranscription, ModelDescriptor, PROTOCOL_VERSION, decode_frame,
    write_frame,
};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::error::{AppError, AppResult};
use crate::transcription::TranscriptionSegmentInput;

pub(super) async fn run_sidecar(
    app: &tauri::AppHandle,
    model_id: &str,
    model_path: &std::path::Path,
    request: FileTranscription,
    mut on_progress: impl FnMut(u64, u64),
    should_cancel: impl Fn() -> bool,
) -> AppResult<Vec<TranscriptionSegmentInput>> {
    let request_id = opentranscribe_domain::new_id();
    let mut stdin = Vec::new();
    write_command(&mut stdin, opentranscribe_domain::new_id(), Command::Hello)?;
    write_command(
        &mut stdin,
        opentranscribe_domain::new_id(),
        Command::LoadModel(ModelDescriptor {
            model_id: model_id.to_owned(),
            path: model_path.to_string_lossy().into_owned(),
            use_gpu: cfg!(target_os = "macos"),
        }),
    )?;
    write_command(
        &mut stdin,
        request_id,
        Command::TranscribeFile(request.clone()),
    )?;
    write_command(
        &mut stdin,
        opentranscribe_domain::new_id(),
        Command::Shutdown,
    )?;

    let (mut receiver, mut child) = app
        .shell()
        .sidecar("local-transcriber")
        .map_err(|error| AppError::Model(error.to_string()))?
        .set_raw_out(true)
        .spawn()
        .map_err(|error| AppError::Model(error.to_string()))?;
    if let Err(error) = child.write(&stdin) {
        let _ = child.kill();
        return Err(AppError::Model(error.to_string()));
    }

    let result = collect_output(
        &mut receiver,
        &request.job_id,
        &mut on_progress,
        should_cancel,
    )
    .await;
    if result.is_err() {
        let _ = child.kill();
    }
    result
}

async fn collect_output(
    receiver: &mut tauri::async_runtime::Receiver<CommandEvent>,
    job_id: &str,
    on_progress: &mut impl FnMut(u64, u64),
    should_cancel: impl Fn() -> bool,
) -> AppResult<Vec<TranscriptionSegmentInput>> {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut segments = Vec::new();
    let mut completed = false;
    let mut exit_code = None;

    let mut cancellation = tokio::time::interval(std::time::Duration::from_millis(50));
    loop {
        if should_cancel() {
            return Err(AppError::JobCanceled);
        }

        let event = tokio::select! {
            _ = cancellation.tick() => continue,
            event = receiver.recv() => match event {
                Some(event) => event,
                None => break,
            },
        };

        match event {
            CommandEvent::Stdout(bytes) => {
                stdout.extend(bytes);

                while let Some(envelope) =
                    decode_frame::<Envelope<Event>>(&mut stdout).map_err(protocol_error)?
                {
                    if envelope.protocol_version != PROTOCOL_VERSION {
                        return Err(AppError::Model(
                            "The desktop and transcriber protocol versions differ.".to_owned(),
                        ));
                    }
                    handle_sidecar_event(
                        envelope.body,
                        job_id,
                        &mut segments,
                        &mut completed,
                        on_progress,
                    )?;
                }
            }
            CommandEvent::Stderr(bytes) => {
                if stderr.len() < 8_192 {
                    stderr.extend(bytes.into_iter().take(8_192 - stderr.len()));
                }
            }
            CommandEvent::Error(message) => return Err(AppError::Model(message)),
            CommandEvent::Terminated(payload) => exit_code = payload.code,
            _ => {}
        }
    }

    if should_cancel() {
        return Err(AppError::JobCanceled);
    }

    if exit_code != Some(0) {
        let details = String::from_utf8_lossy(&stderr);
        return Err(AppError::Model(format!(
            "local transcriber exited with code {exit_code:?}: {}",
            details.trim()
        )));
    }

    if !completed {
        return Err(AppError::Model(
            "local transcriber exited before completing the job".to_owned(),
        ));
    }

    Ok(segments)
}

fn handle_sidecar_event(
    event: Event,
    job_id: &str,
    segments: &mut Vec<TranscriptionSegmentInput>,
    completed: &mut bool,
    on_progress: &mut impl FnMut(u64, u64),
) -> AppResult<()> {
    match event {
        Event::Segment {
            job_id: event_job_id,
            start_ms,
            end_ms,
            text,
            speaker_label,
        } if event_job_id == job_id => {
            segments.push(TranscriptionSegmentInput {
                start_ms,
                end_ms,
                text,
                speaker_label,
            });
        }
        Event::JobProgress {
            job_id: event_job_id,
            completed_ms,
            total_ms,
        } if event_job_id == job_id => on_progress(completed_ms, total_ms),
        Event::Completed {
            job_id: event_job_id,
        } if event_job_id == job_id => *completed = true,
        Event::Error { message, .. } => return Err(AppError::Model(message)),
        _ => {}
    }

    Ok(())
}

fn write_command(writer: &mut Vec<u8>, request_id: String, command: Command) -> AppResult<()> {
    write_frame(
        writer,
        &Envelope {
            protocol_version: PROTOCOL_VERSION,
            request_id,
            body: command,
        },
    )
    .map_err(protocol_error)
}

fn protocol_error(error: opentranscribe_transcriber_protocol::FrameError) -> AppError {
    AppError::Model(error.to_string())
}

#[cfg(test)]
mod tests {
    use opentranscribe_transcriber_protocol::Event;

    use super::{collect_output, handle_sidecar_event};
    use crate::error::AppError;

    #[test]
    fn consumes_the_local_sidecar_file_transcription_contract() {
        let mut segments = Vec::new();
        let mut completed = false;
        let mut progress = Vec::new();

        handle_sidecar_event(
            Event::JobProgress {
                job_id: "job".to_owned(),
                completed_ms: 500,
                total_ms: 1_000,
            },
            "job",
            &mut segments,
            &mut completed,
            &mut |completed, total| progress.push((completed, total)),
        )
        .expect("progress should be accepted");
        handle_sidecar_event(
            Event::Segment {
                job_id: "job".to_owned(),
                start_ms: 0,
                end_ms: 1_000,
                text: "Testing the local contract.".to_owned(),
                speaker_label: Some("1".to_owned()),
            },
            "job",
            &mut segments,
            &mut completed,
            &mut |_, _| {},
        )
        .expect("final segment should be accepted");
        handle_sidecar_event(
            Event::Completed {
                job_id: "job".to_owned(),
            },
            "job",
            &mut segments,
            &mut completed,
            &mut |_, _| {},
        )
        .expect("completion should be accepted");

        assert_eq!(progress, vec![(500, 1_000)]);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].start_ms, 0);
        assert_eq!(segments[0].end_ms, 1_000);
        assert_eq!(segments[0].text, "Testing the local contract.");
        assert_eq!(segments[0].speaker_label.as_deref(), Some("1"));
        assert!(completed);
    }

    #[test]
    fn ignores_local_sidecar_events_for_another_job() {
        let mut segments = Vec::new();
        let mut completed = false;
        let mut progress = Vec::new();

        for event in [
            Event::JobProgress {
                job_id: "other".to_owned(),
                completed_ms: 500,
                total_ms: 1_000,
            },
            Event::Segment {
                job_id: "other".to_owned(),
                start_ms: 0,
                end_ms: 1_000,
                text: "Wrong job".to_owned(),
                speaker_label: None,
            },
            Event::Completed {
                job_id: "other".to_owned(),
            },
        ] {
            handle_sidecar_event(
                event,
                "job",
                &mut segments,
                &mut completed,
                &mut |completed, total| progress.push((completed, total)),
            )
            .expect("unrelated event should be ignored");
        }

        assert!(progress.is_empty());
        assert!(segments.is_empty());
        assert!(!completed);
    }

    #[test]
    fn surfaces_local_sidecar_errors() {
        let mut segments = Vec::new();
        let mut completed = false;
        let error = handle_sidecar_event(
            Event::Error {
                code: "model_load_failed".to_owned(),
                message: "model could not be loaded".to_owned(),
            },
            "job",
            &mut segments,
            &mut completed,
            &mut |_, _| {},
        )
        .expect_err("sidecar error should fail the provider");

        assert!(error.to_string().contains("model could not be loaded"));
    }
    #[tokio::test]
    async fn cancels_even_when_inference_emits_no_events() {
        let (_sender, mut receiver) = tauri::async_runtime::channel(1);
        let started = std::time::Instant::now();
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            collect_output(&mut receiver, "job", &mut |_, _| {}, || {
                started.elapsed().as_millis() >= 60
            }),
        )
        .await
        .expect("cancellation must not wait for sidecar output");
        assert!(matches!(result, Err(AppError::JobCanceled)));
    }

    #[tokio::test]
    async fn rejects_exit_without_completed_output() {
        let (sender, mut receiver) = tauri::async_runtime::channel(1);
        sender
            .send(tauri_plugin_shell::process::CommandEvent::Terminated(
                tauri_plugin_shell::process::TerminatedPayload {
                    code: Some(0),
                    signal: None,
                },
            ))
            .await
            .expect("termination event");
        drop(sender);
        let result = collect_output(&mut receiver, "job", &mut |_, _| {}, || false).await;
        assert!(
            matches!(result, Err(AppError::Model(message)) if message.contains("before completing"))
        );
    }
}
