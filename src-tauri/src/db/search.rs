use crate::models::SearchResult;
use rusqlite::{Connection, params};

/// SQLite FTS5 adapter for global search queries.
pub fn global_search_in_connection(
    connection: &Connection,
    query: &str,
) -> Result<Vec<SearchResult>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let match_query = format!("\"{}\"", query.replace('"', "\"\""));
    let mut statement = connection
        .prepare(
            "SELECT id, source_type, title,
                    snippet(global_search, 3, '<mark>', '</mark>', '…', 18)
             FROM global_search WHERE global_search MATCH ?1 ORDER BY rank LIMIT 50",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![match_query], |row| {
            Ok(SearchResult {
                id: row.get(0)?,
                source_type: row.get(1)?,
                title: row.get(2)?,
                snippet: row.get(3)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}
