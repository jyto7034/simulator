pub mod mulligan {
    use actix::Addr;
    use simulator_core::{
        card::types::PlayerKind,
        enums::{ZoneType, COUNT_OF_MULLIGAN_CARDS},
        game::{message::GetPlayerZoneCards, GameActor},
        server::actor::ServerMessage,
        test::{spawn_server, WebSocketTest},
    };
    use uuid::Uuid;

    // 플레이어별 테스트 로직을 위한 헬퍼 함수
    async fn player_mulligan_sequence(
        player_kind: PlayerKind,
        player_id: Uuid,
        addr: std::net::SocketAddr,
        game_actor_addr: Addr<GameActor>, // GameActor 주소 전달
    ) -> Vec<Uuid> {
        let player_kind_str = player_kind.as_str();
        let url = format!("ws://{}/game", addr);
        let cookie = format!("user_id={}", player_id);
        let mut ws = WebSocketTest::connect(url, cookie).await.unwrap();

        // 1. 초기 HeartbeatConnected 메시지 수신
        println!(
            "[{}] Waiting for initial HeartbeatConnected message...",
            player_kind_str
        );
        let initial_extractor = |message: ServerMessage| match message {
            ServerMessage::HeartbeatConnected { player, session_id } => {
                assert_eq!(player, player_kind_str);
                assert!(!session_id.is_nil());
                println!(
                    "[{}] Initial HeartbeatConnected received. Session ID: {}",
                    player_kind_str, session_id
                );
            }
            _ => panic!(
                "[{}] Expected HeartbeatConnected as the first message, but got {:?}",
                player_kind_str, message
            ),
        };
        ws.expect_message(initial_extractor).await;

        // 2. MulliganDealCards 메시지 수신
        println!(
            "[{}] Waiting for MulliganDealCards message...",
            player_kind_str
        );
        let mulligan_extractor = |message: ServerMessage| -> Vec<Uuid> {
            match message {
                ServerMessage::MulliganDealCards { player, cards } => {
                    // 중요: MulliganDealCards 메시지의 player 필드가 이 카드를 받는 플레이어를 지칭해야 함
                    assert_eq!(
                        player, player_kind_str,
                        "[{}] Mulligan cards for wrong player",
                        player_kind_str
                    );
                    assert_eq!(
                        cards.len(),
                        COUNT_OF_MULLIGAN_CARDS,
                        "[{}] Incorrect number of mulligan cards",
                        player_kind_str
                    );
                    for card_uuid in &cards {
                        assert!(
                            !card_uuid.is_nil(),
                            "[{}] Nil UUID in mulligan cards",
                            player_kind_str
                        );
                    }
                    println!(
                        "[{}] MulliganDealCards received with {} cards.",
                        player_kind_str,
                        cards.len()
                    );
                    cards
                }
                // 이 시점에는 다른 메시지가 오면 안 됨 (HeartbeatConnected는 이미 처리됨)
                _ => panic!(
                    "[{}] Expected MulliganDealCards message, but got {:?}",
                    player_kind_str, message
                ),
            }
        };
        let dealt_cards = ws.expect_message(mulligan_extractor).await;

        // 3. 받은 카드가 덱에 없는지 확인 (GameActor에게 요청)
        println!(
            "[{}] Verifying dealt cards are not in deck...",
            player_kind_str
        );
        let deck_cards_result = game_actor_addr
            .send(GetPlayerZoneCards {
                // GameActor는 Uuid로 플레이어를 식별하거나, PlayerKind를 Uuid로 변환할 수 있어야 함
                zone: ZoneType::Deck,
                player_type: player_kind,
            })
            .await;

        match deck_cards_result {
            Ok(deck_card_objects) => {
                let deck_uuids: Vec<Uuid> = deck_card_objects
                    .iter()
                    .map(|card| card.get_uuid())
                    .collect();
                for dealt_card_uuid in dealt_cards.iter() {
                    assert!(
                        !deck_uuids.contains(dealt_card_uuid),
                        "[{}] Deck should not contain card {} that was dealt in mulligan",
                        player_kind_str,
                        dealt_card_uuid
                    );
                }
                println!(
                    "[{}] Dealt cards correctly removed from deck.",
                    player_kind_str
                );
            }
            Err(e) => panic!(
                "[{}] GameActor returned error getting deck cards: {:?}",
                player_kind_str, e
            ),
        }

        // 멀리건 단계 완료를 위해 추가적인 메시지 전송/수신 로직이 필요할 수 있음
        // 예: ws.send(UserAction::CompleteMulligan).await;
        //     ws.expect_message(ServerMessage::MulliganPhaseEnd).await;

        dealt_cards // 받은 카드 목록 반환
    }

    #[actix_web::test]
    async fn test_mulligan_deal_cards_to_each_player_concurrently() {
        let (addr, state, _handle) = spawn_server().await;
        let game_actor_addr = state.game.clone(); // AppServerState에 Addr<GameActor> 필드 추가 가정

        let player1_id = state.player1_id;
        let player2_id = state.player2_id;

        // 두 플레이어의 멀리건 시퀀스를 병렬로 실행
        let (p1_results, p2_results) = tokio::join!(
            player_mulligan_sequence(
                PlayerKind::Player1,
                player1_id,
                addr,
                game_actor_addr.clone()
            ),
            player_mulligan_sequence(
                PlayerKind::Player2,
                player2_id,
                addr,
                game_actor_addr.clone()
            )
        );

        println!("Player 1 mulligan cards: {:?}", p1_results);
        println!("Player 2 mulligan cards: {:?}", p2_results);

        // 추가 검증: P1과 P2가 받은 카드가 서로 다른지 등
        let mut all_dealt_cards = p1_results.clone();
        all_dealt_cards.extend(p2_results.clone());
        let unique_cards_count = all_dealt_cards
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        assert_eq!(
            unique_cards_count,
            COUNT_OF_MULLIGAN_CARDS * 2,
            "Dealt cards between players are not unique."
        );

        println!("Test test_mulligan_deal_cards_to_each_player_concurrently completed.");
    }
}

