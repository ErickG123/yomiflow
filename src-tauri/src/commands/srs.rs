use crate::AppState;
use crate::models::{Deck, DeckWithCounts, HeatmapEntry, ReviewCard, StreakStats, Tag};
use tauri::State;

#[tauri::command]
pub fn add_to_srs(
    entry_id: i64,
    deck_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::srs::add_to_srs(&connection, entry_id, deck_id)
}

#[tauri::command]
pub fn remove_from_srs(
    entry_id: i64,
    deck_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::srs::remove_from_srs(&mut connection, entry_id, deck_id)
}

#[tauri::command]
pub fn get_due_cards(
    deck_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ReviewCard>, String> {
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::srs::due_cards(&connection, &deck_id)
}

#[tauri::command]
pub fn get_decks(state: State<'_, AppState>) -> Result<Vec<DeckWithCounts>, String> {
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::srs::decks(&connection)
}

#[tauri::command]
pub fn create_deck(
    name: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<Deck, String> {
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::srs::create_deck(&connection, name, description)
}

#[tauri::command]
pub fn update_deck(
    id: String,
    name: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::srs::update_deck(&connection, &id, &name, description.as_deref())
}

#[tauri::command]
pub fn delete_deck(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::srs::delete_deck(&connection, &id)
}

#[tauri::command]
pub fn get_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::srs::tags(&connection)
}

#[tauri::command]
pub fn create_tag(
    name: String,
    color: Option<String>,
    state: State<'_, AppState>,
) -> Result<Tag, String> {
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::srs::create_tag(&connection, name, color)
}

#[tauri::command]
pub fn answer_card(card_id: String, rating: i32, state: State<'_, AppState>) -> Result<(), String> {
    let mut connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::srs::answer_card(&mut connection, &card_id, rating)
}

#[tauri::command]
pub fn get_study_heatmap(state: State<'_, AppState>) -> Result<Vec<HeatmapEntry>, String> {
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::srs::study_heatmap(&connection)
}

#[tauri::command]
pub fn get_study_streaks(state: State<'_, AppState>) -> Result<StreakStats, String> {
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::srs::study_streaks(&connection)
}

#[tauri::command]
pub fn migrate_sm2_to_fsrs(state: State<'_, AppState>) -> Result<usize, String> {
    let mut connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::srs::migrate_sm2_to_fsrs(&mut connection)
}
