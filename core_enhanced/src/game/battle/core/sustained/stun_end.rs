//! Actual Stun removal produces a transient event. The existing effect owners
//! remain the only registry; no duplicated stunned-target set is retained.
use super::*;
use crate::game::{
    ability::{
        state_program::StateEvent,
        sustained::{HitStatusKind, HitStatusProfile},
    },
    battle::{buffs::BuffKind, runtime_control},
};

impl BattleCore {
    fn has_owned_stun_contribution(
        &self,
        owner: UnitInstanceId,
        target: UnitInstanceId,
    ) -> Result<bool, GameError> {
        control::charge(
            self.sustained.records.len() as u64
                + self.active_areas.len() as u64
                + self.buffs.len() as u64,
            "owner_stun_end_lookup",
        )?;
        // Include pending removals at this same timestamp: only the last
        // actual removal may emit, even when several effects expire together.
        if self.sustained.records.values().any(|r| {
            r.caster == owner
                && r.target == target
                && r.hit_status
                    .as_ref()
                    .is_some_and(|p| p.kind == HitStatusKind::Stun)
        }) {
            return Ok(true);
        }
        for area in self
            .active_areas
            .values()
            .filter(|a| a.caster_instance_id == owner)
        {
            if let Some(ground) = &area.ground_status {
                if ground.has_stun_target(target)? {
                    return Ok(true);
                }
            }
        }
        Ok(self.buffs.keys().any(|key| {
            key.caster_instance_id == owner
                && key.target_instance_id == target
                && self
                    .game_data
                    .buff_data
                    .get(key.buff_id)
                    .is_some_and(|b| b.kind == BuffKind::Stun)
        }))
    }

    pub(in crate::game::battle::core) fn dispatch_equipment_stun_end(
        &mut self,
        owner: UnitInstanceId,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        if self.sustained.closed || !self.game_data.equipment_state_catalog().has_owner_stun_end {
            return Ok(());
        }
        let Some(unit) = self.units.get(&owner).filter(|u| {
            u.is_active() && u.is_combatant() && u.stats.current_health > 0 && !u.corroded
        }) else {
            return Ok(());
        };
        if !self.units.get(&target).is_some_and(|u| {
            u.is_active() && u.is_combatant() && u.stats.current_health > 0 && u.owner != unit.owner
        }) || self.has_owned_stun_contribution(owner, target)?
        {
            return Ok(());
        }
        let mut cursor = None;
        loop {
            control::charge(self.items.len() as u64, "equipment_stun_end_providers")?;
            let candidate = self
                .items
                .iter()
                .filter(|(id, item)| {
                    cursor.is_none_or(|c| **id > c)
                        && item.owner_unit_instance == owner
                        && self.units[&owner]
                            .equipment_slots
                            .as_ref()
                            .is_none_or(|slots| {
                                slots
                                    .slots
                                    .iter()
                                    .flatten()
                                    .any(|s| s.runtime_item_instance_id == **id)
                            })
                        && self
                            .game_data
                            .equipment_state_catalog()
                            .by_equipment
                            .get(&item.base_uuid)
                            .is_some_and(|c| {
                                c.program
                                    .handlers
                                    .iter()
                                    .any(|h| h.event == StateEvent::OwnerStunEnded)
                            })
                })
                .min_by_key(|(id, _)| **id)
                .map(|(id, item)| (*id, item.source_snapshot()));
            let Some((id, equipment)) = candidate else {
                break;
            };
            cursor = Some(id);
            let activation = self.ensure_equipment_state(owner, equipment, now)?;
            if !self
                .sustained
                .equipped
                .get(&activation)
                .is_some_and(|r| self.equipped_state_owner_valid(r))
            {
                continue;
            }
            self.run_state_event(
                activation,
                state_program::StateEventInput::OwnerStunEnded { target },
                now,
            )?;
        }
        control::poll("equipment_stun_end_dispatched")
    }

    pub(super) fn apply_event_target_status(
        &mut self,
        owner: UnitInstanceId,
        target: UnitInstanceId,
        duration_ms: u32,
        profile: &HitStatusProfile,
        now: u64,
    ) -> Result<(), GameError> {
        let Some(side) = self
            .units
            .get(&owner)
            .filter(|u| u.is_active() && !u.corroded)
            .map(|u| u.owner)
        else {
            return Ok(());
        };
        if !self.units.get(&target).is_some_and(|u| {
            u.is_active() && u.stats.current_health > 0 && u.is_combatant() && u.owner != side
        }) {
            return Ok(());
        }
        let previous = self.sustained.next;
        // Install first: immunity/refusal must preserve any old contribution.
        // A replaced Stun must not create a spurious end between old and new.
        if !self.begin_committed_hit_status(
            owner,
            target,
            side,
            duration_ms,
            crate::authoring::clone_shape::controlled_clone(profile)?,
            now,
        )? {
            return Ok(());
        }
        loop {
            control::charge(
                self.sustained.records.len() as u64,
                "event_target_status_replace",
            )?;
            let equipment = self.runtime_fact_source.equipment;
            let old = self
                .sustained
                .records
                .iter()
                .find(|(id, r)| {
                    **id <= previous
                        && r.caster == owner
                        && r.target == target
                        && r.source.equipment == equipment
                        && r.hit_status
                            .as_ref()
                            .is_some_and(|p| p.definition == profile.definition)
                })
                .map(|(id, _)| *id);
            let Some(old) = old else {
                break;
            };
            self.end_sustained(old, now, SustainedEndReason::Replaced)?;
        }
        runtime_control::poll()?;
        control::poll("event_target_status_applied")
    }
}
