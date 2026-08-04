#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::{SystemAudioCapture, availability, format};
#[cfg(target_os = "macos")]
pub use macos::{SystemAudioCapture, availability, format, permission_granted};
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub use unsupported::{SystemAudioCapture, availability, format};
#[cfg(target_os = "windows")]
pub use windows::{SystemAudioCapture, availability, format};

#[cfg(not(target_os = "macos"))]
pub fn permission_granted() -> Option<bool> {
    None
}
