use actix_web::{test, web, App};
use serde_json::json;
use simulator_auth_server::auth_server::{
    db_operation,
    end_point::{delete_player_handler, steam_authentication_handler},
    types::AppState,
};
use sqlx::PgPool;
use std::sync::Once;
use steamworks::{Client, TicketForWebApiResponse};
use tracing::info;

static INIT: Once = Once::new();

fn setup_test_environment() {
    INIT.call_once(|| {
        dotenvy::dotenv().ok();
        let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();
        std::fs::write("steam_appid.txt", "480").expect("Failed to write steam_appid.txt for test");
    });
}

/// 웹 API용 티켓을 발급받는 헬퍼 함수
fn get_web_api_ticket() -> (u64, String) {
    let client = Client::init().unwrap();
    let steam_id = client.user().steam_id().raw();

    let (tx, rx) = std::sync::mpsc::channel();

    let _cb = client.register_callback(move |resp: TicketForWebApiResponse| {
        if resp.result.is_ok() {
            let ticket_hex = hex::encode(resp.ticket);
            tx.send(Some(ticket_hex)).unwrap();
        } else {
            tx.send(None).unwrap();
        }
    });

    // 웹 API용 티켓을 요청합니다.
    client
        .user()
        .authentication_session_ticket_for_webapi("test_identity");

    for _ in 0..100 {
        client.run_callbacks();
        if let Ok(Some(ticket)) = rx.try_recv() {
            return (steam_id, ticket);
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    panic!("Failed to get web api ticket");
}

async fn setup_test_app() -> (
    impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    PgPool,
    u64,
    String,
) {
    setup_test_environment();

    let db_pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    let http_client = reqwest::Client::new();
    let steam_web_api_key = std::env::var("STEAM_WEB_API_KEY").unwrap();
    let (my_steam_id, steam_ticket_hex) = tokio::task::spawn_blocking(get_web_api_ticket)
        .await
        .unwrap();

    let app_state = AppState {
        http_client,
        db_pool: db_pool.clone(),
        steam_web_api_key,
        app_id: 480,
        expected_identity: std::env::var("EXPECTED_IDENTITY")
            .unwrap_or("test_identity".to_string()),
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .service(web::scope("/auth").service(steam_authentication_handler))
            .service(web::scope("/test").service(delete_player_handler)),
    )
    .await;

    (app, db_pool, my_steam_id, steam_ticket_hex)
}

#[actix_web::test]
async fn test_web_api_auth_and_delete_flow() {
    let (app, db_pool, my_steam_id, steam_ticket_hex) = setup_test_app().await;

    // 1. 인증 엔드포인트 호출하여 플레이어 생성
    let req_body = json!({ "ticket": steam_ticket_hex });
    let req = test::TestRequest::post()
        .uri("/auth/steam")
        .insert_header(("Content-Type", "application/json"))
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "Auth request failed with status: {}",
        resp.status()
    );

    // DB에 플레이어가 생성되었는지 확인
    let player = db_operation::get_player_by_id(&db_pool, my_steam_id as i64)
        .await
        .unwrap()
        .expect("Player should exist after auth");
    assert_eq!(player.id, my_steam_id as i64);
    info!("✅ Player created successfully.");

    // 2. 삭제 엔드포인트 호출
    let req = test::TestRequest::delete()
        .uri(&format!("/test/player/{}", my_steam_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "Delete request failed with status: {}",
        resp.status()
    );

    // DB에서 플레이어가 삭제되었는지 확인
    let player = db_operation::get_player_by_id(&db_pool, my_steam_id as i64)
        .await
        .unwrap();
    assert!(player.is_none(), "Player should not exist after deletion");
    info!("✅ Player deleted successfully.");
}
