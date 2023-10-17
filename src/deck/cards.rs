use crate::deck::Card;
use crate::enums::constant::{self, CardType, SpellType};
use crate::exception::exception::Exception;
use crate::game::IResource;
use rand::Rng;

/// 다수의 카드를 보다 더 효율적으로 관리하기 위한 구조체입니다.
/// 예를 들어 카드 서치, 수정 등이 있습니다.
#[derive(Debug)]
pub struct Cards {
    pub v_card: Vec<Card>,
}

impl Cards {
    fn is_deck_empty(&self) -> Result<(), Exception> {
        if self.v_card.is_empty() {
            return Err(Exception::NoCardsLeft);
        } else {
            return Ok(());
        }
    }

    fn draw_spell(&self, spell_type: SpellType, cnt: usize) -> Result<Vec<&Card>, Exception> {
        // Deck 이 비어 있는지 확인합니다.
        self.is_deck_empty()?;

        Ok(self.find_by_card_type(CardType::Spell(spell_type), cnt))
    }

    fn draw_random(&self) -> Result<&Card, Exception> {
        self.is_deck_empty()?;

        let mut rng = rand::thread_rng();
        let random_number = rng.gen_range(0..self.v_card.len());

        Ok(&self.v_card[random_number])
    }

    fn draw_bottom(&mut self) -> Result<&Card, Exception> {
        self.is_deck_empty()?;

        match &mut self.v_card.first() {
            Some(card) => {
                if card.get_count().get() != 0 {
                    Ok(card)
                } else {
                    Err(Exception::NoCardLeft)
                }
            }
            None => Err(Exception::NoCardsLeft),
        }
    }

    fn draw_top(&self) -> Result<&Card, Exception> {
        self.is_deck_empty()?;

        let card = self.v_card.last().unwrap();
        if card.get_count().get() != 0 {
            Ok(card)
        } else {
            Err(Exception::NoCardLeft)
        }
    }

    fn draw_by_card_type(&self, card_type: CardType, cnt: usize) -> Result<Vec<&Card>, Exception> {
        self.is_deck_empty()?;

        Ok(self.find_by_card_type(card_type, cnt))
    }

    fn find_by_uuid(&self, uuid: String, cnt: usize) -> Vec<&Card> {
        // uuid 에 해당하는 카드를 집계합니다.
        // count 가 0 개인 경우, 스킵하고 다음 카드를 찾습니다.
        let ans: Vec<_> = self
            .v_card
            .iter()
            .filter(|item| {
                item.get_uuid().cmp(&uuid) == std::cmp::Ordering::Equal
                    && item.get_count().get() != 0
            })
            .take(cnt as usize)
            .collect();

        ans
    }

    fn find_by_name(&self, name: String, cnt: usize) -> Vec<&Card> {
        // name 에 해당하는 카드를 집계합니다.
        // count 가 0 개인 경우, 스킵하고 다음
        let ans: Vec<_> = self
            .v_card
            .iter()
            .filter(|item| {
                item.get_name().cmp(&name) == std::cmp::Ordering::Equal
                    && item.get_count().get() != 0
            })
            .take(cnt as usize)
            .collect();

        ans
    }

    fn find_by_card_type(&self, card_type: CardType, cnt: usize) -> Vec<&Card> {
        // cond 에 해당하는 카드를 집계합니다.
        // count 가 0 개인 경우, 스킵하고 다음 카드를 찾습니다.
        let filter = |cond: CardType| {
            let filtered: Vec<_> = self
                .v_card
                .iter()
                .filter(|item| item.get_card_type() == &cond && item.get_count().get() != 0)
                .take(cnt)
                .collect();
            filtered
        };

        match card_type {
            CardType::Dummy => {
                vec![]
            }
            CardType::Unit => filter(CardType::Unit),
            CardType::Field => filter(CardType::Field),
            CardType::Spell(SpellType::FastSpell) => filter(CardType::Spell(SpellType::FastSpell)),
            CardType::Spell(SpellType::SlowSpell) => filter(CardType::Spell(SpellType::SlowSpell)),
        }
    }
}

impl Cards {
    pub fn new(cards: &Vec<Card>) -> Cards {
        Cards {
            v_card: cards.to_vec(),
        }
    }

    pub fn len(&self) -> usize {
        self.v_card.len()
    }

    pub fn push(&mut self, card: &Card) {
        self.v_card.push(card.clone());
    }

    pub fn dummy() -> Cards {
        Cards { v_card: vec![] }
    }

    pub fn get_card_count(&self) -> u32 {
        constant::MAX_CARD_SIZE
    }

    pub fn empty(&self) -> bool {
        false
    }

    /// 주어진 검색 조건으로 카드를 찾습니다.
    pub fn search(&self, find_type: constant::FindType, count_of_card: usize) -> Vec<&Card> {
        // 100 대신 덱의 카드 갯수로 바꿔야함.

        // find 함수가 카드를 몇 개까지 찾게 할 지 정하는 변수.
        let cnt = if count_of_card == 0 {
            100
        } else {
            count_of_card
        };
        use constant::*;

        match find_type {
            FindType::FindByUUID(uuid) => self.find_by_uuid(uuid, cnt),
            FindType::FindByName(name) => self.find_by_name(name, cnt),
            FindType::FindByCardType(card_type) => self.find_by_card_type(card_type, cnt),
        }
    }

    // 덱으로부터 카드 한 장을 draw 합니다.
    //
    pub fn draw(&mut self, draw_type: constant::CardDrawType, count_of_card: usize) -> Vec<&Card> {
        use constant::*;

        // find 함수가 카드를 몇 개까지 찾게 할 지 정하는 변수.
        let cnt = if count_of_card == 0 {
            100
        } else {
            count_of_card
        };

        let decrease = |cards: &mut Vec<Card>, card_uuid: UUID| {
            cards.iter_mut().for_each(|item| {
                if item.get_uuid() == &card_uuid {
                    item.get_count().decrease();
                }
            });
        };

        let v_cards = match draw_type {
            CardDrawType::Top => {
                vec![self.draw_top().unwrap()]
            }
            CardDrawType::Random => {
                vec![self.draw_random().unwrap()]
            }
            CardDrawType::Bottom => {
                vec![self.draw_bottom().unwrap()]
            }
            CardDrawType::CardType(CardType::Spell(SpellType::FastSpell)) => {
                self.draw_spell(SpellType::FastSpell, cnt).unwrap()
            }
            CardDrawType::CardType(CardType::Spell(SpellType::SlowSpell)) => {
                self.draw_spell(SpellType::SlowSpell, cnt).unwrap()
            }
            CardDrawType::CardType(CardType::Field) => {
                self.draw_by_card_type(CardType::Field, cnt).unwrap()
            }
            CardDrawType::CardType(CardType::Unit) => {
                self.draw_by_card_type(CardType::Unit, cnt).unwrap()
            }
            _ => {
                vec![]
            }
        };

        // Draw 된 카드들의 count 를 하나씩 감소시킵니다.
        for item in v_cards {
            decrease(&mut self.v_card, item.get_uuid().clone());
        }

        v_cards
    }
}
