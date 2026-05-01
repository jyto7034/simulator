#![allow(dead_code)]

pub const BOARD_WIDTH: u8 = 7;
pub const BOARD_HEIGHT: u8 = 8;
pub const BOARD_SIZE: (u8, u8) = (BOARD_WIDTH, BOARD_HEIGHT);

use std::path::PathBuf;
use std::sync::Arc;

use game_core::game::ability::{
    DeliveryDef, SkillCastTargetingDef, SkillDef, SkillKind, SkillPresentationDef, SkillStepDef,
    SkillTarget, StepTargetingMode,
};
use game_core::game::battle::timeline::Timeline;
use game_core::game::data::abnormality_data::{AbnormalityDatabase, AbnormalityMetadata};
use game_core::game::data::artifact_data::{ArtifactDatabase, ArtifactMetadata};
use game_core::game::data::bonus_data::{BonusDatabase, BonusMetadata, BonusType};
use game_core::game::data::equipment_data::{EquipmentDatabase, EquipmentMetadata, EquipmentType};
use game_core::game::data::event_pools::{EventPhasePool, EventPoolConfig, WeightedEvent};
use game_core::game::data::pve_data::{
    PveEncounter, PveEncounterDatabase, PvePosition, PveUnitData,
};
use game_core::game::data::random_event_data::{
    RandomEventDatabase, RandomEventInnerMetadata, RandomEventMetadata,
};
use game_core::game::data::shop_data::{ShopDatabase, ShopMetadata, ShopType};
use game_core::game::data::skill_data::SkillDatabase;
use game_core::game::data::GameDataBase;
use game_core::game::enums::{RewardMode, RiskLevel};
use game_core::game::events::event_selection::random::RandomEventType;
use uuid::Uuid;

pub fn timeline_exports_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("timeline_exports")
}

pub fn write_timeline_export(name: &str, timeline: &Timeline) -> PathBuf {
    let out_dir = timeline_exports_dir();
    let out_path = out_dir.join(format!("{name}.json"));
    let parent_dir = out_path.parent().unwrap_or_else(|| {
        panic!(
            "timeline export path must have parent: {}",
            out_path.display()
        )
    });
    std::fs::create_dir_all(parent_dir).expect("create timeline_exports directory");
    timeline
        .write_pretty_json(&out_path)
        .expect("write timeline json");
    out_path
}

pub fn empty_event_pools() -> EventPoolConfig {
    let empty = EventPhasePool {
        shops: vec![],
        bonuses: vec![],
        random_events: vec![],
    };
    EventPoolConfig {
        dawn: empty.clone(),
        noon: empty.clone(),
        dusk: empty.clone(),
        midnight: empty.clone(),
        white: empty,
    }
}

pub fn empty_game_data() -> Arc<GameDataBase> {
    Arc::new(GameDataBase::new(
        game_core::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools: empty_event_pools(),
        },
    ))
}

