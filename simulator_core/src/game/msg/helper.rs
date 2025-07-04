use crate::{
    card::Card,
    sync::snapshots::{CardSnapshot, PrivateCardSnapshot},
};

/// `Card`를 `CardSnapshot`으로 변환하는 헬퍼 함수입니다.
/// `CardSnapshot`은 `Card`의 공개된 정보만 담고 있습니다.
///
/// # Arguments
///
/// * `card` - 변환할 `Card` 객체에 대한 참조입니다.
///
/// # Returns
///
/// `Card`의 공개 정보를 담은 `CardSnapshot` 객체입니다.
///
/// # Examples
///
/// ```
/// // TODO: 예제 코드 추가
/// ```
pub fn to_card_snapshot(card: &Card) -> CardSnapshot {
    // Card의 공개 정보를 바탕으로 CardSnapshot 생성
    todo!()
}

/// `Card`를 `PrivateCardSnapshot`으로 변환하는 헬퍼 함수입니다.
/// `PrivateCardSnapshot`은 `Card`의 모든 정보를 담고 있습니다.
///
/// # Arguments
///
/// * `card` - 변환할 `Card` 객체에 대한 참조입니다.
///
/// # Returns
///
/// `Card`의 모든 정보를 담은 `PrivateCardSnapshot` 객체입니다.
///
/// # Examples
///
/// ```
/// // TODO: 예제 코드 추가
/// ```
pub fn to_private_card_snapshot(card: &Card) -> PrivateCardSnapshot {
    // Card의 모든 정보를 바탕으로 PrivateCardSnapshot 생성
    todo!()
}