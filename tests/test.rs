#[cfg(test)]
pub mod heartbeat {
    use std::time::Duration;

    use async_tungstenite::{
        tokio::connect_async,
        tungstenite::{self, http::Request, Message},
    };
    use card_game::{
        card::types::PlayerType,
        enums::CLIENT_TIMEOUT,
        game::phase::Phase,
        test::{spawn_server, WebSocketTest},
    };
    use futures::StreamExt;
    use serde::{Deserialize, Serialize};
    use tracing::info;
    use uuid::Uuid;

    #[actix_web::test]
    #[should_panic]
    async fn test_heartbeat_connection_timeout() {
        // 서버 인스턴스 생성
        let (addr, state, _) = spawn_server().await;
        let player_type = PlayerType::Player1.as_str();

        // 하트비트 연결 URL 및 쿠키 설정
        let url = format!("ws://{}/heartbeat", addr);
        let cookie = format!("user_id={}; game_step={}", player_type, "heartbeat");

        {
            state
                .game
                .lock()
                .await
                .get_phase_state_mut()
                .set_phase(Phase::Heartbeat);
        }

        // WebSocket 연결 수립
        let mut ws = WebSocketTest::connect(url, cookie).await.unwrap();
        info!("하트비트 WebSocket 연결 성공");

        // 연결 안정성 테스트 - 메시지 전송
        #[derive(Serialize)]
        struct TestMessage {
            #[serde(rename = "type")]
            msg_type: String,
            content: String,
        }

        let test_msg = TestMessage {
            msg_type: "test".to_string(),
            content: "heartbeat test".to_string(),
        };

        // 메시지 전송
        ws.send(Message::Text(serde_json::to_string(&test_msg).unwrap()))
            .await
            .unwrap();
        info!("테스트 메시지 전송 완료");

        // 충분한 시간 대기하여 서버가 연결을 유지하는지 확인
        tokio::time::sleep(Duration::from_secs(CLIENT_TIMEOUT + 1)).await;

        // 여전히 메시지를 보낼 수 있다면 연결이 유지되고 있는 것
        let keepalive_msg = TestMessage {
            msg_type: "keepalive".to_string(),
            content: "still connected".to_string(),
        };

        match ws
            .send(Message::Text(
                serde_json::to_string(&keepalive_msg).unwrap(),
            ))
            .await
        {
            Ok(_) => info!("연결이 유지되고 있음 확인"),
            Err(e) => panic!("연결이 끊김: {:?}", e),
        }

        // 정상 종료
        ws.send(Message::Close(None)).await.unwrap();
    }

    #[actix_web::test]
    async fn test_heartbeat_connection() {
        // 서버 인스턴스 생성
        let (addr, state, _) = spawn_server().await;
        let player_type = PlayerType::Player1.as_str();

        // 하트비트 연결 URL 및 쿠키 설정
        let url = format!("ws://{}/heartbeat", addr);
        let cookie = format!("user_id={}; game_step={}", player_type, "heartbeat");

        {
            state
                .game
                .lock()
                .await
                .get_phase_state_mut()
                .set_phase(Phase::Heartbeat);
        }

        // WebSocket 연결 수립
        let mut ws = WebSocketTest::connect(url, cookie).await.unwrap();
        info!("하트비트 WebSocket 연결 성공");

        // 연결 안정성 테스트 - 메시지 전송
        #[derive(Serialize)]
        struct TestMessage {
            #[serde(rename = "type")]
            msg_type: String,
            content: String,
        }

        let test_msg = TestMessage {
            msg_type: "test".to_string(),
            content: "heartbeat test".to_string(),
        };

        // 메시지 전송
        ws.send(Message::Text(serde_json::to_string(&test_msg).unwrap()))
            .await
            .unwrap();
        info!("테스트 메시지 전송 완료");

        // 충분한 시간 대기하여 서버가 연결을 유지하는지 확인
        tokio::time::sleep(Duration::from_secs(CLIENT_TIMEOUT + 1)).await;

        // 여전히 메시지를 보낼 수 있다면 연결이 유지되고 있는 것
        let keepalive_msg = TestMessage {
            msg_type: "keepalive".to_string(),
            content: "still connected".to_string(),
        };

        info!("연결 유지 확인을 위한 keepalive 메시지 전송 시도");

        // keepalive 메시지는 전송이 되는데, CloseMsg Send 는 또 실패함.
        // 로그 순서상 keepalive 메시지도 전송이 실패되어야 하는 것 같은데
        // match 분기문으로 더 상세한 로그 찍어야할 듯

        let result = ws
            .send(Message::Text(
                serde_json::to_string(&keepalive_msg).unwrap(),
            ))
            .await;

        match result {
            Ok(_) => match ws.send(Message::Close(None)).await {
                Ok(_) => info!("연결이 유지되고 있음 확인"),
                Err(_) => info!("연결이 끊김 확인"),
            },
            Err(e) => {
                match ws.send(Message::Close(None)).await {
                    Ok(_) => info!("연결이 유지되고 있음 확인"),
                    Err(_) => info!("연결이 끊김 확인"),
                }
                info!("예상 못 한 분기점: {}", e);
            }
        }
    }

