use std::path::Path;

use opentranscribe_domain::{Project, SearchFilters, Session, SessionLifecycle, Transcript};
use rusqlite::{Connection, params, params_from_iter, types::Value};

use crate::error::AppResult;

pub struct LibraryIndex {
    connection: Connection,
}

pub struct IndexedSearchResult {
    pub session_id: String,
    pub session_title: String,
    pub kind: String,
    pub entity_id: String,
    pub excerpt: String,
}

impl LibraryIndex {
    pub fn open(path: &Path) -> AppResult<Self> {
        let connection = Connection::open(path)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        connection.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                relative_path TEXT NOT NULL,
                revision INTEGER NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                project_id TEXT,
                title TEXT NOT NULL,
                relative_path TEXT NOT NULL,
                lifecycle TEXT NOT NULL,
                revision INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                duration_ms INTEGER NOT NULL
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS search_fts USING fts5(
                session_id UNINDEXED,
                kind UNINDEXED,
                entity_id UNINDEXED,
                text,
                tokenize = 'unicode61'
            );
            ",
        )?;
        connection.execute_batch("DROP TABLE IF EXISTS jobs;")?;

        Ok(Self { connection })
    }

    pub fn clear_content(&self) -> AppResult<()> {
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute_batch(
            "
            DELETE FROM search_fts;
            DELETE FROM sessions;
            DELETE FROM projects;
            ",
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn replace_project(&self, project: &Project, relative_path: &str) -> AppResult<()> {
        self.connection.execute(
            "
            INSERT INTO projects (id, name, relative_path, revision, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                relative_path = excluded.relative_path,
                revision = excluded.revision,
                updated_at = excluded.updated_at
            ",
            params![
                project.id,
                project.name,
                relative_path,
                i64::try_from(project.revision).expect("project revision must fit SQLite integer"),
                project.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn replace_session(&self, session: &Session, relative_path: &str) -> AppResult<()> {
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(
            "
            INSERT INTO sessions (
                id, project_id, title, relative_path, lifecycle, revision, created_at, duration_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                project_id = excluded.project_id,
                title = excluded.title,
                relative_path = excluded.relative_path,
                lifecycle = excluded.lifecycle,
                revision = excluded.revision,
                duration_ms = excluded.duration_ms
            ",
            params![
                session.id,
                session.project_id,
                session.title,
                relative_path,
                lifecycle_name(&session.lifecycle),
                i64::try_from(session.revision).expect("session revision must fit SQLite integer"),
                session.created_at.to_rfc3339(),
                i64::try_from(session.duration_ms)
                    .expect("session duration must fit SQLite integer"),
            ],
        )?;
        transaction.execute(
            "DELETE FROM search_fts WHERE session_id = ?1 AND kind = 'session'",
            params![session.id],
        )?;
        transaction.execute(
            "
            INSERT INTO search_fts (session_id, kind, entity_id, text)
            VALUES (?1, 'session', ?1, ?2)
            ",
            params![session.id, session.title],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn remove_session(&self, session_id: &str) -> AppResult<()> {
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(
            "DELETE FROM search_fts WHERE session_id = ?1",
            params![session_id],
        )?;
        transaction.execute("DELETE FROM sessions WHERE id = ?1", params![session_id])?;
        transaction.commit()?;
        Ok(())
    }

    pub fn replace_transcript(&self, transcript: &Transcript) -> AppResult<()> {
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(
            "DELETE FROM search_fts WHERE session_id = ?1 AND kind = 'transcript'",
            params![transcript.session_id],
        )?;

        for segment in &transcript.segments {
            transaction.execute(
                "
                INSERT INTO search_fts (session_id, kind, entity_id, text)
                VALUES (?1, 'transcript', ?2, ?3)
                ",
                params![transcript.session_id, segment.id, segment.text],
            )?;
        }

        transaction.commit()?;
        Ok(())
    }

    pub fn replace_notes(&self, session_id: &str, markdown: &str) -> AppResult<()> {
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(
            "DELETE FROM search_fts WHERE session_id = ?1 AND kind = 'notes'",
            params![session_id],
        )?;

        if !markdown.trim().is_empty() {
            transaction.execute(
                "
                INSERT INTO search_fts (session_id, kind, entity_id, text)
                VALUES (?1, 'notes', ?1, ?2)
                ",
                params![session_id, markdown],
            )?;
        }

        transaction.commit()?;
        Ok(())
    }

    pub fn search(
        &self,
        query: &str,
        filters: &SearchFilters,
    ) -> AppResult<Vec<IndexedSearchResult>> {
        let terms = query
            .split_whitespace()
            .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
            .collect::<Vec<_>>();

        if terms.is_empty() {
            return Ok(Vec::new());
        }

        let mut sql = String::from(
            "
            SELECT
                search_fts.session_id,
                sessions.title,
                search_fts.kind,
                search_fts.entity_id,
                snippet(search_fts, 3, '', '', ' … ', 24)
            FROM search_fts
            INNER JOIN sessions ON sessions.id = search_fts.session_id
            WHERE search_fts MATCH ?1
              AND sessions.lifecycle != 'trashed'
            ",
        );
        let mut values = vec![Value::Text(terms.join(" "))];

        if let Some(project_id) = &filters.project_id {
            values.push(Value::Text(project_id.clone()));
            sql.push_str(&format!(" AND sessions.project_id = ?{}", values.len()));
        }

        if !filters.content_kinds.is_empty() {
            let mut placeholders = Vec::new();

            for content_kind in &filters.content_kinds {
                values.push(Value::Text(content_kind.clone()));
                placeholders.push(format!("?{}", values.len()));
            }

            sql.push_str(&format!(
                " AND search_fts.kind IN ({})",
                placeholders.join(", ")
            ));
        }

        sql.push_str(" ORDER BY bm25(search_fts), sessions.created_at DESC LIMIT 100");

        let mut statement = self.connection.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(values.iter()), |row| {
            Ok(IndexedSearchResult {
                session_id: row.get(0)?,
                session_title: row.get(1)?,
                kind: row.get(2)?,
                entity_id: row.get(3)?,
                excerpt: row.get(4)?,
            })
        })?;

        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}

fn lifecycle_name(lifecycle: &SessionLifecycle) -> &'static str {
    match lifecycle {
        SessionLifecycle::Draft => "draft",
        SessionLifecycle::Recording => "recording",
        SessionLifecycle::Paused => "paused",
        SessionLifecycle::Finalizing => "finalizing",
        SessionLifecycle::Ready => "ready",
        SessionLifecycle::NeedsAttention => "needs_attention",
        SessionLifecycle::Recovered => "recovered",
        SessionLifecycle::Trashed => "trashed",
    }
}
