use rand::{Rng, SeedableRng};
use tracing::info;

use crate::{
    config::balance,
    ecs::resources::{Qliphoth, QliphothLevel},
    game::{
        enums::{
            GameOption, OrdealOption, OrdealType, PhaseEvent, PhaseEventType, PhaseType,
            SuppressionOption,
        },
        events::{
            event_selection::EventSelectionGenerator, ordeal_battle::OrdealBattleGenerator,
            suppression::SuppressionGenerator, EventGenerator, GeneratorContext,
        },
        managers::ordeal_scheduler::OrdealScheduler,
    },
};

pub struct EventManager;

impl EventManager {
    /// Qliphoth 레벨에 따라 이벤트 타입 결정
    ///
    /// # 규칙
    /// - Stable (10~7): 원래 스케줄대로 (EventSelection)
    /// - Caution (6~4): 가중치 기반 Suppress 또는 EventSelection
    /// - Critical (3~1): 강제 Suppress (Breach)
    /// - Meltdown (0): 특수 이벤트 (TODO: 백야)
    fn determine_event_type(
        qliphoth: &Qliphoth,
        ordeal: OrdealType,
        phase: PhaseType,
        ctx: &GeneratorContext,
    ) -> PhaseEventType {
        let mut rng = rand::rngs::StdRng::seed_from_u64(ctx.random_seed);
        match qliphoth.level() {
            QliphothLevel::Stable => {
                // 안정 상태: 원래 스케줄 따름
                let schedules = OrdealScheduler::get_phase_schedule(ordeal);
                schedules
                    .iter()
                    .find(|s| s.phase == phase)
                    .map(|s| s.event_type)
                    .unwrap_or(PhaseEventType::EventSelection)
            }

            QliphothLevel::Caution => {
                // 주의 상태: 진압 작업 발생 확률 체크
                let suppress_chance = balance::qliphoth_suppress_chance();
                // 0~99 범위 난수 생성 (총 100개 값)
                let roll = rng.gen_range(0..100);

                // roll < suppress_chance 이면 진압 발생
                // 예: suppress_chance = 50 → roll이 0~49일 때 true (50%)
                if roll < suppress_chance.caution {
                    info!(
                        "Caution state triggered Suppress event (roll={}/100, threshold={}%)",
                        roll, suppress_chance.caution
                    );
                    PhaseEventType::Suppression
                } else {
                    // 원래 스케줄 따름 (EventSelection)
                    let schedules = OrdealScheduler::get_phase_schedule(ordeal);
                    schedules
                        .iter()
                        .find(|s| s.phase == phase)
                        .map(|s| s.event_type)
                        .unwrap_or(PhaseEventType::EventSelection)
                }
            }

            QliphothLevel::Critical => {
                // 위험 상태: 강제 Breach (Suppression)
                info!("Critical state: forcing Breach event");
                PhaseEventType::Suppression
            }

            QliphothLevel::Meltdown => {
                // 붕괴 상태: 특수 이벤트
                // TODO: 백야 이벤트 구현
                info!("Meltdown state: triggering special event");
                PhaseEventType::Suppression // 임시로 Suppression
            }
        }
    }

    pub fn generate_event(
        qliphoth: Qliphoth,
        ordeal: OrdealType,
        phase: PhaseType,
        ctx: &GeneratorContext,
    ) -> PhaseEvent {
        // 1. Qliphoth 레벨에 따라 이벤트 타입 결정
        let event_type = Self::determine_event_type(&qliphoth, ordeal, phase, ctx);

        info!(
            "Generating event for ordeal={:?}, phase={:?}, qliphoth={:?} (amount={}), event_type={:?}",
            ordeal, phase, qliphoth.level(), qliphoth.amount(), event_type
        );

        match event_type {
            PhaseEventType::EventSelection => {
                let generator = EventSelectionGenerator;
                let options = generator.generate(ctx);
                let [shop, bonus, random] = options;

                let shop = match shop {
                    GameOption::Shop { shop } => shop,
                    _ => unreachable!("EventSelection generator must return a Shop option"),
                };
                let bonus = match bonus {
                    GameOption::Bonus { bonus } => bonus,
                    _ => unreachable!("EventSelection generator must return a Bonus option"),
                };
                let random = match random {
                    GameOption::Random { event } => event,
                    _ => unreachable!("EventSelection generator must return a Random option"),
                };

                PhaseEvent::EventSelection {
                    shop,
                    bonus,
                    random,
                }
            }
            PhaseEventType::Suppression => {
                let generator = SuppressionGenerator;
                let options = generator.generate(ctx);
                let candidates = options.map(|option| match option {
                    GameOption::SuppressAbnormality {
                        abnormality_id,
                        risk_level,
                        uuid,
                    } => SuppressionOption {
                        abnormality_id,
                        risk_level,
                        uuid,
                    },
                    _ => unreachable!("Suppression generator must return suppression options"),
                });

                PhaseEvent::Suppression { candidates }
            }
            PhaseEventType::Ordeal => {
                let generator = OrdealBattleGenerator;
                let options = generator.generate(ctx);
                let candidates = options.map(|option| match option {
                    GameOption::OrdealBattle {
                        ordeal_type,
                        difficulty,
                        uuid,
                    } => OrdealOption {
                        ordeal_type,
                        difficulty,
                        uuid,
                    },
                    _ => unreachable!("Ordeal generator must return ordeal options"),
                });

                PhaseEvent::Ordeal { candidates }
            }
        }
    }
}