    #[actix_web::test]
    async fn test_heartbeat_session_management() {
        // 서버 인스턴스 생성
        let (addr, state, _) = spawn_server().await;
        let player_type = PlayerType::Player1.as_str();

        {
            state
                .game
                .lock()
                .await
                .get_phase_state_mut()
                .set_phase(Phase::Heartbeat);
        }

        // 하트비트 연결 URL 및 쿠키 설정
        let url = format!("ws://{}/heartbeat", addr);
        let cookie = format!("user_id={}; game_step={}", player_type, "heartbeat");

        // 첫 번째 세션 생성
        let mut ws1 = WebSocketTest::connect(url.clone(), cookie.clone())
            .await
            .unwrap();
        info!("첫 번째 WebSocket 연결 생성");

        // 세션 ID 확인을 위한 초기 메시지 수신
        #[derive(Deserialize)]
        struct ConnectionMessage {
            #[serde(rename = "type")]
            #[allow(dead_code)]
            msg_type: String,
            #[allow(dead_code)]
            player: String,
            session_id: String,
        }

        let session_id = ws1
            .expect_message(|msg: ConnectionMessage| msg.session_id)
            .await
            .parse::<Uuid>()
            .unwrap();
        info!("첫 번째 세션 ID: {}", session_id);

        // 첫 번째 세션이 활성화되었는지 확인
        assert!(
            state
                .session_manager
                .is_valid_session(PlayerType::Player1, session_id, Phase::Heartbeat)
                .await
        );

        // 첫 번째 세션 종료
        ws1.send(Message::Close(None)).await.unwrap();

        // 세션이 종료되었는지 확인하기 위한 충분한 시간 대기
        tokio::time::sleep(Duration::from_millis(500)).await;

        // 첫 번째 세션이 종료되었는지 확인
        assert!(
            !state
                .session_manager
                .is_valid_session(PlayerType::Player1, session_id, Phase::Heartbeat)
                .await
        );

        // 두 번째 세션 생성이 가능한지 확인
        let mut ws2 = WebSocketTest::connect(url, cookie).await.unwrap();
        info!("두 번째 WebSocket 연결 생성");

        let new_session_id = ws2
            .expect_message(|msg: ConnectionMessage| msg.session_id)
            .await;
        info!("두 번째 세션 ID: {}", new_session_id);

        // 새 세션 ID가 다른지 확인
        assert_ne!(session_id.to_string(), new_session_id);

        // 두 번째 세션이 활성화되었는지 확인
        assert!(
            state
                .session_manager
                .is_valid_session(
                    PlayerType::Player1,
                    new_session_id.parse::<Uuid>().unwrap(),
                    Phase::Heartbeat
                )
                .await
        );
    }

