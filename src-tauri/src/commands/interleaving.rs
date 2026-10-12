use crate::AppState;
use crate::models::InterleavedReviewCard;
use rusqlite::params;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

#[tauri::command]
pub fn get_interleaved_review_queue(
    deck_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<InterleavedReviewCard>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut queue = Vec::new();

    let mut vocabulary = conn
        .prepare(
            "SELECT c.id, c.deck_id, d.kanji, d.reading, d.meaning, c.state, d.pitch
         FROM cards c JOIN decks deck ON deck.id = c.deck_id
         JOIN dictionary_entries d ON c.entry_id = d.id
         WHERE (c.due_date <= strftime('%s', 'now') OR c.state = 0)
           AND (?1 IS NULL OR c.deck_id = ?1)
         ORDER BY c.due_date LIMIT 100",
        )
        .map_err(|e| e.to_string())?;
    let rows = vocabulary
        .query_map(params![deck_id], |row| {
            Ok(InterleavedReviewCard::Vocabulary {
                card_id: row.get(0)?,
                deck_id: row.get(1)?,
                kanji: row.get(2)?,
                reading: row.get(3)?,
                meaning: row.get(4)?,
                state: row.get(5)?,
                pitch: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;
    queue.extend(
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
    );
    drop(vocabulary);

    let mut images = conn
        .prepare(
            "SELECT image.id, image.deck_id FROM image_occlusion_cards image
         JOIN decks deck ON deck.id = image.deck_id
         WHERE (image.state = 0 OR image.due_date <= strftime('%s', 'now'))
           AND (?1 IS NULL OR image.deck_id = ?1)
         ORDER BY image.due_date LIMIT 100",
        )
        .map_err(|e| e.to_string())?;
    let rows = images
        .query_map(params![deck_id], |row| {
            Ok(InterleavedReviewCard::ImageOcclusion {
                card_id: row.get(0)?,
                deck_id: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;
    queue.extend(
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
    );
    drop(images);

    let mut grammar = conn.prepare(
        "SELECT note.id, note.deck_id, COALESCE(note.title, ''), COALESCE(note.content_md, '')
         FROM notebook_entries note JOIN decks deck ON deck.id = note.deck_id
         WHERE note.srs_stage > 0 AND (note.next_review_date IS NULL OR note.next_review_date <= strftime('%s', 'now'))
           AND (?1 IS NULL OR note.deck_id = ?1)
         ORDER BY note.next_review_date LIMIT 100",
    ).map_err(|e| e.to_string())?;
    let rows = grammar
        .query_map(params![deck_id], |row| {
            Ok(InterleavedReviewCard::Grammar {
                card_id: row.get(0)?,
                deck_id: row.get(1)?,
                title: row.get(2)?,
                content_md: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    queue.extend(
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
    );
    drop(grammar);

    let day_ago = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        - 86_400;
    let mut kana = conn
        .prepare(
            "SELECT character, romaji, COALESCE(alphabet, kana_type), COALESCE(category, 'base')
         FROM kana_mastery WHERE last_practiced = 0 OR last_practiced <= ?1
         ORDER BY last_practiced ASC LIMIT 8",
        )
        .map_err(|e| e.to_string())?;
    let rows = kana
        .query_map(params![day_ago], |row| {
            Ok(InterleavedReviewCard::Kana {
                character: row.get(0)?,
                romaji: row.get(1)?,
                alphabet: row.get(2)?,
                category: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    if deck_id.is_none() {
        queue.extend(
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?,
        );
    }

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    Ok(crate::domain::interleaving::interleave_by_type(queue, seed))
}

#[tauri::command]
pub fn answer_grammar_review(
    card_id: String,
    rating: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !(1..=4).contains(&rating) {
        return Err("Rating must be between 1 and 4".to_string());
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let stage: i32 = conn
        .query_row(
            "SELECT srs_stage FROM notebook_entries WHERE id = ?1",
            params![card_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let (next_stage, delay): (i32, i64) = match rating {
        1 => (stage.max(1), 600),
        2 => (stage.max(1), 43_200),
        3 => (
            stage + 1,
            86_400_i64.saturating_mul(2_i64.pow(stage.clamp(0, 10) as u32)),
        ),
        4 => (
            stage + 2,
            86_400_i64.saturating_mul(4_i64.pow(stage.clamp(0, 8) as u32)),
        ),
        _ => unreachable!(),
    };
    conn.execute("UPDATE notebook_entries SET srs_stage = ?1, next_review_date = ?2, updated_at = ?3 WHERE id = ?4",
        params![next_stage, now + delay, now.to_string(), card_id]).map_err(|e| e.to_string())?;
    Ok(())
}
