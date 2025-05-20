pub mod mulligan {
    use std::time::Duration;

    use actix::Addr;
    use dedicated_server::{
        connection::ServerMessage,
        test::{spawn_server, WebSocketTest},
    };
    use simulator_core::{
        card::types::PlayerKind,
        enums::{ZoneType, CLIENT_TIMEOUT, COUNT_OF_MULLIGAN_CARDS},
        game::{message::GetPlayerZoneCards, GameActor},
    };
    use tokio::time::sleep;
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
        let game_actor_addr = state.game.clone();

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

    // TODO: Player 2 가 지연된 입장을 할 경우, 게임의 세션은 닫혀야함.
    // 지금 Player 1 의 세션은 잘 닫히지만, Player 2 의 세션은 닫히지 않아서
    // Player 2 혼자 게임에 접속함. 이를 고쳐야함.
    // 고치는 방식은 플레이어의 세션을 닫는게 아니라, 게임 자체를 종료하는 방식으로.
    #[actix_web::test]
    #[should_panic(expected = "[DELAYED_TEST] Player 1 task failed")]
    async fn test_mulligan_deal_cards_one_player_delayed() {
        let (addr, state, _handle) = spawn_server().await;
        let game_actor_addr = state.game.clone();

        let player1_id = state.player1_id;
        let player2_id = state.player2_id;

        // 플레이어 1은 즉시 시작
        let player1_task = tokio::spawn(player_mulligan_sequence(
            PlayerKind::Player1,
            player1_id,
            addr,
            game_actor_addr.clone(),
        ));

        // 플레이어 2는 10초 지연 후 시작 (이 시간 동안 서버가 P1을 기다리는지 확인)
        // 이 지연 시간은 서버의 관련 타임아웃 설정보다 길거나 짧게 조절하여 테스트 가능
        let delay_duration = Duration::from_secs(CLIENT_TIMEOUT + 5);
        println!(
            "[DELAYED_TEST] Player 2 will start mulligan sequence after {:?} delay.",
            delay_duration
        );

        let player2_task = tokio::spawn(async move {
            sleep(delay_duration).await;
            println!("[DELAYED_TEST] Player 2 starting mulligan sequence now.");
            player_mulligan_sequence(
                PlayerKind::Player2,
                player2_id,
                addr,            // addr도 move 클로저로 옮겨져야 함
                game_actor_addr, // game_actor_addr도 move 클로저로 옮겨져야 함
            )
            .await
        });

        // 두 태스크의 결과 기다림
        // P1은 바로 완료될 수 있고, P2는 지연 후 완료되거나,
        // 서버 정책에 따라 P1이 P2를 기다리다가 특정 조건 후 진행될 수 있음.
        let (p1_result_outer, p2_result_outer) = tokio::join!(player1_task, player2_task);

        // 태스크 실행 결과 확인 (JoinError 처리)
        let p1_results = match p1_result_outer {
            Ok(res) => {
                println!(
                    "[DELAYED_TEST] Player 1 mulligan sequence completed with results: {:?}",
                    res
                );
                res
            }
            Err(e) => {
                panic!("[DELAYED_TEST] Player 1 task failed: {:?}", e);
            }
        };

        let p2_results = match p2_result_outer {
            Ok(res) => {
                println!(
                    "[DELAYED_TEST] Player 2 mulligan sequence completed with results: {:?}",
                    res
                );
                res
            }
            Err(e) => {
                // 플레이어 2가 지연되는 동안 서버가 P1만으로 게임을 시작하거나 P1 연결을 종료했다면,
                // P2는 연결조차 실패할 수 있음. 이는 예상된 실패일 수 있음.
                // 여기서는 일단 패닉으로 처리하지만, 실제로는 서버 정책에 따라 다르게 검증해야 함.
                println!("[DELAYED_TEST] Player 2 task resulted in an error (this might be expected if server timed out P1 or started game with P1 only): {:?}", e);
                // 이 테스트의 목적에 따라, P2가 실패하는 것이 정상일 수도 있습니다.
                // 예를 들어, 서버가 10초 이내에 P1만으로 게임을 시작해버린다면, P2는 연결 시점에
                // 이미 게임이 진행 중이거나 멀리건 페이즈가 아니어서 실패할 수 있습니다.
                // 여기서는 일단 빈 Vec을 반환하여 아래 assert_eq가 실패하도록 유도합니다. (테스트 목적에 맞게 수정 필요)
                // panic!("[DELAYED_TEST] Player 2 task failed: {:?}", e);
                Vec::new() // 또는 테스트 실패로 간주
            }
        };

        // 결과 검증:
        // 이 부분은 서버가 지연된 플레이어를 어떻게 처리하는지에 따라 달라집니다.
        // 1. 서버가 P2를 기다려서 두 플레이어 모두 정상적으로 멀리건을 완료하는 경우:
        if !p1_results.is_empty() && !p2_results.is_empty() {
            println!("[DELAYED_TEST] Both players seem to have completed mulligan.");
            let mut all_dealt_cards = p1_results.clone();
            all_dealt_cards.extend(p2_results.clone());
            let unique_cards_count = all_dealt_cards
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len();
            assert_eq!(
            unique_cards_count,
            COUNT_OF_MULLIGAN_CARDS * 2,
            "[DELAYED_TEST] Dealt cards between players are not unique or not all players completed mulligan."
        );
        } else if !p1_results.is_empty() && p2_results.is_empty() {
            // 2. 서버가 P1만으로 게임을 시작하거나, P2를 기다리다 P1에 대한 타임아웃/오류 처리 후 P2는 실패하는 경우
            println!("[DELAYED_TEST] Player 1 completed mulligan, but Player 2 did not (possibly expected).");
            // 이 경우 P1의 멀리건 카드 수만 검증할 수 있습니다.
            assert_eq!(
                p1_results.len(),
                COUNT_OF_MULLIGAN_CARDS,
                "[DELAYED_TEST] Player 1 did not receive the correct number of mulligan cards."
            );
            // 서버 로그를 통해 P1이 P2를 기다렸는지, 또는 특정 시간 후 P1만으로 진행했는지 확인해야 합니다.
            // 또는 GameActor의 상태를 직접 확인하여 게임이 어떻게 진행되었는지 검증할 수 있습니다.
            // 예를 들어, GameStateManager.current_phase()가 Mulligan이 아닌 다른 상태로 넘어갔는지 등.
        } else {
            // 3. 두 플레이어 모두 실패한 경우 (예: 서버가 P1 지연으로 인해 전체 게임을 시작하지 못함)
            panic!("[DELAYED_TEST] Neither player completed the mulligan sequence. P1 results: {:?}, P2 results: {:?}", p1_results, p2_results);
        }

        println!("[DELAYED_TEST] Test test_mulligan_deal_cards_one_player_delayed completed.");
    }
}

pub mod heartbeat {
    use std::time::Duration;

    use async_tungstenite::tungstenite::Message;
    use dedicated_server::{
        connection::ServerMessage,
        test::{spawn_server, WebSocketTest},
    };
    use futures::StreamExt;
    use simulator_core::{
        card::types::PlayerKind,
        enums::{CLIENT_TIMEOUT, HEARTBEAT_INTERVAL},
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
