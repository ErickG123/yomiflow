use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcclusionMask {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageOcclusionCard {
    pub id: String,
    pub deck_id: String,
    pub image_data: String,
    pub masks: Vec<OcclusionMask>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageOcclusionSummary {
    pub id: String,
    pub deck_id: String,
    pub mask_count: usize,
    pub created_at: String,
    pub state: i32,
}
