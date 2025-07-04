use std::{
    io::Read,
    net::{SocketAddr, TcpListener},
    time::Duration,
};

use actix::{Actor, Addr};
use actix_web::{
    dev::ServerHandle,
    web::{self, Data},
    App, HttpServer,
};
use async_tungstenite::tungstenite::{self, error::UrlError, http::Request, Message};
use ctor::ctor;
use futures::{SinkExt, StreamExt};
use rand::{seq::SliceRandom, thread_rng};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use simulator_core::{
    card::{types::PlayerKind, Card},
    card_gen::CardGenerator,
    enums::{CARD_JSON_PATH, CLIENT_TIMEOUT, HEARTBEAT_INTERVAL, MAX_CARD_SIZE},
    game::GameActor,
    setup_logger,
    utils::{json, parse_json_to_deck_code},
};
use tracing::info;
use url::Url;
use uuid::Uuid;

use crate::server::game;

pub struct ServerState {
    pub game: Addr<GameActor>,
    pub player1_id: Uuid,
    pub player2_id: Uuid,
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

pub fn create_server_state() -> web::Data<ServerState> {
    let (deck_json, _original_cards) = generate_random_deck_json();
    let (deck_json2, _) = generate_random_deck_json();

    let deck_codes = parse_json_to_deck_code(Some(deck_json), Some(deck_json2))
        .expect("Failed to parse deck code");

    let player1_id = Uuid::new_v4();
    let player2_id = Uuid::new_v4();

    let game_actor = GameActor::create(|_ctx| {
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
        info!("[TEST] Response: {}", self.response);
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

    /// Sends a close frame and attempts to gracefully close the WebSocket connection.
    /// This makes the "disconnect" action explicit and reliable for tests.
    pub async fn close(&mut self) -> Result<(), tungstenite::Error> {
        info!("[TEST_WS] Sending Close frame to server.");
        // First, send a close message to the server.
        self.sink.send(Message::Close(None)).await?;
        // Then, fully close the sink.
        self.sink.close().await
    }

    pub async fn send(&mut self, msg: impl Into<Message>) -> Result<(), tungstenite::Error> {
        self.sink.send(msg.into()).await // 싱크를 통해 메시지 전송
    }

    pub async fn expect_message_opt<T, F>(&mut self, mut extractor: F) -> T
    where
        T: DeserializeOwned,
        F: FnMut(T) -> Option<T>, // 클로저가 Option<T>를 반환하도록 변경
    {
        let callback = async {
            loop {
                match self.stream.next().await {
                    Some(Ok(Message::Text(text))) => {
                        info!("[TEST] Received message: {}", text);
                        if let Ok(parsed) = serde_json::from_str::<T>(&text) {
                            // 클로저를 실행하고, Some(T)가 반환되면 루프를 종료하고 값을 반환
                            if let Some(result) = extractor(parsed) {
                                return result;
                            }
                            // None이 반환되면, 원하는 메시지가 아니므로 계속 루프를 돕니다.
                        } else {
                            info!("[TEST] Failed to parse into expected type: {}", text);
                            continue;
                        }
                    }
                    // ... (Ping, Pong, Close 등 나머지 핸들링은 동일)
                    Some(Ok(Message::Ping(_))) => continue,
                    Some(Ok(Message::Pong(_))) => continue,
                    Some(Ok(Message::Close(reason))) => {
                        panic!("WebSocket closed unexpectedly. Reason: {:?}", reason);
                    }
                    Some(Ok(msg)) => {
                        info!("[TEST] Ignoring other message type: {:?}", msg);
                        continue;
                    }
                    Some(Err(e)) => panic!("WebSocket error: {:?}", e),
                    None => panic!("WebSocket closed unexpectedly"),
                }
            }
        };

        match tokio::time::timeout(Duration::from_secs(CLIENT_TIMEOUT), callback).await {
            Ok(result) => result,
            Err(_) => panic!("Expected message timeout after {} seconds", CLIENT_TIMEOUT),
        }
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
                        info!("[TEST] Received message: {}", text);
                        if let Ok(parsed) = serde_json::from_str::<T>(&text) {
                            return extractor(parsed);
                        } else {
                            info!("[TEST] Failed to parse into expected type: {}", text);
                            // 중요: 여기서 continue를 해야 다른 타입 메시지를 기다림
                            continue;
                        }
                    }
                    Some(Ok(Message::Ping(_))) => {
                        info!("[TEST] Received ping (auto-pong by tungstenite), ignoring.");
                        continue; // 자동으로 pong 처리됨
                    }
                    Some(Ok(Message::Pong(_))) => {
                        info!("[TEST] Received Pong, ignoring.");
                        continue; // Pong은 무시하고 다음 메시지 기다림
                    }
                    Some(Ok(Message::Close(reason))) => {
                        panic!("WebSocket closed unexpectedly while waiting for specific message. Reason: {:?}", reason);
                    }
                    Some(Ok(msg)) => {
                        info!("[TEST] Ignoring other message type: {:?}", msg);
                        continue; // 다른 메시지 타입 무시
                    }
                    Some(Err(e)) => panic!("WebSocket error: {:?}", e),
                    None => panic!("WebSocket closed unexpectedly"),
                }
            }
        };

        match tokio::time::timeout(Duration::from_secs(CLIENT_TIMEOUT), callback).await {
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
