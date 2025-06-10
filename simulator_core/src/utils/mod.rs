pub mod json;

use crate::card::cards::Cards;
use crate::card::types::PlayerKind;
use crate::card_gen::{CardGenerator, Keys};
use crate::enums::*;
use crate::exception::{GameError, SystemError, GameplayError, DeckError};
use base64::{decode, encode};
use byteorder::WriteBytesExt;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::{Cursor, Write};
use std::vec;
use tracing::warn;
use uuid::Uuid;

pub fn generate_uuid() -> Result<Uuid, GameError> {
    let uuid = Uuid::new_v4();
    Ok(uuid)
}

pub fn read_game_config_json() -> Result<json::GameConfigJson, GameError> {
    let file_path = GAME_CONFIG_JSON_PATH;

    // 파일 열기
    let mut file = File::open(file_path).expect("Failed to open file");

    // 파일 내용을 문자열로 읽기
    let mut json_data = String::new();
    file.read_to_string(&mut json_data)
        .expect("Failed to read file");

    let card_json: json::GameConfigJson = match serde_json::from_str(&json_data[..]) {
        Ok(data) => data,
        Err(e) => return Err(GameError::System(SystemError::Json(e))),
    };

    Ok(card_json)
}

pub fn parse_json_to_deck_code(
    p1_card_json: Option<Value>,
    p2_card_json: Option<Value>,
) -> Result<(String, String), GameError> {
    match (&p1_card_json, &p2_card_json) {
        (None, None) => return Err(GameError::Gameplay(GameplayError::DeckError(DeckError::ParseFailed("Decode error".to_string())))),
        (None, Some(_)) => return Err(GameError::Gameplay(GameplayError::DeckError(DeckError::CodeMissingFor(PlayerKind::Player1)))),
        (Some(_), None) => return Err(GameError::Gameplay(GameplayError::DeckError(DeckError::CodeMissingFor(PlayerKind::Player2)))),
        _ => {}
    }

    fn parse_deck_json(
        json_value: Option<Value>,
        player_num: usize,
    ) -> Result<json::Decks, GameError> {
        if let Some(value) = json_value {
            serde_json::from_value(value).map_err(|e| GameError::System(SystemError::Json(e)))
        } else {
            let file_path = match player_num {
                PLAYER_1 => DECK_JSON_PATH_P1,
                PLAYER_2 => DECK_JSON_PATH_P2,
                _ => return Err(GameError::System(SystemError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "Path not found")))),
            };

            let mut file = File::open(file_path).map_err(|e| GameError::System(SystemError::Io(e)))?;
            let mut json_data = String::new();
            file.read_to_string(&mut json_data)
                .map_err(|e| GameError::System(SystemError::Io(e)))?;

            serde_json::from_str(&json_data).map_err(|e| GameError::System(SystemError::Json(e)))
        }
    }

    fn create_card_vector(decks: &json::Decks, keys: &Keys, num: i32) -> Vec<i32> {
        decks.decks[0]
            .cards
            .iter()
            .filter(|card| card.num == num)
            .filter_map(|card| keys.get_usize_by_string(&card.id))
            .collect()
    }

    fn generate_deck_code(
        player_num: usize,
        json_value: Option<Value>,
    ) -> Result<String, GameError> {
        let decks = parse_deck_json(json_value, player_num)?;
        let keys = Keys::new();

        // deckcode 에서 카드 1장 인 것과 2장 인 것을 따로 생성함.
        let card1 = create_card_vector(&decks, &keys, 1);
        let card2 = create_card_vector(&decks, &keys, 2);

        let dbf_hero = 930;
        let format = 2;

        Ok(deck_encode(card1, card2, dbf_hero, format))
    }

    // 두 플레이어의 덱 코드 생성
    let p1_code = generate_deck_code(PLAYER_1, p1_card_json)?;
    let p2_code = generate_deck_code(PLAYER_2, p2_card_json)?;

    Ok((p1_code, p2_code))
}

