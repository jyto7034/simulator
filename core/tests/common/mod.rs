use game_core::game::data::abnormality_data::{AbnormalityDatabase, AbnormalityMetadata};
use game_core::game::data::artifact_data::{ArtifactDatabase, ArtifactMetadata};
use game_core::game::data::bonus_data::{BonusDatabase, BonusMetadata};
use game_core::game::data::equipment_data::{EquipmentDatabase, EquipmentMetadata};
use game_core::game::data::event_pools::{EventPhasePool, EventPoolConfig, WeightedEvent};
use game_core::game::data::random_event_data::{
    EventRiskLevel, RandomEventDatabase, RandomEventMetadata,
};
use game_core::game::data::shop_data::{ShopDatabase, ShopMetadata, ShopProduct, ShopType};
use game_core::game::data::GameData;
use game_core::game::enums::{EquipmentType, RiskLevel};
use game_core::game::events::event_selection::bonus::BonusType;
use game_core::game::events::event_selection::random::RandomEventType;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// 테스트용 GameData 생성 - 하드코딩된 예측 가능한 데이터
///
/// 테스트에 최적화된 최소한의 데이터를 생성합니다.
/// - 리롤 가능한 상점 1개
/// - 최소한의 아이템/장비/아티팩트
/// - 예측 가능한 UUID 사용
#[cfg(test)]
pub fn create_test_game_data() -> Arc<GameData> {
    // UUID 생성 (테스트에서 고정된 UUID 사용)
    let shop_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let bonus_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
    let event_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap();

    // 아티팩트 데이터
    let artifact1 = ArtifactMetadata {
        id: "test_artifact_1".to_string(),
        uuid: Uuid::parse_str("a0000001-0000-0000-0000-000000000001").unwrap(),
        name: "Test Artifact 1".to_string(),
        description: "Test artifact for testing".to_string(),
        rarity: RiskLevel::HE,
        price: 100,
    };

    let artifact2 = ArtifactMetadata {
        id: "test_artifact_2".to_string(),
        uuid: Uuid::parse_str("a0000002-0000-0000-0000-000000000002").unwrap(),
        name: "Test Artifact 2".to_string(),
        description: "Another test artifact".to_string(),
        rarity: RiskLevel::WAW,
        price: 200,
    };

    let artifact3 = ArtifactMetadata {
        id: "test_artifact_3".to_string(),
        uuid: Uuid::parse_str("a0000002-0000-0000-0000-000000000003").unwrap(),
        name: "Test Artifact 3".to_string(),
        description: "Another test artifact".to_string(),
        rarity: RiskLevel::WAW,
        price: 200,
    };

    // 장비 데이터
    let equipment1 = EquipmentMetadata {
        id: "test_weapon_1".to_string(),
        uuid: Uuid::parse_str("e0000001-0000-0000-0000-000000000001").unwrap(),
        name: "Test Weapon".to_string(),
        equipment_type: EquipmentType::Weapon,
        rarity: RiskLevel::HE,
        price: 150,
    };

    let equipment2 = EquipmentMetadata {
        id: "test_suit_1".to_string(),
        uuid: Uuid::parse_str("e0000002-0000-0000-0000-000000000002").unwrap(),
        name: "Test Suit".to_string(),
        equipment_type: EquipmentType::Suit,
        rarity: RiskLevel::TETH,
        price: 80,
    };

    let equipment3 = EquipmentMetadata {
        id: "test_suit_2".to_string(),
        uuid: Uuid::parse_str("e0000002-0000-0000-0000-000000000003").unwrap(),
        name: "Test Suit".to_string(),
        equipment_type: EquipmentType::Suit,
        rarity: RiskLevel::TETH,
        price: 80,
    };

    // 환상체 데이터
    let abnormality1 = AbnormalityMetadata {
        id: "test_abnorm_1".to_string(),
        uuid: Uuid::parse_str("b0000001-0000-0000-0000-000000000001").unwrap(),
        name: "Test Abnormality".to_string(),
        risk_level: RiskLevel::HE,
        price: 120,
    };

    // 상점 데이터 (리롤 가능)
    let shop = ShopMetadata {
        name: "Test Rerollable Shop".to_string(),
        uuid: shop_uuid,
        shop_type: ShopType::Shop,
        items_raw: vec![
            ShopProduct::Artifact("test_artifact_1".to_string()),
            ShopProduct::Artifact("test_artifact_2".to_string()),
            ShopProduct::Artifact("test_artifact_3".to_string()),
            ShopProduct::Equipment("test_weapon_1".to_string()),
            ShopProduct::Equipment("test_suit_1".to_string()),
            ShopProduct::Equipment("test_suit_2".to_string()),
        ],
        visible_items: vec![], // hydrate에서 채워짐
        hidden_items: vec![],  // hydrate에서 채워짐
        can_reroll: true,      // 테스트용 - 항상 리롤 가능
    };

    // 보너스 데이터
    let bonus = BonusMetadata {
        bonus_type: BonusType::Enkephalin,
        uuid: bonus_uuid,
        name: "Test Enkephalin Bonus".to_string(),
        description: "Grants 50-100 Enkephalin".to_string(),
        icon: "enkephalin_icon.png".to_string(),
        min_amount: 50,
        max_amount: 100,
    };

    // 랜덤 이벤트 데이터
    let random_event = RandomEventMetadata {
        id: "test_event".to_string(),
        uuid: event_uuid,
        event_type: RandomEventType::SuspiciousBox,
        name: "Test Event".to_string(),
        description: "A test event".to_string(),
        image: "test.png".to_string(),
        risk_level: EventRiskLevel::Low,
    };

    // 이벤트 풀 설정 (Dawn만 설정)
    let dawn_pool = EventPhasePool {
        shops: vec![WeightedEvent {
            weight: 1,
            uuid: shop_uuid,
        }],
        bonuses: vec![WeightedEvent {
            weight: 1,
            uuid: bonus_uuid,
        }],
        random_events: vec![WeightedEvent {
            weight: 1,
            uuid: event_uuid,
        }],
    };

    let empty_pool = EventPhasePool {
        shops: vec![],
        bonuses: vec![],
        random_events: vec![],
    };

    let event_pools = EventPoolConfig {
        dawn: dawn_pool,
        noon: empty_pool.clone(),
        dusk: empty_pool.clone(),
        midnight: empty_pool.clone(),
        white: empty_pool,
    };

    // Database 생성
    let mut artifacts = ArtifactDatabase::new(vec![artifact1, artifact2, artifact3]);
    artifacts.init_map();

    let mut equipments = EquipmentDatabase::new(vec![equipment1, equipment2, equipment3]);
    equipments.init_map();

    let mut abnormalities = AbnormalityDatabase::new(vec![abnormality1]);
    abnormalities.init_map();

    let shops_db = ShopDatabase::new(vec![shop]);
    let bonuses = BonusDatabase::new(vec![bonus]);
    let random_events = RandomEventDatabase::new(vec![random_event]);

    // GameData 생성
    let mut game_data = GameData {
        shops_db,
        event_pools,
        bonuses,
        random_events,
        abnormalities,
        equipments,
        artifacts,
        item_uuid_map: HashMap::new(),
    };

    game_data.build_item_uuid_map();

    Arc::new(game_data)
}

