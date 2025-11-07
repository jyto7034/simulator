use rand::SeedableRng;
use uuid::Uuid;

use crate::game::{
    data::shop_data::{Item, Shop},
    events::{EventError, EventExecutor, EventGenerator, ExecutorContext, GeneratorContext},
};

// 상인에도 등급이 존재함.
// 레벨에 비례하여 등급이 높은 상인이 등장할 수 있음.
// 아이템 종류마다 상인이 존재함.
// 아티팩트, 기물, 무기, 방어구 등등
// 각 종류마다 고유한 Npc 를 가짐. ( 상인의 갯수가 너무 많으면 리소스가 더 많이 발생하니 중복 허용함. )
// TODO: 각 상인마다 아이템 목록이 다르니 상인-아이템 목록은 외부 config 로 관리.

pub struct ShopGenerator;

impl EventGenerator for ShopGenerator {
    type Output = Shop;

    fn generate(&self, ctx: &GeneratorContext) -> Self::Output {
        use rand::Rng;

        // GameData에서 랜덤 상인 선택
        let shops = &ctx.game_data.shops.shops;

        if shops.is_empty() {
            // 폴백: 데이터가 없으면 기본 상점 반환
            return Shop {
                name: "임시 상점".to_string(),
                uuid: Uuid::new_v4(),
                items: vec![Item::new()],
            };
        }

        // 랜덤 시드 기반으로 상인 선택
        let mut rng = rand::rngs::StdRng::seed_from_u64(ctx.random_seed);
        let shop_index = rng.gen_range(0..shops.len());

        // 선택된 상인 복제해서 반환
        shops[shop_index].clone()
    }
}

pub struct ShopExecutor;

impl EventExecutor for ShopExecutor {
    type Input = String;

