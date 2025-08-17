use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use actix_web_prom::PrometheusMetricsBuilder;
// use dedicated_server::{server::game as game_ws_handler, setup_logger};
use metrics::{register_custom_metrics, ACTIVE_SESSIONS};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// match_server로부터 받을 요청 구조체
#[derive(Deserialize, Debug)]
struct CreateSessionRequest {
    players: Vec<Uuid>,
}

// match_server에게 보낼 응답 구조체
#[derive(Serialize, Debug)]
struct CreateSessionResponse {
    server_address: String,
    session_id: Uuid,
}

// 서버의 상태를 관리하는 구조체
struct ServerState {
    redis_conn: redis::aio::ConnectionManager,
    // 이 서버 인스턴스의 고유 ID (Redis 키로 사용됨)
    server_id: String,
    // 외부에서 접근 가능한 주소(매치 서버가 호출할 주소)
    advertise_address: String,
}

#[post("/session/create")]
async fn create_session(
    req: web::Json<CreateSessionRequest>,
    state: web::Data<ServerState>,
) -> impl Responder {
    let session_id = Uuid::new_v4();
    tracing::info!(
        "Received request to create session for players: {:?}",
        req.players
    );

    // --- 상태 변경 로직 추가 ---
    let mut redis_conn = state.redis_conn.clone();
    let server_key = &state.server_id;

    // 외부에서 접근 가능한 주소로 상태를 업데이트합니다.
    let advertise_address = &state.advertise_address;
    let new_server_info = serde_json::to_string(&serde_json::json!({
        "address": advertise_address,
        "status": "busy"
    }))
    .unwrap();

    // SET 명령어로 Redis에 있는 서버 정보를 덮어씁니다.
    match redis_conn
        .set::<_, _, ()>(server_key, new_server_info)
        .await
    {
        Ok(_) => tracing::info!("Server {} status updated to busy.", server_key),
        Err(e) => {
            tracing::error!(
                "Failed to update server {} status to busy: {}",
                server_key,
                e
            );
            // 에러 발생 시, 세션 생성을 중단하고 에러 응답을 보냅니다.
            return HttpResponse::InternalServerError().body("Failed to update server status.");
        }
    }
    // --- 상태 변경 로직 끝 ---

    ACTIVE_SESSIONS.inc();

    let response = CreateSessionResponse {
        server_address: format!(
            "ws://{}/game?session_id={}",
            state.advertise_address, session_id
        ),
        session_id,
    };

    // Spawn a background task to simulate a running session and revert status to 'idle'
    {
        // let mut redis_conn = state.redis_conn.clone();
        // let server_key = state.server_id.clone();
        // let advertise_address = state.advertise_address.clone();
        // let simulated_secs: u64 = std::env::var("DEDICATED_SERVER_SIMULATED_SESSION_SECS")
        //     .ok()
        //     .and_then(|s| s.parse().ok())
        //     .unwrap_or(5);
        // actix_rt::spawn(async move {
        //     tokio::time::sleep(std::time::Duration::from_secs(simulated_secs)).await;
        //     let idle_info = serde_json::to_string(&serde_json::json!({
        //         "address": advertise_address,
        //         "status": "idle"
        //     }))
        //     .unwrap();
        //     let _ = redis_conn.set::<_, _, ()>(&server_key, idle_info).await;
        //     ACTIVE_SESSIONS.dec();
        //     tracing::info!(
        //         "Simulated session finished. Server {} status reverted to idle.",
        //         server_key
        //     );
        // });
    }

    HttpResponse::Ok().json(response)
}

// TODO: 게임 세션이 종료될 때, 이 함수와 같은 로직을 호출하여
//       서버 상태를 다시 "idle"로 변경해야 합니다.
// async fn set_server_status_idle(state: web::Data<ServerState>) { ... }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // setup_logger();

    let prometheus = PrometheusMetricsBuilder::new("dedicated_server")
        .endpoint("/metrics")
        .build()
        .unwrap();

    register_custom_metrics(&prometheus.registry).expect("Failed to register custom metrics");

    let redis_client =
        redis::Client::open("redis://127.0.0.1:6379/").expect("Failed to create Redis client");
    let redis_conn = redis::aio::ConnectionManager::new(redis_client)
        .await
        .expect("Failed to create Redis connection manager");

    let server_id = format!("dedicated_server:{}", Uuid::new_v4());

    // 바인드 주소(서버가 리슨할 주소)와 광고 주소(외부 접근 가능 주소)를 분리합니다.
    let bind_address =
        std::env::var("DEDICATED_SERVER_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8088".to_string());
    // 매치 서버가 접근 가능한 주소. 도커/호스트 환경에 맞게 지정해야 합니다.
    // 예: 호스트에서 둘 다 로컬이면 "127.0.0.1:8088",
    //     Docker Desktop(Windows)에서 매치 서버가 컨테이너이면 "host.docker.internal:8088"
    let advertise_address = std::env::var("DEDICATED_SERVER_ADVERTISE_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8088".to_string());

    let server_state = web::Data::new(ServerState {
        redis_conn: redis_conn.clone(),
        server_id: server_id.clone(),
        advertise_address: advertise_address.clone(),
    });

    // 서버 시작 시 "idle" 상태로 Redis에 등록
    let mut conn_for_startup = redis_conn.clone();
    let server_info = serde_json::to_string(&serde_json::json!({
        "address": advertise_address,
        "status": "idle"
    }))
    .unwrap();

    let _: () = conn_for_startup
        .set(&server_id, server_info)
        .await
        .expect("Failed to register server in Redis");
    tracing::info!(
        "Successfully registered server in Redis with key: {}",
        server_id
    );

    tracing::info!("Starting dedicated server on {}", bind_address);

    HttpServer::new(move || {
        App::new()
            .wrap(prometheus.clone())
            .app_data(server_state.clone())
            .service(create_session)
        // .service(game_ws_handler)
    })
    .bind(bind_address)?
    .run()
    .await
}
