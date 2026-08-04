use std::path::PathBuf;

use opentranscribe_domain::CaptureDevice;

use crate::audio::packet_writer::{AudioFormat, CaptureSignals};
use crate::error::{AppError, AppResult};

pub fn availability() -> bool {
    false
}

pub fn format() -> AppResult<AudioFormat> {
    Err(AppError::Audio(platform_message().to_owned()))
}

pub struct SystemAudioCapture {
    pub device: CaptureDevice,
}

impl SystemAudioCapture {
    pub fn start(_recovery_directory: PathBuf, _signals: CaptureSignals) -> AppResult<Self> {
        Err(AppError::Audio(platform_message().to_owned()))
    }

    pub fn stop(self) -> AppResult<()> {
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn platform_message() -> &'static str {
    "System output capture requires the WASAPI loopback adapter."
}

#[cfg(target_os = "linux")]
fn platform_message() -> &'static str {
    "System output capture requires the PipeWire adapter."
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn platform_message() -> &'static str {
    "System output capture is unavailable on this platform."
}
