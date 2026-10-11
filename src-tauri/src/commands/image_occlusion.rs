use crate::AppState;
use crate::models::{ImageOcclusionCard, ImageOcclusionSummary, OcclusionMask};
use base64::Engine;
use rusqlite::params;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

#[tauri::command]
pub fn save_image_occlusion_card(
    image_data: String,
    masks: Vec<OcclusionMask>,
    deck_id: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    if masks.is_empty() || masks.len() > 100 {
        return Err("A card must contain between 1 and 100 masks".to_string());
    }
    for mask in &masks {
        if !mask.x.is_finite()
            || !mask.y.is_finite()
            || !mask.width.is_finite()
            || !mask.height.is_finite()
            || mask.x < 0.0
            || mask.y < 0.0
            || mask.width <= 0.0
            || mask.height <= 0.0
            || mask.x + mask.width > 1.0
            || mask.y + mask.height > 1.0
        {
            return Err("Mask coordinates must be normalized between 0 and 1".to_string());
        }
    }

    let (mime, encoded) = image_data
        .split_once(',')
        .ok_or_else(|| "Image data must be a base64 data URL".to_string())?;
    let extension = match mime {
        "data:image/png;base64" => "png",
        "data:image/jpeg;base64" => "jpg",
        "data:image/webp;base64" => "webp",
        _ => return Err("Only PNG, JPEG, and WebP images are supported".to_string()),
    };
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| e.to_string())?;
    if bytes.is_empty() || bytes.len() > 15 * 1024 * 1024 {
        return Err("Image must be smaller than 15 MB".to_string());
    }

    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let image_dir = app_data.join("image_occlusion");
    fs::create_dir_all(&image_dir).map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let file_path = image_dir.join(format!("{id}.{extension}"));
    fs::write(&file_path, bytes).map_err(|e| e.to_string())?;

    let masks_json = serde_json::to_string(&masks).map_err(|e| e.to_string())?;
    let deck_id = deck_id.unwrap_or_else(|| "default".to_string());
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    if let Err(error) = conn.execute(
        "INSERT INTO image_occlusion_cards (id, deck_id, image_path, masks_json) VALUES (?1, ?2, ?3, ?4)",
        params![id, deck_id, file_path.to_string_lossy(), masks_json],
    ) {
        let _ = fs::remove_file(file_path);
        return Err(error.to_string());
    }
    Ok(id)
}

#[tauri::command]
pub fn list_image_occlusion_cards(
    state: State<'_, AppState>,
) -> Result<Vec<ImageOcclusionSummary>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut statement = conn
        .prepare("SELECT id, deck_id, masks_json, created_at, state FROM image_occlusion_cards WHERE state = 0 OR due_date <= strftime('%s', 'now') ORDER BY due_date, created_at DESC")
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i32>(4)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut cards = Vec::new();
    for row in rows {
        let (id, deck_id, masks_json, created_at, state) = row.map_err(|e| e.to_string())?;
        let masks: Vec<OcclusionMask> =
            serde_json::from_str(&masks_json).map_err(|e| e.to_string())?;
        cards.push(ImageOcclusionSummary {
            id,
            deck_id,
            mask_count: masks.len(),
            created_at,
            state,
        });
    }
    Ok(cards)
}

#[tauri::command]
pub fn answer_image_occlusion_card(
    card_id: String,
    rating: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let (stability, difficulty, reps, card_state, last_review): (f64, f64, i32, i32, i64) = tx
        .query_row(
            "SELECT stability, difficulty, reps, state, last_review FROM image_occlusion_cards WHERE id = ?1",
            params![card_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .map_err(|e| e.to_string())?;
    let schedule = crate::domain::fsrs::calculate_fsrs_step(
        rating,
        stability,
        difficulty,
        reps,
        card_state,
        last_review,
        now,
    )?;
    tx.execute(
        "UPDATE image_occlusion_cards SET stability = ?1, difficulty = ?2, reps = ?3, state = ?4, due_date = ?5, last_review = ?6 WHERE id = ?7",
        params![schedule.stability, schedule.difficulty, schedule.reps, schedule.state, schedule.due_date, now, card_id],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO image_occlusion_review_logs (card_id, rating, review_datetime) VALUES (?1, ?2, ?3)",
        params![card_id, rating, now],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_image_occlusion_card(
    card_id: String,
    state: State<'_, AppState>,
) -> Result<ImageOcclusionCard, String> {
    let (id, deck_id, path, masks_json, created_at) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT id, deck_id, image_path, masks_json, created_at FROM image_occlusion_cards WHERE id = ?1",
            params![card_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?)),
        )
        .map_err(|e| e.to_string())?
    };
    let image_bytes = fs::read(&path).map_err(|e| e.to_string())?;
    let extension = std::path::Path::new(&path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("png");
    let mime = match extension {
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        _ => "image/png",
    };
    Ok(ImageOcclusionCard {
        id,
        deck_id,
        image_data: format!(
            "data:{mime};base64,{}",
            base64::engine::general_purpose::STANDARD.encode(image_bytes)
        ),
        masks: serde_json::from_str(&masks_json).map_err(|e| e.to_string())?,
        created_at,
    })
}
