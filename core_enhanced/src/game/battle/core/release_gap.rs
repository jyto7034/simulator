//! Mounted copies own elapsed time; a released action owns one immutable bonus.
use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        ability::equipment_attack::CompiledEquipmentAttack,
        battle::{
            damage::{DamageSource, DamageSourceSnapshot},
            ids::UnitInstanceId,
            runtime_control,
            runtime_facts::EquipmentSourceSnapshot,
        },
        behavior::GameError,
    },
};
use std::{collections::BTreeSet, mem::size_of, sync::Arc};

#[derive(Debug, Clone)]
pub(super) struct ReleaseGapClock {
    pub owner: UnitInstanceId,
    pub employee: uuid::Uuid,
    equipment: EquipmentSourceSnapshot,
    program: Arc<CompiledEquipmentAttack>,
    since: u64,
}
crate::authoring::clone_shape::struct_shape!(ReleaseGapClock {
    owner,
    employee,
    equipment,
    program,
    since
});

#[derive(Debug)]
pub(crate) struct ReleaseGapCapture {
    equipment: EquipmentSourceSnapshot,
    program: Arc<CompiledEquipmentAttack>,
    since: u64,
    released_at: u64,
    pub percent: u32,
}
impl PartialEq for ReleaseGapCapture {
    fn eq(&self, other: &Self) -> bool {
        self.equipment == other.equipment
            && self.program.fingerprint == other.program.fingerprint
            && self.since == other.since
            && self.released_at == other.released_at
            && self.percent == other.percent
    }
}
impl Eq for ReleaseGapCapture {}
impl ReleaseGapCapture {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        (size_of::<Self>() + 64).checked_add(self.program.retained_bytes)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReleaseGapView {
    pub actor: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub since_ms: u64,
    pub elapsed_ms: u64,
    pub percent: u32,
    pub max_percent: u32,
    pub capped: bool,
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReleaseGapWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub actor: UnitInstanceId,
    pub time_ms: u64,
    pub since_ms: u64,
    pub elapsed_ms: u64,
    pub percent: u32,
    pub max_percent: u32,
    pub equipment: EquipmentSourceSnapshot,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
crate::authoring::clone_shape::struct_shape!(ReleaseGapWitness {
    generation,
    actor,
    time_ms,
    since_ms,
    elapsed_ms,
    percent,
    max_percent,
    equipment,
    definition_id,
    source_file,
    field_path
});
impl ReleaseGapWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>()
            .checked_add(self.definition_id.capacity())?
            .checked_add(self.source_file.capacity())?
            .checked_add(self.field_path.capacity())?;
        if let Some(g) = &self.generation {
            bytes = bytes
                .checked_add(size_of_val(g.as_ref()))?
                .checked_add(g.semantic_sha256.capacity())?;
        }
        Some(bytes)
    }
}
impl ReleaseGapClock {
    fn percent(&self, now: u64) -> u32 {
        let policy = self
            .program
            .release_gap
            .as_ref()
            .expect("release-gap provider");
        (u128::from(now.saturating_sub(self.since)) * u128::from(policy.per_second_percent) / 1000)
            .min(u128::from(policy.max_percent)) as u32
    }
}

impl BattleCore {
    pub(super) fn reconcile_release_gaps(&mut self, now: u64) -> Result<(), GameError> {
        if !self.game_data.equipment_attack_catalog().has_release_gap {
            return Ok(());
        }
        if self.sustained.closed {
            self.equipment_attack_state.release_gaps.clear();
            return Ok(());
        }
        control::charge(self.units.len() as u64, "release_gap_wearers")?;
        let mut live = BTreeSet::new();
        for owner in self.units.keys().copied() {
            for (equipment, program) in
                self.equipped_attack_providers(owner, |p| p.release_gap.is_some())?
            {
                let key = equipment.runtime_item_instance_id;
                runtime_control::admit_retained_growth(
                    0,
                    Some(size_of::<uuid::Uuid>() + 96),
                    "release_gap_live_copy",
                )?;
                live.insert(key);
                if self
                    .equipment_attack_state
                    .release_gaps
                    .get(&key)
                    .is_none_or(|c| c.owner != owner || c.equipment != equipment)
                {
                    runtime_control::admit_retained_growth(
                        self.equipment_attack_state
                            .retained_extent()
                            .unwrap_or(usize::MAX),
                        Some(size_of::<ReleaseGapClock>() + 128),
                        "release_gap_state",
                    )?;
                    self.equipment_attack_state.release_gaps.insert(
                        key,
                        ReleaseGapClock {
                            owner,
                            employee: self.units[&owner].source_owned_uuid,
                            equipment,
                            program,
                            since: now,
                        },
                    );
                }
            }
        }
        control::charge(
            self.equipment_attack_state.release_gaps.len() as u64,
            "release_gap_cleanup",
        )?;
        self.equipment_attack_state
            .release_gaps
            .retain(|key, _| live.contains(key));
        control::poll("release_gap_reconciled")
    }

