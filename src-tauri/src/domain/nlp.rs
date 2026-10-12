use crate::models::{BunsetsuChunk, MorphologicalToken};
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
        let dictionary = load_dictionary("embedded://ipadic").map_err(|e| {
            format!(
                "Falha ao carregar dicionário IPADIC embutido do Lindera: {}",
                e
            )
        })?;
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

pub fn extract_lexical_tokens(text: &str) -> Result<Vec<crate::models::LexicalToken>, String> {
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
        let mut cursor = 0;
        let mut result = Vec::new();
        for token in tokens.iter_mut() {
            let surface = token.surface.to_string();
            if surface.is_empty() {
                continue;
            }
            let Some(relative_start) = text[cursor..].find(&surface) else {
                continue;
            };
            let start_byte = cursor + relative_start;
            let end_byte = start_byte + surface.len();
            cursor = end_byte;

            let details = token.details();
            let pos = details.first().copied().unwrap_or("");
            if pos.starts_with("記号") || surface.chars().all(char::is_whitespace) {
                continue;
            }
            let lemma = details.get(6).copied().unwrap_or("*");
            if lemma.is_empty() || lemma == "*" {
                continue;
            }
            let reading = details
                .get(7)
                .copied()
                .filter(|value| !value.is_empty() && *value != "*")
                .map(katakana_to_hiragana)
                .unwrap_or_default();
            result.push(crate::models::LexicalToken {
                surface,
                lemma: lemma.to_string(),
                reading,
                start_byte,
                end_byte,
            });
        }
        Ok(result)
    })
}

pub fn katakana_to_hiragana(katakana: &str) -> String {
    let mut hiragana = String::new();
    for c in katakana.chars() {
        let code = c as u32;
        // Katakana range 0x30A1..=0x30F6 maps to Hiragana 0x3041..=0x3096 (diff 0x60)
        if (0x30A1..=0x30F6).contains(&code)
            && let Some(h_char) = char::from_u32(code - 0x60)
        {
            hiragana.push(h_char);
            continue;
        }
        hiragana.push(c);
    }
    hiragana
}

fn contains_kanji(text: &str) -> bool {
    text.chars()
        .any(|c| matches!(c as u32, 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF | 0x20000..=0x3134F))
}

