use crate::deck::{Card, Cards};
use crate::enums::constant;
use crate::exception::exception::Exception;
use crate::game::Game;
use crate::task::Task;
use crate::zone::Zone;

pub struct EffectZone {
    zone_cards: Cards,
    zone_size: usize,
}
impl EffectZone{
    pub fn new() -> EffectZone {
        EffectZone {
            zone_cards: Cards::new(&vec![]),
            zone_size: constant::UNIT_ZONE_SIZE,
        }
    }
}

impl Zone for EffectZone {
    /// 현재 Zone 에 존재하는 모든 카드를 반환합니다.
    fn get_cards(&mut self) -> &mut Cards {
        &mut self.zone_cards
    }

    /// 현재 Zone 에 카드를 추가 합니다.
    fn add_card(&mut self, _card: Card) -> Result<(), Exception> {
        todo!()
    }
    
    /// 특정 카드를 현재 Zone 으로부터 삭제합니다.
    fn remove_card(&mut self, _card: Card) -> Result<(), Exception> {
        todo!()
    }

    fn run(&self, game: &mut Game) {
        for item in self.zone_cards.v_card.iter(){
            if let Some(proc) = &game.procedure{
                proc.as_ref().borrow_mut().add_task(Task::new(item.clone()));
            }
        }
    }
}