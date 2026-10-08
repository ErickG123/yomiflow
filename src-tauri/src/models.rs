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
pub struct SrsCard {
    pub id: String,
    pub entry_id: i64,
    pub due_date: i64,
    pub stability: f64,
    pub difficulty: f64,
    pub reps: i32,
    pub state: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReviewLog {
    pub id: i64,
    pub card_id: String,
    pub rating: i32,
    pub review_datetime: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReviewCard {
    pub card_id: String,
    pub kanji: Option<String>,
    pub reading: String,
    pub meaning: String,
    pub state: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardStats {
    pub due_today: i32,
    pub new_cards: i32,
    pub total_cards: i32,
}
