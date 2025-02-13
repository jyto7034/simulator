const CARD_NUM: usize = 25;

#[cfg(test)]
mod utils_test {
    use card_game::{
        test::generate_random_deck_json,
        utils::{deckcode_to_cards, parse_json_to_deck_code},
    };

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
    use actix_web::{test, web};
    use card_game::{
        server::{
            end_point::handle_mulligan_cards,
            types::{SessionKey, ServerState},
        },
        test::{generate_random_deck_json, initialize_app},
        utils::parse_json_to_deck_code,
    };
    use serde_json::json;
    use tokio::sync::Mutex;

    fn create_server_state() -> web::Data<ServerState> {
        let (deck_json, original_cards) = generate_random_deck_json();
        let (deck_json2, _) = generate_random_deck_json();

        // 2. JSON을 덱 코드로 변환
        let deck_codes = parse_json_to_deck_code(Some(deck_json), Some(deck_json2))
            .expect("Failed to parse deck code");

        let app = initialize_app(deck_codes.0, deck_codes.1, 0);

        web::Data::new(ServerState {
            game: Mutex::new(app.game),
            player_cookie: SessionKey("".to_string()),
            opponent_cookie: SessionKey("".to_string()),
        })
    }

    use actix_web::{App, HttpServer};
    use async_tungstenite::{tokio::connect_async, tungstenite::Message};
    use std::net::TcpListener;
    use futures_util::stream::StreamExt;
    use tokio::time::{sleep, Duration};
    /// index 엔드포인트에 대한 테스트
    #[actix_web::test]
    async fn test_websocket_integration() -> std::io::Result<()> {
        // 테스트용 상태 생성 (실제 사용하는 서버 상태 생성 함수를 사용)
        let server_state = create_server_state();
        
        // 사용 가능한 포트에 바인딩합니다.
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;
        
        // App 생성 및 필요한 data/서비스 등록
        let server = HttpServer::new(move || {
            App::new()
                .app_data(server_state.clone())
                // 테스트를 위해 AuthPlayer 를 bypass하거나, 미리 등록한 더미 Guard 를 사용하도록 구성되어 있다고 가정합니다.
                .service(handle_mulligan_cards)
        })
        .listen(listener)?
        .run();
        
        // actix_web::rt::spawn 대신 tokio::spawn 또는 actix_web::rt::spawn 사용 (현재 예제에서는 tokio::spawn 사용)
        tokio::spawn(server);
        
        // 서버가 기동될 시간을 잠시 기다립니다.
        sleep(Duration::from_millis(100)).await;
        
        // WebSocket URL 구성 (예: "ws://127.0.0.1:{포트}/mulligan_step")
        let url = format!("ws://{}{}", addr, "/mulligan_step");
        
        // async-tungstenite를 사용하여 WebSocket 연결을 생성합니다.
        let (mut ws_stream, response) = connect_async(&url)
            .await
            .expect("Failed to connect");
        
        // HTTP 핸드셰이크가 성공하면 응답 상태는 101 Switching Protocols 입니다.
        assert_eq!(
            response.status(),
            async_tungstenite::tungstenite::http::StatusCode::SWITCHING_PROTOCOLS
        );
        
        // 엔드포인트 코드에서는 업그레이드 후 deal 메시지(new_cards_json)를 즉시 클라이언트로 전송하도록 구성되어 있습니다.
        // 첫 번째로 서버에서 오는 메시지를 확인합니다.
        if let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Initial message from server: {}", text);
                    // 여기서 실제 테스트 전용 assert 로 deal 메시지 형식이나 내용 검증을 추가할 수 있습니다.
                }
                Ok(other) => println!("Unexpected message type: {:?}", other),
                Err(e) => panic!("WebSocket error (initial message): {:?}", e),
            }
        }
        
        // // 테스트로 클라이언트에서 서버로 메시지를 전송하여 후속 처리가 되는지 확인합니다.
        // ws_stream
        //     .send(Message::Text("Hello, server!".into()))
        //     .await
        //     .expect("Failed to send message");
        
        // // 후속 메시지를 받아서 검증합니다.
        // if let Some(msg) = ws_stream.next().await {
        //     match msg {
        //         Ok(Message::Text(text)) => {
        //             println!("Received after sending: {}", text);
        //             // 필요하다면 추가 검증(assert) 로직 추가 가능
        //         }
        //         Ok(other) => println!("Non-text message received: {:?}", other),
        //         Err(e) => panic!("WebSocket error (after sending): {:?}", e),
        //     }
        // }
        
        Ok(())
    }
}
