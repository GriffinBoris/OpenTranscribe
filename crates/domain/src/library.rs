use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{AppSettings, Job, Project, SCHEMA_VERSION, Session};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct LibraryManifest {
    pub schema_version: u32,
    pub id: String,
    pub created_at: DateTime<Utc>,
}

impl LibraryManifest {
    pub fn new() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            id: crate::new_id(),
            created_at: crate::now(),
        }
    }
}

impl Default for LibraryManifest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct LibraryDescriptor {
    pub id: String,
    pub path: String,
    #[ts(type = "number")]
    pub project_count: usize,
    #[ts(type = "number")]
    pub session_count: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct AppSnapshot {
    pub library: Option<LibraryDescriptor>,
    pub projects: Vec<Project>,
    pub recent_sessions: Vec<Session>,
    pub active_jobs: Vec<Job>,
    pub settings: AppSettings,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct SearchFilters {
    pub project_id: Option<String>,
    pub content_kinds: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct SearchResult {
    pub session_id: String,
    pub session_title: String,
    pub kind: String,
    pub excerpt: String,
    #[ts(type = "number | null")]
    pub timestamp_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct SearchPage {
    pub results: Vec<SearchResult>,
    pub next_cursor: Option<String>,
}
