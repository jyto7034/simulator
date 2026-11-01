//! A copy's one battle use outlives deployment; the blast and exit are atomic.
use super::*;
use crate::game::{
    battle::{
        damage::{DamageModifiers, DamageSource, DamageSourceSnapshot},
        runtime_facts::{ActionCancelReason, ProtectionReceipt},
        tile_range::FacingDirection,
    },
    resources::Position,
};

pub(in crate::game::battle::core) struct LastWillPlan {
    equipment: EquipmentSourceSnapshot,
    program: Arc<CompiledEquipmentDefense>,
    snapshot: DamageSourceSnapshot,
    center: Position,
    facing: FacingDirection,
    hp_before: u32,
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct LastWillWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub actor: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub time_ms: u64,
    pub center: Position,
    pub facing: FacingDirection,
    pub hp_before: u32,
    pub hp_protected: u32,
    pub hp_after: u32,
    pub attack: u32,
    pub base_damage: u32,
    pub targets: u32,
    pub incapacitated: bool,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
crate::authoring::clone_shape::struct_shape!(LastWillWitness {
    generation,
    actor,
    equipment,
    time_ms,
    center,
    facing,
    hp_before,
    hp_protected,
    hp_after,
    attack,
    base_damage,
    targets,
    incapacitated,
    definition_id,
    source_file,
    field_path
});
impl LastWillWitness {
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
    pub(in crate::game::battle::core) fn prepare_last_will(
        &mut self,
        actor: UnitInstanceId,
        pending: u32,
        now: u64,
        incoming: &RuntimeFactSource,
        receipt: &mut ProtectionReceipt,
    ) -> Result<Option<LastWillPlan>, GameError> {
        if self.sustained.closed || !self.game_data.equipment_defense_catalog().has_last_will {
            return Ok(None);
        }
        let Some(unit) = self
            .units
            .get(&actor)
            .filter(|u| u.is_active() && !u.corroded && Self::is_employee_resonance_unit(u))
        else {
            return Ok(None);
        };
        let hp = unit.stats.current_health;
        if hp == 0 || pending < hp {
            return Ok(None);
        }
        if self.equipment_defense.last_will_resolving.contains(&actor) {
            receipt.prevented = receipt.prevented.saturating_add(pending - (hp - 1));
            return Ok(None);
        }
        let attack = unit.stats.attack;
        let facing = unit.facing_direction.unwrap_or(FacingDirection::Up);
        let center = self
            .sample_unit_body_at(actor, now)
            .ok_or(GameError::InvalidAction)?
            .projected_tile();
        let mut selected = None;
        self.visit_equipment_defenses(actor, |equipment, program| {
            if let Some(copy) = equipment.persistent_copy_uuid {
                if program.last_will_source.is_some()
                    && !self.equipment_defense.last_will_used.contains(&copy)
                    && selected.is_none()
                {
                    selected = Some((equipment, Arc::clone(program)));
                }
            }
        })?;
        let Some((equipment, program)) = selected else {
            return Ok(None);
        };
        let EquipmentDefenseProgram::LastWill {
            attack_percent,
            damage_type,
            ..
        } = &program.program
        else {
            unreachable!()
        };
        let damage = u32::try_from(u64::from(attack) * u64::from(*attack_percent) / 100)
            .map_err(|_| runtime_control::invariant("last_will_damage_overflow"))?;
        let fact = RuntimeFactSource {
            actor: Some(actor),
            equipment: Some(equipment),
            reaction: true,
            authored: program.last_will_source.clone(),
            ..Default::default()
        }
        .inherit_causal_context(incoming)?;
        let old = std::mem::replace(&mut self.runtime_fact_source, fact);
        let snapshot = self.damage_source_snapshot_template_for_unit(
            actor,
            DamageSource::Ability,
            *damage_type,
            damage,
            DamageModifiers::default(),
            0,
            now,
            false,
        );
        self.runtime_fact_source = old;
        let mut snapshot = snapshot.ok_or(GameError::InvalidAction)?;
        snapshot.crit_roll_percent = Some(100);
        runtime_control::admit_retained_growth(
            0,
            snapshot
                .fact_retained_extent()
                .and_then(|n| n.checked_add(512)),
            "last_will_prepare",
        )?;
        self.equipment_defense
            .last_will_used
            .insert(equipment.persistent_copy_uuid.unwrap());
        self.equipment_defense.last_will_resolving.insert(actor);
        receipt.prevented = receipt.prevented.saturating_add(pending - (hp - 1));
        control::poll("last_will_reserved")?;
        Ok(Some(LastWillPlan {
            equipment,
            program,
            snapshot,
            center,
            facing,
            hp_before: hp,
        }))
    }

