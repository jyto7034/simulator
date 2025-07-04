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

/// `MT_001` 함수는 주어진 `CardJson`과 개수를 사용하여 새로운 `Card`를 생성합니다.
///
/// 이 함수는 `CardBuilder`를 사용하여 카드를 생성하고, 드로우 효과와 스탯 변경 효과를 추가합니다.
#[allow(non_snake_case)]
///
/// # Arguments
///
/// * `card_json` - 카드의 정보를 담고 있는 `CardJson` 구조체에 대한 참조자입니다.
/// * `count` - 카드의 개수입니다. 현재는 사용되지 않습니다.
///
/// # Returns
///
/// * 생성된 `Card` 객체를 반환합니다.
///
/// # Examples
///
/// ```
/// // MT_001 함수를 사용하는 예제
/// // CardJson 구조체가 미리 정의되어 있어야 합니다.
/// // let card_json = CardJson { ... };
/// // let card = MT_001(&card_json, 1);
/// ```
pub fn MT_001(card_json: &CardJson, count: i32) -> Card {
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

/// `MT_002` 함수는 `MT_001` 함수를 호출하여 새로운 `Card`를 생성합니다.
///
/// # Arguments
///
/// * `card_json` - 카드의 정보를 담고 있는 `CardJson` 구조체에 대한 참조자입니다.
/// * `count` - 카드의 개수입니다. 현재는 사용되지 않습니다.
///
/// # Returns
///
/// * 생성된 `Card` 객체를 반환합니다.
///
/// # Examples
///
/// ```
/// // MT_002 함수를 사용하는 예제
/// // CardJson 구조체가 미리 정의되어 있어야 합니다.
/// // let card_json = CardJson { ... };
/// // let card = MT_002(&card_json, 1);
/// ```
#[allow(non_snake_case)]
pub fn MT_002(card_json: &CardJson, count: i32) -> Card {
    MT_001(card_json, count)
}

/// `MT_003` 함수는 `MT_001` 함수를 호출하여 새로운 `Card`를 생성합니다.
///
/// # Arguments
///
/// * `card_json` - 카드의 정보를 담고 있는 `CardJson` 구조체에 대한 참조자입니다.
/// * `count` - 카드의 개수입니다. 현재는 사용되지 않습니다.
///
/// # Returns
///
/// * 생성된 `Card` 객체를 반환합니다.
///
/// # Examples
///
/// ```
/// // MT_003 함수를 사용하는 예제
/// // CardJson 구조체가 미리 정의되어 있어야 합니다.
/// // let card_json = CardJson { ... };
/// // let card = MT_003(&card_json, 1);
/// ```
#[allow(non_snake_case)]
pub fn MT_003(card_json: &CardJson, count: i32) -> Card {
    MT_001(card_json, count)
}
/// `MT_004` 함수는 `MT_001` 함수를 호출하여 새로운 `Card`를 생성합니다.
///
/// # Arguments
///
/// * `card_json` - 카드의 정보를 담고 있는 `CardJson` 구조체에 대한 참조자입니다.
/// * `count` - 카드의 개수입니다. 현재는 사용되지 않습니다.
///
/// # Returns
///
/// * 생성된 `Card` 객체를 반환합니다.
///
/// # Examples
///
/// ```
/// // MT_004 함수를 사용하는 예제
/// // CardJson 구조체가 미리 정의되어 있어야 합니다.
/// // let card_json = CardJson { ... };
/// // let card = MT_004(&card_json, 1);
/// ```
#[allow(non_snake_case)]
pub fn MT_004(card_json: &CardJson, count: i32) -> Card {
    MT_001(card_json, count)
}
/// `MT_005` 함수는 `MT_001` 함수를 호출하여 새로운 `Card`를 생성합니다.
///
/// # Arguments
///
/// * `card_json` - 카드의 정보를 담고 있는 `CardJson` 구조체에 대한 참조자입니다.
/// * `count` - 카드의 개수입니다. 현재는 사용되지 않습니다.
///
/// # Returns
///
/// * 생성된 `Card` 객체를 반환합니다.
///
/// # Examples
///
/// ```
/// // MT_005 함수를 사용하는 예제
/// // CardJson 구조체가 미리 정의되어 있어야 합니다.
/// // let card_json = CardJson { ... };
/// // let card = MT_005(&card_json, 1);
/// ```
#[allow(non_snake_case)]
pub fn MT_005(card_json: &CardJson, count: i32) -> Card {
    MT_001(card_json, count)
}
/// `MT_006` 함수는 `MT_001` 함수를 호출하여 새로운 `Card`를 생성합니다.
///
/// # Arguments
///
/// * `card_json` - 카드의 정보를 담고 있는 `CardJson` 구조체에 대한 참조자입니다.
/// * `count` - 카드의 개수입니다. 현재는 사용되지 않습니다.
///
/// # Returns
///
/// * 생성된 `Card` 객체를 반환합니다.
///
/// # Examples
///
/// ```
/// // MT_006 함수를 사용하는 예제
/// // CardJson 구조체가 미리 정의되어 있어야 합니다.
/// // let card_json = CardJson { ... };
/// // let card = MT_006(&card_json, 1);
/// ```
#[allow(non_snake_case)]
pub fn MT_006(card_json: &CardJson, count: i32) -> Card {
    MT_001(card_json, count)
}
/// `MT_007` 함수는 `MT_001` 함수를 호출하여 새로운 `Card`를 생성합니다.
///
/// # Arguments
///
/// * `card_json` - 카드의 정보를 담고 있는 `CardJson` 구조체에 대한 참조자입니다.
/// * `count` - 카드의 개수입니다. 현재는 사용되지 않습니다.
///
/// # Returns
///
/// * 생성된 `Card` 객체를 반환합니다.
///
/// # Examples
///
/// ```
/// // MT_007 함수를 사용하는 예제
/// // CardJson 구조체가 미리 정의되어 있어야 합니다.
/// // let card_json = CardJson { ... };
/// // let card = MT_007(&card_json, 1);
/// ```
#[allow(non_snake_case)]
pub fn MT_007(card_json: &CardJson, count: i32) -> Card {
    MT_001(card_json, count)
}
/// `MT_008` 함수는 `MT_001` 함수를 호출하여 새로운 `Card`를 생성합니다.
///
/// # Arguments
///
/// * `card_json` - 카드의 정보를 담고 있는 `CardJson` 구조체에 대한 참조자입니다.
/// * `count` - 카드의 개수입니다. 현재는 사용되지 않습니다.
///
/// # Returns
///
/// * 생성된 `Card` 객체를 반환합니다.
///
/// # Examples
///
/// ```
/// // MT_008 함수를 사용하는 예제
/// // CardJson 구조체가 미리 정의되어 있어야 합니다.
/// // let card_json = CardJson { ... };
/// // let card = MT_008(&card_json, 1);
/// ```
#[allow(non_snake_case)]
pub fn MT_008(card_json: &CardJson, count: i32) -> Card {
    MT_001(card_json, count)
}
/// `MT_009` 함수는 `MT_001` 함수를 호출하여 새로운 `Card`를 생성합니다.
///
/// # Arguments
///
/// * `card_json` - 카드의 정보를 담고 있는 `CardJson` 구조체에 대한 참조자입니다.
/// * `count` - 카드의 개수입니다. 현재는 사용되지 않습니다.
///
/// # Returns
///
/// * 생성된 `Card` 객체를 반환합니다.
///
/// # Examples
///
/// ```
/// // MT_009 함수를 사용하는 예제
/// // CardJson 구조체가 미리 정의되어 있어야 합니다.
/// // let card_json = CardJson { ... };
/// // let card = MT_009(&card_json, 1);
/// ```
#[allow(non_snake_case)]
pub fn MT_009(card_json: &CardJson, count: i32) -> Card {
    MT_001(card_json, count)
}
/// `MT_010` 함수는 `MT_001` 함수를 호출하여 새로운 `Card`를 생성합니다.
///
/// # Arguments
///
/// * `card_json` - 카드의 정보를 담고 있는 `CardJson` 구조체에 대한 참조자입니다.
/// * `count` - 카드의 개수입니다. 현재는 사용되지 않습니다.
///
/// # Returns
///
/// * 생성된 `Card` 객체를 반환합니다.
///
/// # Examples
///
/// ```
/// // MT_010 함수를 사용하는 예제
/// // CardJson 구조체가 미리 정의되어 있어야 합니다.
/// // let card_json = CardJson { ... };
/// // let card = MT_010(&card_json, 1);
/// ```
#[allow(non_snake_case)]
pub fn MT_010(card_json: &CardJson, count: i32) -> Card {
    MT_001(card_json, count)
}