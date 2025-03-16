use std::{io::Read, time::Duration};

use async_tungstenite::{
    tokio::{connect_async, TokioAdapter},
    tungstenite::{self, http::Request, Message},
    WebSocketStream,
};
use futures_util::StreamExt;
use rand::{seq::SliceRandom, thread_rng};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tokio::{net::TcpStream, sync::Mutex};
use uuid::Uuid;

use crate::{
    card::{cards::CardVecExt, types::PlayerType, Card},
    card_gen::CardGenerator,
    enums::{DeckCode, CARD_JSON_PATH, MAX_CARD_SIZE, TIMEOUT},
    server::{
        end_point::{handle_draw, handle_mulligan},
        jsons::{draw, mulligan, ErrorMessage},
        session::PlayerSessionManager,
        types::{ServerState, SessionKey},
    },
    utils::{json, parse_json_to_deck_code},
    zone::zone::Zone,
    VecStringExt,
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
        session_manager: PlayerSessionManager::new(TIMEOUT),
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
            .service(handle_mulligan)
            .service(handle_draw)
    })
    .listen(listener)
    .unwrap()
    .run();

    let handle = server.handle();
    tokio::spawn(server);

    (addr, server_state_clone, handle)
}

/// ws_stream 에서 Deal 메시지를 기다리고 파싱하여 카드 리스트를 반환합니다.
pub async fn expect_mulligan_deal_message(
    ws_stream: &mut async_tungstenite::WebSocketStream<
        async_tungstenite::tokio::TokioAdapter<tokio::net::TcpStream>,
    >,
) -> Vec<Uuid> {
    let timeout = tokio::time::timeout(Duration::from_secs(5),
async{
    loop{
        if let Some(msg) = ws_stream.next().await{
            match msg{
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<mulligan::ServerMessage>(&text) {
                        Ok(mulligan::ServerMessage::Deal(data)) => {
                            return data.cards
                        }
                        Ok(other) => panic!(
                            "Expected a MulliganMessage::Deal message, but received a different variant: {:?}",
                            other
                        ),
                        Err(e) => panic!("Failed to parse the deal message JSON: {:?}", e),
                    }
                },
                Ok(Message::Ping(_)) => {
                    ws_stream.send(Message::Pong(vec![])).await.ok();
                    continue;
                },
                Ok(_) => continue,
                Err(_) => panic!("WebSocket error"),
            }
        }
    }
}).await;
    match timeout {
        Ok(cards) => cards.to_vec_uuid(),
        Err(_) => panic!(
            "Did not receive any message from the server while expecting MulliganMessage::Deal."
        ),
    }
}

/// ws_stream 에서 Complete 메시지를 기다리고 파싱하여 카드 리스트를 반환합니다.
pub async fn expect_mulligan_complete_message(
    ws_stream: &mut async_tungstenite::WebSocketStream<
        async_tungstenite::tokio::TokioAdapter<tokio::net::TcpStream>,
    >,
) -> Vec<Uuid> {
    // 타임아웃 설정
    let timeout = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        async {
            // 원하는 메시지가 올 때까지 계속 메시지를 받습니다
            loop {
                if let Some(msg) = ws_stream.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            match serde_json::from_str::<mulligan::ClientMessage>(&text) {
                                Ok(mulligan::ClientMessage::Complete(data)) => {
                                    return data.cards
                                }
                                Ok(other) => panic!(
                                    "Expected a MulliganMessage::Complete message, but received a different variant: {:?}",
                                    other
                                ),
                                Err(e) => panic!("Failed to parse the reroll answer JSON: {:?}", e),
                            }
                        }
                        // ping, pong, binary 등 다른 메시지 타입은 무시
                        Ok(Message::Ping(_)) => {
                            // ping 메시지에 자동으로 pong 응답
                            ws_stream.send(Message::Pong(vec![])).await.ok();
                            continue;
                        }
                        Ok(_) => continue,
                        Err(_) => panic!("WebSocket error"),
                    }
                }
            }
        }
    ).await;

    match timeout {
        Ok(cards) => cards.to_vec_uuid(),
        Err(_) => panic!("Did not receive any message from the server while expecting MulliganMessage::Complete."),
    }
}

pub fn verify_mulligan_cards(
    server_state: &ServerState,
    player_type: PlayerType,
    rerolled_cards: &[Uuid],
    deal_cards: Option<&[Uuid]>,
    reroll_count: usize,
) {
    let game = server_state.game.try_lock().unwrap();
    let player = game.get_player_by_type(player_type).get();
    let deck_cards = player.get_deck().get_cards();

    // 덱 크기 검증
    if deck_cards.len() != 25 {
        panic!(
            "Mulligan error: Wrong deck size. expected: {}, Got: {}",
            25,
            deck_cards.len()
        );
    }

    // 뽑은 카드가 덱에 없는지 확인
    for card in rerolled_cards {
        if deck_cards.contains_uuid(card.clone()) {
            panic!(
                "Mulligan error (reroll_count = {}): Rerolled card {:?} should not be present in deck",
                reroll_count, card
            );
        }
    }

    // RerollRequest 경우 이전 카드가 덱에 복원되었는지 확인
    if let Some(cards) = deal_cards {
        for card in cards {
            if !deck_cards.contains_uuid(card.clone()) {
                panic!(
                    "Mulligan restore error (reroll_count = {}): Restored card {:?} not found in deck",
                    reroll_count, card
                );
            }
        }
    }
}

pub struct RequestTest {
    pub response: String,
}

impl RequestTest {
    pub async fn connect(
        step: &str,
        addr: SocketAddr,
        cookie: String,
    ) -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://{}/{}", addr, step))
            .header("Cookie", cookie)
            .send()
            .await?;

