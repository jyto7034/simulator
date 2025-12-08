use std::sync::Arc;

use game_core::game::data::abnormality_data::{AbnormalityDatabase, AbnormalityMetadata};
use game_core::game::data::artifact_data::{ArtifactDatabase, ArtifactMetadata};
use game_core::game::data::bonus_data::{BonusDatabase, BonusMetadata, BonusType};
use game_core::game::data::equipment_data::{EquipmentDatabase, EquipmentMetadata, EquipmentType};
use game_core::game::data::event_pools::{EventPhasePool, EventPoolConfig, WeightedEvent};
use game_core::game::data::random_event_data::{RandomEventDatabase, RandomEventMetadata};
use game_core::game::data::shop_data::{ShopDatabase, ShopMetadata, ShopType};
use game_core::game::data::GameDataBase;
use game_core::game::enums::RiskLevel;
use game_core::game::events::event_selection::random::RandomEventType;
use uuid::Uuid;

/// 테스트용 GameDataBase 생성 - 하드코딩된 예측 가능한 데이터
///
/// Dawn Phase I에서
/// - 리롤 가능한 상점 1개
/// - 최소한의 아이템/장비/아티팩트
/// - 예측 가능한 UUID
/// 를 사용합니다.
#[cfg(test)]
pub fn create_test_game_data() -> Arc<GameDataBase> {
    // 고정 UUID (테스트 재현성용)

    use game_core::game::data::random_event_data::RandomEventInnerMetadata;
    let shop_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let bonus_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
    let event_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap();

    // 아티팩트 메타데이터
    let artifact1 = ArtifactMetadata {
        id: "test_artifact_1".to_string(),
        uuid: Uuid::parse_str("a0000001-0000-0000-0000-000000000001").unwrap(),
        name: "Test Artifact 1".to_string(),
        description: "Test artifact for testing".to_string(),
        rarity: RiskLevel::HE,
        price: 100,
        modifiers: Vec::new(),
    };

    let artifact2 = ArtifactMetadata {
        id: "test_artifact_2".to_string(),
        uuid: Uuid::parse_str("a0000002-0000-0000-0000-000000000002").unwrap(),
        name: "Test Artifact 2".to_string(),
        description: "Another test artifact".to_string(),
        rarity: RiskLevel::WAW,
        price: 200,
        modifiers: Vec::new(),
    };

    let artifact3 = ArtifactMetadata {
        id: "test_artifact_3".to_string(),
        uuid: Uuid::parse_str("a0000002-0000-0000-0000-000000000003").unwrap(),
        name: "Test Artifact 3".to_string(),
        description: "Another test artifact".to_string(),
        rarity: RiskLevel::WAW,
        price: 200,
        modifiers: Vec::new(),
    };

    // 장비 메타데이터
    let equipment1 = EquipmentMetadata {
        id: "test_weapon_1".to_string(),
        uuid: Uuid::parse_str("e0000001-0000-0000-0000-000000000001").unwrap(),
        name: "Test Weapon".to_string(),
        equipment_type: EquipmentType::Weapon,
        rarity: RiskLevel::HE,
        price: 150,
        modifiers: Vec::new(),
    };

    let equipment2 = EquipmentMetadata {
        id: "test_suit_1".to_string(),
        uuid: Uuid::parse_str("e0000002-0000-0000-0000-000000000002").unwrap(),
        name: "Test Suit".to_string(),
        equipment_type: EquipmentType::Suit,
        rarity: RiskLevel::TETH,
        price: 80,
        modifiers: Vec::new(),
    };

    let equipment3 = EquipmentMetadata {
        id: "test_suit_2".to_string(),
        uuid: Uuid::parse_str("e0000002-0000-0000-0000-000000000003").unwrap(),
        name: "Test Suit 2".to_string(),
        equipment_type: EquipmentType::Suit,
        rarity: RiskLevel::TETH,
        price: 80,
        modifiers: Vec::new(),
    };

    // 환상체 메타데이터
    let abnormality1 = AbnormalityMetadata {
        id: "test_abnorm_1".to_string(),
        uuid: Uuid::parse_str("b0000001-0000-0000-0000-000000000001").unwrap(),
        name: "Test Abnormality".to_string(),
        risk_level: RiskLevel::HE,
        price: 120,
        max_health: 100,
        attack: 30,
        defense: 5,
        attack_interval_ms: 1500,
        abilities: Vec::new(),
    };

    // 상점 메타데이터 (리롤 가능)
    // visible_items / hidden_items 를 나눠서 reroll 테스트가 의미 있게 동작하도록 구성
    let shop = ShopMetadata {
        id: "test_shop".to_string(),
        name: "Test Rerollable Shop".to_string(),
        uuid: shop_uuid,
        shop_type: ShopType::Shop,
        can_reroll: true,
        visible_items: vec![artifact1.uuid, equipment1.uuid],
        hidden_items: vec![
            artifact2.uuid,
            equipment2.uuid,
            equipment3.uuid,
            abnormality1.uuid,
        ],
    };

    // 보너스 메타데이터 (고정 100 엔케팔린)
    let bonus = BonusMetadata {
        id: "test_bonus_enkephalin".to_string(),
        bonus_type: BonusType::Enkephalin,
        uuid: bonus_uuid,
        name: "Test Enkephalin Bonus".to_string(),
        description: "Grants fixed 100 Enkephalin".to_string(),
        icon: "enkephalin_icon.png".to_string(),
        amount: 100,
    };

    // 랜덤 이벤트 메타데이터
    let random_event = RandomEventMetadata {
        id: "test_event".to_string(),
        uuid: event_uuid,
        event_type: RandomEventType::Bonus,
        name: "Test Event".to_string(),
        description: "A test event".to_string(),
        image: "test.png".to_string(),
        risk_level: RiskLevel::ALEPH,
        inner_metadata: RandomEventInnerMetadata::Bonus(bonus_uuid),
    };

    // 이벤트 풀 (Dawn만 설정)
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

    // 각 Database 생성
    let artifacts_db = ArtifactDatabase::new(vec![artifact1, artifact2, artifact3]);
    let equipments_db = EquipmentDatabase::new(vec![equipment1, equipment2, equipment3]);
    let abnormalities_db = AbnormalityDatabase::new(vec![abnormality1]);

    let shops_db = ShopDatabase::new(vec![shop]);
    let bonuses_db = BonusDatabase::new(vec![bonus]);
    let mut random_events_db = RandomEventDatabase::new(vec![random_event]);

    // 랜덤 이벤트 내부 맵 초기화
    random_events_db.init_map();

    Arc::new(GameDataBase::new(
        Arc::new(abnormalities_db),
        Arc::new(artifacts_db),
        Arc::new(equipments_db),
        Arc::new(shops_db),
        Arc::new(bonuses_db),
        Arc::new(random_events_db),
        event_pools,
    ))
}

