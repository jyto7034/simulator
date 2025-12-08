use bevy_ecs::component::Component;

use crate::game::enums::{Lane, RiskLevel, Side};

#[derive(Clone)]
struct AbnormalityStats {
    max_hp: u32,
    attack: u32,
    attack_interval_ms: u64,
    // 나중을 위한 방어/속도/특수 스탯 등
}

#[derive(Clone)]
struct AbnormalityState {
    hp: i32,
    next_attack_time_ms: u64,
    // buffs: Vec<BuffInstance>,
    // debuffs: Vec<DebuffInstance>,
}

#[derive(Component, Clone)]
pub struct Abnormality {
    id: String,
    owner: Side,
    lane: Lane,
    stats: AbnormalityStats,
    state: AbnormalityState,
    risk: RiskLevel,
}

impl Abnormality {}
