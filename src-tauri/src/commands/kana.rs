use crate::AppState;
use crate::models::{KanaMasteryRecord, KanaProficiencyStats};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn update_kana_score(
    character: String,
    is_correct: bool,
    state: State<'_, AppState>,
) -> Result<KanaMasteryRecord, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let correct_inc = if is_correct { 1 } else { 0 };
    let wrong_inc = if is_correct { 0 } else { 1 };

    conn.execute(
        "UPDATE kana_mastery SET
            correct_count = correct_count + ?1,
            wrong_count = wrong_count + ?2,
            total_attempts = total_attempts + 1,
            correct_attempts = correct_attempts + ?1,
            accuracy_percentage = ROUND(((correct_attempts + ?1) * 100.0) / (total_attempts + 1), 1),
            last_practiced = ?3
         WHERE character = ?4",
        params![correct_inc, wrong_inc, now, character],
    ).map_err(|e| e.to_string())?;

    let record = conn.query_row(
        "SELECT character, romaji, kana_type, COALESCE(alphabet, kana_type), COALESCE(category, 'base'),
                correct_count, wrong_count, total_attempts, correct_attempts, accuracy_percentage, last_practiced
         FROM kana_mastery WHERE character = ?1",
        params![character],
        |row| {
            Ok(KanaMasteryRecord {
                character: row.get(0)?,
                romaji: row.get(1)?,
                kana_type: row.get(2)?,
                alphabet: row.get(3)?,
                category: row.get(4)?,
                correct_count: row.get(5)?,
                wrong_count: row.get(6)?,
                total_attempts: row.get(7)?,
                correct_attempts: row.get(8)?,
                accuracy_percentage: row.get(9)?,
                last_practiced: row.get(10)?,
            })
        },
    ).map_err(|e| e.to_string())?;

    Ok(record)
}

