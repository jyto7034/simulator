use actix::{Actor, System};
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use actix_web_prom::PrometheusMetricsBuilder;
use match_server::{
    env::Settings,
    matchmaker::Matchmaker,
    provider::DedicatedServerProvider,
    pubsub::{RedisSubscriber, SubscriptionManager},
    ws_session::MatchmakingSession,
    AppState, LoggerManager,
};
use simulator_metrics::register_custom_metrics;
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc;
use tracing::{error, info};

#[get("/ws/")]
async fn matchmaking_ws_route(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let session = MatchmakingSession::new(
        state.matchmaker_addr.clone(),
        state.sub_manager_addr.clone(),
        Duration::from_secs(state.settings.matchmaking.heartbeat_interval_seconds),
        Duration::from_secs(state.settings.matchmaking.client_timeout_seconds),
    );
    ws::start(session, &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let settings = Settings::new().expect("Failed to load settings.");
    
    // RAII 패턴으로 로거 매니저 생성
    let logger_manager = Arc::new(LoggerManager::setup(&settings));

    let prometheus = PrometheusMetricsBuilder::new("match_server")
        .endpoint("/metrics")
        .build()
        .expect("Failed to build Prometheus metrics.");

    register_custom_metrics(&prometheus.registry).expect("Failed to register custom metrics");

    let redis_client =
        redis::Client::open(settings.redis.url.clone()).expect("Failed to create Redis client");
    let redis_conn_manager = redis::aio::ConnectionManager::new(redis_client.clone())
        .await
        .expect("Failed to create Redis connection manager");
    info!("Redis connection manager created.");

    // --- Graceful Shutdown Channel ---
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

    // --- Start New Pub/Sub Actors ---
    let sub_manager_addr = SubscriptionManager::new().start();
    info!("SubscriptionManager actor started.");

    RedisSubscriber::new(
        redis_client.clone(),
        sub_manager_addr.clone(),
        settings.clone(),
        shutdown_tx.clone(),
    )
    .start();
    info!("RedisSubscriber actor started.");
    // --- End of Start New Actors ---

    let provider_addr =
        DedicatedServerProvider::new(redis_conn_manager.clone(), settings.clone()).start();
    info!("DedicatedServerProvider actor started.");

    let matchmaker_addr = Matchmaker::new(
        redis_conn_manager.clone(),
        settings.matchmaking.clone(),
        provider_addr.clone(),
    )
    .start();
    info!("Matchmaker actor started.");

    let app_state = AppState {
        settings: settings.clone(),
        matchmaker_addr: matchmaker_addr.clone(),
        sub_manager_addr,
        redis_conn_manager: redis_conn_manager.clone(),
        _logger_manager: logger_manager, // RAII 패턴으로 메모리 관리
    };

    let bind_address = format!("{}:{}", settings.server.bind_address, settings.server.port);
    info!("Starting Actix-Web server on {}", bind_address);

    // --- 서버 시작 준비 ---
    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(prometheus.clone())
            .app_data(web::Data::new(app_state.clone()))
            .service(matchmaking_ws_route)
            .service(match_server::events::event_stream_ws)
            // 디버그 엔드포인트들 추가
            .service(match_server::debug::debug_queue_status)
            .service(match_server::debug::debug_active_sessions)
            .service(match_server::debug::debug_loading_sessions)
            .service(match_server::debug::debug_redis_health)
            .service(match_server::debug::debug_ghost_detection)
            .service(match_server::debug::debug_matchmaker_state)
            .service(match_server::debug::debug_dashboard)
    })
    .bind(&bind_address)?
    .run();

    // 종료가 비정상적인지 여부를 추적하는 플래그
    let mut is_error_shutdown = false;

    // --- 종료 신호 대기 ---
    tokio::select! {
        // 서버가 자체적으로 종료된 경우 (예: 바인딩 실패 후 등 드문 경우)
        res = &mut server => {
            error!("Server exited unexpectedly on its own.");
            return res;
        },

        // RedisSubscriber로부터 비정상 종료 신호를 받은 경우
        _ = shutdown_rx.recv() => {
            error!("Critical error signal received. Initiating graceful shutdown...");
            is_error_shutdown = true; // 비정상 종료임을 표시
            // Actix 시스템 전체에 종료 신호를 보냄
            System::current().stop();
        },

        // 사용자가 Ctrl+C를 누른 경우
        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl+C received. Initiating graceful shutdown...");
            // Actix 시스템 전체에 종료 신호를 보냄
            System::current().stop();
        }
    }

    // --- 실제 종료 대기 ---
    // `tokio::select!` 블록이 끝난 후, 즉 종료 신호가 감지된 후에
    // 서버(및 Actix 시스템)가 완전히 멈출 때까지 여기서 기다립니다.
    // 이 `await`는 모든 액터의 `stopping` 메서드가 완료된 후에 반환됩니다.
    info!("Waiting for all actors and connections to close...");
    server.await?;
    info!("System has shut down gracefully.");

    // 비정상 종료 신호로 시작된 경우, 모든 정리가 끝난 이 시점에서 프로세스를 종료합니다.
    if is_error_shutdown {
        info!("Exiting with error code to trigger K8s restart.");
        std::process::exit(1);
    }

    Ok(())
}
