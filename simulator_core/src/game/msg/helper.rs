use crate::{
    card::Card,
    sync::snapshots::{CardSnapshot, PrivateCardSnapshot},
};

// Card -> CardSnapshot 변환 헬퍼
pub fn to_card_snapshot(card: &Card) -> CardSnapshot {
    // Card의 공개 정보를 바탕으로 CardSnapshot 생성
    todo!()
}

// Card -> PrivateCardSnapshot 변환 헬퍼
pub fn to_private_card_snapshot(card: &Card) -> PrivateCardSnapshot {
    // Card의 모든 정보를 바탕으로 PrivateCardSnapshot 생성
    todo!()
}
