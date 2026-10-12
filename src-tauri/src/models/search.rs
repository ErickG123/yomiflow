use serde::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub id: String,
    pub source_type: String,
    pub title: String,
    pub snippet: String,
}
