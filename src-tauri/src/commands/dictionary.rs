use crate::AppState;
use crate::models::{DictionaryEntry, KanjiEntry};
use rusqlite::params;
use tauri::State;

#[derive(serde::Deserialize)]
struct JishoResponse {
    data: Vec<JishoItem>,
}

#[derive(serde::Deserialize)]
struct JishoItem {
    japanese: Vec<JishoJapanese>,
    senses: Vec<JishoSense>,
}

#[derive(serde::Deserialize)]
struct JishoJapanese {
    word: Option<String>,
    reading: Option<String>,
}

#[derive(serde::Deserialize)]
struct JishoSense {
    english_definitions: Vec<String>,
    parts_of_speech: Vec<String>,
    #[serde(default)]
    sentences: Option<Vec<JishoSentence>>,
    #[serde(default)]
    examples: Option<Vec<JishoSentence>>,
}

#[derive(serde::Deserialize)]
struct JishoSentence {
    #[serde(default)]
    english: Option<String>,
    #[serde(default)]
    japanese: Option<String>,
    #[serde(default)]
    text: Option<String>,
}

pub async fn query_jisho_fallback(keyword: &str) -> Vec<DictionaryEntry> {
    if keyword.trim().is_empty() {
        return Vec::new();
    }

    let url = format!(
        "https://jisho.org/api/v1/search/words?keyword={}",
        urlencoding_simple(keyword)
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("YomiFlow-App/0.1.0")
        .build();

    let client = match client {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return Vec::new(),
    };

    let jisho_data: JishoResponse = match resp.json().await {
        Ok(data) => data,
        Err(_) => return Vec::new(),
    };

    let mut results = Vec::new();
    for (idx, item) in jisho_data.data.into_iter().take(20).enumerate() {
        let jp = match item.japanese.into_iter().next() {
            Some(j) => j,
            None => continue,
        };

        let kanji = jp.word;
        let reading = jp
            .reading
            .unwrap_or_else(|| kanji.clone().unwrap_or_default());
        if reading.is_empty() && kanji.is_none() {
            continue;
        }

        let mut example_sentence = None;

        let meanings = item
            .senses
            .into_iter()
            .take(3)
            .map(|s| {
                if example_sentence.is_none() {
                    let first_example = s
                        .sentences
                        .as_ref()
                        .and_then(|list| list.first())
                        .or_else(|| s.examples.as_ref().and_then(|list| list.first()));

                    if let Some(ex) = first_example {
                        let jp_text = ex.japanese.as_deref().or(ex.text.as_deref());
                        let en_text = ex.english.as_deref();
                        if let Some(jp) = jp_text {
                            example_sentence = Some(match en_text {
                                Some(en) => format!("{} ({})", jp, en),
                                None => jp.to_string(),
                            });
                        }
                    }
                }

                let pos = if !s.parts_of_speech.is_empty() {
                    format!("({}) ", s.parts_of_speech.join(", "))
                } else {
                    String::new()
                };
                format!("{}{}", pos, s.english_definitions.join("; "))
            })
            .collect::<Vec<_>>()
            .join(" | ");

        if meanings.is_empty() {
            continue;
        }

        // Generate dynamic negative ID for transient online entries
        let synthetic_id = -1000 - (idx as i64);

        results.push(DictionaryEntry {
            id: synthetic_id,
            kanji,
            reading,
            meaning: meanings,
            in_srs: false,
            rank: None,
            pitch: None,
            example_sentence,
        });
    }

    results
}

fn urlencoding_simple(s: &str) -> String {
    let mut encoded = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
            encoded.push(b as char);
        } else {
            encoded.push_str(&format!("%{:02X}", b));
        }
    }
    encoded
}

