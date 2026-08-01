use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use opentranscribe_domain::{
    SearchFilters, SearchPage, SearchResult, Session, Transcript, TranscriptRun,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::{AppError, AppResult};

use super::atomic_file;
use super::repository::{LibraryRepository, relative_path};

#[derive(Clone, Debug, Serialize)]
pub struct SavedDocument {
    pub content_hash: String,
}

pub struct ExportInput {
    pub session: Session,
    pub transcript: Transcript,
    pub notes: String,
    pub directory: PathBuf,
}

#[derive(Serialize)]
pub struct SessionWorkspace {
    pub session: Session,
    pub notes: String,
    pub notes_hash: String,
    pub transcript: Option<Transcript>,
    pub transcript_run: Option<TranscriptRun>,
}

impl LibraryRepository {
    pub fn save_notes(
        &self,
        session_id: &str,
        markdown: &str,
        expected_hash: &str,
    ) -> AppResult<SavedDocument> {
        let directory = self.session_directory(session_id)?;
        let path = directory.join("notes.md");
        let current_hash = hash_bytes(&fs::read(&path)?);

        if current_hash != expected_hash {
            return Err(AppError::RevisionConflict);
        }

        atomic_file::write(&path, markdown.as_bytes())?;
        self.index.replace_notes(session_id, markdown)?;
        Ok(SavedDocument {
            content_hash: hash_bytes(markdown.as_bytes()),
        })
    }

    pub fn search(&self, query: &str, filters: &SearchFilters) -> AppResult<SearchPage> {
        let indexed_results = self.index.search(query.trim(), filters)?;
        let mut results = Vec::with_capacity(indexed_results.len());
        let mut transcripts = HashMap::new();

        for result in indexed_results {
            let timestamp_ms = if result.kind == "transcript" {
                if !transcripts.contains_key(&result.session_id) {
                    transcripts.insert(
                        result.session_id.clone(),
                        self.session_workspace(&result.session_id)?.transcript,
                    );
                }

                transcripts
                    .get(&result.session_id)
                    .and_then(Option::as_ref)
                    .and_then(|transcript| {
                        transcript
                            .segments
                            .iter()
                            .find(|segment| segment.id == result.entity_id)
                    })
                    .map(|segment| segment.start_ms)
            } else {
                None
            };

            results.push(SearchResult {
                session_id: result.session_id,
                session_title: result.session_title,
                kind: result.kind,
                excerpt: result.excerpt,
                timestamp_ms,
            });
        }

        Ok(SearchPage {
            results,
            next_cursor: None,
        })
    }

    pub fn session_workspace(&self, session_id: &str) -> AppResult<SessionWorkspace> {
        let directory = self.session_directory(session_id)?;
        let session = serde_json::from_slice(&fs::read(directory.join("session.json"))?)?;
        let notes_bytes = fs::read(directory.join("notes.md"))?;
        let transcript_path = directory.join("transcripts/transcript.json");
        let transcript: Option<Transcript> = if transcript_path.exists() {
            Some(serde_json::from_slice(&fs::read(transcript_path)?)?)
        } else {
            None
        };
        let transcript_run_path = transcript.as_ref().map(|transcript| {
            directory
                .join("transcripts/runs")
                .join(&transcript.source_run_id)
                .join("run.json")
        });
        let transcript_run = match transcript_run_path {
            Some(path) if path.exists() => Some(serde_json::from_slice(&fs::read(path)?)?),
            _ => None,
        };

        Ok(SessionWorkspace {
            session,
            notes: String::from_utf8(notes_bytes.clone())
                .map_err(|error| AppError::Io(std::io::Error::other(error)))?,
            notes_hash: hash_bytes(&notes_bytes),
            transcript,
            transcript_run,
        })
    }

    pub fn export_input(&self, session_id: &str) -> AppResult<ExportInput> {
        let directory = self.session_directory(session_id)?;
        let workspace = self.session_workspace(session_id)?;
        let transcript = workspace.transcript.ok_or_else(|| {
            AppError::Export("transcribe this session before exporting it".to_owned())
        })?;

        Ok(ExportInput {
            session: workspace.session,
            transcript,
            notes: workspace.notes,
            directory: directory.join("exports"),
        })
    }

    pub fn refresh_search_index(&self) -> AppResult<()> {
        self.index.clear_content()?;

        for project in self.projects()? {
            let directory = self.project_directory(&project.id)?;
            self.index
                .replace_project(&project, &relative_path(&self.root, &directory))?;
        }

        for session in self.sessions()? {
            let directory = self.session_directory(&session.id)?;
            self.index
                .replace_session(&session, &relative_path(&self.root, &directory))?;

            let notes = String::from_utf8(fs::read(directory.join("notes.md"))?)
                .map_err(|error| AppError::Io(std::io::Error::other(error)))?;
            self.index.replace_notes(&session.id, &notes)?;

            let transcript_path = directory.join("transcripts/transcript.json");
            if transcript_path.exists() {
                let transcript = serde_json::from_slice(&fs::read(transcript_path)?)?;
                self.index.replace_transcript(&transcript)?;
            }
        }

        Ok(())
    }
}

fn hash_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
