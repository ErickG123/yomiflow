use crate::models::{Deck, DeckWithCounts, HeatmapEntry, ReviewCard, StreakStats, Tag};
use rusqlite::{Connection, params};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub fn add_to_srs(
    connection: &Connection,
    entry_id: i64,
    deck_id: Option<String>,
) -> Result<(), String> {
    let target_deck = resolve_deck_id(connection, deck_id)?;
    let exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM cards WHERE entry_id = ?1 AND deck_id = ?2)",
            params![entry_id, target_deck],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if exists {
        return Ok(());
    }
    connection
        .execute(
            "INSERT INTO cards (id, entry_id, deck_id, due_date, stability, difficulty, reps, state)
             VALUES (?1, ?2, ?3, ?4, 0.0, 0.0, 0, 0)",
            params![
                Uuid::new_v4().to_string(),
                entry_id,
                target_deck,
                unix_timestamp()?
            ],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn resolve_deck_id(connection: &Connection, deck_id: Option<String>) -> Result<String, String> {
    let target = deck_id.unwrap_or_else(|| "default".to_owned());
    let exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM decks WHERE id = ?1)",
            params![target],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if !exists {
        return Err(format!("Deck '{target}' does not exist"));
    }
    Ok(target)
}

pub fn add_to_srs_in_conn(
    connection: &Connection,
    entry_id: i64,
    deck_id: Option<String>,
) -> Result<(), String> {
    add_to_srs(connection, entry_id, deck_id)
}

pub fn remove_from_srs(
    connection: &mut Connection,
    entry_id: i64,
    deck_id: Option<String>,
) -> Result<(), String> {
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    let card_ids = {
        let mut statement = transaction
            .prepare("SELECT id FROM cards WHERE entry_id = ?1 AND (?2 IS NULL OR deck_id = ?2)")
            .map_err(|error| error.to_string())?;
        statement
            .query_map(params![entry_id, deck_id], |row| row.get::<_, String>(0))
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?
    };
    for card_id in card_ids {
        transaction
            .execute(
                "DELETE FROM review_logs WHERE card_id = ?1",
                params![card_id],
            )
            .map_err(|error| error.to_string())?;
        transaction
            .execute("DELETE FROM card_tags WHERE card_id = ?1", params![card_id])
            .map_err(|error| error.to_string())?;
    }
    transaction
        .execute(
            "DELETE FROM cards WHERE entry_id = ?1 AND (?2 IS NULL OR deck_id = ?2)",
            params![entry_id, deck_id],
        )
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())
}

