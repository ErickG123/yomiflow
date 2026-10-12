use crate::AppState;
use rusqlite::{OptionalExtension, params};
use tauri::State;

/// Tauri adapter: loads reference geometry, then delegates scoring to the domain.
#[tauri::command]
pub async fn evaluate_kanji_drawing(
    kanji: String,
    user_strokes: Vec<Vec<crate::models::StrokeCoordinate>>,
    state: State<'_, AppState>,
) -> Result<f32, String> {
    if user_strokes.is_empty() {
        return Ok(0.0);
    }

    let conn = state.db.lock().map_err(|error| error.to_string())?;
    let svg = conn
        .query_row(
            "SELECT svg_data FROM kanji_strokes WHERE kanji = ?1 LIMIT 1",
            params![kanji],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;

    let strokes = user_strokes
        .iter()
        .map(|stroke| stroke.iter().map(|point| (point.x, point.y)).collect())
        .collect::<Vec<Vec<(f32, f32)>>>();

    Ok(crate::domain::grading::evaluate_kanji_drawing(
        svg.as_deref(),
        &strokes,
    ))
}
