//! Equipment reactions share the positive actual-heal boundary, existing pools
//! absorb damage and existing target states compose speed. No parallel balance.
use super::super::equipment_support::{
    ActualHealContribution, ActualHealReactionWitness, HealHasteOutcome, HealShieldOutcome,
};
use super::*;
use crate::game::{
    ability::{
        equipment_support::{
            ActualHealHasteProgram, ActualHealShieldProgram, CompiledEquipmentSupport,
        },
        sustained::TargetStateProfile,
    },
    battle::{runtime_control, runtime_facts::EquipmentSourceSnapshot},
};
use std::mem::size_of;
use uuid::Uuid;
const HASTE_FAMILY: &str = "equipment_actual_heal_haste";
#[derive(Debug, Clone, Copy)]
pub(super) struct HealShieldState {
    owner: UnitInstanceId,
    rule: ActualHealShieldProgram,
    activation: Option<u64>,
    remainder: u8,
    expires_at: u64,
}
crate::authoring::clone_shape::struct_shape!(HealShieldState {
    owner,
    rule,
    activation,
    remainder,
    expires_at
});
impl BattleCore {
    fn actual_heal_copy_live(
        &self,
        copy: Uuid,
        owner: UnitInstanceId,
        rule: ActualHealShieldProgram,
    ) -> bool {
        self.units.get(&owner).is_some_and(|u| {
            u.is_active()
                && !u.corroded
                && u.is_combatant()
                && u.stats.current_health > 0
                && u.equipment_slots.as_ref().is_none_or(|s| {
                    s.slots
                        .iter()
                        .flatten()
                        .any(|s| s.runtime_item_instance_id == copy)
                })
        }) && self.items.get(&copy).is_some_and(|i| {
            i.owner_unit_instance == owner
                && self
                    .game_data
                    .equipment_support_catalog()
                    .by_equipment
                    .get(&i.base_uuid)
                    .and_then(|p| p.program.actual_heal_reaction)
                    .and_then(|p| p.self_shield)
                    == Some(rule)
        })
    }
    pub(super) fn reconcile_actual_heal_shields(&mut self, now: u64) -> Result<(), GameError> {
        let mut cursor = None;
        loop {
            let n = self.sustained.heal_shields.len() as u64;
            control::charge(n, "actual_heal_shield_reconcile")?;
            runtime_control::charge(n, "actual_heal_shield_reconcile")?;
            let Some((copy, mut state)) = self
                .sustained
                .heal_shields
                .iter()
                .find(|(id, _)| cursor.is_none_or(|c| **id > c))
                .map(|(k, v)| (*k, *v))
            else {
                break;
            };
            cursor = Some(copy);
            if state.expires_at <= now || !self.actual_heal_copy_live(copy, state.owner, state.rule)
            {
                self.sustained.heal_shields.remove(&copy);
                if let Some(id) = state.activation {
                    self.end_sustained(
                        id,
                        now,
                        if state.expires_at <= now {
                            SustainedEndReason::Expired
                        } else {
                            SustainedEndReason::SourceUnavailable
                        },
                    )?;
                }
                continue;
            }
            let cap = (u64::from(self.units[&state.owner].stats.max_health)
                * u64::from(state.rule.max_hp_cap_percent)
                / 100) as u32;
            if let Some(id) = state.activation {
                if let Some(pool) = self
                    .sustained
                    .records
                    .get_mut(&id)
                    .and_then(|r| r.shield.as_mut())
                {
                    pool.capacity = cap;
                    pool.remaining = pool.remaining.min(cap);
                    if pool.remaining >= cap {
                        state.remainder = 0;
                    }
                    if pool.remaining == 0 {
                        self.end_sustained(id, now, SustainedEndReason::Depleted)?;
                        state.activation = None;
                    }
                } else {
                    state.activation = None;
                }
            }
            if cap == 0 {
                state.remainder = 0;
            }
            self.sustained.heal_shields.insert(copy, state);
        }
        Ok(())
    }
    pub(in crate::game::battle::core) fn actual_heal_equipment_reactions(
        &mut self,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        actual: u32,
        now: u64,
    ) -> Result<(), GameError> {
        if actual == 0
            || self.sustained.closed
            || !self.equipment_support_pair_eligible(actor, target)
            || self.units[&actor].stats.current_health == 0
            || self.units[&target].stats.current_health == 0
            || !self
                .game_data
                .equipment_support_catalog()
                .has_actual_heal_reaction
        {
            return Ok(());
        }
        self.reconcile_actual_heal_shields(now)?;
        let data = Arc::clone(&self.game_data);
        let catalog = data.equipment_support_catalog();
        let observing = self.authoring_observation.is_some();
        let capacity = self.units[&actor]
            .equipment_slots
            .as_ref()
            .map_or(self.items.len(), |s| s.slots.iter().flatten().count());
        let mut contributors = Vec::new();
        if observing {
            runtime_control::admit_retained_growth(
                0,
                capacity.checked_mul(size_of::<ActualHealContribution>()),
                "actual_heal_observation_providers",
            )?;
            contributors.reserve_exact(capacity);
        }
        let mut cursor = None;
        loop {
            control::charge(capacity as u64, "actual_heal_providers")?;
            runtime_control::charge(capacity as u64, "actual_heal_providers")?;
            let slots = self.units[&actor].equipment_slots.as_ref();
            let equipped = slots
                .into_iter()
                .flat_map(|s| s.slots.iter().flatten())
                .map(|s| {
                    self.items
                        .get(&s.runtime_item_instance_id)
                        .ok_or_else(|| runtime_control::invariant("actual_heal_slot"))
                });
            let legacy = self
                .items
                .values()
                .take(if slots.is_none() { self.items.len() } else { 0 })
                .filter(|i| i.owner_unit_instance == actor)
                .map(Ok);
            let mut selected = None;
            for item in equipped.chain(legacy) {
                let item = item?;
                if item.owner_unit_instance != actor {
                    return Err(runtime_control::invariant("actual_heal_owner"));
                }
                let equipment = item.source_snapshot();
                let id = equipment.runtime_item_instance_id;
                if cursor.is_some_and(|c| id <= c) {
                    continue;
                }
                if let Some(plan) = catalog
                    .by_equipment
                    .get(&item.base_uuid)
                    .filter(|p| p.program.actual_heal_reaction.is_some())
                {
                    if selected.as_ref().is_none_or(|(old, _, _)| id < *old) {
                        selected = Some((id, equipment, Arc::clone(plan)));
                    }
                }
            }
            let Some((copy, equipment, plan)) = selected else {
                break;
            };
            cursor = Some(copy);
            let rule = plan
                .program
                .actual_heal_reaction
                .expect("selected reaction");
            let self_shield = rule
                .self_shield
                .map(|rule| {
                    self.accumulate_actual_heal_shield(actor, actual, now, equipment, &plan, rule)
                })
                .transpose()?;
            let target_haste = rule
                .target_haste
                .map(|rule| {
                    self.refresh_actual_heal_haste(actor, target, now, equipment, &plan, rule)
                })
                .transpose()?;
            if observing {
                let source = plan
                    .reaction_source
                    .as_ref()
                    .expect("compiled reaction source");
                let bytes = plan
                    .definition_id
                    .len()
                    .checked_add(source.source_file.len())
                    .and_then(|n| n.checked_add(source.field_path.len()));
                runtime_control::admit_retained_growth(
                    0,
                    bytes,
                    "actual_heal_observation_contribution",
                )?;
                control::charge(
                    bytes.unwrap_or(usize::MAX) as u64,
                    "actual_heal_observation_copy",
                )?;
                contributors.push(ActualHealContribution {
                    equipment,
                    definition_id: plan.definition_id.clone(),
                    source_file: source.source_file.clone(),
                    field_path: source.field_path.clone(),
                    self_shield,
                    target_haste,
                });
            }
            control::poll("actual_heal_reaction_provider_applied")?;
        }
        if observing && !contributors.is_empty() {
            self.runtime_fact_source
                .admit_owner("actual_heal_observation_source")?;
            control::charge(
                self.runtime_fact_source
                    .retained_extent()
                    .unwrap_or(usize::MAX) as u64,
                "actual_heal_observation_source_copy",
            )?;
            let healing_source = self.runtime_fact_source.observation();
            let witness = ActualHealReactionWitness {
                generation: self.content_identity.clone(),
                time_ms: now,
                actor,
                target,
                actual_healed: actual,
                healing_source,
                contributors,
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "actual_heal_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_actual_heal_reaction = Some(witness);
        }
        control::poll("actual_heal_reaction_applied")
    }
    fn accumulate_actual_heal_shield(
        &mut self,
        actor: UnitInstanceId,
        actual: u32,
        now: u64,
        equipment: EquipmentSourceSnapshot,
        plan: &CompiledEquipmentSupport,
        rule: ActualHealShieldProgram,
    ) -> Result<HealShieldOutcome, GameError> {
        let copy = equipment.runtime_item_instance_id;
        let expires_at = now
            .checked_add(u64::from(rule.duration_ms))
            .ok_or_else(|| overflow("actual_heal_shield_expiry"))?;
        let mut state =
            self.sustained
                .heal_shields
                .get(&copy)
                .copied()
                .unwrap_or(HealShieldState {
                    owner: actor,
                    rule,
                    activation: None,
                    remainder: 0,
                    expires_at,
                });
        if !self.sustained.heal_shields.contains_key(&copy) {
            runtime_control::admit_retained_growth(
                0,
                Some(size_of::<Uuid>() + size_of::<HealShieldState>() + 96),
                "actual_heal_shield_state",
            )?;
            self.admit_authoring_runtime_growth(1, "actual_heal_shield_state")?;
        }
        let cap = (u64::from(self.units[&actor].stats.max_health)
            * u64::from(rule.max_hp_cap_percent)
            / 100) as u32;
        let before = state
            .activation
            .and_then(|id| self.sustained.records.get(&id))
            .and_then(|r| r.shield)
            .map_or(0, |p| p.remaining);
        let remainder_before = state.remainder;
        let hundredths =
            u64::from(actual) * u64::from(rule.actual_heal_percent) + u64::from(state.remainder);
        let after = (u64::from(before) + hundredths / 100).min(u64::from(cap)) as u32;
        state.remainder = if after >= cap {
            0
        } else {
            (hundredths % 100) as u8
        };
        state.expires_at = expires_at;
        if let Some(id) = state.activation {
            let r = self
                .sustained
                .records
                .get_mut(&id)
                .expect("reconciled heal shield");
            r.shield.as_mut().unwrap().remaining = after;
            r.shield.as_mut().unwrap().capacity = cap;
            r.expires_at = expires_at;
            self.event_queue.discard_sustained_expiry(id)?;
            self.event_queue.push(BattleEvent::SustainedExpiry {
                time_ms: expires_at,
                activation: id,
            });
        } else if after > 0 {
            let source = RuntimeFactSource {
                actor: Some(actor),
                equipment: Some(equipment),
                authored: plan.reaction_source.clone(),
                ..Default::default()
            };
            let prior = std::mem::replace(&mut self.runtime_fact_source, source);
            let old = self.sustained.next;
            let result = self.begin_sustained_owned_with_shield(
                actor,
                actor,
                SustainedEffectDef::IndependentShield {
                    duration_ms: rule.duration_ms,
                    amount: after,
                    priority: rule.priority,
                },
                now,
                None,
                None,
                None,
                Some(after),
            );
            self.runtime_fact_source = prior;
            result?;
            if self.sustained.next == old {
                return Err(runtime_control::invariant("actual_heal_shield_start"));
            }
            let id = self.sustained.next;
            self.sustained
                .records
                .get_mut(&id)
                .unwrap()
                .shield
                .as_mut()
                .unwrap()
                .capacity = cap;
            state.activation = Some(id);
        }
        self.sustained.heal_shields.insert(copy, state);
        Ok(HealShieldOutcome {
            actual_heal_percent: rule.actual_heal_percent,
            max_hp_cap_percent: rule.max_hp_cap_percent,
            before,
            after,
            cap,
            remainder_before,
            remainder_after: state.remainder,
            expires_at_ms: expires_at,
        })
    }
    fn refresh_actual_heal_haste(
        &mut self,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        now: u64,
        equipment: EquipmentSourceSnapshot,
        plan: &CompiledEquipmentSupport,
        rule: ActualHealHasteProgram,
    ) -> Result<HealHasteOutcome, GameError> {
        let expires_at = now
            .checked_add(u64::from(rule.duration_ms))
            .ok_or_else(|| overflow("actual_heal_haste_expiry"))?;
        let n = self.sustained.records.len() as u64;
        control::charge(n, "actual_heal_haste_scan")?;
        runtime_control::charge(n, "actual_heal_haste_scan")?;
        let existing = self
            .sustained
            .records
            .iter()
            .find(|(_, r)| {
                r.caster == actor
                    && r.target == target
                    && r.source.equipment == Some(equipment)
                    && r.target_state
                        .as_ref()
                        .is_some_and(|p| p.definition == HASTE_FAMILY)
            })
            .map(|(id, r)| (*id, r.expires_at));
        let live = existing.filter(|(_, expiry)| *expiry > now);
        if let Some((id, _)) = existing.filter(|(_, expiry)| *expiry <= now) {
            self.end_sustained(id, now, SustainedEndReason::Expired)?;
        }
        if let Some((id, _)) = live {
            let r = self.sustained.records.get_mut(&id).unwrap();
            r.expires_at = expires_at;
            r.target_state.as_mut().unwrap().attack_speed_points = rule.attack_speed_points;
            self.event_queue.discard_sustained_expiry(id)?;
            self.event_queue.push(BattleEvent::SustainedExpiry {
                time_ms: expires_at,
                activation: id,
            });
        } else {
            let source = RuntimeFactSource {
                actor: Some(actor),
                equipment: Some(equipment),
                authored: plan.reaction_source.clone(),
                ..Default::default()
            };
            let prior = std::mem::replace(&mut self.runtime_fact_source, source);
            let old = self.sustained.next;
            let result = self.begin_sustained_owned_with_shield(
                actor,
                target,
                SustainedEffectDef::TargetState {
                    duration_ms: rule.duration_ms,
                    profile: TargetStateProfile {
                        definition: HASTE_FAMILY.into(),
                        attack_percent: 0,
                        defense_percent: 0,
                        attack_speed_points: rule.attack_speed_points,
                        hp_floor_one: false,
                        damage_invulnerable: false,
                        basic_attack_locked: false,
                        active_skill_locked: false,
                        received_heal_percent: 0,
                        end_on_source_exit: false,
                    },
                },
                now,
                None,
                None,
                None,
                None,
            );
            self.runtime_fact_source = prior;
            result?;
            if self.sustained.next == old {
                return Err(runtime_control::invariant("actual_heal_haste_start"));
            }
        }
        Ok(HealHasteOutcome {
            attack_speed_points: rule.attack_speed_points,
            expires_at_ms: expires_at,
            refreshed: live.is_some(),
        })
    }
}