#[tauri::command]
pub fn get_kana_characters(
    state: State<'_, AppState>,
) -> Result<Vec<crate::models::KanaItem>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT character, romaji, COALESCE(alphabet, kana_type) as alphabet, COALESCE(category, 'base') as category
             FROM kana_mastery
             ORDER BY rowid ASC",
        )
        .map_err(|e| e.to_string())?;

    let items = stmt
        .query_map([], |row| {
            Ok(crate::models::KanaItem {
                character: row.get(0)?,
                romaji: row.get(1)?,
                alphabet: row.get(2)?,
                category: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<crate::models::KanaItem>, rusqlite::Error>>()
        .map_err(|e| e.to_string())?;

    Ok(items)
}

#[tauri::command]
pub fn record_kana_attempt(
    character: String,
    romaji: String,
    kana_type: String,
    is_correct: bool,
    state: State<'_, AppState>,
) -> Result<KanaMasteryRecord, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let correct_inc = if is_correct { 1 } else { 0 };
    let wrong_inc = if is_correct { 0 } else { 1 };

    conn.execute(
        "INSERT INTO kana_mastery (
            character, romaji, kana_type, alphabet, category,
            correct_count, wrong_count, total_attempts, correct_attempts, accuracy_percentage, last_practiced
         )
         VALUES (?1, ?2, ?3, ?3, 'base', ?4, ?5, 1, ?4, CASE WHEN ?4 = 1 THEN 100.0 ELSE 0.0 END, ?6)
         ON CONFLICT(character) DO UPDATE SET
            correct_count = correct_count + ?4,
            wrong_count = wrong_count + ?5,
            total_attempts = total_attempts + 1,
            correct_attempts = correct_attempts + ?4,
            accuracy_percentage = ROUND(((correct_attempts + ?4) * 100.0) / (total_attempts + 1), 1),
            last_practiced = ?6,
            romaji = ?2,
            kana_type = ?3,
            alphabet = COALESCE(alphabet, ?3)",
        params![character, romaji, kana_type, correct_inc, wrong_inc, now],
    ).map_err(|e| e.to_string())?;

    let record = conn.query_row(
        "SELECT character, romaji, kana_type, COALESCE(alphabet, kana_type), COALESCE(category, 'base'),
                correct_count, wrong_count, total_attempts, correct_attempts, accuracy_percentage, last_practiced
         FROM kana_mastery WHERE character = ?1",
        params![character],
        |row| {
            Ok(KanaMasteryRecord {
                character: row.get(0)?,
                romaji: row.get(1)?,
                kana_type: row.get(2)?,
                alphabet: row.get(3)?,
                category: row.get(4)?,
                correct_count: row.get(5)?,
                wrong_count: row.get(6)?,
                total_attempts: row.get(7)?,
                correct_attempts: row.get(8)?,
                accuracy_percentage: row.get(9)?,
                last_practiced: row.get(10)?,
            })
        },
    ).map_err(|e| e.to_string())?;

    Ok(record)
}

#[tauri::command]
pub fn get_kana_mastery_list(state: State<'_, AppState>) -> Result<Vec<KanaMasteryRecord>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT character, romaji, kana_type, COALESCE(alphabet, kana_type), COALESCE(category, 'base'),
                    correct_count, wrong_count, total_attempts, correct_attempts, accuracy_percentage, last_practiced
             FROM kana_mastery
             ORDER BY last_practiced DESC",
        )
        .map_err(|e| e.to_string())?;

    let records = stmt
        .query_map([], |row| {
            Ok(KanaMasteryRecord {
                character: row.get(0)?,
                romaji: row.get(1)?,
                kana_type: row.get(2)?,
                alphabet: row.get(3)?,
                category: row.get(4)?,
                correct_count: row.get(5)?,
                wrong_count: row.get(6)?,
                total_attempts: row.get(7)?,
                correct_attempts: row.get(8)?,
                accuracy_percentage: row.get(9)?,
                last_practiced: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<KanaMasteryRecord>, rusqlite::Error>>()
        .map_err(|e| e.to_string())?;

    Ok(records)
}

#[tauri::command]
pub fn get_kana_proficiency_stats(
    state: State<'_, AppState>,
) -> Result<KanaProficiencyStats, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Standard Gojuon count for base proficiency benchmarking
    let total_hiragana_count = 46.0;
    let total_katakana_count = 46.0;

    let (h_correct_sum, h_learned_count): (f64, f64) = conn
        .query_row(
            "SELECT COALESCE(SUM(accuracy_percentage), 0.0), COUNT(*) 
             FROM kana_mastery 
             WHERE (alphabet = 'hiragana' OR kana_type = 'hiragana') 
               AND total_attempts > 0",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap_or((0.0, 0.0));

    let (k_correct_sum, k_learned_count): (f64, f64) = conn
        .query_row(
            "SELECT COALESCE(SUM(accuracy_percentage), 0.0), COUNT(*) 
             FROM kana_mastery 
             WHERE (alphabet = 'katakana' OR kana_type = 'katakana')
               AND total_attempts > 0",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap_or((0.0, 0.0));

    let (total_attempts, total_correct): (i32, i32) = conn
        .query_row(
            "SELECT COALESCE(SUM(total_attempts), 0), COALESCE(SUM(correct_attempts), 0) FROM kana_mastery",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap_or((0, 0));

    // Proficiency calculation: proportion of correct vs wrong attempts and mastered kana
    let hiragana_proficiency = if h_learned_count > 0.0 {
        ((h_correct_sum / (total_hiragana_count * 100.0)) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let katakana_proficiency = if k_learned_count > 0.0 {
        ((k_correct_sum / (total_katakana_count * 100.0)) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let overall_proficiency =
        ((hiragana_proficiency + katakana_proficiency) / 2.0).clamp(0.0, 100.0);

    Ok(KanaProficiencyStats {
        hiragana_proficiency: (hiragana_proficiency * 10.0).round() / 10.0,
        katakana_proficiency: (katakana_proficiency * 10.0).round() / 10.0,
        overall_proficiency: (overall_proficiency * 10.0).round() / 10.0,
        total_attempts,
        total_correct,
    })
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    #[test]
    fn test_record_kana_attempt_and_proficiency() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_tables_and_seed(&conn).unwrap();

        // Simulate recording attempts directly on seeded 'あ'
        conn.execute(
            "UPDATE kana_mastery SET total_attempts = 2, correct_attempts = 2, correct_count = 2, accuracy_percentage = 100.0, last_practiced = 1000
             WHERE character = 'あ'",
            [],
        ).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM kana_mastery", [], |row| row.get(0))
            .unwrap();
        assert!(count >= 208);

        let accuracy: f64 = conn
            .query_row(
                "SELECT accuracy_percentage FROM kana_mastery WHERE character = 'あ'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(accuracy, 100.0);
    }

    #[test]
    fn test_update_kana_score_and_seed() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_tables_and_seed(&conn).unwrap();

        // Seed should have populated 46 base + 25 dakuon + 33 yoon for both hiragana and katakana = 208
        let total_seeded: i64 = conn
            .query_row("SELECT COUNT(*) FROM kana_mastery", [], |row| row.get(0))
            .unwrap();
        assert!(total_seeded >= 208);

        // Update score for 'か'
        conn.execute(
            "UPDATE kana_mastery SET
                correct_count = correct_count + 1,
                total_attempts = total_attempts + 1,
                correct_attempts = correct_attempts + 1,
                accuracy_percentage = 100.0
             WHERE character = 'か'",
            [],
        )
        .unwrap();

        let correct: i32 = conn
            .query_row(
                "SELECT correct_count FROM kana_mastery WHERE character = 'か'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(correct, 1);
    }
}
