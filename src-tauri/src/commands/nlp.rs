use crate::AppState;
use crate::models::{MorphologicalToken, NotebookEntry};
use lindera::dictionary::load_dictionary;
use lindera::mode::Mode;
use lindera::segmenter::Segmenter;
use rusqlite::params;
use std::borrow::Cow;
use std::sync::Mutex;
use tauri::State;

static SEGMENTER: Mutex<Option<Segmenter>> = Mutex::new(None);

fn get_or_create_segmenter<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&mut Segmenter) -> Result<R, String>,
{
    let mut lock = SEGMENTER
        .lock()
        .map_err(|e| format!("Falha no mutex do segmenter: {}", e))?;

    if lock.is_none() {
        let dictionary = load_dictionary("embedded://ipadic")
            .map_err(|e| format!("Falha ao carregar dicionário IPADIC embutido do Lindera: {}", e))?;
        *lock = Some(Segmenter::new(Mode::Normal, dictionary, None));
    }

    let segmenter = lock.as_mut().unwrap();
    f(segmenter)
}

fn map_pos_category(pos_str: &str) -> String {
    if pos_str.starts_with("動詞") {
        "verb".to_string()
    } else if pos_str.starts_with("助詞") {
        "particle".to_string()
    } else if pos_str.starts_with("名詞") {
        "noun".to_string()
    } else if pos_str.starts_with("形容詞") {
        "adjective".to_string()
    } else if pos_str.starts_with("助動詞") {
        "auxiliary".to_string()
    } else {
        "other".to_string()
    }
}

#[tauri::command]
pub fn analyze_japanese_text(text: String) -> Result<Vec<MorphologicalToken>, String> {
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }

    get_or_create_segmenter(|segmenter| {
        let mut tokens = segmenter
            .segment(Cow::Borrowed(&text))
            .map_err(|e| format!("Erro durante análise morfológica Lindera: {}", e))?;

        let mut result = Vec::new();
        for token in tokens.iter_mut() {
            let surface = token.surface.to_string();
            let details = token.details();

            let raw_pos = details.first().copied().unwrap_or("未知語");
            let pos = map_pos_category(raw_pos);

            let reading = details.get(7).map(|s| s.to_string());

            result.push(MorphologicalToken {
                surface,
                reading,
                pos,
            });
        }

        Ok(result)
    })
}

#[tauri::command]
pub fn save_notebook_entry(
    id: String,
    title: String,
    content_md: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    conn.execute(
        "INSERT INTO notebook_entries (id, title, content_md, created_at, updated_at) 
         VALUES (?1, ?2, ?3, ?4, ?4)
         ON CONFLICT(id) DO UPDATE SET title = ?2, content_md = ?3, updated_at = ?4",
        params![id, title, content_md, now.to_string()],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_notebook_entries(state: State<'_, AppState>) -> Result<Vec<NotebookEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, title, content_md, created_at, updated_at FROM notebook_entries ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;

    let entries = stmt
        .query_map([], |row| {
            Ok(NotebookEntry {
                id: row.get(0)?,
                title: row.get(1)?,
                content_md: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<NotebookEntry>, rusqlite::Error>>()
        .map_err(|e| e.to_string())?;

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

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
