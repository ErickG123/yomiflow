use crate::AppState;
use crate::models::{BunsetsuChunk, LintSuggestion, MorphologicalToken, NotebookEntry};
use tauri::State;

#[tauri::command]
pub fn analyze_japanese_text(text: String) -> Result<Vec<MorphologicalToken>, String> {
    crate::domain::nlp::segment_and_tag_japanese(&text)
}

#[tauri::command]
pub fn generate_furigana(text: String) -> Result<String, String> {
    crate::domain::nlp::generate_furigana(&text)
}

#[tauri::command]
pub fn lint_japanese_text(text: String) -> Result<Vec<LintSuggestion>, String> {
    crate::domain::nlp::lint_japanese_text(&text)
}

#[tauri::command]
pub fn analyze_text_readability(
    text: String,
    state: State<'_, AppState>,
) -> Result<crate::models::ReadabilityAnalysis, String> {
    let tokens = crate::domain::nlp::extract_lexical_tokens(&text)?;
    let now = unix_timestamp()?;
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    let retention_index = crate::db::nlp::load_retention_index(&connection, &tokens)?;
    Ok(crate::domain::readability::analyze(
        &text,
        &tokens,
        &retention_index,
        now,
    ))
}

#[tauri::command]
pub fn generate_syntax_tree(text: String) -> Result<Vec<BunsetsuChunk>, String> {
    crate::domain::nlp::generate_syntax_tree(&text)
}

#[tauri::command]
pub fn inject_furigana_epub(
    html_text: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    let mastered = crate::db::nlp::mastered_kanji(&connection)?;
    crate::domain::nlp::inject_furigana_for_unlearned_words(&html_text, &mastered)
}

#[tauri::command]
pub fn save_notebook_entry(
    id: String,
    title: String,
    content_md: String,
    deck_id: Option<String>,
    next_review_date: Option<i64>,
    srs_stage: Option<i32>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let now = unix_timestamp()?.to_string();
    let entry = NotebookEntry {
        id,
        title,
        content_md,
        created_at: now.clone(),
        updated_at: now,
        deck_id,
        next_review_date,
        srs_stage,
    };
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::nlp::save_notebook_entry(&connection, &entry)
}

#[tauri::command]
pub fn link_notebook_to_deck(
    id: String,
    deck_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let updated_at = unix_timestamp()?;
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::nlp::link_notebook_entry(&connection, &id, deck_id.as_deref(), updated_at)
}

#[tauri::command]
pub fn get_notebook_entries(state: State<'_, AppState>) -> Result<Vec<NotebookEntry>, String> {
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::nlp::notebook_entries(&connection)
}

#[tauri::command]
pub fn delete_notebook_entry(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::nlp::delete_notebook_entry(&connection, &id)
}

fn unix_timestamp() -> Result<i64, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_commands_handle_empty_and_japanese_text() {
        assert!(analyze_japanese_text(String::new()).unwrap().is_empty());
        let tokens = analyze_japanese_text("私は学生です".to_owned()).unwrap();
        assert!(!tokens.is_empty());
        assert_eq!(tokens[0].pos, "noun");
    }
}
