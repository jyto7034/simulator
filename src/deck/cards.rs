use crate::deck::Card;
use crate::enums::constant::{self, CardType, SpellType, UUID, CmpType, CardParam};
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
    // --------------------------------------------------------
    // 카드 뭉치에 카드가 존재하는지 확인합니다.
    // --------------------------------------------------------
    fn is_deck_empty(&self) -> Result<(), Exception> {
        if self.v_card.is_empty() {
            return Err(Exception::NoCardsLeft);
        } else {
            return Ok(());
        }
    }

    // --------------------------------------------------------
    // 스펠 카드를 타입에 따라 뽑습니다.
    // 카드를 소모합니다.
    // --------------------------------------------------------
    // TODO: 
    //  - 카드 소모
    // --------------------------------------------------------

    fn draw_spell(&self, spell_type: SpellType) -> Result<Card, Exception> {
        // Deck 이 비어 있는지 확인합니다.
        self.is_deck_empty()?;

        self.find_by_card_type(CardType::Spell(spell_type))
    }

    // --------------------------------------------------------
    // 카드를 무작위로 뽑습니다.
    // 카드를 소모합니다.
    // --------------------------------------------------------
    // Returns:
    //  - Ok  : 문제없이 카드를 뽑았을 때, Card 를 반환합니다.
    //  - Err : 카드 뭉치에 카드가 없을 때, NoCardsLeft 를 반환합니다.
    //
    // Exceptions:
    //  - ThreadRng 참고, 카드 삭제 부분
    // --------------------------------------------------------
    fn draw_random(&mut self) -> Result<Card, Exception> {
        self.is_deck_empty()?;
        
        // 난수 생성기
        let mut rng = rand::thread_rng();
        
        let random_index = rng.gen_range(0..self.v_card.len());
        let ans = self.v_card[random_index].clone();
        self.v_card.remove(random_index);

        Ok(ans)
    }

    // --------------------------------------------------------
    // 카드 뭉치 하단에 있는 카드를 뽑습니다.
    // --------------------------------------------------------
    // TODO: 
    //  - 카드 소모
    // --------------------------------------------------------
    // Returns:
    //  - Ok  : 문제없이 카드를 뽑았을 때, Card 를 반환합니다.
    //  - Err : 카드 뭉치에 카드가 없을 때, NoCardsLeft 를 반환합니다.
    //
    // Exceptions:
    //  - 카드 삭제 부분
    // --------------------------------------------------------
    fn draw_bottom(&self) -> Result<Card, Exception> {
        self.is_deck_empty()?;
        
        let ans = self.v_card.first().unwrap().clone();
        self.v_card.remove(0);

        Ok(ans)
    }

    // --------------------------------------------------------
    // 카드 뭉치 하단에 있는 카드를 뽑습니다.
    // --------------------------------------------------------
    // Returns:
    //  - Ok  : 문제없이 카드를 뽑았을 때, Card 를 반환합니다.
    //  - Err : 카드 뭉치에 카드가 없을 때, NoCardsLeft 를 반환합니다.
    //
    // Exceptions:
    //  - 카드 삭제 부분
    // --------------------------------------------------------
    fn draw_top(&self) -> Result<Card, Exception> {
        self.is_deck_empty()?;

        let ans = self.v_card.last().unwrap().clone();
        self.v_card.remove(self.v_card.len() - 1);
        
        Ok(ans)
    }

    // --------------------------------------------------------
    // 카드 타입에 따라 뽑습니다.
    // 카드를 소모합니다.
    // --------------------------------------------------------
    // Returns:
    //  - Ok  : 문제없이 카드를 뽑았을 때, Card 를 반환합니다.
    //  - Err : 카드 뭉치에 카드가 없을 때, NoCardsLeft 를 반환합니다.
    //
    // Exceptions:
    //  - 카드 삭제 부분
    // --------------------------------------------------------
    fn draw_by_card_type(&self, card_type: CardType) -> Result<Card, Exception> {
        self.is_deck_empty()?;

        let ans = self.find_by_card_type(card_type)?;
        self.v_card.retain()

        Ok(ans)
    }

    // --------------------------------------------------------
    // uuid 에 해당하는 카드를 찾아내서 복사-반환합니다.
    // 카드를 소모하지 않습니다.
    // --------------------------------------------------------
    // Returns:
    //  - Ok  : 문제없이 카드를 뽑았을 때, Card 를 반환합니다.
    //  - Err : 카드 뭉치에 카드가 없을 때, NoCardsLeft 를 반환합니다.
    //
    // Exceptions:
    //  - 카드 삭제 부분
    // --------------------------------------------------------
    fn find_by_uuid(&mut self, uuid: String) -> Result<Card, Exception> {
        // uuid 에 해당하는 카드를 집계합니다.
        // count 가 0 개인 경우, 스킵하고 다음 카드를 찾습니다.
        match self.v_card.iter().find(|item| item.cmp(CmpType::Uuid(uuid.clone()))).cloned(){
            Some(card) => Ok(card),
            None => Err(Exception::CardsNotFound),
        }
    }
    
    // --------------------------------------------------------
    // name 에 해당하는 카드를 찾아내서 복사-반환합니다.
    // 카드를 소모하지 않습니다.
    // --------------------------------------------------------
    fn find_by_name(&mut self, name: String) -> Result<Card, Exception> {
        // name 에 해당하는 카드를 집계합니다.
        // count 가 0 개인 경우, 스킵하고 다음
        match self.v_card.iter().find(|item| item.cmp(CmpType::Name(name.clone()))).cloned(){
            Some(card) => Ok(card),
            None => Err(Exception::CardsNotFound),
        }
    }

    // --------------------------------------------------------
    // 타입에 해당하는 카드를 찾아내서 복사-반환합니다.
    // 카드를 소모하지 않습니다.
    // --------------------------------------------------------
    fn find_by_card_type(&self, card_type: CardType) -> Result<Card, Exception> {
        // cond 에 해당하는 카드를 집계합니다.
        // count 가 0 개인 경우, 스킵하고 다음 카드를 찾습니다.
        let filter = |cond: CardType| {
            match self.v_card.iter().find(|item| item.get_card_type() == &cond).cloned(){
                Some(card) => Ok(card),
                None => Err(Exception::CardsNotFound),
            }
        };

        match card_type {
            CardType::Dummy => panic!(),
            CardType::Unit => filter(CardType::Unit),
            CardType::Field => filter(CardType::Field),
            CardType::Spell(SpellType::FastSpell) => filter(CardType::Spell(SpellType::FastSpell)),
            CardType::Spell(SpellType::SlowSpell) => filter(CardType::Spell(SpellType::SlowSpell)),
        }
    }
}

