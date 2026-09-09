use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use opentranscribe_transcriber_protocol::{
    Command, Envelope, Event, ModelDescriptor, PROTOCOL_VERSION, decode_frame, write_frame,
};
use tauri::async_runtime::Receiver;
use tauri_plugin_shell::{
    ShellExt,
    process::{CommandChild, CommandEvent},
};

use crate::error::{AppError, AppResult};

#[derive(Default)]
pub(crate) struct DictationWorkers {
    pub speech: Worker,
    pub cleanup: Worker,
}

#[derive(Default)]
pub(crate) struct Worker {
    process: Option<Process>,
}

struct Process {
    child: Option<CommandChild>,
    receiver: Receiver<CommandEvent>,
    stdout: Vec<u8>,
    model_path: Option<String>,
}

impl Drop for Process {
    fn drop(&mut self) {
        if let Some(child) = self.child.take() {
            let _ = child.kill();
        }
    }
}

impl Worker {
    pub fn unload(&mut self) {
        self.process = None;
    }

    pub async fn prepare(
        &mut self,
        app: &tauri::AppHandle,
        executable: &str,
        model_id: &str,
        path: &Path,
        canceled: &AtomicBool,
    ) -> AppResult<()> {
        if self.process.is_none() {
            let (receiver, child) = app
                .shell()
                .sidecar(executable)
                .map_err(|error| AppError::Model(error.to_string()))?
                .set_raw_out(true)
                .spawn()
                .map_err(|error| AppError::Model(error.to_string()))?;
            self.process = Some(Process {
                child: Some(child),
                receiver,
                stdout: Vec::new(),
                model_path: None,
            });
            self.request(Command::Hello, canceled, |_| {}).await?;
        }
        let path = path.to_string_lossy().into_owned();
        if self
            .process
            .as_ref()
            .and_then(|process| process.model_path.as_ref())
            != Some(&path)
        {
            self.request(
                Command::LoadModel(ModelDescriptor {
                    model_id: model_id.to_owned(),
                    path: path.clone(),
                    use_gpu: cfg!(target_os = "macos"),
                }),
                canceled,
                |_| {},
            )
            .await?;
            self.process.as_mut().expect("prepared process").model_path = Some(path);
        }
        Ok(())
    }

    pub async fn request(
        &mut self,
        command: Command,
        canceled: &AtomicBool,
        mut on_event: impl FnMut(&Event),
    ) -> AppResult<Vec<Event>> {
        let result = self
            .process
            .as_mut()
            .ok_or_else(|| AppError::Model("The local runtime is not ready.".to_owned()))?
            .request(command, canceled, &mut on_event)
            .await;
        if result.is_err() {
            self.unload();
        }
        result
    }
}

impl Process {
    async fn request(
        &mut self,
        command: Command,
        canceled: &AtomicBool,
        on_event: &mut impl FnMut(&Event),
    ) -> AppResult<Vec<Event>> {
        let request_id = opentranscribe_domain::new_id();
        let mut bytes = Vec::new();
        write_frame(
            &mut bytes,
            &Envelope {
                protocol_version: PROTOCOL_VERSION,
                request_id: request_id.clone(),
                body: command,
            },
        )
        .map_err(|error| AppError::Model(error.to_string()))?;
        self.child
            .as_mut()
            .expect("live child")
            .write(&bytes)
            .map_err(|error| AppError::Model(error.to_string()))?;
        let mut events = Vec::new();
        let mut cancellation = tokio::time::interval(Duration::from_millis(25));
        loop {
            if canceled.load(Ordering::SeqCst) {
                return Err(AppError::JobCanceled);
            }
            tokio::select! {
                _ = cancellation.tick() => {},
                event = self.receiver.recv() => {
                    match event {
                        Some(CommandEvent::Stdout(bytes)) => {
                            self.stdout.extend(bytes);
                            while let Some(envelope) = decode_frame::<Envelope<Event>>(&mut self.stdout).map_err(|error| AppError::Model(error.to_string()))? {
                                if envelope.protocol_version != PROTOCOL_VERSION || envelope.request_id != request_id {
                                    return Err(AppError::Model("Unexpected local runtime response.".to_owned()));
                                }
                                if let Event::Error { message, .. } = envelope.body { return Err(AppError::Model(message)); }
                                let complete = matches!(envelope.body, Event::Ready { .. } | Event::Status { .. } | Event::Completed { .. } | Event::NormalizedText { .. });
                                on_event(&envelope.body);
                                events.push(envelope.body);
                                if complete { return Ok(events); }
                            }
                        },
                        Some(CommandEvent::Stderr(_)) => {},
                        Some(CommandEvent::Error(message)) => return Err(AppError::Model(message)),
                        Some(CommandEvent::Terminated(_)) | None => return Err(AppError::Model("The local runtime stopped unexpectedly. Try again.".to_owned())),
                        _ => {},
                    }
                }
            }
        }
    }
}
