//! Trap activation is a new action; the creation cast is observation only.
use crate::game::{
    ability::SkillId,
    battle::{
        damage::{CombatRollIdentity, CombatRollKind, DamageSource, DamageSourceSnapshot},
        ids::UnitInstanceId,
        tile_range::{FacingDirection, TileRangePattern, TileRangePolicy, TileReachQuery},
    },
    behavior::GameError,
    data::independent_object_data::IndependentTrapDef,
};

use super::{
    damage_runtime::{CommittedDamageInput, CommittedDamagePolicy},
    independent_objects::{IndependentObject, ObjectRemovalReason},
    BattleCore,
};

impl BattleCore {
    fn trap_targets(
        &self,
        object: &IndependentObject,
        pattern: &TileRangePattern,
        air_capable: bool,
    ) -> Result<Vec<UnitInstanceId>, GameError> {
        use crate::game::battle::runtime_control;
        Self::charge_authoring_tile_pattern(pattern, self.units.len().saturating_add(1) as u64)?;
        if !crate::authoring::control::is_active() {
            runtime_control::charge(pattern.rows.len() as u64, "trap_pattern_rows")?;
            let cells = pattern
                .rows
                .iter()
                .try_fold(0u64, |n, row| n.checked_add(row.len() as u64 + 1));
            // Membership revalidates the pattern and scans its anchor/cells.
            // Six visits per byte bound those scans, without a second native
            // debit when the shared authoring projection meter already ran.
            let work = cells
                .and_then(|n| n.checked_mul(self.units.len() as u64 + 1))
                .and_then(|n| n.checked_mul(6))
                .ok_or(GameError::InvalidAction)?;
            runtime_control::charge(work, "trap_pattern_scan")?;
        }
        crate::authoring::control::charge(self.units.len() as u64, "trap_target_scan")?;
        runtime_control::admit_retained_growth(
            0,
            self.units
                .len()
                .checked_mul(std::mem::size_of::<UnitInstanceId>()),
            "trap_target_storage",
        )?;
        let query = TileReachQuery::try_new(
            &self.battlefield,
            TileRangePolicy::Pattern,
            Some(pattern),
            Some(object.position),
            Some(FacingDirection::Up),
        )
        .map_err(|error| GameError::InvalidStaticData(format!("trap range: {error:?}")))?;
        let mut targets = Vec::with_capacity(self.units.len());
        for unit in self.units.values() {
            if unit.is_combatant()
                && unit.owner != object.side
                && self.single_target_can_target_unit(unit.instance_id, air_capable)
                && query.contains(unit.body.projected_tile())
            {
                targets.push(unit.instance_id);
            }
        }
        let sort_work = (targets.len() as u64)
            .saturating_mul(u64::from(usize::BITS - targets.len().leading_zeros()) + 1);
        crate::authoring::control::charge(sort_work, "trap_target_sort")?;
        targets.sort_unstable();
        Ok(targets)
    }

    pub(super) fn trigger_independent_trap(
        &mut self,
        id: UnitInstanceId,
        trap: &IndependentTrapDef,
        now_ms: u64,
    ) -> Result<(), GameError> {
        let Some(object) = self.independent_objects.objects.get(&id) else {
            return Ok(());
        };
        if self
            .trap_targets(object, &trap.trigger_range, trap.air_capable)?
            .is_empty()
        {
            return Ok(());
        }
        let mut targets = self.trap_targets(object, &trap.damage_range, trap.air_capable)?;
        if let Some(max_targets) = trap.max_targets {
            targets.truncate(max_targets as usize);
        }
        if targets.is_empty() {
            return Ok(());
        }
        let object = crate::authoring::clone_shape::controlled_clone(object)?;
        let base_damage = trap_base_damage(trap, object.source_attack)?;
        let mut source = crate::game::battle::runtime_facts::RuntimeFactSource {
            actor: Some(id),
            extended_observation:
                crate::game::battle::runtime_facts::RuntimeFactSource::extended_observation(
                    self.independent_action_origin(id)?,
                    None,
                )?,
            ..Default::default()
        };
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(32),
            "trap_intrinsic_id",
        )?;
        let intrinsic = SkillId::from("$independent_trap");
        source.admit_owner("trap_action_source")?;
        let action =
            self.runtime_facts
                .accept(id, Some(&intrinsic), source.clone(), now_ms, false)?;
        source.action = Some(action.clone());
        self.runtime_facts.release(&action, 1, now_ms)?;
        let mut attacked = false;
        for (index, target) in targets.into_iter().enumerate() {
            source.admit_owner("trap_damage_source")?;
            let snapshot = DamageSourceSnapshot {
                fact_source: source.clone(),
                skill_cast_source: source.cast.clone(),
                value_expr_capture: None,
                source_id: id,
                source_side: object.side,
                source_attack: object.source_attack,
                source: DamageSource::Ability,
                damage_type: trap.damage_type,
                base_damage,
                modifiers: Default::default(),
                crit_roll_percent: Some(99),
                crit_roll_identity: CombatRollIdentity::new(
                    CombatRollKind::SkillCrit,
                    Some(id),
                    id.0,
                )
                .with_target(target)
                .with_hit_index(index as u32),
                minimum_damage: 0,
                committed_at_ms: now_ms,
                basic_lifesteal: vec![],
                on_attack_modifiers: Default::default(),
                on_attack_bonus_damage: vec![],
                on_attack_modifier_sources: vec![],
            };
            attacked |= self
                .execute_committed_damage(CommittedDamageInput {
                    target_id: target,
                    source_snapshot: snapshot,
                    committed_at_ms: now_ms,
                    policy: CommittedDamagePolicy::GeneralCommand,
                })
                .is_some();
            crate::game::battle::runtime_control::poll()?;
            crate::authoring::control::poll("trap_damage_complete")?;
        }
        self.runtime_facts.dispatched(&action, 0, now_ms)?;
        if attacked {
            self.retire_independent_object(
                id,
                object.epoch,
                now_ms,
                ObjectRemovalReason::AttackConsumed,
            )?;
        }
        Ok(())
    }
}

/// Shared by read-only eligibility, creation release, and trap execution.
pub(super) fn trap_base_damage(
    trap: &IndependentTrapDef,
    source_attack: u32,
) -> Result<u32, GameError> {
    u64::from(source_attack)
        .checked_mul(u64::from(trap.attack_percent))
        .and_then(|n| (n / 100).checked_add(u64::from(trap.damage_flat)))
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| GameError::InvalidStaticData("trap damage exceeds u32".into()))
}
