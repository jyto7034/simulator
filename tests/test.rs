#[cfg(test)]
mod utils_test {
    use card_game::{
        enums::MAX_CARD_SIZE,
        test::generate_random_deck_json,
        utils::{deckcode_to_cards, parse_json_to_deck_code},
    };

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
        for item in p1_cards {
            if !original_cards.contains(item) {
                panic!("deck encode/dedcode error");
            }
        }

        // 카드 수 검증
        assert_eq!(
            p1_cards.len(),
            MAX_CARD_SIZE,
            "Deck should have {MAX_CARD_SIZE} cards"
        );
        assert_eq!(
            original_cards.len(),
            MAX_CARD_SIZE,
            "Original deck should have {MAX_CARD_SIZE} cards"
        );
    }
}

#[cfg(test)]
pub mod game_test {
    use std::{net::SocketAddr, time::Duration};

    use actix_web::web::Data;
    use async_tungstenite::{tokio::connect_async, tungstenite::{http::Request, Message}};
    use card_game::{card::{cards::CardVecExt, types::PlayerType}, enums::COUNT_OF_MULLIGAN_CARDS, server::{jsons::MulliganMessage, types::ServerState}, test::spawn_server, zone::zone::Zone};
    use futures_util::StreamExt;
    use serde_json::json;
    use tokio::time::sleep;

    #[actix_web::test]
    async fn test_mulligan_reroll_restore_variants() -> std::io::Result<()> {
        // console_subscriber::init();
        async fn run_mulligan_case(reroll_count: usize) -> std::io::Result<()> {
            async fn run_mulligan_case_each_player(
                reroll_count: usize,
                player_type: &str,
                addr: SocketAddr,
                server_state: Data<ServerState>,
            ) -> std::io::Result<()> {
                // WebSocket 서버의 URL 생성 (예: "ws://127.0.0.1:{포트}/mulligan_step")
                let url = format!("ws://{}{}", addr, "/mulligan_step");

                // 테스트용 쿠키 값 지정 (Request Guard 내부에서 state.player_cookie 또는 state.opponent_cookie와 비교)
                let cookie_value = format!("user_id={}", player_type);
                // println!("input cookie: {}", cookie_value);

                // http::Request 빌더를 사용하여 요청을 생성하면서 쿠키 헤더 추가
                let request = Request::builder()
                    .uri(&url)
                    .header("Cookie", cookie_value)
                    .header("Host", addr.to_string())
                    .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
                    .header("Upgrade", "websocket")
                    .header("Connection", "Upgrade")
                    .header("Sec-WebSocket-Version", "13")
                    .body(())
                    .expect("요청 생성 실패");

                // async-tungstenite를 통해 WebSocket 연결 시도
                let (mut ws_stream, response) = connect_async(request).await.expect("연결 실패");

                // HTTP 핸드쉐이크가 성공하면 응답 상태는 101 Switching Protocols여야 합니다.
                assert_eq!(
                    response.status(),
                    async_tungstenite::tungstenite::http::StatusCode::SWITCHING_PROTOCOLS
                );

                // 엔드포인트 코드에서는 업그레이드 후 deal 메시지(new_cards_json)를 즉시 클라이언트로 전송하도록 구성되어 있습니다.
                // 첫 번째로 서버에서 오는 메시지를 확인합니다.
                let deal_cards = if let Some(Ok(Message::Text(text))) = ws_stream.next().await {
                    if let MulliganMessage::Deal(data) =
                        serde_json::from_str::<MulliganMessage>(&text)
                            .expect("Failed to parse the deal message JSON")
                    {
                        data.cards
                    } else {
                        panic!("Expected a MulliganMessage::Deal message, but received a different message variant.");
                    }
                } else {
                    panic!("Did not receive any message from the server while expecting MulliganMessage::Deal.");
                };

                // mulligan 을 통해 뽑혀진 카드가 Deck 에서 제대로 제거가 됐는지 확인합니다.
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

                // 테스트로 클라이언트에서 서버로 메시지를 전송하여 후속 처리가 되는지 확인합니다.
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

                let rerolled_cards = if let Some(Ok(Message::Text(text))) = ws_stream.next().await {
                    if let MulliganMessage::RerollAnswer(data) =
                        serde_json::from_str::<MulliganMessage>(&text)
                            .expect("Failed to parse the reroll answer JSON")
                    {
                        data.cards
                    } else {
                        panic!("Expected a MulliganMessage::RerollAnswer message, but received a different message variant.");
                    }
                } else {
                    panic!("Did not receive any message from the server while expecting MulliganMessage::RerollAnswer.");
                };

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
                    for card in &deal_cards[reroll_count..] {
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

                let json = json!({
                    "action": "complete",
                    "payload": {
                        "player": player_type,
                        "cards": []
                    }
                });

                ws_stream
                    .send(Message::Text(json.to_string()))
                    .await
                    .expect("Failed to send message");

                Ok(())
            }

            let (addr, server_state, handle) = spawn_server();

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
            handle.stop(true).await;

            Ok(())
        }

        for reroll_count in (1..=5).rev() {
            run_mulligan_case(reroll_count).await?;
        }
        Ok(())
    }
}
//  {} card(s) being rerolled.
