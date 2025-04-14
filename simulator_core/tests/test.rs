pub mod heartbeat {
    use simulator_core::{
        card::types::PlayerType,
        test::{spawn_server, WebSocketTest},
    };

    #[actix_web::test]
    async fn test_heartbeat_initialize_msg() {
        let (addr, _, _handle) = spawn_server().await;

        let player_type = PlayerType::Player1.as_str();

        // 하트비트 연결 URL 및 쿠키 설정
        let url = format!("ws://{}/heartbeat", addr);
        let cookie = format!("user_id={}; game_step={}", player_type, "heartbeat");

        // 예시: 서버에 GET 요청 보내기
        let mut ws = WebSocketTest::connect(url, cookie).await.unwrap();
    }
}
