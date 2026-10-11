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
    pub deck_id: Option<String>,
    pub next_review_date: Option<i64>,
    pub srs_stage: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct BunsetsuChunk {
    pub id: usize,
    pub text: String,
    pub pos: String,
    pub target_id: Option<usize>,
}
