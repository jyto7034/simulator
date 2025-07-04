use crate::{
    card::{
        types::{OwnerType, StatType},
        Card,
    },
    effect::{DrawEffect, ModifyStatEffect},
    enums::ZoneType,
    selector::single::SingleCardSelector,
    utils::json::CardJson,
};

use super::builder::CardBuilder;
// -------------------------------------------------- FIELD
// [HM_001] Hieda no Akyuu - COST:?? [ATK:??/HP:?]
// - Set: Human, Rarity: C
// --------------------------------------------------------
// Text: 낮동안 인간 카드를 사용할 때 마다 서로 1장 드로우 한다.
// --------------------------------------------------------
// Behaviors:
// - ListenOtherEvent
// - DrawCardFromDeck
// --------------------------------------------------------
#[allow(non_snake_case)]
/// `HM_001` 함수는 주어진 `CardJson`과 개수를 기반으로 특정 인간 카드(히에다노 아큐)를 생성합니다.
///
/// 이 카드는 다음과 같은 효과를 가집니다:
/// * 2장 드로우
/// * 모든 카드의 공격력을 2 증가
///
/// # Arguments
///
/// * `card_json` - 카드 정보를 담고 있는 `CardJson` 구조체에 대한 참조.
/// * `count` - 카드의 개수 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 인스턴스.
// TODO: count 매개변수가 실제로 사용되는지 확인하고, 사용되지 않는다면 제거하거나 활용 방법을 고려.
// TODO: 효과에 대한 설명을 더 자세하게 기술 (예: 드로우 효과의 주체, 공격력 증가의 지속 시간 등).
// TODO: 카드 이름과 설명을 주석에 포함 (히에다노 아큐에 대한 설명).
pub fn HM_001(card_json: &CardJson, count: i32) -> Card {
    CardBuilder::new(card_json)
        .unwrap()
        .add_effect(DrawEffect { count: 2 })
        .add_effect(ModifyStatEffect {
            stat_type: StatType::Attack,
            amount: 2,
            target_selector: Box::new(SingleCardSelector::new(ZoneType::None, OwnerType::Any)),
        })
        .build()
}

#[allow(non_snake_case)]
/// `HM_002` 함수는 `HM_001` 함수를 호출하여 카드를 생성합니다. 현재는 `HM_001`과 동일한 동작을 수행합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보를 담고 있는 `CardJson` 구조체에 대한 참조.
/// * `count` - 카드의 개수 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 인스턴스.
// TODO: HM_002의 기능이 HM_001과 동일하다면, HM_002의 존재 이유를 명확히 하거나 제거를 고려.
// TODO: 만약 다른 기능을 수행하도록 변경될 예정이라면, 변경될 기능에 대한 주석을 추가.
// TODO: count 매개변수가 실제로 사용되는지 확인하고, 사용되지 않는다면 제거하거나 활용 방법을 고려.
pub fn HM_002(card_json: &CardJson, count: i32) -> Card {
    HM_001(card_json, count)
}

