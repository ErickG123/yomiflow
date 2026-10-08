use crate::AppState;
use rusqlite::params;
use serde_json::Value;
use std::fs;
use std::path::Path;
use tauri::{Emitter, State, Window};

fn extract_jlpt_from_filename(name: &str) -> Option<i32> {
    let lower = name.to_lowercase();
    for i in 1..=5 {
        if lower.contains(&format!("n{}", i)) || lower.contains(&format!("jlpt-{}", i)) {
            return Some(i);
        }
    }
    None
}

#[tauri::command]
pub async fn import_dictionary(
    folder_path: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<crate::models::ImportResult, String> {
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    let path = Path::new(&folder_path);

    if !path.exists() || !path.is_dir() {
        return Err("Provided path is not a valid directory".to_string());
    }

    let entries = fs::read_dir(path).map_err(|e| e.to_string())?;
    let mut result = crate::models::ImportResult::default();

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_path = entry.path();

        if file_path.is_file()
            && file_path
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        {
            let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let _ = window.emit("backend-progress", format!("Importando: {}...", file_name));

            // Tag bank files (tag_bank_*.json)
            if file_name.starts_with("tag_bank")
                && let Ok(content) = fs::read_to_string(&file_path)
                && let Ok(tags) = crate::importer::parse_tag_bank(&content)
            {
                let count = tags.len();
                crate::db::bulk_insert_tags(&mut conn, tags).map_err(|e| e.to_string())?;
                result.tags_imported += count;
            }
            // Kanji bank files (kanji_bank_*.json)
            else if file_name.starts_with("kanji_bank")
                && let Ok(content) = fs::read_to_string(&file_path)
                && let Ok(kanjis) = crate::importer::parse_kanji_bank(&content)
            {
                let count = kanjis.len();
                crate::db::bulk_insert_kanjis(&mut conn, kanjis).map_err(|e| e.to_string())?;
                result.kanjis_imported += count;
            }
            // Yomichan term banks usually named term_bank_*.json
            else if (file_name.starts_with("term_bank") || file_name.ends_with(".json"))
                && let Ok(content) = fs::read_to_string(&file_path)
                && let Ok(json) = serde_json::from_str::<Value>(&content)
                && let Some(items) = json.as_array()
            {
                let tx = conn.transaction().map_err(|e| e.to_string())?;
                {
                    let mut stmt = tx
                        .prepare(
                            "INSERT INTO dictionary_entries (kanji, reading, meaning) 
                             VALUES (?1, ?2, ?3)",
                        )
                        .map_err(|e| e.to_string())?;

                    for item in items {
                        if let Some(arr) = item.as_array()
                            && !arr.is_empty()
                        {
                            let term = arr.first().and_then(|v| v.as_str()).unwrap_or("");
                            let reading = arr.get(1).and_then(|v| v.as_str()).unwrap_or(term);

                            let meaning = if let Some(defs) = arr.get(5).and_then(|v| v.as_array())
                            {
                                defs.iter()
                                    .filter_map(|d| {
                                        d.as_str()
                                            .or_else(|| d.get("content").and_then(|c| c.as_str()))
                                    })
                                    .collect::<Vec<_>>()
                                    .join("; ")
                            } else {
                                arr.get(5)
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string()
                            };

                            if !term.is_empty() || !reading.is_empty() {
                                stmt.execute(params![
                                    if term == reading {
                                        None::<String>
                                    } else {
                                        Some(term.to_string())
                                    },
                                    reading,
                                    meaning
                                ])
                                .map_err(|e| e.to_string())?;
                                result.terms_imported += 1;
                            }
                        }
                    }
                }
                tx.commit().map_err(|e| e.to_string())?;
            }
        }
    }

    result.total_imported = result.terms_imported + result.kanjis_imported + result.tags_imported;

    Ok(result)
}

#[tauri::command]
pub async fn import_jlpt_kanjis(
    path: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let target_path = Path::new(&path);
    if !target_path.exists() {
        return Err("Provided path does not exist".to_string());
    }

    let mut total_imported = 0;
    let mut files_to_process = Vec::new();

    if target_path.is_file() {
        if target_path
            .extension()
            .map(|ext| ext == "json")
            .unwrap_or(false)
        {
            files_to_process.push(target_path.to_path_buf());
        }
    } else if target_path.is_dir() {
        let dir_entries = fs::read_dir(target_path).map_err(|e| e.to_string())?;
        for entry in dir_entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let p = entry.path();
            if p.is_file() && p.extension().map(|ext| ext == "json").unwrap_or(false) {
                files_to_process.push(p);
            }
        }
    }

    if files_to_process.is_empty() {
        return Err("No JSON files found at the specified path".to_string());
    }

    for file in files_to_process {
        let file_name = file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let _ = window.emit(
            "backend-progress",
            format!("A processar Kanji: {}...", file_name),
        );

        let content = fs::read_to_string(&file).map_err(|e| e.to_string())?;
        let fallback_jlpt = extract_jlpt_from_filename(&file_name);

        let kanjis = crate::importer::parse_jlpt_kanjis(&content, fallback_jlpt)?;
        let count = kanjis.len();

        if count > 0 {
            let mut conn = state.db.lock().map_err(|e| e.to_string())?;
            crate::db::bulk_insert_kanjis(&mut conn, kanjis).map_err(|e| e.to_string())?;
            total_imported += count;
        }
    }

    let _ = window.emit(
        "backend-progress",
        format!(
            "Importação de Kanjis concluída: {} registos.",
            total_imported
        ),
    );

    Ok(total_imported)
}