    pub(in crate::game::battle::core) fn resolve_last_will(
        &mut self,
        plan: LastWillPlan,
        incoming: &DamageSourceSnapshot,
        now: u64,
    ) -> Result<(), GameError> {
        let actor = plan.snapshot.source_id;
        self.runtime_facts
            .cancel_unreleased_basic(actor, ActionCancelReason::CasterDied, now)?;
        self.cancel_structured_flows(actor, now, ActionCancelReason::CasterDied);
        self.cancel_resonance_use_control(actor);
        let pending = {
            let unit = self.units.get_mut(&actor).ok_or(GameError::InvalidAction)?;
            unit.basic_attack_epoch = unit.basic_attack_epoch.wrapping_add(1);
            unit.pending_basic_attack = false;
            unit.basic_attack_windup_until_ms = None;
            unit.pending_cast = false;
            unit.pending_cast_cause = None;
            unit.pending_skill_cast.take()
        };
        if let Some(pending) = pending {
            self.cancel_pending_action(
                pending.action.as_ref(),
                now,
                ActionCancelReason::CasterDied,
            );
        }
        control::poll("last_will_before_blast")?;
        let EquipmentDefenseProgram::LastWill { range, .. } = &plan.program.program else {
            unreachable!()
        };
        let tiles = range
            .affected_tiles(plan.center, plan.facing)
            .map_err(GameError::InvalidStaticData)?;
        let targets = self.collect_tile_area_targets_at(
            now,
            plan.snapshot.source_side,
            actor,
            &tiles,
            crate::game::ability::SkillHitTargetFilter::Enemies,
            false,
        );
        let area_percent = self.area_damage_percent(actor, plan.snapshot.source_side,
            &plan.snapshot.fact_source, &targets, now)?;
        control::poll("last_will_targets")?;
        let old = std::mem::replace(
            &mut self.runtime_fact_source,
            plan.snapshot.fact_source.clone(),
        );
        let prior_observation = self.authoring_direct_source.take();
        let prior_effect = self.recording_skill_effect_source.take();
        let scope = self.begin_strike_scope();
        let result = (|| {
            for &target in &targets {
                if !self.units.get(&target).is_some_and(|u| u.is_active()) {
                    continue;
                }
                runtime_control::admit_retained_growth(
                    0,
                    plan.snapshot.fact_retained_extent(),
                    "last_will_hit_copy",
                )?;
                let mut snapshot = self.materialize_damage_source_snapshot_for_target_hit(
                    plan.snapshot.clone(),
                    target,
                    0,
                );
                snapshot.modifiers.damage_amp_percent = snapshot.modifiers.damage_amp_percent.saturating_add(area_percent);
                self.execute_committed_damage(super::super::damage_runtime::CommittedDamageInput {
                    target_id: target,
                    source_snapshot: snapshot,
                    committed_at_ms: now,
                    policy: super::super::damage_runtime::CommittedDamagePolicy::GeneralCommand,
                })
                .ok_or(GameError::InvalidAction)?;
                control::poll("last_will_hit")?;
            }
            Ok::<_, GameError>(())
        })();
        self.end_strike_scope(scope);
        self.authoring_direct_source = prior_observation;
        self.recording_skill_effect_source = prior_effect;
        self.runtime_fact_source = old;
        result?;
        // This is an ordinary incapacitation, not another damage hit: no second
        // shield/prevention pass, and healing during the blast cannot cancel it.
        let old = std::mem::replace(&mut self.runtime_fact_source, incoming.fact_source.clone());
        self.finalize_unit_death(
            Some(incoming.source_id),
            actor,
            plan.snapshot.source_side,
            now,
        );
        self.runtime_fact_source = old;
        control::poll("last_will_after_exit")?;
        self.equipment_defense.last_will_resolving.remove(&actor);
        if self.authoring_observation.is_some() {
            let bytes = size_of::<LastWillWitness>()
                + plan.program.definition_id.len()
                + plan.program.source_file.len()
                + plan.program.field_path.len();
            runtime_control::admit_retained_growth(0, Some(bytes), "last_will_witness_copy")?;
            control::charge(bytes as u64, "last_will_witness_copy")?;
            let witness = LastWillWitness {
                generation: self.content_identity.clone(),
                actor,
                equipment: plan.equipment,
                time_ms: now,
                center: plan.center,
                facing: plan.facing,
                hp_before: plan.hp_before,
                hp_protected: 1,
                hp_after: self.units[&actor].stats.current_health,
                attack: plan.snapshot.source_attack,
                base_damage: plan.snapshot.base_damage,
                targets: targets.len() as u32,
                incapacitated: self.units[&actor].lifecycle.is_dead(),
                definition_id: plan.program.definition_id.clone(),
                source_file: plan.program.source_file.clone(),
                field_path: plan.program.field_path.clone(),
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "last_will_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_last_will = Some(witness);
        }
        Ok(())
    }
}