    #[actix_web::test]
    async fn test_heartbeat_ping_pong() {
        // 서버 인스턴스 생성
        let (addr, state, _) = spawn_server().await;
        let player_type = PlayerType::Player1.as_str();

        {
            state
                .game
                .lock()
                .await
                .get_phase_state_mut()
                .set_phase(Phase::Heartbeat);
        }

        // 하트비트 연결 URL 및 쿠키 설정
        let url = format!("ws://{}/heartbeat", addr);
        let cookie = format!("user_id={}; game_step={}", player_type, "heartbeat");

        // 커스텀 WebSocket 클라이언트 생성
        let request = Request::builder()
            .uri(&url)
            .header("Cookie", cookie)
            .header("Host", url.split('/').nth(2).unwrap_or("localhost"))
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Version", "13")
            .body(())
            .unwrap();

        let (mut stream, response) = connect_async(request).await.unwrap();
        assert_eq!(
            response.status(),
            tungstenite::http::StatusCode::SWITCHING_PROTOCOLS
        );
        info!("타임아웃 테스트용 WebSocket 연결 성공");

        // ping 메시지 수신 응답
        match stream.next().await {
            Some(Ok(Message::Ping(_))) => {
                info!("Ping 메시지 수신됨");
                // Pong으로 응답
                info!("Pong 메시지 전송 시도");
                stream
                    .send(Message::Pong(Vec::new()))
                    .await
                    .expect("Failed to send pong message");
            }
            Some(Ok(msg)) => info!("Ping 아닌 다른 메시지 수신됨: {:?}", msg),
            Some(Err(e)) => panic!("메시지 수신 오류: {:?}", e),
            None => panic!("Ping 메시지 대기 중 타임아웃"),
        }

        match stream.next().await {
            Some(Ok(Message::Ping(_))) => info!("Ping 메시지 수신됨"),
            Some(Ok(_)) => {
                panic!("Ping 아닌 다른 메시지 수신됨");
            }
            Some(Err(e)) => info!("연결 오류 발생: {:?}", e),
            None => panic!("서버가 연결을 종료함"),
        }
    }
}

#[cfg(test)]
pub mod draw {
    use std::{collections::HashSet, net::SocketAddr};

    use card_game::{
        card::{cards::CardVecExt, types::PlayerType},
        enums::HAND_ZONE_SIZE,
        exception::*,
        game::phase::Phase,
        server::jsons::draw,
        test::{spawn_server, RequestTest},
        zone::zone::Zone,
    };
    use uuid::Uuid;

