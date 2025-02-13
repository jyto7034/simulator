use crate::{
    card::{cards::Cards, types::PlayerType},
    selector::mulligan::MulliganState,
    zone::{deck::Deck, effect::Effect, field::Field, graveyard::Graveyard, hand::Hand},
    OptArc,
};

#[derive(Clone, Debug)]
pub struct Resoruce {
    cost: usize,
    limit: usize,
}

impl Resoruce {
    pub fn new(cost: usize, limit: usize) -> Resoruce {
        Resoruce { cost, limit }
    }
    pub fn is_empty(&self) -> bool {
        self.cost == 0
    }
    fn increase(&mut self) -> &mut Self {
        self.cost += 1;
        self
    }

    fn decrease(&mut self) -> &mut Self {
        self.cost -= 1;
        self
    }

    fn set(&mut self, cost: usize) -> &mut Self {
        self.cost = cost;
        self
    }

    fn get(&self) -> usize {
        self.cost
    }

    fn add(&mut self, cost: usize) {
        // TODO!!
        // 추가하고자 하는 리소스가 limit 을 넘어가는지 확인하고 제한해야됨.
        self.cost += cost;
    }
}

/// 플레이어를 행동, 상태 등을 다루는 구조체 입니다.
pub struct Player {
    pub opponent: OptArc<Player>,
    player_type: PlayerType,
    mulligan_state: MulliganState,
    cards: Cards,
    cost: Resoruce,
    mana: Resoruce,

    hand: Hand,
    deck: Deck,
    graveyard: Graveyard,
    effect: Effect,
    field: Field,
}

impl Player {
    pub fn new(
        opponent: OptArc<Player>,
        player_type: PlayerType,
        cards: Cards,
        cost: Resoruce,
        mana: Resoruce,
    ) -> Player {
        Player {
            opponent,
            player_type,
            mulligan_state: MulliganState::new(),
            cards,
            hand: Hand::new(),
            deck: Deck::new(),
            graveyard: Graveyard::new(),
            effect: Effect::new(),
            field: Field::new(),
            cost,
            mana,
        }
    }

    pub fn get_mulligan_state_mut(&mut self) -> &mut MulliganState {
        &mut self.mulligan_state
    }

    pub fn get_player_type(&self) -> PlayerType {
        self.player_type
    }

    pub fn get_opponent(&self) -> &OptArc<Player> {
        &self.opponent
    }

    pub fn get_cards(&self) -> &Cards {
        &self.cards
    }

    pub fn get_cost(&mut self) -> &mut Resoruce {
        &mut self.cost
    }

    pub fn get_mana(&mut self) -> &mut Resoruce {
        &mut self.mana
    }

    pub fn set_cards(&mut self, new_cards: Cards) {
        self.cards = new_cards;
    }

    pub fn set_cost(&mut self, cost: usize) {
        self.cost.set(cost);
    }

    pub fn set_mana(&mut self, cost: usize) {
        self.mana.set(cost);
    }
}

impl Player {
    pub fn get_hand_mut(&mut self) -> &mut Hand {
        &mut self.hand
    }

    pub fn get_deck_mut(&mut self) -> &mut Deck {
        &mut self.deck
    }

    pub fn get_graveyard_mut(&mut self) -> &mut Graveyard {
        &mut self.graveyard
    }

    pub fn get_effect_mut(&mut self) -> &mut Effect {
        &mut self.effect
    }

    pub fn get_field_mut(&mut self) -> &mut Field {
        &mut self.field
    }

    pub fn get_hand(&self) -> &Hand {
        &self.hand
    }

    pub fn get_deck(&self) -> &Deck {
        &self.deck
    }

    pub fn get_graveyard(&self) -> &Graveyard {
        &self.graveyard
    }

    pub fn get_effect(&self) -> &Effect {
        &self.effect
    }

    pub fn get_field(&self) -> &Field {
        &self.field
    }
}
