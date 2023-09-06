use crate::enums::constant;
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

    

    fn load_card_data(player_cards: &json::Decks) -> Result<Vec<Cards>, Exception>{
        // 거대한 json 파일을 읽는 방법 따로 구현해야댐
        // json 을 쌩으로 로드하면 좆댐;

        let file_path = "C:/work/rust/simulator/Resource/cards.json";

        // 파일 열기
        let mut file = File::open(file_path).expect("Failed to open file");

        // 파일 내용을 문자열로 읽기
        let mut json_data = String::new();
        file.read_to_string(&mut json_data)
            .expect("Failed to read file");

        let card_json: Vec<json::CardJson> = match serde_json::from_str(&json_data[..]) {
            Ok(data) => data,
            Err(_) => return Err(Exception::JsonParseFailed),
        };

        let mut p1_cards: Vec<Card> = vec![];
        let mut p2_cards: Vec<Card> = vec![];

        use constant::{PLAYER_1, PLAYER_2};

        // player_cards 에는 플레이어의 덱 정보가 담겨있음.
        // 카드의 종류, 갯수만 있을 뿐, 실질적인 정보는 없고 카드의 id 만 있기 때문에 이것을 사용하여 
        // cards.json 에서 데이터를 가져와야함.
        for card_data in card_json{
            for player_card in &player_cards.decks[PLAYER_1].cards{
                match card_data.id {
                    Some(id) if player_card.id == id => {
                        p1_cards.push()
                    }
                    _ => {}
                }
            }
            for player_card in &player_cards.decks[PLAYER_2].cards{
                match card_data.id {
                    Some(id) if player_card.id == id => {
                    }
                    _ => {}
                }
            }
        }

        Ok()
    }
}
