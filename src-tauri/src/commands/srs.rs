use crate::AppState;
use crate::models::{Deck, DeckWithCounts, HeatmapEntry, ReviewCard, StreakStats, Tag};
use rusqlite::params;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn add_to_srs(
    entry_id: i64,
    deck_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    add_to_srs_in_conn(&conn, entry_id, deck_id)
}

pub fn add_to_srs_in_conn(
    conn: &rusqlite::Connection,
    entry_id: i64,
    deck_id: Option<String>,
) -> Result<(), String> {
    let target_deck = resolve_deck_id(conn, deck_id)?;
    let card_id = Uuid::new_v4().to_string();
    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;

    let already_in_deck: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM cards WHERE entry_id = ?1 AND deck_id = ?2)",
            params![entry_id, target_deck],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if already_in_deck {
        return Ok(());
    }

    conn.execute(
        "INSERT INTO cards (id, entry_id, deck_id, due_date, stability, difficulty, reps, state)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            card_id,
            entry_id,
            target_deck,
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

pub(crate) fn resolve_deck_id(
    conn: &rusqlite::Connection,
    deck_id: Option<String>,
) -> Result<String, String> {
    let target_deck = deck_id.unwrap_or_else(|| "default".to_string());
    let deck_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM decks WHERE id = ?1)",
            params![target_deck],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;

    if !deck_exists {
        return Err(format!("Deck '{}' does not exist", target_deck));
    }

    Ok(target_deck)
}

