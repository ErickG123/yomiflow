use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardStats {
    pub due_today: i32,
    pub new_cards: i32,
    pub total_cards: i32,
    pub total_dictionary_words: i32,
    pub total_kanjis: i32,
    pub total_tags: i32,
}
