use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use reqwest::StatusCode;
use reqwest::blocking::Client;
use reqwest::header::RANGE;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::Manager;

use crate::error::{AppError, AppResult};
use crate::storage::atomic_file;

use super::LocalModel;
use super::catalog::{MODELS, ModelDefinition, find};

// Models sit beside the default library in the documents directory so the
// download is visible and removable without knowing platform data paths. This
// mirrors the folder name chosen by `default_library_path`.
const DOCUMENTS_DIRECTORY: &str = "OpenTranscribe";
const MODELS_DIRECTORY: &str = "models";
const MANIFEST_FILENAME: &str = "install.json";
const READ_BUFFER_BYTES: usize = 64 * 1024;
const PROGRESS_INTERVAL_BYTES: u64 = 1_048_576;

/// What was verified when a model was installed. Recording it lets later
/// launches accept the file after a stat instead of rehashing hundreds of
/// megabytes, while still noticing a file that was replaced or truncated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct InstallManifest {
    sha256: String,
    byte_count: u64,
    modified_at_millis: u64,
}

pub fn models_root(app: &tauri::AppHandle) -> AppResult<PathBuf> {
    let settings = crate::commands::application::load_settings(app)?;

    if let Some(path) = settings.local_models_directory {
        return Ok(PathBuf::from(path));
    }

    Ok(app
        .path()
        .document_dir()
        .map_err(|error| AppError::Application(error.to_string()))?
        .join(DOCUMENTS_DIRECTORY)
        .join(MODELS_DIRECTORY))
}

pub fn move_models(app: &tauri::AppHandle, target: &Path) -> AppResult<Option<PathBuf>> {
    migrate_legacy_models(app)?;
    let source = models_root(app)?;

    if source == target || !source.exists() {
        return Ok(None);
    }

    move_models_between(&source, target)?;
    Ok(Some(source))
}

pub fn move_models_between(source: &Path, target: &Path) -> AppResult<()> {
    if target.starts_with(&source) || source.starts_with(target) {
        return Err(AppError::Model(
            "choose a model folder outside the current model folder".to_owned(),
        ));
    }

    if target.exists() {
        if fs::read_dir(target)?.next().is_some() {
            return Err(AppError::Model(
                "choose an empty folder for local models".to_owned(),
            ));
        }

        fs::remove_dir(target)?;
    }

    let parent = target.parent().ok_or_else(|| {
        AppError::Model("choose a folder for local models instead of a filesystem root".to_owned())
    })?;
    fs::create_dir_all(parent)?;

    if fs::rename(&source, target).is_err() {
        if let Err(error) = copy_directory(&source, target) {
            let _ = fs::remove_dir_all(target);
            return Err(error);
        }
        fs::remove_dir_all(source)?;
    }

    Ok(())
}

pub fn remove_all(app: &tauri::AppHandle) -> AppResult<()> {
    let settings = crate::commands::application::load_settings(app)?;
    let root = models_root(app)?;

    if settings.local_models_directory.is_none() {
        if root.exists() {
            fs::remove_dir_all(root)?;
        }
        return Ok(());
    }

    remove_catalog_directories(&root)
}

pub fn statuses(app: &tauri::AppHandle) -> AppResult<Vec<LocalModel>> {
    migrate_legacy_models(app)?;
    MODELS.iter().map(|model| status(app, model)).collect()
}

pub fn installed_path(app: &tauri::AppHandle, model_id: &str) -> AppResult<PathBuf> {
    migrate_legacy_models(app)?;
    let definition = find(model_id).ok_or_else(|| AppError::Model("unknown model".to_owned()))?;

    if !is_installed(app, definition)? {
        return Err(AppError::Model(format!(
            "{} is not installed",
            definition.label
        )));
    }

    model_path(app, definition)
}

