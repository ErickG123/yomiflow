use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NhkNewsItem {
    pub id: String,
    pub title: String,
    pub title_with_ruby: Option<String>,
    pub body_text: String,
    pub body_html: String,
    pub image_url: Option<String>,
    pub audio_url: Option<String>,
    pub published_at: Option<String>,
    pub web_url: Option<String>,
}
