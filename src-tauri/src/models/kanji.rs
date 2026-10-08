use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KanjiEntry {
    pub character: String,
    pub onyomi: Option<String>,
    pub kunyomi: Option<String>,
    pub meaning: String,
    pub jlpt_level: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JlptKanjiEntry {
    pub id: Option<i64>,
    pub kanji: Option<String>,
    pub character: Option<String>,
    pub utf: Option<String>,
    pub jlpt: Option<serde_json::Value>,
    pub jlpt_level: Option<i32>,
    pub description: Option<String>,
    pub meaning: Option<serde_json::Value>,
    pub meanings: Option<Vec<String>>,
    pub onyomi: Option<serde_json::Value>,
    pub kunyomi: Option<serde_json::Value>,
    pub on_readings: Option<Vec<String>>,
    pub kun_readings: Option<Vec<String>>,
    pub readings: Option<serde_json::Value>,
    pub strokes: Option<serde_json::Value>,
}
