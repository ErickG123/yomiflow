use crate::AppState;
use crate::models::SearchResult;
use tauri::State;

/// IPC adapter: validates access to app state and delegates FTS5 work to the repository layer.
#[tauri::command]
pub fn global_search(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let connection = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::search::global_search_in_connection(&connection, &query)
}