pub mod heartbeat {
    use std::time::Duration;

    use async_tungstenite::tungstenite::Message;
    use futures::StreamExt;
    use simulator_core::{
        card::types::PlayerKind,
        enums::{CLIENT_TIMEOUT, HEARTBEAT_INTERVAL},
        server::actor::ServerMessage,
        test::{spawn_server, WebSocketTest},
    };
    use tokio::time::{sleep, timeout};
    use tracing::info;
    use uuid::Uuid;

    #[actix_web::test]
    async fn test_heartbeat_initialize_msg() {
        let (addr, state, _handle) = spawn_server().await;

        let player_type = PlayerKind::Player1.as_str();

        // 하트비트 연결 URL 및 쿠키 설정
        let url = format!("ws://{}/game", addr);
        let cookie = format!("user_id={}", state.player1_id);

        // 예시: 서버에 GET 요청 보내기
        let mut ws = WebSocketTest::connect(url, cookie).await.unwrap();

        let extractor = |message: ServerMessage| match message {
            ServerMessage::HeartbeatConnected { player, session_id } => {
                assert_eq!(player, player_type);
                assert!(!session_id.is_nil());
            }
            _ => panic!("Expected HeartbeatConnected message"),
        };
        ws.expect_message(extractor).await;
    }

    #[actix_web::test]
    async fn test_heartbeat_timeout() {
        let (addr, state, _handle) = spawn_server().await;

        let player1_id = state.player1_id;
        let player_kind_str = PlayerKind::Player1.as_str();

        let url = format!("ws://{}/game", addr);
        let cookie = format!("user_id={}", player1_id);

        let mut ws = WebSocketTest::connect(url, cookie).await.unwrap();

        // 1. 초기 연결 메시지 수신
        info!("[TEST] Waiting for initial connection message...");
        let initial_extractor = |message: ServerMessage| -> Uuid {
            match message {
                ServerMessage::HeartbeatConnected { player, session_id } => {
                    assert_eq!(player, player_kind_str);
                    assert!(!session_id.is_nil());
                    info!(
                        "[TEST] Initial connection message received. Session ID: {}",
                        session_id
                    );
                    session_id
                }
                _ => panic!("Expected HeartbeatConnected message"),
            }
        };
        if timeout(
            Duration::from_secs(HEARTBEAT_INTERVAL),
            ws.expect_message(initial_extractor),
        )
        .await
        .is_err()
        {
            panic!("Timeout waiting for initial HeartbeatConnected message.");
        }

        // 2. 충분히 기다림 (CLIENT_TIMEOUT + 여유)
        let wait_duration = Duration::from_secs(CLIENT_TIMEOUT + 5);
        info!(
            "[TEST] Waiting for {} seconds to see if server closes connection (should NOT close due to auto Pong)...",
            wait_duration.as_secs()
        );

        sleep(wait_duration).await;

        // 3. 연결이 살아있는지 송수신 테스트 (예: Ping 보내고 Pong 받기)
        info!("[TEST] Sending Ping to check if connection is alive...");
        let ping_data = b"test_ping".to_vec();
        let send_result = ws.send(Message::Ping(ping_data.clone())).await;

        let pong_received = match send_result {
            Ok(_) => {
                // Pong 응답을 기다림
                timeout(Duration::from_secs(2), async {
                    loop {
                        match ws.stream.next().await {
                            Some(Ok(Message::Pong(data))) if data == ping_data => break true,
                            Some(Ok(_)) => continue,
                            Some(Err(_)) | None => break false,
                        }
                    }
                })
                .await
                .unwrap_or(false)
            }
            Err(_) => false, // 이미 연결이 끊겼다면
        };

        if pong_received {
            panic!("[TEST] Test Failed: Connection is still alive after client timeout (should be closed).");
        } else {
            info!("[TEST] Test Success: Connection is closed after client timeout (as expected).");
        }
    }

