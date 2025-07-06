use actix::Actor;
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use actix_web_prom::PrometheusMetricsBuilder;
use match_server::{
    env::Settings, matchmaker, provider::DedicatedServerProvider, ws_session::MatchmakingSession,
    AppState,
};
use simulator_metrics::register_custom_metrics;
use tracing::info;

#[get("/ws/")]
async fn matchmaking_ws_route(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let session =
        MatchmakingSession::new(state.matchmaker_addr.clone(), state.redis_client.clone());
    ws::start(session, &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let settings = Settings::new().expect("Failed to load settings.");
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            &settings.server.log_level,
        ))
        .init();

    info!("Logger Initialized.");

    // Prometheus 미들웨어 설정
    let prometheus = PrometheusMetricsBuilder::new("match_server")
        .endpoint("/metrics")
        .build()
        .unwrap();

    // 공용 크레이트에서 정의한 커스텀 메트릭 등록
    register_custom_metrics(&prometheus.registry).expect("Failed to register custom metrics");

    let redis_client =
        redis::Client::open(settings.redis.url.clone()).expect("Failed to create Redis client");
    let redis_conn_manager = redis::aio::ConnectionManager::new(redis_client.clone())
        .await
        .expect("Failed to create Redis connection manager");
    info!("Redis connection manager created.");

    let provider_addr = DedicatedServerProvider::new(redis_conn_manager.clone()).start();
    info!("DedicatedServerProvider actor started.");

    let matchmaker_addr = matchmaker::Matchmaker::new(
        redis_conn_manager,
        settings.matchmaking.clone(),
        provider_addr.clone(),
    )
    .start();
    info!("Matchmaker actor started.");

    let app_state = AppState {
        jwt_secret: settings.jwt.secret.clone(),
        redis_client: redis_client.clone(),
        matchmaker_addr: matchmaker_addr.clone(),
        provider_addr,
    };

    let bind_address = format!("{}:{}", settings.server.bind_address, settings.server.port);
    info!("Starting Actix-Web server on {}", bind_address);

    HttpServer::new(move || {
        App::new()
            .wrap(prometheus.clone()) // Prometheus 미들웨어 추가
            .app_data(web::Data::new(app_state.clone()))
            .service(matchmaking_ws_route)
    })
    .bind(&bind_address)?
    .run()
    .await
}
