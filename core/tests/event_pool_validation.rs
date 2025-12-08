/// 이벤트 풀 검증 테스트
///
/// 목적:
/// 1. EventPhasePool에서 UUID를 제대로 선택하는가?
/// 2. 선택한 UUID가 GameData에 실제로 존재하는가?
/// 3. Generator들이 올바른 메타데이터를 반환하는가?
/// 4. 폴백 케이스가 제대로 작동하는가?
use game_core::game::data::abnormality_data::AbnormalityDatabase;
use game_core::game::data::artifact_data::ArtifactDatabase;
use game_core::game::data::bonus_data::{BonusDatabase, BonusMetadata, BonusType};
use game_core::game::data::equipment_data::EquipmentDatabase;
use game_core::game::data::event_pools::{EventPhasePool, EventPoolConfig, WeightedEvent};
use game_core::game::data::random_event_data::{
    RandomEventDatabase, RandomEventInnerMetadata, RandomEventMetadata,
};
use game_core::game::data::shop_data::{ShopDatabase, ShopMetadata, ShopType};
use game_core::game::data::GameDataBase;
use game_core::game::enums::{GameOption, OrdealType, RiskLevel};
use game_core::game::events::event_selection::bonus::BonusGenerator;
use game_core::game::events::event_selection::random::{RandomEventGenerator, RandomEventType};
use game_core::game::events::event_selection::shop::ShopGenerator;
use game_core::game::events::{EventGenerator, GeneratorContext};
use rand::SeedableRng;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================
// 테스트 데이터 생성
// ============================================================

/// 최소 테스트용 GameDataBase 생성
fn create_minimal_game_data() -> Arc<GameDataBase> {
    // 고정 UUID (예측 가능한 테스트용)
    let shop_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let bonus_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
    let event_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap();

    // Shop 데이터
    let shop = ShopMetadata {
        id: "test_shop".to_string(),
        name: "Test Shop".to_string(),
        uuid: shop_uuid,
        shop_type: ShopType::Shop,
        can_reroll: true,
        visible_items: vec![],
        hidden_items: vec![],
    };

    // Bonus 데이터
    let bonus = BonusMetadata {
        id: "test_bonus".to_string(),
        bonus_type: BonusType::Enkephalin,
        uuid: bonus_uuid,
        name: "Test Bonus".to_string(),
        description: "Test bonus description".to_string(),
        icon: "test.png".to_string(),
        amount: 75,
    };

    // RandomEvent 데이터
    let random_event = RandomEventMetadata {
        id: "test_event".to_string(),
        uuid: event_uuid,
        event_type: RandomEventType::Bonus,
        name: "Test Event".to_string(),
        description: "Test event description".to_string(),
        image: "test.png".to_string(),
        risk_level: RiskLevel::ZAYIN,
        inner_metadata: RandomEventInnerMetadata::Bonus(Uuid::nil()),
    };

    // 이벤트 풀 (Dawn만 설정)
    let dawn_pool = EventPhasePool {
        shops: vec![WeightedEvent {
            weight: 100,
            uuid: shop_uuid,
        }],
        bonuses: vec![WeightedEvent {
            weight: 100,
            uuid: bonus_uuid,
        }],
        random_events: vec![WeightedEvent {
            weight: 100,
            uuid: event_uuid,
        }],
    };

    // 빈 풀 (다른 Ordeal용)
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

    // GameDataBase 생성
    let abnormality_data = Arc::new(AbnormalityDatabase::new(vec![]));
    let artifact_data = Arc::new(ArtifactDatabase::new(vec![]));
    let equipment_data = Arc::new(EquipmentDatabase::new(vec![]));
    let shop_data = Arc::new(ShopDatabase::new(vec![shop]));
    let bonus_data = Arc::new(BonusDatabase::new(vec![bonus]));
    let random_event_data = Arc::new(RandomEventDatabase::new(vec![random_event]));

    Arc::new(GameDataBase::new(
        abnormality_data,
        artifact_data,
        equipment_data,
        shop_data,
        bonus_data,
        random_event_data,
        event_pools,
    ))
}