    #[actix_web::test]
    async fn test_draw_concurrency() {
        // 서버 설정
        let (addr, state, _) = spawn_server().await;

        {
            state
                .game
                .lock()
                .await
                .get_phase_state_mut()
                .set_phase(Phase::DrawPhase);
        }

        // 플레이어별 쿠키 설정
        let player1_cookie = format!(
            "user_id={}; game_step=drawphase",
            PlayerType::Player1.as_str()
        );
        let player2_cookie = format!(
            "user_id={}; game_step=drawphase",
            PlayerType::Player2.as_str()
        );

        // 두 플레이어가 동시에 드로우하는 테스트 함수
        async fn draw_card(addr: SocketAddr, cookie: String) -> (PlayerType, Uuid) {
            let mut response = RequestTest::connect("draw_phase", addr, cookie.clone())
                .await
                .expect("Failed to connect");

            // 플레이어 타입 확인
            let player_type = if cookie.contains("player1") {
                PlayerType::Player1
            } else {
                PlayerType::Player2
            };

            // 드로우된 카드 UUID 반환
            (player_type, response.expect_draw_card())
        }

        // 두 요청을 동시에 실행
        let mut tasks = Vec::new();

        // 각 플레이어가 5번씩 요청을 보냄
        let addr_clone = addr.clone();
        let p1_cookie = player1_cookie.clone();
        let p2_cookie = player2_cookie.clone();

        // Player1 태스크
        let task1 = tokio::spawn(async move { draw_card(addr_clone.clone(), p1_cookie).await });
        tasks.push(task1);

        // Player2 태스크
        let task2 = tokio::spawn(async move { draw_card(addr_clone, p2_cookie).await });
        tasks.push(task2);

        // 모든 태스크가 완료될 때까지 기다림
        let results = futures::future::join_all(tasks).await;

        // 결과 검증
        let mut player1_cards = Vec::new();
        let mut player2_cards = Vec::new();

        for result in results {
            // 각 태스크의 결과 확인
            match result {
                Ok((player_type, card_uuid)) => {
                    if player_type == PlayerType::Player1 {
                        player1_cards.push(card_uuid);
                    } else {
                        player2_cards.push(card_uuid);
                    }
                }
                Err(e) => panic!("Task failed: {:?}", e),
            }
        }

        // 두 플레이어가 모두 카드를 받았는지 확인
        assert_eq!(player1_cards.len(), 1);
        assert_eq!(player2_cards.len(), 1);

        // 각 플레이어가 받은 카드가 중복되지 않는지 확인
        let all_cards: HashSet<_> = player1_cards.iter().chain(player2_cards.iter()).collect();
        assert_eq!(all_cards.len(), player1_cards.len() + player2_cards.len());

        // 게임 상태 검증
        {
            let game = state.game.lock().await;

            // Player1의 모든 카드가 핸드에 있는지 확인
            let player1 = game.get_player_by_type(PlayerType::Player1).get();
            for &uuid in &player1_cards {
                assert!(player1.get_hand().get_cards().contains_uuid(uuid));
                assert!(!player1.get_deck().get_cards().contains_uuid(uuid));
            }

            // Player2의 모든 카드가 핸드에 있는지 확인
            let player2 = game.get_player_by_type(PlayerType::Player2).get();
            for &uuid in &player2_cards {
                assert!(player2.get_hand().get_cards().contains_uuid(uuid));
                assert!(!player2.get_deck().get_cards().contains_uuid(uuid));
            }
        }

        println!("플레이어1 드로우 카드: {:?}", player1_cards);
        println!("플레이어2 드로우 카드: {:?}", player2_cards);
    }
    #[actix_web::test]
    async fn test_draw_hand_is_full() {
        let (addr, state, _) = spawn_server().await;
        let player_type = PlayerType::Player1.as_str();
        let cookie = format!("user_id={}; game_step=drawphase", player_type);
        {
            state
                .game
                .lock()
                .await
                .get_phase_state_mut()
                .set_phase(Phase::DrawPhase);
        }

        // HAND_ZONE_SIZE + 1 회 반복하여 카드를 뽑는다.
        for _ in 0..HAND_ZONE_SIZE + 1 {
            let response = RequestTest::connect("draw_phase", addr, cookie.clone())
                .await
                .expect("Failed to connect");

            // 카드를 예상하되, 만약 parse error 발생 시, body.contains 을 통해 No Card Left ( 혹은 다른 오류 ) 오류 인지 확인함.
            let result = serde_json::from_str::<draw::ServerMessage>(&response.response.as_str());

            // Draw 메시지가 아닌 경우.
            // EXCEEDED_CARD_LIMIT 메시지가 포함되어 있는지 확인한다.
            if result.is_err() {
                assert!(response.response.contains(EXCEEDED_CARD_LIMIT));
            } else {
                let draw::ServerMessage::DrawAnswer(payload) = result.unwrap();
                let card_uuid = payload.cards.parse::<Uuid>().unwrap();
                // 검증 단계
                {
                    let game = state.game.lock().await;
                    let player = game.get_player_by_type(player_type).get();
                    let deck = player.get_deck();
                    if deck.get_cards().contains_uuid(card_uuid) {
                        panic!("Card is not removed from deck");
                    }

                    let hand = player.get_hand();
                    if !hand.get_cards().contains_uuid(card_uuid) {
                        panic!("Card is not added to hand");
                    }
                }

                // draw 상태 초기화
                {
                    let mut game = state.game.lock().await;

                    game.get_phase_state_mut()
                        .reset_player_completed(player_type.into());
                }
            }
        }
    }

