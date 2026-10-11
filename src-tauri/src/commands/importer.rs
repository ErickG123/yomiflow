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

#[tauri::command]
pub async fn import_kanjivg_folder(
    folder_path: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let db_mutex = state.db.clone();

    let total_inserted = tokio::task::spawn_blocking(move || -> Result<usize, String> {
        let path = Path::new(&folder_path);
        if !path.exists() || !path.is_dir() {
            return Err("O caminho fornecido não é uma pasta válida.".to_string());
        }

        let entries = fs::read_dir(path).map_err(|e| e.to_string())?;
        let mut count = 0usize;

        let mut conn = db_mutex.lock().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        {
            let mut stmt = tx
                .prepare("INSERT OR REPLACE INTO kanji_strokes (kanji, svg_data) VALUES (?1, ?2)")
                .map_err(|e| e.to_string())?;

            for entry in entries {
                let entry = entry.map_err(|e| e.to_string())?;
                let file_path = entry.path();

                if file_path.is_file()
                    && file_path
                        .extension()
                        .map(|ext| ext == "svg")
                        .unwrap_or(false)
                {
                    let file_name = file_path.file_stem().and_then(|n| n.to_str()).unwrap_or("");

                    // Some KanjiVG files have variant suffixes like '05bdd-Kaisho', take the hex part
                    let hex_part = file_name.split('-').next().unwrap_or(file_name);

                    if let Ok(code_point) = u32::from_str_radix(hex_part, 16)
                        && let Some(ch) = std::char::from_u32(code_point)
                        && let Ok(svg_content) = fs::read_to_string(&file_path)
                    {
                        stmt.execute(params![ch.to_string(), svg_content])
                            .map_err(|e| e.to_string())?;
                        count += 1;
                    }
                }
            }
        }

        tx.commit().map_err(|e| e.to_string())?;
        Ok(count)
    })
    .await
    .map_err(|e| format!("Falha na execução da tarefa em segundo plano: {}", e))??;

    let _ = window.emit(
        "backend-progress",
        format!(
            "Importação KanjiVG concluída com sucesso: {} ficheiros SVG importados.",
            total_inserted
        ),
    );

    Ok(total_inserted)
}

#[tauri::command]
pub async fn get_kanji_svg(
    kanji: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT svg_data FROM kanji_strokes WHERE kanji = ?1 LIMIT 1")
        .map_err(|e| e.to_string())?;

    let mut rows = stmt.query(params![kanji]).map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let svg: String = row.get(0).map_err(|e| e.to_string())?;
        Ok(Some(svg))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn import_frequency_list(
    file_path: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let path = Path::new(&file_path);
    if !path.exists() || !path.is_file() {
        return Err("O caminho fornecido não é um ficheiro válido.".to_string());
    }

    let _ = window.emit("backend-progress", "Lendo ficheiro de frequência...");
    let content = fs::read_to_string(path).map_err(|e| format!("Erro ao ler ficheiro: {}", e))?;

    let mut items: Vec<(String, i64)> = Vec::new();

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "json" {
        // Support:
        // 1. Array of pairs: [["word", 1], ["word2", 2]]
        // 2. Array of objects: [{"term": "word", "rank": 1}]
        // 3. Object map: {"word": 1, "word2": 2}
        let parsed: Value =
            serde_json::from_str(&content).map_err(|e| format!("Formato JSON inválido: {}", e))?;

        match parsed {
            Value::Array(arr) => {
                for item in arr {
                    if let Value::Array(pair) = item {
                        if pair.len() >= 2
                            && let (Some(term), Some(rank)) = (
                                pair[0].as_str(),
                                pair[1]
                                    .as_i64()
                                    .or_else(|| pair[1].as_str().and_then(|s| s.parse().ok())),
                            )
                        {
                            items.push((term.trim().to_string(), rank));
                        }
                    } else if let Value::Object(obj) = item {
                        let term_val = obj
                            .get("term")
                            .or_else(|| obj.get("word"))
                            .and_then(|v| v.as_str());
                        let rank_val =
                            obj.get("rank")
                                .or_else(|| obj.get("ranking"))
                                .and_then(|v| {
                                    v.as_i64()
                                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                                });

                        if let (Some(term), Some(rank)) = (term_val, rank_val) {
                            items.push((term.trim().to_string(), rank));
                        }
                    }
                }
            }
            Value::Object(map) => {
                for (term, rank_val) in map {
                    if let Some(rank) = rank_val
                        .as_i64()
                        .or_else(|| rank_val.as_str().and_then(|s| s.parse().ok()))
                    {
                        items.push((term.trim().to_string(), rank));
                    }
                }
            }
            _ => return Err("Estrutura JSON não suportada para frequências.".to_string()),
        }
    } else {
        // CSV / TSV fallback (ex: word,rank or word\trank)
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let separator = if line.contains('\t') { '\t' } else { ',' };
            let parts: Vec<&str> = line.split(separator).collect();
            if parts.len() >= 2 {
                let term = parts[0].trim().trim_matches('"');
                if let Ok(rank) = parts[1].trim().trim_matches('"').parse::<i64>() {
                    items.push((term.to_string(), rank));
                }
            }
        }
    }

    if items.is_empty() {
        return Err("Nenhum dado de frequência válido encontrado no ficheiro.".to_string());
    }

    let _ = window.emit(
        "backend-progress",
        format!("Gravando {} frequências no SQLite...", items.len()),
    );

    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    {
        let mut stmt = tx
            .prepare("INSERT OR REPLACE INTO word_frequencies (term, rank) VALUES (?1, ?2)")
            .map_err(|e| e.to_string())?;

        for (term, rank) in &items {
            stmt.execute(params![term, rank])
                .map_err(|e| e.to_string())?;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;

    let total = items.len();
    let _ = window.emit(
        "backend-progress",
        format!(
            "Importação concluída: {} palavras indexadas com ranking.",
            total
        ),
    );

    Ok(total)
}
