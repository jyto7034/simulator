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

/// 카드 ID `PB_001`에 해당하는 카드를 생성합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보가 담긴 JSON 데이터.
/// * `count` - 카드의 수량 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 객체.
///
/// # Examples
///
/// ```
/// // let card_json = ...; // CardJson 객체를 초기화합니다.
/// // let card = PB_001(&card_json, 1);
/// // ...
/// ```
#[allow(non_snake_case)]
pub fn PB_001(card_json: &CardJson, count: i32) -> Card {
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
/// 카드 ID `PB_002`에 해당하는 카드를 생성합니다.  `PB_001`과 동일한 로직을 사용합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보가 담긴 JSON 데이터.
/// * `count` - 카드의 수량 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 객체.
///
/// # Examples
///
/// ```
/// // let card_json = ...; // CardJson 객체를 초기화합니다.
/// // let card = PB_002(&card_json, 1);
/// // ...
/// ```
#[allow(non_snake_case)]
pub fn PB_002(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
/// 카드 ID `PB_003`에 해당하는 카드를 생성합니다.  `PB_001`과 동일한 로직을 사용합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보가 담긴 JSON 데이터.
/// * `count` - 카드의 수량 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 객체.
///
/// # Examples
///
/// ```
/// // let card_json = ...; // CardJson 객체를 초기화합니다.
/// // let card = PB_003(&card_json, 1);
/// // ...
/// ```
#[allow(non_snake_case)]
pub fn PB_003(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
/// 카드 ID `PB_004`에 해당하는 카드를 생성합니다.  `PB_001`과 동일한 로직을 사용합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보가 담긴 JSON 데이터.
/// * `count` - 카드의 수량 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 객체.
///
/// # Examples
///
/// ```
/// // let card_json = ...; // CardJson 객체를 초기화합니다.
/// // let card = PB_004(&card_json, 1);
/// // ...
/// ```
#[allow(non_snake_case)]
pub fn PB_004(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
/// 카드 ID `PB_005`에 해당하는 카드를 생성합니다.  `PB_001`과 동일한 로직을 사용합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보가 담긴 JSON 데이터.
/// * `count` - 카드의 수량 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 객체.
///
/// # Examples
///
/// ```
/// // let card_json = ...; // CardJson 객체를 초기화합니다.
/// // let card = PB_005(&card_json, 1);
/// // ...
/// ```
#[allow(non_snake_case)]
pub fn PB_005(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
/// 카드 ID `PB_006`에 해당하는 카드를 생성합니다.  `PB_001`과 동일한 로직을 사용합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보가 담긴 JSON 데이터.
/// * `count` - 카드의 수량 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 객체.
///
/// # Examples
///
/// ```
/// // let card_json = ...; // CardJson 객체를 초기화합니다.
/// // let card = PB_006(&card_json, 1);
/// // ...
/// ```
#[allow(non_snake_case)]
pub fn PB_006(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
/// 카드 ID `PB_007`에 해당하는 카드를 생성합니다.  `PB_001`과 동일한 로직을 사용합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보가 담긴 JSON 데이터.
/// * `count` - 카드의 수량 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 객체.
///
/// # Examples
///
/// ```
/// // let card_json = ...; // CardJson 객체를 초기화합니다.
/// // let card = PB_007(&card_json, 1);
/// // ...
/// ```
#[allow(non_snake_case)]
pub fn PB_007(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
/// 카드 ID `PB_008`에 해당하는 카드를 생성합니다.  `PB_001`과 동일한 로직을 사용합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보가 담긴 JSON 데이터.
/// * `count` - 카드의 수량 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 객체.
///
/// # Examples
///
/// ```
/// // let card_json = ...; // CardJson 객체를 초기화합니다.
/// // let card = PB_008(&card_json, 1);
/// // ...
/// ```
#[allow(non_snake_case)]
pub fn PB_008(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
/// 카드 ID `PB_009`에 해당하는 카드를 생성합니다.  `PB_001`과 동일한 로직을 사용합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보가 담긴 JSON 데이터.
/// * `count` - 카드의 수량 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 객체.
///
/// # Examples
///
/// ```
/// // let card_json = ...; // CardJson 객체를 초기화합니다.
/// // let card = PB_009(&card_json, 1);
/// // ...
/// ```
#[allow(non_snake_case)]
pub fn PB_009(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
/// 카드 ID `PB_010`에 해당하는 카드를 생성합니다. `PB_001`과 동일한 로직을 사용합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보가 담긴 JSON 데이터.
/// * `count` - 카드의 수량 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 객체.
///
/// # Examples
///
/// ```
/// // let card_json = ...; // CardJson 객체를 초기화합니다.
/// // let card = PB_010(&card_json, 1);
/// // ...
/// ```
#[allow(non_snake_case)]
pub fn PB_010(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
/// 카드 ID `PB_011`에 해당하는 카드를 생성합니다.  `PB_001`과 동일한 로직을 사용합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보가 담긴 JSON 데이터.
/// * `count` - 카드의 수량 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 객체.
///
/// # Examples
///
/// ```
/// // let card_json = ...; // CardJson 객체를 초기화합니다.
/// // let card = PB_011(&card_json, 1);
/// // ...
/// ```
#[allow(non_snake_case)]
pub fn PB_011(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
/// 카드 ID `PB_012`에 해당하는 카드를 생성합니다.  `PB_001`과 동일한 로직을 사용합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보가 담긴 JSON 데이터.
/// * `count` - 카드의 수량 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 객체.
///
/// # Examples
///
/// ```
/// // let card_json = ...; // CardJson 객체를 초기화합니다.
/// // let card = PB_012(&card_json, 1);
/// // ...
/// ```
#[allow(non_snake_case)]
pub fn PB_012(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
/// 카드 ID `PB_013`에 해당하는 카드를 생성합니다.  `PB_001`과 동일한 로직을 사용합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보가 담긴 JSON 데이터.
/// * `count` - 카드의 수량 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 객체.
///
/// # Examples
///
/// ```
/// // let card_json = ...; // CardJson 객체를 초기화합니다.
/// // let card = PB_013(&card_json, 1);
/// // ...
/// ```
#[allow(non_snake_case)]
pub fn PB_013(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
/// 카드 ID `PB_014`에 해당하는 카드를 생성합니다.  `PB_001`과 동일한 로직을 사용합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보가 담긴 JSON 데이터.
/// * `count` - 카드의 수량 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 객체.
///
/// # Examples
///
/// ```
/// // let card_json = ...; // CardJson 객체를 초기화합니다.
/// // let card = PB_014(&card_json, 1);
/// // ...
/// ```
#[allow(non_snake_case)]
pub fn PB_014(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
/// 카드 ID `PB_015`에 해당하는 카드를 생성합니다.  `PB_001`과 동일한 로직을 사용합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보가 담긴 JSON 데이터.
/// * `count` - 카드의 수량 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 객체.
///
/// # Examples
///
/// ```
/// // let card_json = ...; // CardJson 객체를 초기화합니다.
/// // let card = PB_015(&card_json, 1);
/// // ...
/// ```
#[allow(non_snake_case)]
pub fn PB_015(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
/// 카드 ID `PB_016`에 해당하는 카드를 생성합니다.  `PB_001`과 동일한 로직을 사용합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보가 담긴 JSON 데이터.
/// * `count` - 카드의 수량 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 객체.
///
/// # Examples
///
/// ```
/// // let card_json = ...; // CardJson 객체를 초기화합니다.
/// // let card = PB_016(&card_json, 1);
/// // ...
/// ```
#[allow(non_snake_case)]
pub fn PB_016(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
/// 카드 ID `PB_017`에 해당하는 카드를 생성합니다.  `PB_001`과 동일한 로직을 사용합니다.
///
/// # Arguments
///
/// * `card_json` - 카드 정보가 담긴 JSON 데이터.
/// * `count` - 카드의 수량 (현재 사용되지 않음).
///
/// # Returns
///
/// 생성된 `Card` 객체.
///
/// # Examples
///
/// ```
/// // let card_json = ...; // CardJson 객체를 초기화합니다.
/// // let card = PB_017(&card_json, 1);
/// // ...
/// ```
#[allow(non_snake_case)]
pub fn PB_017(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}