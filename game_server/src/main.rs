use actix::Actor;
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use game_core::game::data::{
    abnormality_data::AbnormalityDatabase, artifact_data::ArtifactDatabase,
    bonus_data::BonusDatabase, equipment_data::EquipmentDatabase, event_pools::EventPoolConfig,
    pve_data::PveEncounterDatabase, random_event_data::RandomEventDatabase,
    shop_data::ShopDatabase, skill_data::SkillDatabase, GameDataBase, GameDataBaseParts,
};
use game_server::{
    env::Settings,
    extract_client_ip, flush_redis_default,
    game::{
        load_balance_actor::LoadBalanceActor, match_coordinator::MatchCoordinator,
        player_game_actor::session::PlayerGameSession, pubsub::spawn_redis_subscribers,
    },
    init_retry_config,
    matchmaking::matchmaker::{spawn_matchmakers, MatchmakerDeps},
    matchmaking::session::Session,
    matchmaking::subscript::SubScriptionManager,
    shared::event_stream::{EventStreamSession, StreamSessionId},
    shared::metrics::MetricsCtx,
    AppState, GameMode, LoggerManager,
};
use prometheus::{Encoder, TextEncoder};
use std::{sync::Arc, time::Duration};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use uuid::Uuid;

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

    ws::start(session, &req, stream)
}

#[get("/events/stream")]
async fn events_stream_route(
    req: HttpRequest,
    stream: web::Payload,
    _state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    // 쿼리 파라미터에서 session_id 추출
    let query = req.query_string();
    let session_id = StreamSessionId::from_query(query).map(|s| s.session_id);

    if let Some(ref sid) = session_id {
        info!("Event stream connection with session_id: {}", sid);
    } else {
        warn!("Event stream connection without session_id - will not subscribe");
    }

    // EventStreamSession 시작
    let session = EventStreamSession::new(session_id);
    ws::start(session, &req, stream)
}

#[get("/game")]
async fn player_game_ws_route(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let session = PlayerGameSession::new(
        state.clone(),
        Duration::from_secs(state.settings.matchmaking.heartbeat_interval_seconds),
        Duration::from_secs(state.settings.matchmaking.heartbeat_timeout),
    );

    ws::start(session, &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    flush_redis_default().await.unwrap();

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

    // 5. Redis 클라이언트 생성
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client =
        redis::Client::open(redis_url.clone()).expect("Failed to create Redis client");

    let redis_conn_manager = redis::aio::ConnectionManager::new(redis_client.clone())
        .await
        .expect("Failed to create Redis connection manager");
    info!("Redis connection established: {}", redis_url);

    let game_data = load_game_data_from_ron();
    info!("Game data loaded for meta-game sessions");

    // 6. 전역 Shutdown Token 생성
    let shutdown_token = CancellationToken::new();

    // 7. SubScriptionManager 시작
    let sub_manager_addr = SubScriptionManager::new().start();
    info!("SubScriptionManager actor started");

    // 8. Metrics 초기화
    let metrics = Arc::new(MetricsCtx::new());
    let metrics_registry = prometheus::Registry::new();
    metrics::register_custom_metrics(&metrics_registry).expect("Failed to register custom metrics");
    info!("Metrics initialized and registered");

    // 9. Circuit Breaker 생성 (Matchmaker와 Redis Pub/Sub에서 공유)
    let redis_circuit = Arc::new(game_server::shared::circuit_breaker::CircuitBreaker::new(
        5, 60,
    ));

    // 10. LoadBalanceActor 시작 (Matchmaker보다 먼저 생성)
    let load_balance_addr = LoadBalanceActor::new(metrics.clone()).start();
    info!("LoadBalanceActor started");

    // 11. Matchmaker Dependencies 준비
    let matchmaker_deps = MatchmakerDeps {
        redis: redis_conn_manager.clone(),
        settings: settings.matchmaking.clone(),
        subscription_addr: sub_manager_addr.clone(),
        load_balance_addr: load_balance_addr.clone(),
        metrics: metrics.clone(),
        shutdown_token: shutdown_token.clone(),
        redis_circuit: redis_circuit.clone(),
    };

    // 12. Matchmaker들 시작 (Normal, Ranked)
    let game_modes = vec![GameMode::Normal, GameMode::Ranked];
    let matchmakers =
        spawn_matchmakers(&matchmaker_deps, game_modes).expect("Failed to spawn matchmakers");
    info!("Matchmakers started: Normal, Ranked");

    // 13. MatchCoordinator 시작
    let match_coordinator_addr = MatchCoordinator::new(
        matchmakers.clone(),
        load_balance_addr.clone(),
        redis_conn_manager.clone(),
    )
    .start();
    info!("MatchCoordinator started");

    // 14. Redis Pub/Sub 구독 시작 (match_result, battle_result)
    let current_run_id = Uuid::new_v4();
    let pod_id = std::env::var("POD_ID").unwrap_or_else(|_| {
        let default_pod_id = format!("pod-{}", current_run_id);
        info!("POD_ID not set, using default: {}", default_pod_id);
        default_pod_id
    });

    spawn_redis_subscribers(
        redis_client.clone(),
        pod_id.clone(),
        load_balance_addr.clone(),
        shutdown_token.clone(),
        redis_circuit.clone(),
    )
    .await;
    info!("Redis Pub/Sub subscribers started for pod: {}", pod_id);

    // 14. Rate Limiter 초기화 (10 requests/second per IP)
    let rate_limiter = Arc::new(game_server::RateLimiter::new(10));
    info!("Rate limiter initialized: 10 req/sec per IP");

    // 15. AppState 구성
    let app_state = AppState {
        settings: settings.clone(),
        matchmakers,
        sub_manager_addr,
        load_balance_addr,
        match_coordinator_addr,
        redis: redis_conn_manager.clone(),
        logger_manager,
        current_run_id,
        game_data,
        metrics,
        metrics_registry: metrics_registry.clone(),
        rate_limiter,
    };

    // 16. HTTP 서버 시작
    let bind_address = format!("{}:{}", settings.server.bind_address, settings.server.port);
    info!("Starting HTTP server on {}", bind_address);

    let server = HttpServer::new(move || {
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
            .service(events_stream_route)
            .service(player_game_ws_route)
            .route("/metrics", web::get().to(metrics_route))
            .route("/health", web::get().to(health_route))
            .route("/ready", web::get().to(ready_route))
    })
    .bind(&bind_address)?
    .run();

    info!("Match Server is running on {}", bind_address);

    // 17. 종료 신호 대기
    let server_handle = server.handle();
    tokio::select! {
        res = server => {
            error!("Server exited unexpectedly");
            return res;
        },

        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl+C received. Initiating graceful shutdown...");
            shutdown_token.cancel();  // 모든 Actor에 종료 신호

            // Graceful shutdown
            server_handle.stop(true).await;
            info!("System has shut down gracefully");
        },
    }

    Ok(())
}

