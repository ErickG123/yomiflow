use crate::AppState;
use crate::models::SearchResult;
use rusqlite::{Connection, params};
use tauri::State;

#[tauri::command]
pub fn global_search(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    global_search_in_connection(&connection, &query)
}

pub fn global_search_in_connection(
    connection: &Connection,
    query: &str,
) -> Result<Vec<SearchResult>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    // Quoting the input treats it as a phrase and prevents malformed FTS query operators.
    let match_query = format!("\"{}\"", query.replace('"', "\"\""));
    let mut statement = connection
        .prepare(
            "SELECT id, source_type, title,
                    snippet(global_search, 3, '<mark>', '</mark>', '…', 18)
             FROM global_search
             WHERE global_search MATCH ?1
             ORDER BY rank
             LIMIT 50",
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
