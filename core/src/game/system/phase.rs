use bevy_ecs::world::World;

use crate::game::{
    enums::{PhaseEventType, PhaseSchedule},
    ordeal_scheduler::OrdealScheduler,
    resources::CurrentOrdeal,
};

pub struct PhaseResolver {
    pub current_phase_index: usize,
    pub phase_schedule: Vec<PhaseSchedule>,
}

impl PhaseResolver {
    pub fn new(ordeal: &CurrentOrdeal) -> Self {
        let phase_schedule = OrdealScheduler::get_phase_schedule(ordeal.ordeal_type);
        Self {
            current_phase_index: 0,
            phase_schedule,
        }
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

    pub fn execute_current_phase(&self, world: &mut World) {
        if let Some(phase) = self.current_phase() {
            match phase.event_type {
                PhaseEventType::EventSelection => {
                    self.generate_random_events(world);
                }
                PhaseEventType::Suppression => {
                    self.generate_suppression_choices(world);
                }
                PhaseEventType::Ordeal => {
                    self.start_ordeal_battle(world);
                }
            }
        }
    }

    // === 런타임 이벤트 생성 ===

    fn generate_random_events(&self, _world: &mut World) {
        // TODO: 상점, 골드, 환상체, 퀘스트 등 랜덤 3개 생성
        // 예: [상점, 골드 획득, 환상체 획득]
        // 플레이어가 1개 선택
    }

    fn generate_suppression_choices(&self, _world: &mut World) {
        // TODO: 3개 몬스터 랜덤 생성
        // 예: [ZAYIN 몬스터, TETH 몬스터, HE 몬스터]
        // 플레이어가 1개 선택 → 전투
    }

    fn start_ordeal_battle(&self, _world: &mut World) {
        // TODO: Ghost 매칭 + 전투 시작
        // 시련 색상 선택 (3~4가지)
    }
}