impl Cards {
    
    // --------------------------------------------------------
    // 카드 뭉치를 새로 만듭니다.
    // --------------------------------------------------------
    pub fn new(cards: &Vec<Card>) -> Cards {
        Cards {
            v_card: cards.to_vec(),
        }
    }

    // --------------------------------------------------------
    // 카드 뭉치에 존재하는 카드들의 갯수를 반환합니다.
    // --------------------------------------------------------
    pub fn len(&self) -> usize {
        self.v_card.len()
    }

    // --------------------------------------------------------
    // 카드 뭉치에 카드를 새롭게 추가합니다.
    // --------------------------------------------------------
    pub fn add_card(&mut self, card: Card) -> Result<(), Exception>{
        if self.is_exceed() == false{
            self.v_card.push(card);
            Ok(())
        }else {
            Err(Exception::ExceededCardLimit)
        }
    }

    // --------------------------------------------------------
    // 빈 카드를 하나 만들어 반환합니다.
    // --------------------------------------------------------
    pub fn dummy() -> Cards {
        Cards { v_card: vec![] }
    }

    // --------------------------------------------------------
    // 카드 뭉치가 가질 수 있는 최대 갯수를 반환합니다.
    // --------------------------------------------------------
    // TODO:
    //  - 굳이 있어야 하나?
    // --------------------------------------------------------
    pub fn get_card_count(&self) -> u32 {
        constant::MAX_CARD_SIZE
    }

    // --------------------------------------------------------
    // 카드 뭉치가 포화상태인지 확인합니다.
    // --------------------------------------------------------
    pub fn is_exceed(&self) -> bool{
        self.v_card.len() >= constant::MAX_CARD_SIZE as usize
    }

    // --------------------------------------------------------
    // 카드를 카드 뭉치로부터 제거합니다.
    // --------------------------------------------------------
    pub fn remove(&mut self, card: CardParam){
        match card {
            CardParam::Uuid(uuid) => 
            self.v_card.retain(|item| item.cmp(CardParam::Uuid(uuid))),
            CardParam::Card(card) => 
            self.v_card.retain(|item| item.cmp(CardParam::Uuid(card.get_uuid().clone()))),
        }
    }

    // --------------------------------------------------------
    // 카드를 찾습니다
    // 카드를 소모하지 않습니다.
    // --------------------------------------------------------
    pub fn search(&mut self, find_type: constant::FindType) -> Result<Card, Exception> {
        use constant::*;

        match find_type {
            FindType::FindByUUID(uuid) => self.find_by_uuid(uuid),
            FindType::FindByName(name) => self.find_by_name(name),
            FindType::FindByCardType(card_type) => self.find_by_card_type(card_type),
        }
    }

    // --------------------------------------------------------
    // 카드 뭉치에서 한 장을 뽑습니다.
    // 뽑을 카드가 존재하지 않을 경우.(=덱사) 해당 에러를 외부로
    // 전파하여 처리합니다.
    // --------------------------------------------------------
    pub fn draw(&mut self, draw_type: constant::CardDrawType) -> Result<Card, Exception> {
        use constant::*;

        let count = match draw_type {
            CardDrawType::Random(count) => count,
            CardDrawType::CardType(_, count) => count,
            _ => 0,
        };

        // find 함수가 카드를 몇 개까지 찾게 할 지 정하는 변수.
        let cnt = if count == 0 { 100 } else { count };

        // 실제로 draw 하는 부분 입니다.
        let draw_cards = match draw_type {
            CardDrawType::Top => {
                self.draw_top()
            }
            CardDrawType::Random(_) => {
                self.draw_random()
            },
            CardDrawType::Bottom => {
                self.draw_bottom()
            },
            CardDrawType::CardType(CardType::Spell(SpellType::FastSpell), _) => {
                self.draw_spell(SpellType::FastSpell)
            },
            CardDrawType::CardType(CardType::Spell(SpellType::SlowSpell), _) => {
                self.draw_spell(SpellType::SlowSpell)
            },
            CardDrawType::CardType(CardType::Field, _) => {
                self.draw_by_card_type(CardType::Field)
            },
            CardDrawType::CardType(CardType::Unit, _) => {
                self.draw_by_card_type(CardType::Unit)
            },
            _ => {
                panic!()
            }
        };

        draw_cards
    }
}
