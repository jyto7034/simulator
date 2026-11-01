mod actual_heal_reaction;
mod recipient;
mod shield_beneficiary;
pub use shield_beneficiary::ShieldBeneficiaryDamageWitness;
pub use recipient::{ReceivedHealWitness,ControlDurationWitness};
pub(in crate::game::battle::core) use recipient::{sustained_control_kind,buff_control_kind};
pub use actual_heal_reaction::{ActualHealContribution, ActualHealReactionWitness, HealShieldOutcome, HealHasteOutcome};
mod basic_heal_echo;
pub use basic_heal_echo::BasicHealEchoWitness;
// One outgoing support adjustment at the direct-heal / new-shield boundary.
use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        battle::{
            ids::UnitInstanceId,
            runtime_control,
            runtime_facts::{EquipmentSourceSnapshot, RuntimeFactSource},
        },
        behavior::GameError,
    },
};
use std::{mem::size_of, sync::Arc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportChannel {
    Heal,
    Shield,
}
crate::authoring::clone_shape::copy_shape!(SupportChannel);

#[derive(Debug, Clone, serde::Serialize)]
pub struct SupportContribution {
    pub equipment: EquipmentSourceSnapshot,
    pub percent: u32,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
crate::authoring::clone_shape::struct_shape!(SupportContribution {
    equipment,
    percent,
    definition_id,
    source_file,
    field_path
});
#[derive(Debug, Clone, serde::Serialize)]
pub struct EquipmentSupportWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub actor: UnitInstanceId,
    pub target: UnitInstanceId,
    pub channel: SupportChannel,
    pub base_amount: u32,
    pub increased_amount: u32,
    pub percent: u64,
    pub contributors: Vec<SupportContribution>,
    pub source: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(EquipmentSupportWitness {
    generation,
    time_ms,
    actor,
    target,
    channel,
    base_amount,
    increased_amount,
    percent,
    contributors,
    source
});
impl EquipmentSupportWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>()
            .checked_add(self.source.retained_extent()?)?
            .checked_add(
                self.contributors
                    .capacity()
                    .checked_mul(size_of::<SupportContribution>())?,
            )?;
        if let Some(generation) = &self.generation {
            bytes = bytes
                .checked_add(size_of_val(generation.as_ref()))?
                .checked_add(generation.semantic_sha256.capacity())?;
        }
        for c in &self.contributors {
            bytes = bytes
                .checked_add(c.definition_id.capacity())?
                .checked_add(c.source_file.capacity())?
                .checked_add(c.field_path.capacity())?;
        }
        Some(bytes)
    }
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealBarrierWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub actor: UnitInstanceId,
    pub target: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
    pub hp_before: u32,
    pub max_hp: u32,
    pub actual_healed: u64,
    pub base_capacity: u32,
    pub increased_capacity: u32,
    pub remaining: u32,
    pub expires_at_ms: u64,
    pub ready_at_ms: u64,
    pub kept_existing: bool,
    pub healing_source: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(HealBarrierWitness {
    generation,
    time_ms,
    actor,
    target,
    equipment,
    definition_id,
    source_file,
    field_path,
    hp_before,
    max_hp,
    actual_healed,
    base_capacity,
    increased_capacity,
    remaining,
    expires_at_ms,
    ready_at_ms,
    kept_existing,
    healing_source
});
impl HealBarrierWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut n = size_of::<Self>()
            .checked_add(self.definition_id.capacity())?
            .checked_add(self.source_file.capacity())?
            .checked_add(self.field_path.capacity())?
            .checked_add(self.healing_source.retained_extent()?)?;
        if let Some(g) = &self.generation {
            n = n
                .checked_add(size_of_val(g.as_ref()))?
                .checked_add(g.semantic_sha256.capacity())?;
        }
        Some(n)
    }
}
impl BattleCore {
    pub(super) fn equipment_support_pair_eligible(
        &self,
        actor: UnitInstanceId,
        target: UnitInstanceId,
    ) -> bool {
        actor != target
            && self
                .units
                .get(&actor)
                .zip(self.units.get(&target))
                .is_some_and(|(a, t)| {
                    a.is_active()
                        && !a.corroded
                        && a.is_combatant()
                        && t.is_active()
                        && !t.corroded
                        && t.is_combatant()
                        && t.owner == a.owner
                })
    }
    pub(super) fn equipment_support_amount(
        &mut self,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        amount: u32,
        channel: SupportChannel,
        time_ms: u64,
    ) -> Result<u32, GameError> {
        let catalog = self.game_data.equipment_support_catalog();
        if amount == 0 || actor == target || catalog.by_equipment.is_empty() {
            return Ok(amount);
        }
        if !self.equipment_support_pair_eligible(actor, target) {
            return Ok(amount);
        }
        let unit = &self.units[&actor];
        let slots = unit.equipment_slots.as_ref();
        control::charge(
            if slots.is_some() { 3 } else { self.items.len() } as u64,
            "equipment_support_providers",
        )?;
        let slot_items = slots
            .into_iter()
            .flat_map(|s| s.slots.iter().flatten())
            .map(|s| {
                self.items
                    .get(&s.runtime_item_instance_id)
                    .ok_or_else(|| runtime_control::invariant("equipment_support_slot"))
            });
        let legacy_items = self
            .items
            .values()
            .take(if slots.is_some() { 0 } else { self.items.len() })
            .filter(|i| i.owner_unit_instance == actor)
            .map(Ok);
        let mut percent = 0u64;
        let mut contributors = Vec::new();
        let mut contribution_bytes = Some(0usize);
        for item in slot_items.chain(legacy_items) {
            let item = item?;
            if item.owner_unit_instance != actor {
                return Err(runtime_control::invariant("equipment_support_owner"));
            }
            let Some(plan) = catalog.by_equipment.get(&item.base_uuid) else {
                continue;
            };
            let increase = match channel {
                SupportChannel::Heal => plan.program.heal_percent,
                SupportChannel::Shield => plan.program.shield_percent,
            };
            if increase == 0 {
                continue;
            }
            percent = percent
                .checked_add(u64::from(increase))
                .ok_or_else(|| runtime_control::invariant("equipment_support_percent"))?;
            if self.authoring_observation.is_some() {
                let suffix = match channel {
                    SupportChannel::Heal => "heal_percent",
                    SupportChannel::Shield => "shield_percent",
                };
                let field_bytes = plan.field_path.len().checked_add(1 + suffix.len());
                let bytes = size_of::<SupportContribution>()
                    .checked_add(plan.definition_id.len())
                    .and_then(|n| n.checked_add(plan.source_file.len()))
                    .and_then(|n| field_bytes.and_then(|field| n.checked_add(field)));
                contribution_bytes =
                    contribution_bytes.and_then(|old| bytes.and_then(|n| old.checked_add(n)));
                // This vector is still local, so the core's retained snapshot
                // cannot account for earlier contributors until publication.
                runtime_control::admit_retained_growth(
                    0,
                    contribution_bytes,
                    "equipment_support_contribution",
                )?;
                control::charge(
                    bytes.unwrap_or(usize::MAX) as u64,
                    "equipment_support_contribution_copy",
                )?;
                let mut field_path =
                    String::with_capacity(field_bytes.expect("admitted field size"));
                field_path.push_str(&plan.field_path);
                field_path.push('.');
                field_path.push_str(suffix);
                contributors.reserve_exact(1);
                contributors.push(SupportContribution {
                    equipment: item.source_snapshot(),
                    percent: increase,
                    definition_id: plan.definition_id.clone(),
                    source_file: plan.source_file.clone(),
                    field_path,
                });
            }
        }
        let cap = match channel {
            SupportChannel::Heal => i32::MAX as u32,
            SupportChannel::Shield => u32::MAX,
        };
        let increased =
            (u128::from(amount) * (100 + u128::from(percent)) / 100).min(u128::from(cap)) as u32;
        if !contributors.is_empty() {
            let source_bytes = self.runtime_fact_source.retained_extent();
            runtime_control::admit_retained_growth(
                0,
                source_bytes
                    .and_then(|n| n.checked_add(size_of::<EquipmentSupportWitness>()))
                    .and_then(|n| contribution_bytes.and_then(|c| n.checked_add(c))),
                "equipment_support_source",
            )?;
            control::charge(
                source_bytes.unwrap_or(usize::MAX) as u64,
                "equipment_support_source_copy",
            )?;
            let witness = EquipmentSupportWitness {
                generation: self.content_identity.clone(),
                time_ms,
                actor,
                target,
                channel,
                base_amount: amount,
                increased_amount: increased,
                percent,
                contributors,
                source: self.runtime_fact_source.observation(),
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "equipment_support_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_equipment_support = Some(witness);
        }
        control::poll("equipment_support_evaluated")?;
        Ok(increased)
    }

    pub(super) fn apply_direct_heal_and_record(
        &mut self,
        actor: Option<UnitInstanceId>,
        target: UnitInstanceId,
        delta: i32,
        now: u64,
    ) -> Result<(), GameError> {
        self.apply_direct_heal_with_observation_origin(actor, target, delta, now, None)
    }

    pub(super) fn apply_direct_heal_with_observation_origin(
        &mut self,
        actor: Option<UnitInstanceId>,
        target: UnitInstanceId,
        delta: i32,
        now: u64,
        observation_origin: Option<&RuntimeFactSource>,
    ) -> Result<(), GameError> {
        let amount = if delta > 0 {
            if let Some(actor) = actor {
                self.equipment_support_amount(
                    actor,
                    target,
                    delta as u32,
                    SupportChannel::Heal,
                    now,
                )? as i32
            } else {
                delta
            }
        } else {
            delta
        };
        let amount = if amount > 0 {
            self.received_heal_amount(actor, target, amount as u32, now)? as i32
        } else { amount };
        let before = self
            .units
            .get(&target)
            .map(|u| (u.stats.current_health, u.stats.max_health));
        self.apply_hp_delta_with_observation_origin(
            actor,
            target,
            amount,
            now,
            crate::game::battle::event_log::HpChangeReason::Command,
            true,
            observation_origin,
        );
        control::poll("direct_heal_applied")?;
        if amount > 0 {
            if let (Some(actor), Some((hp_before, max_hp)), Some(unit)) =
                (actor, before, self.units.get(&target))
            {
                let actual = unit.stats.current_health.saturating_sub(hp_before);
                if actual > 0 {
                    self.actual_heal_equipment_reactions(actor, target, actual, now)?;
                    self.heal_barrier_after_direct_heal(
                        actor, target, hp_before, max_hp, actual, now,
                    )?;
                }
            }
        }
        Ok(())
    }
}
