use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct AudioTrackMetadata {
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub cover_data_url: Option<String>,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct AudioPlayerSnapshot {
    pub track: Option<AudioTrackMetadata>,
    pub is_playing: bool,
    pub position_ms: u64,
    pub volume: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct PassiveListeningEntry {
    pub id: String,
    pub track_path: String,
    pub title: String,
    pub artist: String,
    pub listened_ms: u64,
    pub started_at: String,
    pub updated_at: String,
}
