use crate::enums::CardType;
use crate::exception::exception::Exception;
use crate::utils::json;
use crate::deck::{Card, Cards};
use std::fs::File;
use std::io::Read;
use std::process::{Command, Stdio};

pub struct Utils {}

impl Utils {
    pub fn generate_uuid() -> Result<String, Exception> {
        let output = if let Ok(ans) = Command::new("C:\\work\\rust\\simulator\\uuidgen")
            .stdout(Stdio::piped())
            .output()
        {
            ans
        } else {
            return Err(Exception::GenerateUUIDFaild);
        };

        let uuid = String::from_utf8_lossy(&output.stdout).into_owned();
        Ok(uuid)
    }

    pub fn parse_json() -> Result<json::Decks, Exception> {
        let file_path = "C:/work/rust/simulator/Datas/data.json";

        // 파일 열기
        let mut file = File::open(file_path).expect("Failed to open file");

        // 파일 내용을 문자열로 읽기
        let mut json_data = String::new();
        file.read_to_string(&mut json_data)
            .expect("Failed to read file");

        let decks: json::Decks = match serde_json::from_str(&json_data[..]) {
            Ok(data) => data,
            Err(_) => return Err(Exception::JsonParseFailed),
        };

        Ok(decks)
    }

    fn load_card_data(cards: &json::Decks) -> Result<Cards, Exception>{
        // 거대한 json 파일을 읽는 방법 따로 구현해야댐
        // json 을 쌩으로 로드하면 좆댐;

        let file_path = "C:/work/rust/simulator/Resource/cards.json";

        // 파일 열기
        let mut file = File::open(file_path).expect("Failed to open file");

        // 파일 내용을 문자열로 읽기
        let mut json_data = String::new();
        file.read_to_string(&mut json_data)
            .expect("Failed to read file");

        let decks: json::Decks = match serde_json::from_str(&json_data[..]) {
            Ok(data) => data,
            Err(_) => return Err(Exception::JsonParseFailed),
        };

        let mut cards: Vec<Card> = vec![];


        // data.json 에서 가져온 card 데이터를 Vec 으로 밀어넣음.
        // 이 때 cards 에 담기는 데이터는 사용할 수 없는 데이터임.
        for deck in &decks.decks {
            // println!("Hero: {:?}", deck.Hero);
            for card in &deck.cards {
                cards.push(Card {   
                    card_type: CardType::Agent,
                    uuid: "asd".to_string(),
                    name: card.num.to_string(),
                    count: card.id.len(),
                });
            }
        };

        // cards 에 담긴 데이터를 사용하여 실질적인 카드 데이터를 cards.json 으로부터 가져옴.
        for item in cards{
            let target_id = item.name
        }

        Ok()
    }
}