#[tauri::command]
pub fn remove_from_srs(
    entry_id: i64,
    deck_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Find card_id associated with this entry_id
    let card_ids = {
        let mut stmt = tx
            .prepare("SELECT id FROM cards WHERE entry_id = ?1 AND (?2 IS NULL OR deck_id = ?2)")
            .map_err(|e| e.to_string())?;

        stmt.query_map(params![entry_id, deck_id], |row| row.get::<_, String>(0))
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

        tx.execute("DELETE FROM card_tags WHERE card_id = ?1", params![card_id])
            .map_err(|e| e.to_string())?;
    }

    // Delete cards
    tx.execute(
        "DELETE FROM cards WHERE entry_id = ?1 AND (?2 IS NULL OR deck_id = ?2)",
        params![entry_id, deck_id],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_due_cards(
    deck_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ReviewCard>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT c.id, d.kanji, d.reading, d.meaning, c.state, d.pitch
             FROM cards c
             JOIN dictionary_entries d ON c.entry_id = d.id
             WHERE c.deck_id = ?1 AND (c.due_date <= strftime('%s', 'now') OR c.state = 0)
             ORDER BY c.due_date ASC
             LIMIT 20",
        )
        .map_err(|e| e.to_string())?;

    let cards = stmt
        .query_map(params![deck_id], |row| {
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
pub fn get_decks(state: State<'_, AppState>) -> Result<Vec<DeckWithCounts>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT 
                d.id, 
                d.name, 
                d.description,
                (SELECT COUNT(*) FROM cards c WHERE c.deck_id = d.id AND c.state = 0) as new_count,
                (SELECT COUNT(*) FROM cards c WHERE c.deck_id = d.id AND c.state > 0 AND c.due_date <= strftime('%s', 'now')) as due_count,
                (SELECT COUNT(*) FROM cards c WHERE c.deck_id = d.id) as total_count
             FROM decks d
             ORDER BY d.name ASC",
        )
        .map_err(|e| e.to_string())?;

    let decks = stmt
        .query_map([], |row| {
            Ok(DeckWithCounts {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                new_count: row.get(3)?,
                due_count: row.get(4)?,
                total_count: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<DeckWithCounts>, rusqlite::Error>>()
        .map_err(|e| e.to_string())?;

    Ok(decks)
}

#[tauri::command]
pub fn create_deck(
    name: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<Deck, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    create_deck_in_conn(&conn, name, description)
}

pub fn create_deck_in_conn(
    conn: &rusqlite::Connection,
    name: String,
    description: Option<String>,
) -> Result<Deck, String> {
    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO decks (id, name, description) VALUES (?1, ?2, ?3)",
        params![id, name, description],
    )
    .map_err(|e| e.to_string())?;

    Ok(Deck {
        id,
        name,
        description,
    })
}

#[tauri::command]
pub fn update_deck(
    id: String,
    name: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE decks SET name = ?1, description = ?2 WHERE id = ?3",
        params![name, description, id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_deck(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    delete_deck_in_conn(&conn, id)
}

pub fn delete_deck_in_conn(conn: &rusqlite::Connection, id: String) -> Result<(), String> {
    if id == "default" {
        return Err("O baralho padrão não pode ser excluído.".to_string());
    }

    // SQLite ON DELETE CASCADE handles deleting all associated cards automatically
    conn.execute("DELETE FROM decks WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, COALESCE(name, id), color FROM tags ORDER BY name ASC")
        .map_err(|e| e.to_string())?;

    let tags = stmt
        .query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<Tag>, rusqlite::Error>>()
        .map_err(|e| e.to_string())?;

    Ok(tags)
}

#[tauri::command]
pub fn create_tag(
    name: String,
    color: Option<String>,
    state: State<'_, AppState>,
) -> Result<Tag, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO tags (id, name, color) VALUES (?1, ?2, ?3)",
        params![id, name, color],
    )
    .map_err(|e| e.to_string())?;

    Ok(Tag { id, name, color })
}

#[tauri::command]
pub fn answer_card(card_id: String, rating: i32, state: State<'_, AppState>) -> Result<(), String> {
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    answer_card_in_conn(&mut conn, card_id, rating)
}

pub fn answer_card_in_conn(
    conn: &mut rusqlite::Connection,
    card_id: String,
    rating: i32,
) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let (current_stability, current_difficulty, current_reps, current_state): (f64, f64, i32, i32) = tx
        .query_row(
            "SELECT COALESCE(stability, 0.0), COALESCE(difficulty, 0.0), reps, state FROM cards WHERE id = ?1",
            params![&card_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|e| e.to_string())?;

    let last_review: i64 = tx
        .query_row(
            "SELECT COALESCE(MAX(review_datetime), 0) FROM review_logs WHERE card_id = ?1",
            params![&card_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;

    let fsrs_state = crate::domain::fsrs::calculate_fsrs_step(
        rating,
        current_stability,
        current_difficulty,
        current_reps,
        current_state,
        last_review,
        now,
    )?;

    tx.execute(
        "UPDATE cards SET stability = ?1, difficulty = ?2, reps = ?3, due_date = ?4, state = ?5 WHERE id = ?6",
        params![
            fsrs_state.stability,
            fsrs_state.difficulty,
            fsrs_state.reps,
            fsrs_state.due_date,
            fsrs_state.state,
            &card_id
        ],
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

#[tauri::command]
pub fn migrate_sm2_to_fsrs(state: State<'_, AppState>) -> Result<usize, String> {
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    migrate_sm2_to_fsrs_in_conn(&mut conn).map_err(|e| e.to_string())
}

pub fn migrate_sm2_to_fsrs_in_conn(
    conn: &mut rusqlite::Connection,
) -> Result<usize, rusqlite::Error> {
    let tx = conn.transaction()?;

    // Fetch cards that have stability == 0.0 or difficulty == 0.0
    let mut stmt = tx.prepare(
        "SELECT id, reps, due_date FROM cards WHERE stability = 0.0 OR difficulty = 0.0",
    )?;

    let unmigrated_cards: Vec<(String, i32, i64)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .filter_map(|r| r.ok())
        .collect();

    drop(stmt);

    let mut count = 0;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    for (card_id, reps, due_date) in unmigrated_cards {
        // Estimate interval in days from reps and due_date
        let interval_days = if reps <= 0 {
            0.0
        } else {
            let days_until_due = ((due_date - now) as f64 / 86400.0).max(1.0);
            days_until_due.max(reps as f64)
        };

        let default_ease_factor = 2.5;
        let (stability, difficulty) =
            crate::domain::fsrs::convert_sm2_to_fsrs(reps, interval_days, default_ease_factor);

        tx.execute(
            "UPDATE cards SET stability = ?1, difficulty = ?2 WHERE id = ?3",
            params![stability, difficulty, card_id],
        )?;

        count += 1;
    }

    tx.commit()?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_to_srs_is_idempotent_per_deck_and_allows_multiple_decks() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::create_tables_and_seed(&conn).unwrap();
        conn.execute(
            "INSERT INTO decks (id, name, description) VALUES (?1, ?2, ?3)",
            params!["deck-extra", "Extra", "Extra deck"],
        )
        .unwrap();

        add_to_srs_in_conn(&conn, 1, Some("default".to_string())).unwrap();
        add_to_srs_in_conn(&conn, 1, Some("deck-extra".to_string())).unwrap();
        add_to_srs_in_conn(&conn, 1, Some("deck-extra".to_string())).unwrap();

        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM cards WHERE entry_id = ?1",
                params![1],
                |row| row.get(0),
            )
            .unwrap();
        let extra_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM cards WHERE entry_id = ?1 AND deck_id = ?2",
                params![1, "deck-extra"],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(total, 2);
        assert_eq!(extra_count, 1);
    }

    #[test]
    fn add_to_srs_returns_clear_error_for_unknown_deck() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::create_tables_and_seed(&conn).unwrap();

        let error = add_to_srs_in_conn(&conn, 1, Some("missing-deck".to_string())).unwrap_err();

        assert_eq!(error, "Deck 'missing-deck' does not exist");
        let card_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM cards", [], |row| row.get(0))
            .unwrap();
        assert_eq!(card_count, 0);
    }

    #[test]
    fn test_deck_isolation_in_memory() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::create_tables_and_seed(&conn).unwrap();

        // Create two separate decks
        conn.execute(
            "INSERT INTO decks (id, name, description) VALUES (?1, ?2, ?3)",
            params![
                "deck-jlpt-n3",
                "JLPT N3",
                "Decks de gramática e vocabulário"
            ],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO decks (id, name, description) VALUES (?1, ?2, ?3)",
            params![
                "deck-anime",
                "Anime Vocab",
                "Vocabulário extraído de animes"
            ],
        )
        .unwrap();

        // Insert card in deck A
        conn.execute(
            "INSERT INTO cards (id, entry_id, deck_id, due_date, stability, difficulty, reps, state)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params!["card-jlpt-1", 1, "deck-jlpt-n3", 1000, 2.5, 0.3, 0, 0],
        ).unwrap();

        // Insert card in deck B
        conn.execute(
            "INSERT INTO cards (id, entry_id, deck_id, due_date, stability, difficulty, reps, state)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params!["card-anime-1", 2, "deck-anime", 1000, 2.5, 0.3, 0, 0],
        ).unwrap();

        // Query due cards for deck A only
        let mut stmt_a = conn.prepare(
            "SELECT c.id FROM cards c WHERE c.deck_id = ?1 AND (c.due_date <= strftime('%s', 'now') OR c.state = 0)"
        ).unwrap();

        let cards_a: Vec<String> = stmt_a
            .query_map(params!["deck-jlpt-n3"], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(cards_a.len(), 1);
        assert_eq!(cards_a[0], "card-jlpt-1");

        // Query due cards for deck B only
        let mut stmt_b = conn.prepare(
            "SELECT c.id FROM cards c WHERE c.deck_id = ?1 AND (c.due_date <= strftime('%s', 'now') OR c.state = 0)"
        ).unwrap();

        let cards_b: Vec<String> = stmt_b
            .query_map(params!["deck-anime"], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(cards_b.len(), 1);
        assert_eq!(cards_b[0], "card-anime-1");

        // Ensure deck A has no card from deck B
        assert!(!cards_a.contains(&"card-anime-1".to_string()));
    }

    #[test]
    fn test_migrate_sm2_to_fsrs() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::create_tables_and_seed(&conn).unwrap();

        // Insert legacy card with stability=0.0 and difficulty=0.0
        conn.execute(
            "INSERT INTO cards (id, entry_id, deck_id, due_date, stability, difficulty, reps, state)
             VALUES ('card-legacy', 1, 'default', 1700000000, 0.0, 0.0, 3, 2)",
            [],
        ).unwrap();

        let migrated = migrate_sm2_to_fsrs_in_conn(&mut conn).unwrap();
        assert!(migrated >= 1);

        let (stability, difficulty): (f64, f64) = conn
            .query_row(
                "SELECT stability, difficulty FROM cards WHERE id = 'card-legacy'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert!(stability > 0.0);
        assert!(difficulty > 0.0);
    }
}
