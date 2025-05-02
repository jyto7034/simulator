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
    use tracing::{info, warn};
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

        // 1. 초기 연결 메시지 수신 (선택 사항)
        info!("Waiting for initial connection message...");
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
                } // _ => panic!("Expected HeartbeatConnected message first"),
            }
        };
        // 초기 메시지를 기대하지만, 타임아웃되면 실패 처리
        if timeout(
            Duration::from_secs(HEARTBEAT_INTERVAL),
            ws.expect_message(initial_extractor),
        )
        .await
        .is_err()
        {
            panic!("Timeout waiting for initial HeartbeatConnected message.");
        }

        // 2. 서버의 Ping을 기다리지만 Pong으로 응답하지 않음
        info!("Waiting for Ping, but will not send Pong...");
        let ping_wait_result = timeout(Duration::from_secs(HEARTBEAT_INTERVAL * 2), async {
            loop {
                match ws.stream.next().await {
                    Some(Ok(Message::Ping(_))) => {
                        info!("Ping received, NOT sending Pong.");
                        break; // Ping을 받았으니 대기 종료
                    }
                    Some(Ok(Message::Text(t))) => {
                        info!("Received unexpected Text while waiting for Ping: {}", t);
                        continue; // 무시하고 계속 Ping 기다림
                    }
                    Some(Ok(_)) => continue, // 다른 메시지 무시
                    Some(Err(e)) => panic!("WebSocket error while waiting for Ping: {:?}", e),
                    None => panic!("WebSocket closed unexpectedly while waiting for Ping"),
                }
            }
        })
        .await;

        if ping_wait_result.is_err() {
            panic!(
                "Timeout: Did not receive first Ping within {} seconds.",
                (HEARTBEAT_INTERVAL * 2)
            );
        }

        // 3. CLIENT_TIMEOUT + 약간의 여유 시간 동안 대기
        //    서버가 타임아웃으로 연결을 종료할 것으로 기대
        let wait_duration = Duration::from_secs(CLIENT_TIMEOUT + 5); // 타임아웃 + 여유 시간
        info!(
            "Waiting for {} seconds for server to close connection due to timeout...",
            wait_duration.as_secs()
        );

        let close_check_result = timeout(wait_duration, async {
            // 서버가 연결을 닫을 때까지 메시지를 읽으려고 시도
            loop {
                match ws.stream.next().await {
                    Some(Ok(Message::Close(reason))) => {
                        info!("Server closed connection as expected. Reason: {:?}", reason);
                        return true; // 서버가 닫음
                    }
                    Some(Ok(msg)) => {
                        // 타임아웃 기간 동안 다른 메시지가 오면 안 됨 (오류 가능성)
                        warn!("Received unexpected message during timeout wait: {:?}", msg);
                        continue; // 계속 대기
                    }
                    Some(Err(e)) => {
                        info!(
                            "WebSocket error during timeout wait (expected if closed): {:?}",
                            e
                        );
                        return true; // 에러 발생도 연결 종료 신호로 간주
                    }
                    None => {
                        info!("WebSocket stream ended (closed by server).");
                        return true; // 스트림 종료 = 연결 닫힘
                    }
                }
            }
        })
        .await;

        // 4. 결과 확인
        match close_check_result {
            Ok(true) => {
                info!("Test Success: Server closed the connection due to heartbeat timeout.");
            }
            Ok(false) => {
                // 루프가 값을 반환하지 않은 경우 (이론상 발생 안 함)
                panic!("Test Error: Timeout check loop finished unexpectedly.");
            }
            Err(_) => {
                panic!("Test Failed: Server did NOT close the connection after timeout period.");
            }
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
        info!("Waiting for initial connection message...");
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
        info!("Waiting for Ping message...");
        let ping_received = timeout(Duration::from_secs(HEARTBEAT_INTERVAL * 2), async {
            loop {
                match ws.stream.next().await {
                    Some(Ok(Message::Ping(ping_data))) => {
                        info!("Ping received!");
                        return Some(ping_data);
                    }
                    Some(Ok(Message::Text(text))) => {
                        info!("Received unexpected Text while waiting for Ping: {}", text);
                        continue;
                    }
                    Some(Ok(Message::Pong(_))) => {
                        info!("Received Pong while waiting for Ping, ignoring.");
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
                info!("Sending Pong response...");
                ws.send(Message::Pong(ping_data.clone()))
                    .await
                    .expect("Failed to send Pong");
                info!("Pong sent.");

                // 잠시 대기하여 서버가 Pong을 처리하고 연결을 유지하는지 확인
                info!("Waiting to see if connection is maintained...");
                sleep(Duration::from_secs(2)).await; // 짧은 시간 대기

                // 연결이 살아있는지 확인
                let test_msg = Message::Pong(ping_data);
                match ws.send(test_msg).await {
                    Ok(_) => info!("Connection seems alive after Pong."),
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
        info!("Closing WebSocket connection.");
        ws.send(Message::Close(None)).await.ok(); // Close 메시지 전송 시도
    }
}
