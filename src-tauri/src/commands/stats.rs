use crate::AppState;
use crate::models::HeatmapEntry;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn get_review_forecast(
    start_date: String,
    end_date: String,
    state: State<'_, AppState>,
) -> Result<Vec<HeatmapEntry>, String> {
    crate::domain::stats::validate_date_ymd(&start_date)?;
    crate::domain::stats::validate_date_ymd(&end_date)?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT forecast_date, SUM(count) as total_count FROM (
                SELECT 
                    DATE(datetime(due_date, 'unixepoch', 'localtime')) as forecast_date, 
                    COUNT(*) as count 
                FROM cards 
                WHERE DATE(datetime(due_date, 'unixepoch', 'localtime')) BETWEEN ?1 AND ?2
                GROUP BY forecast_date 
                UNION ALL 
                SELECT 
                    DATE(datetime(next_review_date, 'unixepoch', 'localtime')) as forecast_date, 
                    COUNT(*) as count 
                FROM notebook_entries 
                WHERE next_review_date IS NOT NULL 
                  AND DATE(datetime(next_review_date, 'unixepoch', 'localtime')) BETWEEN ?1 AND ?2
                GROUP BY forecast_date
             )
             GROUP BY forecast_date 
             ORDER BY forecast_date ASC",
        )
        .map_err(|e| e.to_string())?;

    let entries = stmt
        .query_map(params![start_date, end_date], |row| {
            Ok(HeatmapEntry {
                date: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<HeatmapEntry>, rusqlite::Error>>()
        .map_err(|e| e.to_string())?;

    Ok(entries)
}

#[tauri::command]
pub fn get_study_history(
    year: u32,
    month: u32,
    state: State<'_, AppState>,
) -> Result<Vec<HeatmapEntry>, String> {
    let prefix = crate::domain::stats::format_year_month(year, month)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT 
                DATE(datetime(review_datetime, 'unixepoch', 'localtime')) as history_date, 
                COUNT(*) as count 
             FROM review_logs 
             WHERE strftime('%Y-%m', datetime(review_datetime, 'unixepoch', 'localtime')) = ?1
             GROUP BY history_date 
             ORDER BY history_date ASC",
        )
        .map_err(|e| e.to_string())?;

    let entries = stmt
        .query_map(params![prefix], |row| {
            Ok(HeatmapEntry {
                date: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<HeatmapEntry>, rusqlite::Error>>()
        .map_err(|e| e.to_string())?;

    Ok(entries)
}

#[tauri::command]
pub fn get_jlpt_coverage_stats(
    state: State<'_, AppState>,
) -> Result<crate::models::JlptAnalyticsStats, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Standard official JLPT benchmark totals (kanji, vocabulary)
    // N5: 103 kanjis, 800 vocab
    // N4: 181 kanjis, 1500 vocab
    // N3: 343 kanjis, 3750 vocab
    // N2: 415 kanjis, 6000 vocab
    // N1: 1136 kanjis, 10000 vocab
    let benchmark_data = [
        (5, "N5", 103, 800),
        (4, "N4", 181, 1500),
        (3, "N3", 343, 3750),
        (2, "N2", 415, 6000),
        (1, "N1", 1136, 10000),
    ];

    let mut levels = Vec::new();
    let mut total_learned_kanji_all = 0;
    let mut total_learned_vocab_all = 0;

    for (level, name, default_kanji_total, default_vocab_total) in benchmark_data {
        // Query total kanjis registered in database for this level if available, otherwise benchmark
        let db_kanji_total: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM kanjis WHERE jlpt_level = ?1",
                params![level],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let total_kanji = if db_kanji_total > 0 {
            db_kanji_total.max(default_kanji_total)
        } else {
            default_kanji_total
        };

        // Query distinct kanji learned in cards table for this level
        // A card is considered learned if state > 0 (Learning, Review, or Relearning) or reps > 0
        let learned_kanji: i32 = conn
            .query_row(
                "SELECT COUNT(DISTINCT k.character) 
                 FROM kanjis k
                 JOIN dictionary_entries d ON (d.kanji LIKE '%' || k.character || '%')
                 JOIN cards c ON c.entry_id = d.id
                 WHERE k.jlpt_level = ?1 AND (c.reps > 0 OR c.state > 0)",
                params![level],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Query cards tagged with JLPT level or associated with kanji of this level
        // If cards have tags like 'N5', 'n5', 'JLPT N5', or entries with kanji exclusively in this level
        let learned_vocab: i32 = conn
            .query_row(
                "SELECT COUNT(DISTINCT c.id)
                 FROM cards c
                 JOIN dictionary_entries d ON c.entry_id = d.id
                 LEFT JOIN card_tags ct ON ct.card_id = c.id
                 LEFT JOIN tags t ON t.id = ct.tag_id
                 WHERE (c.reps > 0 OR c.state > 0)
                   AND (
                     LOWER(t.name) = LOWER(?1)
                     OR LOWER(t.name) LIKE '%' || LOWER(?1) || '%'
                     OR EXISTS (
                        SELECT 1 FROM kanjis k 
                        WHERE k.jlpt_level = ?2 AND d.kanji LIKE '%' || k.character || '%'
                     )
                   )",
                params![name, level],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let kanji_percentage = if total_kanji > 0 {
            ((learned_kanji as f64 / total_kanji as f64) * 100.0).min(100.0)
        } else {
            0.0
        };

        let vocab_percentage = if default_vocab_total > 0 {
            ((learned_vocab as f64 / default_vocab_total as f64) * 100.0).min(100.0)
        } else {
            0.0
        };

        let overall_percentage = ((kanji_percentage * 0.5) + (vocab_percentage * 0.5)).min(100.0);

        total_learned_kanji_all += learned_kanji;
        total_learned_vocab_all += learned_vocab;

        levels.push(crate::models::JlptLevelCoverage {
            level,
            level_name: name.to_string(),
            total_kanji,
            learned_kanji,
            kanji_percentage: (kanji_percentage * 10.0).round() / 10.0,
            total_vocab: default_vocab_total,
            learned_vocab,
            vocab_percentage: (vocab_percentage * 10.0).round() / 10.0,
            overall_percentage: (overall_percentage * 10.0).round() / 10.0,
        });
    }

    Ok(crate::models::JlptAnalyticsStats {
        levels,
        total_kanji_learned: total_learned_kanji_all,
        total_vocab_learned: total_learned_vocab_all,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_date_aggregation_and_forecast_query() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_tables_and_seed(&conn).unwrap();

        // 2026-10-15 12:00:00 UTC -> 1792065600
        let timestamp_day1: i64 = 1792065600;
        // 2026-10-16 12:00:00 UTC -> 1792152000
        let timestamp_day2: i64 = 1792152000;

        conn.execute(
            "INSERT INTO cards (id, entry_id, due_date, stability, difficulty, reps, state) 
             VALUES (?1, 1, ?2, 2.0, 0.3, 1, 2)",
            params!["c1", timestamp_day1],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO cards (id, entry_id, due_date, stability, difficulty, reps, state) 
             VALUES (?1, 2, ?2, 2.0, 0.3, 1, 2)",
            params!["c2", timestamp_day1],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO cards (id, entry_id, due_date, stability, difficulty, reps, state) 
             VALUES (?1, 3, ?2, 2.0, 0.3, 1, 2)",
            params!["c3", timestamp_day2],
        )
        .unwrap();

        // Query grouped by DATE(datetime(due_date, 'unixepoch', 'localtime'))
        let mut stmt = conn
            .prepare(
                "SELECT 
                    DATE(datetime(due_date, 'unixepoch', 'localtime')) as forecast_date, 
                    COUNT(*) as count 
                 FROM cards 
                 GROUP BY forecast_date 
                 ORDER BY forecast_date ASC",
            )
            .unwrap();

        let results: Vec<(String, i32)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].1, 2); // 2 cards on day 1
        assert_eq!(results[1].1, 1); // 1 card on day 2
    }

    #[test]
    fn test_study_history_query() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_tables_and_seed(&conn).unwrap();

        // Insert card first for foreign key
        conn.execute(
            "INSERT INTO cards (id, entry_id, due_date, stability, difficulty, reps, state) 
             VALUES ('c1', 1, 1000, 2.0, 0.3, 1, 2)",
            [],
        )
        .unwrap();

        let review_time: i64 = 1792065600; // 2026-10-15
        conn.execute(
            "INSERT INTO review_logs (card_id, rating, review_datetime) VALUES ('c1', 3, ?1)",
            params![review_time],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO review_logs (card_id, rating, review_datetime) VALUES ('c1', 4, ?1)",
            params![review_time],
        )
        .unwrap();

        let date_ymd: String = conn
            .query_row(
                "SELECT DATE(datetime(?1, 'unixepoch', 'localtime'))",
                params![review_time],
                |row| row.get(0),
            )
            .unwrap();

        let month_prefix = &date_ymd[0..7]; // YYYY-MM

        let mut stmt = conn
            .prepare(
                "SELECT 
                    DATE(datetime(review_datetime, 'unixepoch', 'localtime')) as history_date, 
                    COUNT(*) as count 
                 FROM review_logs 
                 WHERE strftime('%Y-%m', datetime(review_datetime, 'unixepoch', 'localtime')) = ?1
                 GROUP BY history_date",
            )
            .unwrap();

        let rows: Vec<(String, i32)> = stmt
            .query_map(params![month_prefix], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, date_ymd);
        assert_eq!(rows[0].1, 2);
    }

    #[test]
    fn test_jlpt_coverage_query() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_tables_and_seed(&conn).unwrap();

        // Insert N5 kanjis
        conn.execute(
            "INSERT INTO kanjis (character, meaning, jlpt_level) VALUES ('日', 'sun/day', 5)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO kanjis (character, meaning, jlpt_level) VALUES ('本', 'book/origin', 5)",
            [],
        )
        .unwrap();

        // Insert dictionary entry containing '日' and '本'
        conn.execute(
            "INSERT INTO dictionary_entries (id, kanji, reading, meaning) VALUES (10, '日本', 'にほん', 'Japan')",
            [],
        )
        .unwrap();

        // Insert learned card
        conn.execute(
            "INSERT INTO cards (id, entry_id, due_date, stability, difficulty, reps, state) 
             VALUES ('card_jp', 10, 1000, 2.5, 0.3, 3, 2)",
            [],
        )
        .unwrap();

        let count_n5_kanji: i32 = conn
            .query_row(
                "SELECT COUNT(DISTINCT k.character) 
                 FROM kanjis k
                 JOIN dictionary_entries d ON (d.kanji LIKE '%' || k.character || '%')
                 JOIN cards c ON c.entry_id = d.id
                 WHERE k.jlpt_level = 5 AND (c.reps > 0 OR c.state > 0)",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count_n5_kanji, 2);
    }
}
