use actix::{Actor, System};
use match_server::metrics::MetricsCtx;

use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use match_server::{
    blacklist::BlacklistManager,
    env::Settings,
    loading_session::LoadingSessionManager,
    matchmaker::Matchmaker,
    provider::DedicatedServerProvider,
    pubsub::{RedisSubscriber, SubscriptionManager},
    state_events::set_state_events_enabled,
    ws_session::MatchmakingSession,
    AppState, LoggerManager,
};
use prometheus::{Encoder, TextEncoder};
use std::{
    net::IpAddr,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// Extract client IP address from HttpRequest, considering proxy headers
fn extract_client_ip(req: &HttpRequest) -> Option<IpAddr> {
    // 1. X-Forwarded-For header (most common for load balancers/proxies)
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take the first IP in the comma-separated list (original client)
            if let Some(ip_str) = forwarded_str.split(',').next() {
                if let Ok(ip) = ip_str.trim().parse::<IpAddr>() {
                    debug!("Extracted client IP from X-Forwarded-For: {}", ip);
                    return Some(ip);
                }
            }
        }
    }
    
    // 2. X-Real-IP header (Nginx and other reverse proxies)
    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                debug!("Extracted client IP from X-Real-IP: {}", ip);
                return Some(ip);
            }
        }
    }
    
    // 3. CF-Connecting-IP header (Cloudflare)
    if let Some(cf_ip) = req.headers().get("cf-connecting-ip") {
        if let Ok(ip_str) = cf_ip.to_str() {
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                debug!("Extracted client IP from CF-Connecting-IP: {}", ip);
                return Some(ip);
            }
        }
    }
    
    // 4. Direct connection IP (fallback for development/direct access)
    if let Some(peer_addr) = req.connection_info().peer_addr() {
        // Remove port number if present
        if let Some(ip_str) = peer_addr.split(':').next() {
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                debug!("Extracted client IP from peer_addr: {}", ip);
                return Some(ip);
            }
        }
    }
    
    debug!("Could not extract client IP from request");
    None
}

#[get("/ws/")]
async fn matchmaking_ws_route(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    // Extract client IP for blacklist IP change detection
    let client_ip = extract_client_ip(&req);
    
    let session = MatchmakingSession::new(
        state.matchmaker_addr.clone(),
        state.sub_manager_addr.clone(),
        Duration::from_secs(state.settings.matchmaking.heartbeat_interval_seconds),
        Duration::from_secs(state.settings.matchmaking.client_timeout_seconds),
        state.clone(),
        client_ip,
    );

    ws::start(session, &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let settings = Settings::new().expect("Failed to load settings.");
    // Apply feature toggles
    set_state_events_enabled(settings.redis.enable_state_events);

    // RAII 패턴으로 로거 매니저 생성
    let logger_manager = Arc::new(LoggerManager::setup(&settings));

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

    // Prepare shared run_id store before actors so both Matchmaker and AppState share it
    let current_run_id = Arc::new(RwLock::new(None));

    // Initialize MetricsCtx with current_run_id
    let metrics_ctx = std::sync::Arc::new(MetricsCtx::new());

    // Initialize BlacklistManager
    let blacklist_manager_addr = BlacklistManager::new(
        redis_client.clone(), 
        settings.blacklist.clone()
    ).start();
    info!("BlacklistManager actor started.");

    let matchmaker_addr = Matchmaker::new(
        redis_conn_manager.clone(),
        settings.matchmaking.clone(),
        provider_addr.clone(),
        sub_manager_addr.clone(),
        blacklist_manager_addr.clone(),
        metrics_ctx.clone(),
    )
    .start();
    info!("Matchmaker actor started.");

    // Initialize LoadingSessionManager with proper matchmaker reference
    let loading_session_manager_addr = LoadingSessionManager::new(
        blacklist_manager_addr.clone(),
        matchmaker_addr.clone(),
        redis_conn_manager.clone(),
    ).start();
    info!("LoadingSessionManager actor started.");

    // Set loading session manager reference in matchmaker
    matchmaker_addr.do_send(match_server::matchmaker::messages::SetLoadingSessionManager {
        addr: loading_session_manager_addr.clone(),
    });

    // Initialize Prometheus registry and register custom metrics
    let registry = prometheus::Registry::new();
    if let Err(e) = metrics::register_custom_metrics(&registry) {
        eprintln!("Failed to register custom metrics: {}", e);
    }

    let app_state = AppState {
        settings: settings.clone(),
        matchmaker_addr: matchmaker_addr.clone(),
        sub_manager_addr,
        blacklist_manager_addr,
        loading_session_manager_addr,
        redis_conn_manager: redis_conn_manager.clone(),
        _logger_manager: logger_manager, // RAII 패턴으로 메모리 관리
        current_run_id: current_run_id.clone(),
        metrics_registry: registry.clone(),
        metrics: metrics_ctx.clone(),
    };

    let bind_address = format!("{}:{}", settings.server.bind_address, settings.server.port);
    info!("Starting Actix-Web server on {}", bind_address);

    // --- 서버 시작 준비 ---
    let mut server = HttpServer::new(move || {
        // /metrics route
        let metrics_route = |state: web::Data<AppState>| async move {
            let metric_families = state.metrics_registry.gather();
            let mut buffer = Vec::new();
            let encoder = TextEncoder::new();
            if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
                return HttpResponse::InternalServerError()
                    .body(format!("metrics encode error: {}", e));
            }
            HttpResponse::Ok()
                .content_type(encoder.format_type())
                .body(buffer)
        };

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(matchmaking_ws_route)
            .route("/metrics", actix_web::web::get().to(metrics_route))
            .service(match_server::events::event_stream_ws)
            // 디버그 엔드포인트들 추가
            .service(match_server::debug::debug_queue_status)
            .service(match_server::debug::debug_active_sessions)
            .service(match_server::debug::debug_loading_sessions)
            .service(match_server::debug::debug_redis_health)
            .service(match_server::debug::debug_ghost_detection)
            .service(match_server::admin::test_reset)
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
