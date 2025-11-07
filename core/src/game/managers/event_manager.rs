use crate::game::{
    enums::{OrdealType, PhaseEvent, PhaseEventType, PhaseType},
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
                PhaseEvent::EventSelection(options)
            }
            PhaseEventType::Suppression => {
                let generator = SuppressionGenerator;
                let data = generator.generate(ctx);
                PhaseEvent::Suppression(data)
            }
            PhaseEventType::Ordeal => {
                let generator = OrdealBattleGenerator;
                let battle = generator.generate(ctx);
                PhaseEvent::Ordeal(battle)
            }
        }
    }
}
