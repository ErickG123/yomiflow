use crate::models::{DashboardStats, DictionaryEntry, ReviewCard};
use rusqlite::{OptionalExtension, params};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;
use uuid::Uuid;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
}

#[tauri::command]
pub fn search_dictionary(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<DictionaryEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let search_pattern = format!("%{}%", query);

    let mut stmt = conn
        .prepare(
            "SELECT d.id, d.kanji, d.reading, d.meaning, CASE WHEN c.id IS NOT NULL THEN 1 ELSE 0 END as in_srs
             FROM dictionary_entries d
             LEFT JOIN cards c ON d.id = c.entry_id
             WHERE d.kanji LIKE ?1 OR d.reading LIKE ?1 OR d.meaning LIKE ?1 
             LIMIT 50",
        )
        .map_err(|e| e.to_string())?;

    let entries = stmt
        .query_map(params![search_pattern], |row| {
            Ok(DictionaryEntry {
                id: row.get(0)?,
                kanji: row.get(1)?,
                reading: row.get(2)?,
                meaning: row.get(3)?,
                in_srs: row.get::<_, i32>(4)? == 1,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<DictionaryEntry>, rusqlite::Error>>()
        .map_err(|e| e.to_string())?;

    Ok(entries)
}

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
            0_i32
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_due_cards(state: State<'_, AppState>) -> Result<Vec<ReviewCard>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT cards.id, dictionary_entries.kanji, dictionary_entries.reading, dictionary_entries.meaning, cards.state
             FROM cards
             INNER JOIN dictionary_entries ON cards.entry_id = dictionary_entries.id
             WHERE cards.state = 0 OR cards.due_date <= strftime('%s', 'now')
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

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let (current_reps, _current_state): (i32, i32) = tx
        .query_row(
            "SELECT reps, state FROM cards WHERE id = ?1",
            params![&card_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    let (next_due_date, next_reps, next_state) = match rating {
        1 => {
            // Again: 1 minute, reps resets to 0, state: Learning (1)
            (now + 60, 0, 1)
        }
        2 => {
            // Hard: 12 hours, reps + 1, state: Learning (1)
            (now + (12 * 60 * 60), current_reps + 1, 1)
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

    Ok(DashboardStats {
        due_today,
        new_cards,
        total_cards,
    })
}

#[tauri::command]
pub fn import_dictionary(folder_path: String, state: State<'_, AppState>) -> Result<usize, String> {
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    let path = Path::new(&folder_path);

    if !path.exists() || !path.is_dir() {
        return Err("Provided path is not a valid directory".to_string());
    }

    let entries = fs::read_dir(path).map_err(|e| e.to_string())?;
    let mut total_imported = 0;

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_path = entry.path();

        if file_path.is_file()
            && file_path
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        {
            let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Yomichan term banks usually named term_bank_*.json
            if (file_name.starts_with("term_bank") || file_name.ends_with(".json"))
                && let Ok(content) = fs::read_to_string(&file_path)
                && let Ok(json) = serde_json::from_str::<Value>(&content)
                && let Some(items) = json.as_array()
            {
                let mut stmt = tx
                    .prepare(
                        "INSERT INTO dictionary_entries (kanji, reading, meaning) 
                         VALUES (?1, ?2, ?3)",
                    )
                    .map_err(|e| e.to_string())?;

                for item in items {
                    if let Some(arr) = item.as_array() {
                        // Yomichan format: [kanji/term, reading, definition_tags, rules, score, [glossary...], sequence, term_tags]
                        if !arr.is_empty() {
                            let term = arr.first().and_then(|v| v.as_str()).unwrap_or("");
                            let reading = arr.get(1).and_then(|v| v.as_str()).unwrap_or(term);

                            let meaning = if let Some(defs) = arr.get(5).and_then(|v| v.as_array())
                            {
                                defs.iter()
                                    .filter_map(|d| {
                                        d.as_str()
                                            .or_else(|| d.get("content").and_then(|c| c.as_str()))
                                    })
                                    .collect::<Vec<_>>()
                                    .join("; ")
                            } else {
                                arr.get(5)
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string()
                            };

                            if !term.is_empty() || !reading.is_empty() {
                                stmt.execute(params![
                                    if term == reading {
                                        None::<String>
                                    } else {
                                        Some(term.to_string())
                                    },
                                    reading,
                                    meaning
                                ])
                                .map_err(|e| e.to_string())?;
                                total_imported += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok(total_imported)
}

#[tauri::command]
pub fn get_setting(key: String, state: State<'_, AppState>) -> Result<Option<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = ?1")
        .map_err(|e| e.to_string())?;

    let result = stmt
        .query_row(params![key], |row| row.get::<_, String>(0))
        .optional();

    match result {
        Ok(opt) => Ok(opt),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn set_setting(key: String, value: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
