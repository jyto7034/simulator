use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Item {
    pub id: String,
    pub dbfid: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Card {
    pub id: String,
    pub num: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Hero {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Deck {
    pub Hero: Vec<Hero>,
    pub cards: Vec<Card>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Decks {
    pub decks: Vec<Deck>,
}

#[derive(Debug, Deserialize)]
pub struct Names {
    pub name1: String,
    pub name2: String,
}

#[derive(Debug, Deserialize)]
pub struct DeckCodes {
    pub code1: String,
    pub code2: String,
}

#[derive(Debug, Deserialize)]
pub struct GameConfigJson {
    pub DeckCodes: Vec<DeckCodes>,
    pub Attacker: i32,
    pub Names: Vec<Names>,
}

/*
    "id": "AT_010",
    "attack": 3,
    "health": 3,
    "cost": 5,
    "rarity": "RARE",
    "collectible": true,
    "name": "Ram Wrangler",
    "text": "<b>Battlecry:</b> If you have a Beast, summon a\nrandom Beast.",
    "type": "Agent"
*/
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CardJson {
    pub id: Option<String>,
    pub dbfid: Option<i32>,
    pub cost: Option<i32>,
    pub name: Option<String>,
    pub text: Option<String>,
    pub attack: Option<i32>,
    pub health: Option<i32>,
    pub collectible: Option<bool>,
}

impl CardJson {
    pub fn new() -> CardJson {
        CardJson {
            id: None,
            dbfid: None,
            cost: None,
            name: None,
            text: None,
            attack: None,
            health: None,
            collectible: None,
        }
    }
}
