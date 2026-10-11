use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokeCoordinate {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleDialogue {
    pub id: usize,
    pub start_time: f64,
    pub end_time: f64,
    pub text: String,
}

pub type Subtitle = SubtitleDialogue;