pub fn due_cards(connection: &Connection, deck_id: &str) -> Result<Vec<ReviewCard>, String> {
    let mut statement = connection
        .prepare(
            "SELECT c.id, d.kanji, d.reading, d.meaning, c.state, d.pitch
             FROM cards c JOIN dictionary_entries d ON c.entry_id = d.id
             WHERE c.deck_id = ?1 AND (c.due_date <= strftime('%s', 'now') OR c.state = 0)
             ORDER BY c.due_date ASC LIMIT 20",
        )
        .map_err(|error| error.to_string())?;
    statement
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
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn decks(connection: &Connection) -> Result<Vec<DeckWithCounts>, String> {
    let mut statement = connection
        .prepare(
            "SELECT d.id, d.name, d.description,
                (SELECT COUNT(*) FROM cards c WHERE c.deck_id = d.id AND c.state = 0),
                (SELECT COUNT(*) FROM cards c WHERE c.deck_id = d.id AND c.state > 0 AND c.due_date <= strftime('%s', 'now')),
                (SELECT COUNT(*) FROM cards c WHERE c.deck_id = d.id)
             FROM decks d ORDER BY d.name ASC",
        )
        .map_err(|error| error.to_string())?;
    statement
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
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn create_deck(
    connection: &Connection,
    name: String,
    description: Option<String>,
) -> Result<Deck, String> {
    let deck = Deck {
        id: Uuid::new_v4().to_string(),
        name,
        description,
    };
    connection
        .execute(
            "INSERT INTO decks (id, name, description) VALUES (?1, ?2, ?3)",
            params![deck.id, deck.name, deck.description],
        )
        .map_err(|error| error.to_string())?;
    Ok(deck)
}

pub fn create_deck_in_conn(
    connection: &Connection,
    name: String,
    description: Option<String>,
) -> Result<Deck, String> {
    create_deck(connection, name, description)
}

pub fn update_deck(
    connection: &Connection,
    id: &str,
    name: &str,
    description: Option<&str>,
) -> Result<(), String> {
    connection
        .execute(
            "UPDATE decks SET name = ?1, description = ?2 WHERE id = ?3",
            params![name, description, id],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn delete_deck(connection: &Connection, id: &str) -> Result<(), String> {
    if id == "default" {
        return Err("The default deck cannot be deleted.".to_owned());
    }
    connection
        .execute("DELETE FROM decks WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn delete_deck_in_conn(connection: &Connection, id: String) -> Result<(), String> {
    delete_deck(connection, &id)
}

pub fn tags(connection: &Connection) -> Result<Vec<Tag>, String> {
    let mut statement = connection
        .prepare("SELECT id, COALESCE(name, id), color FROM tags ORDER BY name ASC")
        .map_err(|error| error.to_string())?;
    statement
        .query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn create_tag(
    connection: &Connection,
    name: String,
    color: Option<String>,
) -> Result<Tag, String> {
    let tag = Tag {
        id: Uuid::new_v4().to_string(),
        name,
        color,
    };
    connection
        .execute(
            "INSERT INTO tags (id, name, color) VALUES (?1, ?2, ?3)",
            params![tag.id, tag.name, tag.color],
        )
        .map_err(|error| error.to_string())?;
    Ok(tag)
}

pub fn answer_card(connection: &mut Connection, card_id: &str, rating: i32) -> Result<(), String> {
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    let (stability, difficulty, repetitions, state) = transaction
        .query_row(
            "SELECT COALESCE(stability, 0.0), COALESCE(difficulty, 0.0), reps, state FROM cards WHERE id = ?1",
            params![card_id],
            |row| Ok((row.get::<_, f64>(0)?, row.get::<_, f64>(1)?, row.get::<_, i32>(2)?, row.get::<_, i32>(3)?)),
        )
        .map_err(|error| error.to_string())?;
    let last_review = transaction
        .query_row(
            "SELECT COALESCE(MAX(review_datetime), 0) FROM review_logs WHERE card_id = ?1",
            params![card_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);
    let now = unix_timestamp()?;
    let schedule = crate::domain::fsrs::calculate_fsrs_step(
        rating,
        stability,
        difficulty,
        repetitions,
        state,
        last_review,
        now,
    )?;
    transaction
        .execute(
            "UPDATE cards SET stability = ?1, difficulty = ?2, reps = ?3, due_date = ?4, state = ?5 WHERE id = ?6",
            params![schedule.stability, schedule.difficulty, schedule.reps, schedule.due_date, schedule.state, card_id],
        )
        .map_err(|error| error.to_string())?;
    transaction
        .execute(
            "INSERT INTO review_logs (card_id, rating, review_datetime) VALUES (?1, ?2, ?3)",
            params![card_id, rating, now],
        )
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())
}

pub fn answer_card_in_conn(
    connection: &mut Connection,
    card_id: String,
    rating: i32,
) -> Result<(), String> {
    answer_card(connection, &card_id, rating)
}

pub fn study_heatmap(connection: &Connection) -> Result<Vec<HeatmapEntry>, String> {
    let mut statement = connection
        .prepare(
            "SELECT strftime('%Y-%m-%d', datetime(review_datetime, 'unixepoch', 'localtime')), COUNT(*)
             FROM review_logs GROUP BY 1 ORDER BY 1 ASC",
        )
        .map_err(|error| error.to_string())?;
    statement
        .query_map([], |row| {
            Ok(HeatmapEntry {
                date: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn study_streaks(connection: &Connection) -> Result<StreakStats, String> {
    let today = connection
        .query_row(
            "SELECT strftime('%Y-%m-%d', datetime('now', 'localtime'))",
            [],
            |row| row.get::<_, String>(0),
        )
        .map_err(|error| error.to_string())?;
    let mut statement = connection
        .prepare(
            "SELECT DISTINCT strftime('%Y-%m-%d', datetime(review_datetime, 'unixepoch', 'localtime'))
             FROM review_logs ORDER BY 1 DESC",
        )
        .map_err(|error| error.to_string())?;
    let dates = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(crate::domain::stats::calculate_streaks(&today, &dates))
}

pub fn migrate_sm2_to_fsrs(connection: &mut Connection) -> Result<usize, String> {
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    let legacy_cards = {
        let mut statement = transaction
            .prepare(
                "SELECT id, reps, due_date FROM cards WHERE stability = 0.0 OR difficulty = 0.0",
            )
            .map_err(|error| error.to_string())?;
        statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i32>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?
    };
    let now = unix_timestamp()?;
    for (id, repetitions, due_date) in &legacy_cards {
        let interval_days = if *repetitions <= 0 {
            0.0
        } else {
            ((*due_date - now) as f64 / 86_400.0)
                .max(1.0)
                .max(*repetitions as f64)
        };
        let (stability, difficulty) =
            crate::domain::fsrs::convert_sm2_to_fsrs(*repetitions, interval_days, 2.5);
        transaction
            .execute(
                "UPDATE cards SET stability = ?1, difficulty = ?2 WHERE id = ?3",
                params![stability, difficulty, id],
            )
            .map_err(|error| error.to_string())?;
    }
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(legacy_cards.len())
}

fn unix_timestamp() -> Result<i64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_update_schedules_card_in_memory() {
        let mut connection = Connection::open_in_memory().unwrap();
        crate::db::create_tables_and_seed(&connection).unwrap();
        add_to_srs(&connection, 1, None).unwrap();
        let card_id = connection
            .query_row("SELECT id FROM cards LIMIT 1", [], |row| {
                row.get::<_, String>(0)
            })
            .unwrap();

        answer_card(&mut connection, &card_id, 3).unwrap();

        let (due, reps, state): (i64, i32, i32) = connection
            .query_row(
                "SELECT due_date, reps, state FROM cards WHERE id = ?1",
                params![card_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert!(due > unix_timestamp().unwrap());
        assert_eq!(reps, 1);
        assert_eq!(state, 2);
    }
}
