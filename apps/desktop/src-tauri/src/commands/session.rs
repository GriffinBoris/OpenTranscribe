use opentranscribe_domain::{AppEvent, Session, Transcript};
use serde::Deserialize;
use std::path::PathBuf;
use tauri::Manager;

use super::{AppState, send_event, with_repository};
use crate::error::{AppError, AppResult};
use crate::exports::{ExportFormat, ExportResult, ExportService};
use crate::storage::{LibraryRepository, SavedDocument, SessionAudioSource, SessionWorkspace};

#[derive(Deserialize)]
pub struct SaveNotesRequest {
    pub session_id: String,
    pub markdown: String,
    pub expected_hash: String,
}

#[derive(Deserialize)]
pub struct ExportSessionRequest {
    pub session_id: String,
    pub format: ExportFormat,
    pub destination_path: PathBuf,
}

#[derive(Deserialize)]
pub struct MoveSessionRequest {
    pub session_id: String,
    pub project_id: Option<String>,
}

#[derive(Deserialize)]
pub struct RenameSessionRequest {
    pub session_id: String,
    pub title: String,
}

#[derive(Deserialize)]
pub struct UpdateTranscriptSegmentRequest {
    pub session_id: String,
    pub segment_id: String,
    pub text: String,
    pub expected_revision: u64,
}

#[derive(Deserialize)]
pub struct RenameSpeakerRequest {
    pub session_id: String,
    pub speaker_id: String,
    pub display_name: String,
    pub expected_revision: u64,
}

#[derive(Deserialize)]
pub struct MergeSpeakersRequest {
    pub session_id: String,
    pub source_speaker_id: String,
    pub target_speaker_id: String,
    pub expected_revision: u64,
}

#[tauri::command]
pub fn save_notes(
    request: SaveNotesRequest,
    state: tauri::State<'_, AppState>,
) -> AppResult<SavedDocument> {
    with_repository(&state, |repository| {
        repository.save_notes(
            &request.session_id,
            &request.markdown,
            &request.expected_hash,
        )
    })
}

