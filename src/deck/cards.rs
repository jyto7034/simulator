use crate::deck::Card;
use crate::enums::constant::{self, CardType, SpellType, UUID};
use crate::exception::exception::Exception;
use crate::game::IResource;
use rand::Rng;

/// 다수의 카드를 보다 더 효율적으로 관리하기 위한 구조체입니다.
/// 예를 들어 카드 서치, 수정 등이 있습니다.
#[derive(Debug, Clone)]
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

    fn draw_spell(&self, spell_type: SpellType, cnt: usize) -> Result<Vec<Card>, Exception> {
        // Deck 이 비어 있는지 확인합니다.
        self.is_deck_empty()?;

        Ok(self.find_by_card_type(CardType::Spell(spell_type), cnt))
    }

    // --------------------------------------------------------
    // cnt 만큼의 갯수만큼 카드를 무작위로 뽑습니다.
    // --------------------------------------------------------
    // Exceptions:
    // --------------------------------------------------------
    fn draw_random(&mut self, cnt: usize) -> Result<Vec<Card>, Exception> {
        self.is_deck_empty()?;
    
        let mut rng = rand::thread_rng();

        // 카드 index 정보를 vec 으로 만듭니다.
        // 이 벡터는 v_card 를 참조하기 위하여 생성됩니다.
        let mut available_indices: Vec<usize> = (0..self.v_card.len()).collect();
        let mut ans: Vec<Card> = vec![];
        
        // available_indices 가 비어있다면, 모든 카드를 참조한 것입니다.
        while !available_indices.is_empty(){
            // 무작위 기능을 사용하여 임의의 card index 를 하나 가져옵니다.
            let random_index = rng.gen_range(0..available_indices.len());
            let random_number = available_indices[random_index];

            // card index 로 해당 카드의 참조를 생성합니다.
            let card = &mut self.v_card[random_number];
            // 해당 카드의 사용 가능 횟수가 남아있는지 확인합니다.
            if !card.get_count().is_empty() {
                // 사용가능한 카드면 해당 카드의 사용 가능 횟수를 차감합니다.
                card.get_count_mut().decrease();
                
                // ans 에 밀어넣습니다.
                ans.push(card.clone());
            } else {
                // 사용 불가능한 카드의 index 이기 때문에, card index 벡터로부터 삭제합니다.
                available_indices.remove(random_index);
            }
        }
    
        // 모든 카드가 다 뽑혔을 경우 예외 처리 또는 결과 반환
        Err(Exception::NoCardLeft)
    }
    

    fn draw_bottom(&self) -> Result<Card, Exception> {
        
        self.is_deck_empty()?;

        match self.v_card.first() {
            Some(card) => {
                if card.get_count().get() != 0 {
                    Ok(card.clone())
                } else {
                    Err(Exception::NoCardLeft)
                }
            }
            None => Err(Exception::NoCardsLeft),
        }
    }

    fn draw_top(&self) -> Result<Card, Exception> {
        self.is_deck_empty()?;
        println!("{:#?}", self.v_card);
        let card = self.v_card.last().unwrap().clone();
        if card.get_count().get() != 0 {
            Ok(card)
        } else {
            Err(Exception::NoCardLeft)
        }
    }

    fn draw_by_card_type(&self, card_type: CardType, cnt: usize) -> Result<Vec<Card>, Exception> {
        self.is_deck_empty()?;

        Ok(self.find_by_card_type(card_type, cnt))
    }

    fn find_by_uuid(&self, uuid: String, cnt: usize) -> Vec<Card> {
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
            .map(|item| item.clone())
            .collect();

        ans
    }

    fn find_by_name(&self, name: String, cnt: usize) -> Vec<Card> {
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
            .map(|item| item.clone())
            .collect();

        ans
    }

    fn find_by_card_type(&self, card_type: CardType, cnt: usize) -> Vec<Card> {
        // cond 에 해당하는 카드를 집계합니다.
        // count 가 0 개인 경우, 스킵하고 다음 카드를 찾습니다.
        let filter = |cond: CardType| {
            let filtered: Vec<Card> = self
                .v_card
                .iter()
                .filter(|item| item.get_card_type() == &cond && item.get_count().get() != 0)
                .take(cnt)
                .map(|item| item.clone())
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

    // --------------------------------------------------------
    // 이미 존재하는 카드의 사용 가능한 횟수를 1 증가 시킵니다.
    // --------------------------------------------------------
    // Exceptions:
    // - 사용 횟수가 제한을 넘어가는지.
    // --------------------------------------------------------
    pub fn add_card(&mut self, card_uuid: UUID) {
        self.v_card.iter_mut().for_each(|card| {
            if card.get_uuid() == &card_uuid {
                card.get_count_mut().increase();
            }
        })
    }

    // --------------------------------------------------------
    // 카드를 새롭게 추가합니다.
    // --------------------------------------------------------
    // Exceptions:
    // - 이미 카드가 존재하는지.
    // --------------------------------------------------------
    pub fn push(&mut self, card: Card) {
        self.v_card.push(card.clone())
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
    pub fn search(&self, find_type: constant::FindType, count: usize) -> Vec<Card> {
        // 100 대신 덱의 카드 갯수로 바꿔야함.

        // find 함수가 카드를 몇 개까지 찾게 할 지 정하는 변수.
        let cnt = if count == 0 {
            100
        } else {
            count
        };
        use constant::*;

        match find_type {
            FindType::FindByUUID(uuid) => self.find_by_uuid(uuid, cnt),
            FindType::FindByName(name) => self.find_by_name(name, cnt),
            FindType::FindByCardType(card_type) => self.find_by_card_type(card_type, cnt),
        }
    }

    // 덱으로부터 카드 n장을 draw 합니다.
    pub fn draw(&mut self, draw_type: constant::CardDrawType) -> Vec<Card> {
        use constant::*;

        let count = match draw_type {
            CardDrawType::Random(count) => count,
            CardDrawType::CardType(_, count) => count,
            _ => 0
        };
        
        // find 함수가 카드를 몇 개까지 찾게 할 지 정하는 변수.
        let cnt = if count == 0 {
            100
        } else {
            count
        };

        // 실제로 draw 하는 부분 입니다.
        let draw_cards = match draw_type {
            CardDrawType::Top => {
                vec![self.draw_top().unwrap()]
            }
            CardDrawType::Random(_) => {
                self.draw_random(cnt).unwrap()
            }
            CardDrawType::Bottom => {
                vec![self.draw_bottom().unwrap()]
            }
            CardDrawType::CardType(CardType::Spell(SpellType::FastSpell), _) => {
                self.draw_spell(SpellType::FastSpell, cnt).unwrap()
            }
            CardDrawType::CardType(CardType::Spell(SpellType::SlowSpell), _) => {
                self.draw_spell(SpellType::SlowSpell, cnt).unwrap()
            }
            CardDrawType::CardType(CardType::Field, _) => {
                self.draw_by_card_type(CardType::Field, cnt).unwrap()
            }
            CardDrawType::CardType(CardType::Unit, _) => {
                self.draw_by_card_type(CardType::Unit, cnt).unwrap()
            }
            _ => {
                vec![]
            }
        };

        draw_cards
    }
}
