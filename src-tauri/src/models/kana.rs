use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KanaMasteryRecord {
    pub character: String,
    pub romaji: String,
    pub kana_type: String, // "hiragana" or "katakana"
    pub total_attempts: i32,
    pub correct_attempts: i32,
    pub accuracy_percentage: f64,
    pub last_practiced: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KanaProficiencyStats {
    pub hiragana_proficiency: f64, // 0.0 to 100.0
    pub katakana_proficiency: f64, // 0.0 to 100.0
    pub overall_proficiency: f64,  // 0.0 to 100.0
    pub total_attempts: i32,
    pub total_correct: i32,
}