pub fn deckcode_to_cards_single(deckcode: String) -> Result<Cards, GameError> {
    // TODO: 거대한 카드 json 을 한 번에 읽어오는 것보다, 필요한 카드만 읽어오는 방법으로 개선해야함.
    //       (예: JSON 스트리밍 파서 사용 또는 데이터베이스 사용)
    let file_path = CARD_JSON_PATH;

    let mut file = File::open(file_path).map_err(|e| {
        warn!(
            "Failed to open card JSON file at {}: {}. Error: {}",
            file_path, CARD_JSON_PATH, e
        );
        GameError::System(SystemError::Io(e))
    })?;

    let mut json_data = String::new();
    file.read_to_string(&mut json_data).map_err(|e| {
        warn!(
            "Failed to read card JSON file: {}. Error: {}",
            CARD_JSON_PATH, e
        );
        GameError::System(SystemError::Io(e))
    })?;

    let all_cards_data: Vec<json::CardJson> = match serde_json::from_str(&json_data[..]) {
        Ok(data) => data,
        Err(e) => {
            warn!(
                "Failed to parse card JSON data from {}. Error: {}",
                CARD_JSON_PATH, e
            );
            return Err(GameError::System(SystemError::Json(e)));
        }
    };

    let decoded_deck = match deck_decode(deckcode) {
        Ok(data) => data,
        Err(_) => {
            warn!("Failed to decode deck code.");
            return Err(GameError::Gameplay(GameplayError::DeckError(DeckError::ParseFailed("Deck decode failed".to_string()))));
        }
    };

    let card_generator = CardGenerator::new();
    let mut deck_cards: Cards = Vec::with_capacity(MAX_CARD_SIZE);

    // all_cards_data를 dbfid 기준으로 HashMap으로 만들어 빠른 조회를 가능하게 합니다.
    let card_data_map: HashMap<i32, &json::CardJson> = all_cards_data
        .iter()
        .filter_map(|cd| cd.dbfid.map(|id| (id, cd)))
        .collect();

    // decoded_deck.0 (1장씩 있는 카드 ID 리스트) 처리
    for &dbfid in &decoded_deck.0 {
        if let Some(card_data) = card_data_map.get(&dbfid) {
            let card = card_generator.gen_card_by_id_i32(dbfid, card_data, 1);
            deck_cards.push(card);
        } else {
            warn!(
                "Card data not found for dbfid (1-copy): {}. This card will be skipped.",
                dbfid
            );
        }
    }

    // decoded_deck.1 (2장씩 있는 카드 ID 리스트) 처리
    for &dbfid in &decoded_deck.1 {
        if let Some(card_data) = card_data_map.get(&dbfid) {
            // `count`가 2이므로, `gen_card_by_id_i32`를 두 번 호출하여
            // 각각의 카드 인스턴스를 생성합니다.
            let card1 = card_generator.gen_card_by_id_i32(dbfid, card_data, 1); // 첫 번째 인스턴스
            deck_cards.push(card1);

            let card2 = card_generator.gen_card_by_id_i32(dbfid, card_data, 1); // 두 번째 인스턴스
            deck_cards.push(card2);
        } else {
            warn!(
                "Card data not found for dbfid (2-copy): {}. These cards will be skipped.",
                dbfid
            );
        }
    }

    Ok(deck_cards)
}

pub fn load_card_id() -> Result<Vec<(String, i32)>, GameError> {
    let file_path = CARD_ID_JSON_PATH;

    // 파일 열기
    let mut file = File::open(file_path).expect("Failed to open file");

    // 파일 내용을 문자열로 읽기
    let mut json_data = String::new();
    file.read_to_string(&mut json_data)
        .expect("Failed to read file");

    let card_json: Vec<json::Item> = match serde_json::from_str(&json_data[..]) {
        Ok(data) => data,
        Err(e) => return Err(GameError::System(SystemError::Json(e))),
    };

    let mut ids = vec![];

    for item in &card_json {
        ids.push((item.id.clone(), item.dbfid));
    }
    Ok(ids)
}