fn load_game_data_from_ron() -> Arc<GameDataBase> {
    let shops_ron = include_str!("../../game_resources/data/events/shops/base.ron");
    let random_shops_ron = include_str!("../../game_resources/data/events/shops/random.ron");
    let bonuses_ron = include_str!("../../game_resources/data/events/bonuses/base.ron");
    let random_bonuses_ron = include_str!("../../game_resources/data/events/bonuses/random.ron");
    let random_events_ron = include_str!("../../game_resources/data/events/random_events.ron");
    let event_pools_ron = include_str!("../../game_resources/data/events/event_pools.ron");
    let abnormalities_ron = include_str!("../../game_resources/data/abnormalities/base.ron");
    let random_abnormalities_ron =
        include_str!("../../game_resources/data/abnormalities/random.ron");
    let equipments_ron = include_str!("../../game_resources/data/equipments/base.ron");
    let artifacts_ron = include_str!("../../game_resources/data/artifacts/base.ron");
    let skills_ron = include_str!("../../game_resources/data/skills/base.ron");
    let pve_ron = include_str!("../../game_resources/data/pve/encounters.ron");

    let mut shops_db: ShopDatabase =
        ron::de::from_str(shops_ron).expect("Failed to deserialize shops/base.ron");
    let random_shops_db: ShopDatabase =
        ron::de::from_str(random_shops_ron).expect("Failed to deserialize shops/random.ron");

    let mut bonuses_db: BonusDatabase =
        ron::de::from_str(bonuses_ron).expect("Failed to deserialize bonuses/base.ron");
    let random_bonuses_db: BonusDatabase =
        ron::de::from_str(random_bonuses_ron).expect("Failed to deserialize bonuses/random.ron");

    let random_events_db: RandomEventDatabase =
        ron::de::from_str(random_events_ron).expect("Failed to deserialize random_events.ron");
    let event_pools: EventPoolConfig =
        ron::de::from_str(event_pools_ron).expect("Failed to deserialize event_pools.ron");

    let mut abnormalities_db: AbnormalityDatabase =
        ron::de::from_str(abnormalities_ron).expect("Failed to deserialize abnormalities/base.ron");
    let random_abnormalities_db: AbnormalityDatabase = ron::de::from_str(random_abnormalities_ron)
        .expect("Failed to deserialize abnormalities/random.ron");

    let equipments_db: EquipmentDatabase =
        ron::de::from_str(equipments_ron).expect("Failed to deserialize equipments/base.ron");
    let artifacts_db: ArtifactDatabase =
        ron::de::from_str(artifacts_ron).expect("Failed to deserialize artifacts/base.ron");
    let skill_db: SkillDatabase =
        ron::de::from_str(skills_ron).expect("Failed to deserialize skills/base.ron");
    let pve_db: PveEncounterDatabase =
        ron::de::from_str(pve_ron).expect("Failed to deserialize pve/encounters.ron");

    shops_db.shops.extend(random_shops_db.shops);
    bonuses_db.bonuses.extend(random_bonuses_db.bonuses);
    abnormalities_db.items.extend(random_abnormalities_db.items);

    Arc::new(GameDataBase::new(GameDataBaseParts {
        abnormality_data: Arc::new(abnormalities_db),
        artifact_data: Arc::new(artifacts_db),
        equipment_data: Arc::new(equipments_db),
        shop_data: Arc::new(shops_db),
        bonus_data: Arc::new(bonuses_db),
        random_event_data: Arc::new(random_events_db),
        pve_data: Arc::new(pve_db),
        skill_data: Arc::new(skill_db),
        event_pools,
    }))
}
