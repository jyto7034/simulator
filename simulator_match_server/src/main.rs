use actix::Actor;
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use actix_web_prom::PrometheusMetricsBuilder;
use match_server::{
    env::Settings,
    matchmaker,
    provider::DedicatedServerProvider,
    pubsub::{RedisSubscriber, SubscriptionManager},
    setup_logger,
    ws_session::MatchmakingSession,
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
    let session = MatchmakingSession::new(
        state.matchmaker_addr.clone(),
        state.sub_manager_addr.clone(),
    );
    ws::start(session, &req, stream)
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let settings = Settings::new().expect("Failed to load settings.");
    setup_logger();

    let prometheus = PrometheusMetricsBuilder::new("match_server")
        .endpoint("/metrics")
        .build()
        .unwrap();

    register_custom_metrics(&prometheus.registry).expect("Failed to register custom metrics");

    let redis_client =
        redis::Client::open(settings.redis.url.clone()).expect("Failed to create Redis client");
    let redis_conn_manager = redis::aio::ConnectionManager::new(redis_client.clone())
        .await
        .expect("Failed to create Redis connection manager");
    info!("Redis connection manager created.");

    // --- Start New Pub/Sub Actors ---
    let sub_manager_addr = SubscriptionManager::new().start();
    info!("SubscriptionManager actor started.");

    RedisSubscriber::new(redis_client.clone(), sub_manager_addr.clone()).start();
    info!("RedisSubscriber actor started.");
    // --- End of Start New Actors ---

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
        matchmaker_addr: matchmaker_addr.clone(),
        sub_manager_addr,
    };

    let bind_address = format!("{}:{}", settings.server.bind_address, settings.server.port);
    info!("Starting Actix-Web server on {}", bind_address);

    HttpServer::new(move || {
        App::new()
            .wrap(prometheus.clone())
            .app_data(web::Data::new(app_state.clone()))
            .service(matchmaking_ws_route)
    })
    .bind(&bind_address)?
    .run()
    .await
}
