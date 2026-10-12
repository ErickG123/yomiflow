use crate::AppState;
use crate::models::{BunsetsuChunk, LintSuggestion, MorphologicalToken, NotebookEntry};
use crate::models::{ReadabilityAnalysis, UnknownWordOccurrence};
use rusqlite::params;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub fn analyze_japanese_text(text: String) -> Result<Vec<MorphologicalToken>, String> {
    crate::domain::nlp::segment_and_tag_japanese(&text)
}

#[tauri::command]
pub fn generate_furigana(text: String) -> Result<String, String> {
    crate::domain::nlp::generate_furigana(&text)
}

#[tauri::command]
pub fn lint_japanese_text(text: String) -> Result<Vec<LintSuggestion>, String> {
    crate::domain::nlp::lint_japanese_text(&text)
}

#[tauri::command]
pub fn analyze_text_readability(
    text: String,
    state: State<'_, AppState>,
) -> Result<ReadabilityAnalysis, String> {
    let tokens = crate::domain::nlp::extract_lexical_tokens(&text)?;
    if tokens.is_empty() {
        return Ok(ReadabilityAnalysis {
            total_words: 0,
            known_words: 0,
            comprehension_percentage: 0.0,
            average_retention_percentage: 0.0,
            unknown_words: Vec::new(),
        });
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut statement = conn
        .prepare(
            "SELECT c.state, c.stability, c.reps, COALESCE(MAX(log.review_datetime), 0)
             FROM dictionary_entries d
             JOIN cards c ON c.entry_id = d.id
             LEFT JOIN review_logs log ON log.card_id = c.id
             WHERE d.kanji = ?1 COLLATE NOCASE OR d.reading = ?2 COLLATE NOCASE
             GROUP BY c.id",
        )
        .map_err(|e| e.to_string())?;
    let mut retention_cache: HashMap<(String, String), (bool, f64)> = HashMap::new();
    let mut unknown_words = Vec::new();
    let mut known_words = 0;
    let mut total_retention = 0.0;

    for token in &tokens {
        let key = (token.lemma.clone(), token.reading.clone());
        let (is_known, retention) = if let Some(value) = retention_cache.get(&key) {
            *value
        } else {
            let rows = statement
                .query_map(rusqlite::params![token.lemma, token.reading], |row| {
                    Ok((
                        row.get::<_, i32>(0)?,
                        row.get::<_, f64>(1)?,
                        row.get::<_, i32>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                })
                .map_err(|e| e.to_string())?;
            let mut best_retention: f64 = 0.0;
            let mut found_card = false;
            for row in rows {
                let (card_state, stability, reps, last_review) = row.map_err(|e| e.to_string())?;
                found_card = true;
                best_retention = best_retention.max(calculate_retention(
                    card_state,
                    stability,
                    reps,
                    last_review,
                    now,
                ));
            }
            retention_cache.insert(key, (found_card, best_retention));
            (found_card, best_retention)
        };

        total_retention += retention;
        if is_known {
            known_words += 1;
        } else {
            let start = text[..token.start_byte].encode_utf16().count();
            let end = text[..token.end_byte].encode_utf16().count();
            unknown_words.push(UnknownWordOccurrence {
                surface: token.surface.clone(),
                lemma: token.lemma.clone(),
                start,
                end,
                retention_percentage: 0.0,
            });
        }
    }

    Ok(ReadabilityAnalysis {
        total_words: tokens.len(),
        known_words,
        comprehension_percentage: (known_words as f64 * 1000.0 / tokens.len() as f64).round()
            / 10.0,
        average_retention_percentage: if known_words > 0 {
            (total_retention * 1000.0 / known_words as f64).round() / 10.0
        } else {
            0.0
        },
        unknown_words,
    })
}

fn calculate_retention(
    card_state: i32,
    stability: f64,
    reps: i32,
    last_review: i64,
    now: i64,
) -> f64 {
    if card_state == 0 || reps <= 0 || stability <= 0.0 {
        return 0.0;
    }
    let elapsed_days = (now.saturating_sub(last_review) as f64 / 86_400.0).max(0.1);
    let retrievability = (1.0 + 19.0 * elapsed_days / (9.0 * stability)).powf(-0.5);
    match card_state {
        1 => retrievability * 0.65,
        2 => retrievability,
        _ => 0.0,
    }
}

#[tauri::command]
pub fn generate_syntax_tree(text: String) -> Result<Vec<BunsetsuChunk>, String> {
    crate::domain::nlp::generate_syntax_tree(&text)
}

#[tauri::command]
pub fn inject_furigana_epub(
    html_text: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Cross-reference with SRS: only words with high retention stage / mastery skip furigana.
    // Words with low retention (e.g. state == 0 / new, or stability < 10.0, or reps < 3) still receive furigana.
    let mastered_words = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT d.kanji 
                 FROM cards c 
                 JOIN dictionary_entries d ON c.entry_id = d.id 
                 WHERE d.kanji IS NOT NULL 
                   AND (c.stability >= 10.0 OR (c.state = 2 AND c.reps >= 3))",
            )
            .map_err(|e| e.to_string())?;

        stmt.query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect::<std::collections::HashSet<String>>()
    };

    crate::domain::nlp::inject_furigana_for_unlearned_words(&html_text, &mastered_words)
}

#[tauri::command]
pub fn save_notebook_entry(
    id: String,
    title: String,
    content_md: String,
    deck_id: Option<String>,
    next_review_date: Option<i64>,
    srs_stage: Option<i32>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    conn.execute(
        "INSERT INTO notebook_entries (id, title, content_md, created_at, updated_at, deck_id, next_review_date, srs_stage) 
         VALUES (?1, ?2, ?3, ?4, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET 
            title = ?2, 
            content_md = ?3, 
            updated_at = ?4,
            deck_id = COALESCE(?5, notebook_entries.deck_id),
            next_review_date = COALESCE(?6, notebook_entries.next_review_date),
            srs_stage = COALESCE(?7, notebook_entries.srs_stage)",
        params![id, title, content_md, now.to_string(), deck_id, next_review_date, srs_stage.unwrap_or(0)],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn link_notebook_to_deck(
    id: String,
    deck_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    conn.execute(
        "UPDATE notebook_entries SET deck_id = ?1, updated_at = ?2 WHERE id = ?3",
        params![deck_id, now.to_string(), id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_notebook_entries(state: State<'_, AppState>) -> Result<Vec<NotebookEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    get_notebook_entries_from_connection(&conn).map_err(|e| e.to_string())
}

fn get_notebook_entries_from_connection(
    conn: &rusqlite::Connection,
) -> rusqlite::Result<Vec<NotebookEntry>> {
    let mut stmt = conn
        .prepare("SELECT id, title, content_md, CAST(created_at AS TEXT), CAST(updated_at AS TEXT), deck_id, next_review_date, srs_stage FROM notebook_entries ORDER BY updated_at DESC")?;

    let entries = stmt
        .query_map([], |row| {
            Ok(NotebookEntry {
                id: row.get(0)?,
                title: row.get(1)?,
                content_md: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                deck_id: row.get(5)?,
                next_review_date: row.get(6)?,
                srs_stage: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<NotebookEntry>, rusqlite::Error>>()?;

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notebook_entry_timestamps_are_returned_as_strings_for_integer_storage() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE notebook_entries (
                id TEXT PRIMARY KEY,
                title TEXT,
                content_md TEXT,
                created_at DATETIME,
                updated_at DATETIME,
                deck_id TEXT,
                next_review_date DATETIME,
                srs_stage INTEGER
            );
            INSERT INTO notebook_entries VALUES ('note-1', 'Title', 'Body', 1792065600, 1792065600, NULL, NULL, 0);",
        )
        .unwrap();

        let entries = get_notebook_entries_from_connection(&conn).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].created_at, "1792065600");
        assert_eq!(entries[0].updated_at, "1792065600");
    }

    #[test]
    fn test_analyze_empty_and_whitespace_text_no_panic() {
        let empty_res = analyze_japanese_text("".to_string()).unwrap();
        assert!(empty_res.is_empty());

        let whitespace_res = analyze_japanese_text("   \n\t ".to_string()).unwrap();
        assert!(whitespace_res.is_empty());
    }

    #[test]
    fn test_analyze_japanese_sentence_pos_tagging() {
        let text = "私は学生です".to_string();
        let tokens = analyze_japanese_text(text).unwrap();

        assert!(!tokens.is_empty());
        // 私 -> noun
        assert_eq!(tokens[0].surface, "私");
        assert_eq!(tokens[0].pos, "noun");

        // は -> particle
        assert_eq!(tokens[1].surface, "は");
        assert_eq!(tokens[1].pos, "particle");

        // 学生 -> noun
        assert_eq!(tokens[2].surface, "学生");
        assert_eq!(tokens[2].pos, "noun");

        // です -> auxiliary
        assert_eq!(tokens[3].surface, "です");
        assert_eq!(tokens[3].pos, "auxiliary");
    }

    #[test]
    fn test_analyze_verb_tagging() {
        let text = "食べる".to_string();
        let tokens = analyze_japanese_text(text).unwrap();
        assert_eq!(tokens[0].surface, "食べる");
        assert_eq!(tokens[0].pos, "verb");
    }
}
