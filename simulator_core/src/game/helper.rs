use uuid::Uuid;

use crate::{
    card::{cards::CardVecExt, types::PlayerKind},
    exception::GameError,
    player::message::GetCardsByUuid,
};

use super::GameActor;

// impl Game {
//     pub fn restore_then_reroll_mulligan_cards<T: Into<PlayerType>>(
//         &mut self,
//         player_type: T,
//         exclude_cards: Vec<Uuid>,
//     ) -> Result<Vec<Uuid>, GameError> {
//         let player_type = player_type.into();
//         self.restore_card(player_type, &exclude_cards)?;
//         let new_cards = self.get_mulligan_cards(player_type, exclude_cards.len())?;
//         Ok(new_cards)
//     }
// }

#[macro_export]
macro_rules! downcast_effect {
    ($effect:expr, $target_type:ty) => {
        if $effect.get_effect_type() == <$target_type>::static_effect_type() {
            if let Some(specific) = $effect.as_any().downcast_ref::<$target_type>() {
                Some(specific)
            } else {
                None
            }
        } else {
            None
        }
    };
}

pub async fn wait_for_input() -> Result<(), GameError> {
    todo!()
}

impl GameActor {
    /// 파라미터로 들어오는 카드들을 덱의 맨 밑으로 복원합니다.
    ///
    /// # Parameters
    /// * `player_type` - 카드를 복원할 플레이어 타입
    /// * `src_cards` - 복원할 카드들의 UUID 목록
    ///
    /// # Returns
    /// * `Ok(())` - 모든 카드가 성공적으로 덱의 맨 밑에 추가됨
    /// * `Err(GameError)` - 카드 복원 중 오류 발생
    ///
    /// # Errors
    /// * `GameError::CardNotFound` - 지정된 UUID를 가진 카드를 플레이어가 소유하지 않은 경우
    /// * `GameError::ExceededCardLimit` - 덱에 자리가 없어 카드를 추가할 수 없는 경우
    ///
    pub async fn restore_card(
        &self,
        player_type: PlayerKind,
        src_cards: &Vec<Uuid>,
    ) -> Result<(), GameError> {
        let player = self.get_player_addr_by_kind(player_type);

        // UUID에 해당하는 카드 목록을 반환
        // let mut result = vec![];
        // for uuid in msg.uuid {
        //     if let Some(card) = self.get_cards().find_by_uuid(uuid) {
        //         result.push(card.clone());
        //     } else {
        //         return vec![]; // 카드가 없으면 빈 벡터 반환
        //     }
        // }
        // result

        let mut result = vec![];
        let player_cards = self
            .all_cards
            .get(&player_type)
            .unwrap_or_else(|| panic!("Player cards not found for player type: {:?}", player_type));
        for uuid in src_cards {
            if let Some(card) = player_cards.find_by_uuid(uuid.clone()) {
                result.push(card.clone());
            } else {
                return Err(GameError::CardNotFound);
            }
        }

        // match player.get_cards().find_by_uuid(card_uuid.clone()) {
        //     Some(card) => card.clone(),
        //     None => return Err(GameError::CardNotFound),
        // }
        // self.get_player_by_type(player_type)
        //     .get()
        //     .get_deck_mut()
        //     .add_card(vec![card.clone()], Box::new(BottomInsert))?;
        Ok(())
    }
}
