use crate::{deck::{Cards, Card, card}, exception::exception::Exception};

/// 게임의 필드를 구현하는 구조체입니다
/// 무덤, 유닛/효과 카드 필드, 핸드, 덱 등이 있습니다.
pub trait Zone{
    /// 현재 Zone 에 존재하는 모든 카드를 반환합니다. 
    fn get_cards(&self) -> &Cards;
    
    /// 현재 Zone 에 카드를 추가 합니다. 
    fn add_card(&mut self, card: &Card) -> Result<(), Exception>;

    /// 특정 카드를 현재 Zone 으로부터 삭제합니다.
    fn remove_card(&mut self, card: &Card) -> Result<(), Exception>;
}