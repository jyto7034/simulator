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

/// Fixed at active-use acceptance, before focus or its first damage.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CastDamageContribution {
    pub equipment: super::runtime_facts::EquipmentSourceSnapshot,
    pub percent: u32,
    pub authored: Arc<EffectSource>,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CastDamageSnapshot {
    pub actor: UnitInstanceId,
    pub percent: u32,
    pub contributors: Vec<CastDamageContribution>,
}
crate::authoring::clone_shape::struct_shape!(CastDamageContribution {
    equipment,
    percent,
    authored
});
crate::authoring::clone_shape::struct_shape!(CastDamageSnapshot {
    actor,
    percent,
    contributors
});
impl CastDamageSnapshot {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = std::mem::size_of::<Self>().checked_add(64)?.checked_add(
            self.contributors
                .capacity()
                .checked_mul(std::mem::size_of::<CastDamageContribution>())?,
        )?;
        for c in &self.contributors {
            let source = super::runtime_facts::RuntimeFactSource {
                authored: Some(c.authored.clone()),
                ..Default::default()
            };
            bytes = bytes.checked_add(source.retained_extent()?)?;
        }
        Some(bytes)
    }
}
