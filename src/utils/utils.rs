use crate::card_gen::card_gen::CardGenertor;
use crate::deck::{Card, Cards};
use crate::enums::constant;
use crate::exception::exception::Exception;
use crate::utils::json;
use std::fs::File;
use std::io::Read;
use std::process::{Command, Stdio};

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
    let file_path = constant::DECK_JSON_PATH;

    // 파일 열기
    let mut file = File::open(file_path).expect("Failed to open file");

    // 파일 내용을 문자열로 읽기
    let mut json_data = String::new();
    file.read_to_string(&mut json_data)
        .expect("Failed to read file");

    println!("{:#?}", json_data);

    let decks: json::Decks = match serde_json::from_str(&json_data[..]) {
        Ok(data) => data,
        Err(_) => return Err(Exception::JsonParseFailed),
    };

    Ok(decks)
}

pub fn load_card_data(player_cards: &json::Decks) -> Result<Vec<Cards>, Exception> {
    // 거대한 json 파일을 읽는 방법 따로 구현해야댐
    // json 을 쌩으로 로드하면 좆댐;

    let file_path = constant::CARD_JSON_PATH;

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
    let mut ps_cards: Vec<&mut Vec<Card>> = vec![&mut p1_cards, &mut p2_cards];
    use constant::{PLAYER_1, PLAYER_2};
    use json::CardJson;

    let mut check_values_exist =
        |player_num: usize, card_data: &CardJson| -> Result<(), Exception> {
            for player_card in &player_cards.decks[0].cards {
                if let Some(id) = &card_data.id {
                    if player_card.id == *id {
                        ps_cards[player_num].push(CardGenertor::gen_card_by_id(id.to_string()));
                    }
                } else {
                    return Err(Exception::DeckParseError);
                }
            }
            Ok(())
        };

    // player_cards 에는 플레이어의 덱 정보가 담겨있음.
    // 카드의 종류, 갯수만 있을 뿐, 실질적인 정보는 없고 카드의 id 만 있기 때문에 이것을 사용하여
    // cards.json 에서 데이터를 가져와야함.
    for card_data in card_json {
        check_values_exist(PLAYER_1, &card_data)?;
        check_values_exist(PLAYER_2, &card_data)?;
    }

    Ok(vec![Cards::new(&p1_cards), Cards::new(&p2_cards)])
}