/// 여러 아이템이 있는 GameDataBase 생성 (가중치 테스트용)
fn create_weighted_game_data() -> Arc<GameDataBase> {
    // 3개의 상점 (가중치 다름)
    let shop1_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000011").unwrap();
    let shop2_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000012").unwrap();
    let shop3_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000013").unwrap();

    let shop1 = ShopMetadata {
        id: "shop_common".to_string(),
        name: "Common Shop".to_string(),
        uuid: shop1_uuid,
        shop_type: ShopType::Shop,
        can_reroll: true,
        visible_items: vec![],
        hidden_items: vec![],
    };

    let shop2 = ShopMetadata {
        id: "shop_rare".to_string(),
        name: "Rare Shop".to_string(),
        uuid: shop2_uuid,
        shop_type: ShopType::Shop,
        can_reroll: true,
        visible_items: vec![],
        hidden_items: vec![],
    };

    let shop3 = ShopMetadata {
        id: "shop_legendary".to_string(),
        name: "Legendary Shop".to_string(),
        uuid: shop3_uuid,
        shop_type: ShopType::DiscountShop,
        can_reroll: false,
        visible_items: vec![],
        hidden_items: vec![],
    };

    // 가중치 풀: Common(70%), Rare(25%), Legendary(5%)
    let weighted_pool = EventPhasePool {
        shops: vec![
            WeightedEvent {
                weight: 70,
                uuid: shop1_uuid,
            },
            WeightedEvent {
                weight: 25,
                uuid: shop2_uuid,
            },
            WeightedEvent {
                weight: 5,
                uuid: shop3_uuid,
            },
        ],
        bonuses: vec![],
        random_events: vec![],
    };

    let empty_pool = EventPhasePool {
        shops: vec![],
        bonuses: vec![],
        random_events: vec![],
    };

    let event_pools = EventPoolConfig {
        dawn: weighted_pool,
        noon: empty_pool.clone(),
        dusk: empty_pool.clone(),
        midnight: empty_pool.clone(),
        white: empty_pool,
    };

    let abnormality_data = Arc::new(AbnormalityDatabase::new(vec![]));
    let artifact_data = Arc::new(ArtifactDatabase::new(vec![]));
    let equipment_data = Arc::new(EquipmentDatabase::new(vec![]));
    let shop_data = Arc::new(ShopDatabase::new(vec![shop1, shop2, shop3]));
    let bonus_data = Arc::new(BonusDatabase::new(vec![]));
    let random_event_data = Arc::new(RandomEventDatabase::new(vec![]));

    Arc::new(GameDataBase::new(
        abnormality_data,
        artifact_data,
        equipment_data,
        shop_data,
        bonus_data,
        random_event_data,
        event_pools,
    ))
}

// ============================================================
// EventPhasePool 기본 기능 테스트
// ============================================================

#[test]
fn test_choose_weighted_uuid_single_item() {
    let pool = vec![WeightedEvent {
        weight: 100,
        uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
    }];

    let mut rng = rand::rngs::StdRng::seed_from_u64(12345);
    let result = EventPhasePool::choose_weighted_uuid(&pool, &mut rng);

    assert!(result.is_some());
    assert_eq!(
        result.unwrap(),
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    );
}

#[test]
fn test_choose_weighted_uuid_empty_pool() {
    let pool: Vec<WeightedEvent> = vec![];

    let mut rng = rand::rngs::StdRng::seed_from_u64(12345);
    let result = EventPhasePool::choose_weighted_uuid(&pool, &mut rng);

    assert!(result.is_none());
}

#[test]
fn test_choose_weighted_uuid_zero_weight() {
    let pool = vec![
        WeightedEvent {
            weight: 0,
            uuid: Uuid::new_v4(),
        },
        WeightedEvent {
            weight: 0,
            uuid: Uuid::new_v4(),
        },
    ];

    let mut rng = rand::rngs::StdRng::seed_from_u64(12345);
    let result = EventPhasePool::choose_weighted_uuid(&pool, &mut rng);

    // 가중치가 모두 0이면 None 반환
    assert!(result.is_none());
}

#[test]
fn test_choose_weighted_uuid_deterministic() {
    let pool = vec![
        WeightedEvent {
            weight: 50,
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
        },
        WeightedEvent {
            weight: 50,
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
        },
    ];

    // 같은 seed → 같은 결과
    let mut rng1 = rand::rngs::StdRng::seed_from_u64(99999);
    let result1 = EventPhasePool::choose_weighted_uuid(&pool, &mut rng1);

    let mut rng2 = rand::rngs::StdRng::seed_from_u64(99999);
    let result2 = EventPhasePool::choose_weighted_uuid(&pool, &mut rng2);

    assert_eq!(result1, result2);
}

// ============================================================
// ShopGenerator 테스트
// ============================================================

