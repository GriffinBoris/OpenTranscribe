use std::fs;

use opentranscribe_domain::{Session, SessionLifecycle};

use super::atomic_file;
use super::repository::{LibraryRepository, read_directories, relative_path};
use crate::error::{AppError, AppResult};

impl LibraryRepository {
    pub fn trashed_sessions(&self) -> AppResult<Vec<Session>> {
        let mut sessions = Vec::new();

        for entry in read_directories(&self.root.join("Trash/sessions"))? {
            let manifest_path = entry.path().join("session.json");

            if manifest_path.exists() {
                sessions.push(serde_json::from_slice(&fs::read(manifest_path)?)?);
            }
        }

        sessions.sort_by(|left: &Session, right: &Session| right.created_at.cmp(&left.created_at));
        Ok(sessions)
    }

    pub fn trash_session(&self, session_id: &str) -> AppResult<Session> {
        let source = self.session_directory_path(session_id)?;
        let destination = self
            .root
            .join("Trash/sessions")
            .join(source.file_name().ok_or(AppError::NotFound)?);
        let mut session: Session = serde_json::from_slice(&fs::read(source.join("session.json"))?)?;
        fs::rename(source, &destination)?;
        session.lifecycle = SessionLifecycle::Trashed;
        session.revision += 1;
        atomic_file::write_json(&destination.join("session.json"), &session)?;
        self.index.remove_session(session_id)?;
        Ok(session)
    }

    pub fn restore_session(&self, session_id: &str) -> AppResult<Session> {
        let source = self.trashed_session_directory(session_id)?;
        let mut session: Session = serde_json::from_slice(&fs::read(source.join("session.json"))?)?;
        let parent = match session.project_id.as_deref() {
            Some(project_id) => self.project_directory(project_id)?,
            None => self.root.join("Inbox"),
        };
        let destination = parent.join(source.file_name().ok_or(AppError::NotFound)?);
        fs::rename(source, &destination)?;
        session.lifecycle = SessionLifecycle::Ready;
        session.revision += 1;
        atomic_file::write_json(&destination.join("session.json"), &session)?;
        self.index
            .replace_session(&session, &relative_path(&self.root, &destination))?;
        Ok(session)
    }

    fn trashed_session_directory(&self, session_id: &str) -> AppResult<std::path::PathBuf> {
        for entry in read_directories(&self.root.join("Trash/sessions"))? {
            let manifest_path = entry.path().join("session.json");

            if !manifest_path.exists() {
                continue;
            }

            let session: Session = serde_json::from_slice(&fs::read(manifest_path)?)?;

            if session.id == session_id {
                return Ok(entry.path());
            }
        }

        Err(AppError::NotFound)
    }
}
