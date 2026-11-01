//! Stage-selected shots use the existing projectile and forced-movement consumers.
use super::{equipment_attack::EquipmentAttackCapture, movement::types::WorldVec2, BattleCore};
use crate::{
    authoring::{clone_shape::controlled_clone, control},
    game::{
        ability::weapon_marks::WeaponStage,
        battle::{
            damage::{DamageSource, DamageSourceSnapshot},
            ids::UnitInstanceId,
            runtime_control,
            runtime_facts::{EquipmentSourceSnapshot, RuntimeFactSource},
        },
        behavior::GameError,
        enums::Side,
    },
};
use std::{mem::size_of, sync::Arc};

pub(in crate::game::battle::core) struct WeaponStageHit {
    capture: Arc<EquipmentAttackCapture>,
    source: RuntimeFactSource,
    actor: UnitInstanceId,
    side: Side,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WeaponStageWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub actor: UnitInstanceId,
    pub target: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub stage: WeaponStage,
    pub time_ms: u64,
    pub interval_ms: u64,
    pub windup_ms: u32,
    pub push_distance_units: u32,
    pub actual_hp_loss: u64,
    pub from_data_units: (i64, i64),
    pub to_data_units: (i64, i64),
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
crate::authoring::clone_shape::struct_shape!(WeaponStageWitness {
    generation,
    actor,
    target,
    equipment,
    stage,
    time_ms,
    interval_ms,
    windup_ms,
    push_distance_units,
    actual_hp_loss,
    from_data_units,
    to_data_units,
    definition_id,
    source_file,
    field_path
});
impl WeaponStageWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>()
            .checked_add(self.definition_id.capacity())?
            .checked_add(self.source_file.capacity())?
            .checked_add(self.field_path.capacity())?;
        if let Some(g) = &self.generation {
            bytes = bytes
                .checked_add(std::mem::size_of_val(g.as_ref()))?
                .checked_add(g.semantic_sha256.capacity())?;
        }
        Some(bytes)
    }
}
impl BattleCore {
    pub(in crate::game::battle::core) fn prepare_weapon_stage_hit(
        &self,
        snapshot: &DamageSourceSnapshot,
    ) -> Result<Option<WeaponStageHit>, GameError> {
        let Some(capture) = &snapshot.equipment_attack else {
            return Ok(None);
        };
        if capture.program.weapon_stages.is_none()
            || snapshot.source != DamageSource::BasicAttack
            || snapshot.fact_source.reaction
            || snapshot.fact_source.guard_release().is_some()
            || snapshot
                .fact_source
                .cast
                .as_ref()
                .is_some_and(|c| c.proc_origin.is_some())
        {
            return Ok(None);
        }
        Ok(Some(WeaponStageHit {
            capture: Arc::clone(capture),
            source: controlled_clone(&snapshot.fact_source)?,
            actor: snapshot.source_id,
            side: snapshot.source_side,
        }))
    }

    pub(in crate::game::battle::core) fn resolve_weapon_stage_hit(
        &mut self,
        plan: WeaponStageHit,
        target: UnitInstanceId,
        direction: WorldVec2,
        now: u64,
        summary: super::types::CommandExecutionSummary,
    ) -> Result<(), GameError> {
        if summary.effective_hit_count == 0 {
            return Ok(());
        }
        let stages = plan
            .capture
            .program
            .weapon_stages
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        let stage = plan.capture.weapon_stage;
        let shot = stages.definition.shot(stage);
        let authored = stages.source(stage);
        let Some(unit) = self.units.get(&target) else {
            return Ok(());
        };
        let from = self
            .sample_unit_body_at(target, now)
            .ok_or(GameError::InvalidAction)?
            .position;
        let movable = unit.is_active()
            && unit.is_combatant()
            && !unit.is_airborne()
            && unit.owner != plan.side
            && target != plan.actor
            && shot.push_distance_units > 0;
        if movable {
            let mut source = plan.source;
            source.actor = Some(plan.actor);
            source.equipment = Some(plan.capture.equipment);
            source.authored = Some(Arc::clone(authored));
            let prior = std::mem::replace(&mut self.runtime_fact_source, source);
            let result = self.push_shot_target(
                plan.actor,
                target,
                direction.normalized_or_zero(),
                shot.push_distance_units as f32 / super::movement::types::DATA_UNITS_PER_WORLD,
                now,
            );
            self.runtime_fact_source = prior;
            result?;
        }
        control::poll("weapon_stage_after_hit")?;
        if self.authoring_observation.is_some() {
            let bytes = size_of::<WeaponStageWitness>()
                + authored.site.action.definition_id.len()
                + authored.source_file.len()
                + authored.field_path.len();
            runtime_control::admit_retained_growth(0, Some(bytes), "weapon_stage_witness_copy")?;
            control::charge(bytes as u64, "weapon_stage_witness_copy")?;
            let witness = WeaponStageWitness {
                generation: self.content_identity.clone(),
                actor: plan.actor,
                target,
                equipment: plan.capture.equipment,
                stage,
                time_ms: now,
                interval_ms: shot.interval_ms,
                windup_ms: shot.windup_ms,
                push_distance_units: shot.push_distance_units,
                actual_hp_loss: summary.actual_hp_loss,
                from_data_units: from.to_data_units(),
                to_data_units: self
                    .sample_unit_body_at(target, now)
                    .map_or(from, |u| u.position)
                    .to_data_units(),
                definition_id: authored.site.action.definition_id.to_string(),
                source_file: authored.source_file.clone(),
                field_path: authored.field_path.clone(),
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "weapon_stage_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_weapon_stage = Some(witness);
        }
        Ok(())
    }
}
