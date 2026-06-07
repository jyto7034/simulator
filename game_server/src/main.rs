use actix::Actor;
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use game_core::game::{
    battle::buffs::BuffDatabase,
    data::{
        abnormality_data::AbnormalityDatabase,
        artifact_data::ArtifactDatabase,
        consumable_data::ConsumableDatabase,
        corroded_employee_data::CorrodedEmployeeProfileDatabase,
        corroded_wave_data::CorrodedWavePresetDatabase,
        employee_data::{RecruitmentEmployeeCandidateDatabase, StarterEmployeeCandidateDatabase},
        equipment_data::EquipmentDatabase,
        pve_data::PveEncounterDatabase,
        reward_data::RewardDatabase,
        shop_data::ShopDatabase,
        skill_data::SkillDatabase,
        skill_fragment_data::SkillFragmentDatabase,
        GameDataBase, GameDataBuilder,
    },
};
use game_server::{
    env::Settings,
    game::{load_balance_actor::LoadBalanceActor, player_game_actor::session::PlayerGameSession},
    init_retry_config, AppState, LoggerManager,
};
use prometheus::{Encoder, TextEncoder};
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

    // 8. Metrics 초기화
    let metrics_registry = prometheus::Registry::new();
    metrics::register_custom_metrics(&metrics_registry).expect("Failed to register custom metrics");
    info!("Metrics initialized and registered");

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
        metrics_registry: metrics_registry.clone(),
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
            .service(player_game_ws_route)
            .route("/metrics", web::get().to(metrics_route))
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
    let shops_ron = include_str!("../../game_resources/data/events/shops/base.ron");
    let rewards_ron = include_str!("../../game_resources/data/events/rewards/base.ron");
    let abnormalities_ron = include_str!("../../game_resources/data/abnormalities/base.ron");
    let corroded_employees_ron =
        include_str!("../../game_resources/data/enemies/corroded_employees.ron");
    let corroded_wave_presets_ron =
        include_str!("../../game_resources/data/enemies/corroded_wave_presets.ron");
    let starter_candidates_ron =
        include_str!("../../game_resources/data/employees/starter_candidates.ron");
    let recruitment_candidates_ron =
        include_str!("../../game_resources/data/employees/recruitment_candidates.ron");
    let equipments_ron = include_str!("../../game_resources/data/equipments/base.ron");
    let artifacts_ron = include_str!("../../game_resources/data/artifacts/base.ron");
    let consumables_ron = include_str!("../../game_resources/data/consumables/base.ron");
    let buffs_ron = include_str!("../../game_resources/data/buffs/base.ron");
    let skills_ron = include_str!("../../game_resources/data/skills/base.ron");
    let skill_fragments_ron = include_str!("../../game_resources/data/skill_fragments/base.ron");
    let pve_ron = include_str!("../../game_resources/data/pve/encounters.ron");

    let shops_db: ShopDatabase =
        ron::de::from_str(shops_ron).expect("Failed to deserialize shops/base.ron");

    let rewards_db: RewardDatabase =
        ron::de::from_str(rewards_ron).expect("Failed to deserialize rewards/base.ron");

    let abnormalities_db: AbnormalityDatabase =
        ron::de::from_str(abnormalities_ron).expect("Failed to deserialize abnormalities/base.ron");
    let corroded_employee_db: CorrodedEmployeeProfileDatabase =
        ron::de::from_str(corroded_employees_ron)
            .expect("Failed to deserialize corroded_employees.ron");
    let corroded_wave_db: CorrodedWavePresetDatabase = ron::de::from_str(corroded_wave_presets_ron)
        .expect("Failed to deserialize corroded_wave_presets.ron");
    let starter_employee_db: StarterEmployeeCandidateDatabase =
        ron::de::from_str(starter_candidates_ron)
            .expect("Failed to deserialize starter_candidates.ron");
    let recruitment_employee_db: RecruitmentEmployeeCandidateDatabase =
        ron::de::from_str(recruitment_candidates_ron)
            .expect("Failed to deserialize recruitment_candidates.ron");

    let equipments_db: EquipmentDatabase =
        ron::de::from_str(equipments_ron).expect("Failed to deserialize equipments/base.ron");
    let artifacts_db: ArtifactDatabase =
        ron::de::from_str(artifacts_ron).expect("Failed to deserialize artifacts/base.ron");
    let consumables_db: ConsumableDatabase =
        ron::de::from_str(consumables_ron).expect("Failed to deserialize consumables/base.ron");
    let buffs_db: BuffDatabase =
        ron::de::from_str(buffs_ron).expect("Failed to deserialize buffs/base.ron");
    let skill_db: SkillDatabase =
        ron::de::from_str(skills_ron).expect("Failed to deserialize skills/base.ron");
    let skill_fragment_db: SkillFragmentDatabase = ron::de::from_str(skill_fragments_ron)
        .expect("Failed to deserialize skill_fragments/base.ron");
    let pve_db: PveEncounterDatabase =
        ron::de::from_str(pve_ron).expect("Failed to deserialize pve/encounters.ron");

    GameDataBuilder::empty()
        .with_abnormality_data(Arc::new(abnormalities_db))
        .with_corroded_employee_data(Arc::new(corroded_employee_db))
        .with_corroded_wave_data(Arc::new(corroded_wave_db))
        .with_starter_employee_data(Arc::new(starter_employee_db))
        .with_recruitment_employee_data(Arc::new(recruitment_employee_db))
        .with_artifact_data(Arc::new(artifacts_db))
        .with_consumable_data(Arc::new(consumables_db))
        .with_equipment_data(Arc::new(equipments_db))
        .with_shop_data(Arc::new(shops_db))
        .with_reward_data(Arc::new(rewards_db))
        .with_pve_data(Arc::new(pve_db))
        .with_buff_data(Arc::new(buffs_db))
        .with_skill_data(Arc::new(skill_db))
        .with_skill_fragment_data(Arc::new(SkillFragmentDatabase::with_builtin_starter(
            skill_fragment_db.fragments,
        )))
        .build_arc()
}
