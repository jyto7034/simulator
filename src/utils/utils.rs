use crate::card_gen::card_gen::CardGenertor;
use crate::deck::{Card, Cards, Deck};
use crate::enums::constant::{self, DeckCode, UUID_GENERATOR_PATH};
use crate::exception::exception::Exception;
use crate::utils::json;
use base64::{decode, encode};
use byteorder::WriteBytesExt;
use std::fs::File;
use std::io::Read;
use std::io::{Cursor, Write};
use std::process::{Command, Stdio};

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

pub fn parse_json_to_deck_code() -> Result<DeckCode, Exception> {
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

    let mut card1 = vec![];
    let mut card2 = vec![];
    for card in &decks.decks[0].cards{
        match card.num{
            1 => card1.push(card.dbf_id),
            2 => card2.push(card.dbf_id),
            _ => {},
        }
    }

    Ok("asd".to_string())
}

pub fn load_card_data(player_cards: DeckCode) -> Result<Vec<Cards>, Exception> {
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

    let card_genertor = CardGenertor::new();

    // gen_card_by_id 의 count 임의로 1로 해둠.
    let mut check_values_exist =
        |player_num: usize, card_data: &CardJson| -> Result<(), Exception> {
            // for player_card in &player_cards.decks[0].cards {
            //     if let Some(id) = &card_data.id {
            //         if player_card.id == *id {
            //             for _ in 0..player_card.num {
            //                 ps_cards[player_num].push(card_genertor.gen_card_by_id(
            //                     id.to_string(),
            //                     card_data,
            //                     1,
            //                 ));
            //             }
            //         }
            //     } else {
            //         return Err(Exception::DeckParseError);
            //     }
            // }
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

pub fn load_card_id() -> Result<Vec<String>, Exception> {
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
        ids.push(item.id.clone());
    }
    Ok(ids)
}

const DECK_CODE_VERSION: u32 = 1;
fn deck_decode(deck_code: String) -> Result<Deck, ()> {
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
        println!("{}", code[pos]);
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
        Ok(data) => {
            println!("{}", data);
        }
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
    let hero_type = match hero_type {
        Ok(hero_id) => {
            println!("{}", hero_id);
            hero_id
        }
        Err(_) => {
            return Err(());
        }
    };

    //Deck deckInfo(format, hero->GetCardClass());

    // Single-copy cards
    let num = read_varint(&mut pos).unwrap();
    for idx in 0..num {
        let cardID = read_varint(&mut pos).unwrap();
        println!("{}", cardID);
        // deckInfo.AddCard(Cards::FindCardByDbfID(cardID)->id, 1);
    }

    // 2-copy cards
    let num = read_varint(&mut pos).unwrap();
    for idx in 0..num {
        let cardID = read_varint(&mut pos).unwrap();
        println!("{}", cardID);
        // deckInfo.AddCard(Cards::FindCardByDbfID(cardID)->id, 2);
    }

    // 하스스톤은 덱에서 같은 카드를 세 개 이상 구성하지 못함. ( 최대 2개 ) 근데 왜 n-copy 코드가 있는지는 잘 모르겠음..
    // n-copy cards
    let num = read_varint(&mut pos).unwrap();
    for idx in 0..num {
        let cardID = read_varint(&mut pos).unwrap();
        let count = read_varint(&mut pos).unwrap();
        println!("{}, {}", cardID, count);
        // deckInfo.AddCard(Cards::FindCardByDbfID(cardID)->id, count);
    }

    Ok(Deck {
        raw_deck_code: "".to_string(),
    })
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