#[tauri::command]
pub async fn search_dictionary(
    query: String,
    deck_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<DictionaryEntry>, String> {
    let local_entries = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let search_pattern = format!("%{}%", query);

        let mut stmt = conn
            .prepare(
                "SELECT d.id, d.kanji, d.reading, d.meaning,
                        CASE WHEN EXISTS (
                            SELECT 1 FROM cards c WHERE c.entry_id = d.id
                            AND (?2 IS NULL OR c.deck_id = ?2)
                        ) THEN 1 ELSE 0 END as in_srs,
                        wf.rank,
                        d.pitch
                 FROM dictionary_entries d
                 LEFT JOIN word_frequencies wf ON (wf.term = d.kanji OR wf.term = d.reading)
                 WHERE d.kanji LIKE ?1 OR d.reading LIKE ?1 OR d.meaning LIKE ?1 
                 ORDER BY CASE WHEN wf.rank IS NOT NULL THEN 0 ELSE 1 END, wf.rank ASC, d.id ASC
                 LIMIT 50",
            )
            .map_err(|e| e.to_string())?;

        stmt.query_map(params![search_pattern, deck_id], |row| {
            Ok(DictionaryEntry {
                id: row.get(0)?,
                kanji: row.get(1)?,
                reading: row.get(2)?,
                meaning: row.get(3)?,
                in_srs: row.get::<_, i32>(4)? == 1,
                rank: row.get(5)?,
                pitch: row.get(6)?,
                example_sentence: None,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<DictionaryEntry>, rusqlite::Error>>()
        .map_err(|e| e.to_string())?
    };

    if !local_entries.is_empty() {
        return Ok(local_entries);
    }

    // Silent fallback to Jisho REST API
    let jisho_results = query_jisho_fallback(&query).await;
    Ok(jisho_results)
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

    let mut query = String::from(
        "SELECT character, onyomi, kunyomi, meaning, jlpt_level FROM kanjis WHERE character IN (",
    );
    for index in 0..unique_chars.len() {
        if index > 0 {
            query.push_str(", ");
        }
        query.push('?');
    }
    query.push(')');

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

#[tauri::command]
pub fn add_custom_word_to_srs(
    kanji: Option<String>,
    reading: String,
    meaning: String,
    deck_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let target_deck = crate::db::srs::resolve_deck_id(&conn, deck_id)?;

    // Check if matching entry already exists
    let existing_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM dictionary_entries WHERE reading = ?1 AND ((kanji IS NULL AND ?2 IS NULL) OR kanji = ?2) LIMIT 1",
            params![reading, kanji],
            |row| row.get(0),
        )
        .ok();

    let entry_id = match existing_id {
        Some(id) => id,
        None => {
            conn.execute(
                "INSERT INTO dictionary_entries (kanji, reading, meaning) VALUES (?1, ?2, ?3)",
                params![kanji, reading, meaning],
            )
            .map_err(|e| e.to_string())?;
            conn.last_insert_rowid()
        }
    };

    let card_id = uuid::Uuid::new_v4().to_string();
    let current_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;

    // Check if card is already in SRS
    let card_exists: bool = conn
        .query_row(
            "SELECT 1 FROM cards WHERE entry_id = ?1 AND deck_id = ?2 LIMIT 1",
            params![entry_id, target_deck],
            |_| Ok(true),
        )
        .unwrap_or(false);

    if !card_exists {
        conn.execute(
            "INSERT INTO cards (id, entry_id, deck_id, due_date, stability, difficulty, reps, state)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                card_id,
                entry_id,
                target_deck,
                current_timestamp,
                0.0_f64,
                0.0_f64,
                0_i32,
                0_i32
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(entry_id)
}

#[tauri::command]
pub fn add_readability_word_to_srs(
    surface: String,
    lemma: String,
    reading: String,
    context: String,
    include_context: bool,
    deck_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut conn = state.db.lock().map_err(|error| error.to_string())?;
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    let target_deck = crate::db::srs::resolve_deck_id(&tx, deck_id)?;
    let (kanji, meaning) = if include_context {
        (surface.clone(), context)
    } else {
        (lemma.clone(), String::new())
    };

    let existing_id: Option<i64> = tx
        .query_row(
            "SELECT id FROM dictionary_entries WHERE kanji = ?1 AND reading = ?2 AND meaning = ?3 LIMIT 1",
            params![kanji, reading, meaning],
            |row| row.get(0),
        )
        .ok();
    let entry_id = match existing_id {
        Some(id) => id,
        None => {
            tx.execute(
                "INSERT INTO dictionary_entries (kanji, reading, meaning) VALUES (?1, ?2, ?3)",
                params![kanji, reading, meaning],
            )
            .map_err(|error| error.to_string())?;
            tx.last_insert_rowid()
        }
    };

    crate::db::srs::add_to_srs(&tx, entry_id, Some(target_deck))?;
    tx.commit().map_err(|error| error.to_string())
}
