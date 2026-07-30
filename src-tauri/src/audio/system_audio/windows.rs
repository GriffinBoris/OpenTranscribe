use std::path::PathBuf;

use cpal::StreamConfig;
use cpal::traits::{DeviceTrait, HostTrait};
use opentranscribe_domain::CaptureDevice;

use crate::audio::cpal_capture::CpalCapture;
use crate::audio::packet_writer::CaptureSignals;
use crate::error::{AppError, AppResult};

pub fn availability() -> bool {
    cpal::default_host().default_output_device().is_some()
}

pub struct SystemAudioCapture {
    pub device: CaptureDevice,
    capture: CpalCapture,
}

impl SystemAudioCapture {
    pub fn start(recovery_directory: PathBuf, signals: CaptureSignals) -> AppResult<Self> {
        let output = cpal::default_host()
            .default_output_device()
            .ok_or_else(|| AppError::Audio("no default system output is available".to_owned()))?;
        let supported_config = output.default_output_config().map_err(audio_error)?;
        let sample_format = supported_config.sample_format();
        let config: StreamConfig = supported_config.into();
        let device = CaptureDevice {
            stable_id: output.id().map_err(audio_error)?.to_string(),
            label: output.description().map_err(audio_error)?.name().to_owned(),
            sample_rate_hz: config.sample_rate,
            channels: config.channels,
        };
        let capture = CpalCapture::start(
            output,
            config,
            sample_format,
            recovery_directory,
            "system",
            signals,
        )?;

        Ok(Self { device, capture })
    }

    pub fn stop(self) -> AppResult<()> {
        self.capture.stop()
    }
}

fn audio_error(error: impl std::fmt::Display) -> AppError {
    AppError::Audio(error.to_string())
}
