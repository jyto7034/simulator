use crate::deck::Card;
use crate::enums::constant::{self, CardType, SpellType};
use crate::exception::exception::Exception;
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
        self.is_deck_empty()?;

        Ok(self.find_by_card_type(CardType::Spell(spell_type), cnt))
    }

    fn draw_random(&self) -> Result<&Card, Exception> {
        self.is_deck_empty()?;

        let mut rng = rand::thread_rng();
        let random_number = rng.gen_range(0..self.v_card.len());

        Ok(&self.v_card[random_number])
    }

    fn draw_bottom(&self) -> Result<&Card, Exception> {
        match self.v_card.first() {
            Some(card) => Ok(card),
            None => Err(Exception::NoCardsLeft),
        }
    }

    fn draw_top(&self) -> Result<&Card, Exception> {
        self.is_deck_empty()?;

        Ok(&self.v_card.last().unwrap())
    }

    fn draw_by_card_type(&self, card_type: CardType, cnt: usize) -> Result<Vec<&Card>, Exception> {
        self.is_deck_empty()?;

        Ok(self.find_by_card_type(card_type, cnt))
    }

    fn find_by_uuid(&self, uuid: String, cnt: usize) -> Vec<&Card> {
        let ans: Vec<_> = self
            .v_card
            .iter()
            .filter(|item| item.uuid.cmp(&uuid) == std::cmp::Ordering::Equal)
            .take(cnt as usize)
            .collect();

        ans
    }

    fn find_by_name(&self, name: String, cnt: usize) -> Vec<&Card> {
        let ans: Vec<_> = self
            .v_card
            .iter()
            .filter(|item| item.name.cmp(&name) == std::cmp::Ordering::Equal)
            .take(cnt as usize)
            .collect();

        ans
    }

    fn find_by_card_type(&self, card_type: CardType, cnt: usize) -> Vec<&Card> {
        let filter = |cond: CardType| {
            let filtered: Vec<_> = self
                .v_card
                .iter()
                .filter(|item| item.card_type == cond)
                .take(cnt)
                .collect();
            filtered
        };

        match card_type {
            CardType::Dummy => {
                vec![]
            }
            CardType::Agent => filter(CardType::Agent),
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
    pub fn search(
        &self,
        find_type: constant::FindType,
        count_of_card: Option<usize>,
    ) -> Vec<&Card> {
        // 100 대신 덱의 카드 갯수로 바꿔야함.
        let cnt = count_of_card.unwrap_or(100);
        use constant::*;

        match find_type {
            FindType::FindByUUID(uuid) => self.find_by_uuid(uuid, cnt),
            FindType::FindByName(name) => self.find_by_name(name, cnt),
            FindType::FindByCardType(card_type) => self.find_by_card_type(card_type, cnt),
        }
    }

    pub fn draw(
        &mut self,
        draw_type: constant::CardDrawType,
        count_of_card: Option<usize>,
    ) -> Vec<&Card> {
        use constant::*;
        let cnt = count_of_card.unwrap_or(100);

        match draw_type {
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
            CardDrawType::CardType(CardType::Agent) => {
                self.draw_by_card_type(CardType::Agent, cnt).unwrap()
            }
            _ => {
                vec![]
            }
        }
    }
}