fn escape_html_text(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

/// Generates HTML ruby annotations from Lindera readings. All token text is escaped
/// because the Angular caller intentionally trusts this generated HTML.
pub fn generate_furigana(text: &str) -> Result<String, String> {
    if text.trim().is_empty() {
        return Ok(escape_html_text(text));
    }

    let tokens = segment_and_tag_japanese(text)?;
    let mut output = String::with_capacity(text.len());
    for token in tokens {
        let surface = escape_html_text(&token.surface);
        let Some(reading) = token.reading.filter(|reading| !reading.trim().is_empty()) else {
            output.push_str(&surface);
            continue;
        };

        let reading = katakana_to_hiragana(&reading);
        if contains_kanji(&token.surface) && reading != token.surface {
            output.push_str("<ruby>");
            output.push_str(&surface);
            output.push_str("<rt>");
            output.push_str(&escape_html_text(&reading));
            output.push_str("</rt></ruby>");
        } else {
            output.push_str(&surface);
        }
    }
    Ok(output)
}

/// Injects <ruby>SURFACE<rt>HIRAGANA</rt></ruby> dynamically into HTML/text,
/// skipping words that exist in the user's SRS vocabulary set.
pub fn inject_furigana_for_unlearned_words(
    text: &str,
    known_srs_words: &std::collections::HashSet<String>,
) -> Result<String, String> {
    if text.trim().is_empty() {
        return Ok(text.to_string());
    }

    let tokens = segment_and_tag_japanese(text)?;
    let mut output = String::new();

    for token in tokens {
        let surface = &token.surface;

        // Only inject furigana if the token contains Kanji AND is not yet in the user's SRS deck
        if contains_kanji(surface)
            && !known_srs_words.contains(surface)
            && let Some(ref raw_reading) = token.reading
        {
            let reading_hiragana = katakana_to_hiragana(raw_reading);
            if !reading_hiragana.is_empty() && reading_hiragana != *surface {
                output.push_str(&format!(
                    "<ruby>{}<rt>{}</rt></ruby>",
                    surface, reading_hiragana
                ));
                continue;
            }
        }

        output.push_str(surface);
    }

    Ok(output)
}

/// Groups tokens into Bunsetsu chunks and establishes a left-branching dependency tree
pub fn generate_syntax_tree(text: &str) -> Result<Vec<BunsetsuChunk>, String> {
    let tokens = segment_and_tag_japanese(text)?;
    if tokens.is_empty() {
        return Ok(Vec::new());
    }

    // 1. Group tokens into Bunsetsu (chunks)
    // A Bunsetsu starts with an independent word (Content word: noun, verb, adjective, etc.)
    // and absorbs following dependent words (particles, auxiliaries).
    let mut bunsetsu_raw: Vec<Vec<MorphologicalToken>> = Vec::new();
    let mut current_chunk: Vec<MorphologicalToken> = Vec::new();

    for token in tokens {
        let is_content_word = matches!(token.pos.as_str(), "noun" | "verb" | "adjective");
        let is_auxiliary = token.pos == "auxiliary";

        if !current_chunk.is_empty() {
            let has_content = current_chunk
                .iter()
                .any(|t| matches!(t.pos.as_str(), "noun" | "verb" | "adjective"));
            let has_particle = current_chunk.iter().any(|t| t.pos == "particle");

            if is_content_word && (has_content || has_particle) {
                bunsetsu_raw.push(current_chunk);
                current_chunk = Vec::new();
            } else if is_auxiliary && has_content {
                // E.g. 学生 (noun) followed by です (auxiliary): separate as distinct chunk
                // unless it is attaching directly to a verb/adjective stem
                let last_is_noun = current_chunk
                    .last()
                    .map(|t| t.pos == "noun")
                    .unwrap_or(false);
                if last_is_noun {
                    bunsetsu_raw.push(current_chunk);
                    current_chunk = Vec::new();
                }
            }
        }

        current_chunk.push(token);
    }

    if !current_chunk.is_empty() {
        bunsetsu_raw.push(current_chunk);
    }

    if bunsetsu_raw.is_empty() {
        return Ok(Vec::new());
    }

    // 2. Build preliminary chunks with combined POS tags
    struct TempChunk {
        id: usize,
        text: String,
        pos: String,
        last_particle: Option<String>,
        primary_pos: String,
    }

    let mut temp_chunks: Vec<TempChunk> = Vec::new();

    for (idx, chunk_tokens) in bunsetsu_raw.into_iter().enumerate() {
        let id = idx + 1;
        let mut text = String::new();
        let mut pos_parts = Vec::new();
        let mut last_particle = None;
        let mut primary_pos = "other".to_string();

        for t in &chunk_tokens {
            text.push_str(&t.surface);
            if !pos_parts.contains(&t.pos) {
                pos_parts.push(t.pos.clone());
            }
            if t.pos == "particle" {
                last_particle = Some(t.surface.clone());
            }
            if matches!(t.pos.as_str(), "verb" | "noun" | "adjective") && primary_pos == "other" {
                primary_pos = t.pos.clone();
            }
        }

        if primary_pos == "other" && !chunk_tokens.is_empty() {
            primary_pos = chunk_tokens[0].pos.clone();
        }

        let combined_pos = if pos_parts.len() > 1 {
            pos_parts.join("+")
        } else {
            pos_parts
                .first()
                .cloned()
                .unwrap_or_else(|| "other".to_string())
        };

        temp_chunks.push(TempChunk {
            id,
            text,
            pos: combined_pos,
            last_particle,
            primary_pos,
        });
    }

    let chunk_count = temp_chunks.len();
    let mut final_chunks = Vec::new();

    // 3. Dependency heuristic (Left-Branching / Japanese SOV syntax):
    // - The last chunk (root, typically predicate/verb/copula) has target_id: None.
    // - A chunk ending with particle 'の' points to the immediate next noun chunk (or next chunk).
    // - Chunks with topic/case markers 'は', 'が', 'を', 'に', 'で', 'へ', 'と', 'から', 'まで' point to the main predicate (last chunk, or next verb).
    // - General default for non-terminal chunks: points to the final predicate or the closest downstream verb/chunk.
    for i in 0..chunk_count {
        let current = &temp_chunks[i];
        if i == chunk_count - 1 {
            final_chunks.push(BunsetsuChunk {
                id: current.id,
                text: current.text.clone(),
                pos: current.pos.clone(),
                target_id: None,
            });
            continue;
        }

        let mut target_id = Some(temp_chunks[chunk_count - 1].id);

        if let Some(ref p) = current.last_particle {
            if p == "の" {
                if i + 1 < chunk_count {
                    target_id = Some(temp_chunks[i + 1].id);
                }
            } else if matches!(
                p.as_str(),
                "は" | "が" | "を" | "に" | "で" | "へ" | "と" | "から" | "まで"
            ) {
                let next_verb = temp_chunks[i + 1..]
                    .iter()
                    .find(|c| c.primary_pos == "verb" || c.pos.contains("verb"));
                if let Some(verb_chunk) = next_verb {
                    target_id = Some(verb_chunk.id);
                } else {
                    target_id = Some(temp_chunks[chunk_count - 1].id);
                }
            }
        } else if (current.primary_pos == "noun" || current.primary_pos == "adjective")
            && i + 1 < chunk_count
        {
            target_id = Some(temp_chunks[i + 1].id);
        }

        final_chunks.push(BunsetsuChunk {
            id: current.id,
            text: current.text.clone(),
            pos: current.pos.clone(),
            target_id,
        });
    }

    Ok(final_chunks)
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

    #[test]
    fn test_katakana_to_hiragana() {
        assert_eq!(katakana_to_hiragana("サクラ"), "さくら");
        assert_eq!(katakana_to_hiragana("ニホン"), "にほん");
        assert_eq!(katakana_to_hiragana("漢字"), "漢字");
    }

    #[test]
    fn test_generate_furigana_for_kanji_and_kana() {
        let result = generate_furigana("私は学生です").unwrap();
        assert_eq!(
            result,
            "<ruby>私<rt>わたし</rt></ruby>は<ruby>学生<rt>がくせい</rt></ruby>です"
        );

        assert_eq!(
            generate_furigana("ひらがなカタカナ").unwrap(),
            "ひらがなカタカナ"
        );
    }

    #[test]
    fn test_generate_furigana_escapes_untrusted_html() {
        assert_eq!(generate_furigana("<script>").unwrap(), "&lt;script&gt;");
    }

    #[test]
    fn test_inject_furigana_skips_known_srs_words() {
        let mut known = std::collections::HashSet::new();
        known.insert("学生".to_string());

        let res = inject_furigana_for_unlearned_words("私は学生です", &known).unwrap();
        // 私 is unknown kanji -> has ruby
        assert!(res.contains("<ruby>私<rt>わたし</rt></ruby>"));
        // 学生 is in known SRS words -> no ruby!
        assert!(res.contains("学生"));
        assert!(!res.contains("<ruby>学生"));
    }

    #[test]
    fn test_generate_syntax_tree_watashi_wa_gakusei_desu() {
        let tree = generate_syntax_tree("私は学生です").unwrap();
        assert_eq!(tree.len(), 3);

        // 1: 私は (noun+particle) -> points to です (id 3)
        assert_eq!(tree[0].id, 1);
        assert_eq!(tree[0].text, "私は");
        assert_eq!(tree[0].pos, "noun+particle");
        assert_eq!(tree[0].target_id, Some(3));

        // 2: 学生 (noun) -> points to です (id 3)
        assert_eq!(tree[1].id, 2);
        assert_eq!(tree[1].text, "学生");
        assert_eq!(tree[1].pos, "noun");
        assert_eq!(tree[1].target_id, Some(3));

        // 3: です (auxiliary) -> root (None)
        assert_eq!(tree[2].id, 3);
        assert_eq!(tree[2].text, "です");
        assert_eq!(tree[2].target_id, None);
    }

    #[test]
    fn test_generate_syntax_tree_empty() {
        let tree = generate_syntax_tree("").unwrap();
        assert!(tree.is_empty());
    }
}
