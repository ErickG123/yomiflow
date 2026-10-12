use app_lib::commands::srs::{
    add_to_srs_in_conn, answer_card_in_conn, create_deck_in_conn, delete_deck_in_conn,
};
use app_lib::database::migrations::{latest_migration_version, run_migrations};
use app_lib::{db, domain::nlp::generate_syntax_tree};
use rusqlite::{Connection, params};
use std::time::{SystemTime, UNIX_EPOCH};

fn setup_test_db() -> Connection {
    let connection = Connection::open_in_memory().expect("open isolated in-memory database");
    db::create_tables_and_seed(&connection).expect("create test schema and run seeds");
    connection
}

#[test]
fn test_schema_migrations_apply_from_zero_to_latest() {
    let connection = Connection::open_in_memory().expect("open empty in-memory database");

    run_migrations(&connection).expect("apply every schema migration");

    let current_version: u32 = connection
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .expect("read final migration version");
    assert_eq!(current_version, latest_migration_version());

    for object in [
        ("table", "decks"),
        ("table", "dictionary_entries"),
        ("table", "cards"),
        ("table", "study_sessions"),
        ("table", "image_occlusion_cards"),
        ("view", "flashcards"),
    ] {
        let exists: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = ?1 AND name = ?2",
                [object.0, object.1],
                |row| row.get(0),
            )
            .expect("query migrated schema object");
        assert_eq!(
            exists, 1,
            "{} {} should exist after migration",
            object.0, object.1
        );
    }

    run_migrations(&connection).expect("re-running migrations is idempotent");
    let version_after_second_run: u32 = connection
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .expect("read migration version after second run");
    assert_eq!(version_after_second_run, current_version);
}

#[test]
fn test_cascade_delete_deck_removes_cards() {
    let connection = setup_test_db();
    let deck = create_deck_in_conn(&connection, "Integration deck".into(), None)
        .expect("create deck using SRS service");

    for index in 0..3 {
        connection
            .execute(
                "INSERT INTO dictionary_entries (kanji, reading, meaning) VALUES (?1, ?2, ?3)",
                params![format!("語{index}"), format!("ご{index}"), "word"],
            )
            .expect("create dictionary entry");
        let entry_id = connection.last_insert_rowid();
        add_to_srs_in_conn(&connection, entry_id, Some(deck.id.clone()))
            .expect("add card using SRS service");
    }

    let cards_before: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM cards WHERE deck_id = ?1",
            params![deck.id],
            |row| row.get(0),
        )
        .expect("count cards before deletion");
    assert_eq!(cards_before, 3);

    delete_deck_in_conn(&connection, deck.id.clone()).expect("delete deck using SRS service");

    let cards_after: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM cards WHERE deck_id = ?1",
            params![deck.id],
            |row| row.get(0),
        )
        .expect("count cards after deletion");
    assert_eq!(
        cards_after, 0,
        "foreign-key cascade should remove all cards"
    );
}

#[test]
fn test_fsrs_review_updates_next_due_date() {
    let mut connection = setup_test_db();
    connection
        .execute(
            "INSERT INTO dictionary_entries (kanji, reading, meaning) VALUES (?1, ?2, ?3)",
            params!["学生", "がくせい", "student"],
        )
        .expect("create dictionary entry");
    let entry_id = connection.last_insert_rowid();

    add_to_srs_in_conn(&connection, entry_id, None).expect("create new SRS card");
    let card_id: String = connection
        .query_row(
            "SELECT id FROM cards WHERE entry_id = ?1",
            params![entry_id],
            |row| row.get(0),
        )
        .expect("load created card");

    let reviewed_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after unix epoch")
        .as_secs() as i64;
    answer_card_in_conn(&mut connection, card_id.clone(), 3)
        .expect("answer card with Good rating via SRS service");

    let (next_due, reps, review_count): (i64, i32, i64) = connection
        .query_row(
            "SELECT c.due_date, c.reps,
                    (SELECT COUNT(*) FROM review_logs WHERE card_id = c.id)
             FROM cards c WHERE c.id = ?1",
            params![card_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("read reviewed card state");

    assert!(
        next_due > reviewed_at,
        "Good should schedule the card in the future"
    );
    assert_eq!(reps, 1, "the review repetition count should increase");
    assert_eq!(review_count, 1, "the review should be persisted");
}

#[test]
fn test_lindera_bunsetsu_pipeline() {
    let chunks = generate_syntax_tree("私は学生です").expect("segment Japanese sentence");

    assert_eq!(
        chunks
            .iter()
            .map(|chunk| chunk.text.as_str())
            .collect::<Vec<_>>(),
        vec!["私は", "学生", "です"]
    );
    assert!(chunks.iter().all(|chunk| !chunk.pos.is_empty()));

    let serialized = serde_json::to_string(&chunks).expect("serialize bunsetsu chunks");
    let decoded: serde_json::Value =
        serde_json::from_str(&serialized).expect("deserialize bunsetsu JSON");
    assert_eq!(decoded.as_array().map(Vec::len), Some(chunks.len()));
}