/// 테스트용 `GameDataBase` 생성 (작고 결정적인 데이터).
///
/// - 리롤 가능한 상점 1개 (visible/hidden 구성)
/// - Equipment/Artifact/Abnormality 최소 1개
/// - Bonus/RandomEvent/PvE encounter 최소 1개
pub fn create_test_game_data() -> Arc<GameDataBase> {
    let shop_uuid = Uuid::from_u128(1);
    let bonus_uuid = Uuid::from_u128(2);
    let event_uuid = Uuid::from_u128(3);

    let artifact1 = ArtifactMetadata {
        id: "test_artifact_1".to_string(),
        uuid: Uuid::from_u128(0xA000_0001),
        name: "Test Artifact 1".to_string(),
        description: "Test artifact for testing".to_string(),
        rarity: RiskLevel::HE,
        price: 100,
        triggered_effects: Default::default(),
        ability_activations: vec![],
    };
    let artifact2 = ArtifactMetadata {
        id: "test_artifact_2".to_string(),
        uuid: Uuid::from_u128(0xA000_0002),
        name: "Test Artifact 2".to_string(),
        description: "Another test artifact".to_string(),
        rarity: RiskLevel::WAW,
        price: 200,
        triggered_effects: Default::default(),
        ability_activations: vec![],
    };

    let equipment1 = EquipmentMetadata {
        id: "test_weapon_1".to_string(),
        uuid: Uuid::from_u128(0xE000_0001),
        name: "Test Weapon".to_string(),
        equipment_type: EquipmentType::Weapon,
        rarity: RiskLevel::HE,
        price: 150,
        allow_duplicate_equip: true,
        triggered_effects: Default::default(),
        ability_activations: vec![],
    };
    let equipment2 = EquipmentMetadata {
        id: "test_armor_1".to_string(),
        uuid: Uuid::from_u128(0xE000_0002),
        name: "Test Armor".to_string(),
        equipment_type: EquipmentType::Armor,
        rarity: RiskLevel::TETH,
        price: 80,
        allow_duplicate_equip: true,
        triggered_effects: Default::default(),
        ability_activations: vec![],
    };

    let skill_id = "test_skill".to_string();
    let skills_db = SkillDatabase::new(vec![SkillDef {
        id: skill_id.clone(),
        name: "test_skill".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 0,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "step_01".to_string(),
            delay_ms: 0,
            range_units: 1.0,
            target: SkillTarget::SelfUnit,
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Instant,
            effects: vec![],
            presentation: SkillPresentationDef::default(),
        }],
    }]);

    let abnormality1 = AbnormalityMetadata {
        id: "test_abnorm_1".to_string(),
        uuid: Uuid::from_u128(0xB000_0001),
        name: "Test Abnormality".to_string(),
        risk_level: RiskLevel::HE,
        price: 120,
        max_health: 100,
        attack: 30,
        defense: 5,
        magic_resist: 0,
        movement: Default::default(),
        basic_attack: Default::default(),
        resonance: Default::default(),
        skill_id: Some(skill_id),
    };
    let abnormality2 = AbnormalityMetadata {
        id: "test_abnorm_2".to_string(),
        uuid: Uuid::from_u128(0xB000_0002),
        name: "Test Abnormality 2".to_string(),
        risk_level: RiskLevel::TETH,
        price: 90,
        max_health: 80,
        attack: 20,
        defense: 4,
        magic_resist: 0,
        movement: Default::default(),
        basic_attack: Default::default(),
        resonance: Default::default(),
        skill_id: None,
    };
    let abnormality3 = AbnormalityMetadata {
        id: "test_abnorm_3".to_string(),
        uuid: Uuid::from_u128(0xB000_0003),
        name: "Test Abnormality 3".to_string(),
        risk_level: RiskLevel::ZAYIN,
        price: 70,
        max_health: 70,
        attack: 18,
        defense: 3,
        magic_resist: 0,
        movement: Default::default(),
        basic_attack: Default::default(),
        resonance: Default::default(),
        skill_id: None,
    };

    let shop = ShopMetadata {
        id: "test_shop".to_string(),
        name: "Test Rerollable Shop".to_string(),
        uuid: shop_uuid,
        shop_type: ShopType::Shop,
        can_reroll: true,
        visible_items: vec![artifact1.uuid, equipment1.uuid],
        hidden_items: vec![artifact2.uuid, equipment2.uuid, abnormality1.uuid],
    };

    let bonus = BonusMetadata {
        id: "test_bonus_enkephalin".to_string(),
        bonus_type: BonusType::Enkephalin,
        uuid: bonus_uuid,
        name: "Test Enkephalin Bonus".to_string(),
        description: "Grants fixed Enkephalin".to_string(),
        icon: "enkephalin_icon.png".to_string(),
        amount: 100,
    };

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
    let mut event_pools = empty_event_pools();
    event_pools.dawn = dawn_pool;

    let artifacts_db = ArtifactDatabase::new(vec![artifact1, artifact2]);
    let equipments_db = EquipmentDatabase::new(vec![equipment1, equipment2]);
    let abnormalities_db = AbnormalityDatabase::new(vec![
        abnormality1.clone(),
        abnormality2.clone(),
        abnormality3.clone(),
    ]);
    let shops_db = ShopDatabase::new(vec![shop]);
    let bonuses_db = BonusDatabase::new(vec![bonus]);

    let random_events_db = RandomEventDatabase::new(vec![random_event]);

    let pve_db = PveEncounterDatabase::new(vec![
        PveEncounter {
            id: "pve_test_1".to_string(),
            abnormality_id: abnormality1.id.clone(),
            difficulty: 1,
            risk_level: RiskLevel::ZAYIN,
            reward_mode: RewardMode::ChooseOne,
            reward_bonus_uuids: vec![bonus_uuid],
            units: vec![PveUnitData {
                abnormality_id: abnormality1.id.clone(),
                position: PvePosition { x: 1, y: 1 },
                tier: game_core::game::enums::Tier::I,
            }],
            static_obstacles: vec![],
        },
        PveEncounter {
            id: "pve_test_2".to_string(),
            abnormality_id: abnormality2.id.clone(),
            difficulty: 1,
            risk_level: RiskLevel::TETH,
            reward_mode: RewardMode::ChooseOne,
            reward_bonus_uuids: vec![bonus_uuid],
            units: vec![PveUnitData {
                abnormality_id: abnormality2.id.clone(),
                position: PvePosition { x: 2, y: 1 },
                tier: game_core::game::enums::Tier::I,
            }],
            static_obstacles: vec![],
        },
        PveEncounter {
            id: "pve_test_3".to_string(),
            abnormality_id: abnormality3.id.clone(),
            difficulty: 1,
            risk_level: RiskLevel::ZAYIN,
            reward_mode: RewardMode::ChooseOne,
            reward_bonus_uuids: vec![bonus_uuid],
            units: vec![PveUnitData {
                abnormality_id: abnormality3.id.clone(),
                position: PvePosition { x: 3, y: 1 },
                tier: game_core::game::enums::Tier::I,
            }],
            static_obstacles: vec![],
        },
    ]);

    Arc::new(GameDataBase::new(
        game_core::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(abnormalities_db),
            artifact_data: Arc::new(artifacts_db),
            equipment_data: Arc::new(equipments_db),
            shop_data: Arc::new(shops_db),
            bonus_data: Arc::new(bonuses_db),
            random_event_data: Arc::new(random_events_db),
            pve_data: Arc::new(pve_db),
            skill_data: Arc::new(skills_db),
            event_pools,
        },
    ))
}

