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
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS dictionary_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kanji TEXT,
            reading TEXT NOT NULL,
            meaning TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_dictionary_kanji ON dictionary_entries(kanji);
        CREATE INDEX IF NOT EXISTS idx_dictionary_reading ON dictionary_entries(reading);

        CREATE TABLE IF NOT EXISTS decks (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT
        );

        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT,
            color TEXT,
            category TEXT,
            notes TEXT
        );

        CREATE TABLE IF NOT EXISTS cards (
            id TEXT PRIMARY KEY,
            entry_id INTEGER NOT NULL,
            deck_id TEXT,
            due_date INTEGER NOT NULL,
            stability REAL NOT NULL,
            difficulty REAL NOT NULL,
            reps INTEGER NOT NULL,
            state INTEGER NOT NULL,
            FOREIGN KEY(entry_id) REFERENCES dictionary_entries(id) ON DELETE CASCADE,
            FOREIGN KEY(deck_id) REFERENCES decks(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS card_tags (
            card_id TEXT NOT NULL,
            tag_id TEXT NOT NULL,
            PRIMARY KEY (card_id, tag_id),
            FOREIGN KEY(card_id) REFERENCES cards(id) ON DELETE CASCADE,
            FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS review_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            card_id TEXT NOT NULL,
            rating INTEGER NOT NULL,
            review_datetime INTEGER NOT NULL,
            FOREIGN KEY(card_id) REFERENCES cards(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT
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

        CREATE TABLE IF NOT EXISTS notebook_entries (
            id TEXT PRIMARY KEY,
            title TEXT,
            content_md TEXT,
            created_at DATETIME,
            updated_at DATETIME,
            deck_id TEXT,
            next_review_date DATETIME,
            srs_stage INTEGER DEFAULT 0,
            FOREIGN KEY(deck_id) REFERENCES decks(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS kana_mastery (
            character TEXT PRIMARY KEY,
            romaji TEXT NOT NULL,
            kana_type TEXT NOT NULL,
            alphabet TEXT,
            category TEXT,
            correct_count INTEGER DEFAULT 0,
            wrong_count INTEGER DEFAULT 0,
            total_attempts INTEGER DEFAULT 0,
            correct_attempts INTEGER DEFAULT 0,
            accuracy_percentage REAL DEFAULT 0.0,
            last_practiced INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS nhk_news_cache (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            title_with_ruby TEXT,
            body_text TEXT NOT NULL,
            body_html TEXT NOT NULL,
            image_url TEXT,
            audio_url TEXT,
            published_at TEXT,
            web_url TEXT,
            cached_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS epub_reading_positions (
            book_hash TEXT PRIMARY KEY,
            cfi TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS badges (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            icon TEXT NOT NULL,
            category TEXT NOT NULL,
            target_metric TEXT NOT NULL,
            target_value INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS user_badges (
            badge_id TEXT PRIMARY KEY,
            unlocked_at INTEGER NOT NULL,
            FOREIGN KEY(badge_id) REFERENCES badges(id) ON DELETE CASCADE
        );

        CREATE VIEW IF NOT EXISTS flashcards AS 
        SELECT id, entry_id, deck_id, due_date, stability, difficulty, reps, state FROM cards;
        ",
    )?;

    // Migrations for existing databases
    let _ = conn.execute("ALTER TABLE kanjis ADD COLUMN jlpt_level INTEGER", []);
    let _ = conn.execute("ALTER TABLE dictionary_entries ADD COLUMN pitch TEXT", []);
    let _ = conn.execute("ALTER TABLE cards ADD COLUMN deck_id TEXT", []);
    let _ = conn.execute(
        "ALTER TABLE cards ADD COLUMN stability REAL DEFAULT 0.0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE cards ADD COLUMN difficulty REAL DEFAULT 0.0",
        [],
    );
    let _ = conn.execute("ALTER TABLE cards ADD COLUMN reps INTEGER DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE tags ADD COLUMN name TEXT", []);
    let _ = conn.execute("ALTER TABLE tags ADD COLUMN color TEXT", []);
    let _ = conn.execute("ALTER TABLE notebook_entries ADD COLUMN deck_id TEXT", []);
    let _ = conn.execute(
        "ALTER TABLE notebook_entries ADD COLUMN next_review_date DATETIME",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE notebook_entries ADD COLUMN srs_stage INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute("ALTER TABLE kana_mastery ADD COLUMN alphabet TEXT", []);
    let _ = conn.execute("ALTER TABLE kana_mastery ADD COLUMN category TEXT", []);
    let _ = conn.execute(
        "ALTER TABLE kana_mastery ADD COLUMN correct_count INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE kana_mastery ADD COLUMN wrong_count INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "CREATE VIEW IF NOT EXISTS flashcards AS SELECT id, entry_id, deck_id, due_date, stability, difficulty, reps, state FROM cards",
        [],
    );

    // Seed kana table with all characters (Base, Dakuon, Yoon)
    let _ = crate::db::kana_seed::seed_kana_mastery(conn);

    // Seed default deck if none exists
    let deck_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM decks", [], |row| row.get(0))
        .unwrap_or(0);
    if deck_count == 0 {
        let _ = conn.execute(
            "INSERT INTO decks (id, name, description) VALUES (?1, ?2, ?3)",
            params![
                "default",
                "Padrão",
                "Baralho principal de vocabulário e estudos"
            ],
        );
    }

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

    // Seed achievement badges
    let badge_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM badges", [], |row| row.get(0))
        .unwrap_or(0);

    if badge_count == 0 {
        let badges_data = [
            (
                "first_card",
                "Primeiro Passo",
                "Adicione ou revise o seu primeiro cartão",
                "pi-flag",
                "srs",
                "cards_learned",
                1,
            ),
            (
                "kanji_10",
                "Aprendiz de Kanji",
                "Aprenda os seus primeiros 10 kanjis",
                "pi-book",
                "kanji",
                "kanji_count",
                10,
            ),
            (
                "kanji_50",
                "Explorador de Ideogramas",
                "Aprenda 50 kanjis no SRS",
                "pi-compass",
                "kanji",
                "kanji_count",
                50,
            ),
            (
                "kanji_100",
                "Mestre dos 100 Kanjis",
                "Domine 100 kanjis no seu vocabulário",
                "pi-star",
                "kanji",
                "kanji_count",
                100,
            ),
            (
                "streak_3",
                "Foco Inicial",
                "Mantenha uma ofensiva de 3 dias de estudo",
                "pi-bolt",
                "streak",
                "streak_days",
                3,
            ),
            (
                "streak_7",
                "Semana Perfeita",
                "Mantenha uma ofensiva de 7 dias consecutivos",
                "pi-calendar",
                "streak",
                "streak_days",
                7,
            ),
            (
                "streak_30",
                "Guerreiro da Disciplina",
                "Atingiu uma ofensiva ininterrupta de 30 dias",
                "pi-trophy",
                "streak",
                "streak_days",
                30,
            ),
            (
                "kana_master",
                "Sensei dos Kana",
                "Complete o treino e alcance 100% no Kana Dojo",
                "pi-check-circle",
                "kana",
                "kana_proficiency",
                100,
            ),
        ];

        for (id, name, desc, icon, category, metric, val) in badges_data {
            let _ = conn.execute(
                "INSERT INTO badges (id, name, description, icon, category, target_metric, target_value)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![id, name, desc, icon, category, metric, val],
            );
        }
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
            .query_row(
                "SELECT reps FROM cards WHERE id = ?1",
                params!["test-card-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(card_reps, 1);

        // Insert and query kanji_strokes table
        conn.execute(
            "INSERT INTO kanji_strokes (kanji, svg_data) VALUES (?1, ?2)",
            params!["日", "<svg><path d=\"M...\"/></svg>"],
        )
        .unwrap();

        let svg: String = conn
            .query_row(
                "SELECT svg_data FROM kanji_strokes WHERE kanji = ?1",
                params!["日"],
                |row| row.get(0),
            )
            .unwrap();
        assert!(svg.contains("<svg>"));
    }

    #[test]
    fn test_foreign_keys_and_cascade_deletion() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();
        create_tables_and_seed(&conn).unwrap();

        // Check default deck exists
        let default_deck_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM decks WHERE id = 'default'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(default_deck_exists, 1);

        // Create a custom deck
        conn.execute(
            "INSERT INTO decks (id, name, description) VALUES (?1, ?2, ?3)",
            params!["deck-jlpt-n3", "JLPT N3", "Vocabulário intermediário"],
        )
        .unwrap();

        // Create a tag
        conn.execute(
            "INSERT INTO tags (id, name, color) VALUES (?1, ?2, ?3)",
            params!["tag-verbs", "Verbos", "#3b82f6"],
        )
        .unwrap();

        // Insert a card in custom deck
        conn.execute(
            "INSERT INTO cards (id, entry_id, deck_id, due_date, stability, difficulty, reps, state) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params!["card-custom-1", 1, "deck-jlpt-n3", 1700000000, 2.5, 0.3, 0, 0],
        ).unwrap();

        // Link card to tag
        conn.execute(
            "INSERT INTO card_tags (card_id, tag_id) VALUES (?1, ?2)",
            params!["card-custom-1", "tag-verbs"],
        )
        .unwrap();

        // Assert card_tag exists
        let ct_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM card_tags WHERE card_id = 'card-custom-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(ct_count, 1);

        // Delete the card and verify CASCADE deletes from card_tags
        conn.execute("DELETE FROM cards WHERE id = 'card-custom-1'", [])
            .unwrap();
        let ct_count_after: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM card_tags WHERE card_id = 'card-custom-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(ct_count_after, 0);

        // Deleting the tag cascades to any remaining references
        conn.execute("DELETE FROM tags WHERE id = 'tag-verbs'", [])
            .unwrap();
        let tag_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tags WHERE id = 'tag-verbs'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tag_count, 0);

        // Verify Deleting a Deck cascades and deletes all its cards automatically
        conn.execute(
            "INSERT INTO cards (id, entry_id, deck_id, due_date, stability, difficulty, reps, state) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params!["card-custom-2", 1, "deck-jlpt-n3", 1700000000, 2.5, 0.3, 0, 0],
        ).unwrap();
        let card_before: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM cards WHERE deck_id = 'deck-jlpt-n3'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(card_before, 1);

        conn.execute("DELETE FROM decks WHERE id = 'deck-jlpt-n3'", [])
            .unwrap();
        let card_after: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM cards WHERE deck_id = 'deck-jlpt-n3'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(card_after, 0);

        // Verify app_settings table exists and persists onboarding status
        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('onboarding_completed', 'true')",
            [],
        )
        .unwrap();
        let onboarding_val: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'onboarding_completed'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(onboarding_val, "true");

        // Verify foreign key constraint fails when card has an invalid/non-existent deck_id
        let invalid_insert_res = conn.execute(
            "INSERT INTO cards (id, entry_id, deck_id, due_date, stability, difficulty, reps, state) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params!["card-invalid-deck", 1, "nonexistent-deck-id", 1700000000, 2.5, 0.3, 0, 0],
        );
        assert!(invalid_insert_res.is_err(), "Card with non-existent deck_id must violate foreign key constraint");
    }
}
