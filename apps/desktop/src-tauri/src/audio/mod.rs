mod cpal_capture;
mod finalizer;
mod importer;
mod microphone;
mod mixer;
mod packet_writer;
mod recorder;
mod system_audio;
mod waveform;

pub use finalizer::{finalize_microphone_track, finalize_system_track};
pub use importer::extract_audio;
pub use mixer::mix_tracks;
pub use recorder::{
    AudioDevices, RecordingCapture, RecordingController, RecordingStatus, StartRecordingOptions,
};
pub use waveform::waveform_peaks;