/// 실제 RON 파일에서 GameDataBase 로드
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
pub fn load_game_data_from_ron() -> Arc<GameDataBase> {
    // 1. RON 파일 포함 (컴파일 타임)
    let shops_ron = include_str!("../../../game_resources/data/events/shops/base.ron");
    let random_shops_ron = include_str!("../../../game_resources/data/events/shops/random.ron");
    let bonuses_ron = include_str!("../../../game_resources/data/events/bonuses/base.ron");
    let random_bonuses_ron = include_str!("../../../game_resources/data/events/bonuses/random.ron");
    let random_events_ron = include_str!("../../../game_resources/data/events/random_events.ron");
    let event_pools_ron = include_str!("../../../game_resources/data/events/event_pools.ron");
    let abnormalities_ron = include_str!("../../../game_resources/data/abnormalities/base.ron");
    let random_abnormalities_ron =
        include_str!("../../../game_resources/data/abnormalities/random.ron");
    let equipments_ron = include_str!("../../../game_resources/data/equipments/base.ron");
    let artifacts_ron = include_str!("../../../game_resources/data/artifacts/base.ron");

    // 2. RON 역직렬화
    let mut shops_db: ShopDatabase =
        ron::de::from_str(shops_ron).expect("Failed to deserialize shops.ron");
    let random_shops_db: ShopDatabase =
        ron::de::from_str(random_shops_ron).expect("Failed to deserialize random_shops.ron");

    let mut bonuses_db: BonusDatabase =
        ron::de::from_str(bonuses_ron).expect("Failed to deserialize bonuses.ron");
    let random_bonuses_db: BonusDatabase =
        ron::de::from_str(random_bonuses_ron).expect("Failed to deserialize random_bonuses.ron");

    let mut random_events_db: RandomEventDatabase =
        ron::de::from_str(random_events_ron).expect("Failed to deserialize random_events.ron");

    let event_pools: EventPoolConfig =
        ron::de::from_str(event_pools_ron).expect("Failed to deserialize event_pools.ron");

    let mut abnormalities_db: AbnormalityDatabase =
        ron::de::from_str(abnormalities_ron).expect("Failed to deserialize abnormalities.ron");
    let random_abnormalities_db: AbnormalityDatabase = ron::de::from_str(random_abnormalities_ron)
        .expect("Failed to deserialize random_abnormalities.ron");

    let equipments_db: EquipmentDatabase =
        ron::de::from_str(equipments_ron).expect("Failed to deserialize equipments.ron");

    let artifacts_db: ArtifactDatabase =
        ron::de::from_str(artifacts_ron).expect("Failed to deserialize artifacts.ron");

    // 3. 보조 맵 / lookup 테이블 초기화
    random_events_db.init_map();

    // 4. 랜덤 이벤트 전용 상점들을 메인 ShopDatabase 에 합침
    shops_db.shops.extend(random_shops_db.shops);

    // 4. 랜덤 전용 보너스 / 기물 병합
    bonuses_db.bonuses.extend(random_bonuses_db.bonuses);
    abnormalities_db.items.extend(random_abnormalities_db.items);

    Arc::new(GameDataBase::new(
        Arc::new(abnormalities_db),
        Arc::new(artifacts_db),
        Arc::new(equipments_db),
        Arc::new(shops_db),
        Arc::new(bonuses_db),
        Arc::new(random_events_db),
        event_pools,
    ))
}
