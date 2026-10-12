use crate::models::InterleavedReviewCard;

/// Interleaves active study items while preserving a deterministic seedable shuffle.
pub fn interleave_by_type(
    items: Vec<InterleavedReviewCard>,
    mut state: u64,
) -> Vec<InterleavedReviewCard> {
    if state == 0 {
        state = 0x9E37_79B9_7F4A_7C15;
    }
    let mut groups: [Vec<InterleavedReviewCard>; 4] = std::array::from_fn(|_| Vec::new());
    for item in items {
        let group = match item {
            InterleavedReviewCard::Vocabulary { .. } => 0,
            InterleavedReviewCard::ImageOcclusion { .. } => 1,
            InterleavedReviewCard::Grammar { .. } => 2,
            InterleavedReviewCard::Kana { .. } => 3,
        };
        groups[group].push(item);
    }
    for group in &mut groups {
        shuffle(group, &mut state);
    }

    let mut output = Vec::new();
    let mut cursor = (state as usize) % groups.len();
    while groups.iter().any(|group| !group.is_empty()) {
        for offset in 0..groups.len() {
            let group = (cursor + offset) % groups.len();
            if let Some(item) = groups[group].pop() {
                output.push(item);
                cursor = (group + 1) % groups.len();
                break;
            }
        }
    }
    output
}

fn shuffle<T>(items: &mut [T], state: &mut u64) {
    for index in (1..items.len()).rev() {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        items.swap(index, (*state as usize) % (index + 1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind(card: &InterleavedReviewCard) -> u8 {
        match card {
            InterleavedReviewCard::Vocabulary { .. } => 0,
            InterleavedReviewCard::ImageOcclusion { .. } => 1,
            InterleavedReviewCard::Grammar { .. } => 2,
            InterleavedReviewCard::Kana { .. } => 3,
        }
    }

    #[test]
    fn balances_card_types_and_preserves_all_items() {
        let cards = vec![
            InterleavedReviewCard::Vocabulary {
                card_id: "v1".into(),
                deck_id: "d".into(),
                kanji: None,
                reading: "r".into(),
                meaning: "m".into(),
                state: 0,
                pitch: None,
            },
            InterleavedReviewCard::Vocabulary {
                card_id: "v2".into(),
                deck_id: "d".into(),
                kanji: None,
                reading: "r".into(),
                meaning: "m".into(),
                state: 0,
                pitch: None,
            },
            InterleavedReviewCard::ImageOcclusion {
                card_id: "i1".into(),
                deck_id: "d".into(),
            },
            InterleavedReviewCard::Grammar {
                card_id: "g1".into(),
                deck_id: "d".into(),
                title: "t".into(),
                content_md: "c".into(),
            },
            InterleavedReviewCard::Kana {
                character: "k".into(),
                romaji: "k".into(),
                alphabet: "gojuon".into(),
                category: "base".into(),
            },
        ];
        let result = interleave_by_type(cards, 42);
        assert_eq!(result.len(), 5);
        assert!(
            result
                .windows(2)
                .all(|window| kind(&window[0]) != kind(&window[1]))
        );
    }
}