/// `create_test_game_data()`를 기반으로, RandomEvent 1개만 교체한 테스트용 GameData 생성.
///
/// - Random 옵션 라우팅(Shop/Bonus/Suppress) 테스트에 사용
pub fn create_test_game_data_with_random_event(
    random_event: RandomEventMetadata,
) -> Arc<GameDataBase> {
    let base = create_test_game_data();
    let base = base.as_ref();

    let random_events_db = RandomEventDatabase::new(vec![random_event.clone()]);

    let mut event_pools = base.event_pools.clone();
    event_pools.dawn.random_events = vec![WeightedEvent {
        weight: 1,
        uuid: random_event.uuid,
    }];

    Arc::new(GameDataBase::new(
        game_core::game::data::GameDataBaseParts {
            abnormality_data: Arc::clone(&base.abnormality_data),
            artifact_data: Arc::clone(&base.artifact_data),
            equipment_data: Arc::clone(&base.equipment_data),
            shop_data: Arc::clone(&base.shop_data),
            bonus_data: Arc::clone(&base.bonus_data),
            random_event_data: Arc::new(random_events_db),
            pve_data: Arc::clone(&base.pve_data),
            skill_data: Arc::clone(&base.skill_data),
            event_pools,
        },
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
    // Given: RON 파일 include_str! 로 포함 (컴파일 타임)
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
    let skills_ron = include_str!("../../../game_resources/data/skills/base.ron");
    let pve_ron = include_str!("../../../game_resources/data/pve/encounters.ron");

    // When: RON 역직렬화
    let mut shops_db: ShopDatabase =
        ron::de::from_str(shops_ron).expect("Failed to deserialize shops.ron");
    let random_shops_db: ShopDatabase =
        ron::de::from_str(random_shops_ron).expect("Failed to deserialize random_shops.ron");

    let mut bonuses_db: BonusDatabase =
        ron::de::from_str(bonuses_ron).expect("Failed to deserialize bonuses.ron");
    let random_bonuses_db: BonusDatabase =
        ron::de::from_str(random_bonuses_ron).expect("Failed to deserialize random_bonuses.ron");

    let random_events_db: RandomEventDatabase =
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

    let skill_db: SkillDatabase =
        ron::de::from_str(skills_ron).expect("Failed to deserialize skills.ron");

    let pve_db: PveEncounterDatabase =
        ron::de::from_str(pve_ron).expect("Failed to deserialize pve encounters.ron");

    // When: 랜덤 이벤트 전용 상점들을 메인 ShopDatabase 에 합침
    shops_db.shops.extend(random_shops_db.shops);

    // When: 랜덤 전용 보너스 / 기물 병합
    bonuses_db.bonuses.extend(random_bonuses_db.bonuses);
    abnormalities_db.items.extend(random_abnormalities_db.items);

    Arc::new(GameDataBase::new(
        game_core::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(abnormalities_db),
            artifact_data: Arc::new(artifacts_db),
            equipment_data: Arc::new(equipments_db),
            shop_data: Arc::new(shops_db),
            bonus_data: Arc::new(bonuses_db),
            random_event_data: Arc::new(random_events_db),
            pve_data: Arc::new(pve_db),
            skill_data: Arc::new(skill_db),
            event_pools,
        },
    ))
}
