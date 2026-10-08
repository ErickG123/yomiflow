use crate::models::{JlptKanjiEntry, KanjiEntry, TagEntry};
use serde_json::Value;

pub fn parse_tag_bank(content: &str) -> Result<Vec<TagEntry>, String> {
    let json: Value = serde_json::from_str(content).map_err(|e| e.to_string())?;
    let items = json.as_array().ok_or("Invalid JSON array in tag bank")?;

    let mut tags = Vec::new();

    for item in items {
        if let Some(arr) = item.as_array() {
            // Yomichan tag bank format:
            // [name (String), category (String), order (Number), notes (String), score (Number), ...]
            if !arr.is_empty() {
                let name = arr.first().and_then(|v| v.as_str()).unwrap_or("").trim();
                if name.is_empty() {
                    continue;
                }

                let category = arr.get(1).and_then(|v| v.as_str()).map(|s| s.to_string());
                let notes = arr.get(3).and_then(|v| v.as_str()).map(|s| s.to_string());

                tags.push(TagEntry {
                    id: name.to_string(),
                    category,
                    notes,
                });
            }
        }
    }

    Ok(tags)
}

pub fn parse_kanji_bank(content: &str) -> Result<Vec<KanjiEntry>, String> {
    let json: Value = serde_json::from_str(content).map_err(|e| e.to_string())?;
    let items = json.as_array().ok_or("Invalid JSON array in kanji bank")?;

    let mut kanjis = Vec::new();

    for item in items {
        if let Some(arr) = item.as_array() {
            // Yomichan kanji bank format:
            // [character (String), onyomi (String), kunyomi (String), tags (String), meanings (Array of Strings), stats/misc]
            if !arr.is_empty() {
                let character = arr.first().and_then(|v| v.as_str()).unwrap_or("").trim();
                if character.is_empty() {
                    continue;
                }

                let onyomi = arr
                    .get(1)
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.to_string());

                let kunyomi = arr
                    .get(2)
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.to_string());

                let meaning = if let Some(meanings_arr) = arr.get(4).and_then(|v| v.as_array()) {
                    meanings_arr
                        .iter()
                        .filter_map(|m| m.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                } else {
                    arr.get(4)
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string()
                };

                kanjis.push(KanjiEntry {
                    character: character.to_string(),
                    onyomi,
                    kunyomi,
                    meaning,
                    jlpt_level: None,
                });
            }
        }
    }

    Ok(kanjis)
}

/// Helper to extract string or array of strings into a comma-separated list
fn extract_comma_separated(value: &Option<Value>) -> Option<String> {
    match value {
        Some(Value::String(s)) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        Some(Value::Array(arr)) => {
            let list: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.trim()))
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            if list.is_empty() {
                None
            } else {
                Some(list.join(", "))
            }
        }
        _ => None,
    }
}

/// Parse JLPT level from an entry or filename fallback
fn parse_jlpt_level(raw_level: &Option<Value>, fallback: Option<i32>) -> Option<i32> {
    if let Some(val) = raw_level {
        match val {
            Value::Number(n) => {
                if let Some(num) = n.as_i64() {
                    return Some(num as i32);
                }
            }
            Value::String(s) => {
                let cleaned = s.trim().to_uppercase();
                if let Some(stripped) = cleaned.strip_prefix('N') {
                    if let Ok(lvl) = stripped.parse::<i32>() {
                        return Some(lvl);
                    }
                } else if let Ok(lvl) = cleaned.parse::<i32>() {
                    return Some(lvl);
                }
            }
            _ => {}
        }
    }
    fallback
}

/// Parses a JSON content string from AnchorI/jlpt-kanji-dictionary format.
/// Supports both array-of-objects (`jlpt-kanji.json`, `jlpt-n5.json`) and object-keyed formats.
pub fn parse_jlpt_kanjis(
    content: &str,
    fallback_jlpt: Option<i32>,
) -> Result<Vec<KanjiEntry>, String> {
    let raw_json: Value = serde_json::from_str(content).map_err(|e| e.to_string())?;
    let mut results = Vec::new();

    let entries: Vec<Value> = match raw_json {
        Value::Array(arr) => arr,
        Value::Object(map) => map.into_values().collect(),
        _ => return Err("Invalid JSON: root must be an array or object".to_string()),
    };

    for item in entries {
        // Attempt to deserialize or inspect object fields
        if let Ok(jlpt_entry) = serde_json::from_value::<JlptKanjiEntry>(item.clone()) {
            let character = jlpt_entry
                .kanji
                .or(jlpt_entry.character)
                .or(jlpt_entry.utf)
                .unwrap_or_default()
                .trim()
                .to_string();

            if character.is_empty() {
                continue;
            }

            let jlpt_level = jlpt_entry
                .jlpt_level
                .or_else(|| parse_jlpt_level(&jlpt_entry.jlpt, fallback_jlpt));

            // Extract Onyomi
            let onyomi = extract_comma_separated(&jlpt_entry.onyomi)
                .or_else(|| jlpt_entry.on_readings.map(|v| v.join(", ")));

            // Extract Kunyomi
            let kunyomi = extract_comma_separated(&jlpt_entry.kunyomi)
                .or_else(|| jlpt_entry.kun_readings.map(|v| v.join(", ")));

            // Extract Meaning
            let mut meaning = extract_comma_separated(&jlpt_entry.meaning)
                .or_else(|| jlpt_entry.meanings.map(|v| v.join(", ")))
                .unwrap_or_default();

            // If meaning is empty, check description field
            if meaning.is_empty()
                && let Some(desc) = jlpt_entry.description
            {
                if let Some(idx) = desc.find("that means ") {
                    let after = &desc[idx + "that means ".len()..];
                    if let Some(dot_idx) = after.find('.') {
                        meaning = after[..dot_idx].trim().to_string();
                    } else {
                        meaning = after.trim().to_string();
                    }
                } else {
                    meaning = desc.trim().to_string();
                }
            }

            results.push(KanjiEntry {
                character,
                onyomi,
                kunyomi,
                meaning,
                jlpt_level,
            });
        }
    }

    Ok(results)
}
