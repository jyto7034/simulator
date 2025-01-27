
use card_game::card::Card;
use card_game::card_gen::CardGenerator;
use card_game::{app::App, utils::*, enums::*};
use serde_json::{json, Value};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::io::Read;

const CARD_NUM: usize = 25;

pub fn initialize_app(p1_deck: DeckCode, p2_deck: DeckCode, attacker: usize) -> App {
    let mut app = App::instantiate();

    app.initialize_game(p1_deck, p2_deck, attacker)
        .expect("app initialize failed");
    app
}

fn generate_random_deck_json() -> (Value, Vec<Card>) {
    // 카드 JSON 파일 로드
    let file_path = CARD_JSON_PATH;
    let mut file = std::fs::File::open(file_path).expect("Failed to open cards.json");
    let mut json_data = String::new();
    file.read_to_string(&mut json_data).expect("Failed to read file");
    
    let cards: Vec<json::CardJson> = serde_json::from_str(&json_data).expect("Failed to parse JSON");
    
    let mut rng = thread_rng();
    let selected_cards: Vec<json::CardJson> = cards
        .into_iter() 
        .filter(|card| card.collectible == Some(true))
        .collect::<Vec<_>>()
        .choose_multiple(&mut rng, CARD_NUM)
        .cloned()
        .collect();

    
    // 선택된 카드로 덱 JSON 생성
    let deck_json = json!({
        "decks": [{
            "Hero": [{
                "name": "player1"
            }],
            "cards": selected_cards.iter().map(|card| {
                json!({
                    "id": card.id.clone(),
                    "num": 1
                })
            }).collect::<Vec<_>>()
        }]
    });

    // 원본 카드 정보 저장
    let card_generator = CardGenerator::new();
    let original_cards: Vec<Card> = selected_cards
        .iter()
        .map(|card| card_generator.gen_card_by_id_string(card.id.clone().unwrap(), card, 0))
        .collect();
    (deck_json, original_cards)
}


#[cfg(test)]
mod tests {
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
        let cards_vec = deckcode_to_cards(deck_codes.0, deck_codes.1)
            .expect("Failed to load card data");

        // 4. 결과 검증
        let p1_cards = &cards_vec[0];
        // for item in p1_cards{
        //     if !original_cards.contains(item) {
        //         panic!("deck encode/dedcode error");
        //     }
        // }

        // 카드 수 검증
        assert_eq!(p1_cards.len(), CARD_NUM, "Deck should have {CARD_NUM} cards");
        assert_eq!(original_cards.len(), CARD_NUM, "Original deck should have {CARD_NUM} cards");
    }

    // #[test]
    // fn test_muligun(){
        
    // }
}