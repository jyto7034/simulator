use crate::{card::types::PlayerType, enums::UUID};

pub struct MulliganState {
    player_ready: bool,
    select_cards: Vec<UUID>,
}

impl MulliganState {
    pub fn new() -> Self {
        Self {
            player_ready: false,
            select_cards: vec![],
        }
    }

    pub fn confirm_selection(&mut self) {
        self.player_ready = true;
    }

    pub fn get_select_cards(&mut self) -> &mut Vec<UUID> {
        &mut self.select_cards
    }

    pub fn is_ready(&self) -> bool {
        self.player_ready
    }
}
