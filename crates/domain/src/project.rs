use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::SCHEMA_VERSION;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Project {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    #[ts(type = "number")]
    pub revision: u64,
    pub glossary: Vec<String>,
    pub openai_profile_id: Option<String>,
    pub language_hint: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    pub fn new(name: String) -> Self {
        let timestamp = crate::now();

        Self {
            schema_version: SCHEMA_VERSION,
            id: crate::new_id(),
            name,
            revision: 1,
            glossary: Vec::new(),
            openai_profile_id: None,
            language_hint: None,
            created_at: timestamp,
            updated_at: timestamp,
        }
    }
}
