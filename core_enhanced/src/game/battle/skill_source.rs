//! Battle-local authoring provenance. These records are not wire or save data.
use std::sync::Arc;

use crate::game::ability::{authoring::EffectSource, SkillId};

use super::{cooldown::CooldownSource, event_log::BattleEventCause, ids::UnitInstanceId};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SkillProcOrigin {
    #[serde(skip)]
    pub event_seq: u64,
    pub activation_source: CooldownSource,
    pub binding_index: usize,
}

crate::authoring::clone_shape::struct_shape!(SkillProcOrigin {
    event_seq,
    activation_source,
    binding_index
});

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SkillCastExecutionSource {
    pub skill_id: SkillId,
    pub caster_instance_id: UnitInstanceId,
    pub cast_seq: u64,
    #[serde(skip)]
    pub ability_event_seq: u64,
    #[serde(skip)]
    pub cause: BattleEventCause,
    pub proc_origin: Option<SkillProcOrigin>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillEffectExecutionSource {
    pub authored: Arc<EffectSource>,
    pub cast: Arc<SkillCastExecutionSource>,
}