pub fn download(
    app: &tauri::AppHandle,
    model_id: &str,
    mut on_progress: impl FnMut(u64, u64),
) -> AppResult<LocalModel> {
    migrate_legacy_models(app)?;
    let definition = find(model_id).ok_or_else(|| AppError::Model("unknown model".to_owned()))?;

    if is_installed(app, definition)? {
        return status(app, definition);
    }

    let directory = model_directory(app, definition)?;
    fs::create_dir_all(&directory)?;
    let path = directory.join(definition.filename);
    let temporary_path = directory.join(format!("{}.download", definition.filename));

    let mut resume_from = resumable_bytes(&temporary_path, definition.byte_count);
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(2 * 60 * 60))
        .build()
        .map_err(|error| AppError::Model(error.to_string()))?;
    let mut request = client.get(definition.download_url);

    if resume_from > 0 {
        request = request.header(RANGE, format!("bytes={resume_from}-"));
    }

    let mut response = request
        .send()
        .map_err(|error| AppError::Model(error.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::Model(format!(
            "model download returned HTTP {}",
            response.status()
        )));
    }

    // A server that does not honour the range header answers with the whole
    // file instead, so the partial bytes on disk have to be discarded.
    if resume_from > 0 && response.status() != StatusCode::PARTIAL_CONTENT {
        resume_from = 0;
    }

    let mut hasher = Sha256::new();
    let mut file = if resume_from > 0 {
        hash_prefix(&temporary_path, resume_from, &mut hasher)?;
        OpenOptions::new().append(true).open(&temporary_path)?
    } else {
        File::create(&temporary_path)?
    };
    let mut completed = resume_from;
    let mut reported = resume_from;
    let mut buffer = [0_u8; READ_BUFFER_BYTES];

    on_progress(completed, definition.byte_count);

    loop {
        let count = response
            .read(&mut buffer)
            .map_err(|error| AppError::Model(error.to_string()))?;

        if count == 0 {
            break;
        }

        file.write_all(&buffer[..count])?;
        hasher.update(&buffer[..count]);
        completed += count as u64;

        if completed == definition.byte_count || completed - reported >= PROGRESS_INTERVAL_BYTES {
            on_progress(completed, definition.byte_count);
            reported = completed;
        }
    }

    file.sync_all()?;

    // The partial file is deliberately left in place so the next attempt
    // resumes from here instead of refetching what already arrived.
    if completed != definition.byte_count {
        return Err(AppError::Model(format!(
            "model download was incomplete: expected {} bytes, received {completed}",
            definition.byte_count
        )));
    }

    if hex::encode(hasher.finalize()) != definition.sha256 {
        // A resumed transfer can be poisoned by a partial file left by an
        // earlier, different download. Drop it so a retry starts clean.
        let _ = fs::remove_file(&temporary_path);
        return Err(AppError::Model(
            "model download failed its integrity check".to_owned(),
        ));
    }

    if path.exists() {
        fs::remove_file(&path)?;
    }

    fs::rename(&temporary_path, &path)?;
    write_manifest(&directory, &path, definition)?;
    status(app, definition)
}

pub fn remove(app: &tauri::AppHandle, model_id: &str) -> AppResult<LocalModel> {
    let definition = find(model_id).ok_or_else(|| AppError::Model("unknown model".to_owned()))?;
    let directory = model_directory(app, definition)?;

    if directory.exists() {
        fs::remove_dir_all(&directory)?;
    }

    status(app, definition)
}

fn status(app: &tauri::AppHandle, definition: &ModelDefinition) -> AppResult<LocalModel> {
    Ok(LocalModel {
        id: definition.id.to_owned(),
        preset: definition.preset.to_owned(),
        label: definition.label.to_owned(),
        description: definition.description.to_owned(),
        byte_count: definition.byte_count,
        installed: is_installed(app, definition)?,
    })
}

fn model_directory(app: &tauri::AppHandle, definition: &ModelDefinition) -> AppResult<PathBuf> {
    Ok(models_root(app)?.join(definition.id))
}

fn model_path(app: &tauri::AppHandle, definition: &ModelDefinition) -> AppResult<PathBuf> {
    Ok(model_directory(app, definition)?.join(definition.filename))
}

fn manifest_path(directory: &Path) -> PathBuf {
    directory.join(MANIFEST_FILENAME)
}

fn is_installed(app: &tauri::AppHandle, definition: &ModelDefinition) -> AppResult<bool> {
    let directory = model_directory(app, definition)?;
    let path = directory.join(definition.filename);
    let Ok(metadata) = fs::metadata(&path) else {
        return Ok(false);
    };

    if metadata.len() != definition.byte_count {
        return Ok(false);
    }

    let modified_at_millis = modified_at_millis(&metadata);

    match read_manifest(&manifest_path(&directory)) {
        Some(manifest) => Ok(manifest_describes(
            &manifest,
            definition,
            modified_at_millis,
        )),
        // Installed before manifests existed, or the manifest was lost. Hash
        // once and record the result rather than discarding a download the
        // user already paid several hundred megabytes for.
        None => {
            if hash_file(&path)? != definition.sha256 {
                return Ok(false);
            }

            write_manifest(&directory, &path, definition)?;
            Ok(true)
        }
    }
}

fn manifest_describes(
    manifest: &InstallManifest,
    definition: &ModelDefinition,
    modified_at_millis: u64,
) -> bool {
    manifest.sha256 == definition.sha256
        && manifest.byte_count == definition.byte_count
        && manifest.modified_at_millis == modified_at_millis
}