#[test]
fn test_shop_generator_returns_valid_shop() {
    let game_data = create_minimal_game_data();
    let world = bevy_ecs::world::World::new();

    let ctx = GeneratorContext::new(&world, &game_data, 12345);
    let generator = ShopGenerator;

    let result = generator.generate(&ctx);

    // GameOption::Shop이어야 함
    match result {
        GameOption::Shop { shop } => {
            assert_eq!(shop.name, "Test Shop");
            assert_eq!(shop.shop_type, ShopType::Shop);
            assert!(shop.can_reroll);
        }
        _ => panic!("Expected GameOption::Shop"),
    }
}

#[test]
fn test_shop_generator_uuid_exists_in_database() {
    let game_data = create_minimal_game_data();
    let world = bevy_ecs::world::World::new();

    let ctx = GeneratorContext::new(&world, &game_data, 12345);
    let generator = ShopGenerator;

    let result = generator.generate(&ctx);

    // UUID가 GameData에 실제로 존재하는지 확인
    match result {
        GameOption::Shop { shop } => {
            let found = game_data.shop_data.get_by_uuid(&shop.uuid);
            assert!(found.is_some(), "Shop UUID should exist in database");
            assert_eq!(found.unwrap().name, "Test Shop");
        }
        _ => panic!("Expected GameOption::Shop"),
    }
}

#[test]
fn test_shop_generator_empty_pool_fallback() {
    let game_data = create_minimal_game_data();
    let mut world = bevy_ecs::world::World::new();

    // Noon Pool (비어있음)
    use game_core::ecs::resources::GameProgression;
    world.insert_resource(GameProgression {
        current_ordeal: OrdealType::Noon,
        current_phase: game_core::game::enums::PhaseType::I,
    });

    let ctx = GeneratorContext::new(&world, &game_data, 12345);
    let generator = ShopGenerator;

    let result = generator.generate(&ctx);

    // 폴백: 빈 풀일 때 임시 상점 반환
    match result {
        GameOption::Shop { shop } => {
            assert_eq!(shop.name, "임시 상점");
            assert_eq!(shop.uuid, Uuid::nil());
        }
        _ => panic!("Expected GameOption::Shop"),
    }
}

#[test]
fn test_shop_generator_weighted_selection() {
    let game_data = create_weighted_game_data();
    let world = bevy_ecs::world::World::new();

    // 100번 시도해서 통계 수집
    let mut common_count = 0;
    let mut rare_count = 0;
    let mut legendary_count = 0;

    for seed in 0..100 {
        let ctx = GeneratorContext::new(&world, &game_data, seed);
        let generator = ShopGenerator;
        let result = generator.generate(&ctx);

        match result {
            GameOption::Shop { shop } => match shop.id.as_str() {
                "shop_common" => common_count += 1,
                "shop_rare" => rare_count += 1,
                "shop_legendary" => legendary_count += 1,
                _ => {}
            },
            _ => panic!("Expected GameOption::Shop"),
        }
    }

    println!(
        "Common: {}, Rare: {}, Legendary: {}",
        common_count, rare_count, legendary_count
    );

    // 가중치 검증: Common이 가장 많이 나와야 함
    assert!(
        common_count > rare_count,
        "Common should appear more than Rare"
    );
    assert!(
        rare_count > legendary_count,
        "Rare should appear more than Legendary"
    );
}

// ============================================================
// BonusGenerator 테스트
// ============================================================

#[test]
fn test_bonus_generator_returns_valid_bonus() {
    let game_data = create_minimal_game_data();
    let world = bevy_ecs::world::World::new();

    let ctx = GeneratorContext::new(&world, &game_data, 12345);
    let generator = BonusGenerator;

    let result = generator.generate(&ctx);

    match result {
        GameOption::Bonus { bonus } => {
            assert_eq!(bonus.name, "Test Bonus");
            assert_eq!(bonus.bonus_type, BonusType::Enkephalin);
            assert_eq!(bonus.amount, 75);
        }
        _ => panic!("Expected GameOption::Bonus"),
    }
}

#[test]
fn test_bonus_generator_uuid_exists_in_database() {
    let game_data = create_minimal_game_data();
    let world = bevy_ecs::world::World::new();

    let ctx = GeneratorContext::new(&world, &game_data, 12345);
    let generator = BonusGenerator;

    let result = generator.generate(&ctx);

    match result {
        GameOption::Bonus { bonus } => {
            let found = game_data.bonus_data.get_by_uuid(&bonus.uuid);
            assert!(found.is_some(), "Bonus UUID should exist in database");
            assert_eq!(found.unwrap().name, "Test Bonus");
        }
        _ => panic!("Expected GameOption::Bonus"),
    }
}

