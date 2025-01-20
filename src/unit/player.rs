use crate::{
    card::{Card, cards::Cards},
    enums::{CardDrawType, ChoiceType, InsertType, PlayerType, ZoneType},
    exception::Exception,
    zone::{
        deck_zone::DeckZone, effect_zone::EffectZone, graveyard_zone::GraveyardZone,
        hand_zone::HandZone, zone::Zone,
    },
    OptRcRef,
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
#[derive(Clone)]
pub struct Player {
    pub opponent: OptRcRef<Player>,
    player_type: PlayerType,
    cards: Cards,
    cost: Resoruce,
    mana: Resoruce,

    hand_zone: HandZone,
    deck_zone: DeckZone,
    graveyard_zone: GraveyardZone,
    effect_zone: EffectZone,
}

impl Player {
    pub fn new(
        opponent: OptRcRef<Player>,
        player_type: PlayerType,
        cards: Cards,
        cost: Resoruce,
        mana: Resoruce,
    ) -> Player {
        Player {
            opponent,
            player_type,
            cards,
            hand_zone: HandZone::new(),
            deck_zone: DeckZone::new(),
            graveyard_zone: GraveyardZone::new(),
            effect_zone: EffectZone::new(),
            cost,
            mana,
        }
    }

    // --------------------------------------------------------
    // 주어진 파라미터에 따라 draw 합니다.
    // 만약 count 가 해당 Zone 이 갖고 있는 카드의 갯수를 초과한다면
    // Zone 이 갖고 있는 만큼만 return 합니다.
    // --------------------------------------------------------
    // TODO:
    //  - [?] 객체 단위 관리로 변경해야함.
    //  - draw 의 Result 제대로 처리해야함.
    // --------------------------------------------------------
    pub fn draw(
        &mut self,
        zone_type: ZoneType,
        draw_type: CardDrawType,
    ) -> Result<Vec<Card>, Exception> {
        // 전처리 해야됨. 아마도

        self.get_zone(zone_type).get_cards().draw(draw_type)
    }

    // --------------------------------------------------------
    // ChoiceType 에 따라 처리합니다.
    // --------------------------------------------------------
    // Parameters:
    // --------------------------------------------------------
    // Exceptions:
    // --------------------------------------------------------
    pub fn choice_card(&mut self, choice_type: ChoiceType) -> Vec<Card> {
        match choice_type {
            ChoiceType::Mulligun => todo!(),
            ChoiceType::Target => todo!(),
        }
    }

    pub fn add_card(
        &mut self,
        zone_type: ZoneType,
        card: Card,
        insert_type: InsertType,
    ) -> Result<(), Exception> {
        self.get_zone(zone_type).add_card(card, insert_type)
    }

    pub fn get_opponent(&self) -> &OptRcRef<Player> {
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

    pub fn get_zone(&mut self, zone_type: ZoneType) -> Box<&mut dyn Zone> {
        match zone_type {
            ZoneType::HandZone => Box::new(&mut self.hand_zone),
            ZoneType::DeckZone => Box::new(&mut self.deck_zone),
            ZoneType::GraveyardZone => Box::new(&mut self.graveyard_zone),
            ZoneType::EffectZone => Box::new(&mut self.effect_zone),
            ZoneType::None => todo!(),
        }
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
