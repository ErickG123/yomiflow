use crate::AppState;
use crate::models::{HeatmapEntry, ReviewCard, StreakStats};
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
            "SELECT c.id, d.kanji, d.reading, d.meaning, c.state, d.pitch
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
                pitch: row.get(5)?,
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

    let (next_due_date, next_reps, next_state) =
        calculate_srs_step(rating, current_reps, current_state, now)?;

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
pub async fn get_study_heatmap(state: State<'_, AppState>) -> Result<Vec<HeatmapEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT strftime('%Y-%m-%d', datetime(review_datetime, 'unixepoch', 'localtime')) as date, COUNT(*) as count 
             FROM review_logs 
             GROUP BY date 
             ORDER BY date ASC",
        )
        .map_err(|e| e.to_string())?;

    let entries = stmt
        .query_map([], |row| {
            Ok(HeatmapEntry {
                date: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<HeatmapEntry>, rusqlite::Error>>()
        .map_err(|e| e.to_string())?;

    Ok(entries)
}

fn parse_ymd_to_days(ymd: &str) -> Option<i64> {
    let parts: Vec<&str> = ymd.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let y: i64 = parts[0].parse().ok()?;
    let m: i64 = parts[1].parse().ok()?;
    let d: i64 = parts[2].parse().ok()?;

    // Rata Die / Julian-like formula for days since epoch
    let m_adj = (m + 9) % 12;
    let y_adj = y - m_adj / 10;
    let days =
        365 * y_adj + y_adj / 4 - y_adj / 100 + y_adj / 400 + (m_adj * 306 + 5) / 10 + (d - 1);
    Some(days)
}

#[tauri::command]
pub async fn get_study_streaks(state: State<'_, AppState>) -> Result<StreakStats, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let today_ymd: String = conn
        .query_row(
            "SELECT strftime('%Y-%m-%d', datetime('now', 'localtime'))",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT strftime('%Y-%m-%d', datetime(review_datetime, 'unixepoch', 'localtime')) as date 
             FROM review_logs 
             ORDER BY date DESC",
        )
        .map_err(|e| e.to_string())?;

    let distinct_dates = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<String>, rusqlite::Error>>()
        .map_err(|e| e.to_string())?;

    if distinct_dates.is_empty() {
        return Ok(StreakStats {
            current_streak: 0,
            max_streak: 0,
        });
    }

    let today_days = match parse_ymd_to_days(&today_ymd) {
        Some(d) => d,
        None => {
            return Ok(StreakStats {
                current_streak: 0,
                max_streak: 0,
            });
        }
    };

    let days_list: Vec<i64> = distinct_dates
        .iter()
        .filter_map(|d| parse_ymd_to_days(d))
        .collect();

    if days_list.is_empty() {
        return Ok(StreakStats {
            current_streak: 0,
            max_streak: 0,
        });
    }

    // Calculate current streak
    let mut current_streak = 0;
    let mut expected_day = if days_list[0] == today_days {
        today_days
    } else if days_list[0] == today_days - 1 {
        today_days - 1
    } else {
        0
    };

    if expected_day != 0 {
        for &day in &days_list {
            if day == expected_day {
                current_streak += 1;
                expected_day -= 1;
            } else if day > expected_day {
                continue;
            } else {
                break;
            }
        }
    }

    // Calculate max streak across all historical study days
    let mut max_streak = 1;
    let mut streak_count = 1;

    for i in 1..days_list.len() {
        if days_list[i] == days_list[i - 1] - 1 {
            streak_count += 1;
        } else if days_list[i] < days_list[i - 1] - 1 {
            streak_count = 1;
        }
        if streak_count > max_streak {
            max_streak = streak_count;
        }
    }

    if current_streak > max_streak {
        max_streak = current_streak;
    }

    Ok(StreakStats {
        current_streak,
        max_streak,
    })
}

/// Pure business logic for FSRS-inspired interval calculation based on rating:
/// 1: Again (10 min, reset reps, state 1)
/// 2: Hard (12h if new else reps*1.2 days)
/// 3: Good (1d if new else reps*2.5 days)
/// 4: Easy (4d if new else reps*4.0 days)
pub fn calculate_srs_step(
    rating: i32,
    current_reps: i32,
    current_state: i32,
    now: i64,
) -> Result<(i64, i32, i32), String> {
    match rating {
        1 => {
            let interval = 10 * 60;
            Ok((now + interval, 0, 1))
        }
        2 => {
            let interval = if current_reps == 0 {
                12 * 60 * 60
            } else {
                ((current_reps as f64) * 1.2 * 24.0 * 60.0 * 60.0) as i64
            };
            Ok((
                now + interval,
                current_reps + 1,
                if current_state == 0 { 1 } else { 2 },
            ))
        }
        3 => {
            let interval = if current_reps == 0 {
                24 * 60 * 60
            } else {
                ((current_reps as f64) * 2.5 * 24.0 * 60.0 * 60.0) as i64
            };
            Ok((now + interval, current_reps + 1, 2))
        }
        4 => {
            let interval = if current_reps == 0 {
                4 * 24 * 60 * 60
            } else {
                let calculated = ((current_reps as f64) * 4.0 * 24.0 * 60.0 * 60.0) as i64;
                calculated.max(4 * 24 * 60 * 60)
            };
            Ok((now + interval, current_reps + 1, 2))
        }
        _ => Err("Invalid rating: must be between 1 and 4".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_srs_step_again() {
        let now = 1000000;
        let (due, reps, state) = calculate_srs_step(1, 5, 2, now).unwrap();
        assert_eq!(due, now + 10 * 60);
        assert_eq!(reps, 0);
        assert_eq!(state, 1);
    }

    #[test]
    fn test_calculate_srs_step_hard_new_card() {
        let now = 1000000;
        let (due, reps, state) = calculate_srs_step(2, 0, 0, now).unwrap();
        assert_eq!(due, now + 12 * 3600);
        assert_eq!(reps, 1);
        assert_eq!(state, 1);
    }

    #[test]
    fn test_calculate_srs_step_good_progress() {
        let now = 1000000;
        // First review
        let (due1, reps1, state1) = calculate_srs_step(3, 0, 0, now).unwrap();
        assert_eq!(due1, now + 24 * 3600);
        assert_eq!(reps1, 1);
        assert_eq!(state1, 2);

        // Second review with reps=2
        let (due2, reps2, state2) = calculate_srs_step(3, 2, 2, now).unwrap();
        let expected_interval = (2.0 * 2.5 * 86400.0) as i64;
        assert_eq!(due2, now + expected_interval);
        assert_eq!(reps2, 3);
        assert_eq!(state2, 2);
    }

    #[test]
    fn test_calculate_srs_step_easy() {
        let now = 1000000;
        let (due, reps, state) = calculate_srs_step(4, 0, 0, now).unwrap();
        assert_eq!(due, now + 4 * 86400);
        assert_eq!(reps, 1);
        assert_eq!(state, 2);
    }

    #[test]
    fn test_calculate_srs_step_invalid_rating() {
        let res = calculate_srs_step(0, 0, 0, 1000);
        assert!(res.is_err());
        let res5 = calculate_srs_step(5, 0, 0, 1000);
        assert!(res5.is_err());
    }
}
