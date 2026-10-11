use crate::models::MorphologicalToken;
use lindera::dictionary::load_dictionary;
use lindera::mode::Mode;
use lindera::segmenter::Segmenter;
use std::borrow::Cow;
use std::sync::Mutex;

static SEGMENTER: Mutex<Option<Segmenter>> = Mutex::new(None);

pub const MAX_JAPANESE_TEXT_BYTES: usize = 50_000; // Limit payload to prevent CPU/memory exhaustion

fn get_or_create_segmenter<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&mut Segmenter) -> Result<R, String>,
{
    let mut lock = SEGMENTER
        .lock()
        .map_err(|e| format!("Falha no mutex do segmenter: {}", e))?;

    if lock.is_none() {
        let dictionary = load_dictionary("embedded://ipadic")
            .map_err(|e| format!("Falha ao carregar dicionário IPADIC embutido do Lindera: {}", e))?;
        *lock = Some(Segmenter::new(Mode::Normal, dictionary, None));
    }

    let segmenter = lock.as_mut().unwrap();
    f(segmenter)
}

pub fn map_pos_category(pos_str: &str) -> String {
    if pos_str.starts_with("動詞") {
        "verb".to_string()
    } else if pos_str.starts_with("助詞") {
        "particle".to_string()
    } else if pos_str.starts_with("名詞") {
        "noun".to_string()
    } else if pos_str.starts_with("形容詞") {
        "adjective".to_string()
    } else if pos_str.starts_with("助動詞") {
        "auxiliary".to_string()
    } else {
        "other".to_string()
    }
}

/// Pure domain morphological segmentation and categorization logic using Lindera
pub fn segment_and_tag_japanese(text: &str) -> Result<Vec<MorphologicalToken>, String> {
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }

    if text.len() > MAX_JAPANESE_TEXT_BYTES {
        return Err(format!(
            "Payload de texto excede o limite máximo permitido de {} bytes",
            MAX_JAPANESE_TEXT_BYTES
        ));
    }

    get_or_create_segmenter(|segmenter| {
        let mut tokens = segmenter
            .segment(Cow::Borrowed(text))
            .map_err(|e| format!("Erro durante análise morfológica Lindera: {}", e))?;

        let mut result = Vec::new();
        for token in tokens.iter_mut() {
            let surface = token.surface.to_string();
            let mut reading: Option<String> = None;
            let mut pos = "other".to_string();

            let details = token.details();
            if let Some(pos_val) = details.first() {
                pos = map_pos_category(pos_val);
            }

            // In IPADIC, reading in Katakana is usually at index 7
            if details.len() > 7 && !details[7].trim().is_empty() && details[7] != "*" {
                reading = Some(details[7].to_string());
            }

            result.push(MorphologicalToken {
                surface,
                reading,
                pos,
            });
        }

        Ok(result)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_nlp_empty_and_whitespace() {
        let res = segment_and_tag_japanese("").unwrap();
        assert!(res.is_empty());

        let res_ws = segment_and_tag_japanese("   \n\t  ").unwrap();
        assert!(res_ws.is_empty());
    }

    #[test]
    fn test_domain_nlp_exceeding_max_payload() {
        let large_text = "あ".repeat(MAX_JAPANESE_TEXT_BYTES + 10);
        let err = segment_and_tag_japanese(&large_text);
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("excede o limite"));
    }

    #[test]
    fn test_domain_nlp_pos_tagging_and_readings() {
        let tokens = segment_and_tag_japanese("私は学生です").unwrap();
        assert_eq!(tokens.len(), 4);

        assert_eq!(tokens[0].surface, "私");
        assert_eq!(tokens[0].pos, "noun");

        assert_eq!(tokens[1].surface, "は");
        assert_eq!(tokens[1].pos, "particle");

        assert_eq!(tokens[2].surface, "学生");
        assert_eq!(tokens[2].pos, "noun");

        assert_eq!(tokens[3].surface, "です");
        assert_eq!(tokens[3].pos, "auxiliary");
    }

    #[test]
    fn test_domain_nlp_special_chars_and_punctuation_no_panic() {
        let complex = "「鬼滅の刃」：１２３！？@#$^&*()_+{}[]\\|;:'\",.<>/?\n\r\t";
        let res = segment_and_tag_japanese(complex);
        assert!(res.is_ok());
    }
}
