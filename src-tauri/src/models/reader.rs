use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaVolume {
    pub title: String,
    pub filename: String,
    pub filepath: String,
}
