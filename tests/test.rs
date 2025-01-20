
use card_game::card::Card;
use card_game::card_gen::CardGenerator;
use card_game::{app::App, game::Game, procedure::Procedure, OptRcRef, utils::*, enums::*};
use serde_json::{json, Value};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::io::Read;

pub fn initialize_app(p1_deck: String, p2_deck: String, attacker: usize) -> App {
    let mut app = App {
        game: Game {
            player1: OptRcRef::none(),
            player2: OptRcRef::none(),
        },
        procedure: Procedure {
            tasks: vec![],
            trigger_tasks: vec![],
        },
    };

    app.initialize_game(p1_deck, p2_deck, attacker)
        .expect("app initialize failed");
    app
}

fn generate_random_deck() -> (Value, Vec<Card>) {
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
        .choose_multiple(&mut rng, 30)
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

    println!("{}", selected_cards.len());
    
    // 원본 카드 정보 저장
    let card_generator = CardGenerator::new();
    let original_cards: Vec<Card> = selected_cards
        .iter()
        .map(|card| card_generator.gen_card_by_id_string(card.id.clone().unwrap(), card, 1))
        .collect();
    
    (deck_json, original_cards)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deck_encode_decode_with_load() {
        // 1. 랜덤 덱 생성
        let (deck_json, original_cards) = generate_random_deck();
        let (deck_json2, _) = generate_random_deck();
        
        println!("{}", original_cards.len());
        
        // 2. JSON을 덱 코드로 변환
        let deck_codes = parse_json_to_deck_code(Some(deck_json), Some(deck_json2))
            .expect("Failed to parse deck code");
        
        // 3. 덱 코드를 Cards로 변환
        let cards_vec = load_card_data(deck_codes)
            .expect("Failed to load card data");
        
        // 4. 결과 검증
        let p1_cards = &cards_vec[0];
        
        // 카드 수 검증
        assert_eq!(p1_cards.len(), 30, "Deck should have 30 cards");
        assert_eq!(original_cards.len(), 30, "Original deck should have 30 cards");
        
        // 카드 내용 검증
        // let loaded_cards = p1_cards.v_card;
        // for (orig, loaded) in original_cards.iter().zip(loaded_cards.iter()) {
        //     assert_eq!(orig.get_id(), loaded.get_id(), 
        //         "Card ID mismatch: original={}, loaded={}", 
        //         orig.get_id(), loaded.get_id());
            
        //     assert_eq!(orig.get_name(), loaded.get_name(), 
        //         "Card name mismatch for card ID {}: original={}, loaded={}", 
        //         orig.get_id(), orig.get_name(), loaded.get_name());
            
        //     // 필요한 경우 다른 속성들도 검증
        //     // assert_eq!(orig.get_cost(), loaded.get_cost(), "Cost mismatch");
        //     // assert_eq!(orig.get_attack(), loaded.get_attack(), "Attack mismatch");
        //     // assert_eq!(orig.get_health(), loaded.get_health(), "Health mismatch");
        // }
        
        // // 디버깅을 위한 출력
        // println!("Successfully verified {} cards", original_cards.len());
        // println!("First few cards comparison:");
        // for i in 0..std::cmp::min(5, original_cards.len()) {
        //     println!("Original: {:?}, Loaded: {:?}", 
        //         original_cards[i].get_name(),
        //         loaded_cards[i].get_name());
        // }
    }

    // #[test]
    // fn test_deck_conversion() {
    //     // 랜덤 덱 생성
    //     let (deck_json, original_cards) = generate_random_deck();
    //     let (deck_json2, _) = generate_random_deck();
        
    //     // 덱 코드로 변환
    //     let (p1_deck, p2_deck) = parse_json_to_deck_code(Some(deck_json), Some(deck_json2))
    //         .expect("Failed to parse deck code");
        
    //     // 앱 초기화
    //     let app = initialize_app(p1_deck, p2_deck, PLAYER_1);
        
    //     // 변환된 카드 가져오기
    //     let converted_cards = app
    //         .game
    //         .get_player(PlayerType::Player1)
    //         .get()
    //         .get_cards()
    //         .clone();
        
    //     // // 카드 비교
    //     // assert_eq!(original_cards.len(), converted_cards.len(), "Card count mismatch");
        
    //     // // 각 카드의 속성 비교
    //     // for (orig, conv) in original_cards.iter().zip(converted_cards.v_card) {
    //     //     assert_eq!(orig.get_name(), conv.get_name(), "Card Name mismatch");
    //     //     // 필요한 다른 속성들도 비교 가능
    //     // }
    // }
}