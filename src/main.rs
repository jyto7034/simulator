use base64::{decode, encode};
use byteorder::WriteBytesExt;
use std::io::{Cursor, Write};

struct Deck {}

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
    println!("1");
    for idx in 0..num {
        let cardID = read_varint(&mut pos).unwrap();
        println!("{}", cardID);
        // deckInfo.AddCard(Cards::FindCardByDbfID(cardID)->id, 1);
    }

    // 2-copy cards
    println!("2");
    let num = read_varint(&mut pos).unwrap();
    for idx in 0..num {
        let cardID = read_varint(&mut pos).unwrap();
        println!("{}", cardID);
        // deckInfo.AddCard(Cards::FindCardByDbfID(cardID)->id, 2);
    }

    // n-copy cards
    println!("n");
    let num = read_varint(&mut pos).unwrap();
    // for idx in 0..num {
    //     let cardID = read_varint(&mut pos).unwrap();
    //     let count = read_varint(&mut pos).unwrap();
    //     println!("{}, {}", cardID, count);
    //     // deckInfo.AddCard(Cards::FindCardByDbfID(cardID)->id, count);
    // }

    Ok(Deck {})
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

fn serialize_deck(deck1: Vec<i32>, deck2: Vec<i32>, dbf_hero: i32, format: i32) -> String {
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

fn main() {
    let deck1 = vec![72536, 72923, 86092, 86120, 86626, 101033]; // Example deck1 DBF IDs
    let deck2 = vec![
        69622, 69623, 72119, 77556, 77557, 82369, 86109, 86111, 86112, 86209, 101557, 101698,
    ]; // Example deck2 DBF IDs
    let dbf_hero = 930; // Example hero DBF ID
    let format = 2; // Example format (standard)

    let deck_string = serialize_deck(deck1, deck2, dbf_hero, format);
    println!("{deck_string}");
    let deck_string1 =
        "AAECAaIHBti2BNu5BMygBeigBeKkBamVBgz2nwT3nwS3swT03QT13QTBgwXdoAXfoAXgoAXBoQW1mQbCmgYA"
            .to_string();
    let deck_string2 =
        "AAECAQcG2LYE27kEzKAF6KAF4qQFqZUGDPafBPefBLezBPTdBPXdBMGDBd2gBd+gBeCgBcGhBbWZBsKaBgA="
            .to_string();
    match deck_decode(deck_string1) {
        Ok(_) => println!("ok"),
        Err(_) => println!("no"),
    }

    println!("#####################################");
    match deck_decode(deck_string2) {
        Ok(_) => println!("ok"),
        Err(_) => println!("no"),
    }
}