    #[actix_web::test]
    async fn test_heartbeat_ping_pong_once() {
        let (addr, state, _handle) = spawn_server().await;

        let player1_id = state.player1_id;
        let player_kind_str = PlayerKind::Player1.as_str();

        let url = format!("ws://{}/game", addr);
        let cookie = format!("user_id={}", player1_id);

        let mut ws = WebSocketTest::connect(url, cookie).await.unwrap();

        // 1. 초기 연결 메시지 수신 (선택 사항이지만, 이전 테스트에서 확인했으므로 여기서도 확인)
        info!("[TEST] Waiting for initial connection message...");
        let initial_extractor = |message: ServerMessage| -> Uuid {
            match message {
                ServerMessage::HeartbeatConnected { player, session_id } => {
                    assert_eq!(player, player_kind_str);
                    assert!(!session_id.is_nil());
                    info!(
                        "Initial connection message received. Session ID: {}",
                        session_id
                    );
                    session_id
                }
                _ => panic!("Expected HeartbeatConnected message first"),
            }
        };
        let _session_id = ws.expect_message(initial_extractor).await;

        // 2. 서버로부터 Ping 메시지 기다리기
        info!("[TEST] Waiting for Ping message...");
        let ping_received = timeout(Duration::from_secs(HEARTBEAT_INTERVAL * 2), async {
            loop {
                match ws.stream.next().await {
                    Some(Ok(Message::Ping(ping_data))) => {
                        info!("[TEST] Ping received!");
                        return Some(ping_data);
                    }
                    Some(Ok(Message::Text(text))) => {
                        info!(
                            "[TEST] Received unexpected Text while waiting for Ping: {}",
                            text
                        );
                        continue;
                    }
                    Some(Ok(Message::Pong(_))) => {
                        info!("[TEST] Received Pong while waiting for Ping, ignoring.");
                        continue;
                    }
                    Some(Ok(_)) => continue,
                    Some(Err(e)) => panic!("WebSocket error while waiting for Ping: {:?}", e),
                    None => panic!("WebSocket closed unexpectedly while waiting for Ping"),
                }
            }
        })
        .await;

        // 3. Ping 수신 및 Pong 전송 확인
        match ping_received {
            Ok(Some(ping_data)) => {
                // Ping을 받았으므로 Pong 전송
                info!("[TEST] Sending Pong response...");
                ws.send(Message::Pong(ping_data.clone()))
                    .await
                    .expect("Failed to send Pong");
                info!("[TEST] Pong sent.");

                // 잠시 대기하여 서버가 Pong을 처리하고 연결을 유지하는지 확인
                info!("[TEST] Waiting to see if connection is maintained...");
                sleep(Duration::from_secs(2)).await; // 짧은 시간 대기

                // 연결이 살아있는지 확인
                let test_msg = Message::Pong(ping_data);
                match ws.send(test_msg).await {
                    Ok(_) => info!("[TEST] Connection seems alive after Pong."),
                    Err(e) => panic!("Connection seems closed after Pong: {:?}", e),
                }
            }
            Ok(None) => {
                // 루프가 값을 반환하지 않고 종료된 경우
                panic!("Ping waiting loop exited unexpectedly without returning Ping data.");
            }
            Err(_) => {
                panic!(
                    "Timeout: Did not receive Ping within {} seconds.",
                    (HEARTBEAT_INTERVAL * 2)
                );
            }
        }

        // 테스트 종료 (정상적으로 연결 종료)
        info!("[TEST] Closing WebSocket connection.");
        ws.send(Message::Close(None)).await.ok(); // Close 메시지 전송 시도
    }
}
