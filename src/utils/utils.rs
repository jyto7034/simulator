use crate::card::card::Card;
use crate::card::cards::Cards;
use crate::card_gen::card_gen::{CardGenerator, Keys};
use crate::enums::constant::{self, DeckCode, PLAYER_1, PLAYER_2, UUID_GENERATOR_PATH};
use crate::exception::exception::Exception;
use crate::utils::json;
use base64::{decode, encode};
use byteorder::WriteBytesExt;
use std::fs::File;
use std::io::Read;
use std::io::{Cursor, Write};
use std::process::{Command, Stdio};
use std::vec;

trait VecExtensions<T> {
    // id 순으로
    fn next_task(&mut self) -> Option<T>;
}

impl<T> VecExtensions<T> for Vec<T>
where
    T: Default + Copy,
{
    fn next_task(&mut self) -> Option<T> {
        if self.len() == 0 {
            None
        } else {
            let data = self.remove(0);
            Some(data)
        }
    }
}

pub fn generate_uuid() -> Result<String, Exception> {
    let output = if let Ok(ans) = Command::new(UUID_GENERATOR_PATH)
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

pub fn read_game_config_json() -> Result<json::GameConfigJson, Exception> {
    let file_path = constant::GAME_CONFIG_JSON_PATH;

    // 파일 열기
    let mut file = File::open(file_path).expect("Failed to open file");

    // 파일 내용을 문자열로 읽기
    let mut json_data = String::new();
    file.read_to_string(&mut json_data)
        .expect("Failed to read file");

    let card_json: json::GameConfigJson = match serde_json::from_str(&json_data[..]) {
        Ok(data) => data,
        Err(_) => return Err(Exception::JsonParseFailed),
    };

    Ok(card_json)
}

pub fn parse_json_to_deck_code() -> Result<(DeckCode, DeckCode), Exception> {
    let json_to_deck_code = |player_num: usize| {
        let file_path = match player_num {
            PLAYER_1 => constant::DECK_JSON_PATH_P1,
            PLAYER_2 => constant::DECK_JSON_PATH_P2,
            _ => return Err(Exception::PathNotExist),
        };

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

        let keys = Keys::new();

        let create_card_vector = |decks: &json::Decks, keys: &Keys, num: i32| -> Vec<i32> {
            decks.decks[0]
                .cards
                .iter()
                .filter(|card| card.num == num)
                .filter_map(|card| keys.get_i32_by_string(&card.id))
                .collect()
        };

        let card1 = create_card_vector(&decks, &keys, 1);
        let card2 = create_card_vector(&decks, &keys, 2);

        // 사전 설정
        let dbf_hero = 930; // Example hero DBF ID
        let format = 2; // Example format (standard)

        let deck_code = deck_encode(card1, card2, dbf_hero, format);

        Ok(deck_code)
    };
    let deck_code = (json_to_deck_code(PLAYER_1), json_to_deck_code(PLAYER_2));
    match deck_code {
        (Ok(code1), Ok(code2)) => return Ok((code1, code2)),
        (Ok(_), Err(err)) => {
            println!("{err}");
            return Err(Exception::DecodeError);
        }
        (Err(err), Ok(_)) => {
            println!("{err}");
            return Err(Exception::DecodeError);
        }
        (Err(_), Err(_)) => Err(Exception::PlayerDataNotIntegrity),
    }
}

pub fn load_card_data(deck_code: (DeckCode, DeckCode)) -> Result<Vec<Cards>, Exception> {
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

    let decoded_deck1 = match deck_decode(deck_code.0) {
        Ok(data) => data,
        Err(_err) => return Err(Exception::JsonParseFailed),
    };
    let decoded_deck2 = match deck_decode(deck_code.1) {
        Ok(data) => data,
        Err(_err) => return Err(Exception::JsonParseFailed),
    };

    use json::CardJson;

    let card_genertor = CardGenerator::new();

    let mut p1_cards = vec![];
    let mut p2_cards = vec![];

    let check_values_exist = |card_data: &CardJson,
                              decoded_deck: &(Vec<i32>, Vec<i32>),
                              p_cards: &mut Vec<Card>|
     -> Result<(), Exception> {
        for dbfid in &decoded_deck.0 {
            match card_data.dbfid {
                Some(_dbfid) => {
                    if &_dbfid == dbfid {
                        p_cards.push(card_genertor.gen_card_by_id_i32(*dbfid, card_data, 1));
                    }
                }
                None => {}
            }
        }
        for dbfid in &decoded_deck.1 {
            match card_data.dbfid {
                Some(_dbfid) => {
                    if &_dbfid == dbfid {
                        p_cards.push(card_genertor.gen_card_by_id_i32(*dbfid, card_data, 2));
                    }
                }
                None => {}
            }
        }
        Ok(())
    };

    // player_cards 에는 플레이어의 덱 정보가 담겨있음.
    // 카드의 종류, 갯수만 있을 뿐, 실질적인 정보는 없고 카드의 id 만 있기 때문에 이것을 사용하여
    // cards.json 에서 데이터를 가져와야함.
    // println!("card_json: {:#?}", card_json);
    for card_data in card_json {
        check_values_exist(&card_data, &decoded_deck1, &mut p1_cards)?;
        check_values_exist(&card_data, &decoded_deck2, &mut p2_cards)?;
    }

    Ok(vec![Cards::new(&p1_cards), Cards::new(&p2_cards)])
}

pub fn load_card_id() -> Result<Vec<(String, i32)>, Exception> {
    let file_path = constant::CARD_ID_JSON_PATH;

    // 파일 열기
    let mut file = File::open(file_path).expect("Failed to open file");

    // 파일 내용을 문자열로 읽기
    let mut json_data = String::new();
    file.read_to_string(&mut json_data)
        .expect("Failed to read file");

    let card_json: Vec<json::Item> = match serde_json::from_str(&json_data[..]) {
        Ok(data) => data,
        Err(_) => return Err(Exception::JsonParseFailed),
    };

    let mut ids = vec![];

    for item in &card_json {
        ids.push((item.id.clone(), item.dbfid));
    }
    Ok(ids)
}

const DECK_CODE_VERSION: u32 = 1;
fn deck_decode(deck_code: String) -> Result<(Vec<i32>, Vec<i32>), ()> {
    let code = decode(deck_code).unwrap();
    let mut pos = 0;

    let read_varint = |pos: &mut usize| {
        let mut shift = 0;
        let mut result = 0;

        loop {
            if *pos >= code.len() {
                return Err(());
            }

            let ch = code[*pos] as i32;

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
        _1_cards.push(card_id);
    }

    // 2-copy cards
    let num = read_varint(&mut pos).unwrap();
    for _idx in 0..num {
        let card_id = read_varint(&mut pos).unwrap();
        _2_cards.push(card_id);
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

fn write_varint<W: Write>(writer: &mut W, mut value: i32) -> std::io::Result<()> {
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

fn deck_encode(deck1: Vec<i32>, deck2: Vec<i32>, dbf_hero: i32, format: i32) -> String {
    let mut baos = Cursor::new(Vec::new());

    write_varint(&mut baos, 0).unwrap(); // always zero
    write_varint(&mut baos, 1).unwrap(); // encoding version number
    write_varint(&mut baos, format).unwrap(); // standard = 2, wild = 1
    write_varint(&mut baos, 1).unwrap(); // number of heroes in heroes array, always 1
    write_varint(&mut baos, dbf_hero).unwrap(); // DBF ID of hero

    write_varint(&mut baos, deck1.len() as i32).unwrap(); // number of 1-quantity cards
    for dbf_id in &deck1 {
        write_varint(&mut baos, *dbf_id).unwrap();
    }

    write_varint(&mut baos, deck2.len() as i32).unwrap(); // number of 2-quantity cards
    for dbf_id in &deck2 {
        write_varint(&mut baos, *dbf_id).unwrap();
    }

    write_varint(&mut baos, 0).unwrap(); // the number of cards that have quantity greater than 2. Always 0 for constructed

    let deck_bytes = baos.into_inner();

    let deck_string = encode(&deck_bytes);

    deck_string
}
