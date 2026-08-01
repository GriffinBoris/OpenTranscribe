use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;
use opentranscribe_domain::{
    LibraryDescriptor, LibraryManifest, Project, Session, SessionLifecycle, SessionSource,
};

use crate::error::{AppError, AppResult};

use super::atomic_file;
use super::index::LibraryIndex;
use super::job_record;

const RESERVED_NAMES: [&str; 4] = ["inbox", "projects", "trash", ".opentranscribe"];

pub struct LibraryRepository {
    pub(super) root: PathBuf,
    manifest: LibraryManifest,
    pub(super) index: LibraryIndex,
}

impl LibraryRepository {
    pub fn initialize(path: &Path) -> AppResult<Self> {
        fs::create_dir_all(path.join(".opentranscribe"))?;
        fs::create_dir_all(path.join(".opentranscribe/jobs"))?;
        fs::create_dir_all(path.join("Inbox"))?;
        fs::create_dir_all(path.join("Projects"))?;
        fs::create_dir_all(path.join("Trash/sessions"))?;
        fs::create_dir_all(path.join("Trash/projects"))?;

        let manifest_path = path.join(".opentranscribe/library.json");
        let manifest = if manifest_path.exists() {
            serde_json::from_slice(&fs::read(&manifest_path)?)?
        } else {
            let manifest = LibraryManifest::new();
            atomic_file::write_json(&manifest_path, &manifest)?;
            manifest
        };
        let index = LibraryIndex::open(&path.join(".opentranscribe/index.sqlite3"))?;

        let repository = Self {
            root: path.to_owned(),
            manifest,
            index,
        };
        repository.refresh_search_index()?;
        repository.mark_recoverable_recordings()?;
        job_record::mark_interrupted(&repository.jobs_directory())?;
        Ok(repository)
    }

    pub fn descriptor(&self) -> AppResult<LibraryDescriptor> {
        let projects = self.projects()?;
        let sessions = self.sessions()?;

        Ok(LibraryDescriptor {
            id: self.manifest.id.clone(),
            path: self.root.to_string_lossy().into_owned(),
            project_count: projects.len(),
            session_count: sessions.len(),
        })
    }

    pub fn root_path(&self) -> &Path {
        &self.root
    }

    pub fn projects(&self) -> AppResult<Vec<Project>> {
        let mut projects = Vec::new();

        for entry in read_directories(&self.root.join("Projects"))? {
            let path = entry.path().join("project.json");

            if path.exists() {
                projects.push(serde_json::from_slice(&fs::read(path)?)?);
            }
        }

        projects.sort_by(|left: &Project, right: &Project| left.name.cmp(&right.name));
        Ok(projects)
    }

    pub fn sessions(&self) -> AppResult<Vec<Session>> {
        let mut sessions = read_sessions_from(&self.root.join("Inbox"))?;

        for project in read_directories(&self.root.join("Projects"))? {
            sessions.extend(read_sessions_from(&project.path())?);
        }

        sessions.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        Ok(sessions)
    }

    pub fn create_project(&self, name: String) -> AppResult<Project> {
        validate_name(&name)?;
        let project = Project::new(name.trim().to_owned());
        let directory_name = format!(
            "{} [{}]",
            sanitize_name(&project.name),
            &project.id[project.id.len() - 6..]
        );
        let directory = self.root.join("Projects").join(&directory_name);
        fs::create_dir(&directory)?;
        atomic_file::write_json(&directory.join("project.json"), &project)?;
        self.index
            .replace_project(&project, &format!("Projects/{directory_name}"))?;
        Ok(project)
    }

    #[cfg(test)]
    pub fn create_session(
        &self,
        title: String,
        project_id: Option<String>,
        source: SessionSource,
    ) -> AppResult<Session> {
        self.create_session_manifest(title, project_id, source, None)
    }

    pub fn create_recording_session(
        &self,
        title: String,
        project_id: Option<String>,
        language_hint: Option<String>,
    ) -> AppResult<Session> {
        self.create_session_manifest(title, project_id, SessionSource::Recording, language_hint)
    }

    fn create_session_manifest(
        &self,
        title: String,
        project_id: Option<String>,
        source: SessionSource,
        language_hint: Option<String>,
    ) -> AppResult<Session> {
        let mut session = Session::new(title.trim().to_owned(), project_id.clone(), source);
        session.language_hint = language_hint;
        let directory_name = session_directory_name(&session);
        let parent = match project_id {
            Some(project_id) => self.project_directory(&project_id)?,
            None => self.root.join("Inbox"),
        };
        let directory = parent.join(&directory_name);

        write_session_directory(&directory, &session)?;
        self.index
            .replace_session(&session, &relative_path(&self.root, &directory))?;
        Ok(session)
    }

