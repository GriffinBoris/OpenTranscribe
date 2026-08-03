use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cpal::StreamConfig;
use cpal::traits::{DeviceTrait, HostTrait};
use opentranscribe_domain::CaptureDevice;
use serde::{Deserialize, Serialize};

use super::microphone::MicrophoneCapture;
use super::packet_writer::CaptureSignals;
use super::system_audio::{
    SystemAudioCapture, availability as system_audio_availability,
    permission_granted as system_audio_permission_granted,
};
use crate::error::{AppError, AppResult};
use crate::transcription::LiveAudioSink;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AudioDevice {
    pub id: String,
    pub label: String,
    pub is_default: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct AudioDevices {
    pub microphones: Vec<AudioDevice>,
    pub system_audio_available: bool,
    pub system_audio_permission_granted: Option<bool>,
    pub system_audio_permission_settings_available: bool,
}

#[derive(Clone)]
pub struct StartRecordingOptions {
    pub session_id: String,
    pub microphone_device_id: Option<String>,
    pub capture_system_audio: bool,
    pub microphone_live_audio: Option<LiveAudioSink>,
    pub system_live_audio: Option<LiveAudioSink>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecordingStatus {
    pub session_id: String,
    pub is_paused: bool,
    pub captures_system_audio: bool,
    pub elapsed_ms: u64,
    pub microphone_peak: f32,
    pub system_peak: f32,
    pub dropped_packets: u64,
}

#[derive(Clone, Debug)]
pub struct RecordingCapture {
    pub microphone: CaptureDevice,
    pub system_output: Option<CaptureDevice>,
}

struct ActiveRecording {
    session_id: String,
    captures_system_audio: bool,
    started: Instant,
    paused_at: Option<Instant>,
    paused_duration: Duration,
    paused: Arc<AtomicBool>,
    microphone_peak: Arc<AtomicU32>,
    system_peak: Arc<AtomicU32>,
    dropped_packets: Arc<AtomicU64>,
    streams: Box<dyn RecordingStreams>,
}

trait RecordingStreams: Send {
    fn stop(self: Box<Self>);
}

struct NativeRecordingStreams {
    microphone: MicrophoneCapture,
    system_audio: Option<SystemAudioCapture>,
}

impl RecordingStreams for NativeRecordingStreams {
    fn stop(self: Box<Self>) {
        if let Err(error) = self.microphone.stop() {
            log::warn!("microphone stopped with an error; recovery chunks were preserved: {error}");
        }

        if let Some(system_audio) = self.system_audio
            && let Err(error) = system_audio.stop()
        {
            log::warn!(
                "system audio stopped with an error; recovery chunks were preserved: {error}"
            );
        }
    }
}

#[derive(Default)]
pub struct RecordingController {
    active: Mutex<Option<ActiveRecording>>,
}

impl RecordingController {
    pub fn devices(&self) -> AppResult<AudioDevices> {
        let host = cpal::default_host();
        let default_id = host
            .default_input_device()
            .and_then(|device| device.id().ok())
            .map(|id| id.to_string());
        let microphones = host
            .input_devices()
            .map_err(audio_error)?
            .enumerate()
            .map(|(index, device)| {
                let label = device
                    .description()
                    .map(|description| description.name().to_owned())
                    .unwrap_or_else(|_| format!("Microphone {}", index + 1));
                let id = device
                    .id()
                    .map(|id| id.to_string())
                    .unwrap_or_else(|_| label.clone());
                AudioDevice {
                    is_default: default_id.as_ref() == Some(&id),
                    id,
                    label,
                }
            })
            .collect();

        Ok(AudioDevices {
            microphones,
            system_audio_available: system_audio_availability(),
            system_audio_permission_granted: system_audio_permission_granted(),
            system_audio_permission_settings_available: cfg!(target_os = "macos"),
        })
    }

    pub fn start(
        &self,
        options: StartRecordingOptions,
        recovery_directory: PathBuf,
    ) -> AppResult<RecordingCapture> {
        self.start_with(options, recovery_directory, start_native_streams)
    }

    fn start_with(
        &self,
        options: StartRecordingOptions,
        recovery_directory: PathBuf,
        start_streams: impl FnOnce(
            &StartRecordingOptions,
            PathBuf,
            CaptureSignals,
            CaptureSignals,
        ) -> AppResult<(RecordingCapture, Box<dyn RecordingStreams>)>,
    ) -> AppResult<RecordingCapture> {
        let mut active = self.active.lock().expect("recording lock poisoned");

        if active.is_some() {
            return Err(AppError::RecordingActive);
        }

        fs::create_dir_all(&recovery_directory)?;
        let paused = Arc::new(AtomicBool::new(true));
        let microphone_peak = Arc::new(AtomicU32::new(0));
        let system_peak = Arc::new(AtomicU32::new(0));
        let dropped_packets = Arc::new(AtomicU64::new(0));
        let microphone_signals = CaptureSignals {
            paused: Arc::clone(&paused),
            peak: Arc::clone(&microphone_peak),
            dropped_packets: Arc::clone(&dropped_packets),
            live_audio: options.microphone_live_audio.clone(),
        };
        let system_signals = CaptureSignals {
            paused: Arc::clone(&paused),
            peak: Arc::clone(&system_peak),
            dropped_packets: Arc::clone(&dropped_packets),
            live_audio: options.system_live_audio.clone(),
        };
        let (capture, streams) = start_streams(
            &options,
            recovery_directory,
            microphone_signals,
            system_signals,
        )?;
        paused.store(false, Ordering::Relaxed);

        *active = Some(ActiveRecording {
            session_id: options.session_id,
            captures_system_audio: capture.system_output.is_some(),
            started: Instant::now(),
            paused_at: None,
            paused_duration: Duration::ZERO,
            paused,
            microphone_peak,
            system_peak,
            dropped_packets,
            streams,
        });
        Ok(capture)
    }

    pub fn pause(&self, paused: bool) -> AppResult<RecordingStatus> {
        let mut active = self.active.lock().expect("recording lock poisoned");
        let active = active.as_mut().ok_or(AppError::RecordingInactive)?;

        if paused && active.paused_at.is_none() {
            active.paused_at = Some(Instant::now());
        }

        if !paused && let Some(paused_at) = active.paused_at.take() {
            active.paused_duration += paused_at.elapsed();
        }

        active.paused.store(paused, Ordering::Relaxed);
        Ok(status_for(active))
    }

    pub fn status(&self) -> Option<RecordingStatus> {
        self.active
            .lock()
            .expect("recording lock poisoned")
            .as_ref()
            .map(status_for)
    }

    pub fn stop(&self) -> AppResult<RecordingStatus> {
        let active = self
            .active
            .lock()
            .expect("recording lock poisoned")
            .take()
            .ok_or(AppError::RecordingInactive)?;
        let status = status_for(&active);
        active.streams.stop();

        Ok(status)
    }
}

fn start_native_streams(
    options: &StartRecordingOptions,
    recovery_directory: PathBuf,
    microphone_signals: CaptureSignals,
    system_signals: CaptureSignals,
) -> AppResult<(RecordingCapture, Box<dyn RecordingStreams>)> {
    let host = cpal::default_host();
    let device = select_input_device(&host, options.microphone_device_id.as_deref())?;
    let supported_config = device.default_input_config().map_err(audio_error)?;
    let sample_format = supported_config.sample_format();
    let config: StreamConfig = supported_config.into();
    let system_audio = options
        .capture_system_audio
        .then(|| SystemAudioCapture::start(recovery_directory.clone(), system_signals))
        .transpose()?;
    let system_output = system_audio.as_ref().map(|capture| capture.device.clone());
    let microphone_device = CaptureDevice {
        stable_id: device.id().map_err(audio_error)?.to_string(),
        label: device.description().map_err(audio_error)?.name().to_owned(),
        sample_rate_hz: config.sample_rate,
        channels: config.channels,
    };
    let microphone = match MicrophoneCapture::start(
        device,
        config,
        sample_format,
        recovery_directory,
        microphone_signals,
    ) {
        Ok(capture) => capture,
        Err(error) => {
            if let Some(system_audio) = system_audio
                && let Err(stop_error) = system_audio.stop()
            {
                log::warn!(
                    "system audio cleanup failed after microphone start failure: {stop_error}"
                );
            }
            return Err(error);
        }
    };

    Ok((
        RecordingCapture {
            microphone: microphone_device,
            system_output,
        },
        Box::new(NativeRecordingStreams {
            microphone,
            system_audio,
        }),
    ))
}

fn select_input_device(host: &cpal::Host, requested: Option<&str>) -> AppResult<cpal::Device> {
    if let Some(requested) = requested {
        return host
            .input_devices()
            .map_err(audio_error)?
            .find(|device| {
                device
                    .id()
                    .map(|id| id.to_string() == requested)
                    .unwrap_or(false)
            })
            .ok_or_else(|| {
                AppError::Audio(format!("microphone is no longer available: {requested}"))
            });
    }

    host.default_input_device()
        .ok_or_else(|| AppError::Audio("no default microphone is available".to_owned()))
}

fn status_for(active: &ActiveRecording) -> RecordingStatus {
    let elapsed = recorded_elapsed(
        active.started,
        active.paused_duration,
        active.paused_at,
        Instant::now(),
    );

    RecordingStatus {
        session_id: active.session_id.clone(),
        is_paused: active.paused.load(Ordering::Relaxed),
        captures_system_audio: active.captures_system_audio,
        elapsed_ms: elapsed.as_millis() as u64,
        microphone_peak: f32::from_bits(active.microphone_peak.load(Ordering::Relaxed)),
        system_peak: f32::from_bits(active.system_peak.load(Ordering::Relaxed)),
        dropped_packets: active.dropped_packets.load(Ordering::Relaxed),
    }
}

fn recorded_elapsed(
    started: Instant,
    paused_duration: Duration,
    paused_at: Option<Instant>,
    now: Instant,
) -> Duration {
    let current_pause = paused_at.map(|started| now - started).unwrap_or_default();
    (now - started).saturating_sub(paused_duration + current_pause)
}

fn audio_error(error: impl std::fmt::Display) -> AppError {
    AppError::Audio(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    use tempfile::tempdir;

    use super::{
        AppError, AppResult, CaptureDevice, RecordingCapture, RecordingController,
        RecordingStreams, StartRecordingOptions, recorded_elapsed,
    };

    struct TestStreams {
        stops: Arc<AtomicUsize>,
    }

    impl RecordingStreams for TestStreams {
        fn stop(self: Box<Self>) {
            self.stops.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn options(capture_system_audio: bool) -> StartRecordingOptions {
        StartRecordingOptions {
            session_id: "session-1".to_owned(),
            microphone_device_id: None,
            capture_system_audio,
            microphone_live_audio: None,
            system_live_audio: None,
        }
    }

    fn capture(captures_system_audio: bool) -> RecordingCapture {
        let device = CaptureDevice {
            stable_id: "microphone-1".to_owned(),
            label: "Test microphone".to_owned(),
            sample_rate_hz: 48_000,
            channels: 1,
        };

        RecordingCapture {
            microphone: device.clone(),
            system_output: captures_system_audio.then_some(device),
        }
    }

    fn start_test_recording(
        controller: &RecordingController,
        recovery_directory: PathBuf,
        capture_system_audio: bool,
        stops: Arc<AtomicUsize>,
    ) -> AppResult<RecordingCapture> {
        controller.start_with(
            options(capture_system_audio),
            recovery_directory,
            move |_, _, microphone_signals, system_signals| {
                microphone_signals
                    .peak
                    .store(0.75_f32.to_bits(), Ordering::Relaxed);
                system_signals
                    .peak
                    .store(0.5_f32.to_bits(), Ordering::Relaxed);
                microphone_signals
                    .dropped_packets
                    .store(3, Ordering::Relaxed);
                Ok((
                    capture(capture_system_audio),
                    Box::new(TestStreams { stops }),
                ))
            },
        )
    }

    #[test]
    fn excludes_completed_and_active_pause_time_from_recording_elapsed_time() {
        let now = Instant::now();
        let elapsed = recorded_elapsed(
            now - Duration::from_secs(100),
            Duration::from_secs(20),
            Some(now - Duration::from_secs(30)),
            now,
        );

        assert_eq!(elapsed, Duration::from_secs(50));
    }

    #[test]
    fn reports_levels_and_runs_the_full_pause_resume_stop_lifecycle_without_hardware() {
        let directory = tempdir().expect("temporary directory should exist");
        let controller = RecordingController::default();
        let stops = Arc::new(AtomicUsize::new(0));

        start_test_recording(
            &controller,
            directory.path().join("recovery"),
            true,
            Arc::clone(&stops),
        )
        .expect("test recording should start");

        let started = controller.status().expect("recording should be active");
        assert_eq!(started.session_id, "session-1");
        assert!(!started.is_paused);
        assert!(started.captures_system_audio);
        assert_eq!(started.microphone_peak, 0.75);
        assert_eq!(started.system_peak, 0.5);
        assert_eq!(started.dropped_packets, 3);

        assert!(
            controller
                .pause(true)
                .expect("recording should pause")
                .is_paused
        );
        assert!(
            !controller
                .pause(false)
                .expect("recording should resume")
                .is_paused
        );
        let stopped = controller.stop().expect("recording should stop");

        assert_eq!(stopped.session_id, "session-1");
        assert!(stopped.captures_system_audio);
        assert!(controller.status().is_none());
        assert_eq!(stops.load(Ordering::Relaxed), 1);
        assert!(matches!(
            controller.stop(),
            Err(AppError::RecordingInactive)
        ));
    }

    #[test]
    fn rejects_concurrent_starts_without_creating_a_second_capture() {
        let directory = tempdir().expect("temporary directory should exist");
        let controller = RecordingController::default();
        let stops = Arc::new(AtomicUsize::new(0));

        start_test_recording(
            &controller,
            directory.path().join("first"),
            false,
            Arc::clone(&stops),
        )
        .expect("first recording should start");

        let second_start_attempts = AtomicUsize::new(0);
        let error = controller
            .start_with(
                options(false),
                directory.path().join("second"),
                |_, _, _, _| {
                    second_start_attempts.fetch_add(1, Ordering::Relaxed);
                    Ok((capture(false), Box::new(TestStreams { stops })))
                },
            )
            .expect_err("second recording should be rejected");

        assert!(matches!(error, AppError::RecordingActive));
        assert_eq!(second_start_attempts.load(Ordering::Relaxed), 0);
        assert!(!directory.path().join("second").exists());
        controller.stop().expect("first recording should stop");
    }

    #[test]
    fn leaves_the_controller_available_after_capture_startup_fails() {
        let directory = tempdir().expect("temporary directory should exist");
        let controller = RecordingController::default();
        let recovery_directory = directory.path().join("recovery");

        let error = controller
            .start_with(options(false), recovery_directory.clone(), |_, _, _, _| {
                Err(AppError::Audio("test capture failed".to_owned()))
            })
            .expect_err("failed capture should be returned");

        assert!(matches!(error, AppError::Audio(message) if message == "test capture failed"));
        assert!(recovery_directory.is_dir());
        assert!(controller.status().is_none());

        start_test_recording(
            &controller,
            directory.path().join("retry"),
            false,
            Arc::new(AtomicUsize::new(0)),
        )
        .expect("controller should allow a retry after startup failure");
        controller.stop().expect("retry recording should stop");
    }
}
