#[derive(Clone, Copy)]
struct StrokeDirection {
    dx: f32,
    dy: f32,
}

/// Scores user strokes against optional KanjiVG path data without Tauri or database access.
pub fn evaluate_kanji_drawing(svg: Option<&str>, user_strokes: &[Vec<(f32, f32)>]) -> f32 {
    if user_strokes.is_empty() {
        return 0.0;
    }

    let Some(svg) = svg else {
        return evaluate_fallback_stroke_coherence(user_strokes);
    };
    let reference = extract_svg_paths(svg);
    if reference.is_empty() {
        return evaluate_fallback_stroke_coherence(user_strokes);
    }

    let count_diff = (reference.len() as i32 - user_strokes.len() as i32).abs();
    let count_score = match count_diff {
        0 => 0.35,
        1 => 0.20,
        2 => 0.10,
        _ => 0.0,
    };

    let compared_count = user_strokes.len().min(reference.len());
    let direction_sum = user_strokes
        .iter()
        .zip(reference.iter())
        .take(compared_count)
        .filter_map(|(stroke, reference)| {
            let &(start_x, start_y) = stroke.first()?;
            let &(end_x, end_y) = stroke.last()?;
            if stroke.len() < 2 {
                return None;
            }
            let dx = end_x - start_x;
            let dy = end_y - start_y;
            let magnitude = (dx * dx + dy * dy).sqrt();
            let reference_magnitude =
                (reference.dx * reference.dx + reference.dy * reference.dy).sqrt();
            if magnitude <= 1.0 || reference_magnitude <= 0.001 {
                return None;
            }
            let similarity =
                ((dx * reference.dx + dy * reference.dy) / (magnitude * reference_magnitude) + 1.0)
                    / 2.0;
            Some(similarity.clamp(0.0, 1.0))
        })
        .sum::<f32>();
    let direction_score = if compared_count > 0 {
        direction_sum / compared_count as f32 * 0.45
    } else {
        0.0
    };

    let Some((min_x, max_x, min_y, max_y)) = stroke_bounds(user_strokes) else {
        return count_score + direction_score;
    };
    let width = (max_x - min_x).max(0.0);
    let height = (max_y - min_y).max(0.0);
    let spatial_score = if width > 40.0 && height > 40.0 && width < 250.0 && height < 250.0 {
        0.20
    } else if width > 20.0 && height > 20.0 {
        0.10
    } else {
        0.05
    };

    (count_score + direction_score + spatial_score).clamp(0.0, 1.0)
}

fn stroke_bounds(strokes: &[Vec<(f32, f32)>]) -> Option<(f32, f32, f32, f32)> {
    let mut points = strokes.iter().flatten();
    let &(first_x, first_y) = points.next()?;
    Some(points.fold(
        (first_x, first_x, first_y, first_y),
        |(min_x, max_x, min_y, max_y), &(x, y)| {
            (min_x.min(x), max_x.max(x), min_y.min(y), max_y.max(y))
        },
    ))
}

fn extract_svg_paths(svg: &str) -> Vec<StrokeDirection> {
    let mut paths = Vec::new();
    let mut cursor = 0;
    while let Some(index) = svg[cursor..].find("d=\"") {
        let start = cursor + index + 3;
        let Some(end) = svg[start..].find('"') else {
            break;
        };
        if let Some(direction) = parse_svg_path_direction(&svg[start..start + end]) {
            paths.push(direction);
        }
        cursor = start + end + 1;
    }
    paths
}

fn parse_svg_path_direction(path: &str) -> Option<StrokeDirection> {
    let numbers = path
        .split(|character: char| {
            !(character.is_ascii_digit() || character == '.' || character == '-')
        })
        .filter_map(|number| number.parse::<f32>().ok())
        .collect::<Vec<_>>();
    if numbers.len() < 4 {
        return None;
    }
    Some(StrokeDirection {
        dx: numbers[numbers.len() - 2] - numbers[0],
        dy: numbers[numbers.len() - 1] - numbers[1],
    })
}

fn evaluate_fallback_stroke_coherence(strokes: &[Vec<(f32, f32)>]) -> f32 {
    if strokes.is_empty() {
        return 0.0;
    }
    let valid_strokes = strokes
        .iter()
        .filter(|stroke| {
            if stroke.len() < 2 {
                return false;
            }
            let (start_x, start_y) = stroke[0];
            let (end_x, end_y) = stroke[stroke.len() - 1];
            ((end_x - start_x).powi(2) + (end_y - start_y).powi(2)).sqrt() > 8.0
        })
        .count();
    (0.60 + valid_strokes as f32 / strokes.len() as f32 * 0.30).clamp(0.0, 0.90)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_strokes_have_zero_score() {
        assert_eq!(evaluate_kanji_drawing(None, &[]), 0.0);
        assert_eq!(evaluate_kanji_drawing(Some("<svg/>"), &[]), 0.0);
    }

    #[test]
    fn malformed_or_missing_reference_uses_safe_fallback() {
        let strokes = vec![vec![(0.0, 0.0), (30.0, 30.0)]];
        let score = evaluate_kanji_drawing(Some("<svg><path d='bad'/></svg>"), &strokes);
        assert!((0.60..=0.90).contains(&score));
        assert_eq!(score, evaluate_kanji_drawing(None, &strokes));
    }

    #[test]
    fn matching_svg_direction_and_spread_score_above_fallback() {
        let svg = r#"<svg><path d="M 0 0 L 100 100"/></svg>"#;
        let strokes = vec![vec![(50.0, 50.0), (150.0, 150.0)]];
        let actual = evaluate_kanji_drawing(Some(svg), &strokes);
        let fallback = evaluate_kanji_drawing(None, &strokes);
        assert!(actual > fallback);
        assert!(actual <= 1.0);
    }

    #[test]
    fn malformed_path_values_do_not_panic() {
        assert!(parse_svg_path_direction("M - . L NaN").is_none());
        assert!(extract_svg_paths("<svg><path d=\"unfinished").is_empty());
    }
}
