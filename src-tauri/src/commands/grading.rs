use crate::AppState;
use crate::models::StrokeCoordinate;
use rusqlite::params;
use tauri::State;

/// Evaluates drawn strokes against KanjiVG SVG data stored in SQLite.
/// Returns a score from 0.0 to 1.0 based on:
/// 1. Stroke count and order agreement.
/// 2. Vector direction matching of each stroke (start -> end angle).
/// 3. Spatial bounding box / aspect ratio sanity.
#[tauri::command]
pub async fn evaluate_kanji_drawing(
    kanji: String,
    user_strokes: Vec<Vec<StrokeCoordinate>>,
    state: State<'_, AppState>,
) -> Result<f32, String> {
    if user_strokes.is_empty() {
        return Ok(0.0);
    }

    // 1. Fetch reference KanjiVG SVG from SQLite
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT svg_data FROM kanji_strokes WHERE kanji = ?1 LIMIT 1")
        .map_err(|e| e.to_string())?;

    let mut rows = stmt.query(params![kanji]).map_err(|e| e.to_string())?;
    let svg_data: Option<String> = if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        Some(row.get(0).map_err(|e| e.to_string())?)
    } else {
        None
    };

    // If no KanjiVG SVG exists yet in DB, evaluate basic geometric coherence
    let Some(svg) = svg_data else {
        return Ok(evaluate_fallback_stroke_coherence(&user_strokes));
    };

    // 2. Extract reference path directions and count from KanjiVG SVG
    let ref_paths = extract_svg_paths(&svg);
    if ref_paths.is_empty() {
        return Ok(evaluate_fallback_stroke_coherence(&user_strokes));
    }

    let ref_count = ref_paths.len();
    let user_count = user_strokes.len();

    // Score 1: Stroke Count Penalty (max score 0.35)
    let count_diff = (ref_count as i32 - user_count as i32).abs();
    let count_score = if count_diff == 0 {
        0.35
    } else if count_diff == 1 {
        0.20
    } else if count_diff == 2 {
        0.10
    } else {
        0.0
    };

    // Score 2: Directional vector comparison (max score 0.45)
    let min_strokes = user_count.min(ref_count);
    let mut direction_score_sum = 0.0f32;

    for i in 0..min_strokes {
        let user_stroke = &user_strokes[i];
        if user_stroke.len() < 2 {
            continue;
        }

        let u_start = &user_stroke[0];
        let u_end = &user_stroke[user_stroke.len() - 1];
        let u_dx = u_end.x - u_start.x;
        let u_dy = u_end.y - u_start.y;
        let u_mag = (u_dx * u_dx + u_dy * u_dy).sqrt();

        let ref_dir = &ref_paths[i];
        let r_dx = ref_dir.dx;
        let r_dy = ref_dir.dy;
        let r_mag = (r_dx * r_dx + r_dy * r_dy).sqrt();

        if u_mag > 1.0 && r_mag > 0.001 {
            // Dot product cosine similarity
            let cos_sim = (u_dx * r_dx + u_dy * r_dy) / (u_mag * r_mag);
            // Map [-1, 1] to [0, 1]
            let stroke_sim = ((cos_sim + 1.0) / 2.0).clamp(0.0, 1.0);
            direction_score_sum += stroke_sim;
        }
    }

    let avg_direction_score = if min_strokes > 0 {
        (direction_score_sum / min_strokes as f32) * 0.45
    } else {
        0.0
    };

    // Score 3: Spatial Distribution / Canvas Usage (max score 0.20)
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for stroke in &user_strokes {
        for pt in stroke {
            if pt.x < min_x { min_x = pt.x; }
            if pt.x > max_x { max_x = pt.x; }
            if pt.y < min_y { min_y = pt.y; }
            if pt.y > max_y { max_y = pt.y; }
        }
    }

    let width = (max_x - min_x).max(0.0);
    let height = (max_y - min_y).max(0.0);

    // Kanji pad is 256x256. A well-drawn kanji covers between 50px and 240px
    let spatial_score = if width > 40.0 && height > 40.0 && width < 250.0 && height < 250.0 {
        0.20
    } else if width > 20.0 && height > 20.0 {
        0.10
    } else {
        0.05
    };

    let total_score = (count_score + avg_direction_score + spatial_score).clamp(0.0, 1.0);
    Ok(total_score)
}

struct RefPathDirection {
    dx: f32,
    dy: f32,
}

/// Parses KanjiVG SVG 'd="M..."' path commands to extract approximate stroke direction vectors.
fn extract_svg_paths(svg: &str) -> Vec<RefPathDirection> {
    let mut paths = Vec::new();
    let mut cursor = 0;

    while let Some(d_idx) = svg[cursor..].find("d=\"") {
        let abs_start = cursor + d_idx + 3;
        if let Some(end_quote) = svg[abs_start..].find('"') {
            let path_d = &svg[abs_start..abs_start + end_quote];
            if let Some(dir) = parse_svg_path_direction(path_d) {
                paths.push(dir);
            }
            cursor = abs_start + end_quote;
        } else {
            break;
        }
    }

    paths
}

fn parse_svg_path_direction(d: &str) -> Option<RefPathDirection> {
    // Collect numbers in path data
    let mut numbers = Vec::new();
    let mut current_num = String::new();

    for c in d.chars() {
        if c.is_ascii_digit() || c == '.' || c == '-' {
            current_num.push(c);
        } else {
            if !current_num.is_empty() && current_num != "-" {
                if let Ok(val) = current_num.parse::<f32>() {
                    numbers.push(val);
                }
                current_num.clear();
            }
        }
    }
    if !current_num.is_empty() && current_num != "-"
        && let Ok(val) = current_num.parse::<f32>() {
        numbers.push(val);
    }

    if numbers.len() >= 4 {
        let x1 = numbers[0];
        let y1 = numbers[1];
        let x2 = numbers[numbers.len() - 2];
        let y2 = numbers[numbers.len() - 1];
        Some(RefPathDirection {
            dx: x2 - x1,
            dy: y2 - y1,
        })
    } else {
        None
    }
}

fn evaluate_fallback_stroke_coherence(user_strokes: &[Vec<StrokeCoordinate>]) -> f32 {
    let stroke_count = user_strokes.len();
    if stroke_count == 0 {
        return 0.0;
    }

    let mut valid_strokes = 0;
    for stroke in user_strokes {
        if stroke.len() >= 2 {
            let start = &stroke[0];
            let end = &stroke[stroke.len() - 1];
            let dist = ((end.x - start.x).powi(2) + (end.y - start.y).powi(2)).sqrt();
            if dist > 8.0 {
                valid_strokes += 1;
            }
        }
    }

    let ratio = valid_strokes as f32 / stroke_count as f32;
    (0.60 + ratio * 0.30).clamp(0.0, 0.90)
}
