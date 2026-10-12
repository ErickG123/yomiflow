# SQLite Database Migrations

## Runtime flow

The migration engine is implemented in `src-tauri/src/database/migrations.rs`, exported by `database::migrations`, and called by `db::create_tables_and_seed` during Tauri startup. Database pragmas are configured before migrations. If a migration fails, the engine rolls back its transaction and returns an error; initialization propagates that error and the app does not continue.

## Version tracking and transactions

The engine creates this control table if needed:

```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY
);
```

It reads `COALESCE(MAX(version), 0)` and applies each migration whose version is greater than the current version. Every migration runs inside its own SQLite transaction. The version row is inserted only after that migration's SQL succeeds, and both schema changes and the version insert commit together. A failure rolls back the migration and returns an error identifying its version.

`MIGRATIONS` is the ordered source of truth for available schema versions. Version 1 loads the current base schema from `schema.sql`; version 2 applies compatibility columns previously added opportunistically during startup and creates the `flashcards` view. Before applying an `ALTER TABLE ... ADD COLUMN`, the engine checks `PRAGMA table_info` so databases that already contain a column can advance safely. New schema changes must be added as a new migration version; do not edit an already shipped migration because existing databases have recorded its version.

## Testing

Migration tests must use `Connection::open_in_memory()` and must never open the user's `yomiflow.db`. The integration case `test_schema_migrations_apply_from_zero_to_latest` verifies the final version, representative schema objects, and idempotent reruns. The migration module unit test verifies that a failing migration rolls back both partial DDL and its version record.

```bash
cd src-tauri
cargo test --test integration_tests test_schema_migrations_apply_from_zero_to_latest
cargo test database::migrations::tests::failed_migration_rolls_back_schema_and_version
```