#[tauri::command]
pub fn session_workspace(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> AppResult<SessionWorkspace> {
    with_repository(&state, |repository| {
        repository.session_workspace(&session_id)
    })
}

#[tauri::command]
pub fn session_audio_sources(
    session_id: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<Vec<SessionAudioSource>> {
    let sources = with_repository(&state, |repository| {
        repository.session_audio_sources(&session_id)
    })?;

    for source in &sources {
        app.asset_protocol_scope()
            .allow_file(&source.path)
            .map_err(|error| AppError::Application(error.to_string()))?;
    }

    Ok(sources)
}

#[tauri::command]
pub fn session_waveform(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> AppResult<Vec<f32>> {
    with_repository(&state, |repository| {
        repository.session_waveform(&session_id)
    })
}

#[tauri::command]
pub fn recover_recording(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> AppResult<Session> {
    if state.recorder.status().is_some() {
        return Err(AppError::Application(
            "stop the current recording before recovering another session".to_owned(),
        ));
    }

    let session = with_repository(&state, |repository| {
        repository.recover_recording(&session_id)
    })?;
    send_event(&state, AppEvent::LibraryChanged);
    Ok(session)
}

#[tauri::command]
pub fn reveal_session(session_id: String, state: tauri::State<'_, AppState>) -> AppResult<()> {
    let path = with_repository(&state, |repository| {
        repository.session_directory_path(&session_id)
    })?;
    tauri_plugin_opener::reveal_item_in_dir(path)
        .map_err(|error| AppError::Application(error.to_string()))
}

#[tauri::command]
pub fn rename_session(
    request: RenameSessionRequest,
    state: tauri::State<'_, AppState>,
) -> AppResult<Session> {
    let title = request.title.trim().to_owned();

    if title.is_empty() {
        return Err(AppError::Application(
            "meeting title cannot be empty".to_owned(),
        ));
    }

    let session = with_repository(&state, |repository| {
        repository.rename_session(&request.session_id, title)
    })?;
    send_event(&state, AppEvent::LibraryChanged);
    Ok(session)
}

#[tauri::command]
pub fn move_session(
    request: MoveSessionRequest,
    state: tauri::State<'_, AppState>,
) -> AppResult<Session> {
    if state
        .recorder
        .status()
        .is_some_and(|status| status.session_id == request.session_id)
    {
        return Err(AppError::Application(
            "stop the recording before moving it to a project".to_owned(),
        ));
    }

    let has_active_job = with_repository(&state, |repository| {
        repository.has_active_job_for_session(&request.session_id)
    })?;

    if has_active_job {
        return Err(AppError::Application(
            "cancel active processing before moving this session to a project".to_owned(),
        ));
    }

    let session = with_repository(&state, |repository| {
        repository.move_session(&request.session_id, request.project_id)
    })?;
    send_event(&state, AppEvent::LibraryChanged);
    Ok(session)
}

#[tauri::command]
pub fn trashed_sessions(state: tauri::State<'_, AppState>) -> AppResult<Vec<Session>> {
    with_repository(&state, LibraryRepository::trashed_sessions)
}

#[tauri::command]
pub fn trash_session(session_id: String, state: tauri::State<'_, AppState>) -> AppResult<Session> {
    if state
        .recorder
        .status()
        .is_some_and(|status| status.session_id == session_id)
    {
        return Err(AppError::Application(
            "stop the recording before moving it to trash".to_owned(),
        ));
    }

    let has_active_job = with_repository(&state, |repository| {
        repository.has_active_job_for_session(&session_id)
    })?;

    if has_active_job {
        return Err(AppError::Application(
            "cancel active processing before moving this session to trash".to_owned(),
        ));
    }

    let session = with_repository(&state, |repository| repository.trash_session(&session_id))?;
    send_event(&state, AppEvent::LibraryChanged);
    Ok(session)
}

#[tauri::command]
pub fn restore_session(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> AppResult<Session> {
    let session = with_repository(&state, |repository| repository.restore_session(&session_id))?;
    send_event(&state, AppEvent::LibraryChanged);
    Ok(session)
}

#[tauri::command]
pub fn empty_trash(state: tauri::State<'_, AppState>) -> AppResult<()> {
    with_repository(&state, LibraryRepository::empty_trash)?;
    send_event(&state, AppEvent::LibraryChanged);
    Ok(())
}

#[tauri::command]
pub fn update_transcript_segment(
    request: UpdateTranscriptSegmentRequest,
    state: tauri::State<'_, AppState>,
) -> AppResult<Transcript> {
    let text = request.text.trim().to_owned();
    if text.is_empty() {
        return Err(AppError::Application(
            "transcript text cannot be empty".to_owned(),
        ));
    }

    with_repository(&state, |repository| {
        repository.update_transcript_segment(
            &request.session_id,
            &request.segment_id,
            text,
            request.expected_revision,
        )
    })
}

#[tauri::command]
pub fn rename_speaker(
    request: RenameSpeakerRequest,
    state: tauri::State<'_, AppState>,
) -> AppResult<Transcript> {
    let display_name = request.display_name.trim().to_owned();

    if display_name.is_empty() {
        return Err(AppError::Application(
            "speaker name cannot be empty".to_owned(),
        ));
    }

    with_repository(&state, |repository| {
        repository.rename_speaker(
            &request.session_id,
            &request.speaker_id,
            display_name,
            request.expected_revision,
        )
    })
}

#[tauri::command]
pub fn merge_speakers(
    request: MergeSpeakersRequest,
    state: tauri::State<'_, AppState>,
) -> AppResult<Transcript> {
    with_repository(&state, |repository| {
        repository.merge_speakers(
            &request.session_id,
            &request.source_speaker_id,
            &request.target_speaker_id,
            request.expected_revision,
        )
    })
}

#[tauri::command]
pub fn export_session(
    request: ExportSessionRequest,
    state: tauri::State<'_, AppState>,
) -> AppResult<ExportResult> {
    let input = with_repository(&state, |repository| {
        repository.export_input(&request.session_id)
    })?;
    ExportService::export(input, request.format, request.destination_path)
}
