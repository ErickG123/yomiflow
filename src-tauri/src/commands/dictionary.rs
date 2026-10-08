use crate::AppState;
use crate::models::{DictionaryEntry, KanjiEntry};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn search_dictionary(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<DictionaryEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let search_pattern = format!("%{}%", query);

    let mut stmt = conn
        .prepare(
            "SELECT d.id, d.kanji, d.reading, d.meaning, CASE WHEN c.id IS NOT NULL THEN 1 ELSE 0 END as in_srs
             FROM dictionary_entries d
             LEFT JOIN cards c ON d.id = c.entry_id
             WHERE d.kanji LIKE ?1 OR d.reading LIKE ?1 OR d.meaning LIKE ?1 
             LIMIT 50",
        )
        .map_err(|e| e.to_string())?;

    let entries = stmt
        .query_map(params![search_pattern], |row| {
            Ok(DictionaryEntry {
                id: row.get(0)?,
                kanji: row.get(1)?,
                reading: row.get(2)?,
                meaning: row.get(3)?,
                in_srs: row.get::<_, i32>(4)? == 1,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<DictionaryEntry>, rusqlite::Error>>()
        .map_err(|e| e.to_string())?;

    Ok(entries)
}

#[tauri::command]
pub async fn search_kanjis(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<KanjiEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let search_pattern = format!("%{}%", query);

    let mut stmt = conn
        .prepare(
            "SELECT character, onyomi, kunyomi, meaning, jlpt_level 
             FROM kanjis 
             WHERE character LIKE ?1 OR meaning LIKE ?1 
             LIMIT 20",
        )
        .map_err(|e| e.to_string())?;

    let kanjis = stmt
        .query_map(params![search_pattern], |row| {
            Ok(KanjiEntry {
                character: row.get(0)?,
                onyomi: row.get(1)?,
                kunyomi: row.get(2)?,
                meaning: row.get(3)?,
                jlpt_level: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<KanjiEntry>, rusqlite::Error>>()
        .map_err(|e| e.to_string())?;

    Ok(kanjis)
}

#[tauri::command]
pub fn get_kanji_breakdown(
    word: String,
    state: State<'_, AppState>,
) -> Result<Vec<KanjiEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Extract unique characters from the string
    let mut unique_chars = Vec::new();
    for c in word.chars() {
        let s = c.to_string();
        if !unique_chars.contains(&s) {
            unique_chars.push(s);
        }
    }

    if unique_chars.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = unique_chars
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        "SELECT character, onyomi, kunyomi, meaning, jlpt_level FROM kanjis WHERE character IN ({})",
        placeholders
    );

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let params: Vec<&dyn rusqlite::ToSql> = unique_chars
        .iter()
        .map(|s| s as &dyn rusqlite::ToSql)
        .collect();

    let kanjis = stmt
        .query_map(params.as_slice(), |row| {
            Ok(KanjiEntry {
                character: row.get(0)?,
                onyomi: row.get(1)?,
                kunyomi: row.get(2)?,
                meaning: row.get(3)?,
                jlpt_level: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<KanjiEntry>, rusqlite::Error>>()
        .map_err(|e| e.to_string())?;

    Ok(kanjis)
}
