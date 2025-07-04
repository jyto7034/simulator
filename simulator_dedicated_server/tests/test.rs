pub mod mulligan {
    use std::{collections::HashSet, sync::Arc, time::Duration};

    use actix::Addr;
    use async_tungstenite::tungstenite::Message;
    use dedicated_server::{
        connection::{ServerErrorCode, ServerMessage},
        enums::HEARTBEAT_INTERVAL,
        test::{spawn_server, WebSocketTest},
    };
    use futures::StreamExt;
    use simulator_core::{
        card::types::PlayerKind,
        enums::{ZoneType, CLIENT_TIMEOUT, COUNT_OF_MULLIGAN_CARDS},
        game::{msg::zones::GetPlayerZoneCards, GameActor},
    };
    use tokio::{
        join,
        sync::Barrier,
        time::{sleep, timeout},
    };
    use tracing::info;
    use uuid::Uuid;

    // 플레이어별 테스트 로직을 위한 헬퍼 함수
    async fn connect_and_register_player(
        player_kind: PlayerKind,
        player_id: Uuid,
        addr: std::net::SocketAddr,
    ) -> WebSocketTest {
        let player_kind_str = player_kind.as_str();
        let url = format!("ws://{}/game", addr);
        let cookie = format!("user_id={}", player_id);
        let mut ws = WebSocketTest::connect(url, cookie).await.unwrap();

        // 1. 서버가 연결을 등록하도록 수동으로 첫 Pong을 보냅니다.
        ws.send(Message::Pong(vec![]))
            .await
            .expect("Failed to send initial Pong");
        info!(
            "[{}] Manually sent initial Pong to trigger registration.",
            player_kind_str
        );

        // 2. HeartbeatConnected 메시지를 수신하여 등록 완료를 확인합니다.
        //    이 함수는 한 명만 연결할 때도, 두 명이 동시에 연결할 때도 사용되므로,
        //    다른 메시지(MulliganDealCards)가 먼저 올 수도 있습니다.
        //    따라서 expect_message 루프가 HeartbeatConnected를 찾을 때까지 다른 메시지를 무시해야 합니다.
        ws.expect_message(|message: ServerMessage| {
            match message {
                ServerMessage::HeartbeatConnected { player, session_id } => {
                    assert_eq!(player, player_kind_str);
                    assert_eq!(session_id, player_id);
                    info!(
                        "[{}] Initial HeartbeatConnected received. Session ID: {}",
                        player_kind_str, session_id
                    );
                }
                // MulliganDealCards는 이 함수에서 처리하지 않으므로, 만약 먼저 온다면 패닉을 일으켜 로직 오류를 알립니다.
                // 혹은 테스트 시나리오에 따라 무시하고 계속 HeartbeatConnected를 기다릴 수도 있습니다.
                // 여기서는 일단 패닉 처리하여, 호출 순서가 잘못되었음을 명확히 합니다.
                _ => panic!(
                    "[{}] Expected HeartbeatConnected, but got {:?} instead.",
                    player_kind_str, message
                ),
            }
        })
        .await;

        ws
    }

    async fn connect_and_send_pong(
        player_kind: PlayerKind,
        player_id: Uuid,
        addr: std::net::SocketAddr,
    ) -> WebSocketTest {
        let player_kind_str = player_kind.as_str();
        let url = format!("ws://{}/game", addr);
        let cookie = format!("user_id={}", player_id);
        let mut ws = WebSocketTest::connect(url, cookie).await.unwrap();

        // 서버가 연결을 등록하도록 수동으로 첫 Pong을 보냅니다.
        ws.send(Message::Pong(vec![]))
            .await
            .expect("Failed to send initial Pong");
        info!("[{}] Connected and sent initial Pong.", player_kind_str);

        ws
    }

    /// WebSocketTest 인스턴스를 통해 MulliganDealCards 메시지를 기다리고 검증합니다.
    async fn expect_mulligan_cards(
        ws: &mut WebSocketTest,
        player_kind: PlayerKind,
        game_actor_addr: Addr<GameActor>,
    ) -> Vec<Uuid> {
        let player_kind_str = player_kind.as_str();
        info!(
            "[{}] Waiting for MulliganDealCards message...",
            player_kind_str
        );

        let dealt_cards = ws
            .expect_message(|message: ServerMessage| -> Vec<Uuid> {
                match message {
                    ServerMessage::MulliganDealCards { player, cards } => {
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
                        assert!(
                            cards.iter().all(|c| !c.is_nil()),
                            "[{}] Nil UUID in mulligan cards",
                            player_kind_str
                        );
                        info!(
                            "[{}] MulliganDealCards received with {} cards.",
                            player_kind_str,
                            cards.len()
                        );
                        cards
                    }
                    _ => panic!(
                        "[{}] Expected MulliganDealCards, but got {:?}",
                        player_kind_str, message
                    ),
                }
            })
            .await;

        // 받은 카드가 덱에 없는지 확인
        info!(
            "[{}] Verifying dealt cards are not in deck...",
            player_kind_str
        );
        let deck_cards_result = game_actor_addr
            .send(GetPlayerZoneCards {
                zone: ZoneType::Deck,
                player_type: player_kind,
            })
            .await
            .expect("Mailbox error getting deck cards");

        let deck_uuids: Vec<Uuid> = deck_cards_result
            .iter()
            .map(|card| card.get_uuid())
            .collect();
        for dealt_card_uuid in &dealt_cards {
            assert!(
                !deck_uuids.contains(dealt_card_uuid),
                "[{}] Deck should not contain card {} that was dealt in mulligan",
                player_kind_str,
                dealt_card_uuid
            );
        }
        info!(
            "[{}] Dealt cards correctly removed from deck.",
            player_kind_str
        );

        dealt_cards
    }

    #[actix_web::test]
    async fn test_mulligan_deal_cards_to_each_player_concurrently() {
        let (addr, state, _handle) = spawn_server().await;
        let game_actor_addr = state.game.clone();
        let player1_id = state.player1_id;
        let player2_id = state.player2_id;

        // 1. 두 플레이어를 병렬로 연결하고 등록합니다.
        let (p1_connect_result, p2_connect_result) = tokio::join!(
            connect_and_register_player(PlayerKind::Player1, player1_id, addr),
            connect_and_register_player(PlayerKind::Player2, player2_id, addr)
        );

        let mut ws1 = p1_connect_result;
        let mut ws2 = p2_connect_result;

        // 2. 두 플레이어가 모두 등록되었으므로, 이제 서버는 멀리건 카드를 보내야 합니다.
        let (p1_cards_result, p2_cards_result) = tokio::join!(
            expect_mulligan_cards(&mut ws1, PlayerKind::Player1, game_actor_addr.clone()),
            expect_mulligan_cards(&mut ws2, PlayerKind::Player2, game_actor_addr.clone())
        );

        // 3. 추가 검증
        let mut all_dealt_cards = p1_cards_result;
        all_dealt_cards.extend(p2_cards_result);
        let unique_cards_count = all_dealt_cards.iter().collect::<HashSet<_>>().len();
        assert_eq!(
            unique_cards_count,
            COUNT_OF_MULLIGAN_CARDS * 2,
            "Dealt cards between players are not unique."
        );
        println!("Test test_mulligan_deal_cards_to_each_player_concurrently completed.");
    }

    #[actix_web::test]
    async fn test_mulligan_deal_cards_one_player_delayed() {
        let (addr, state, _handle) = spawn_server().await;

        let player1_id = state.player1_id;
        let player2_id = state.player2_id;

        // --- Player 1's Task ---
        // P1은 즉시 연결하고, 서버가 타임아웃으로 자신을 끊을 때까지 기다립니다.
        let p1_task = tokio::spawn(async move {
            let mut ws1 = connect_and_register_player(PlayerKind::Player1, player1_id, addr).await;
            info!("[DELAYED_TEST] P1 connected and registered. Now waiting for server to close connection due to P2 timeout.");

            // 서버가 P2를 기다리다 타임아웃되어 연결을 끊을 때까지 대기
            // GameActor의 Abort -> ConnectionActor의 stop 흐름
            let close_future = async {
                loop {
                    if let Some(msg) = ws1.stream.next().await {
                        match msg {
                            Ok(Message::Close(_)) => {
                                info!("[DELAYED_TEST] P1 connection closed by server as expected.");
                                break;
                            }
                            Ok(Message::Ping(_)) => continue, // Ping은 무시
                            _ => {}                           // 다른 메시지도 무시
                        }
                    } else {
                        info!("[DELAYED_TEST] P1 stream ended, which is also expected.");
                        break;
                    }
                }
            };

            // CLIENT_TIMEOUT + 약간의 여유시간을 기다립니다.
            match timeout(Duration::from_secs(CLIENT_TIMEOUT + 5), close_future).await {
                Ok(_) => info!("[DELAYED_TEST] P1 task finished successfully."),
                Err(_) => panic!("[DELAYED_TEST] P1 connection was not closed by server within the timeout period."),
            }
        });

        // --- Player 2's Task ---
        // P2는 타임아웃이 발생한 *이후에* 연결을 시도합니다. 이 시도는 실패해야 합니다.
        let p2_task = tokio::spawn(async move {
            // P1이 타임아웃되기에 충분한 시간을 기다립니다.
            sleep(Duration::from_secs(CLIENT_TIMEOUT + 2)).await;
            info!("[DELAYED_TEST] P2 starting connection attempt after timeout.");

            let url = format!("ws://{}/game", addr);
            let cookie = format!("user_id={}", player2_id);
            let reconnect_result = WebSocketTest::connect(url, cookie).await;

            // GameActor가 이미 종료되었으므로 연결 시도는 실패해야 합니다.
            assert!(
                reconnect_result.is_err(),
                "[DELAYED_TEST] P2 connection should have failed, but it succeeded."
            );
            info!("[DELAYED_TEST] P2 connection attempt failed as expected.");
        });

        // 두 태스크의 결과를 기다립니다.
        let (p1_result, p2_result) = tokio::join!(p1_task, p2_task);

        // 태스크 자체가 패닉하지 않았는지 확인
        p1_result.expect("Player 1 task panicked");
        p2_result.expect("Player 2 task panicked");

        println!("[DELAYED_TEST] Test test_mulligan_deal_cards_one_player_delayed completed successfully.");
    }

    #[actix_web::test]
    async fn test_mulligan_deal_cards_on_simultaneous_connection_with_barrier() {
        let (addr, state, _handle) = spawn_server().await;
        let player1_id = state.player1_id;
        let player2_id = state.player2_id;
        let barrier = Arc::new(Barrier::new(2));

        // 1. Act (Phase 1): 동시에 연결 및 Pong 전송
        let p1_barrier = barrier.clone();
        let p1_task = tokio::spawn(async move {
            p1_barrier.wait().await;
            connect_and_send_pong(PlayerKind::Player1, player1_id, addr).await
        });

        let p2_barrier = barrier.clone();
        let p2_task = tokio::spawn(async move {
            p2_barrier.wait().await;
            connect_and_send_pong(PlayerKind::Player2, player2_id, addr).await
        });

        let (p1_result, p2_result) = join!(p1_task, p2_task);
        let mut ws1 = p1_result.unwrap();
        let mut ws2 = p2_result.unwrap();

        info!(
            "[BARRIER_TEST] Both players connected and sent Pong. Now expecting initial messages."
        );

        // 2. Act (Phase 2): 각 클라이언트가 초기 메시지 2개를 수신하고 검증합니다.
        let p1_message_handler = async {
            let mut received_messages = Vec::new();
            // 2개의 메시지를 받습니다.
            received_messages.push(ws1.expect_message_opt(|msg| Some(msg)).await);
            received_messages.push(ws1.expect_message_opt(|msg| Some(msg)).await);

            let mut seen_heartbeat = false;
            let mut dealt_cards: Option<Vec<Uuid>> = None;

            for msg in received_messages {
                match msg {
                    ServerMessage::HeartbeatConnected { player, session_id } => {
                        assert_eq!(player, "Player1");
                        assert_eq!(session_id, player1_id);
                        seen_heartbeat = true;
                    }
                    ServerMessage::MulliganDealCards { player, cards } => {
                        assert_eq!(player, "Player1");
                        assert_eq!(cards.len(), COUNT_OF_MULLIGAN_CARDS);
                        dealt_cards = Some(cards);
                    }
                    _ => panic!("[Player1] Received unexpected message: {:?}", msg),
                }
            }

            assert!(seen_heartbeat, "Player1 did not receive HeartbeatConnected");
            dealt_cards.expect("Player1 did not receive MulliganDealCards")
        };

        let p2_message_handler = async {
            let mut received_messages = Vec::new();
            received_messages.push(ws2.expect_message_opt(|msg| Some(msg)).await);
            received_messages.push(ws2.expect_message_opt(|msg| Some(msg)).await);

            let mut seen_heartbeat = false;
            let mut dealt_cards: Option<Vec<Uuid>> = None;

            for msg in received_messages {
                match msg {
                    ServerMessage::HeartbeatConnected { player, session_id } => {
                        assert_eq!(player, "Player2");
                        assert_eq!(session_id, player2_id);
                        seen_heartbeat = true;
                    }
                    ServerMessage::MulliganDealCards { player, cards } => {
                        assert_eq!(player, "Player2");
                        assert_eq!(cards.len(), COUNT_OF_MULLIGAN_CARDS);
                        dealt_cards = Some(cards);
                    }
                    _ => panic!("[Player2] Received unexpected message: {:?}", msg),
                }
            }

            assert!(seen_heartbeat, "Player2 did not receive HeartbeatConnected");
            dealt_cards.expect("Player2 did not receive MulliganDealCards")
        };

        let (p1_dealt_cards, p2_dealt_cards) = join!(p1_message_handler, p2_message_handler);

        // 3. Assert
        let mut all_dealt_cards = p1_dealt_cards;
        all_dealt_cards.extend(p2_dealt_cards);
        let unique_cards_count = all_dealt_cards.iter().collect::<HashSet<_>>().len();
        assert_eq!(unique_cards_count, COUNT_OF_MULLIGAN_CARDS * 2);

        println!("[BARRIER_TEST] Simultaneous connection test with Barrier PASSED.");
    }
    #[actix_web::test]
    async fn test_rejects_duplicate_connection_for_same_player() {
        // 1. Arrange: 테스트 서버를 시작하고 P1의 정보를 준비합니다.
        let (addr, state, _handle) = spawn_server().await;
        let player1_id = state.player1_id;
        let url = format!("ws://{}/game", addr);
        let cookie = format!("user_id={}", player1_id);

        println!("[DUPLICATE_TEST] Starting duplicate connection test for Player 1.");

        // 2. Act (First Connection): 첫 번째 연결을 성공적으로 맺고, 등록 완료 메시지를 기다립니다.
        let mut ws1 = WebSocketTest::connect(url.clone(), cookie.clone())
            .await
            .expect("First connection should succeed");

        println!("[DUPLICATE_TEST] First connection established.");

        // tungstenite 클라이언트는 서버의 Ping에 자동으로 Pong으로 응답합니다.
        // Pong 응답 후 서버가 성공적으로 등록하고 보내주는 HeartbeatConnected 메시지를 기다립니다.
        ws1.expect_message(|msg: ServerMessage| match msg {
            ServerMessage::HeartbeatConnected { player, session_id } => {
                assert_eq!(player, "Player1");
                assert_eq!(session_id, player1_id);
            }
            _ => panic!("Expected HeartbeatConnected for the first connection"),
        })
        .await;
        println!("[DUPLICATE_TEST] First connection confirmed active.");

        // 3. Act (Second Connection): 동일한 플레이어로 두 번째 연결을 시도합니다.
        let mut ws2 = WebSocketTest::connect(url, cookie)
            .await
            .expect("Second connection handshake should also succeed initially");

        println!("[DUPLICATE_TEST] Second connection established. Now expecting an error message.");

        // 4. Assert (Second Connection receives error and is closed):
        // 두 번째 연결은 중복 세션 에러 메시지를 받고 즉시 종료되어야 합니다.
        ws2.expect_message(|msg: ServerMessage| {
            match msg {
                ServerMessage::Error(payload) => {
                    assert_eq!(payload.code, ServerErrorCode::ActiveSessionExists);
                    println!("[DUPLICATE_TEST] Second connection received correct ActiveSessionExists error.");
                }
                _ => panic!("[DUPLICATE_TEST] Expected an Error message for the second connection, but got {:?}", msg),
            }
        }).await;

        // 에러 메시지 전송 후, 서버는 연결을 닫아야 합니다. 스트림이 닫혔는지 확인합니다.
        let next_msg = timeout(Duration::from_secs(1), ws2.stream.next()).await;
        assert!(
            matches!(next_msg, Ok(None) | Ok(Some(Ok(Message::Close(_))))),
            "Second connection was not closed after sending the error message."
        );
        println!("[DUPLICATE_TEST] Second connection was closed as expected.");

        // 5. Assert (First Connection Intact): 첫 번째 연결이 여전히 살아있는지 확인합니다.
        // 서버로부터 오는 다음 heartbeat ping을 기다리는 것으로 확인할 수 있습니다.
        println!("[DUPLICATE_TEST] Verifying first connection is still alive.");
        let ping_check_result = timeout(Duration::from_secs(HEARTBEAT_INTERVAL + 2), async {
            loop {
                match ws1.stream.next().await {
                    Some(Ok(Message::Ping(_))) => {
                        println!("[DUPLICATE_TEST] First connection received a heartbeat ping. It's alive!");
                        return true;
                    }
                    Some(Ok(_)) => continue, // 다른 메시지는 무시
                    Some(Err(_)) | None => return false, // 스트림이 닫혔으면 실패
                }
            }
        }).await;

        assert!(
            ping_check_result.unwrap_or(false),
            "The first connection was unexpectedly closed after a duplicate attempt."
        );

        println!("[DUPLICATE_TEST] Duplicate connection test PASSED.");
    }

    #[actix_web::test]
    async fn test_game_aborts_if_first_player_disconnects() {
        // 1. Arrange
        let (addr, state, _handle) = spawn_server().await;
        let player1_id = state.player1_id;
        let url = format!("ws://{}/game", addr);
        let cookie = format!("user_id={}", player1_id);

        info!("[ABORT_TEST] Starting test: P1 connects, then disconnects before P2 joins.");

        // 2. Act: P1 connects, confirms, and disconnects.
        let mut ws1 = WebSocketTest::connect(url.clone(), cookie.clone())
            .await
            .expect("P1 initial connection should succeed");

        ws1.send(Message::Pong(vec![]))
            .await
            .expect("Failed to send initial Pong");
        info!("[ABORT_TEST] P1 sent initial Pong.");

        ws1.expect_message(|msg: ServerMessage| {
            if !matches!(msg, ServerMessage::HeartbeatConnected { .. }) {
                panic!("Expected HeartbeatConnected, got {:?}", msg);
            }
        })
        .await;
        info!("[ABORT_TEST] P1 connection confirmed. Server is now waiting for P2.");

        ws1.close().await.expect("Failed to close P1's websocket");
        info!("[ABORT_TEST] P1 connection close command sent.");

        sleep(Duration::from_millis(500)).await;
        info!("[ABORT_TEST] P1 disconnected. Checking if the game session was aborted.");

        // 3. Assert: The game session should be terminated.
        let reconnect_result = WebSocketTest::connect(url, cookie).await;

        if let Ok(mut ws_reconnect) = reconnect_result {
            info!("[ABORT_TEST] Reconnection handshake succeeded. Now triggering registration attempt.");

            // *** KEY CHANGE: Trigger the registration process by sending a Pong ***
            // This will cause the new ConnectionActor to message the (now dead) GameActor.
            let send_pong_result = ws_reconnect.send(Message::Pong(vec![])).await;

            if send_pong_result.is_err() {
                // If sending the Pong fails, it means the connection was already closed. This is a pass.
                info!("[ABORT_TEST] PASSED: Could not send Pong on new connection, as it was already closing.");
            } else {
                // If Pong was sent, we expect the server to detect the dead GameActor and close the connection.
                let first_message =
                    timeout(Duration::from_secs(2), ws_reconnect.stream.next()).await;

                match first_message {
                    Ok(Some(Ok(Message::Close(_)))) | Ok(None) => {
                        info!("[ABORT_TEST] PASSED: Reconnection was closed by the server after registration attempt.");
                    }
                    Ok(Some(Err(e))) => {
                        info!("[ABORT_TEST] PASSED: Reconnection failed with an error, which is an acceptable outcome. Error: {:?}", e);
                    }
                    Ok(Some(Ok(msg))) => {
                        // It's possible to receive an error message before the close frame.
                        if let Ok(server_msg) =
                            serde_json::from_str::<ServerMessage>(&msg.to_string())
                        {
                            if matches!(server_msg, ServerMessage::Error(_)) {
                                info!("[ABORT_TEST] PASSED: Received an error message before closing.");
                                // Check for a subsequent close frame
                                let second_message =
                                    timeout(Duration::from_secs(1), ws_reconnect.stream.next())
                                        .await;
                                assert!(matches!(
                                    second_message,
                                    Ok(Some(Ok(Message::Close(_)))) | Ok(None)
                                ));
                                return;
                            }
                        }
                        panic!(
                            "[ABORT_TEST] FAILED: Reconnection was not closed. Instead, received message: {:?}",
                            msg
                        );
                    }
                    Err(_) => {
                        panic!("[ABORT_TEST] FAILED: Timed out waiting for the server to close the reconnection.");
                    }
                }
            }
        } else if let Err(e) = reconnect_result {
            info!(
                "[ABORT_TEST] PASSED: Reconnection attempt failed at the handshake level, as expected. Error: {}",
                e
            );
        }
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

        // 2. 서버가 연결을 등록하도록 수동으로 첫 Pong을 보냅니다.
        ws.send(Message::Pong(vec![]))
            .await
            .expect("Failed to send initial Pong");
        info!("[TEST] Manually sent initial Pong to trigger registration.");

        let extractor = |message: ServerMessage| match message {
            ServerMessage::HeartbeatConnected { player, session_id } => {
                assert_eq!(player, player_type);
                assert!(!session_id.is_nil());
            }
            _ => panic!("Expected HeartbeatConnected message"),
        };
        ws.expect_message(extractor).await;
    }

    // TODO: 제대로된 사유로 성공하는지 확인해야함.
    #[actix_web::test]
    async fn test_heartbeat_timeout() {
        let (addr, state, _handle) = spawn_server().await;

        let player1_id = state.player1_id;
        let player_kind_str = PlayerKind::Player1.as_str();

        let url = format!("ws://{}/game", addr);
        let cookie = format!("user_id={}", player1_id);

        let mut ws = WebSocketTest::connect(url, cookie).await.unwrap();

        // 2. 서버가 연결을 등록하도록 수동으로 첫 Pong을 보냅니다.
        ws.send(Message::Pong(vec![]))
            .await
            .expect("Failed to send initial Pong");
        info!("[TEST] Manually sent initial Pong to trigger registration.");

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

        // 2. 서버가 연결을 등록하도록 수동으로 첫 Pong을 보냅니다.
        ws.send(Message::Pong(vec![]))
            .await
            .expect("Failed to send initial Pong");
        info!("[TEST] Manually sent initial Pong to trigger registration.");

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