    #[actix_web::test]
    async fn test_draw_re_entry() {
        let (addr, state, _) = spawn_server().await;
        let player_type = PlayerType::Player1.as_str();
        let cookie = format!("user_id={}; game_step=drawphase", player_type);
        {
            state
                .game
                .lock()
                .await
                .get_phase_state_mut()
                .set_phase(Phase::DrawPhase);
        }

        let _ = RequestTest::connect("draw_phase", addr, cookie.clone())
            .await
            .expect("Failed to connect");

        let rt = RequestTest::connect("draw_phase", addr, cookie.clone())
            .await
            .expect("Failed to connect");

        assert!(rt.response.contains(NOT_ALLOWED_RE_ENTRY));
    }

    #[actix_web::test]
    async fn test_draw_no_card_left() {
        let (addr, state, _) = spawn_server().await;
        let player_type = PlayerType::Player1.as_str();
        let cookie = format!("user_id={}; game_step=drawphase", player_type);
        {
            state
                .game
                .lock()
                .await
                .get_phase_state_mut()
                .set_phase(Phase::DrawPhase);
        }

        // 31회 반복하여 카드를 뽑는다.
        for _ in 0..31 {
            let response = RequestTest::connect("draw_phase", addr, cookie.clone())
                .await
                .expect("Failed to connect");

            // 카드를 예상하되, 만약 parse error 발생 시, body.contains 을 통해 No Card Left ( 혹은 다른 오류 ) 오류 인지 확인함.
            let result = serde_json::from_str::<draw::ServerMessage>(&response.response.as_str());

            // Draw 메시지가 아닌 경우.
            // No Card Left 메시지가 포함되어 있는지 확인한다.
            if result.is_err() {
                assert!(response.response.contains(NO_CARDS_LEFT));
            } else {
                let draw::ServerMessage::DrawAnswer(payload) = result.unwrap();
                let card_uuid = payload.cards.parse::<Uuid>().unwrap();
                // 검증 단계
                {
                    let game = state.game.lock().await;
                    let player = game.get_player_by_type(player_type).get();
                    let deck = player.get_deck();
                    if deck.get_cards().contains_uuid(card_uuid) {
                        panic!("Card is not removed from deck");
                    }

                    let hand = player.get_hand();
                    if !hand.get_cards().contains_uuid(card_uuid) {
                        panic!("Card is not added to hand");
                    }
                }

                // draw 상태 초기화 및 Hand 카드 삭제
                {
                    let mut game = state.game.lock().await;

                    game.get_phase_state_mut()
                        .reset_player_completed(player_type.into());

                    // HAND_ZONE_SIZE 를 임의로 수정할 수 없으므로
                    // Hand 카드를 삭제하는 방법으로
                    let card = game
                        .get_cards_by_uuid(card_uuid)
                        .clone()
                        .expect("Card not found");

                    game.get_player_by_type(player_type)
                        .get()
                        .get_hand_mut()
                        .remove_card(card)
                        .expect("Failed to remove card");
                }
            }
        }
    }

