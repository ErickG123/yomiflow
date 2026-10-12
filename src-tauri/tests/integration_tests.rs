use app_lib::commands::search::global_search_in_connection;
use app_lib::commands::srs::{
    add_to_srs_in_conn, answer_card_in_conn, create_deck_in_conn, delete_deck_in_conn,
};
use app_lib::database::migrations::{
    MIGRATIONS, latest_migration_version, run_migrations, run_migrations_with,
};
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
fn test_global_search_triggers_track_cards_notes_and_news() {
    let connection = Connection::open_in_memory().expect("open isolated search database");
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .expect("enable foreign keys");
    run_migrations(&connection).expect("apply search index migration");
    connection
        .execute(
            "INSERT INTO decks (id, name) VALUES ('default', 'Default')",
            [],
        )
        .expect("insert deck");
    connection
        .execute(
            "INSERT INTO dictionary_entries (id, kanji, reading, meaning) VALUES (1, '猫', 'ねこ', 'needle vocabulary')",
            [],
        )
        .expect("insert dictionary entry");
    connection
        .execute(
            "INSERT INTO cards (id, entry_id, deck_id, due_date, stability, difficulty, reps, state)
             VALUES ('card-1', 1, 'default', 0, 0, 0, 0, 0)",
            [],
        )
        .expect("insert vocabulary card");
    connection
        .execute(
            "INSERT INTO notebook_entries (id, title, content_md) VALUES ('note-1', 'Study note', 'note needle phrase')",
            [],
        )
        .expect("insert notebook entry");
    connection
        .execute(
            "INSERT INTO nhk_news_cache (id, title, body_text, body_html, cached_at)
             VALUES ('news-1', 'News headline', 'news needle phrase', '<p>news</p>', 1)",
            [],
        )
        .expect("insert cached news");

    let results = global_search_in_connection(&connection, "needle").expect("search indexed data");
    assert_eq!(results.len(), 3);
    assert!(results.iter().any(|result| result.id == "card:card-1"));
    assert!(results.iter().any(|result| result.id == "note:note-1"));
    assert!(results.iter().any(|result| result.id == "news:news-1"));
    assert!(
        results
            .iter()
            .all(|result| result.snippet.contains("<mark>needle</mark>"))
    );

    connection
        .execute(
            "UPDATE notebook_entries SET content_md = 'updated searchable phrase' WHERE id = 'note-1'",
            [],
        )
        .expect("update note and refresh index");
    assert!(
        global_search_in_connection(&connection, "needle")
            .unwrap()
            .iter()
            .all(|result| result.id != "note:note-1")
    );
    assert!(
        global_search_in_connection(&connection, "searchable")
            .unwrap()
            .iter()
            .any(|result| result.id == "note:note-1")
    );

    connection
        .execute(
            "UPDATE nhk_news_cache SET body_text = 'updated news phrase' WHERE id = 'news-1'",
            [],
        )
        .expect("update news and refresh index");
    assert!(
        global_search_in_connection(&connection, "needle")
            .unwrap()
            .iter()
            .all(|result| result.id != "news:news-1")
    );

    connection
        .execute(
            "UPDATE dictionary_entries SET meaning = 'updated card phrase' WHERE id = 1",
            [],
        )
        .expect("update dictionary data");
    connection
        .execute("UPDATE cards SET state = 1 WHERE id = 'card-1'", [])
        .expect("update card and refresh indexed dictionary fields");
    assert!(
        global_search_in_connection(&connection, "needle")
            .unwrap()
            .iter()
            .all(|result| result.id != "card:card-1")
    );
    assert!(
        global_search_in_connection(&connection, "updated")
            .unwrap()
            .iter()
            .any(|result| result.id == "card:card-1")
    );

    connection
        .execute("DELETE FROM cards WHERE id = 'card-1'", [])
        .expect("delete card and remove index row");
    assert!(
        global_search_in_connection(&connection, "updated")
            .unwrap()
            .iter()
            .all(|result| result.id != "card:card-1")
    );

    connection
        .execute("DELETE FROM nhk_news_cache WHERE id = 'news-1'", [])
        .expect("delete news and remove index row");
    assert!(
        global_search_in_connection(&connection, "news")
            .unwrap()
            .iter()
            .all(|result| result.id != "news:news-1")
    );
    connection
        .execute("DELETE FROM notebook_entries WHERE id = 'note-1'", [])
        .expect("delete note and remove index row");
    assert!(
        global_search_in_connection(&connection, "searchable")
            .unwrap()
            .iter()
            .all(|result| result.id != "note:note-1")
    );
}

#[test]
fn test_global_search_removes_deleted_notes() {
    let connection = Connection::open_in_memory().expect("open isolated search database");
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .unwrap();
    run_migrations(&connection).unwrap();
    connection.execute("INSERT INTO notebook_entries (id, title, content_md) VALUES ('note', 'Note', 'searchable note')", []).unwrap();
    assert!(
        global_search_in_connection(&connection, "searchable")
            .unwrap()
            .iter()
            .any(|item| item.id == "note:note")
    );
    connection
        .execute("DELETE FROM notebook_entries WHERE id = 'note'", [])
        .unwrap();
    assert!(
        global_search_in_connection(&connection, "searchable")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn test_omnisearch_migration_indexes_existing_records() {
    let connection = Connection::open_in_memory().expect("open pre-OmniSearch database");
    run_migrations_with(&connection, &MIGRATIONS[..2]).expect("apply pre-search schema");
    connection
        .execute(
            "INSERT INTO decks (id, name) VALUES ('default', 'Default')",
            [],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO dictionary_entries (kanji, reading, meaning) VALUES ('犬', 'いぬ', 'backfillhound')",
            [],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO cards (id, entry_id, deck_id, due_date, stability, difficulty, reps, state)
             VALUES ('old-card', 1, 'default', 0, 0, 0, 0, 0)",
            [],
        )
        .unwrap();

    run_migrations_with(&connection, &MIGRATIONS[2..]).expect("apply OmniSearch index backfill");

    let results = global_search_in_connection(&connection, "backfillhound").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "card:old-card");
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
