use std::fs;
use std::path::Path;

use opentranscribe_domain::Session;

use super::atomic_file;
use super::repository::{LibraryRepository, relative_path};
use crate::error::{AppError, AppResult};

impl LibraryRepository {
    pub fn move_session(&self, session_id: &str, project_id: Option<String>) -> AppResult<Session> {
        let source = self.session_directory(session_id)?;
        let mut session: Session = serde_json::from_slice(&fs::read(source.join("session.json"))?)?;

        if session.project_id == project_id {
            return Ok(session);
        }

        let parent = match project_id.as_deref() {
            Some(project_id) => self.project_directory(project_id)?,
            None => self.root.join("Inbox"),
        };
        let destination = parent.join(source.file_name().ok_or(AppError::NotFound)?);
        let source_relative = relative_path(&self.root, &source);
        let destination_relative = relative_path(&self.root, &destination);

        session.project_id = project_id;
        session.revision += 1;

        for artifact in &mut session.artifacts {
            artifact.relative_path = rebase_artifact_path(
                &artifact.relative_path,
                &source_relative,
                &destination_relative,
            )?;
        }

        fs::rename(&source, &destination)?;
        if let Err(error) = atomic_file::write_json(&destination.join("session.json"), &session) {
            fs::rename(&destination, &source).map_err(|rollback_error| {
                AppError::Application(format!(
                    "{error}; moving the session back also failed: {rollback_error}"
                ))
            })?;
            return Err(error);
        }

        self.index
            .replace_session(&session, &destination_relative)?;
        Ok(session)
    }
}

fn rebase_artifact_path(path: &str, source: &str, destination: &str) -> AppResult<String> {
    let suffix = Path::new(path).strip_prefix(source).map_err(|_| {
        AppError::Application("session artifact is outside its session directory".to_owned())
    })?;

    Ok(Path::new(destination)
        .join(suffix)
        .to_string_lossy()
        .replace('\\', "/"))
}
