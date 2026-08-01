use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use opentranscribe_domain::{Artifact, ArtifactKind, Codec, Session};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::{AppError, AppResult};

#[derive(Clone, Serialize)]
pub struct SessionAudioSource {
    pub kind: ArtifactKind,
    pub path: PathBuf,
    pub duration_ms: Option<u64>,
}

pub fn microphone_artifact(relative_path: String, path: &Path) -> AppResult<Artifact> {
    audio_artifact(ArtifactKind::Microphone, relative_path, path)
}

pub fn system_artifact(relative_path: String, path: &Path) -> AppResult<Artifact> {
    audio_artifact(ArtifactKind::System, relative_path, path)
}

pub fn mixed_artifact(relative_path: String, path: &Path) -> AppResult<Artifact> {
    audio_artifact(ArtifactKind::Mixed, relative_path, path)
}

pub fn waveform_artifact(
    relative_path: String,
    path: &Path,
    duration_ms: u64,
) -> AppResult<Artifact> {
    Ok(Artifact {
        id: opentranscribe_domain::new_id(),
        kind: ArtifactKind::Waveform,
        relative_path,
        codec: Codec::Json,
        sample_rate_hz: None,
        channels: None,
        duration_ms: Some(duration_ms),
        byte_count: fs::metadata(path)?.len(),
        sha256: hash_file(path)?,
    })
}

fn audio_artifact(kind: ArtifactKind, relative_path: String, path: &Path) -> AppResult<Artifact> {
    let reader =
        hound::WavReader::open(path).map_err(|error| AppError::Audio(error.to_string()))?;
    let spec = reader.spec();
    let duration_ms = u64::from(reader.duration()) * 1_000 / u64::from(spec.sample_rate);

    Ok(Artifact {
        id: opentranscribe_domain::new_id(),
        kind,
        relative_path,
        codec: Codec::WavPcm24,
        sample_rate_hz: Some(spec.sample_rate),
        channels: Some(spec.channels),
        duration_ms: Some(duration_ms),
        byte_count: fs::metadata(path)?.len(),
        sha256: hash_file(path)?,
    })
}

pub fn audio_sources(root: &Path, session: &Session) -> Vec<SessionAudioSource> {
    [
        ArtifactKind::Mixed,
        ArtifactKind::Microphone,
        ArtifactKind::System,
        ArtifactKind::ImportedAudio,
        ArtifactKind::ImportedOriginal,
    ]
    .iter()
    .filter_map(|kind| {
        let artifact = session
            .artifacts
            .iter()
            .find(|artifact| artifact.kind == *kind)?;
        Some(SessionAudioSource {
            kind: artifact.kind.clone(),
            path: root.join(&artifact.relative_path),
            duration_ms: artifact.duration_ms,
        })
    })
    .collect()
}

pub fn has_valid_microphone_artifact(root: &Path, session: &Session) -> AppResult<bool> {
    has_valid_artifact(root, session, ArtifactKind::Microphone)
}

pub fn has_valid_system_artifact(root: &Path, session: &Session) -> AppResult<bool> {
    has_valid_artifact(root, session, ArtifactKind::System)
}

pub fn has_valid_mixed_artifact(root: &Path, session: &Session) -> AppResult<bool> {
    has_valid_artifact(root, session, ArtifactKind::Mixed)
}

fn has_valid_artifact(root: &Path, session: &Session, kind: ArtifactKind) -> AppResult<bool> {
    let Some(artifact) = session
        .artifacts
        .iter()
        .find(|artifact| artifact.kind == kind)
    else {
        return Ok(false);
    };
    let path = root.join(&artifact.relative_path);

    if !path.exists() || fs::metadata(&path)?.len() != artifact.byte_count {
        return Ok(false);
    }

    let reader =
        hound::WavReader::open(&path).map_err(|error| AppError::Audio(error.to_string()))?;
    let duration_ms = u64::from(reader.duration()) * 1_000 / u64::from(reader.spec().sample_rate);

    Ok(artifact.duration_ms == Some(duration_ms) && hash_file(&path)? == artifact.sha256)
}

pub fn hash_file(path: &Path) -> AppResult<String> {
    let mut file = fs::File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let byte_count = file.read(&mut buffer)?;

        if byte_count == 0 {
            break;
        }

        digest.update(&buffer[..byte_count]);
    }

    Ok(hex::encode(digest.finalize()))
}
