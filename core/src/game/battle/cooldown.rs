use uuid::Uuid;

use crate::game::stats::Effect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CooldownSource {
    Unit { unit_instance_id: Uuid },
    Item { item_instance_id: Uuid },
    Artifact { artifact_instance_id: Uuid },
}

#[derive(Debug, Clone)]
pub struct SourcedEffect {
    pub source: CooldownSource,
    pub effect: Effect,
}