    #[actix_web::test]
    async fn test_draw_card() {
        let (addr, state, _) = spawn_server().await;
        let player_type = PlayerType::Player1.as_str();
        let cookie = format!("user_id={}; game_step=drawphase", player_type);
        {
            state
                .game
                .lock()
                .await
                .get_phase_state_mut()
                .set_phase(Phase::DrawPhase);
        }

        let mut response = RequestTest::connect("draw_phase", addr, cookie)
            .await
            .expect("Failed to connect");

        let card_uuid = response.expect_draw_card();

        // 검증 단계
        {
            let game = state.game.lock().await;
            let player = game.get_player_by_type(player_type).get();
            let deck = player.get_deck();
            if deck.get_cards().contains_uuid(card_uuid) {
                panic!("Card is not removed from deck");
            }

            let hand = player.get_hand();
            if !hand.get_cards().contains_uuid(card_uuid) {
                panic!("Card is not added to hand");
            }
        }
    }

    #[actix_web::test]
    async fn test_draw_wrong_phase() {
        let (addr, _, _) = spawn_server().await;
        let player_type = PlayerType::Player1.as_str();
        let cookie = format!("user_id={}; game_step=drawphase", player_type);

        let response = RequestTest::connect("draw_phase", addr, cookie)
            .await
            .expect("Failed to connect");

        assert!(response.response.contains(WRONG_PHASE));
    }
}

#[cfg(test)]
pub mod mulligan {
    use actix_web::{dev::ServerHandle, web::Data};
    use async_tungstenite::tungstenite::Message;
    use card_game::{
        card::types::PlayerType,
        enums::COUNT_OF_MULLIGAN_CARDS,
        exception::*,
        server::types::ServerState,
        test::{spawn_server, verify_mulligan_cards, WebSocketTest},
        zone::zone::Zone,
        VecUuidExt,
    };
    use core::panic;
    use once_cell::sync::Lazy;
    use rand::Rng;
    use serde_json::json;
    use std::{net::SocketAddr, time::Duration};
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

    /// 잘못된 mulligan 시나리오에 대해 서버가 반환한 에러 메시지를 리턴합니다.
    async fn test_mulligan_invalid_scenario(json_payload: serde_json::Value) -> String {
        let (addr, _, _) = spawn_server().await;

        // WebSocketTest 객체를 사용하여 훨씬 더 간결한 코드 작성
        let url = format!("ws://{}/mulligan_phase", addr);
        let cookie = format!(
            "user_id={}; game_step={}",
            PlayerType::Player1.as_str(),
            "mulligan"
        );

        let mut ws = WebSocketTest::connect(url, cookie).await.unwrap();

        // 초기 카드 받기
        let _ = ws.expect_mulligan_deal().await;

        // 에러 발생시키는 메시지 전송
        ws.send(Message::Text(json_payload.to_string()))
            .await
            .expect("Failed to send message");

        // 에러 응답 기다리기
        ws.expect_error().await
    }

    #[actix_web::test]
    async fn test_mulligan_invalid_player() {
        let json = json!({
            "action": "reroll-request",
            "payload": {
                "player": "wrong-player",
                "cards": []
            }
        });
        let error = test_mulligan_invalid_scenario(json).await;
        assert_eq!(error, INVALID_PLAYER);
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
        assert_eq!(error, INVALID_APPROACH);
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
        assert_eq!(error, INVALID_CARDS);
    }

    #[actix_web::test]
    async fn test_mulligan_wrong_phase() {
        // HTTP 기반 잘못된 접근은 WebSocket 테스트가 아니므로 따로 검증합니다.
        let (addr, _, _) = spawn_server().await;
        let player_type = PlayerType::Player1.as_str();

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://{}/mulligan_phase", addr))
            .header("Cookie", format!("user_id={}; game_step=draw", player_type))
            .send()
            .await
            .expect("request failed");

        let status = response.status();
        let body = response.text().await.expect("Failed to read response body");

        assert_eq!(status.as_u16(), 500);

        assert!(body.contains(WRONG_PHASE));
    }

    #[actix_web::test]
    async fn test_mulligan_is_session_close() {
        panic!("Not implemented yet");
    }

