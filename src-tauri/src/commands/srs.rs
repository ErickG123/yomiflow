use crate::AppState;
use crate::models::ReviewCard;
use rusqlite::params;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn add_to_srs(entry_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let card_id = Uuid::new_v4().to_string();
    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;

    conn.execute(
        "INSERT INTO cards (id, entry_id, due_date, stability, difficulty, reps, state)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            card_id,
            entry_id,
            current_timestamp,
            0.0_f64,
            0.0_f64,
            0_i32,
            0_i32 // 0: New
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn remove_from_srs(entry_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Find card_id associated with this entry_id
    let card_ids = {
        let mut stmt = tx
            .prepare("SELECT id FROM cards WHERE entry_id = ?1")
            .map_err(|e| e.to_string())?;

        stmt.query_map(params![entry_id], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<String>, rusqlite::Error>>()
            .map_err(|e| e.to_string())?
    };

    // Delete review logs for each card
    for card_id in &card_ids {
        tx.execute(
            "DELETE FROM review_logs WHERE card_id = ?1",
            params![card_id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Delete cards
    tx.execute("DELETE FROM cards WHERE entry_id = ?1", params![entry_id])
        .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_due_cards(state: State<'_, AppState>) -> Result<Vec<ReviewCard>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT c.id, d.kanji, d.reading, d.meaning, c.state
             FROM cards c
             JOIN dictionary_entries d ON c.entry_id = d.id
             WHERE c.due_date <= strftime('%s', 'now') OR c.state = 0
             ORDER BY c.due_date ASC
             LIMIT 20",
        )
        .map_err(|e| e.to_string())?;

    let cards = stmt
        .query_map([], |row| {
            Ok(ReviewCard {
                card_id: row.get(0)?,
                kanji: row.get(1)?,
                reading: row.get(2)?,
                meaning: row.get(3)?,
                state: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<ReviewCard>, rusqlite::Error>>()
        .map_err(|e| e.to_string())?;

    Ok(cards)
}

#[tauri::command]
pub fn answer_card(card_id: String, rating: i32, state: State<'_, AppState>) -> Result<(), String> {
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let (current_reps, current_state): (i32, i32) = tx
        .query_row(
            "SELECT reps, state FROM cards WHERE id = ?1",
            params![&card_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;

    let (next_due_date, next_reps, next_state) = match rating {
        1 => {
            // Again: 10 minutes, reps reset to 0, state: Learning (1)
            let interval = 10 * 60;
            (now + interval, 0, 1)
        }
        2 => {
            // Hard: if current_reps == 0 then 12 hours else reps * 1.2 days, state: Review (2)
            let interval = if current_reps == 0 {
                12 * 60 * 60
            } else {
                ((current_reps as f64) * 1.2 * 24.0 * 60.0 * 60.0) as i64
            };
            (
                now + interval,
                current_reps + 1,
                if current_state == 0 { 1 } else { 2 },
            )
        }
        3 => {
            // Good: 1 day if reps == 0, otherwise reps * 2.5 days, state: Review (2)
            let interval = if current_reps == 0 {
                24 * 60 * 60
            } else {
                ((current_reps as f64) * 2.5 * 24.0 * 60.0 * 60.0) as i64
            };
            (now + interval, current_reps + 1, 2)
        }
        4 => {
            // Easy: minimum 4 days if reps == 0, otherwise reps * 4 days, state: Review (2)
            let interval = if current_reps == 0 {
                4 * 24 * 60 * 60
            } else {
                let calculated = ((current_reps as f64) * 4.0 * 24.0 * 60.0 * 60.0) as i64;
                calculated.max(4 * 24 * 60 * 60)
            };
            (now + interval, current_reps + 1, 2)
        }
        _ => return Err("Invalid rating: must be between 1 and 4".to_string()),
    };

    tx.execute(
        "UPDATE cards SET reps = ?1, due_date = ?2, state = ?3 WHERE id = ?4",
        params![next_reps, next_due_date, next_state, &card_id],
    )
    .map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO review_logs (card_id, rating, review_datetime) VALUES (?1, ?2, ?3)",
        params![&card_id, rating, now],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(())
}
