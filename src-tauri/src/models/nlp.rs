use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MorphologicalToken {
    pub surface: String,
    pub reading: Option<String>,
    pub pos: String, // "verb", "noun", "particle", or original/other category
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotebookEntry {
    pub id: String,
    pub title: String,
    pub content_md: String,
    pub created_at: String,
    pub updated_at: String,
}
