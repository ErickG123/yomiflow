use rusqlite::{Connection, Transaction};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug)]
pub struct Migration {
    pub version: u32,
    pub sql: &'static str,
}

const LEGACY_COLUMN_MIGRATION: &str = r#"
ALTER TABLE kanjis ADD COLUMN jlpt_level INTEGER;
ALTER TABLE dictionary_entries ADD COLUMN pitch TEXT;
ALTER TABLE cards ADD COLUMN deck_id TEXT;
ALTER TABLE cards ADD COLUMN stability REAL DEFAULT 0.0;
ALTER TABLE cards ADD COLUMN difficulty REAL DEFAULT 0.0;
ALTER TABLE cards ADD COLUMN reps INTEGER DEFAULT 0;
ALTER TABLE tags ADD COLUMN name TEXT;
ALTER TABLE tags ADD COLUMN color TEXT;
ALTER TABLE notebook_entries ADD COLUMN deck_id TEXT;
ALTER TABLE notebook_entries ADD COLUMN next_review_date DATETIME;
ALTER TABLE notebook_entries ADD COLUMN srs_stage INTEGER DEFAULT 0;
ALTER TABLE kana_mastery ADD COLUMN alphabet TEXT;
ALTER TABLE image_occlusion_cards ADD COLUMN due_date INTEGER NOT NULL DEFAULT 0;
ALTER TABLE image_occlusion_cards ADD COLUMN stability REAL NOT NULL DEFAULT 0.0;
ALTER TABLE image_occlusion_cards ADD COLUMN difficulty REAL NOT NULL DEFAULT 0.0;
ALTER TABLE image_occlusion_cards ADD COLUMN reps INTEGER NOT NULL DEFAULT 0;
ALTER TABLE image_occlusion_cards ADD COLUMN state INTEGER NOT NULL DEFAULT 0;
ALTER TABLE image_occlusion_cards ADD COLUMN last_review INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kana_mastery ADD COLUMN category TEXT;
ALTER TABLE kana_mastery ADD COLUMN correct_count INTEGER DEFAULT 0;
ALTER TABLE kana_mastery ADD COLUMN wrong_count INTEGER DEFAULT 0;
CREATE VIEW IF NOT EXISTS flashcards AS
SELECT id, entry_id, deck_id, due_date, stability, difficulty, reps, state FROM cards;
"#;

pub static MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        sql: include_str!("schema.sql"),
    },
    Migration {
        version: 2,
        sql: LEGACY_COLUMN_MIGRATION,
    },
];

#[derive(Debug)]
struct MigrationExecutionError {
    version: u32,
    source: rusqlite::Error,
}

impl Display for MigrationExecutionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "SQLite schema migration {} failed and was rolled back: {}",
            self.version, self.source
        )
    }
}

impl Error for MigrationExecutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

pub fn run_migrations(connection: &Connection) -> Result<(), Box<dyn Error>> {
    run_migrations_with(connection, MIGRATIONS)
}

pub fn run_migrations_with(
    connection: &Connection,
    migrations: &[Migration],
) -> Result<(), Box<dyn Error>> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY);",
    )?;

    let mut current_version: u32 = connection.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;

    for migration in migrations {
        if migration.version <= current_version {
            continue;
        }
        let transaction = connection.unchecked_transaction()?;
        let result = apply_migration(&transaction, migration).and_then(|()| {
            transaction
                .execute(
                    "INSERT INTO schema_migrations (version) VALUES (?1)",
                    [migration.version],
                )
                .map(|_| ())
        });

        if let Err(source) = result {
            transaction.rollback()?;
            return Err(Box::new(MigrationExecutionError {
                version: migration.version,
                source,
            }));
        }

        transaction.commit()?;
        current_version = migration.version;
    }

    Ok(())
}

pub fn latest_migration_version() -> u32 {
    MIGRATIONS
        .iter()
        .map(|migration| migration.version)
        .max()
        .unwrap_or(0)
}

fn apply_migration(transaction: &Transaction<'_>, migration: &Migration) -> rusqlite::Result<()> {
    for statement in migration
        .sql
        .split(';')
        .map(str::trim)
        .filter(|sql| !sql.is_empty())
    {
        if let Some((table, column)) = added_column(statement)
            && column_exists(transaction, table, column)?
        {
            continue;
        }
        transaction.execute_batch(statement)?;
    }
    Ok(())
}

fn added_column(statement: &str) -> Option<(&str, &str)> {
    let parts = statement.split_whitespace().collect::<Vec<_>>();
    if parts.len() >= 6
        && parts[0].eq_ignore_ascii_case("ALTER")
        && parts[1].eq_ignore_ascii_case("TABLE")
        && parts[3].eq_ignore_ascii_case("ADD")
        && parts[4].eq_ignore_ascii_case("COLUMN")
    {
        Some((parts[2], parts[5]))
    } else {
        None
    }
}

fn column_exists(
    transaction: &Transaction<'_>,
    table: &str,
    column: &str,
) -> rusqlite::Result<bool> {
    // Table/column names are sourced only from the checked-in migration definitions.
    let mut statement = transaction.prepare(&format!("PRAGMA table_info({table})"))?;
    let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
    for existing_column in columns {
        if existing_column? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_migration_rolls_back_schema_and_version() {
        let connection = Connection::open_in_memory().unwrap();
        let broken_migration = [Migration {
            version: 1,
            sql: "CREATE TABLE rollback_probe (id INTEGER); INSERT INTO missing_table VALUES (1);",
        }];

        let error = run_migrations_with(&connection, &broken_migration).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("migration 1 failed and was rolled back")
        );

        let table_exists: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'rollback_probe'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let recorded_version: u32 = connection
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 0);
        assert_eq!(recorded_version, 0);
    }
}
