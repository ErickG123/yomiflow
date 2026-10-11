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
pub struct KanjiComponent {
    pub kanji: String,
    pub component: String,
    pub meaning: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KanjiDetails {
    pub kanji: String,
    pub components: Vec<KanjiComponent>,
    pub mnemonic: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JlptLevelCoverage {
    pub level: i32,
    pub level_name: String,
    pub total_kanji: i32,
    pub learned_kanji: i32,
    pub kanji_percentage: f64,
    pub total_vocab: i32,
    pub learned_vocab: i32,
    pub vocab_percentage: f64,
    pub overall_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JlptAnalyticsStats {
    pub levels: Vec<JlptLevelCoverage>,
    pub total_kanji_learned: i32,
    pub total_vocab_learned: i32,
}
