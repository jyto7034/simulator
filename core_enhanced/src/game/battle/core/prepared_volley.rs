//! Copy-owned preparation and serial projectiles; no content-name dispatch.
mod execution;
mod projectile;
#[cfg(test)]
mod tests;
use super::movement::types::WorldVec2;
use super::BattleCore;
use crate::{
    authoring::{clone_shape::controlled_clone, control},
    game::{
        ability::{marks::MarkRatio, prepared_volley::*, SkillDef, SkillEffectDef, SkillId},
        battle::{
            damage::DamageSourceSnapshot,
            enums::BattleEvent,
            ids::UnitInstanceId,
            runtime_facts::{ActionHandle, RuntimeFactSource},
        },
        behavior::GameError,
        credit_ledger::{CreditCarry, CreditLedgers, CreditSettlement},
        enums::Side,
        skill_fragment::FragmentSourceSnapshot,
    },
};
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;
#[derive(Debug, Clone)]
struct Active {
    channel_started_at: u64,
    id: u64,
    caster: UnitInstanceId,
    source: RuntimeFactSource,
    phase: VolleyPhase,
    prepared: u32,
    created: u32,
    deadline: u64,
    next_at: Option<u64>,
    next_cost: Option<u64>,
    flight: Option<u64>,
    hits: BTreeMap<UnitInstanceId, u32>,
}
crate::authoring::clone_shape::struct_shape!(Active {
    channel_started_at,
    id,
    caster,
    source,
    phase,
    prepared,
    created,
    deadline,
    next_at,
    next_cost,
    flight,
    hits
});
#[derive(Debug, Clone)]
struct Owner {
    employee: Uuid,
    caster: Option<UnitInstanceId>,
    fragment: Arc<FragmentSourceSnapshot>,
    program: Arc<PreparedVolleyDef>,
    program_bytes: usize,
    applied: BTreeMap<String, Uuid>,
    before: Option<CreditCarry>,
    balance: i64,
    can_borrow: bool,
    borrowed: bool,
    active: Option<Active>,
}
crate::authoring::clone_shape::struct_shape!(Owner {
    employee,
    caster,
    fragment,
    program,
    program_bytes,
    applied,
    before,
    balance,
    can_borrow,
    borrowed,
    active
});
#[derive(Debug, Clone)]
struct Flight {
    id: u64,
    activation: u64,
    copy: Uuid,
    caster: UnitInstanceId,
    target: UnitInstanceId,
    source: DamageSourceSnapshot,
    program: Arc<PreparedVolleyDef>,
    program_bytes: usize,
    applied: Arc<BTreeMap<String, Uuid>>,
    position: WorldVec2,
    released: u64,
    last: u64,
    traveled: f32,
    hits: Vec<(UnitInstanceId, u64)>,
    counts: BTreeMap<UnitInstanceId, u32>,
}
crate::authoring::clone_shape::struct_shape!(Flight {
    id,
    activation,
    copy,
    caster,
    target,
    source,
    program,
    program_bytes,
    applied,
    position,
    released,
    last,
    traveled,
    hits,
    counts
});
#[derive(Debug, Clone, Default)]
pub(super) struct PreparedVolleyState {
    next: u64,
    owners: BTreeMap<Uuid, Owner>,
    marks: BTreeMap<(Uuid, UnitInstanceId), u64>,
    flights: BTreeMap<u64, Flight>,
    next_flight: Option<u64>,
    processing: Option<RuntimeFactSource>,
    settled: BTreeMap<u64, VolleyFlightResult>,
    settlements: Vec<CreditSettlement>,
}
crate::authoring::clone_shape::struct_shape!(PreparedVolleyState {
    next,
    owners,
    marks,
    flights,
    next_flight,
    processing,
    settled,
    settlements
});
impl PreparedVolleyState {
    pub(super) fn entity_count(&self) -> usize {
        self.owners.len()
            + self.marks.len()
            + self.flights.len()
            + self.settled.len()
            + self.settlements.len()
            + self
                .owners
                .values()
                .map(|o| o.active.as_ref().map_or(0, |a| 1 + a.hits.len()))
                .sum::<usize>()
    }
    pub(super) fn has_use_effect(
        &self,
        action: &ActionHandle,
        owns: impl Fn(&RuntimeFactSource) -> bool,
    ) -> bool {
        self.owners
            .values()
            .filter_map(|o| o.active.as_ref())
            .any(|a| a.source.action.as_ref() == Some(action) && owns(&a.source))
            || self.flights.values().any(|f| {
                f.source.fact_source.action.as_ref() == Some(action) && owns(&f.source.fact_source)
            })
            || self
                .processing
                .as_ref()
                .is_some_and(|s| s.action.as_ref() == Some(action) && owns(s))
    }
    pub(super) fn retained_extent(&self) -> Option<usize> {
        let mut n = self.entity_count().checked_mul(128)?;
        for o in self.owners.values() {
            n = n
                .checked_add(std::mem::size_of::<Owner>())?
                .checked_add(o.program_bytes)?
                .checked_add(o.fragment.retained_extent()?)?;
            for k in o.applied.keys() {
                n = n.checked_add(k.capacity() + 96)?;
            }
            if let Some(b) = &o.before {
                n = n.checked_add(b.definition.capacity())?;
            }
            if let Some(a) = &o.active {
                n = n
                    .checked_add(std::mem::size_of::<Active>())?
                    .checked_add(a.source.retained_extent()?)?
                    .checked_add(a.hits.len().checked_mul(96)?)?;
            }
        }
        for f in self.flights.values() {
            n = n
                .checked_add(std::mem::size_of::<Flight>())?
                .checked_add(f.program_bytes)?
                .checked_add(f.source.fact_retained_extent()?)?
                .checked_add(
                    f.hits
                        .capacity()
                        .checked_mul(std::mem::size_of::<(UnitInstanceId, u64)>())?,
                )?
                .checked_add(f.counts.len().checked_mul(96)?)?;
            for k in f.applied.keys() {
                n = n.checked_add(k.capacity() + 96)?;
            }
        }
        for r in self.settled.values() {
            n = n
                .checked_add(std::mem::size_of::<VolleyFlightResult>())?
                .checked_add(
                    r.hits
                        .capacity()
                        .checked_mul(std::mem::size_of::<(UnitInstanceId, u64)>())?,
                )?;
        }
        for r in &self.settlements {
            n = n
                .checked_add(std::mem::size_of::<CreditSettlement>() + 128)?
                .checked_add(r.after.definition.capacity())?
                .checked_add(r.before.as_ref().map_or(0, |v| v.definition.capacity()))?;
        }
        Some(n)
    }
}
fn program_extent(p: &PreparedVolleyDef) -> Option<usize> {
    use crate::game::relic::structural::support::pattern_extent;
    let mut n = std::mem::size_of::<PreparedVolleyDef>()
        .checked_add(256)?
        .checked_add(
            p.definition.capacity()
                + p.ledger.definition.capacity()
                + p.mark_definition.capacity()
                + p.projectile.definition.capacity(),
        )?
        .checked_add(pattern_extent(&p.selection_range)?)?
        .checked_add(pattern_extent(&p.projectile.area)?)?
        .checked_add(
            p.modifiers
                .capacity()
                .checked_mul(std::mem::size_of::<VolleyModifier>())?,
        )?
        .checked_add(
            p.projectile
                .after
                .capacity()
                .checked_mul(std::mem::size_of::<
                    crate::game::ability::relation_action::PlanStatus,
                >())?,
        )?;
    for s in &p.projectile.after {
        n = n.checked_add(s.definition.capacity() + 64)?;
    }
    for m in &p.modifiers {
        n = n.checked_add(m.id.capacity() + m.relic_definition.capacity() + 128)?;
        match &m.change {
            VolleyChange::Area { pattern } => n = n.checked_add(pattern_extent(pattern)?)?,
            VolleyChange::Status { status } => {
                n = n.checked_add(status.definition.capacity() + 64)?
            }
            _ => (),
        }
    }
    Some(n)
}
fn ratio(v: MarkRatio) -> f32 {
    v.numerator as f64 as f32 / v.denominator as f64 as f32
}
fn continuous(p: &PreparedVolleyDef) -> Option<(MarkRatio, u32, u32)> {
    p.modifiers.iter().find_map(|m| {
        if let VolleyChange::Continuous {
            percent,
            cost_interval_ms,
            interval_ms,
        } = m.change
        {
            Some((percent, cost_interval_ms, interval_ms))
        } else {
            None
        }
    })
}
impl BattleCore {
    pub(super) fn prepared_volley_declaration(skill: &SkillDef) -> Option<&Arc<PreparedVolleyDef>> {
        skill.steps.iter().flat_map(|s| &s.effects).find_map(|e| {
            if let SkillEffectDef::PreparedVolley { program } = &e.op {
                Some(program)
            } else {
                None
            }
        })
    }
    pub(super) fn prepared_volley_skill(skill: &SkillDef) -> bool {
        Self::prepared_volley_declaration(skill).is_some()
    }
    fn volley_key(&self, caster: UnitInstanceId) -> Option<Uuid> {
        self.units
            .get(&caster)?
            .fragment_source
            .as_ref()
            .map(|f| f.copy_uuid)
    }
    fn next_volley_id(&mut self) -> Result<u64, GameError> {
        let next = self
            .sustained
            .volleys
            .next
            .checked_add(1)
            .ok_or(GameError::InvalidAction)?;
        self.sustained.volleys.next = next;
        Ok(next)
    }
    pub(crate) fn register_prepared_volley_copy(
        &mut self,
        employee: Uuid,
        fragment: Arc<FragmentSourceSnapshot>,
        carry: Option<&CreditCarry>,
    ) -> Result<(), GameError> {
        if self.sustained.closed
            || self
                .sustained
                .volleys
                .owners
                .contains_key(&fragment.copy_uuid)
        {
            return Ok(());
        }
        let Some(def) = self
            .game_data
            .skill_data
            .get_by_id(&fragment.skill_id)
            .and_then(Self::prepared_volley_declaration)
        else {
            return Ok(());
        };
        def.validate().map_err(GameError::InvalidStaticData)?;
        if carry.is_some_and(|c| {
            c.definition != def.ledger.definition || c.unpaid > def.ledger.credit_limit
        }) {
            return Err(GameError::InvalidAction);
        }
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            program_extent(def),
            "prepared_volley_program",
        )?;
        let mut p = controlled_clone(def.as_ref())?;
        let mut applied = BTreeMap::new();
        p.modifiers.retain(|m| {
            if let Some(copy) =
                self.equipped_employee_normal_relic_copy(employee, &m.relic_definition)
            {
                applied.insert(m.id.clone(), copy);
                true
            } else {
                false
            }
        });
        let mut periods = Vec::new();
        let mut gain = MarkRatio {
            numerator: 1,
            denominator: 1,
        };
        for m in &p.modifiers {
            match &m.change {
                VolleyChange::MarkDuration { negative, percent } => {
                    periods.push((*negative, *percent))
                }
                VolleyChange::Area { pattern } => p.projectile.area = controlled_clone(pattern)?,
                VolleyChange::Recovery { factor } => {
                    gain = super::marks::ratio_product(gain, *factor)?
                }
                _ => (),
            }
        }
        p.mark_duration_ms = super::marks::modified_period(p.mark_duration_ms, &periods)?;
        p.recovery = u32::try_from(
            u128::from(p.recovery) * u128::from(gain.numerator) / u128::from(gain.denominator),
        )
        .map_err(|_| GameError::InvalidAction)?;
        p.validate().map_err(GameError::InvalidStaticData)?;
        let bytes = program_extent(&p).ok_or(GameError::InvalidAction)?;
        self.admit_authoring_runtime_growth(1, "prepared_volley_owner")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(
                bytes
                    + std::mem::size_of::<Owner>()
                    + 256
                    + applied.keys().map(|k| k.len() + 96).sum::<usize>(),
            ),
            "prepared_volley_owner_storage",
        )?;
        let before = carry.map(controlled_clone).transpose()?;
        let balance = i64::from(p.ledger.grant) - i64::from(carry.map_or(0, |c| c.unpaid));
        self.sustained.volleys.owners.insert(
            fragment.copy_uuid,
            Owner {
                employee,
                caster: None,
                fragment,
                program: p.into(),
                program_bytes: bytes,
                applied,
                before,
                balance,
                can_borrow: carry.is_none_or(|c| !c.borrowed),
                borrowed: false,
                active: None,
            },
        );
        Ok(())
    }
    pub(crate) fn mount_credit_ledgers(&mut self, ledger: &CreditLedgers) -> Result<(), GameError> {
        for (key, o) in &mut self.sustained.volleys.owners {
            if o.active.is_some() {
                return Err(GameError::InvalidAction);
            }
            let c = ledger.entries.get(key);
            if c.is_some_and(|c| {
                c.definition != o.program.ledger.definition
                    || c.unpaid > o.program.ledger.credit_limit
            }) {
                return Err(GameError::InvalidAction);
            }
            o.before = c.map(controlled_clone).transpose()?;
            o.balance = i64::from(o.program.ledger.grant) - i64::from(c.map_or(0, |c| c.unpaid));
            o.can_borrow = c.is_none_or(|c| !c.borrowed);
            o.borrowed = false;
        }
        Ok(())
    }
    pub(super) fn reconcile_prepared_volleys(&mut self, now: u64) -> Result<(), GameError> {
        if self.sustained.closed {
            return Ok(());
        }
        control::charge(
            self.units.len() as u64 + self.sustained.volleys.entity_count() as u64,
            "prepared_volley_reconcile",
        )?;
        let candidates: Vec<_> = self
            .units
            .values()
            .filter(|u| u.is_active() && u.owner == Side::Player && !u.corroded)
            .filter_map(|u| {
                u.fragment_source
                    .as_ref()
                    .map(|f| (u.instance_id, u.source_owned_uuid, f.clone()))
            })
            .collect();
        for (id, employee, fragment) in candidates {
            let key = fragment.copy_uuid;
            self.register_prepared_volley_copy(employee, fragment, None)?;
            if let Some(o) = self.sustained.volleys.owners.get_mut(&key) {
                if o.employee == employee {
                    o.caster = Some(id);
                }
            }
        }
        let keys: Vec<_> = self.sustained.volleys.owners.keys().copied().collect();
        for key in keys {
            let o = &self.sustained.volleys.owners[&key];
            let live = o.caster.is_some_and(|id| {
                self.units.get(&id).is_some_and(|u| {
                    u.is_active()
                        && u.owner == Side::Player
                        && !u.corroded
                        && u.source_owned_uuid == o.employee
                        && u.fragment_source.as_ref().is_some_and(|f| {
                            f.copy_uuid == key && f.skill_id == o.fragment.skill_id
                        })
                })
            });
            let interrupted = live
                && o.active.is_some()
                && !o
                    .program
                    .modifiers
                    .iter()
                    .any(|m| matches!(m.change, VolleyChange::Protection))
                && self
                    .unit_control_state(o.caster.unwrap(), now)
                    .is_some_and(|s| s.hard_cc_active());
            if !live || interrupted {
                self.finish_prepared_volley(key, now, false, false)?;
            }
            if !live {
                self.sustained.volleys.owners.get_mut(&key).unwrap().caster = None;
            }
        }
        self.sustained.volleys.marks.retain(|(_, target), expires| {
            now < *expires && self.units.get(target).is_some_and(|u| u.is_active())
        });
        Ok(())
    }
    pub(super) fn prepared_volley_started_at(&self, caster: UnitInstanceId) -> Option<u64> {
        let active = self.sustained.volleys.owners.get(&self.volley_key(caster)?)?.active.as_ref()?;
        Some(active.channel_started_at)
    }
    pub(super) fn prepared_volley_active(&self, caster: UnitInstanceId) -> bool {
        self.volley_key(caster)
            .and_then(|k| self.sustained.volleys.owners.get(&k))
            .is_some_and(|o| o.active.as_ref().is_some_and(|a| a.caster == caster))
    }
    pub(super) fn prepared_volley_reinput(
        &self,
        caster: UnitInstanceId,
        skill: &SkillId,
    ) -> Option<u64> {
        let o = self
            .sustained
            .volleys
            .owners
            .get(&self.volley_key(caster)?)?;
        if &o.fragment.skill_id != skill {
            return None;
        }
        o.active.as_ref().map(|a| a.id)
    }
    pub(super) fn volley_damage_mark(
        &mut self,
        source: &DamageSourceSnapshot,
        target: UnitInstanceId,
        actual: u32,
        now: u64,
    ) -> Result<(), GameError> {
        if actual == 0
            || source.source_side != Side::Player
            || !self
                .units
                .get(&target)
                .is_some_and(|u| u.owner == Side::Opponent)
        {
            return Ok(());
        }
        let Some(employee) = self
            .units
            .get(&source.source_id)
            .map(|u| u.source_owned_uuid)
        else {
            return Ok(());
        };
        let keys: Vec<_> = self
            .sustained
            .volleys
            .owners
            .iter()
            .filter(|(_, o)| o.employee == employee)
            .map(|(k, o)| (*k, o.program.mark_duration_ms))
            .collect();
        for (key, duration) in keys {
            let expires = now
                .checked_add(duration.into())
                .ok_or(GameError::InvalidAction)?;
            self.admit_authoring_runtime_growth(1, "volley_recovery_mark")?;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(128),
                "volley_mark_storage",
            )?;
            self.sustained.volleys.marks.insert((key, target), expires);
        }
        control::poll("volley_damage_mark")
    }
    pub(super) fn volley_target_died(
        &mut self,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        let keys: Vec<_> = self
            .sustained
            .volleys
            .marks
            .keys()
            .filter(|(_, t)| *t == target)
            .copied()
            .collect();
        for key in keys {
            let expires = self.sustained.volleys.marks.remove(&key).unwrap();
            if now < expires {
                if let Some(o) = self.sustained.volleys.owners.get_mut(&key.0) {
                    o.balance = o
                        .balance
                        .checked_add(i64::from(o.program.recovery))
                        .ok_or(GameError::InvalidAction)?;
                }
            }
        }
        control::poll("volley_recovery_settled")
    }
    pub(crate) fn credit_settlements(&self) -> Result<Vec<CreditSettlement>, GameError> {
        controlled_clone(&self.sustained.volleys.settlements)
    }
    pub fn prepared_volleys_snapshot(&self) -> Result<PreparedVolleysSnapshot, GameError> {
        control::charge(
            self.sustained.volleys.entity_count() as u64,
            "prepared_volley_snapshot",
        )?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.sustained.volleys.retained_extent(),
            "prepared_volley_snapshot_storage",
        )?;
        let state = &self.sustained.volleys;
        Ok(PreparedVolleysSnapshot {
            owners: state
                .owners
                .iter()
                .map(|(copy, o)| PreparedVolleyOwnerView {
                    copy: *copy,
                    employee: o.employee,
                    caster: o.caster,
                    balance: o.balance,
                    can_borrow: o.can_borrow,
                    borrowed: o.borrowed,
                    activation: o.active.as_ref().map(|a| a.id),
                    phase: o.active.as_ref().map(|a| a.phase),
                    prepared: o.active.as_ref().map_or(0, |a| a.prepared),
                    next_at_ms: o.active.as_ref().and_then(|a| a.next_at),
                    next_cost_ms: o.active.as_ref().and_then(|a| a.next_cost),
                    deadline_ms: o
                        .active
                        .as_ref()
                        .filter(|a| {
                            matches!(a.phase, VolleyPhase::Preparing | VolleyPhase::Decaying)
                        })
                        .map(|a| a.deadline),
                    flight: o.active.as_ref().and_then(|a| a.flight),
                })
                .collect(),
            flights: state.flights.keys().copied().collect(),
            settled: state
                .settled
                .values()
                .map(|r| {
                    control::charge(r.hits.len() as u64, "volley_result_copy")?;
                    Ok(r.clone())
                })
                .collect::<Result<_, GameError>>()?,
            marks: state.marks.iter().map(|((k, t), v)| (*k, *t, *v)).collect(),
        })
    }
}