// ============================================================
// RandomEventGenerator 테스트
// ============================================================

#[test]
fn test_random_event_generator_returns_valid_event() {
    let game_data = create_minimal_game_data();
    let world = bevy_ecs::world::World::new();

    let ctx = GeneratorContext::new(&world, &game_data, 12345);
    let generator = RandomEventGenerator;

    let result = generator.generate(&ctx);

    match result {
        GameOption::Random { event } => {
            assert_eq!(event.name, "Test Event");
            assert_eq!(event.event_type, RandomEventType::Bonus);
            assert_eq!(event.risk_level, RiskLevel::ZAYIN);
        }
        _ => panic!("Expected GameOption::Random"),
    }
}

#[test]
fn test_random_event_generator_uuid_exists_in_database() {
    let game_data = create_minimal_game_data();
    let world = bevy_ecs::world::World::new();

    let ctx = GeneratorContext::new(&world, &game_data, 12345);
    let generator = RandomEventGenerator;

    let result = generator.generate(&ctx);

    match result {
        GameOption::Random { event } => {
            let found = game_data.random_event_data.get_by_uuid(&event.uuid);
            assert!(found.is_some(), "Event UUID should exist in database");
            assert_eq!(found.unwrap().name, "Test Event");
        }
        _ => panic!("Expected GameOption::Random"),
    }
}

// ============================================================
// EventPoolConfig 통합 테스트
// ============================================================

#[test]
fn test_event_pool_config_get_pool() {
    let game_data = create_minimal_game_data();

    // Dawn pool 가져오기
    let dawn_pool = game_data.event_pools.get_pool(OrdealType::Dawn);
    assert_eq!(dawn_pool.shops.len(), 1);
    assert_eq!(dawn_pool.bonuses.len(), 1);
    assert_eq!(dawn_pool.random_events.len(), 1);

    // Noon pool 가져오기 (비어있어야 함)
    let noon_pool = game_data.event_pools.get_pool(OrdealType::Noon);
    assert_eq!(noon_pool.shops.len(), 0);
    assert_eq!(noon_pool.bonuses.len(), 0);
    assert_eq!(noon_pool.random_events.len(), 0);
}

// ============================================================
// Generator 일관성 테스트
// ============================================================

#[test]
fn test_all_generators_return_data_from_pool() {
    let game_data = create_minimal_game_data();
    let world = bevy_ecs::world::World::new();
    let ctx = GeneratorContext::new(&world, &game_data, 12345);

    // ShopGenerator
    let shop_result = ShopGenerator.generate(&ctx);
    match shop_result {
        GameOption::Shop { shop } => {
            assert_eq!(
                shop.uuid,
                Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
            );
        }
        _ => panic!("Expected Shop"),
    }

    // BonusGenerator
    let bonus_result = BonusGenerator.generate(&ctx);
    match bonus_result {
        GameOption::Bonus { bonus } => {
            assert_eq!(
                bonus.uuid,
                Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
            );
        }
        _ => panic!("Expected Bonus"),
    }

    // RandomEventGenerator
    let event_result = RandomEventGenerator.generate(&ctx);
    match event_result {
        GameOption::Random { event } => {
            assert_eq!(
                event.uuid,
                Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap()
            );
        }
        _ => panic!("Expected Random"),
    }
}

#[test]
fn test_generators_with_same_seed_return_same_results() {
    let game_data = create_minimal_game_data();
    let world = bevy_ecs::world::World::new();

    let seed = 99999;

    // 첫 번째 실행
    let ctx1 = GeneratorContext::new(&world, &game_data, seed);
    let shop1 = ShopGenerator.generate(&ctx1);

    // 두 번째 실행 (같은 seed)
    let ctx2 = GeneratorContext::new(&world, &game_data, seed);
    let shop2 = ShopGenerator.generate(&ctx2);

    // 같은 결과여야 함
    match (shop1, shop2) {
        (GameOption::Shop { shop: s1 }, GameOption::Shop { shop: s2 }) => {
            assert_eq!(s1.uuid, s2.uuid);
            assert_eq!(s1.name, s2.name);
        }
        _ => panic!("Both should be Shop options"),
    }
}
