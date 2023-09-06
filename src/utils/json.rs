use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Hero {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Card {
    pub id: i32,
    pub num: u32,
}

#[derive(Debug, Deserialize)]
pub struct Deck {
    pub hero: Vec<Hero>,
    pub cards: Vec<Card>,
}

#[derive(Debug, Deserialize)]
pub struct Decks {
    pub decks: Vec<Deck>,
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
#[derive(Debug, Deserialize, Serialize)]
pub struct CardJson {
    pub id: Option<i32>,
    pub cost: Option<i32>,
    pub name: Option<String>,
    pub text: Option<String>,
    pub attack: Option<i32>,
    pub health: Option<i32>,
    pub collectible: Option<bool>,
}