use actix::Actor;
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use game_core::game::data::GameDataBase;
use game_server::{
    env::Settings,
    game::{load_balance_actor::LoadBalanceActor, player_game_actor::session::PlayerGameSession},
    init_retry_config, AppState, LoggerManager,
};
use std::{sync::Arc, time::Duration};
use tracing::{error, info};
use uuid::Uuid;

#[get("/game")]
async fn player_game_ws_route(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let session = PlayerGameSession::new(
        state.clone(),
        Duration::from_secs(state.settings.session.heartbeat_interval_seconds),
        Duration::from_secs(state.settings.session.heartbeat_timeout),
    );

    ws::start(session, &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 1. 환경변수 로드
    dotenv::dotenv().ok();

    // 2. 설정 파일 로드
    let settings = Settings::new().expect("Failed to load settings");

    // 3. 로거 초기화
    let logger_manager = Arc::new(LoggerManager::setup(&settings));
    info!("Logger initialized");

    // 4. Retry config 초기화
    init_retry_config(&settings.retry).await;
    info!("Retry config initialized");

    let game_data = load_game_data_from_ron();
    info!("Game data loaded for meta-game sessions");

    // 10. LoadBalanceActor 시작
    let load_balance_addr = LoadBalanceActor::new().start();
    info!("LoadBalanceActor started");

    let current_run_id = Uuid::new_v4();

    // 15. AppState 구성
    let app_state = AppState {
        settings: settings.clone(),
        load_balance_addr,
        logger_manager,
        current_run_id,
        game_data,
    };

    // 16. HTTP 서버 시작
    let bind_address = format!("{}:{}", settings.server.bind_address, settings.server.port);
    info!("Starting HTTP server on {}", bind_address);

    let server = HttpServer::new(move || {
        // Healthcheck endpoints
        let health_route = || async { HttpResponse::Ok().body("OK") };
        let ready_route = || async { HttpResponse::Ok().body("READY") };

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(player_game_ws_route)
            .route("/health", web::get().to(health_route))
            .route("/ready", web::get().to(ready_route))
    })
    .bind(&bind_address)?
    .run();

    info!("Game Server is running on {}", bind_address);

    // 17. 종료 신호 대기
    let server_handle = server.handle();
    tokio::select! {
        res = server => {
            error!("Server exited unexpectedly");
            return res;
        },

        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl+C received. Initiating graceful shutdown...");

            // Graceful shutdown
            server_handle.stop(true).await;
            info!("System has shut down gracefully");
        },
    }

    Ok(())
}

fn load_game_data_from_ron() -> Arc<GameDataBase> {
    GameDataBase::load_live_embedded()
}