    // 이 테스트 뭔가 문제가 많음
    // TODO: 테스트 마저 작성해야함.
    #[actix_web::test]
    async fn test_mulligan_already_ready() {
        // HTTP 기반 잘못된 접근은 WebSocket 테스트가 아니므로 따로 검증합니다.
        let (addr, _, _) = spawn_server().await;
        let player_type = PlayerType::Player1.as_str();

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://{}/mulligan_phase", addr))
            .header("Cookie", format!("user_id={}; game_step=draw", player_type))
            .send()
            .await
            .expect("request failed");

        let status = response.status();
        let body = response.text().await.expect("Failed to read response body");

        assert_eq!(status.as_u16(), 500);

        assert!(body.contains(ALREADY_READY));
    }

    #[actix_web::test]
    #[should_panic]
    async fn test_mulligan_re_entry() {
        let (addr, _, _) = spawn_server().await;

        let url = format!("ws://{}/mulligan_phase", addr);
        let cookie = format!(
            "user_id={}; game_step={}",
            PlayerType::Player1.as_str(),
            "mulligan"
        );

        let mut ws = WebSocketTest::connect(url.clone(), cookie.clone())
            .await
            .unwrap();

        // 초기 카드 받기
        let _ = ws.expect_mulligan_deal().await;

        // 엔드포인트 재진입
        let _ = WebSocketTest::connect(url, cookie)
            .await
            .expect("Re-entry is not allowed");
    }

    #[actix_web::test]
    async fn test_mulligan_reroll_restore_variants() -> std::io::Result<()> {
        enum MulliganAction {
            RerollRequest,
            Complete,
        }

        impl MulliganAction {
            pub fn as_str(&self) -> &str {
                match self {
                    MulliganAction::RerollRequest => "reroll-request",
                    MulliganAction::Complete => "complete",
                }
            }
        }

        async fn run_mulligan_case(reroll_count: usize) -> std::io::Result<()> {
            async fn run_mulligan_case_each_player(
                reroll_count: usize,
                player_type: &str,
                addr: SocketAddr,
                server_state: Data<ServerState>,
            ) -> std::io::Result<()> {
                let url = format!("ws://{}/mulligan_phase", addr);
                let cookie = format!("user_id={}; game_step={}", player_type, "mulligan");
                let mut ws = WebSocketTest::connect(url, cookie).await.unwrap();

                let mut deal_cards = ws.expect_mulligan_deal().await;

                // 나머지 테스트 로직...
                deal_cards.truncate(reroll_count);

                // TODO: 모든 가능한 수에 대해서 static 하게 테스트 하는게 맞는데
                // 일단 킵
                let action = if rand::thread_rng().gen_bool(0.5) {
                    MulliganAction::RerollRequest
                } else {
                    MulliganAction::Complete
                };

                println!("Player: {}, Action: {}", player_type, action.as_str());

                let json = json!({
                    "action": action.as_str(),
                    "payload": {
                        "player": player_type,
                        "cards": deal_cards.to_vec_string()
                    }
                });

                ws.send(Message::Text(json.to_string()))
                    .await
                    .expect("Failed to send message");

                if let MulliganAction::RerollRequest = action {
                    let cards = ws.expect_mulligan_answer().await;
                    // RerollRequest일 때는 이전 카드들이 덱에 복원되었는지 검증
                    verify_mulligan_cards(
                        &server_state,
                        player_type.into(),
                        &cards,
                        Some(&deal_cards),
                        reroll_count,
                    );
                } else {
                    let cards = ws.expect_mulligan_complete().await;
                    // Complete일 때는 복원 검증 없이 뽑은 카드만 검증
                    verify_mulligan_cards(
                        &server_state,
                        player_type.into(),
                        &cards,
                        None,
                        reroll_count,
                    );
                };
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

            // 각 플레이어의 손패 갯수를 검증합니다.
            // 각 플레이어의 손패 갯수는 5개씩이어야 합니다.
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

            // 각 플레이어가 준비 상태인지 검증합니다.
            // 각 플레이어가 준비 상태가 되었다면 테스트를 통과합니다.
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