    pub fn discard_draft_session(&self, session_id: &str) -> AppResult<()> {
        let directory = self.session_directory(session_id)?;
        let session: Session = serde_json::from_slice(&fs::read(directory.join("session.json"))?)?;
        let notes = fs::read(directory.join("notes.md"))?;

        if session.lifecycle != SessionLifecycle::Draft
            || !session.artifacts.is_empty()
            || !notes.is_empty()
        {
            return Err(AppError::Application(
                "only a new empty draft session can be discarded".to_owned(),
            ));
        }

        fs::remove_dir_all(directory)?;
        self.index.remove_session(session_id)
    }

    pub fn session_directory_path(&self, session_id: &str) -> AppResult<PathBuf> {
        self.session_directory(session_id)
    }

    pub fn rename_session(&self, session_id: &str, title: String) -> AppResult<Session> {
        self.update_session(session_id, |session| {
            session.title = title;
        })
    }

    pub(super) fn update_session(
        &self,
        session_id: &str,
        update: impl FnOnce(&mut Session),
    ) -> AppResult<Session> {
        let directory = self.session_directory(session_id)?;
        let manifest_path = directory.join("session.json");
        let mut session: Session = serde_json::from_slice(&fs::read(&manifest_path)?)?;
        update(&mut session);
        session.revision += 1;
        atomic_file::write_json(&manifest_path, &session)?;
        self.index
            .replace_session(&session, &relative_path(&self.root, &directory))?;
        Ok(session)
    }

    pub(super) fn project_directory(&self, project_id: &str) -> AppResult<PathBuf> {
        read_directories(&self.root.join("Projects"))?
            .into_iter()
            .find_map(|entry| {
                let manifest_path = entry.path().join("project.json");
                let project: Project =
                    serde_json::from_slice(&fs::read(manifest_path).ok()?).ok()?;
                (project.id == project_id).then(|| entry.path())
            })
            .ok_or(AppError::NotFound)
    }

    pub(super) fn jobs_directory(&self) -> PathBuf {
        self.root.join(".opentranscribe/jobs")
    }

    pub(super) fn session_directory(&self, session_id: &str) -> AppResult<PathBuf> {
        let mut parents = vec![self.root.join("Inbox")];
        parents.extend(
            read_directories(&self.root.join("Projects"))?
                .into_iter()
                .map(|entry| entry.path()),
        );

        for parent in parents {
            for entry in read_directories(&parent)? {
                let manifest_path = entry.path().join("session.json");

                if !manifest_path.exists() {
                    continue;
                }

                let session: Session = serde_json::from_slice(&fs::read(manifest_path)?)?;

                if session.id == session_id {
                    return Ok(entry.path());
                }
            }
        }

        Err(AppError::NotFound)
    }
}

pub(super) fn read_directories(path: &Path) -> AppResult<Vec<fs::DirEntry>> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;

        if entry.file_type()?.is_dir() {
            entries.push(entry);
        }
    }

    Ok(entries)
}

pub(super) fn read_files(path: &Path) -> AppResult<Vec<fs::DirEntry>> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;

        if entry.file_type()?.is_file() {
            entries.push(entry);
        }
    }

    entries.sort_by_key(fs::DirEntry::file_name);
    Ok(entries)
}

fn read_sessions_from(path: &Path) -> AppResult<Vec<Session>> {
    let mut sessions = Vec::new();

    for entry in read_directories(path)? {
        let manifest = entry.path().join("session.json");

        if manifest.exists() {
            sessions.push(serde_json::from_slice(&fs::read(manifest)?)?);
        }
    }

    Ok(sessions)
}

pub(super) fn write_session_directory(directory: &Path, session: &Session) -> AppResult<()> {
    fs::create_dir(directory)?;
    fs::create_dir_all(directory.join("audio/.recovery"))?;
    fs::create_dir_all(directory.join("transcripts/runs"))?;
    fs::create_dir_all(directory.join("exports"))?;
    atomic_file::write_json(&directory.join("session.json"), session)?;
    atomic_file::write(&directory.join("notes.md"), b"")?;
    atomic_file::write(&directory.join("markers.json"), b"[]\n")?;
    Ok(())
}

fn validate_name(name: &str) -> AppResult<()> {
    let normalized = name.trim().to_lowercase();

    if normalized.is_empty() || RESERVED_NAMES.contains(&normalized.as_str()) {
        return Err(AppError::InvalidName);
    }

    Ok(())
}

fn sanitize_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|character| match character {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '-',
            _ => character,
        })
        .collect();

    sanitized.trim().chars().take(80).collect()
}

pub(super) fn session_directory_name(session: &Session) -> String {
    let timestamp = session.created_at.with_timezone(&Local);

    format!(
        "{} {} [{}]",
        timestamp.format("%Y-%m-%d %H.%M"),
        sanitize_name(&session.title),
        &session.id[session.id.len() - 6..]
    )
}

pub(super) fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("library-owned paths are below the library root")
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