    fn selected_release_gap(
        &self,
        actor: UnitInstanceId,
        now: u64,
    ) -> Result<Option<&ReleaseGapClock>, GameError> {
        if self.sustained.closed
            || self
                .units
                .get(&actor)
                .is_none_or(|u| !u.is_active() || u.corroded)
        {
            return Ok(None);
        }
        control::charge(
            self.equipment_attack_state.release_gaps.len() as u64,
            "release_gap_selection",
        )?;
        // The runtime clock is reconciled at equipment/lifecycle event boundaries.
        // Also check current copy membership for read-only queries.
        let unit = &self.units[&actor];
        Ok(self
            .equipment_attack_state
            .release_gaps
            .values()
            .filter(|c| {
                c.owner == actor
                    && c.employee == unit.source_owned_uuid
                    && self
                        .items
                        .get(&c.equipment.runtime_item_instance_id)
                        .is_some_and(|i| {
                            i.owner_unit_instance == actor && i.source_snapshot() == c.equipment
                        })
                    && unit.equipment_slots.as_ref().is_none_or(|s| {
                        s.slots.iter().flatten().any(|s| {
                            s.runtime_item_instance_id == c.equipment.runtime_item_instance_id
                        })
                    })
            })
            .max_by(|a, b| {
                a.percent(now).cmp(&b.percent(now)).then_with(|| {
                    b.equipment
                        .runtime_item_instance_id
                        .cmp(&a.equipment.runtime_item_instance_id)
                })
            }))
    }

    /// Derived preview at the caller's current battle time. Reading never
    /// charges, consumes, starts an attack, or advances simulation time.
    pub fn release_gap_snapshot(
        &self,
        actor: UnitInstanceId,
        battle_time_ms: u64,
    ) -> Result<Option<ReleaseGapView>, GameError> {
        let Some(clock) = self.selected_release_gap(actor, battle_time_ms)? else {
            return Ok(None);
        };
        let max_percent = clock.program.release_gap.as_ref().unwrap().max_percent;
        let percent = clock.percent(battle_time_ms);
        Ok(Some(ReleaseGapView {
            actor,
            equipment: clock.equipment,
            since_ms: clock.since,
            elapsed_ms: battle_time_ms.saturating_sub(clock.since),
            percent,
            max_percent,
            capped: percent == max_percent,
        }))
    }

    pub(super) fn capture_release_gap(
        &mut self,
        snapshot: &mut DamageSourceSnapshot,
        now: u64,
    ) -> Result<(), GameError> {
        if !self.game_data.equipment_attack_catalog().has_release_gap
            || snapshot.release_gap.is_some()
            || snapshot.source != DamageSource::BasicAttack
            || snapshot.basic_primary.is_none()
            || snapshot.fact_source.reaction
            || snapshot.fact_source.guard_release().is_some()
            || snapshot
                .fact_source
                .cast
                .as_ref()
                .is_some_and(|c| c.proc_origin.is_some())
        {
            return Ok(());
        }
        let Some(clock) = self.selected_release_gap(snapshot.source_id, now)? else {
            return Ok(());
        };
        runtime_control::admit_retained_growth(
            0,
            (size_of::<ReleaseGapCapture>() + 64).checked_add(clock.program.retained_bytes),
            "release_gap_capture",
        )?;
        let capture = Arc::new(ReleaseGapCapture {
            equipment: clock.equipment,
            program: Arc::clone(&clock.program),
            since: clock.since,
            released_at: now,
            percent: clock.percent(now),
        });
        if self.authoring_observation.is_some() {
            let source = &capture.program.release_gap_source;
            let bytes = size_of::<ReleaseGapWitness>()
                + source.source_file.len()
                + source.field_path.len()
                + source.site.action.definition_id.as_str().len();
            runtime_control::admit_retained_growth(0, Some(bytes), "release_gap_witness_copy")?;
            control::charge(bytes as u64, "release_gap_witness_copy")?;
            let witness = ReleaseGapWitness {
                generation: self.content_identity.clone(),
                actor: snapshot.source_id,
                time_ms: now,
                since_ms: capture.since,
                elapsed_ms: now.saturating_sub(capture.since),
                percent: capture.percent,
                max_percent: capture.program.release_gap.as_ref().unwrap().max_percent,
                equipment: capture.equipment,
                definition_id: source.site.action.definition_id.to_string(),
                source_file: source.source_file.clone(),
                field_path: source.field_path.clone(),
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "release_gap_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_release_gap = Some(witness);
        }
        control::charge(
            self.equipment_attack_state.release_gaps.len() as u64,
            "release_gap_reset",
        )?;
        for clock in self
            .equipment_attack_state
            .release_gaps
            .values_mut()
            .filter(|c| c.owner == snapshot.source_id)
        {
            clock.since = now;
        }
        snapshot.release_gap = Some(capture);
        control::poll("release_gap_captured")
    }
}
