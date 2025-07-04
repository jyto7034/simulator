//! main.rs
//! 
//! 게임 시뮬레이터의 핵심 모듈
//! 이 모듈은 src와 관련된 기능을 제공합니다.

use std::fs;
use std::io::Write;
use std::path::Path;

use actix_web::{App, HttpServer};
use simulator_core::setup_logger;

use clap::Parser;
use uuid::Uuid;

// main
#[derive(Parser)]
#[command(
    name = "card game backend",           // 프로그램 이름
    author = env!("CARGO_PKG_AUTHORS"),       // 작성자
    version = env!("CARGO_PKG_VERSION"),           // 버전
    about = env!("CARGO_PKG_DESCRIPTION"),   // 짧은 설명
    long_about = None,         // 긴 설명 (None은 미사용)
)]
/// 명령줄 인수를 파싱하기 위한 구조체입니다.
/// 플레이어 덱 코드와 공격자를 지정합니다.
///
/// # Examples
/// ```
/// use clap::Parser;
/// #[derive(Parser)]
/// struct Args {
///     #[arg(long = "p1_deck")]
///     #[arg(required = true)]
///     player_1_deckcode: String,
///
///     #[arg(long = "p2_deck")]
///     #[arg(required = true)]
///     player_2_deckcode: String,
///
///     #[arg(required = true)]
///     attacker: usize,
/// }
///
/// // Args 구조체를 사용한 예시
/// ```
struct Args {
    /// 플레이어 1의 덱 코드입니다.
    #[arg(long = "p1_deck")]
    #[arg(required = true)]
    player_1_deckcode: String,

    /// 플레이어 2의 덱 코드입니다.
    #[arg(long = "p2_deck")]
    #[arg(required = true)]
    player_2_deckcode: String,

    /// 공격자를 지정합니다.
    #[arg(required = true)]
    attacker: usize,
}

// TODO: 매칭으로 만난 두 플레이어의 닉네임을 받은 뒤, 게임 공용 서버인 valid server 에 전송하여 실제 플레이어가 맞는지 확인 후, key 값을 리턴 받음.
/// 매칭된 두 플레이어의 세션을 확인하고 고유 세션 키를 생성합니다.
///
/// # Arguments
///
/// * `_nick1`: 첫 번째 플레이어의 닉네임입니다.
/// * `_nick2`: 두 번째 플레이어의 닉네임입니다.
///
/// # Returns
///
/// 두 플레이어에 대한 UUID 쌍 (세션 키)을 반환합니다. 현재는 `todo!()`로 구현되어 있습니다.
///
/// # Examples
///
/// ```
/// use uuid::Uuid;
///
/// // 가상의 닉네임
/// let nick1 = "player1".to_string();
/// let nick2 = "player2".to_string();
///
/// // check_session 함수 호출 (실제 구현은 todo!()로 되어 있음)
/// // let (session_key1, session_key2) = check_session(nick1, nick2);
///
/// // 세션 키가 UUID인지 확인 (실제로는 항상 에러 발생)
/// // assert!(session_key1.is_uuid());
/// // assert!(session_key2.is_uuid());
/// ```
// TODO: valid server 통신 구현
// TODO: 에러 핸들링 추가
pub fn check_session(_nick1: String, _nick2: String) -> (Uuid, Uuid) {
    todo!()
}

#[actix_web::main]
/// 프로그램의 진입점입니다.
/// 카드 레지스트리를 생성하고, Actix 웹 서버를 시작합니다.
/// HTTP 요청을 받아 처리하고 게임 로직을 실행합니다.
///
/// # Arguments
///
/// 이 함수는 인수를 받지 않습니다.
///
/// # Returns
///
/// `std::io::Result<()>`를 반환합니다. 이는 프로그램 실행 결과를 나타냅니다.
/// 성공하면 `Ok(())`를, 실패하면 `Err`를 반환합니다.
///
/// # Examples
///
/// ```rust,no_run
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     // ... Actix 웹 서버 초기화 및 실행 ...
///     Ok(())
/// }
/// ```
// TODO: panic 처리
// TODO: 명령줄 인수 파싱 및 게임 초기화
// TODO: deckcode 파싱 로직 구현
// TODO: actix-web handler 구현
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
        // .service(heartbeat)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}