/// 실제 RON 파일에서 GameData 로드
///
/// 통합 테스트나 실제 서버에서 사용합니다.
/// - 상점 데이터 (shops.ron)
/// - 보너스 데이터 (bonuses.ron)
/// - 랜덤 이벤트 데이터 (random_events.ron)
/// - 이벤트 풀 설정 (event_pools.ron)
/// - 환상체 데이터 (abnormalities.ron)
/// - 장비 데이터 (equipments.ron)
/// - 아티팩트 데이터 (artifacts.ron)
#[allow(dead_code)]
pub fn load_game_data_from_ron() -> Arc<GameData> {
    // 1. RON 파일 포함 (컴파일 타임)
    let shops_ron = include_str!("../../../game_resources/data/events/shops.ron");
    let bonuses_ron = include_str!("../../../game_resources/data/events/bonuses.ron");
    let random_events_ron = include_str!("../../../game_resources/data/events/random_events.ron");
    let event_pools_ron = include_str!("../../../game_resources/data/events/event_pools.ron");
    let abnormalities_ron = include_str!("../../../game_resources/data/abnormalities.ron");
    let equipments_ron = include_str!("../../../game_resources/data/equipments.ron");
    let artifacts_ron = include_str!("../../../game_resources/data/artifacts.ron");

    // 2. RON 역직렬화
    let shops_db: ShopDatabase =
        ron::de::from_str(shops_ron).expect("Failed to deserialize shops.ron");

    let bonuses: BonusDatabase =
        ron::de::from_str(bonuses_ron).expect("Failed to deserialize bonuses.ron");

    let random_events: RandomEventDatabase =
        ron::de::from_str(random_events_ron).expect("Failed to deserialize random_events.ron");

    let event_pools: EventPoolConfig =
        ron::de::from_str(event_pools_ron).expect("Failed to deserialize event_pools.ron");

    let mut abnormalities: AbnormalityDatabase =
        ron::de::from_str(abnormalities_ron).expect("Failed to deserialize abnormalities.ron");

    let mut equipments: EquipmentDatabase =
        ron::de::from_str(equipments_ron).expect("Failed to deserialize equipments.ron");

    let mut artifacts: ArtifactDatabase =
        ron::de::from_str(artifacts_ron).expect("Failed to deserialize artifacts.ron");

    // 3. HashMap 초기화 (serde skip된 필드)
    abnormalities.init_map();
    equipments.init_map();
    artifacts.init_map();

    // 4. GameData 생성
    let mut game_data = GameData {
        shops_db,
        event_pools,
        bonuses,
        random_events,
        abnormalities,
        equipments,
        artifacts,
        item_uuid_map: HashMap::new(),
    };

    game_data.build_item_uuid_map();

    Arc::new(game_data)
}
