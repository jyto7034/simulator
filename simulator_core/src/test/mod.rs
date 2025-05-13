use std::{
    io::Read,
    net::{SocketAddr, TcpListener},
    time::Duration,
};

use actix::Actor;
use async_tungstenite::tungstenite::{self, error::UrlError, http::Request, Message};
use ctor::ctor;
use futures::SinkExt;
use futures_util::StreamExt;
use rand::{seq::SliceRandom, thread_rng};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tracing::info;
use url::Url;
use uuid::Uuid;

use crate::{
    card::{types::PlayerKind, Card},
    card_gen::CardGenerator,
    enums::{CARD_JSON_PATH, HEARTBEAT_INTERVAL, MAX_CARD_SIZE},
    game::GameActor,
    server::{end_point::game, types::ServerState},
    setup_logger,
    utils::{json, parse_json_to_deck_code},
};

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
    App, HttpServer,
};

pub fn create_server_state() -> web::Data<ServerState> {
    let (deck_json, _original_cards) = generate_random_deck_json();
    let (deck_json2, _) = generate_random_deck_json();

    let deck_codes = parse_json_to_deck_code(Some(deck_json), Some(deck_json2))
        .expect("Failed to parse deck code");

    let player1_id = Uuid::new_v4();
    let player2_id = Uuid::new_v4();

    let game_actor = GameActor::create(|ctx| {
        let game_actor = GameActor::new(
            Uuid::new_v4(),
            player1_id,
            player2_id,
            deck_codes.0,
            deck_codes.1,
            PlayerKind::Player1,
        );

        game_actor
    });

    web::Data::new(ServerState {
        game: game_actor,
        player1_id: player1_id,
        player2_id: player2_id,
    })
}

pub async fn spawn_server() -> (SocketAddr, Data<ServerState>, ServerHandle) {
    let server_state = create_server_state();
    let server_state_clone = server_state.clone();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    let server = HttpServer::new(move || App::new().app_data(server_state.clone()).service(game))
        .listen(listener)
        .unwrap()
        .run();

    let handle = server.handle();
    tokio::spawn(server);

    (addr, server_state_clone, handle)
}

pub fn verify_mulligan_cards(
    server_state: &ServerState,
    player_type: PlayerKind,
    rerolled_cards: &[Uuid],
    deal_cards: Option<&[Uuid]>,
    reroll_count: usize,
) {
    todo!()
    // let game = server_state.game.try_lock().unwrap();
    // let player = game.get_player_by_type(player_type).get();
    // let deck_cards = player.get_deck().get_cards();

    // // 덱 크기 검증
    // if deck_cards.len() != 25 {
    //     panic!(
    //         "Mulligan error: Wrong deck size. expected: {}, Got: {}",
    //         25,
    //         deck_cards.len()
    //     );
    // }

    // // 뽑은 카드가 덱에 없는지 확인
    // for card in rerolled_cards {
    //     if deck_cards.contains_uuid(card.clone()) {
    //         panic!(
    //             "Mulligan error (reroll_count = {}): Rerolled card {:?} should not be present in deck",
    //             reroll_count, card
    //         );
    //     }
    // }

    // // RerollRequest 경우 이전 카드가 덱에 복원되었는지 확인
    // if let Some(cards) = deal_cards {
    //     for card in cards {
    //         if !deck_cards.contains_uuid(card.clone()) {
    //             panic!(
    //                 "Mulligan restore error (reroll_count = {}): Restored card {:?} not found in deck",
    //                 reroll_count, card
    //             );
    //         }
    //     }
    // }
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
        info!("Response: {}", self.response);
        let msg = serde_json::from_str::<T>(self.response.as_str())
            .expect("Failed to parse JSON (expect_message)");
        extractor(msg)
    }
}

