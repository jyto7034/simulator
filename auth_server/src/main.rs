use actix_web::{web, App, HttpServer};
use auth_server::{
    auth_server::{
        end_point::{delete_player_handler, steam_authentication_handler},
        types::AppState,
    },
    setup_logger,
};
use sqlx::postgres::PgPoolOptions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    std::fs::write("steam_appid.txt", "480")?;

    // 로거 초기화는 한 번만 수행합니다. try_init()은 중복 초기화 패닉을 방지합니다.
    setup_logger();
    tracing::info!("Logger Initialized.");

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
    let steam_web_api_key =
        std::env::var("STEAM_WEB_API_KEY").expect("STEAM_WEB_API_KEY must be set in .env file");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env file");

    let db_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to create database connection pool");
    tracing::info!("Database connection pool created.");

    let app_state = AppState {
        http_client: reqwest::Client::new(),
        db_pool,
        steam_web_api_key: steam_web_api_key.clone(),
        app_id: 480,
        expected_identity: std::env::var("EXPECTED_IDENTITY")
            .expect("EXPECTED_IDENTITY must be set in .env file"),
        jwt_secret,
    };
    let bind_address = "127.0.0.1:3000";
    tracing::info!("Starting Actix-Web server on {}", bind_address);

    HttpServer::new(move || {
        let auth_scope = web::scope("/auth").service(steam_authentication_handler);

        #[cfg(feature = "test-endpoints")]
        let auth_scope = {
            use auth_server::auth_server::end_point::test_authentication_handler;
            auth_scope.service(test_authentication_handler)
        };

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(auth_scope)
            .service(web::scope("/test").service(delete_player_handler))
    })
    .bind(bind_address)?
    .run()
    .await
}
