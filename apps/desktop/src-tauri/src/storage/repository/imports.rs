use std::fs;
use std::path::Path;

use opentranscribe_domain::{
    Artifact, ArtifactKind, Codec, Session, SessionLifecycle, SessionSource,
};

use crate::audio::{extract_audio, waveform_peaks};
use crate::error::{AppError, AppResult};

use super::atomic_file;
use super::audio_artifact::hash_file;
use super::repository::{
    LibraryRepository, relative_path, session_directory_name, write_session_directory,
};

impl LibraryRepository {
    pub fn import_media(&self, source_path: &Path) -> AppResult<Session> {
        let title = source_path
            .file_stem()
            .ok_or(AppError::InvalidName)?
            .to_string_lossy()
            .into_owned();
        let extension = source_path
            .extension()
            .map(|extension| extension.to_string_lossy().to_lowercase())
            .unwrap_or_else(|| "media".to_owned());
        let mut session = Session::new(title, None, SessionSource::Import);
        let directory_name = session_directory_name(&session);
        let destination = self.root.join("Inbox").join(&directory_name);
        let temporary_directory = tempfile::tempdir_in(self.root.join(".opentranscribe"))?;
        let staged_directory = temporary_directory.path().join(&directory_name);

        write_session_directory(&staged_directory, &session)?;
        let original_name = format!("original.{extension}");
        let staged_original = staged_directory.join("audio").join(&original_name);
        let staged_audio = staged_directory.join("audio/imported.wav");
        fs::copy(source_path, &staged_original)?;
        let imported_audio = extract_audio(&staged_original, &staged_audio)?;
        let original_destination = destination.join("audio").join(&original_name);
        let audio_destination = destination.join("audio/imported.wav");
        let original_artifact = Artifact {
            id: opentranscribe_domain::new_id(),
            kind: ArtifactKind::ImportedOriginal,
            relative_path: relative_path(&self.root, &original_destination),
            codec: Codec::Original,
            sample_rate_hz: None,
            channels: None,
            duration_ms: None,
            byte_count: fs::metadata(&staged_original)?.len(),
            sha256: hash_file(&staged_original)?,
        };
        let imported_artifact = Artifact {
            id: opentranscribe_domain::new_id(),
            kind: ArtifactKind::ImportedAudio,
            relative_path: relative_path(&self.root, &audio_destination),
            codec: Codec::WavPcm24,
            sample_rate_hz: Some(48_000),
            channels: Some(2),
            duration_ms: Some(imported_audio.duration_ms),
            byte_count: fs::metadata(&staged_audio)?.len(),
            sha256: hash_file(&staged_audio)?,
        };
        let staged_waveform = staged_directory.join("audio/waveform.json");
        atomic_file::write_json(&staged_waveform, &waveform_peaks(&staged_audio, 160)?)?;
        let waveform_destination = destination.join("audio/waveform.json");
        let waveform_artifact = Artifact {
            id: opentranscribe_domain::new_id(),
            kind: ArtifactKind::Waveform,
            relative_path: relative_path(&self.root, &waveform_destination),
            codec: Codec::Json,
            sample_rate_hz: None,
            channels: None,
            duration_ms: Some(imported_audio.duration_ms),
            byte_count: fs::metadata(&staged_waveform)?.len(),
            sha256: hash_file(&staged_waveform)?,
        };

        session.lifecycle = SessionLifecycle::Ready;
        session.duration_ms = imported_audio.duration_ms;
        session.artifacts.push(original_artifact);
        session.artifacts.push(imported_artifact);
        session.artifacts.push(waveform_artifact);
        atomic_file::write_json(&staged_directory.join("session.json"), &session)?;
        fs::rename(&staged_directory, &destination)?;
        self.index
            .replace_session(&session, &relative_path(&self.root, &destination))?;
        Ok(session)
    }
}
