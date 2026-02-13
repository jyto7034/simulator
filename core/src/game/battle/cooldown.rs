use crate::game::battle::ids::UnitInstanceId;
use crate::game::stats::Effect;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CooldownSource {
    Unit { unit_instance_id: UnitInstanceId },
    Item { item_instance_id: Uuid },
    Artifact { artifact_instance_id: Uuid },
}

#[derive(Debug, Clone)]
pub struct SourcedEffect {
    pub source: CooldownSource,
    pub effect: Effect,
}
