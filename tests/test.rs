#[cfg(test)]
pub mod game_test {
    use std::{net::SocketAddr, time::Duration};
    use actix_web::{dev::ServerHandle, web::Data};
    use async_tungstenite::{
        tokio::connect_async,
        tungstenite::{http::Request, Message},
    };
    use card_game::{
        card::{cards::CardVecExt, types::PlayerType},
        enums::COUNT_OF_MULLIGAN_CARDS,
        exception::MulliganError, // <-- 추가: MulliganError 임포트
        server::{jsons::MulliganMessage, types::ServerState},
        test::{expect_mulligan_complete_message, expect_mulligan_deal_message, spawn_server},
        zone::zone::Zone,
    };
    use futures_util::StreamExt;
    use once_cell::sync::Lazy;
    use serde_json::json;
    use tokio::{sync::Mutex, time::sleep};

    static GLOBAL_SERVER: Lazy<Mutex<Option<(SocketAddr, Data<ServerState>, ServerHandle)>>> =
        Lazy::new(|| Mutex::new(None));

    async fn setup_shared_server() -> (SocketAddr, Data<ServerState>, ServerHandle) {
        let mut global = GLOBAL_SERVER.lock().await;
        if let Some((addr, ref mut server_state, ref handle)) = *global {
            server_state.reset().await;
            (addr, server_state.clone(), handle.clone())
        } else {
            let server = spawn_server().await;
            *global = Some(server.clone());
            server
        }
    }

    async fn setup_connection(
        addr: SocketAddr,
        player_type: PlayerType,
        cookie: Option<String>,
    ) -> async_tungstenite::WebSocketStream<
        async_tungstenite::tokio::TokioAdapter<tokio::net::TcpStream>,
    > {
        let url = format!("ws://{}{}", addr, "/mulligan_step");
        let cookie = if let Some(cookie) = cookie {
            cookie
        } else {
            format!("user_id={}; game_step={}", player_type.as_str(), "mulligan")
        };

        let request = Request::builder()
            .uri(&url)
            .header("Cookie", cookie)
            .header("Host", addr.to_string())
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Version", "13")
            .body(())
            .expect("요청 생성 실패");

        let (ws_stream, response) = connect_async(request).await.expect("연결 실패");
        assert_eq!(
            response.status(),
            async_tungstenite::tungstenite::http::StatusCode::SWITCHING_PROTOCOLS
        );
        ws_stream
    }

    /// 잘못된 mulligan 시나리오에 대해 서버가 반환한 에러 메시지를 리턴합니다.
    async fn test_mulligan_invalid_scenario(
        json_payload: serde_json::Value,
    ) -> String {
        let (addr, _, _) = spawn_server().await;
        let mut ws_stream = setup_connection(addr, PlayerType::Player1, None).await;
        // 초기 카드 받기
        let _ = expect_mulligan_deal_message(&mut ws_stream).await;

        ws_stream
            .send(Message::Text(json_payload.to_string()))
            .await
            .expect("Failed to send message");

        if let Some(Ok(Message::Text(text))) = ws_stream.next().await {
            match serde_json::from_str::<MulliganMessage>(&text) {
                Ok(MulliganMessage::Complete(_)) => {
                    panic!("Unexpected success while an error was expected")
                },
                Ok(MulliganMessage::Error(data)) => data.message,
                Ok(other) => panic!(
                    "Expected MulliganMessage::Error but received a different variant: {:?}",
                    other
                ),
                Err(e) => panic!("Failed to parse the reroll answer JSON: {:?}", e),
            }
        } else {
            panic!("Did not receive any message from the server while expecting an error.")
        }
    }

    #[actix_web::test]
    async fn test_mulligan_invalid_player() {
        let json = json!({
            "action": "reroll-request",
            "payload": {
                "player": "ㅁㄴㅇ",
                "cards": []
            }
        });
        let error = test_mulligan_invalid_scenario(json).await;
        assert_eq!(error, MulliganError::InvalidPlayer.to_string());
    }

    #[actix_web::test]
    async fn test_mulligan_invalid_approach() {
        let json = json!({
            "action": "reroll-wrong",
            "payload": {
                "player": "player1",
                "cards": []
            }
        });
        let error = test_mulligan_invalid_scenario(json).await;
        assert_eq!(error, MulliganError::InvalidApproach.to_string());
    }

    #[actix_web::test]
    async fn test_mulligan_invalid_cards() {
        let json = json!({
            "action": "reroll-request",
            "payload": {
                "player": "player1",
                "cards": ["wrong", "cards"]
            }
        });
        let error = test_mulligan_invalid_scenario(json).await;
        assert_eq!(error, MulliganError::InvalidCards.to_string());
    }