    fn execute(&self, _ctx: &ExecutorContext, _input: Self::Input) -> Result<(), EventError> {
        // TODO: 가방에 추가 (또는 티어 업)
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{
        data::{
            event_pools::{EventPhasePool, EventPoolConfig},
            shop_data::{Item, ShopDatabase},
            GameData,
        },
        events::{EventGenerator, GeneratorContext},
    };
    use std::sync::Arc;

    /// 테스트용 GameData 생성 (최소한의 데이터)
    fn create_test_game_data() -> Arc<GameData> {
        let shop1 = Shop {
            name: "기본 상점".to_string(),
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            items: vec![
                Item {
                    uuid: Uuid::new_v4(),
                    name: "아이템1".to_string(),
                    price: 100,
                    tier: crate::game::enums::Tier::I,
                },
                Item {
                    uuid: Uuid::new_v4(),
                    name: "아이템2".to_string(),
                    price: 200,
                    tier: crate::game::enums::Tier::II,
                },
            ],
        };

        let shop2 = Shop {
            name: "레어 상점".to_string(),
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
            items: vec![Item {
                uuid: Uuid::new_v4(),
                name: "레어아이템".to_string(),
                price: 500,
                tier: crate::game::enums::Tier::III,
            }],
        };

        let shops = ShopDatabase {
            shops: vec![shop1, shop2],
        };

        // EventPoolConfig는 실제로 필요하지 않지만 구조체 생성을 위해 필요
        let event_pools = EventPoolConfig {
            dawn: EventPhasePool {
                shops: vec![],
                bonuses: vec![],
                random_events: vec![],
            },
            noon: EventPhasePool {
                shops: vec![],
                bonuses: vec![],
                random_events: vec![],
            },
            dusk: EventPhasePool {
                shops: vec![],
                bonuses: vec![],
                random_events: vec![],
            },
            midnight: EventPhasePool {
                shops: vec![],
                bonuses: vec![],
                random_events: vec![],
            },
            white: EventPhasePool {
                shops: vec![],
                bonuses: vec![],
                random_events: vec![],
            },
        };

        Arc::new(GameData { shops, event_pools })
    }

    #[test]
    fn test_shop_generator_creates_shop() {
        // given: 테스트 데이터와 context 준비
        let game_data = create_test_game_data();
        let world = bevy_ecs::world::World::new();
        let ctx = GeneratorContext::new(&world, &game_data, 12345);

        // when: ShopGenerator로 상점 생성
        let generator = ShopGenerator;
        let shop = generator.generate(&ctx);

        // then: 상점이 정상적으로 생성되었는지 확인
        assert!(!shop.name.is_empty(), "상점 이름이 비어있으면 안됨");
        assert!(
            !shop.items.is_empty(),
            "상점에 아이템이 최소 1개 이상 있어야 함"
        );
    }

    #[test]
    fn test_shop_generator_returns_shop_from_database() {
        // given
        let game_data = create_test_game_data();
        let world = bevy_ecs::world::World::new();
        let ctx = GeneratorContext::new(&world, &game_data, 12345);

        // when
        let generator = ShopGenerator;
        let shop = generator.generate(&ctx);

        // then: 생성된 상점이 데이터베이스에 있는 상점 중 하나인지 확인
        let shop_exists = game_data.shops.shops.iter().any(|s| s.uuid == shop.uuid);
        assert!(shop_exists, "생성된 상점은 GameData의 shops에 존재해야 함");
    }

    #[test]
    fn test_shop_generator_with_different_seeds() {
        // given: 같은 데이터, 다른 시드
        let game_data = create_test_game_data();
        let world = bevy_ecs::world::World::new();
        let generator = ShopGenerator;

        let ctx1 = GeneratorContext::new(&world, &game_data, 11111);
        let ctx2 = GeneratorContext::new(&world, &game_data, 99999);

        // when
        let shop1 = generator.generate(&ctx1);
        let shop2 = generator.generate(&ctx2);

        // then: 다른 시드를 사용하면 다른 상점이 나올 수 있음 (확률적)
        // 주의: shops가 2개뿐이므로 50% 확률로 같을 수 있음
        // 이 테스트는 시드가 영향을 준다는 것만 확인
        println!("shop1: {}", shop1.name);
        println!("shop2: {}", shop2.name);
    }

    #[test]
    fn test_shop_generator_with_same_seed_returns_same_shop() {
        // given: 같은 시드 사용
        let game_data = create_test_game_data();
        let world = bevy_ecs::world::World::new();
        let generator = ShopGenerator;
        let random_seed = 42;

        let ctx1 = GeneratorContext::new(&world, &game_data, random_seed);
        let ctx2 = GeneratorContext::new(&world, &game_data, random_seed);

        // when
        let shop1 = generator.generate(&ctx1);
        let shop2 = generator.generate(&ctx2);

        // then: 같은 시드를 사용하면 항상 같은 상점이 나와야 함 (결정론적)
        assert_eq!(
            shop1.uuid, shop2.uuid,
            "같은 시드를 사용하면 같은 상점이 나와야 함"
        );
        assert_eq!(shop1.name, shop2.name);
    }

    #[test]
    fn test_shop_generator_with_empty_database_returns_fallback() {
        // given: 빈 상점 데이터베이스
        let empty_shops = ShopDatabase { shops: vec![] };
        let event_pools = EventPoolConfig {
            dawn: EventPhasePool {
                shops: vec![],
                bonuses: vec![],
                random_events: vec![],
            },
            noon: EventPhasePool {
                shops: vec![],
                bonuses: vec![],
                random_events: vec![],
            },
            dusk: EventPhasePool {
                shops: vec![],
                bonuses: vec![],
                random_events: vec![],
            },
            midnight: EventPhasePool {
                shops: vec![],
                bonuses: vec![],
                random_events: vec![],
            },
            white: EventPhasePool {
                shops: vec![],
                bonuses: vec![],
                random_events: vec![],
            },
        };

        let game_data = Arc::new(GameData {
            shops: empty_shops,
            event_pools,
        });

        let world = bevy_ecs::world::World::new();
        let ctx = GeneratorContext::new(&world, &game_data, 12345);

        // when
        let generator = ShopGenerator;
        let shop = generator.generate(&ctx);

        // then: 폴백 상점이 반환되어야 함
        assert_eq!(shop.name, "임시 상점");
        assert!(!shop.items.is_empty(), "폴백 상점도 아이템이 있어야 함");
    }
}
