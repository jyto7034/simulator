use std::fs;
use std::path::Path;
use std::io::Write;

pub fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("card_registry.rs");
    let mut f = fs::File::create(&dest_path).unwrap();

    // 카드 모듈 디렉토리 스캔
    let modules = ["human", "monster", "public"];
    let mut card_registrations = Vec::new();

    for module in modules {
        let path = format!("src/card_gen/{}.rs", module);
        let content = fs::read_to_string(&path).unwrap();
        
        // 함수 이름 찾기 (예: HM_001, MT_001 등)
        for line in content.lines() {
            if line.contains("pub fn") && (line.contains("HM_") || line.contains("MT_") || line.contains("PB_")) {
                let func_name = line.split("fn ")
                    .nth(1)
                    .unwrap()
                    .split("(")
                    .next()
                    .unwrap();
                card_registrations.push(format!("    {}::{}", module, func_name));
            }
        }
    }

    // 매크로 호출 생성
    write!(f, 
    r#"
        generate_card_map! {{
        {}
        }}
    "#, card_registrations.join(",\n")).unwrap();
}