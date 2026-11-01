//! Battle-copy resources and a paused, single-commit volley preparation.
mod observation;
mod projectile;
#[cfg(test)]
mod tests;
use super::BattleCore;
use crate::{
    authoring::{clone_shape::controlled_clone, control},
    game::{
        ability::{input_plan::*, marks::MarkRatio, SkillDef, SkillEffectDef, SkillId},
        battle::{
            core::movement::types::WorldVec2,
            damage::DamageSourceSnapshot,
            enums::BattleEvent,
            ids::UnitInstanceId,
            runtime_facts::{ActionHandle, RuntimeFactSource},
            types::BattleUnitSourceIdentity,
        },
        behavior::GameError,
        enums::Side,
        resources::Position,
        skill_fragment::FragmentSourceSnapshot,
    },
};
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;
#[derive(Debug, Clone)]
struct Owner {
    employee: Uuid,
    caster: Option<UnitInstanceId>,
    fragment: Arc<FragmentSourceSnapshot>,
    program: Arc<InputPlanDef>,
    normal: u32,
    enhanced: u32,
    next_stock: u64,
    memory: Vec<ShotPlacement>,
    applied: BTreeMap<String, Uuid>,
}
crate::authoring::clone_shape::struct_shape!(Owner {
    employee,
    caster,
    fragment,
    program,
    normal,
    enhanced,
    next_stock,
    memory,
    applied
});
#[derive(Debug, Clone)]
struct Preparation {
    id: u64,
    copy: Uuid,
    caster: UnitInstanceId,
    started: u64,
    normal: u32,
    enhanced: u32,
}
crate::authoring::clone_shape::struct_shape!(Preparation {
    id,
    copy,
    caster,
    started,
    normal,
    enhanced
});
#[derive(Debug, Clone)]
struct PendingVolley {
    preparation: Preparation,
    request: u64,
    layout: Vec<ShotPlacement>,
    directions: Vec<WorldVec2>,
    action: Option<ActionHandle>,
}
crate::authoring::clone_shape::struct_shape!(PendingVolley {
    preparation,
    request,
    layout,
    directions,
    action
});
#[derive(Debug, Clone)]
struct Flight {
    id: u64,
    batch: u64,
    copy: Uuid,
    caster: UnitInstanceId,
    kind: ShotKind,
    program: Arc<InputPlanDef>,
    applied: Arc<BTreeMap<String, Uuid>>,
    source: DamageSourceSnapshot,
    start: WorldVec2,
    aim: WorldVec2,
    direction: WorldVec2,
    released: u64,
    duration: u64,
    last: Option<u64>,
    hits: Vec<UnitInstanceId>,
    actual_hp_loss: u64,
    end: ShotEnd,
    hit_results: Vec<ShotHitResult>,
}
crate::authoring::clone_shape::struct_shape!(Flight {
    id,
    batch,
    copy,
    caster,
    kind,
    program,
    applied,
    source,
    start,
    aim,
    direction,
    released,
    duration,
    last,
    hits,
    actual_hp_loss,
    end,
    hit_results
});
#[derive(Debug, Clone)]
struct Wound {
    count: u32,
    expires: u64,
}
crate::authoring::clone_shape::struct_shape!(Wound { count, expires });
#[derive(Debug, Clone, Default)]
pub(super) struct InputPlanState {
    next: u64,
    owners: BTreeMap<Uuid, Owner>,
    preparation: Option<Preparation>,
    disconnected: bool,
    pending: Option<PendingVolley>,
    receipts: BTreeMap<u64, ShotCommitReceipt>,
    flights: BTreeMap<u64, Flight>,
    next_flight_tick: Option<u64>,
    processing: Option<RuntimeFactSource>,
    settled: BTreeMap<u64, ShotFlightResult>,
    wounds: BTreeMap<(Uuid, String, UnitInstanceId), Wound>,
}
crate::authoring::clone_shape::struct_shape!(InputPlanState {
    next,
    owners,
    preparation,
    disconnected,
    pending,
    receipts,
    flights,
    next_flight_tick,
    processing,
    settled,
    wounds
});
impl InputPlanState {
    pub(super) fn entity_count(&self) -> usize {
        self.owners.len()
            + self.settled.len()
            + self.receipts.len()
            + self.flights.len()
            + self.wounds.len()
            + usize::from(self.preparation.is_some())
            + usize::from(self.pending.is_some())
    }
    pub(super) fn has_use_effect(
        &self,
        action: &ActionHandle,
        owns: impl Fn(&RuntimeFactSource) -> bool,
    ) -> bool {
        self.processing
            .as_ref()
            .is_some_and(|source| source.action.as_ref() == Some(action) && owns(source))
            || self.flights.values().any(|f| {
                f.source.fact_source.action.as_ref() == Some(action) && owns(&f.source.fact_source)
            })
    }
    pub(super) fn retained_extent(&self) -> Option<usize> {
        let mut n = self.entity_count().checked_mul(128)?;
        for o in self.owners.values() {
            n = n
                .checked_add(std::mem::size_of::<Owner>())?
                .checked_add(o.fragment.retained_extent()?)?
                .checked_add(program_extent(&o.program)?)?
                .checked_add(
                    o.memory
                        .capacity()
                        .checked_mul(std::mem::size_of::<ShotPlacement>())?,
                )?;
            for s in o.applied.keys() {
                n = n.checked_add(s.capacity() + 96)?;
            }
        }
        for f in self.flights.values() {
            n = n
                .checked_add(std::mem::size_of::<Flight>())?
                .checked_add(f.source.fact_retained_extent()?)?
                .checked_add(program_extent(&f.program)?)?
                .checked_add(
                    f.hits
                        .capacity()
                        .checked_mul(std::mem::size_of::<UnitInstanceId>())?,
                )?
                .checked_add(
                    f.hit_results
                        .capacity()
                        .checked_mul(std::mem::size_of::<ShotHitResult>())?,
                )?;
            for hit in &f.hit_results {
                n = n.checked_add(hit_status_extent(hit)?)?;
            }
            for s in f.applied.keys() {
                n = n.checked_add(s.capacity() + 96)?;
            }
        }
        for s in self.settled.values() {
            n = n
                .checked_add(std::mem::size_of::<ShotFlightResult>())?
                .checked_add(
                    s.hits
                        .capacity()
                        .checked_mul(std::mem::size_of::<ShotHitResult>())?,
                )?;
        }
        for result in self.settled.values() {
            for hit in &result.hits {
                n = n.checked_add(hit_status_extent(hit)?)?;
            }
        }
        for (_, s, _) in self.wounds.keys() {
            n = n.checked_add(s.capacity())?;
        }
        if let Some(p) = &self.pending {
            n = n
                .checked_add(
                    p.layout
                        .capacity()
                        .checked_mul(std::mem::size_of::<ShotPlacement>())?,
                )?
                .checked_add(
                    p.directions
                        .capacity()
                        .checked_mul(std::mem::size_of::<WorldVec2>())?,
                )?;
        }
        if let Some(source) = &self.processing {
            n = n.checked_add(source.retained_extent()?)?;
        }
        Some(n)
    }
}
fn hit_status_extent(hit: &ShotHitResult) -> Option<usize> {
    let mut n = hit
        .statuses
        .capacity()
        .checked_mul(std::mem::size_of::<ShotStatusApplication>())?;
    for status in &hit.statuses {
        n = n.checked_add(status.definition.capacity())?;
    }
    Some(n)
}
fn program_extent(p: &InputPlanDef) -> Option<usize> {
    use crate::game::relic::structural::support::pattern_extent;
    let mut n = std::mem::size_of::<InputPlanDef>().checked_add(256)?;
    for s in [
        &p.definition,
        &p.normal.definition,
        &p.enhanced.definition,
        &p.cohort.definition,
        &p.cohort.weakness_family,
        &p.normal_projectile.definition,
        &p.enhanced_projectile.definition,
    ] {
        n = n.checked_add(s.capacity() + 64)?;
    }
    n = n
        .checked_add(pattern_extent(&p.placement_range)?)?
        .checked_add(pattern_extent(&p.cohort.attack_range)?)?;
    for projectile in [&p.normal_projectile, &p.enhanced_projectile] {
        n = n.checked_add(
            projectile
                .after
                .capacity()
                .checked_mul(std::mem::size_of::<
                    crate::game::ability::relation_action::PlanStatus,
                >())?,
        )?;
        for s in &projectile.after {
            n = n.checked_add(s.definition.capacity() + 64)?;
        }
    }
    n = n.checked_add(
        p.modifiers
            .capacity()
            .checked_mul(std::mem::size_of::<ShotModifier>())?,
    )?;
    for m in &p.modifiers {
        n = n
            .checked_add(m.id.capacity() + m.relic_definition.capacity() + 128)?
            .checked_add(
                m.requires
                    .capacity()
                    .checked_mul(std::mem::size_of::<String>())?,
            )?;
        for s in &m.requires {
            n = n.checked_add(s.capacity() + 64)?;
        }
        match &m.change {
            ShotChange::Wound { profiles, .. } => {
                n = n.checked_add(
                    profiles
                        .capacity()
                        .checked_mul(std::mem::size_of::<String>())?,
                )?;
                for s in profiles {
                    n = n.checked_add(s.capacity() + 64)?;
                }
            }
            ShotChange::WoundGrant { definition, .. }
            | ShotChange::ControlVulnerability { definition, .. } => {
                n = n.checked_add(definition.capacity() + 64)?
            }
            ShotChange::Status { profile, status } => {
                n = n.checked_add(profile.capacity() + status.definition.capacity() + 128)?
            }
            ShotChange::Push { profile, .. } => n = n.checked_add(profile.capacity() + 64)?,
            ShotChange::Explosion { profile, range, .. } => {
                n = n
                    .checked_add(profile.capacity() + 64)?
                    .checked_add(pattern_extent(range)?)?
            }
            _ => (),
        }
    }
    Some(n)
}
fn ratio_value(r: MarkRatio) -> f32 {
    r.numerator as f32 / r.denominator as f32
}
fn scale_floor(n: u32, r: MarkRatio) -> Result<u32, GameError> {
    u32::try_from(u128::from(n) * u128::from(r.numerator) / u128::from(r.denominator))
        .map_err(|_| GameError::InvalidAction)
}
impl BattleCore {
    pub(super) fn input_plan_declaration(skill: &SkillDef) -> Option<&Arc<InputPlanDef>> {
        skill
            .steps
            .iter()
            .flat_map(|s| &s.effects)
            .find_map(|e| match &e.op {
                SkillEffectDef::InputPlan { program } => Some(program),
                _ => None,
            })
    }
    pub(super) fn input_plan_commit_pending(&self, caster: UnitInstanceId) -> bool {
        self.sustained
            .input_plans
            .pending
            .as_ref()
            .is_some_and(|p| p.preparation.caster == caster)
    }
    pub(super) fn input_plan_skill(skill: &SkillDef) -> bool {
        Self::input_plan_declaration(skill).is_some()
    }
    pub(super) fn input_plan_reprepare_allowed(&self) -> bool {
        self.sustained.input_plans.disconnected && self.sustained.input_plans.preparation.is_none()
    }
    pub fn input_plan_paused(&self) -> bool {
        self.sustained.input_plans.preparation.is_some() || self.sustained.input_plans.disconnected
    }
    pub(super) fn next_shot_id(&mut self) -> Result<u64, GameError> {
        let n = self
            .sustained
            .input_plans
            .next
            .checked_add(1)
            .ok_or(GameError::InvalidAction)?;
        self.sustained.input_plans.next = n;
        Ok(n)
    }
    fn input_plan_key(&self, caster: UnitInstanceId) -> Option<Uuid> {
        self.units
            .get(&caster)?
            .fragment_source
            .as_ref()
            .map(|f| f.copy_uuid)
    }
    pub(crate) fn register_input_plan_copy(
        &mut self,
        employee: Uuid,
        fragment: Arc<FragmentSourceSnapshot>,
        now: u64,
    ) -> Result<(), GameError> {
        if self.sustained.closed
            || self
                .sustained
                .input_plans
                .owners
                .contains_key(&fragment.copy_uuid)
        {
            return Ok(());
        }
        let Some(def) = self
            .game_data
            .skill_data
            .get_by_id(&fragment.skill_id)
            .and_then(Self::input_plan_declaration)
        else {
            return Ok(());
        };
        def.validate().map_err(GameError::InvalidStaticData)?;
        self.input_plan_routes()?;
        let mut program = controlled_clone(def.as_ref())?;
        let mut applied = BTreeMap::new();
        program.modifiers.retain(|m| {
            if let Some(copy) = self
                .equipped_employee_normal_relic_copy(employee, &m.relic_definition)
                .filter(|_| {
                    m.requires.iter().all(|r| {
                        self.equipped_employee_normal_relic_copy(employee, r)
                            .is_some()
                    })
                })
            {
                applied.insert(m.id.clone(), copy);
                true
            } else {
                false
            }
        });
        // The original declaration is validated before inactive conditional rules
        // are removed. A dependent rule always checks its explicit prerequisites.
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            program_extent(&program)
                .and_then(|n| n.checked_add(std::mem::size_of::<Owner>() + 256)),
            "input_plan_owner",
        )?;
        self.admit_authoring_runtime_growth(1, "input_plan_owner_entity")?;
        let copy = fragment.copy_uuid;
        let next_stock = now
            .checked_add(u64::from(program.stock_interval_ms))
            .ok_or(GameError::InvalidAction)?;
        self.register_shared_cohort(employee, crate::game::ability::hostile_cohort::HostileCohortProvider::Fragment { copy }, &program.cohort, now)?;
        self.sustained.input_plans.owners.insert(
            copy,
            Owner {
                employee,
                caster: None,
                fragment,
                normal: program.normal.initial,
                enhanced: program.enhanced.initial,
                program: Arc::new(program),
                next_stock,
                memory: Vec::new(),
                applied,
            },
        );
        self.event_queue.push(BattleEvent::ShotStockTick {
            source: copy,
            time_ms: next_stock,
        });
        Ok(())
    }
    pub(super) fn credit_input_plan_cohort_kill(&mut self, copy: Uuid) -> Result<(), GameError> {
        let Some(o) = self.sustained.input_plans.owners.get_mut(&copy) else { return Ok(()); };
        let mut gain = o.program.cohort.kill_gain;
        for m in &o.program.modifiers {
            if let ShotChange::KillGain { factor } = m.change { gain = scale_floor(gain, factor)?; }
        }
        o.enhanced = add_stock(o.enhanced, gain, o.program.enhanced.capacity)?;
        Ok(())
    }
    pub(super) fn reconcile_input_plans(&mut self, now: u64) -> Result<(), GameError> {
        if self.sustained.closed {
            return Ok(());
        }
        control::charge(
            (self.units.len() + self.sustained.input_plans.entity_count()) as u64,
            "input_plan_reconcile",
        )?;
        for owner in self.sustained.input_plans.owners.values_mut() {
            if owner.caster.is_some_and(|id| {
                !self.units.get(&id).is_some_and(|u| {
                    u.is_active()
                        && !u.corroded
                        && u.owner == Side::Player
                        && u.fragment_source
                            .as_ref()
                            .is_some_and(|f| f.copy_uuid == owner.fragment.copy_uuid)
                })
            }) {
                owner.caster = None;
            }
        }
        let candidates: Vec<_> = self
            .units
            .values()
            .filter(|u| u.is_active() && !u.corroded && u.owner == Side::Player)
            .filter_map(|u| {
                let BattleUnitSourceIdentity::Employee { employee_uuid, .. } = u.source_identity
                else {
                    return None;
                };
                u.fragment_source
                    .as_ref()
                    .map(|f| (u.instance_id, employee_uuid, f.clone()))
            })
            .collect();
        for (caster, employee, fragment) in candidates {
            self.register_input_plan_copy(employee, fragment.clone(), now)?;
            if let Some(o) = self
                .sustained
                .input_plans
                .owners
                .get_mut(&fragment.copy_uuid)
            {
                o.caster = Some(caster);
            }
        }
        if self
            .sustained
            .input_plans
            .preparation
            .as_ref()
            .is_some_and(|p| {
                !self.units.get(&p.caster).is_some_and(|u| {
                    u.is_active()
                        && !u.corroded
                        && u.fragment_source
                            .as_ref()
                            .is_some_and(|f| f.copy_uuid == p.copy)
                })
            })
        {
            self.sustained.input_plans.preparation = None;
        }
        self.sustained
            .input_plans
            .wounds
            .retain(|(_, _, target), w| {
                now < w.expires && self.units.get(target).is_some_and(|u| u.is_active())
            });
        Ok(())
    }
    pub(super) fn shot_stock_tick(&mut self, copy: Uuid, now: u64) -> Result<(), GameError> {
        let Some(o) = self
            .sustained
            .input_plans
            .owners
            .get_mut(&copy)
            .filter(|o| o.next_stock == now)
        else {
            return Ok(());
        };
        let gain = o.program.stock_gain;
        o.normal = add_stock(o.normal, gain, o.program.normal.capacity)?;
        o.next_stock = now
            .checked_add(u64::from(o.program.stock_interval_ms))
            .ok_or(GameError::InvalidAction)?;
        self.event_queue.push(BattleEvent::ShotStockTick {
            source: copy,
            time_ms: o.next_stock,
        });
        control::poll("input_plan_stock_grant")
    }
    pub(super) fn prepare_input_plan(
        &mut self,
        caster: UnitInstanceId,
        skill_id: &SkillId,
        now: u64,
    ) -> Result<u64, GameError> {
        if self.sustained.input_plans.preparation.is_some() || self.sustained_time_is_stale(now) {
            return Err(GameError::InvalidAction);
        }
        self.validate_manual_skill_activation(now, caster, skill_id, None)?;
        self.reconcile_input_plans(now)?;
        let copy = self
            .input_plan_key(caster)
            .ok_or(GameError::InvalidAction)?;
        let o = self
            .sustained
            .input_plans
            .owners
            .get(&copy)
            .ok_or(GameError::InvalidAction)?;
        if o.fragment.skill_id != *skill_id || (o.normal == 0 && o.enhanced == 0) {
            return Err(GameError::InvalidAction);
        }
        let (normal, enhanced) = (o.normal, o.enhanced);
        let id = self.next_shot_id()?;
        self.admit_authoring_runtime_growth(1, "input_plan_preparation")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<Preparation>()),
            "input_plan_preparation_storage",
        )?;
        self.sustained.input_plans.disconnected = false;
        self.sustained.input_plans.preparation = Some(Preparation {
            id,
            copy,
            caster,
            started: now,
            normal,
            enhanced,
        });
        control::poll("input_plan_prepared")?;
        Ok(id)
    }
    fn validate_shot_layout(
        &self,
        p: &Preparation,
        layout: &[ShotPlacement],
    ) -> Result<(), GameError> {
        let o = &self.sustained.input_plans.owners[&p.copy];
        if p.normal != o.normal
            || p.enhanced != o.enhanced
            || layout.len()
                != usize::try_from(u64::from(p.normal) + u64::from(p.enhanced))
                    .map_err(|_| GameError::InvalidAction)?
        {
            return Err(GameError::InvalidAction);
        }
        let unit = self.units.get(&p.caster).ok_or(GameError::UnitNotFound)?;
        let tiles = o
            .program
            .placement_range
            .affected_tiles(
                unit.world_position().project_to_tile(),
                unit.facing_direction
                    .unwrap_or(crate::game::battle::tile_range::FacingDirection::Up),
            )
            .map_err(GameError::InvalidStaticData)?;
        let mut normal = 0u32;
        let mut enhanced = 0u32;
        let mut seen = std::collections::HashSet::new();
        for item in layout {
            control::charge(1, "input_plan_layout_validation")?;
            if !self.projectile_tile_valid(item.tile)
                || !tiles.contains(&item.tile)
                || (!o.program.allow_duplicates && !seen.insert(item.tile))
            {
                return Err(GameError::InvalidAction);
            }
            match item.kind {
                ShotKind::Normal => normal += 1,
                ShotKind::Enhanced => enhanced += 1,
            }
        }
        if (normal, enhanced) != (p.normal, p.enhanced) {
            return Err(GameError::InvalidAction);
        }
        Ok(())
    }
    pub(super) fn commit_input_plan(
        &mut self,
        caster: UnitInstanceId,
        input: ShotCommitInput,
        now: u64,
    ) -> Result<ShotCommitReceipt, GameError> {
        if let Some(receipt) = self.sustained.input_plans.receipts.get(&input.preparation) {
            if receipt.request == input.request && self.input_plan_key(caster) == Some(receipt.copy)
            {
                return Ok(*receipt);
            }
            return Err(GameError::InvalidAction);
        }
        let p = controlled_clone(
            self.sustained
                .input_plans
                .preparation
                .as_ref()
                .ok_or(GameError::InvalidAction)?,
        )?;
        if p.id != input.preparation
            || p.caster != caster
            || p.started != now
            || self.sustained_time_is_stale(now)
        {
            return Err(GameError::InvalidAction);
        }
        let skill = self.sustained.input_plans.owners[&p.copy]
            .fragment
            .skill_id
            .clone();
        self.validate_manual_skill_activation(now, caster, &skill, None)?;
        let layout = match input.layout {
            ShotLayoutInput::Fresh(v) => v,
            ShotLayoutInput::Remembered => {
                controlled_clone(&self.sustained.input_plans.owners[&p.copy].memory)?
            }
        };
        self.validate_shot_layout(&p, &layout)?;
        control::charge(layout.len() as u64, "input_plan_aim_validation")?;
        let directions = match input.aim {
            ShotAimInput::Precise(v) => {
                if v.len() != layout.len() {
                    return Err(GameError::InvalidAction);
                }
                v.into_iter()
                    .map(|d| {
                        let v = WorldVec2::new(d.x as f32, d.y as f32);
                        let len = v.length();
                        if len == 0.0 || !len.is_finite() {
                            Err(GameError::InvalidAction)
                        } else {
                            Ok(v * (1.0 / len))
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?
            }
            ShotAimInput::Focused(tile) => {
                if !self.projectile_tile_valid(tile) {
                    return Err(GameError::InvalidAction);
                }
                layout
                    .iter()
                    .map(|p| {
                        if p.tile == tile {
                            return Err(GameError::InvalidAction);
                        }
                        let v =
                            WorldVec2::from_tile_center(tile) - WorldVec2::from_tile_center(p.tile);
                        Ok(v * (1.0 / v.length()))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            }
        };
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            layout
                .capacity()
                .checked_mul(std::mem::size_of::<ShotPlacement>())
                .and_then(|n| {
                    n.checked_add(directions.capacity() * std::mem::size_of::<WorldVec2>() + 128)
                }),
            "input_plan_pending_storage",
        )?;
        self.sustained.input_plans.pending = Some(PendingVolley {
            preparation: p,
            request: input.request,
            layout,
            directions,
            action: None,
        });
        self.start_manual_skill_cast(now, caster, skill, None)?;
        let pending = self.units[&caster]
            .pending_skill_cast
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        let action = pending.action.clone().ok_or(GameError::InvalidAction)?;
        let start_seq = pending.start_seq;
        self.sustained.input_plans.pending.as_mut().unwrap().action = Some(action.clone());
        // Drain only this immediate accepted action. Other same-time attacks and
        // projectile arrivals remain queued until the entire input commits.
        while !self
            .sustained
            .input_plans
            .receipts
            .contains_key(&input.preparation)
        {
            control::poll("input_plan_atomic_release")?;
            let cast = self.active_skill_casts.iter().find_map(|(seq, c)| {
                (c.fact_source.action.as_ref() == Some(&action)).then_some(*seq)
            });
            let event = self
                .event_queue
                .take_immediate_action_event(&action, caster, start_seq, cast, now)?
                .ok_or(GameError::InvalidAction)?;
            self.process_event(event, now)?;
        }
        let receipt = self.sustained.input_plans.receipts[&input.preparation];
        self.sustained.input_plans.preparation = None;
        self.sustained.input_plans.pending = None;
        control::poll("input_plan_commit_complete")?;
        Ok(receipt)
    }
    pub(super) fn cancel_input_plan(
        &mut self,
        caster: UnitInstanceId,
        id: u64,
        disconnected: bool,
    ) -> Result<(), GameError> {
        let p = self
            .sustained
            .input_plans
            .preparation
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        if p.caster != caster || p.id != id {
            return Err(GameError::InvalidAction);
        }
        self.sustained.input_plans.preparation = None;
        self.sustained.input_plans.pending = None;
        self.sustained.input_plans.disconnected = disconnected;
        Ok(())
    }
    pub(crate) fn input_plan_employee_receipt(
        &self,
        employee: Uuid,
        preparation: u64,
        request: u64,
    ) -> Option<ShotCommitReceipt> {
        self.sustained
            .input_plans
            .receipts
            .get(&preparation)
            .filter(|r| {
                r.request == request
                    && self
                        .sustained
                        .input_plans
                        .owners
                        .get(&r.copy)
                        .is_some_and(|o| o.employee == employee)
            })
            .copied()
    }
    pub(crate) fn input_plan_preparation_id(&self) -> Option<(UnitInstanceId, u64)> {
        self.sustained
            .input_plans
            .preparation
            .as_ref()
            .map(|p| (p.caster, p.id))
    }
    pub(crate) fn resume_disconnected_input_plan(&mut self) {
        self.sustained.input_plans.disconnected = false;
    }
}

fn add_stock(current: u32, gain: u32, capacity: Option<u32>) -> Result<u32, GameError> {
    match capacity {
        Some(cap) => Ok((u64::from(current) + u64::from(gain)).min(u64::from(cap)) as u32),
        None => current.checked_add(gain).ok_or(GameError::InvalidAction),
    }
}
