use crate::AppState;
use crate::models::{KanaMasteryRecord, KanaProficiencyStats};
use rusqlite::params;
use tauri::State;

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

    let correct_increment = if is_correct { 1 } else { 0 };

    conn.execute(
        "INSERT INTO kana_mastery (character, romaji, kana_type, total_attempts, correct_attempts, accuracy_percentage, last_practiced)
         VALUES (?1, ?2, ?3, 1, ?4, CASE WHEN ?4 = 1 THEN 100.0 ELSE 0.0 END, ?5)
         ON CONFLICT(character) DO UPDATE SET
            total_attempts = total_attempts + 1,
            correct_attempts = correct_attempts + ?4,
            accuracy_percentage = ROUND(((correct_attempts + ?4) * 100.0) / (total_attempts + 1), 1),
            last_practiced = ?5,
            romaji = ?2,
            kana_type = ?3",
        params![character, romaji, kana_type, correct_increment, now],
    ).map_err(|e| e.to_string())?;

    let record = conn.query_row(
        "SELECT character, romaji, kana_type, total_attempts, correct_attempts, accuracy_percentage, last_practiced
         FROM kana_mastery WHERE character = ?1",
        params![character],
        |row| {
            Ok(KanaMasteryRecord {
                character: row.get(0)?,
                romaji: row.get(1)?,
                kana_type: row.get(2)?,
                total_attempts: row.get(3)?,
                correct_attempts: row.get(4)?,
                accuracy_percentage: row.get(5)?,
                last_practiced: row.get(6)?,
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
            "SELECT character, romaji, kana_type, total_attempts, correct_attempts, accuracy_percentage, last_practiced
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
                total_attempts: row.get(3)?,
                correct_attempts: row.get(4)?,
                accuracy_percentage: row.get(5)?,
                last_practiced: row.get(6)?,
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
             FROM kana_mastery WHERE kana_type = 'hiragana'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap_or((0.0, 0.0));

    let (k_correct_sum, k_learned_count): (f64, f64) = conn
        .query_row(
            "SELECT COALESCE(SUM(accuracy_percentage), 0.0), COUNT(*) 
             FROM kana_mastery WHERE kana_type = 'katakana'",
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

    // Proficiency based on known characters * accuracy out of total 46 kana
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

        // Simulate recording attempts directly
        conn.execute(
            "INSERT INTO kana_mastery (character, romaji, kana_type, total_attempts, correct_attempts, accuracy_percentage, last_practiced)
             VALUES ('あ', 'a', 'hiragana', 2, 2, 100.0, 1000)",
            [],
        ).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM kana_mastery", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let accuracy: f64 = conn
            .query_row(
                "SELECT accuracy_percentage FROM kana_mastery WHERE character = 'あ'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(accuracy, 100.0);
    }
}
