//! Latest persistent-area mutation, without retaining runtime capabilities or history.
use super::{authoring_observation::MandatoryObservation, types::AreaRuntime};
use crate::{
    authoring::control,
    game::{
        ability::{sustained::SustainedEndReason, SkillId},
        battle::ids::UnitInstanceId,
        behavior::GameError,
    },
};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AreaMembership {
    pub members: usize,
    pub contributions: usize,
}
crate::authoring::clone_shape::copy_shape!(AreaMembership);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AreaChange {
    Started,
    Reconciled,
    Ended { reason: SustainedEndReason },
}
crate::authoring::clone_shape::copy_shape!(AreaChange);

#[derive(Debug, Clone, Serialize)]
pub struct AreaWitness {
    #[serde(skip)]
    pub(crate) generation: Option<std::sync::Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub ordinal: u64,
    pub time_ms: u64,
    pub area_id: Uuid,
    pub skill_id: SkillId,
    pub step_id: String,
    pub caster: UnitInstanceId,
    pub cast_sequence: u64,
    pub started_at_ms: u64,
    pub expires_at_ms: u64,
    pub change: AreaChange,
    pub before: Option<AreaMembership>,
    pub after: Option<AreaMembership>,
}
crate::authoring::clone_shape::struct_shape!(AreaWitness {
    generation,
    ordinal,
    time_ms,
    area_id,
    skill_id,
    step_id,
    caster,
    cast_sequence,
    started_at_ms,
    expires_at_ms,
    change,
    before,
    after
});
impl AreaWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let source_bytes = self.generation.as_ref().map_or(0, |g| {
            std::mem::size_of_val(g.as_ref()) + g.semantic_sha256.capacity()
        });
        std::mem::size_of::<Self>()
            .checked_add(source_bytes)?
            .checked_add(self.skill_id.as_str().len())?
            .checked_add(self.step_id.capacity())
    }
}

pub(super) fn record(
    owner: &mut Option<MandatoryObservation>,
    runtime: &AreaRuntime,
    now: u64,
    change: AreaChange,
    before: Option<AreaMembership>,
) -> Result<(), GameError> {
    let Some(owner) = owner else {
        return Ok(());
    };
    let generation = runtime.source_generation.as_ref();
    let source_bytes = generation.map_or(0, |g| {
        std::mem::size_of_val(g.as_ref()) + g.semantic_sha256.capacity()
    });
    let bytes = std::mem::size_of::<AreaWitness>()
        .checked_add(source_bytes)
        .and_then(|n| n.checked_add(runtime.skill_id.as_str().len()))
        .and_then(|n| n.checked_add(runtime.step_id.len()));
    crate::game::battle::runtime_control::admit_retained_growth(0, bytes, "area_observation_copy")?;
    control::charge(bytes.unwrap_or(usize::MAX) as u64, "area_observation_copy")?;
    let ordinal = owner
        .area_changes
        .checked_add(1)
        .ok_or_else(|| control::reject("area_observation_counter", "area_observation"))?;
    let after = if matches!(change, AreaChange::Ended { .. }) {
        None
    } else {
        runtime.ground_status.as_ref().map(|g| g.membership())
    };
    let witness = AreaWitness {
        generation: generation.cloned(),
        ordinal,
        time_ms: now,
        area_id: runtime.area_id,
        skill_id: runtime.skill_id.clone(),
        step_id: runtime.step_id.clone(),
        caster: runtime.caster_instance_id,
        cast_sequence: runtime.cast_seq,
        started_at_ms: runtime.spawned_at_ms,
        expires_at_ms: runtime.expires_at_ms,
        change,
        before,
        after,
    };
    owner.area_changes = ordinal;
    owner.latest_area = Some(witness);
    control::poll("area_observation_committed")
}

/// Immutable attribution for a finite effect created by a ground declaration.
/// This is a Delivery producer, never a synthetic authored EffectSite.
#[derive(Debug, Clone, Serialize)]
pub struct AreaEffectSource {
    #[serde(skip)]
    pub(crate) generation: Option<std::sync::Arc<crate::game::data::content_pack::ContentIdentity>>,
    /// Creation occurrence only; carries no action or resource capability.
    /// Keep it after the area ends so finite children cannot borrow a later cast.
    #[serde(skip)]
    pub(crate) captured_cast: Option<std::sync::Arc<crate::game::battle::skill_source::SkillCastExecutionSource>>,
    pub area_id: Uuid,
    pub skill_id: SkillId,
    pub step_id: String,
    pub cast_sequence: u64,
    pub status_index: usize,
    pub definition: String,
    pub modifier_source: Option<crate::game::ability::sustained::HitModifierSource>,
}
crate::authoring::clone_shape::struct_shape!(AreaEffectSource {
    generation,
    captured_cast,
    area_id,
    skill_id,
    step_id,
    cast_sequence,
    status_index,
    definition,
    modifier_source
});
impl AreaEffectSource {
    pub(crate) fn matches_cast(
        &self,
        candidate: Option<&std::sync::Arc<crate::game::battle::skill_source::SkillCastExecutionSource>>,
    ) -> bool {
        self.captured_cast.as_ref().is_some_and(|cast| {
            Some(cast) == candidate
                && cast.skill_id == self.skill_id
                && cast.cast_seq == self.cast_sequence
        })
    }

    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let generation = self.generation.as_ref().map_or(0, |g| {
            std::mem::size_of_val(g.as_ref()) + g.semantic_sha256.capacity()
        });
        let modifier = self
            .modifier_source
            .as_ref()
            .map_or(0, |m| m.definition.capacity() + m.rule.capacity());
        let cast = self.captured_cast.as_ref().map_or(Some(0), |cast| {
            std::mem::size_of_val(cast.as_ref()).checked_add(cast.skill_id.as_str().len())
        })?;
        std::mem::size_of::<Self>()
            .checked_add(generation)?
            .checked_add(cast)?
            .checked_add(self.skill_id.as_str().len())?
            .checked_add(self.step_id.capacity())?
            .checked_add(self.definition.capacity())?
            .checked_add(modifier)
    }
}
