use card_game::card::Card;
use card_game::card_gen::CardGenerator;
use card_game::{app::App, enums::*, utils::*};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde_json::{json, Value};
use std::io::Read;

const CARD_NUM: usize = 25;

#[cfg(test)]
mod utils_test {
    use card_game::test::generate_random_deck_json;

    use super::*;

    #[test]
    fn test_deck_encode_decode_with_load() {
        // 1. 랜덤 덱 생성
        let (deck_json, original_cards) = generate_random_deck_json();
        let (deck_json2, _) = generate_random_deck_json();

        // 2. JSON을 덱 코드로 변환
        let deck_codes = parse_json_to_deck_code(Some(deck_json), Some(deck_json2))
            .expect("Failed to parse deck code");

        // 3. 덱 코드를 Cards로 변환
        let cards_vec =
            deckcode_to_cards(deck_codes.0, deck_codes.1).expect("Failed to load card data");

        // 4. 결과 검증
        let p1_cards = &cards_vec[0];
        for item in &p1_cards.v_card {
            if !original_cards.contains(item) {
                panic!("deck encode/dedcode error");
            }
        }

        // 카드 수 검증
        assert_eq!(
            p1_cards.len(),
            CARD_NUM,
            "Deck should have {CARD_NUM} cards"
        );
        assert_eq!(
            original_cards.len(),
            CARD_NUM,
            "Original deck should have {CARD_NUM} cards"
        );
    }
}

#[cfg(test)]
mod game_test {
    use super::*;

    #[test]
    fn asd() {}
}
