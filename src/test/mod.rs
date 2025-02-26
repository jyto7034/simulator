use std::io::Read;

use rand::{seq::SliceRandom, thread_rng};
use serde_json::{json, Value};
use tokio::sync::Mutex;

use crate::{
    card::Card,
    card_gen::CardGenerator,
    enums::{DeckCode, CARD_JSON_PATH, MAX_CARD_SIZE},
    server::{
        end_point::handle_mulligan_cards,
        types::{ServerState, SessionKey},
    },
    utils::{json, parse_json_to_deck_code},
};

pub fn initialize_app(p1_deck: DeckCode, p2_deck: DeckCode, attacker: usize) -> crate::app::App {
    let mut app = crate::app::App::instantiate();

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

use actix_web::{
    dev::ServerHandle,
    web::{self, Data},
    App,
};

pub fn create_server_state() -> web::Data<ServerState> {
    let (deck_json, _original_cards) = generate_random_deck_json();
    let (deck_json2, _) = generate_random_deck_json();

    let deck_codes = parse_json_to_deck_code(Some(deck_json), Some(deck_json2))
        .expect("Failed to parse deck code");

    let app = initialize_app(deck_codes.0, deck_codes.1, 0);

    web::Data::new(ServerState {
        game: Mutex::new(app.game),
        player_cookie: SessionKey("player1".to_string()),
        opponent_cookie: SessionKey("player2".to_string()),
    })
}

use actix_web::HttpServer;
use std::net::{SocketAddr, TcpListener};

pub async fn spawn_server() -> (SocketAddr, Data<ServerState>, ServerHandle) {
    let server_state = create_server_state();
    let server_state_clone = server_state.clone();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(server_state.clone())
            .service(handle_mulligan_cards)
    })
    .listen(listener)
    .unwrap()
    .run();

    let handle = server.handle();
    tokio::spawn(server);

    (addr, server_state_clone, handle)
}
