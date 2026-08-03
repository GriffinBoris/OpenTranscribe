use std::cell::RefCell;
use std::convert::TryInto;
use std::mem;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{Sender, TryRecvError, bounded};
use opentranscribe_domain::CaptureDevice;
use pipewire as pw;
use pw::prelude::{ListenerBuilderT, WritableDict};
use pw::properties;

use crate::audio::packet_writer::{
    AudioFormat, AudioPacket, CaptureSignals, PACKET_QUEUE_CAPACITY, enqueue_samples, write_chunks,
};
use crate::error::{AppError, AppResult};

const SYSTEM_SAMPLE_RATE: u32 = 48_000;
const SYSTEM_CHANNELS: u16 = 2;

pub fn availability() -> bool {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .is_some_and(|directory| directory.join("pipewire-0").exists())
}

pub struct SystemAudioCapture {
    pub device: CaptureDevice,
    stop_sender: Sender<()>,
    capture_thread: JoinHandle<AppResult<()>>,
    writer_thread: JoinHandle<AppResult<()>>,
}

impl SystemAudioCapture {
    pub fn start(recovery_directory: PathBuf, signals: CaptureSignals) -> AppResult<Self> {
        let (packet_sender, packet_receiver) = bounded(PACKET_QUEUE_CAPACITY);
        let (stop_sender, stop_receiver) = bounded(1);
        let (ready_sender, ready_receiver) = bounded(1);
        let writer_thread = thread::spawn(move || {
            write_chunks(
                packet_receiver,
                &recovery_directory,
                "system",
                AudioFormat {
                    channels: SYSTEM_CHANNELS,
                    sample_rate: SYSTEM_SAMPLE_RATE,
                },
            )
        });
        let capture_thread =
            thread::spawn(move || run_capture(packet_sender, stop_receiver, ready_sender, signals));

        match ready_receiver.recv_timeout(Duration::from_secs(10)) {
            Ok(Ok(())) => Ok(Self {
                device: CaptureDevice {
                    stable_id: "pipewire-default-sink".to_owned(),
                    label: "PipeWire Default Output".to_owned(),
                    sample_rate_hz: SYSTEM_SAMPLE_RATE,
                    channels: SYSTEM_CHANNELS,
                },
                stop_sender,
                capture_thread,
                writer_thread,
            }),
            Ok(Err(message)) => {
                let _ = stop_sender.send(());
                let capture_result = join_capture(capture_thread, "PipeWire capture");
                let writer_result = join_capture(writer_thread, "system audio writer");
                writer_result?;

                if let Err(error) = capture_result {
                    log::debug!("PipeWire startup stopped with the reported error: {error}");
                }

                Err(AppError::Audio(message))
            }
            Err(error) => {
                let _ = stop_sender.send(());
                join_capture(capture_thread, "PipeWire capture")?;
                join_capture(writer_thread, "system audio writer")?;
                Err(AppError::Audio(format!(
                    "PipeWire system audio did not start in time: {error}"
                )))
            }
        }
    }

    pub fn stop(self) -> AppResult<()> {
        let _ = self.stop_sender.send(());
        join_capture(self.capture_thread, "PipeWire capture")?;
        join_capture(self.writer_thread, "system audio writer")
    }
}

struct UserData {
    packet_sender: Sender<AudioPacket>,
    signals: CaptureSignals,
}

fn run_capture(
    packet_sender: Sender<AudioPacket>,
    stop_receiver: crossbeam_channel::Receiver<()>,
    ready_sender: Sender<Result<(), String>>,
    signals: CaptureSignals,
) -> AppResult<()> {
    let mainloop = pw::MainLoop::new().map_err(pipewire_error)?;
    let mut props = properties! {
        *pw::keys::MEDIA_TYPE => "Audio",
        *pw::keys::MEDIA_CATEGORY => "Capture",
        *pw::keys::MEDIA_ROLE => "Communication",
    };
    props.insert(*pw::keys::STREAM_CAPTURE_SINK, "true");
    let ready_sender = Rc::new(RefCell::new(Some(ready_sender)));
    let state_ready_sender = Rc::clone(&ready_sender);
    let user_data = UserData {
        packet_sender,
        signals,
    };
    let stream = pw::stream::Stream::with_user_data(
        &mainloop,
        "OpenTranscribe system audio",
        props,
        user_data,
    )
    .state_changed(move |_, state| match state {
        pw::stream::StreamState::Streaming => {
            if let Some(sender) = state_ready_sender.borrow_mut().take() {
                let _ = sender.send(Ok(()));
            }
        }
        pw::stream::StreamState::Error(error) => {
            if let Some(sender) = state_ready_sender.borrow_mut().take() {
                let _ = sender.send(Err(format!("PipeWire stream failed: {error}")));
            }
        }
        _ => {}
    })
    .process(|stream, user_data| {
        let Some(mut buffer) = stream.dequeue_buffer() else {
            return;
        };
        let Some(data) = buffer.datas_mut().first_mut() else {
            return;
        };
        let sample_count = data.chunk().size() as usize / mem::size_of::<f32>();
        let Some(bytes) = data.data() else {
            return;
        };
        let samples = bytes
            .chunks_exact(mem::size_of::<f32>())
            .take(sample_count)
            .map(|sample| {
                f32::from_le_bytes(
                    sample
                        .try_into()
                        .expect("a four-byte chunk is a valid f32 sample"),
                )
            })
            .collect();
        enqueue_samples(
            samples,
            &user_data.packet_sender,
            &user_data.signals,
            AudioFormat {
                channels: SYSTEM_CHANNELS,
                sample_rate: SYSTEM_SAMPLE_RATE,
            },
        );
    })
    .create()
    .map_err(pipewire_error)?;
    stream
        .connect(
            pw::spa::Direction::Input,
            None,
            pw::stream::StreamFlags::AUTOCONNECT
                | pw::stream::StreamFlags::MAP_BUFFERS
                | pw::stream::StreamFlags::RT_PROCESS,
            &mut [],
        )
        .map_err(pipewire_error)?;

    while matches!(stop_receiver.try_recv(), Err(TryRecvError::Empty)) {
        mainloop.iterate(Duration::from_millis(50));
    }

    Ok(())
}

fn pipewire_error(error: impl std::fmt::Display) -> AppError {
    AppError::Audio(format!("PipeWire system audio failed: {error}"))
}

fn join_capture(handle: JoinHandle<AppResult<()>>, description: &str) -> AppResult<()> {
    handle
        .join()
        .map_err(|_| AppError::Audio(format!("{description} thread crashed")))?
}
