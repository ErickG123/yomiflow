use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MorphologicalToken {
    pub surface: String,
    pub reading: Option<String>,
    pub pos: String, // "verb", "noun", "particle", or original/other category
}

#[derive(Debug, Clone)]
pub struct LexicalToken {
    pub surface: String,
    pub lemma: String,
    pub reading: String,
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnknownWordOccurrence {
    pub surface: String,
    pub lemma: String,
    pub start: usize,
    pub end: usize,
    pub retention_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReadabilityAnalysis {
    pub total_words: usize,
    pub known_words: usize,
    pub comprehension_percentage: f64,
    pub average_retention_percentage: f64,
    pub unknown_words: Vec<UnknownWordOccurrence>,
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
pub struct LintSuggestion {
    pub index_start: usize,
    pub index_end: usize,
    pub original: String,
    pub suggestion: String,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct BunsetsuChunk {
    pub id: usize,
    pub text: String,
    pub pos: String,
    pub target_id: Option<usize>,
}
