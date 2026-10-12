use crate::models::{LexicalToken, ReadabilityAnalysis, UnknownWordOccurrence};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct RetentionRecord {
    pub state: i32,
    pub stability: f64,
    pub repetitions: i32,
    pub last_review: i64,
}

pub type RetentionIndex = HashMap<(String, String), Vec<RetentionRecord>>;

pub fn calculate_retention(record: RetentionRecord, now: i64) -> f64 {
    if record.state == 0 || record.repetitions <= 0 || record.stability <= 0.0 {
        return 0.0;
    }
    let elapsed_days = (now.saturating_sub(record.last_review) as f64 / 86_400.0).max(0.1);
    let retrievability = (1.0 + 19.0 * elapsed_days / (9.0 * record.stability)).powf(-0.5);
    match record.state {
        1 => retrievability * 0.65,
        2 => retrievability,
        _ => 0.0,
    }
}

pub fn analyze(
    text: &str,
    tokens: &[LexicalToken],
    retention_index: &RetentionIndex,
    now: i64,
) -> ReadabilityAnalysis {
    let mut unknown_words = Vec::new();
    let mut known_words = 0;
    let mut total_retention = 0.0;

    for token in tokens {
        let records = retention_index
            .get(&(token.lemma.clone(), token.reading.clone()))
            .map(Vec::as_slice)
            .unwrap_or_default();
        let best_retention = records
            .iter()
            .map(|record| calculate_retention(*record, now))
            .fold(0.0_f64, f64::max);
        let is_known = !records.is_empty();
        total_retention += best_retention;

        if is_known {
            known_words += 1;
        } else {
            unknown_words.push(UnknownWordOccurrence {
                surface: token.surface.clone(),
                lemma: token.lemma.clone(),
                reading: token.reading.clone(),
                start: text[..token.start_byte].encode_utf16().count(),
                end: text[..token.end_byte].encode_utf16().count(),
                retention_percentage: 0.0,
            });
        }
    }

    let total_words = tokens.len();
    ReadabilityAnalysis {
        total_words,
        known_words,
        comprehension_percentage: if total_words == 0 {
            0.0
        } else {
            (known_words as f64 * 1000.0 / total_words as f64).round() / 10.0
        },
        average_retention_percentage: if known_words == 0 {
            0.0
        } else {
            (total_retention * 1000.0 / known_words as f64).round() / 10.0
        },
        unknown_words,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(surface: &str, start_byte: usize, end_byte: usize) -> LexicalToken {
        LexicalToken {
            surface: surface.to_owned(),
            lemma: surface.to_owned(),
            reading: String::new(),
            start_byte,
            end_byte,
        }
    }

    #[test]
    fn empty_input_returns_zeroed_metrics() {
        let result = analyze("", &[], &RetentionIndex::new(), 0);
        assert_eq!(result.total_words, 0);
        assert_eq!(result.known_words, 0);
        assert_eq!(result.comprehension_percentage, 0.0);
        assert_eq!(result.average_retention_percentage, 0.0);
    }

    #[test]
    fn comprehension_counts_known_lemmas_and_preserves_utf16_offsets() {
        let text = "😀猫未知";
        let tokens = vec![token("猫", 4, 7), token("未知", 7, 13)];
        let known_key = ("猫".to_owned(), String::new());
        let mut index = RetentionIndex::new();
        index.insert(
            known_key,
            vec![RetentionRecord {
                state: 2,
                stability: 10.0,
                repetitions: 4,
                last_review: 100,
            }],
        );

        let result = analyze(text, &tokens, &index, 100);

        assert_eq!(result.comprehension_percentage, 50.0);
        assert_eq!(result.unknown_words.len(), 1);
        assert_eq!(result.unknown_words[0].start, 3);
        assert_eq!(result.unknown_words[0].end, 5);
    }

    #[test]
    fn new_cards_have_zero_retention() {
        let record = RetentionRecord {
            state: 0,
            stability: 0.0,
            repetitions: 0,
            last_review: 0,
        };
        assert_eq!(calculate_retention(record, 1_000), 0.0);
    }
}
