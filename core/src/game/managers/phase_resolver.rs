use crate::{
    ecs::resources::GameProgression,
    game::{enums::{OrdealType, PhaseSchedule}, managers::ordeal_scheduler::OrdealScheduler},
};

pub struct PhaseResolver {
    pub current_phase_index: usize,
    pub phase_schedule: Vec<PhaseSchedule>,
}

impl PhaseResolver {
    pub fn new(ordeal_type: OrdealType) -> Self {
        let phase_schedule = OrdealScheduler::get_phase_schedule(ordeal_type);
        Self {
            current_phase_index: 0,
            phase_schedule,
        }
    }

    pub fn from_progression(progression: &GameProgression) -> Self {
        Self::new(progression.current_ordeal)
    }

    pub fn current_phase(&self) -> Option<&PhaseSchedule> {
        self.phase_schedule.get(self.current_phase_index)
    }

    pub fn advance_phase(&mut self) -> bool {
        if self.current_phase_index + 1 < self.phase_schedule.len() {
            self.current_phase_index += 1;
            true
        } else {
            false
        }
    }
}
