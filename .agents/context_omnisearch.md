# Global FTS5 OmniSearch

## Data flow

The global search index is created by migration version 3 in `src-tauri/src/database/migrations.rs`:

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS global_search
USING fts5(id, source_type, title, content, snippet);
```

The FTS5 table stores a prefixed source id (`card:`, `note:`, or `news:`), a stable source type, a title, searchable content, and an empty compatibility `snippet` field. Prefixes prevent source ids from colliding across tables.

## SQLite triggers

Three trigger sets keep the search index synchronized within the same SQLite write transaction as the source change:

- `cards` triggers insert, replace, or delete `card:<id>` records. Searchable card content is assembled from its related dictionary entry's kanji, reading, and meaning.
- `notebook_entries` triggers index the note title and `content_md` as `note:<id>`.
- `nhk_news_cache` triggers index the article title and body text as `news:<id>`.

Update triggers remove the previous FTS row and insert its current representation. Delete triggers remove the matching prefixed row. Thus failed source writes cannot leave the index out of sync. Migration statement parsing treats `CREATE TRIGGER ... BEGIN ... END` as one statement, preserving its internal semicolons and transactional migration behavior.

## IPC and UI

`commands::search::global_search` receives a query, treats it as a quoted FTS phrase, binds it as a SQL parameter to `MATCH`, orders matches by FTS rank, and caps the response at 50 results. FTS5's `snippet()` marks matching terms in the content excerpt. `SearchResult` serializes as camelCase for Tauri IPC.

The Command Palette applies a short debounce before calling Angular's `GlobalSearchService`; the service ignores stale responses. The palette groups results by source. It renders each excerpt as escaped Angular text and `<mark>` elements, rather than injecting HTML from database content. Selecting a note opens `/notebook?noteId=...`; selecting a vocabulary card opens the Dicionário with its title as the search query; a news result opens the Web reader.

## Validation

`test_global_search_triggers_track_cards_notes_and_news` uses a fresh in-memory SQLite database. It verifies indexing on insert, refresh on updates, row removal on deletes, FTS snippets, and command-level result mapping. Run it with:

```bash
cd src-tauri
cargo test --test integration_tests test_global_search_triggers_track_cards_notes_and_news
```
