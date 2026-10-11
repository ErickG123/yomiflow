use crate::AppState;
use crate::models::{KanjiComponent, KanjiDetails};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn get_kanji_details(
    kanji: String,
    state: State<'_, AppState>,
) -> Result<KanjiDetails, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // 1. Fetch components
    let mut stmt = conn
        .prepare("SELECT kanji, component, meaning FROM kanji_components WHERE kanji = ?1")
        .map_err(|e| e.to_string())?;

    let components = stmt
        .query_map(params![kanji], |row| {
            Ok(KanjiComponent {
                kanji: row.get(0)?,
                component: row.get(1)?,
                meaning: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<KanjiComponent>, rusqlite::Error>>()
        .map_err(|e| e.to_string())?;

    // 2. Fetch user mnemonic
    let mut m_stmt = conn
        .prepare("SELECT mnemonic FROM user_mnemonics WHERE kanji = ?1 LIMIT 1")
        .map_err(|e| e.to_string())?;

    let mut rows = m_stmt.query(params![kanji]).map_err(|e| e.to_string())?;
    let mnemonic = if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        row.get(0).map_err(|e| e.to_string())?
    } else {
        None
    };

    Ok(KanjiDetails {
        kanji,
        components,
        mnemonic,
    })
}

#[tauri::command]
pub fn save_kanji_mnemonic(
    kanji: String,
    mnemonic: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO user_mnemonics (kanji, mnemonic) VALUES (?1, ?2)",
        params![kanji, mnemonic],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("failed to open in-memory db");
        conn.execute_batch(
            "
            CREATE TABLE kanji_components (
                kanji TEXT NOT NULL,
                component TEXT NOT NULL,
                meaning TEXT NOT NULL
            );
            CREATE TABLE user_mnemonics (
                kanji TEXT PRIMARY KEY,
                mnemonic TEXT NOT NULL
            );
            ",
        )
        .expect("failed to create tables");
        conn
    }

    #[test]
    fn test_kanji_components_and_mnemonic_persistence() {
        let conn = setup_test_db();

        // 1. Insert components
        conn.execute(
            "INSERT INTO kanji_components (kanji, component, meaning) VALUES (?1, ?2, ?3)",
            params!["森", "木", "árvore"],
        )
        .unwrap();

        // 2. Insert and update mnemonic (UPSERT)
        conn.execute(
            "INSERT OR REPLACE INTO user_mnemonics (kanji, mnemonic) VALUES (?1, ?2)",
            params!["森", "Três árvores formam uma floresta densa."],
        )
        .unwrap();

        // 3. Query details directly
        let mut comp_stmt = conn
            .prepare("SELECT kanji, component, meaning FROM kanji_components WHERE kanji = ?1")
            .unwrap();

        let comps: Vec<KanjiComponent> = comp_stmt
            .query_map(params!["森"], |row| {
                Ok(KanjiComponent {
                    kanji: row.get(0)?,
                    component: row.get(1)?,
                    meaning: row.get(2)?,
                })
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].component, "木");
        assert_eq!(comps[0].meaning, "árvore");

        let mnemonic: Option<String> = conn
            .query_row(
                "SELECT mnemonic FROM user_mnemonics WHERE kanji = ?1",
                params!["森"],
                |row| row.get(0),
            )
            .ok();

        assert_eq!(
            mnemonic,
            Some("Três árvores formam uma floresta densa.".to_string())
        );

        // 4. Update mnemonic
        conn.execute(
            "INSERT OR REPLACE INTO user_mnemonics (kanji, mnemonic) VALUES (?1, ?2)",
            params!["森", "História atualizada."],
        )
        .unwrap();

        let updated_mnemonic: String = conn
            .query_row(
                "SELECT mnemonic FROM user_mnemonics WHERE kanji = ?1",
                params!["森"],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(updated_mnemonic, "História atualizada.");
    }
}
