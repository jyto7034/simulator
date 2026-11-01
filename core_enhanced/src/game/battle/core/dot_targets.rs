//! Read-only membership from authoritative DOT stores. No persistent mirror set.
use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        battle::{
            buffs::BuffKind,
            ids::UnitInstanceId,
            runtime_control::{self, BattleRuntimeFault, RuntimeFaultCode},
        },
        behavior::GameError,
        value_expr::{CompiledAmountPlan, EvalMember},
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    mem::size_of,
};

impl BattleCore {
    pub(super) fn owned_dot_targets(
        &self,
        owner: UnitInstanceId,
        now: u64,
        max_members: usize,
    ) -> Result<BTreeSet<UnitInstanceId>, GameError> {
        let mut targets = BTreeSet::new();
        let mut accept = |id: UnitInstanceId| -> Result<(), GameError> {
            control::charge(
                1 + (targets.len().max(1).ilog2() as u64),
                "owned_dot_target",
            )?;
            if targets.contains(&id) {
                return Ok(());
            }
            if targets.len() >= max_members {
                return Err(runtime_control::reject(BattleRuntimeFault {
                    code: RuntimeFaultCode::RetainedResultLimit,
                    site: "owned_dot_members",
                    limit: Some(max_members as u64),
                    requested: Some(targets.len() as u64 + 1),
                }));
            }
            let stride = size_of::<UnitInstanceId>() + 128;
            runtime_control::admit_retained_growth(
                targets.len().saturating_mul(stride),
                targets
                    .len()
                    .checked_add(1)
                    .and_then(|n| n.checked_mul(stride)),
                "owned_dot_member_storage",
            )?;
            targets.insert(id);
            Ok(())
        };
        self.visit_owned_dot_targets(owner, now, &mut accept)?;
        Ok(targets)
    }

    fn visit_owned_dot_targets(
        &self,
        owner: UnitInstanceId,
        now: u64,
        mut visit: impl FnMut(UnitInstanceId) -> Result<(), GameError>,
    ) -> Result<(), GameError> {
        let Some(source) = self.units.get(&owner).filter(|u| {
            u.is_active() && u.is_combatant() && u.stats.current_health > 0 && !u.corroded
        }) else {
            return Ok(());
        };
        let mut accept_live = |id| {
            if self.units.get(&id).is_some_and(|unit| {
                unit.is_active()
                    && unit.stats.current_health > 0
                    && unit.is_combatant()
                    && unit.owner != source.owner
            }) {
                visit(id)?;
            }
            Ok(())
        };
        control::charge(self.buffs.len() as u64, "owned_dot_buff_scan")?;
        for (key, buff) in self.buffs.iter() {
            if key.caster_instance_id == owner && buff.expires_at_ms > now && buff.stacks > 0
                && self.game_data.buff_data.get(key.buff_id).is_some_and(|definition| {
                    matches!(definition.kind, BuffKind::PeriodicDamage { damage_per_tick, .. } if damage_per_tick > 0)
                })
            {
                accept_live(key.target_instance_id)?;
            }
        }
        self.visit_owned_presence_dot_targets(owner, &mut accept_live)?;
        Ok(())
    }

    pub(super) fn target_has_owned_dot(
        &self,
        owner: UnitInstanceId,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<bool, GameError> {
        let mut found = false;
        self.visit_owned_dot_targets(owner, now, |id| {
            found |= id == target;
            Ok(())
        })?;
        Ok(found)
    }

    pub(super) fn state_owner_dot_members(
        &self,
        owner: UnitInstanceId,
        now: u64,
        plan: &CompiledAmountPlan,
    ) -> Result<Vec<EvalMember>, GameError> {
        let targets =
            self.owned_dot_targets(owner, now, plan.evaluation_limits().max_dynamic_members)?;
        control::charge(targets.len() as u64, "owned_dot_frame_members")?;
        runtime_control::admit_retained_growth(
            0,
            targets
                .len()
                .checked_mul(size_of::<EvalMember>() + 36)
                .and_then(|bytes| {
                    bytes.checked_add(size_of::<Vec<EvalMember>>() + 2 * size_of::<usize>())
                }),
            "owned_dot_frame_storage",
        )?;
        Ok(targets
            .into_iter()
            .map(|target| EvalMember {
                identity: target.to_string(),
                value: None,
                owner: None,
                projected_tile: None,
                values: BTreeMap::new(),
            })
            .collect())
    }
}
