use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Duration;

use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use tauri::Manager;

use crate::error::{AppError, AppResult};

use super::LocalModel;
use super::catalog::{MODELS, ModelDefinition, find};

pub fn statuses(app: &tauri::AppHandle) -> AppResult<Vec<LocalModel>> {
    MODELS.iter().map(|model| status(app, model)).collect()
}

pub fn installed_path(app: &tauri::AppHandle, model_id: &str) -> AppResult<PathBuf> {
    let definition = find(model_id).ok_or_else(|| AppError::Model("unknown model".to_owned()))?;
    let path = model_path(app, definition)?;

    if !is_installed(&path, definition)? {
        return Err(AppError::Model(format!(
            "{} is not installed",
            definition.label
        )));
    }

    Ok(path)
}

pub fn download(
    app: &tauri::AppHandle,
    model_id: &str,
    mut on_progress: impl FnMut(u64, u64),
) -> AppResult<LocalModel> {
    let definition = find(model_id).ok_or_else(|| AppError::Model("unknown model".to_owned()))?;
    let path = model_path(app, definition)?;

    if is_installed(&path, definition)? {
        return status(app, definition);
    }

    let directory = path
        .parent()
        .expect("model paths always have a parent directory");
    fs::create_dir_all(directory)?;
    let temporary_path = directory.join(format!("{}.download", definition.filename));
    let mut response = Client::builder()
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(2 * 60 * 60))
        .build()
        .map_err(|error| AppError::Model(error.to_string()))?
        .get(definition.download_url)
        .send()
        .map_err(|error| AppError::Model(error.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::Model(format!(
            "model download returned HTTP {}",
            response.status()
        )));
    }

    let mut file = File::create(&temporary_path)?;
    let mut hasher = Sha256::new();
    let mut completed = 0_u64;
    let mut reported = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];

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

        if completed == definition.byte_count || completed - reported >= 1_048_576 {
            on_progress(completed, definition.byte_count);
            reported = completed;
        }
    }

    file.sync_all()?;

    if completed != definition.byte_count {
        return Err(AppError::Model(format!(
            "model download was incomplete: expected {} bytes, received {completed}",
            definition.byte_count
        )));
    }

    if hex::encode(hasher.finalize()) != definition.sha256 {
        return Err(AppError::Model(
            "model download failed its integrity check".to_owned(),
        ));
    }

    if path.exists() {
        fs::remove_file(&path)?;
    }

    fs::rename(temporary_path, path)?;
    status(app, definition)
}

pub fn remove(app: &tauri::AppHandle, model_id: &str) -> AppResult<LocalModel> {
    let definition = find(model_id).ok_or_else(|| AppError::Model("unknown model".to_owned()))?;
    let path = model_path(app, definition)?;

    if path.exists() {
        fs::remove_file(path)?;
    }

    status(app, definition)
}

fn status(app: &tauri::AppHandle, definition: &ModelDefinition) -> AppResult<LocalModel> {
    let path = model_path(app, definition)?;

    Ok(LocalModel {
        id: definition.id.to_owned(),
        preset: definition.preset.to_owned(),
        label: definition.label.to_owned(),
        description: definition.description.to_owned(),
        byte_count: definition.byte_count,
        installed: is_installed(&path, definition)?,
    })
}

fn model_path(app: &tauri::AppHandle, definition: &ModelDefinition) -> AppResult<PathBuf> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|error| AppError::Application(error.to_string()))?
        .join("models")
        .join(definition.id)
        .join(definition.filename))
}

fn is_installed(path: &PathBuf, definition: &ModelDefinition) -> AppResult<bool> {
    if !path.exists() {
        return Ok(false);
    }

    Ok(fs::metadata(path)?.len() == definition.byte_count)
}
