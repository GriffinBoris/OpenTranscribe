use std::path::PathBuf;

use cpal::{SampleFormat, StreamConfig};

use super::cpal_capture::CpalCapture;
use super::packet_writer::CaptureSignals;
use crate::error::AppResult;

pub struct MicrophoneCapture {
    capture: CpalCapture,
}

impl MicrophoneCapture {
    pub fn start(
        device: cpal::Device,
        config: StreamConfig,
        sample_format: SampleFormat,
        recovery_directory: PathBuf,
        signals: CaptureSignals,
    ) -> AppResult<Self> {
        Ok(Self {
            capture: CpalCapture::start(
                device,
                config,
                sample_format,
                recovery_directory,
                "microphone",
                signals,
            )?,
        })
    }

    pub fn stop(self) -> AppResult<()> {
        self.capture.stop()
    }
}
