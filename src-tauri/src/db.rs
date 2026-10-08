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

    conn.execute_batch(
        "
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
        ",
    )?;

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

    Ok(conn)
}