const DECK_CODE_VERSION: u32 = 1;
pub fn deck_decode(deck_code: String) -> Result<(Vec<i32>, Vec<i32>), ()> {
    let code = decode(deck_code).unwrap();
    let mut pos = 0;

    let read_varint = |pos: &mut usize| {
        let mut shift = 0;
        let mut result = 0;

        loop {
            if *pos >= code.len() {
                return Err(());
            }

            let ch = code[*pos] as usize;

            *pos += 1;

            result |= (ch & 0x7f) << shift;
            shift += 7;

            if (ch & 0x80) == 0 {
                break;
            }
        }
        return Ok(result);
    };

    if code[pos] as char != '\0' {
        println!("Invalid deck code");
        return Err(());
    }
    pos += 1;

    match read_varint(&mut pos) {
        Ok(version) => {
            if version as u32 != DECK_CODE_VERSION {
                println!("Version mismatch");
                return Err(());
            }
        }
        Err(_) => {
            println!("version err");
            return Err(());
        }
    }

    let format = read_varint(&mut pos);
    match format {
        Ok(_) => {}
        Err(_) => {
            println!("Invalid format type");
            return Err(());
        }
    }

    let num = read_varint(&mut pos);
    match num {
        Ok(data) => {
            if data != 1 {
                println!("Hero count must be 1");
                return Err(());
            }
        }
        Err(_) => return Err(()),
    }

    let hero_type = read_varint(&mut pos);
    let _hero_type = match hero_type {
        Ok(hero_id) => hero_id,
        Err(_) => {
            return Err(());
        }
    };

    //Deck deckInfo(format, hero->GetCardClass());
    let mut _1_cards = vec![];
    let mut _2_cards = vec![];

    // Single-copy cards
    let num = read_varint(&mut pos).unwrap();
    for _idx in 0..num {
        let card_id = read_varint(&mut pos).unwrap();
        _1_cards.push(card_id as i32);
    }

    // 2-copy cards
    let num = read_varint(&mut pos).unwrap();
    for _idx in 0..num {
        let card_id = read_varint(&mut pos).unwrap();
        _2_cards.push(card_id as i32);
        // deckInfo.AddCard(Cards::FindCardByDbfID(cardID)->id, 2);
    }

    // 하스스톤은 덱에서 같은 카드를 세 개 이상 구성하지 못함. ( 최대 2개 ) 근데 왜 n-copy 코드가 있는지는 잘 모르겠음..
    // n-copy cards
    let num = read_varint(&mut pos).unwrap();
    for _idx in 0..num {
        let _card_id = read_varint(&mut pos).unwrap();
        let _count = read_varint(&mut pos).unwrap();
        // deckInfo.AddCard(Cards::FindCardByDbfID(cardID)->id, count);
    }
    // println!("{:#?} {:#?}", _1_cards, _2_cards);
    Ok((_1_cards, _2_cards))
}

fn write_varint<W: Write>(writer: &mut W, mut value: usize) -> std::io::Result<()> {
    loop {
        let mut temp: u8 = (value & 0b01111111) as u8;
        value >>= 7;
        if value != 0 {
            temp |= 0b10000000;
        }
        writer.write_u8(temp)?;
        if value == 0 {
            break;
        }
    }
    Ok(())
}

fn deck_encode(deck1: Vec<i32>, deck2: Vec<i32>, dbf_hero: usize, format: usize) -> String {
    let mut baos = Cursor::new(Vec::new());

    write_varint(&mut baos, 0).unwrap(); // always zero
    write_varint(&mut baos, 1).unwrap(); // encoding version number
    write_varint(&mut baos, format).unwrap(); // standard = 2, wild = 1
    write_varint(&mut baos, 1).unwrap(); // number of heroes in heroes array, always 1
    write_varint(&mut baos, dbf_hero).unwrap(); // DBF ID of hero

    write_varint(&mut baos, deck1.len() as usize).unwrap(); // number of 1-quantity cards
    for dbf_id in &deck1 {
        write_varint(&mut baos, *dbf_id as usize).unwrap();
    }

    write_varint(&mut baos, deck2.len() as usize).unwrap(); // number of 2-quantity cards
    for dbf_id in &deck2 {
        write_varint(&mut baos, *dbf_id as usize).unwrap();
    }

    write_varint(&mut baos, 0).unwrap(); // the number of cards that have quantity greater than 2. Always 0 for constructed

    let deck_bytes = baos.into_inner();

    let deck_string = encode(&deck_bytes);

    deck_string
}
