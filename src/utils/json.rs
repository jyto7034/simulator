use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Hero {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Card {
    pub id: String,
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
