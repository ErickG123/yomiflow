use crate::AppState;
use crate::models::DashboardStats;
use tauri::{Emitter, State, Window};

#[tauri::command]
pub fn get_dashboard_stats(state: State<'_, AppState>) -> Result<DashboardStats, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let total_cards: i32 = conn
        .query_row("SELECT COUNT(*) FROM cards", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let new_cards: i32 = conn
        .query_row("SELECT COUNT(*) FROM cards WHERE state = 0", [], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?;

    let due_today: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM cards WHERE state > 0 AND due_date <= strftime('%s', 'now')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let total_dictionary_words: i32 = conn
        .query_row("SELECT COUNT(*) FROM dictionary_entries", [], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?;

    let total_kanjis: i32 = conn
        .query_row("SELECT COUNT(*) FROM kanjis", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let total_tags: i32 = conn
        .query_row("SELECT COUNT(*) FROM tags", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    Ok(DashboardStats {
        due_today,
        new_cards,
        total_cards,
        total_dictionary_words,
        total_kanjis,
        total_tags,
    })
}

#[tauri::command]
pub async fn clear_database(window: Window, state: State<'_, AppState>) -> Result<(), String> {
    let _ = window.emit("backend-progress", "A limpar tabelas e libertar espaço...");

    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute("DELETE FROM review_logs", [])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM cards", [])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM dictionary_entries", [])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM kanjis", [])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM tags", [])
        .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    conn.execute("VACUUM", []).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn check_onboarding_needed(state: State<'_, AppState>) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let dictionary_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM dictionary_entries", [], |row| row.get(0))
        .unwrap_or(0);

    let kanji_strokes_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM kanji_strokes", [], |row| row.get(0))
        .unwrap_or(0);

    // If dictionary has only seed data (<= 3) and no kanji strokes imported yet
    Ok(dictionary_count <= 3 || kanji_strokes_count == 0)
}
