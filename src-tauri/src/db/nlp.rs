use crate::domain::readability::{RetentionIndex, RetentionRecord};
use crate::models::{LexicalToken, NotebookEntry};
use rusqlite::{Connection, params};
use std::collections::HashSet;

pub fn load_retention_index(
    connection: &Connection,
    tokens: &[LexicalToken],
) -> Result<RetentionIndex, String> {
    let mut index = RetentionIndex::new();
    let mut statement = connection
        .prepare(
            "SELECT c.state, c.stability, c.reps, COALESCE(MAX(log.review_datetime), 0)
             FROM dictionary_entries d
             JOIN cards c ON c.entry_id = d.id
             LEFT JOIN review_logs log ON log.card_id = c.id
             WHERE d.kanji = ?1 COLLATE NOCASE OR d.reading = ?2 COLLATE NOCASE
             GROUP BY c.id",
        )
        .map_err(|error| error.to_string())?;

    for token in tokens {
        let key = (token.lemma.clone(), token.reading.clone());
        if index.contains_key(&key) {
            continue;
        }
        let records = statement
            .query_map(params![token.lemma, token.reading], |row| {
                Ok(RetentionRecord {
                    state: row.get(0)?,
                    stability: row.get(1)?,
                    repetitions: row.get(2)?,
                    last_review: row.get(3)?,
                })
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        index.insert(key, records);
    }
    Ok(index)
}

pub fn mastered_kanji(connection: &Connection) -> Result<HashSet<String>, String> {
    let mut statement = connection
        .prepare(
            "SELECT DISTINCT d.kanji
             FROM cards c
             JOIN dictionary_entries d ON c.entry_id = d.id
             WHERE d.kanji IS NOT NULL
               AND (c.stability >= 10.0 OR (c.state = 2 AND c.reps >= 3))",
        )
        .map_err(|error| error.to_string())?;
    statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<HashSet<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn save_notebook_entry(connection: &Connection, entry: &NotebookEntry) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO notebook_entries (id, title, content_md, created_at, updated_at, deck_id, next_review_date, srs_stage)
             VALUES (?1, ?2, ?3, ?4, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                title = ?2,
                content_md = ?3,
                updated_at = ?4,
                deck_id = COALESCE(?5, notebook_entries.deck_id),
                next_review_date = COALESCE(?6, notebook_entries.next_review_date),
                srs_stage = COALESCE(?7, notebook_entries.srs_stage)",
            params![
                entry.id,
                entry.title,
                entry.content_md,
                entry.created_at,
                entry.deck_id,
                entry.next_review_date,
                entry.srs_stage.unwrap_or(0)
            ],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn link_notebook_entry(
    connection: &Connection,
    id: &str,
    deck_id: Option<&str>,
    updated_at: i64,
) -> Result<(), String> {
    connection
        .execute(
            "UPDATE notebook_entries SET deck_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![deck_id, updated_at.to_string(), id],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn notebook_entries(connection: &Connection) -> Result<Vec<NotebookEntry>, String> {
    let mut statement = connection
        .prepare("SELECT id, title, content_md, CAST(created_at AS TEXT), CAST(updated_at AS TEXT), deck_id, next_review_date, srs_stage FROM notebook_entries ORDER BY updated_at DESC")
        .map_err(|error| error.to_string())?;
    statement
        .query_map([], |row| {
            Ok(NotebookEntry {
                id: row.get(0)?,
                title: row.get(1)?,
                content_md: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                deck_id: row.get(5)?,
                next_review_date: row.get(6)?,
                srs_stage: row.get(7)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn delete_notebook_entry(connection: &Connection, id: &str) -> Result<(), String> {
    connection
        .execute("DELETE FROM notebook_entries WHERE id = ?1", [id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notebook_timestamps_are_serialized_as_text_when_sqlite_stores_integers() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE notebook_entries (
                    id TEXT PRIMARY KEY, title TEXT, content_md TEXT,
                    created_at DATETIME, updated_at DATETIME, deck_id TEXT,
                    next_review_date INTEGER, srs_stage INTEGER
                );
                INSERT INTO notebook_entries VALUES ('n1', 'Title', 'Body', 123, 456, NULL, NULL, 0);",
            )
            .unwrap();

        let entries = notebook_entries(&connection).unwrap();

        assert_eq!(entries[0].created_at, "123");
        assert_eq!(entries[0].updated_at, "456");
    }
}
