use std::fs;
use std::path::PathBuf;

use opentranscribe_domain::{ArtifactKind, RecoveryState, Session, SessionLifecycle};

use super::atomic_file;
use super::audio_artifact::{
    SessionAudioSource, audio_sources, has_valid_microphone_artifact, has_valid_mixed_artifact,
    has_valid_system_artifact, microphone_artifact, mixed_artifact, system_artifact,
    waveform_artifact,
};
use super::repository::{LibraryRepository, read_files, relative_path};
use crate::audio::{
    RecordingCapture, finalize_microphone_track, finalize_system_track, mix_tracks, waveform_peaks,
};
use crate::error::AppError;
use crate::error::AppResult;

impl LibraryRepository {
    pub fn recording_directory(&self, session_id: &str) -> AppResult<PathBuf> {
        Ok(self.session_directory(session_id)?.join("audio/.recovery"))
    }

    pub fn begin_recording(
        &self,
        session_id: &str,
        capture: RecordingCapture,
    ) -> AppResult<Session> {
        self.update_session(session_id, move |session| {
            session.lifecycle = SessionLifecycle::Recording;
            session.recovery_state = RecoveryState::Recoverable;
            session.started_at = Some(opentranscribe_domain::now());
            session.microphone = Some(capture.microphone);
            session.system_output = capture.system_output;
        })
    }

    pub fn finish_recording(&self, session_id: &str) -> AppResult<Session> {
        self.finalize_recording(session_id, SessionLifecycle::Ready, RecoveryState::None)
    }

    pub fn mark_recording_needs_attention(&self, session_id: &str) -> AppResult<Session> {
        self.update_session(session_id, |session| {
            session.lifecycle = SessionLifecycle::NeedsAttention;
            session.recovery_state = RecoveryState::Recoverable;
        })
    }

    pub fn recover_recording(&self, session_id: &str) -> AppResult<Session> {
        let session = self.session_workspace(session_id)?.session;

        if session.recovery_state != RecoveryState::Recoverable {
            return Err(AppError::Application(
                "this session has no recoverable audio".to_owned(),
            ));
        }

        self.finalize_recording(
            session_id,
            SessionLifecycle::Recovered,
            RecoveryState::Recovered,
        )
    }

    fn finalize_recording(
        &self,
        session_id: &str,
        lifecycle: SessionLifecycle,
        recovery_state: RecoveryState,
    ) -> AppResult<Session> {
        let directory = self.session_directory(session_id)?;
        let recovery_directory = directory.join("audio/.recovery");
        let current_session = self.session_workspace(session_id)?.session;
        let microphone_track = finalize_microphone_track(
            &recovery_directory,
            &directory.join("audio/microphone.wav"),
        )?;
        let microphone_artifact = microphone_artifact(
            relative_path(&self.root, &microphone_track.path),
            &microphone_track.path,
        )?;
        let mut duration_ms = microphone_artifact
            .duration_ms
            .expect("validated microphone artifacts have a duration");
        let mut artifacts = vec![microphone_artifact];
        let mut waveform_source = microphone_track.path.clone();
        let mut recovery_chunks = microphone_track.recovery_chunks;

        if current_session.system_output.is_some() {
            let system_track =
                finalize_system_track(&recovery_directory, &directory.join("audio/system.wav"))?;
            let system_artifact = system_artifact(
                relative_path(&self.root, &system_track.path),
                &system_track.path,
            )?;
            duration_ms = duration_ms.max(
                system_artifact
                    .duration_ms
                    .expect("validated system artifacts have a duration"),
            );
            artifacts.push(system_artifact);
            recovery_chunks.extend(system_track.recovery_chunks);
            let mixed_path = mix_tracks(
                &microphone_track.path,
                &system_track.path,
                &directory.join("audio/mixed.wav"),
            )?;
            artifacts.push(mixed_artifact(
                relative_path(&self.root, &mixed_path),
                &mixed_path,
            )?);
            waveform_source = mixed_path;
        }

        let waveform_path = directory.join("audio/waveform.json");
        atomic_file::write_json(&waveform_path, &waveform_peaks(&waveform_source, 160)?)?;
        artifacts.push(waveform_artifact(
            relative_path(&self.root, &waveform_path),
            &waveform_path,
            duration_ms,
        )?);

        let session = self.update_session(session_id, |session| {
            session.lifecycle = lifecycle;
            session.recovery_state = recovery_state;
            session.stopped_at = Some(opentranscribe_domain::now());
            session.duration_ms = duration_ms;
            session.artifacts.retain(|artifact| {
                !matches!(
                    artifact.kind,
                    ArtifactKind::Microphone
                        | ArtifactKind::System
                        | ArtifactKind::Mixed
                        | ArtifactKind::Waveform
                )
            });
            session.artifacts.extend(artifacts);
        })?;

        for chunk in recovery_chunks {
            fs::remove_file(chunk)?;
        }

        Ok(session)
    }

    pub fn session_audio_sources(&self, session_id: &str) -> AppResult<Vec<SessionAudioSource>> {
        let workspace = self.session_workspace(session_id)?;
        Ok(audio_sources(&self.root, &workspace.session))
    }

    pub fn session_waveform(&self, session_id: &str) -> AppResult<Vec<f32>> {
        let workspace = self.session_workspace(session_id)?;
        let artifact = workspace
            .session
            .artifacts
            .iter()
            .find(|artifact| artifact.kind == ArtifactKind::Waveform)
            .ok_or(AppError::NotFound)?;
        Ok(serde_json::from_slice(&fs::read(
            self.root.join(&artifact.relative_path),
        )?)?)
    }

    pub(super) fn mark_recoverable_recordings(&self) -> AppResult<()> {
        for session in self.sessions()? {
            let recovery_chunks = read_files(&self.recording_directory(&session.id)?)?;

            if recovery_chunks.is_empty() {
                continue;
            }

            if session.lifecycle == SessionLifecycle::Ready
                && has_valid_microphone_artifact(&self.root, &session)?
                && (session.system_output.is_none()
                    || (has_valid_system_artifact(&self.root, &session)?
                        && has_valid_mixed_artifact(&self.root, &session)?))
            {
                for chunk in recovery_chunks {
                    fs::remove_file(chunk.path())?;
                }
                continue;
            }

            self.update_session(&session.id, |session| {
                session.lifecycle = SessionLifecycle::Recovered;
                session.recovery_state = RecoveryState::Recoverable;
            })?;
        }

        Ok(())
    }
}
