use uuid::Uuid;

#[derive(Clone)]
pub struct MulliganState {
    player_ready: bool,
    select_cards: Vec<Uuid>,
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

    pub fn get_select_cards(&self) -> Vec<Uuid> {
        self.select_cards.clone()
    }

    pub fn add_select_cards(&mut self, cards: Vec<Uuid>) {
        self.select_cards.extend(cards);
    }

    pub fn remove_select_cards(&mut self, cards: Vec<Uuid>) {
        self.select_cards.retain(|x| !cards.contains(x));
    }

    pub fn is_ready(&self) -> bool {
        self.player_ready
    }
}