fn read_manifest(path: &Path) -> Option<InstallManifest> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

fn write_manifest(directory: &Path, path: &Path, definition: &ModelDefinition) -> AppResult<()> {
    let metadata = fs::metadata(path)?;

    atomic_file::write_json(
        &manifest_path(directory),
        &InstallManifest {
            sha256: definition.sha256.to_owned(),
            byte_count: definition.byte_count,
            modified_at_millis: modified_at_millis(&metadata),
        },
    )
}

fn modified_at_millis(metadata: &fs::Metadata) -> u64 {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

/// Bytes of a previous attempt a range request can skip. A partial file that
/// already claims the full length is not trusted, because nothing verified
/// its contents.
fn resumable_bytes(temporary_path: &Path, expected: u64) -> u64 {
    fs::metadata(temporary_path)
        .map(|metadata| usable_resume_offset(metadata.len(), expected))
        .unwrap_or_default()
}

fn usable_resume_offset(existing: u64, expected: u64) -> u64 {
    if existing == 0 || existing >= expected {
        return 0;
    }

    existing
}

fn hash_file(path: &Path) -> AppResult<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; READ_BUFFER_BYTES];

    loop {
        let count = file.read(&mut buffer)?;

        if count == 0 {
            break;
        }

        hasher.update(&buffer[..count]);
    }

    Ok(hex::encode(hasher.finalize()))
}

fn hash_prefix(path: &Path, length: u64, hasher: &mut Sha256) -> AppResult<()> {
    let mut file = File::open(path)?;
    let mut remaining = length;
    let mut buffer = [0_u8; READ_BUFFER_BYTES];

    while remaining > 0 {
        let wanted = remaining.min(READ_BUFFER_BYTES as u64) as usize;
        let count = file.read(&mut buffer[..wanted])?;

        if count == 0 {
            return Err(AppError::Model(
                "the partial model download ended before its recorded length".to_owned(),
            ));
        }

        hasher.update(&buffer[..count]);
        remaining -= count as u64;
    }

    Ok(())
}

/// Models used to live in the platform application-data directory. Move an
/// earlier download into the documents location instead of making the user
/// fetch it again.
fn migrate_legacy_models(app: &tauri::AppHandle) -> AppResult<()> {
    let Ok(legacy_root) = app.path().app_data_dir() else {
        return Ok(());
    };
    let legacy_root = legacy_root.join(MODELS_DIRECTORY);

    if !legacy_root.exists() {
        return Ok(());
    }

    let root = models_root(app)?;

    for definition in MODELS.iter() {
        let legacy_directory = legacy_root.join(definition.id);
        let legacy_path = legacy_directory.join(definition.filename);

        if !legacy_path.exists() {
            continue;
        }

        let directory = root.join(definition.id);
        let path = directory.join(definition.filename);

        if !path.exists() {
            fs::create_dir_all(&directory)?;
            move_file(&legacy_path, &path)?;
        }

        // Only removes a directory that is already empty, so a model that
        // could not be moved is never deleted.
        let _ = fs::remove_dir(&legacy_directory);
    }

    let _ = fs::remove_dir(&legacy_root);
    Ok(())
}

fn move_file(from: &Path, to: &Path) -> AppResult<()> {
    if fs::rename(from, to).is_ok() {
        return Ok(());
    }

    // The documents and application-data directories can sit on different
    // volumes, where a rename cannot work and the bytes have to be copied.
    fs::copy(from, to)?;
    fs::remove_file(from)?;
    Ok(())
}

fn copy_directory(from: &Path, to: &Path) -> AppResult<()> {
    fs::create_dir_all(to)?;

    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let source = entry.path();
        let target = to.join(entry.file_name());

        if source.is_dir() {
            copy_directory(&source, &target)?;
        } else {
            fs::copy(source, target)?;
        }
    }

    Ok(())
}

