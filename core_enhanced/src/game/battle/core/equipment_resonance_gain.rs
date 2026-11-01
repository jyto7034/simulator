//! Equipment gain scaling at explicit automatic/authored positive gain boundaries.
use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        ability::equipment_support::{ResonanceGainChannels, ResonanceGainProgram},
        battle::{ids::UnitInstanceId, runtime_control, runtime_facts::EquipmentSourceSnapshot},
        behavior::GameError,
    },
};
use std::{
    mem::{size_of, size_of_val},
    sync::Arc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResonanceGainChannel {
    Automatic,
    Authored,
}
crate::authoring::clone_shape::copy_shape!(ResonanceGainChannel);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ResonanceGainProvider {
    equipment: EquipmentSourceSnapshot,
    rule: ResonanceGainProgram,
}
crate::authoring::clone_shape::struct_shape!(ResonanceGainProvider { equipment, rule });
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResonanceGainState {
    providers: Vec<ResonanceGainProvider>,
    remainder: u8,
}
crate::authoring::clone_shape::struct_shape!(ResonanceGainState {
    providers,
    remainder
});
impl ResonanceGainState {
    pub(super) fn dynamic_retained_extent(&self) -> Option<usize> {
        self.providers
            .capacity()
            .checked_mul(size_of::<ResonanceGainProvider>())
    }
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResonanceGainContribution {
    pub equipment: EquipmentSourceSnapshot,
    pub percent: u32,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
crate::authoring::clone_shape::struct_shape!(ResonanceGainContribution {
    equipment,
    percent,
    definition_id,
    source_file,
    field_path
});
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResonanceGainWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub actor: UnitInstanceId,
    pub channel: ResonanceGainChannel,
    pub base_amount: u32,
    pub increased_amount: u32,
    pub actual_gain: u32,
    pub before: u32,
    pub after: u32,
    pub max: u32,
    pub percent: u64,
    pub remainder_before: u8,
    pub remainder_after: u8,
    pub contributors: Vec<ResonanceGainContribution>,
}
crate::authoring::clone_shape::struct_shape!(ResonanceGainWitness {
    generation,
    time_ms,
    actor,
    channel,
    base_amount,
    increased_amount,
    actual_gain,
    before,
    after,
    max,
    percent,
    remainder_before,
    remainder_after,
    contributors
});
impl ResonanceGainWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut n = size_of::<Self>().checked_add(
            self.contributors
                .capacity()
                .checked_mul(size_of::<ResonanceGainContribution>())?,
        )?;
        for c in &self.contributors {
            n = n
                .checked_add(c.definition_id.capacity())?
                .checked_add(c.source_file.capacity())?
                .checked_add(c.field_path.capacity())?;
        }
        if let Some(g) = &self.generation {
            n = n
                .checked_add(size_of_val(g.as_ref()))?
                .checked_add(g.semantic_sha256.capacity())?;
        }
        Some(n)
    }
}
impl BattleCore {
    pub(super) fn equipment_resonance_gain_amount(
        &mut self,
        actor: UnitInstanceId,
        amount: u32,
        channel: ResonanceGainChannel,
        before: u32,
        max: u32,
        now: u64,
    ) -> Result<u32, GameError> {
        let Some(unit) = self.units.get(&actor) else {
            return Ok(amount);
        };
        if self.sustained.closed
            || !Self::is_employee_resonance_unit(unit)
            || unit.corroded
            || unit.stats.current_health == 0
        {
            if let Some(unit) = self.units.get_mut(&actor) {
                unit.equipment_resonance_gain = None;
            }
            return Ok(amount);
        }
        let catalog = self.game_data.equipment_support_catalog();
        if !catalog.has_resonance_gain {
            self.units.get_mut(&actor).unwrap().equipment_resonance_gain = None;
            return Ok(amount);
        }
        let slots = unit.equipment_slots.as_ref();
        let capacity = slots.map_or(self.items.len(), |s| s.slots.iter().flatten().count());
        control::charge(capacity as u64, "resonance_gain_providers")?;
        runtime_control::charge(capacity as u64, "resonance_gain_providers")?;
        runtime_control::admit_retained_growth(
            0,
            capacity.checked_mul(size_of::<ResonanceGainProvider>()),
            "resonance_gain_signature",
        )?;
        let mut providers = Vec::with_capacity(capacity);
        let mut contributors = Vec::new();
        let mut witness_bytes = Some(0usize);
        let mut percent = 0u64;
        let slot_items = slots
            .into_iter()
            .flat_map(|s| s.slots.iter().flatten())
            .map(|s| {
                self.items
                    .get(&s.runtime_item_instance_id)
                    .ok_or_else(|| runtime_control::invariant("resonance_gain_slot"))
            });
        let legacy_items = self
            .items
            .values()
            .take(if slots.is_some() { 0 } else { self.items.len() })
            .filter(|i| i.owner_unit_instance == actor)
            .map(Ok);
        for item in slot_items.chain(legacy_items) {
            let item = item?;
            if item.owner_unit_instance != actor {
                return Err(runtime_control::invariant("resonance_gain_owner"));
            }
            let Some(plan) = catalog.by_equipment.get(&item.base_uuid) else {
                continue;
            };
            let Some(rule) = plan.program.resonance_gain.filter(|r| r.percent > 0) else {
                continue;
            };
            let equipment = item.source_snapshot();
            providers.push(ResonanceGainProvider { equipment, rule });
            let applies = matches!(
                (rule.channels, channel),
                (ResonanceGainChannels::AutomaticAndAuthored, _)
                    | (
                        ResonanceGainChannels::Automatic,
                        ResonanceGainChannel::Automatic
                    )
                    | (
                        ResonanceGainChannels::Authored,
                        ResonanceGainChannel::Authored
                    )
            );
            if !applies {
                continue;
            }
            percent = percent
                .checked_add(u64::from(rule.percent))
                .ok_or_else(|| runtime_control::invariant("resonance_gain_percent"))?;
            if self.authoring_observation.is_some() {
                let field_len = plan.field_path.len().checked_add(".resonance_gain".len());
                let bytes = size_of::<ResonanceGainContribution>()
                    .checked_add(plan.definition_id.len())
                    .and_then(|n| n.checked_add(plan.source_file.len()))
                    .and_then(|n| field_len.and_then(|f| n.checked_add(f)));
                witness_bytes = witness_bytes.and_then(|n| bytes.and_then(|b| n.checked_add(b)));
                runtime_control::admit_retained_growth(
                    0,
                    witness_bytes.and_then(|n| {
                        capacity
                            .checked_mul(size_of::<ResonanceGainProvider>())
                            .and_then(|p| n.checked_add(p))
                    }),
                    "resonance_gain_witness_copy",
                )?;
                control::charge(
                    bytes.unwrap_or(usize::MAX) as u64,
                    "resonance_gain_witness_copy",
                )?;
                contributors.reserve_exact(1);
                contributors.push(ResonanceGainContribution {
                    equipment,
                    percent: rule.percent,
                    definition_id: plan.definition_id.clone(),
                    source_file: plan.source_file.clone(),
                    field_path: format!("{}.resonance_gain", plan.field_path),
                });
            }
        }
        let sort_work = (providers.len() as u64)
            .saturating_mul(u64::from(usize::BITS - providers.len().leading_zeros()));
        control::charge(sort_work, "resonance_gain_signature_sort")?;
        runtime_control::charge(sort_work, "resonance_gain_signature_sort")?;
        providers.sort_unstable_by_key(|p| p.equipment);
        contributors.sort_unstable_by_key(|c| c.equipment);
        let old = self.units[&actor].equipment_resonance_gain.as_ref();
        let remainder_before = old
            .filter(|s| s.providers == providers)
            .map_or(0, |s| s.remainder);
        // Only matching channels may spend the fraction. Other channels preserve it.
        let numerator = if percent == 0 {
            0
        } else {
            u128::from(amount) * u128::from(percent) + u128::from(remainder_before)
        };
        let increased = (u128::from(amount) + numerator / 100).min(u128::from(u32::MAX)) as u32;
        let after = before.saturating_add(increased).min(max);
        let remainder_after = if after == max {
            0
        } else if percent == 0 {
            remainder_before
        } else {
            (numerator % 100) as u8
        };
        let state = (!providers.is_empty() && after < max).then_some(ResonanceGainState {
            providers,
            remainder: remainder_after,
        });
        if let Some(state) = &state {
            runtime_control::admit_retained_growth(
                0,
                state.dynamic_retained_extent(),
                "resonance_gain_state",
            )?;
        }
        let witness = if contributors.is_empty() {
            None
        } else {
            Some(ResonanceGainWitness {
                generation: self.content_identity.clone(),
                time_ms: now,
                actor,
                channel,
                base_amount: amount,
                increased_amount: increased,
                actual_gain: after - before,
                before,
                after,
                max,
                percent,
                remainder_before,
                remainder_after,
                contributors,
            })
        };
        if let Some(w) = &witness {
            runtime_control::admit_retained_growth(
                0,
                w.retained_extent(),
                "resonance_gain_observation",
            )?;
        }
        control::poll("resonance_gain_evaluated")?;
        self.units.get_mut(&actor).unwrap().equipment_resonance_gain = state;
        if let Some(witness) = witness {
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_resonance_gain = Some(witness);
        }
        Ok(increased)
    }
}
