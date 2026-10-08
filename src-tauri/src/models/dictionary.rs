use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DictionaryEntry {
    pub id: i64,
    pub kanji: Option<String>,
    pub reading: String,
    pub meaning: String,
    pub in_srs: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TagEntry {
    pub id: String,
    pub category: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ImportResult {
    pub terms_imported: usize,
    pub kanjis_imported: usize,
    pub tags_imported: usize,
    pub total_imported: usize,
}
