use rusqlite::{Connection, Result, params};
use std::fs;
use tauri::Manager;

pub fn initialize_database(
    app_handle: &tauri::AppHandle,
) -> Result<Connection, Box<dyn std::error::Error>> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;

    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir)?;
    }

    let db_path = app_data_dir.join("yomiflow.db");
    println!("Caminho do Banco de Dados: {:?}", db_path);
    let conn = Connection::open(&db_path)?;

    create_tables_and_seed(&conn)?;

    Ok(conn)
}

pub fn create_tables_and_seed(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA temp_store = MEMORY;
        PRAGMA mmap_size = 3000000000;

        CREATE TABLE IF NOT EXISTS dictionary_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kanji TEXT,
            reading TEXT NOT NULL,
            meaning TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_dictionary_kanji ON dictionary_entries(kanji);
        CREATE INDEX IF NOT EXISTS idx_dictionary_reading ON dictionary_entries(reading);

        CREATE TABLE IF NOT EXISTS cards (
            id TEXT PRIMARY KEY,
            entry_id INTEGER NOT NULL,
            due_date INTEGER NOT NULL,
            stability REAL NOT NULL,
            difficulty REAL NOT NULL,
            reps INTEGER NOT NULL,
            state INTEGER NOT NULL,
            FOREIGN KEY(entry_id) REFERENCES dictionary_entries(id)
        );

        CREATE TABLE IF NOT EXISTS review_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            card_id TEXT NOT NULL,
            rating INTEGER NOT NULL,
            review_datetime INTEGER NOT NULL,
            FOREIGN KEY(card_id) REFERENCES cards(id)
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        );

        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            category TEXT,
            notes TEXT
        );

        CREATE TABLE IF NOT EXISTS kanjis (
            character TEXT PRIMARY KEY,
            onyomi TEXT,
            kunyomi TEXT,
            meaning TEXT,
            jlpt_level INTEGER
        );

        CREATE TABLE IF NOT EXISTS kanji_strokes (
            kanji TEXT PRIMARY KEY,
            svg_data TEXT
        );

        CREATE TABLE IF NOT EXISTS word_frequencies (
            term TEXT PRIMARY KEY,
            rank INTEGER
        );

        CREATE TABLE IF NOT EXISTS kanji_components (
            kanji TEXT NOT NULL,
            component TEXT NOT NULL,
            meaning TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_kanji_components_kanji ON kanji_components(kanji);

        CREATE TABLE IF NOT EXISTS user_mnemonics (
            kanji TEXT PRIMARY KEY,
            mnemonic TEXT NOT NULL
        );
        ",
    )?;

    // Migration: ensure jlpt_level column exists if table was previously created without it
    let _ = conn.execute("ALTER TABLE kanjis ADD COLUMN jlpt_level INTEGER", []);
    // Migration: ensure pitch column exists if dictionary_entries was created without it
    let _ = conn.execute("ALTER TABLE dictionary_entries ADD COLUMN pitch TEXT", []);

    // Seed mock data if empty
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM dictionary_entries", [], |row| {
        row.get(0)
    })?;

    if count == 0 {
        conn.execute(
            "INSERT INTO dictionary_entries (kanji, reading, meaning) VALUES (?1, ?2, ?3)",
            params!["食べる", "たべる", "to eat"],
        )?;
        conn.execute(
            "INSERT INTO dictionary_entries (kanji, reading, meaning) VALUES (?1, ?2, ?3)",
            params!["水", "みず", "water"],
        )?;
        conn.execute(
            "INSERT INTO dictionary_entries (kanji, reading, meaning) VALUES (?1, ?2, ?3)",
            params!["猫", "ねこ", "cat"],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_in_memory_initialization_and_queries() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables_and_seed(&conn).unwrap();

        // Check seed count
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM dictionary_entries", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 3);

        // Check dictionary lookup query
        let reading: String = conn
            .query_row(
                "SELECT reading FROM dictionary_entries WHERE kanji = ?1",
                params!["食べる"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(reading, "たべる");

        // Insert and query cards table
        conn.execute(
            "INSERT INTO cards (id, entry_id, due_date, stability, difficulty, reps, state) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params!["test-card-1", 1, 1700000000, 2.5, 0.3, 1, 2],
        ).unwrap();

        let card_reps: i32 = conn
            .query_row("SELECT reps FROM cards WHERE id = ?1", params!["test-card-1"], |row| row.get(0))
            .unwrap();
        assert_eq!(card_reps, 1);

        // Insert and query kanji_strokes table
        conn.execute(
            "INSERT INTO kanji_strokes (kanji, svg_data) VALUES (?1, ?2)",
            params!["日", "<svg><path d=\"M...\"/></svg>"],
        ).unwrap();

        let svg: String = conn
            .query_row(
                "SELECT svg_data FROM kanji_strokes WHERE kanji = ?1",
                params!["日"],
                |row| row.get(0),
            )
            .unwrap();
        assert!(svg.contains("<svg>"));
    }
}
