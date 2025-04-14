use std::fs;
use std::io::Write;
use std::path::Path;

use actix_web::{App, HttpServer};
use simulator_core::server::end_point::heartbeat;
use simulator_core::server::types::SessionKey;
use simulator_core::setup_logger;

use clap::Parser;

// main
#[derive(Parser)]
#[command(
    name = "card game backend",           // 프로그램 이름
    author = env!("CARGO_PKG_AUTHORS"),       // 작성자
    version = env!("CARGO_PKG_VERSION"),           // 버전
    about = env!("CARGO_PKG_DESCRIPTION"),   // 짧은 설명
    long_about = None,         // 긴 설명 (None은 미사용)
)]
struct Args {
    #[arg(long = "p1_deck")]
    #[arg(required = true)]
    player_1_deckcode: String,

    #[arg(long = "p2_deck")]
    #[arg(required = true)]
    player_2_deckcode: String,

    #[arg(required = true)]
    attacker: usize,
}

// TODO: 매칭으로 만난 두 플레이어의 닉네임을 받은 뒤, 게임 공용 서버인 valid server 에 전송하여 실제 플레이어가 맞는지 확인 후, key 값을 리턴 받음.
pub fn check_session(_nick1: String, _nick2: String) -> (SessionKey, SessionKey) {
    todo!()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("card_registry.rs");
    let mut f = fs::File::create(&dest_path).unwrap();

    // 카드 모듈 디렉토리 스캔
    let modules = ["human", "monster", "public"];
    let mut card_registrations = Vec::new();

    for module in modules {
        let path = format!("src/card_gen/{}.rs", module);
        let content = fs::read_to_string(&path).unwrap();

        // 함수 이름 찾기
        for line in content.lines() {
            if line.contains("pub fn")
                && (line.contains("HM_") || line.contains("MT_") || line.contains("PB_"))
            {
                let func_name = line.split("fn ").nth(1).unwrap().split("(").next().unwrap();
                card_registrations.push(format!("    {}::{}", module, func_name));
            }
        }
    }

    // 매크로 호출 생성
    write!(
        f,
        r#"
        // 자동 생성된 카드 레지스트리
        generate_card_map! {{
            {}
        }}
        "#,
        card_registrations.join(",\n")
    )
    .unwrap();

    // let (deck_json, _) = generate_random_deck_json();
    // let (deck_json2, _) = generate_random_deck_json();

    // // 2. JSON을 덱 코드로 변환
    // let deck_codes = parse_json_to_deck_code(Some(deck_json), Some(deck_json2))
    //     .expect("Failed to parse deck code");

    // let app = initialize_app(deck_codes.0, deck_codes.1, 0);

    // let session_keys = check_session("".to_string(), "".to_string());

    // let state = web::Data::new(ServerState {
    //     game: Mutex::new(app.game),
    //     player_cookie: session_keys.0,
    //     opponent_cookie: session_keys.1,
    //     session_manager: PlayerSessionManager::new(CLIENT_TIMEOUT),
    // });

    setup_logger();
    HttpServer::new(move || {
        App::new()
            // .app_data(state.clone())
            // .service(mulligan_phase)
            // .service(draw_phase)
            // .service(standby_phase)
            // .service(main_phase_start_phase)
            .service(heartbeat)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
