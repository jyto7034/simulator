use std::{fs::File, io::Read};

use simulator::{exception::exception::Exception, utils::json};

fn main() {
    let file_path = "E:/work/simulator/Datas/data.json";

    // 파일 열기
    let mut file = File::open(file_path).expect("Failed to open file");

    // 파일 내용을 문자열로 읽기
    let mut json_data = String::new();
    file.read_to_string(&mut json_data)
        .expect("Failed to read file");

    println!("{:#?}", json_data);

    let decks: json::Decks = match serde_json::from_str(&json_data[..]) {
        Ok(data) => data,
        Err(_) => {
            println!("error");
            return;
        }
    };
}