pub struct WebSocketTest {
    pub stream: futures_util::stream::SplitStream<
        async_tungstenite::WebSocketStream<
            async_tungstenite::tokio::TokioAdapter<tokio::net::TcpStream>,
        >,
    >,
    pub sink: futures_util::stream::SplitSink<
        async_tungstenite::WebSocketStream<
            async_tungstenite::tokio::TokioAdapter<tokio::net::TcpStream>,
        >,
        Message,
    >,
}
//-------------------------------
// WebSocketTest 구현
//-------------------------------

impl WebSocketTest {
    pub async fn connect(url: String, cookie: String) -> Result<Self, tungstenite::Error> {
        // ... (connect 로직은 이전과 동일) ...
        let mut url = Url::parse(&url).unwrap();
        if url.scheme() == "http" {
            url.set_scheme("ws").unwrap()
        } else if url.scheme() != "ws" && url.scheme() != "wss" {
            return Err(tungstenite::Error::Url(UrlError::UnsupportedUrlScheme));
        }

        let host = url
            .host_str()
            .ok_or(tungstenite::Error::Url(UrlError::EmptyHostName))?;
        let host_header = if let Some(port) = url.port() {
            format!("{}:{}", host, port)
        } else {
            host.to_string()
        };

        let request = Request::builder()
            .uri(url.as_str())
            .header("Cookie", cookie)
            .header("Host", host_header)
            .header(
                "Sec-WebSocket-Key",
                tungstenite::handshake::client::generate_key(),
            )
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Version", "13")
            .body(())?;

        let (ws_stream, response) = async_tungstenite::tokio::connect_async(request).await?;

        assert_eq!(
            response.status(),
            tungstenite::http::StatusCode::SWITCHING_PROTOCOLS
        );

        let (sink, stream) = ws_stream.split(); // 스트림과 싱크 분리

        Ok(Self { stream, sink }) // 분리된 스트림과 싱크 저장
    }

    pub async fn send(&mut self, msg: impl Into<Message>) -> Result<(), tungstenite::Error> {
        self.sink.send(msg.into()).await // 싱크를 통해 메시지 전송
    }

    pub async fn expect_message<T, F, R>(&mut self, extractor: F) -> R
    where
        T: DeserializeOwned,
        F: Fn(T) -> R,
    {
        let callback = async {
            loop {
                match self.stream.next().await {
                    Some(Ok(Message::Text(text))) => {
                        println!("Received message: {}", text);
                        if let Ok(parsed) = serde_json::from_str::<T>(&text) {
                            return extractor(parsed);
                        } else {
                            println!("Failed to parse into expected type: {}", text);
                            // 중요: 여기서 continue를 해야 다른 타입 메시지를 기다림
                            continue;
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        println!("Received ping, sending pong");
                        // 중요: Pong은 sink를 통해 보내야 함
                        if self.sink.send(Message::Pong(data)).await.is_err() {
                            // 에러 처리 필요
                            eprintln!("Failed to send Pong");
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        println!("Received Pong, ignoring.");
                        continue; // Pong은 무시하고 다음 메시지 기다림
                    }
                    Some(Ok(Message::Close(reason))) => {
                        panic!("WebSocket closed unexpectedly while waiting for specific message. Reason: {:?}", reason);
                    }
                    Some(Ok(msg)) => {
                        println!("Ignoring other message type: {:?}", msg);
                        continue; // 다른 메시지 타입 무시
                    }
                    Some(Err(e)) => panic!("WebSocket error: {:?}", e),
                    None => panic!("WebSocket closed unexpectedly"),
                }
            }
        };
        // 타임아웃 시간은 HEARTBEAT_INTERVAL 보다 길게 설정
        match tokio::time::timeout(Duration::from_secs(HEARTBEAT_INTERVAL + 5), callback).await {
            Ok(result) => result,
            Err(_) => panic!(
                "Expected message timeout after {} seconds",
                HEARTBEAT_INTERVAL + 5
            ),
        }
    }
}

#[ctor]
fn init() {
    setup_logger();
}
