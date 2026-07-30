mod audio;
mod runtime;

use std::io::BufReader;
use std::path::Path;

use opentranscribe_transcriber_protocol::{
    Command, Envelope, Event, FileTranscription, PROTOCOL_VERSION, read_frame, write_frame,
};

use runtime::TranscriberRuntime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut runtime = TranscriberRuntime::default();

    loop {
        let envelope: Envelope<Command> = read_frame(&mut reader)?;

        if envelope.protocol_version != PROTOCOL_VERSION {
            write_event(
                envelope.request_id,
                Event::Error {
                    code: "unsupported_protocol".to_owned(),
                    message: "The desktop and transcriber protocol versions differ.".to_owned(),
                },
            )?;
            continue;
        }

        match envelope.body {
            Command::Hello => write_event(
                envelope.request_id,
                Event::Ready {
                    runtime_version: env!("CARGO_PKG_VERSION").to_owned(),
                },
            )?,
            Command::LoadModel(model) => {
                let request_id = envelope.request_id;
                let event = match runtime.load_model(model.model_id.clone(), Path::new(&model.path))
                {
                    Ok(()) => Event::Status {
                        model_id: Some(model.model_id),
                        backlog_ms: 0,
                    },
                    Err(message) => Event::Error {
                        code: "model_load_failed".to_owned(),
                        message,
                    },
                };
                write_event(request_id, event)?;
            }
            Command::TranscribeFile(request) => {
                transcribe_file(&runtime, envelope.request_id, request)?;
            }
            Command::UnloadModel => {
                runtime.unload_model();
                write_event(
                    envelope.request_id,
                    Event::Status {
                        model_id: runtime.model_id(),
                        backlog_ms: 0,
                    },
                )?;
            }
            Command::AudioChunk(chunk) => write_event(
                envelope.request_id,
                Event::ChunkAck {
                    stream_id: chunk.stream_id,
                    sequence: chunk.sequence,
                },
            )?,
            Command::Shutdown => break,
            _ => write_event(
                envelope.request_id,
                Event::Error {
                    code: "live_transcription_not_available".to_owned(),
                    message: "Live local transcription is not available in this build.".to_owned(),
                },
            )?,
        }
    }

    Ok(())
}

fn transcribe_file(
    runtime: &TranscriberRuntime,
    request_id: String,
    request: FileTranscription,
) -> Result<(), Box<dyn std::error::Error>> {
    let progress_request_id = request_id.clone();
    let progress_job_id = request.job_id.clone();
    let result = runtime.transcribe(&request, move |completed_ms, total_ms| {
        let _ = write_event(
            progress_request_id.clone(),
            Event::JobProgress {
                job_id: progress_job_id.clone(),
                completed_ms,
                total_ms,
            },
        );
    });

    match result {
        Ok((segments, duration_ms)) => {
            for segment in segments {
                write_event(
                    request_id.clone(),
                    Event::Final {
                        stream_id: request.job_id.clone(),
                        start_ms: segment.start_ms,
                        end_ms: segment.end_ms,
                        text: segment.text,
                    },
                )?;
            }

            write_event(
                request_id.clone(),
                Event::JobProgress {
                    job_id: request.job_id.clone(),
                    completed_ms: duration_ms,
                    total_ms: duration_ms,
                },
            )?;
            write_event(
                request_id,
                Event::Completed {
                    job_id: request.job_id,
                },
            )?;
        }
        Err(message) => write_event(
            request_id,
            Event::Error {
                code: "transcription_failed".to_owned(),
                message,
            },
        )?,
    }

    Ok(())
}

fn write_event(
    request_id: String,
    event: Event,
) -> Result<(), opentranscribe_transcriber_protocol::FrameError> {
    write_frame(
        &mut std::io::stdout().lock(),
        &Envelope {
            protocol_version: PROTOCOL_VERSION,
            request_id,
            body: event,
        },
    )
}
