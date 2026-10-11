use crate::AppState;
use rusqlite::params;
use tauri::State;
use uuid::Uuid;

const CATEGORIES: [&str; 3] = ["active_reading", "passive_listening", "srs_review"];

#[tauri::command]
pub fn save_study_session(
    category: String,
    duration_minutes: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !CATEGORIES.contains(&category.as_str()) {
        return Err("Invalid study category".to_string());
    }
    if duration_minutes == 0 || duration_minutes > 180 {
        return Err("Duration must be between 1 and 180 minutes".to_string());
    }

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO study_sessions (id, category, duration_minutes, date) VALUES (?1, ?2, ?3, datetime('now', 'localtime'))",
        params![Uuid::new_v4().to_string(), category, duration_minutes],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
