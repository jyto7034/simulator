use std::fs;
use std::io::Write;
use std::path::Path;

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

fn main() {
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
}
