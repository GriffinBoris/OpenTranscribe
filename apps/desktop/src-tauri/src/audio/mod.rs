use std::fs;
use std::path::Path;

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
pub use waveform::{WAVEFORM_BUCKET_COUNT, waveform_peaks};

use crate::error::AppResult;

pub(crate) fn sync_and_rename(temporary_path: &Path, output_path: &Path) -> AppResult<()> {
    let file = fs::OpenOptions::new().write(true).open(temporary_path)?;
    file.sync_all()?;
    drop(file);
    fs::rename(temporary_path, output_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::sync_and_rename;

    #[test]
    fn closes_the_temporary_file_before_renaming_it() {
        let directory = tempdir().expect("temporary directory should exist");
        let temporary_path = directory.path().join("recording.partial.wav");
        let output_path = directory.path().join("recording.wav");
        std::fs::write(&temporary_path, b"audio").expect("temporary file should be written");

        sync_and_rename(&temporary_path, &output_path)
            .expect("temporary file should be renamed after syncing");

        assert_eq!(std::fs::read(output_path).unwrap(), b"audio");
    }
}
