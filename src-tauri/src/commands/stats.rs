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
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT 
                DATE(datetime(due_date, 'unixepoch', 'localtime')) as forecast_date, 
                COUNT(*) as count 
             FROM cards 
             WHERE DATE(datetime(due_date, 'unixepoch', 'localtime')) BETWEEN ?1 AND ?2
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
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let prefix = format!("{:04}-{:02}", year, month);

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
        ).unwrap();

        conn.execute(
            "INSERT INTO cards (id, entry_id, due_date, stability, difficulty, reps, state) 
             VALUES (?1, 2, ?2, 2.0, 0.3, 1, 2)",
            params!["c2", timestamp_day1],
        ).unwrap();

        conn.execute(
            "INSERT INTO cards (id, entry_id, due_date, stability, difficulty, reps, state) 
             VALUES (?1, 3, ?2, 2.0, 0.3, 1, 2)",
            params!["c3", timestamp_day2],
        ).unwrap();

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
        ).unwrap();

        let review_time: i64 = 1792065600; // 2026-10-15
        conn.execute(
            "INSERT INTO review_logs (card_id, rating, review_datetime) VALUES ('c1', 3, ?1)",
            params![review_time],
        ).unwrap();
        conn.execute(
            "INSERT INTO review_logs (card_id, rating, review_datetime) VALUES ('c1', 4, ?1)",
            params![review_time],
        ).unwrap();

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
}