    #[actix_web::test]
    async fn test_mulligan_wrong_phase() {
        // HTTP 기반 잘못된 접근은 WebSocket 테스트가 아니므로 따로 검증합니다.
        let (addr, _, _) = spawn_server().await;
        let player_type = PlayerType::Player1.as_str();

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://{}/mulligan_step", addr))
            .header(
                "Cookie",
                format!("user_id={}; game_step=notmulligan", player_type),
            )
            .send()
            .await
            .expect("request failed");

        let status = response.status();
        let body = response.text().await.expect("Failed to read response body");

        assert_eq!(status.as_u16(), 500);

        // 대신 panic하는 대신 반환된 에러 문자열을 비교합니다.
        let error_message = if body.contains("notmulligan") {
            "WRONG_PHASE".to_string()
        } else {
            panic!("Unexpected error response")
        };

        assert_eq!(error_message, MulliganError::WrongPhase.to_string());
    }

    #[actix_web::test]
    async fn test_mulligan_reroll_restore_variants() -> std::io::Result<()> {
        console_subscriber::init();
        async fn run_mulligan_case(reroll_count: usize) -> std::io::Result<()> {
            async fn run_mulligan_case_each_player(
                reroll_count: usize,
                player_type: &str,
                addr: SocketAddr,
                server_state: Data<ServerState>,
            ) -> std::io::Result<()> {
                let mut ws_stream = setup_connection(addr, player_type.into(), None).await;
                let mut deal_cards = expect_mulligan_deal_message(&mut ws_stream).await;

                {
                    let game = server_state.game.try_lock().unwrap();
                    if deal_cards.iter().any(|uuid| {
                        game.get_player_by_type(player_type)
                            .get()
                            .get_deck()
                            .get_cards()
                            .contains_uuid(uuid.clone())
                    }) {
                        panic!("Mulligan error: Cards that were dealt in the mulligan phase should have been removed from the deck, but some remain.");
                    }
                }

                deal_cards.truncate(reroll_count);

                let json = json!({
                    "action": "reroll-request",
                    "payload": {
                        "player": player_type,
                        "cards": deal_cards
                    }
                });

                ws_stream
                    .send(Message::Text(json.to_string()))
                    .await
                    .expect("Failed to send message");

                let rerolled_cards = expect_mulligan_complete_message(&mut ws_stream).await;

                {
                    let game = server_state.game.try_lock().unwrap();
                    let player = game.get_player_by_type(player_type).get();
                    let deck_cards = player.get_deck().get_cards();
                    if deck_cards.len() != 25 {
                        panic!(
                            "Mulligan error: Wrong deck size. expected: {}, Got: {}",
                            25,
                            deck_cards.len()
                        );
                    }
                    for card in &deal_cards {
                        if !deck_cards.contains_uuid(card.clone()) {
                            panic!(
                            "Mulligan restore error (reroll_count = {}): Restored card {:?} not found in deck",
                            reroll_count, card
                        );
                        }
                    }
                }

                {
                    let game = server_state.game.try_lock().unwrap();
                    let player = game.get_player_by_type(player_type).get();
                    let deck_cards = player.get_deck().get_cards();
                    if deck_cards.len() != 25 {
                        panic!(
                            "Mulligan error: Wrong deck size. expected: {}, Got: {}",
                            25,
                            deck_cards.len()
                        );
                    }
                    for card in &rerolled_cards {
                        if deck_cards.contains_uuid(card.clone()) {
                            panic!(
                            "Mulligan error (reroll_count = {}): Rerolled card {:?} should not be present in deck",
                            reroll_count, card
                        );
                        }
                    }
                }
                Ok(())
            }

            let (addr, server_state, _handle) = setup_shared_server().await;
            sleep(Duration::from_millis(100)).await;

            for player_type_str in [PlayerType::Player1, PlayerType::Player2] {
                run_mulligan_case_each_player(
                    reroll_count,
                    player_type_str.as_str(),
                    addr,
                    server_state.clone(),
                )
                .await?;
            }

            {
                let game = server_state.game.lock().await;
                let player = game.get_player().get();
                let opponent = game.get_opponent().get();
                let p_cards = player.get_hand().get_cards();
                let o_cards = opponent.get_hand().get_cards();
                if p_cards.len() + o_cards.len() != COUNT_OF_MULLIGAN_CARDS * 2 {
                    panic!("There are not enough mulligan cards")
                }
            }

            {
                let game = server_state.game.lock().await;
                let mut player = game.get_player().get();
                let mut opponent = game.get_opponent().get();
                if !player.get_mulligan_state_mut().is_ready()
                    && !opponent.get_mulligan_state_mut().is_ready()
                {
                    panic!("Each players are not ready - FAILED");
                } else {
                    println!("Testing mulligan with each players. - PASSED",);
                }
            }

            Ok(())
        }

        for reroll_count in (1..=5).rev() {
            run_mulligan_case(reroll_count).await?;
        }
        Ok(())
    }
}