#[allow(non_snake_case)]
/// `HM_003` 함수는 `HM_001` 함수를 호출하여 카드를 생성합니다. 현재는 `HM_001`과 동일한 동작을 수행합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보를 담고 있는 `CardJson` 구조체에 대한 참조.
/// * `count` - 카드의 개수 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 인스턴스.
// TODO: HM_003의 기능이 HM_001과 동일하다면, HM_003의 존재 이유를 명확히 하거나 제거를 고려.
// TODO: 만약 다른 기능을 수행하도록 변경될 예정이라면, 변경될 기능에 대한 주석을 추가.
// TODO: count 매개변수가 실제로 사용되는지 확인하고, 사용되지 않는다면 제거하거나 활용 방법을 고려.
pub fn HM_003(card_json: &CardJson, count: i32) -> Card {
    HM_001(card_json, count)
}
#[allow(non_snake_case)]
/// `HM_004` 함수는 `HM_001` 함수를 호출하여 카드를 생성합니다. 현재는 `HM_001`과 동일한 동작을 수행합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보를 담고 있는 `CardJson` 구조체에 대한 참조.
/// * `count` - 카드의 개수 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 인스턴스.
// TODO: HM_004의 기능이 HM_001과 동일하다면, HM_004의 존재 이유를 명확히 하거나 제거를 고려.
// TODO: 만약 다른 기능을 수행하도록 변경될 예정이라면, 변경될 기능에 대한 주석을 추가.
// TODO: count 매개변수가 실제로 사용되는지 확인하고, 사용되지 않는다면 제거하거나 활용 방법을 고려.
pub fn HM_004(card_json: &CardJson, count: i32) -> Card {
    HM_001(card_json, count)
}
#[allow(non_snake_case)]
/// `HM_005` 함수는 `HM_001` 함수를 호출하여 카드를 생성합니다. 현재는 `HM_001`과 동일한 동작을 수행합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보를 담고 있는 `CardJson` 구조체에 대한 참조.
/// * `count` - 카드의 개수 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 인스턴스.
// TODO: HM_005의 기능이 HM_001과 동일하다면, HM_005의 존재 이유를 명확히 하거나 제거를 고려.
// TODO: 만약 다른 기능을 수행하도록 변경될 예정이라면, 변경될 기능에 대한 주석을 추가.
// TODO: count 매개변수가 실제로 사용되는지 확인하고, 사용되지 않는다면 제거하거나 활용 방법을 고려.
pub fn HM_005(card_json: &CardJson, count: i32) -> Card {
    HM_001(card_json, count)
}
#[allow(non_snake_case)]
/// `HM_006` 함수는 `HM_001` 함수를 호출하여 카드를 생성합니다. 현재는 `HM_001`과 동일한 동작을 수행합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보를 담고 있는 `CardJson` 구조체에 대한 참조.
/// * `count` - 카드의 개수 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 인스턴스.
// TODO: HM_006의 기능이 HM_001과 동일하다면, HM_006의 존재 이유를 명확히 하거나 제거를 고려.
// TODO: 만약 다른 기능을 수행하도록 변경될 예정이라면, 변경될 기능에 대한 주석을 추가.
// TODO: count 매개변수가 실제로 사용되는지 확인하고, 사용되지 않는다면 제거하거나 활용 방법을 고려.
pub fn HM_006(card_json: &CardJson, count: i32) -> Card {
    HM_001(card_json, count)
}
#[allow(non_snake_case)]
/// `HM_007` 함수는 `HM_001` 함수를 호출하여 카드를 생성합니다. 현재는 `HM_001`과 동일한 동작을 수행합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보를 담고 있는 `CardJson` 구조체에 대한 참조.
/// * `count` - 카드의 개수 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 인스턴스.
// TODO: HM_007의 기능이 HM_001과 동일하다면, HM_007의 존재 이유를 명확히 하거나 제거를 고려.
// TODO: 만약 다른 기능을 수행하도록 변경될 예정이라면, 변경될 기능에 대한 주석을 추가.
// TODO: count 매개변수가 실제로 사용되는지 확인하고, 사용되지 않는다면 제거하거나 활용 방법을 고려.
pub fn HM_007(card_json: &CardJson, count: i32) -> Card {
    HM_001(card_json, count)
}
#[allow(non_snake_case)]
/// `HM_008` 함수는 `HM_001` 함수를 호출하여 카드를 생성합니다. 현재는 `HM_001`과 동일한 동작을 수행합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보를 담고 있는 `CardJson` 구조체에 대한 참조.
/// * `count` - 카드의 개수 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 인스턴스.
// TODO: HM_008의 기능이 HM_001과 동일하다면, HM_008의 존재 이유를 명확히 하거나 제거를 고려.
// TODO: 만약 다른 기능을 수행하도록 변경될 예정이라면, 변경될 기능에 대한 주석을 추가.
// TODO: count 매개변수가 실제로 사용되는지 확인하고, 사용되지 않는다면 제거하거나 활용 방법을 고려.
pub fn HM_008(card_json: &CardJson, count: i32) -> Card {
    HM_001(card_json, count)
}
