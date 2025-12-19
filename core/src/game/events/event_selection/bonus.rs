use bevy_ecs::world::World;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{
    ecs::resources::Enkephalin,
    game::{
        behavior::GameError,
        data::{
            bonus_data::{BonusMetadata, BonusType},
            event_pools::EventPhasePool,
        },
        enums::GameOption,
        events::{EventGenerator, GeneratorContext},
    },
};

pub struct BonusGenerator;

impl EventGenerator for BonusGenerator {
    type Output = GameOption;

    fn generate(&self, ctx: &GeneratorContext) -> Self::Output {
        use crate::ecs::resources::GameProgression;
        use crate::game::enums::OrdealType;
        use rand::SeedableRng;

        // 1. 현재 Ordeal 가져오기
        let current_ordeal = ctx
            .world
            .get_resource::<GameProgression>()
            .map(|p| p.current_ordeal)
            .unwrap_or(OrdealType::Dawn);

        // 2. Bonus pool 가져오기
        let pool = &ctx.game_data.event_pools.get_pool(current_ordeal).bonuses;

        // 3. RNG 생성
        let mut rng = rand::rngs::StdRng::seed_from_u64(ctx.random_seed);

        // 4. pool에서 가중치 기반 UUID 선택
        let uuid = match EventPhasePool::choose_weighted_uuid(pool, &mut rng) {
            Some(uuid) => uuid,
            None => {
                // 폴백: pool이 비어있으면 기본 보너스 반환
                warn!(
                    "Bonus pool is empty for ordeal={:?}, using fallback bonus",
                    current_ordeal
                );
                return GameOption::Bonus {
                    bonus: BonusMetadata {
                        bonus_type: BonusType::Enkephalin,
                        uuid: Uuid::nil(),
                        name: "임시 보너스".to_string(),
                        description: "폴백 보너스".to_string(),
                        icon: "default".to_string(),
                        amount: 0,
                        id: String::new(),
                    },
                };
            }
        };

        // 5. GameData에서 Bonus 조회
        let bonus = match ctx.game_data.bonus_data.get_by_uuid(&uuid) {
            Some(bonus) => bonus.clone(), // BonusMetadata 전체를 clone
            None => {
                // 폴백: UUID에 해당하는 Bonus가 없으면 기본값
                warn!(
                    "Bonus uuid {:?} not found in GameData, using fallback",
                    uuid
                );
                BonusMetadata {
                    bonus_type: BonusType::Enkephalin,
                    uuid,
                    name: "알 수 없는 보너스".to_string(),
                    description: "설명 없음".to_string(),
                    icon: "unknown".to_string(),
                    amount: 0,
                    id: String::new(),
                }
            }
        };

        debug!(
            "Generated bonus event: id={}, uuid={}",
            bonus.id, bonus.uuid
        );

        // 6. GameOption 생성 (BonusMetadata 전체 데이터 포함)
        GameOption::Bonus { bonus }
    }
}

/// 보너스 비즈니스 로직 헬퍼
pub struct BonusExecutor;

impl BonusExecutor {
    /// 보너스 지급
    ///
    /// # Arguments
    /// * `world` - ECS World
    /// * `bonus` - 보너스 메타데이터
    pub fn grant_bonus(world: &mut World, bonus: &BonusMetadata) -> Result<(), GameError> {
        let amount = bonus.amount;
        match bonus.bonus_type {
            BonusType::Enkephalin => {
                // Enkephalin 추가
                let mut enkephalin = world
                    .get_resource_mut::<Enkephalin>()
                    .ok_or(GameError::MissingResource("Enkephalin"))?;
                enkephalin.amount += amount;

                info!(
                    "Granted Enkephalin bonus: amount={}, new_total={}",
                    amount, enkephalin.amount
                );
            }
            BonusType::Experience => {
                // TODO: 경험치 추가
                // let mut player_stats = world.get_resource_mut::<PlayerStats>()?;
                // player_stats.exp += amount;
            }
            BonusType::Item => {
                // TODO: 랜덤 아이템 추가
                // let mut inventory = world.get_resource_mut::<Inventory>()?;
                // let random_item = generate_random_item(amount); // 티어 기반
                // inventory.add_item(random_item);
            }
            BonusType::Abnormality => {
                // TODO: 새로운 기물 추가
                // let abnormality = spawn_random_abnormality(world, amount);
            }
        }

        Ok(())
    }
}
