use crate::{
    card::{types::PlayerType, Card},
    enums::ZoneType,
    zone::{deck::Deck, hand::Hand, zone::Zone},
};

use super::Game;

impl Game {
    pub fn get_cards_by_player_and_zone_type(
        &self,
        player_type: PlayerType,
        zone_type: ZoneType,
    ) -> Vec<Card> {
        match (player_type, zone_type) {
            (PlayerType::Player1, ZoneType::Hand) => self.get_player_hand_cards(),
            (PlayerType::Player1, ZoneType::Field) => self.get_player_field_cards(),
            (PlayerType::Player1, ZoneType::Deck) => self.get_player_deck_cards(),
            (PlayerType::Player1, ZoneType::Graveyard) => self.get_player_graveyard_cards(),

            (PlayerType::Player2, ZoneType::Hand) => self.get_opponent_hand_cards(),
            (PlayerType::Player2, ZoneType::Field) => self.get_opponent_field_cards(),
            (PlayerType::Player2, ZoneType::Deck) => self.get_opponent_deck_cards(),
            (PlayerType::Player2, ZoneType::Graveyard) => self.get_opponent_graveyard_cards(),

            (PlayerType::None, _) => panic!("Player type is not allowed to be None"),
            (_, ZoneType::Effect) => todo!(),
            (_, ZoneType::None) => panic!("Zone type is not allowed to be None"),
        }
    }

    pub fn get_player_field_cards(&self) -> Vec<Card> {
        self.get_player()
            .get()
            .get_field()
            .get_cards()
            .clone()
    }

    pub fn with_player_field_cards<F, C>(&mut self, mut condition: C, modifier: F)
    where
        C: FnMut(&Card) -> bool,
        F: FnMut(&mut Card),
    {
        self.get_player()
            .get()
            .get_field_mut()
            .get_cards_mut()
            .iter_mut()
            .filter(|card| condition(card))
            .for_each(modifier);
    }

    pub fn get_opponent_field_cards(&self) -> Vec<Card> {
        self.get_opponent()
            .get()
            .get_field()
            .get_cards()
            .clone()
    }

    pub fn with_opponent_field_cards<F, C>(&mut self, mut condition: C, modifier: F)
    where
        C: FnMut(&Card) -> bool,
        F: FnMut(&mut Card),
    {
        self.get_opponent()
            .get()
            .get_field_mut()
            .get_cards_mut()
            .iter_mut()
            .filter(|card| condition(card))
            .for_each(modifier);
    }

    // 핸드 카드
    pub fn get_player_hand_cards(&self) -> Vec<Card> {
        self.get_player()
            .get()
            .get_hand()
            .get_cards()
            .clone()
    }

    pub fn with_player_hand_cards<F, C>(&mut self, mut condition: C, modifier: F)
    where
        C: FnMut(&Card) -> bool,
        F: FnMut(&mut Card),
    {
        self.get_player()
            .get()
            .get_hand_mut()
            .get_cards_mut()
            .iter_mut()
            .filter(|card| condition(card))
            .for_each(modifier);
    }

    pub fn get_opponent_hand_cards(&self) -> Vec<Card> {
        self.get_opponent()
            .get()
            .get_hand()
            .get_cards()
            .clone()
    }

    pub fn with_opponent_hand_cards<F, C>(&mut self, mut condition: C, modifier: F)
    where
        C: FnMut(&Card) -> bool,
        F: FnMut(&mut Card),
    {
        self.get_opponent()
            .get()
            .get_hand_mut()
            .get_cards_mut()
            .iter_mut()
            .filter(|card| condition(card))
            .for_each(modifier);
    }

    // 묘지 카드
    pub fn get_player_graveyard_cards(&self) -> Vec<Card> {
        self.get_player()
            .get()
            .get_graveyard()
            .get_cards()
            .clone()
    }

    pub fn with_player_graveyard_cards<F, C>(&mut self, mut condition: C, modifier: F)
    where
        C: FnMut(&Card) -> bool,
        F: FnMut(&mut Card),
    {
        self.get_player()
            .get()
            .get_graveyard_mut()
            .get_cards_mut()
            .iter_mut()
            .filter(|card| condition(card))
            .for_each(modifier);
    }

    pub fn get_opponent_graveyard_cards(&self) -> Vec<Card> {
        self.get_opponent()
            .get()
            .get_graveyard()
            .get_cards()
            .clone()
    }

    pub fn with_opponent_graveyard_cards<F, C>(&mut self, mut condition: C, modifier: F)
    where
        C: FnMut(&Card) -> bool,
        F: FnMut(&mut Card),
    {
        self.get_opponent()
            .get()
            .get_graveyard_mut()
            .get_cards_mut()
            .iter_mut()
            .filter(|card| condition(card))
            .for_each(modifier);
    }

    // 덱 카드
    pub fn get_player_deck_cards(&self) -> Vec<Card> {
        self.get_player()
            .get()
            .get_deck()
            .get_cards()
            .clone()
    }

    pub fn with_player_deck_cards<F, C>(&mut self, mut condition: C, modifier: F)
    where
        C: FnMut(&Card) -> bool,
        F: FnMut(&mut Card),
    {
        self.get_player()
            .get()
            .get_deck_mut()
            .get_cards_mut()
            .iter_mut()
            .filter(|card| condition(card))
            .for_each(modifier);
    }

    pub fn get_opponent_deck_cards(&self) -> Vec<Card> {
        self.get_opponent()
            .get()
            .get_deck()
            .get_cards()
            .clone()
    }

    pub fn with_opponent_deck_cards<F, C>(&mut self, mut condition: C, modifier: F)
    where
        C: FnMut(&Card) -> bool,
        F: FnMut(&mut Card),
    {
        self.get_opponent()
            .get()
            .get_deck_mut()
            .get_cards_mut()
            .iter_mut()
            .filter(|card| condition(card))
            .for_each(modifier);
    }
}