fn remove_catalog_directories(root: &Path) -> AppResult<()> {
    for definition in MODELS {
        let directory = root.join(definition.id);

        if directory.exists() {
            fs::remove_dir_all(directory)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use sha2::{Digest, Sha256};
    use tempfile::tempdir;

    use super::{
        InstallManifest, copy_directory, hash_file, hash_prefix, manifest_describes, move_file,
        move_models_between, remove_catalog_directories, usable_resume_offset,
    };
    use crate::local_models::catalog::MODELS;

    fn manifest(sha256: &str, byte_count: u64, modified_at_millis: u64) -> InstallManifest {
        InstallManifest {
            sha256: sha256.to_owned(),
            byte_count,
            modified_at_millis,
        }
    }

    #[test]
    fn resumes_from_a_partial_download() {
        assert_eq!(usable_resume_offset(500, 1_000), 500);
    }

    #[test]
    fn restarts_when_no_bytes_arrived() {
        assert_eq!(usable_resume_offset(0, 1_000), 0);
    }

    #[test]
    fn restarts_when_the_partial_file_is_not_shorter_than_the_model() {
        assert_eq!(usable_resume_offset(1_000, 1_000), 0);
        assert_eq!(usable_resume_offset(1_200, 1_000), 0);
    }

    #[test]
    fn accepts_a_manifest_that_describes_the_installed_file() {
        let definition = &MODELS[0];

        assert!(manifest_describes(
            &manifest(definition.sha256, definition.byte_count, 42),
            definition,
            42,
        ));
    }

    #[test]
    fn rejects_a_manifest_whose_file_changed_after_installation() {
        let definition = &MODELS[0];

        assert!(!manifest_describes(
            &manifest(definition.sha256, definition.byte_count, 42),
            definition,
            43,
        ));
    }

    #[test]
    fn rejects_a_manifest_left_by_a_superseded_catalog_entry() {
        let definition = &MODELS[0];

        assert!(!manifest_describes(
            &manifest(&"0".repeat(64), definition.byte_count, 42),
            definition,
            42,
        ));
    }

    #[test]
    fn hashes_only_the_recorded_prefix_of_a_partial_download() {
        let directory = tempdir().expect("temporary directory should be created");
        let partial = directory.path().join("model.download");
        let complete = directory.path().join("model.bin");
        fs::write(&partial, b"resume-me-please").expect("partial file should be written");
        fs::write(&complete, b"resume-me").expect("complete file should be written");

        let mut hasher = Sha256::new();
        hash_prefix(&partial, 9, &mut hasher).expect("prefix should hash");

        assert_eq!(
            hex::encode(hasher.finalize()),
            hash_file(&complete).expect("file should hash"),
        );
    }

    #[test]
    fn reports_a_partial_file_shorter_than_its_recorded_length() {
        let directory = tempdir().expect("temporary directory should be created");
        let partial = directory.path().join("model.download");
        fs::write(&partial, b"short").expect("partial file should be written");

        let mut hasher = Sha256::new();

        assert!(hash_prefix(&partial, 64, &mut hasher).is_err());
    }

    #[test]
    fn moves_a_model_between_directories() {
        let directory = tempdir().expect("temporary directory should be created");
        let from = directory.path().join("legacy.bin");
        let to = directory.path().join("moved.bin");
        fs::write(&from, b"weights").expect("source file should be written");

        move_file(&from, &to).expect("model should move");

        assert!(!from.exists());
        assert_eq!(fs::read(&to).expect("moved file should read"), b"weights");
    }

    #[test]
    fn copies_model_directories_between_filesystems() {
        let directory = tempdir().expect("temporary directory should be created");
        let from = directory.path().join("from");
        let to = directory.path().join("to");
        fs::create_dir_all(from.join("balanced")).expect("source directory should be created");
        fs::write(from.join("balanced").join("model.bin"), b"weights")
            .expect("source model should be written");

        copy_directory(&from, &to).expect("directory should copy");

        assert_eq!(
            fs::read(to.join("balanced").join("model.bin")).expect("copied model should read"),
            b"weights"
        );
    }

    #[test]
    fn restores_model_directories_after_a_failed_configuration_update() {
        let directory = tempdir().expect("temporary directory should be created");
        let source = directory.path().join("source");
        let target = directory.path().join("target");
        fs::create_dir_all(source.join("fast")).expect("source should be created");
        fs::write(source.join("fast").join("model.bin"), b"weights")
            .expect("model should be written");

        move_models_between(&source, &target).expect("model should move");
        move_models_between(&target, &source).expect("model should be restored");

        assert!(source.join("fast").join("model.bin").exists());
        assert!(!target.exists());
    }

    #[test]
    fn removes_model_directories_without_deleting_other_custom_folder_content() {
        let directory = tempdir().expect("temporary directory should be created");
        let models = directory.path().join("models");
        let model_directory = models.join(MODELS[0].id);
        fs::create_dir_all(&model_directory).expect("model directory should be created");
        fs::write(model_directory.join("model.bin"), b"weights").expect("model should be written");
        fs::write(models.join("keep.txt"), b"user content")
            .expect("user content should be written");

        remove_catalog_directories(&models).expect("catalog models should be removed");

        assert!(!model_directory.exists());
        assert_eq!(
            fs::read(models.join("keep.txt")).expect("user content should remain"),
            b"user content"
        );
    }
}
