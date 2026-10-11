use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KanaMasteryRecord {
    pub character: String,
    pub romaji: String,
    pub kana_type: String, // "hiragana" or "katakana"
    pub alphabet: String,
    pub category: String, // "base", "dakuon", "yoon"
    pub correct_count: i32,
    pub wrong_count: i32,
    pub total_attempts: i32,
    pub correct_attempts: i32,
    pub accuracy_percentage: f64,
    pub last_practiced: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KanaItem {
    pub character: String,
    pub romaji: String,
    pub alphabet: String, // "hiragana" | "katakana"
    pub category: String, // "base" | "dakuon" | "yoon"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KanaProficiencyStats {
    pub hiragana_proficiency: f64, // 0.0 to 100.0
    pub katakana_proficiency: f64, // 0.0 to 100.0
    pub overall_proficiency: f64,  // 0.0 to 100.0
    pub total_attempts: i32,
    pub total_correct: i32,
}
