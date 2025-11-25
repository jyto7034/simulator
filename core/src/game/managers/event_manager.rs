use crate::game::{
    enums::{OrdealOption, OrdealType, PhaseEvent, PhaseEventType, PhaseType, SuppressionOption},
    events::{
        event_selection::EventSelectionGenerator, ordeal_battle::OrdealBattleGenerator,
        suppression::SuppressionGenerator, EventGenerator, GeneratorContext,
    },
    managers::ordeal_scheduler::OrdealScheduler,
};

pub struct EventManager;

impl EventManager {
    pub fn generate_event(
        ordeal: OrdealType,
        phase: PhaseType,
        ctx: &GeneratorContext,
    ) -> PhaseEvent {
        let schedules = OrdealScheduler::get_phase_schedule(ordeal);

        let phase_schedule = schedules
            .iter()
            .find(|phase_schedule| phase == phase_schedule.phase)
            .unwrap();

        match phase_schedule.event_type {
            PhaseEventType::EventSelection => {
                let generator = EventSelectionGenerator;
                let options = generator.generate(ctx);
                let [shop, bonus, random] = options;

                let shop = match shop {
                    crate::game::enums::GameOption::Shop { shop } => shop,
                    _ => unreachable!("EventSelection generator must return a Shop option"),
                };
                let bonus = match bonus {
                    crate::game::enums::GameOption::Bonus { bonus } => bonus,
                    _ => unreachable!("EventSelection generator must return a Bonus option"),
                };
                let random = match random {
                    crate::game::enums::GameOption::Random { event } => event,
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
                    crate::game::enums::GameOption::SuppressAbnormality {
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
                    crate::game::enums::GameOption::OrdealBattle {
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
