use std::io::Read;

use rand::{seq::SliceRandom, thread_rng};
use serde_json::{json, Value};

use crate::{
    app::App,
    card::Card,
    card_gen::CardGenerator,
    enums::{DeckCode, CARD_JSON_PATH, MAX_CARD_SIZE},
    utils::json,
};

pub fn initialize_app(p1_deck: DeckCode, p2_deck: DeckCode, attacker: usize) -> App {
    let mut app = App::instantiate();

    app.initialize_game(p1_deck, p2_deck, attacker)
        .expect("app initialize failed");
    app
}

pub fn generate_random_deck_json() -> (Value, Vec<Card>) {
    // 카드 JSON 파일 로드
    let file_path = CARD_JSON_PATH;
    let mut file = std::fs::File::open(file_path).expect("Failed to open cards.json");
    let mut json_data = String::new();
    file.read_to_string(&mut json_data)
        .expect("Failed to read file");

    let cards: Vec<json::CardJson> =
        serde_json::from_str(&json_data).expect("Failed to parse JSON");

    let mut rng = thread_rng();
    let selected_cards: Vec<json::CardJson> = cards
        .into_iter()
        .filter(|card| card.collectible == Some(true))
        .collect::<Vec<_>>()
        .choose_multiple(&mut rng, MAX_CARD_SIZE)
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