        Ok(RequestTest {
            response: response.text().await.expect("Failed to get response"),
        })
    }

    /// 특정 타입의 메세지를 예상합니다. 예상한 메세지가 아닌 경우, panic! 합니다.
    pub fn expect_message<T, F, R>(&mut self, extractor: F) -> R
    where
        T: DeserializeOwned,
        F: Fn(T) -> R,
    {
        let msg = serde_json::from_str::<T>(self.response.as_str()).expect("Failed to parse JSON");
        extractor(msg)
    }
}

//-------------------------------
// Draw 관련 함수
//-------------------------------
impl RequestTest {
    /// Draw-Answer 메시지를 예상하고 카드 Uuid 를 반환합니다
    pub fn expect_draw_card(&mut self) -> Uuid {
        let extractor = |message: draw::ServerMessage| match message {
            draw::ServerMessage::DrawAnswer(data) => {
                data.cards.parse().unwrap_or_else(|e| {
                    // TODO: Log 함수 사용
                    panic!("Failed to parse card ID: {:?}", e);
                })
            }
        };
        self.expect_message(extractor)
    }

    /// Error 메시지를 예상합니다.
    pub fn expect_error(&mut self) -> String {
        let extractor = |message: ErrorMessage| match message {
            ErrorMessage::Error(data) => data.message,
        };
        self.expect_message(extractor)
    }
}

pub struct WebSocketTest {
    stream: WebSocketStream<TokioAdapter<TcpStream>>,
}
//-------------------------------
// WebSocketTest 구현
//-------------------------------
impl WebSocketTest {
    /// 웹소켓 연결을 생성하고 래퍼 객체를 반환합니다
    pub async fn connect(url: String, cookie: String) -> Result<Self, tungstenite::Error> {
        let request = Request::builder()
            .uri(&url)
            .header("Cookie", cookie)
            .header("Host", url.split('/').nth(2).unwrap_or("localhost"))
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Version", "13")
            .body(())?;

        let (stream, response) = connect_async(request).await?;

        assert_eq!(
            response.status(),
            tungstenite::http::StatusCode::SWITCHING_PROTOCOLS
        );

        Ok(Self { stream })
    }

    /// 메시지를 전송합니다
    pub async fn send(&mut self, msg: impl Into<Message>) -> Result<(), tungstenite::Error> {
        self.stream.send(msg.into()).await
    }

    /// 특정 타입의 메시지가 도착할 때까지 대기하고, 다른 메시지(ping 포함)는 적절히 처리합니다
    pub async fn expect_message<T, F, R>(&mut self, extractor: F) -> R
    where
        T: DeserializeOwned,
        F: Fn(T) -> R,
    {
        let callback = async {
            loop {
                match self.stream.next().await {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(parsed) = serde_json::from_str::<T>(&text) {
                            return extractor(parsed);
                        } else {
                            println!("Failed to parse: {}", text);
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        self.stream.send(Message::Pong(data)).await.ok();
                    }
                    Some(Ok(_)) => continue,
                    Some(Err(e)) => panic!("WebSocket error: {:?}", e),
                    None => panic!("WebSocket closed unexpectedly"),
                }
            }
        };
        match tokio::time::timeout(Duration::from_secs(TIMEOUT), callback).await {
            Ok(result) => result,
            Err(_) => panic!("Expected message timeout after 5 seconds"),
        }
    }

    /// 에러 메시지를 기다리고 에러 문자열을 반환합니다
    pub async fn expect_error(&mut self) -> String {
        let extractor = |message: ErrorMessage| match message {
            ErrorMessage::Error(data) => data.message,
        };
        self.expect_message(extractor).await
    }
}

//-------------------------------
// Mulligan 관련 함수
//-------------------------------
impl WebSocketTest {
    /// 멀리건 딜 메시지를 기다리고 카드 ID 리스트를 반환합니다
    pub async fn expect_mulligan_deal(&mut self) -> Vec<Uuid> {
        self.expect_message(|message: mulligan::ServerMessage| match message {
            mulligan::ServerMessage::Deal(data) => data.cards.to_vec_uuid(),
            other => panic!("Expected MulliganMessage::Deal but got: {:?}", other),
        })
        .await
    }

    /// 멀리건 완료 메시지를 기다리고 카드 ID 리스트를 반환합니다
    pub async fn expect_mulligan_complete(&mut self) -> Vec<Uuid> {
        let extractor = |message: mulligan::ClientMessage| match message {
            mulligan::ClientMessage::Complete(data) => data.cards.to_vec_uuid(),
            other => panic!("Expected MulliganMessage::Complete but got: {:?}", other),
        };
        self.expect_message(extractor).await
    }

    /// Reroll-Answer 메시지를 기다리고 카드 ID 리스트를 반환합니다
    pub async fn expect_mulligan_answer(&mut self) -> Vec<Uuid> {
        let extractor = |message: mulligan::ServerMessage| match message {
            mulligan::ServerMessage::RerollAnswer(data) => data.cards.to_vec_uuid(),
            other => panic!("Expected MulliganMessage::Answer but got: {:?}", other),
        };
        self.expect_message(extractor).await
    }
}

async fn verify_card_removed_from_deck(
    server_state: &Data<ServerState>,
    player_type: &str,
    card: Uuid,
) {
    let game = server_state.game.lock().await;
    let player = game.get_player_by_type(player_type).get();
    let deck = player.get_deck();
    assert!(
        !deck.get_cards().contains_uuid(card),
        "Card {} was not removed from the deck",
        card
    );
}
