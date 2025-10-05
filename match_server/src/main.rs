use actix::{Actor, System};
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use match_server::{
    env::Settings,
    extract_client_ip, init_retry_config,
    matchmaker::{spawn_matchmakers, MatchmakerDeps},
    metrics::MetricsCtx,
    session::Session,
    subscript::SubScriptionManager,
    AppState, GameMode, LoggerManager,
};
use prometheus::{Encoder, TextEncoder};
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

#[get("/ws/")]
async fn matchmaking_ws_route(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let client_ip = extract_client_ip(&req).ok_or_else(|| {
        error!("Failed to extract client IP - rejecting connection");
        actix_web::error::ErrorBadRequest("Unable to determine client IP")
    })?;

    let session = Session::new(
        state.sub_manager_addr.clone(),
        Duration::from_secs(state.settings.matchmaking.heartbeat_interval_seconds),
        Duration::from_secs(state.settings.matchmaking.heartbeat_timeout),
        state.clone(),
        client_ip,
    );

    // WebSocket with default size limits (64KB max frame size by default)
    // Note: actix-web-actors uses default max_frame_size of 64KB
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
    init_retry_config(&settings.retry);
    info!("Retry config initialized");

    // 5. Redis 클라이언트 생성
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client =
        redis::Client::open(redis_url.clone()).expect("Failed to create Redis client");

    let redis_conn_manager = redis::aio::ConnectionManager::new(redis_client.clone())
        .await
        .expect("Failed to create Redis connection manager");
    info!("Redis connection established: {}", redis_url);

    // 6. 전역 Shutdown Token 생성
    let shutdown_token = CancellationToken::new();

    // 7. SubScriptionManager 시작
    let sub_manager_addr = SubScriptionManager::new().start();
    info!("SubScriptionManager actor started");

    // 8. Metrics 초기화
    let metrics = Arc::new(MetricsCtx::new());
    let metrics_registry = prometheus::Registry::new();
    metrics::register_custom_metrics(&metrics_registry)
        .expect("Failed to register custom metrics");
    info!("Metrics initialized and registered");

    // 9. Matchmaker Dependencies 준비
    let matchmaker_deps = MatchmakerDeps {
        redis: redis_conn_manager.clone(),
        settings: settings.matchmaking.clone(),
        subscription_addr: sub_manager_addr.clone(),
        metrics: metrics.clone(),
        shutdown_token: shutdown_token.clone(),
    };

    // 10. Matchmaker들 시작 (Normal, Ranked)
    let game_modes = vec![GameMode::Normal, GameMode::Ranked];
    let matchmakers = spawn_matchmakers(&matchmaker_deps, game_modes)
        .expect("Failed to spawn matchmakers");
    info!("Matchmakers started: Normal, Ranked");

    // 11. Rate Limiter 초기화 (10 requests/second per IP)
    let rate_limiter = Arc::new(match_server::RateLimiter::new(10));
    info!("Rate limiter initialized: 10 req/sec per IP");

    // 12. AppState 구성
    let current_run_id = Arc::new(RwLock::new(None));
    let app_state = AppState {
        settings: settings.clone(),
        matchmakers,
        sub_manager_addr,
        redis: redis_conn_manager.clone(),
        logger_manager,
        current_run_id,
        metrics,
        metrics_registry: metrics_registry.clone(),
        rate_limiter,
    };

    // 13. HTTP 서버 시작
    let bind_address = format!("{}:{}", settings.server.bind_address, settings.server.port);
    info!("Starting HTTP server on {}", bind_address);

    let mut server = HttpServer::new(move || {
        // /metrics 엔드포인트 (optional auth)
        let metrics_route = |req: HttpRequest, state: web::Data<AppState>| async move {
            // Check auth token if configured
            if let Some(expected_token) = &state.settings.server.metrics_auth_token {
                let auth_header = req.headers().get("Authorization");
                let provided_token = auth_header
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.strip_prefix("Bearer "));

                if provided_token != Some(expected_token.as_str()) {
                    return HttpResponse::Unauthorized()
                        .body("Unauthorized: Invalid or missing token");
                }
            }

            let metric_families = state.metrics_registry.gather();
            let mut buffer = Vec::new();
            let encoder = TextEncoder::new();

            if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
                return HttpResponse::InternalServerError()
                    .body(format!("Metrics encode error: {}", e));
            }

            HttpResponse::Ok()
                .content_type(encoder.format_type())
                .body(buffer)
        };

        // Healthcheck endpoints
        let health_route = || async { HttpResponse::Ok().body("OK") };
        let ready_route = || async { HttpResponse::Ok().body("READY") };

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(matchmaking_ws_route)
            .route("/metrics", web::get().to(metrics_route))
            .route("/health", web::get().to(health_route))
            .route("/ready", web::get().to(ready_route))
    })
    .bind(&bind_address)?
    .run();

    info!("Match Server is running on {}", bind_address);

    // 14. 종료 신호 대기
    tokio::select! {
        // 서버 자체 종료 (드문 경우)
        res = &mut server => {
            error!("Server exited unexpectedly");
            return res;
        },

        // Ctrl+C 종료 (정상 종료)
        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl+C received. Initiating graceful shutdown...");
            shutdown_token.cancel();  // 모든 Actor에 종료 신호
            System::current().stop();
        },
    }

    // 15. 모든 Actor와 연결 정리 대기
    info!("Waiting for all actors to shutdown...");
    server.await?;
    info!("System has shut down gracefully");

    Ok(())
}
