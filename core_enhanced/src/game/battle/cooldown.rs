use crate::game::ability::AbilityActivationBinding;
use crate::game::battle::ids::UnitInstanceId;
use crate::game::stats::{Effect, TriggerEffectTarget};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CooldownSource {
    Unit { unit_instance_id: UnitInstanceId },
    Item { item_instance_id: Uuid },
}

#[derive(Debug, Clone)]
pub struct SourcedEffect {
    pub source: CooldownSource,
    pub equipment_source: Option<super::runtime_facts::EquipmentSourceSnapshot>,
    pub target: TriggerEffectTarget,
    pub effect: Effect,
}

#[derive(Debug, Clone)]
pub struct SourcedAbilityActivation {
    pub source: CooldownSource,
    pub equipment_source: Option<super::runtime_facts::EquipmentSourceSnapshot>,
    pub binding: AbilityActivationBinding,
    pub binding_index: usize,
}
