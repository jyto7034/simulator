//! Copy-owned counters and one admitted, cancellable sequence of finite phases.
//! Natural completion is the only path that awards a cast result.
mod execution;
mod observation;
mod program;
#[cfg(test)]
mod tests;
use super::BattleCore;
use crate::{
    authoring::{clone_shape::controlled_clone, control},
    game::{
        ability::{
            counter_channel::*, marks::AppliedMarkModifier, SkillDef, SkillEffectDef, SkillId,
        },
        battle::{
            damage::DamageSourceSnapshot,
            enums::BattleEvent,
            ids::UnitInstanceId,
            runtime_facts::{ActionHandle, RuntimeFactSource},
            tile_range::FacingDirection,
        },
        behavior::GameError,
        resources::Position,
        skill_fragment::FragmentSourceSnapshot,
    },
};
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;

#[derive(Debug, Clone)]
struct Pending {
    action: ActionHandle,
    entry: usize,
    consumed: u32,
    tile: Position,
    origin: Position,
    facing: FacingDirection,
}
crate::authoring::clone_shape::struct_shape!(Pending {
    action,
    entry,
    consumed,
    tile,
    origin,
    facing
});
#[derive(Debug, Clone)]
struct Active {
    channel_started_at: u64,
    id: u64,
    admitted: Pending,
    snapshot: DamageSourceSnapshot,
    phase: usize,
    phase_end: u64,
    next_at: u64,
    tick: u32,
    damaged: bool,
    healed: bool,
}
crate::authoring::clone_shape::struct_shape!(Active {
    channel_started_at,
    id,
    admitted,
    snapshot,
    phase,
    phase_end,
    next_at,
    tick,
    damaged,
    healed
});
#[derive(Debug, Clone)]
struct Owner {
    caster: UnitInstanceId,
    fragment: Arc<FragmentSourceSnapshot>,
    generation: u64,
    program: Arc<CounterChannelDef>,
    applied: Vec<AppliedMarkModifier>,
    count: u32,
    next_gain: Option<u64>,
    aim: Option<Position>,
    pending: Option<Pending>,
    active: Option<Active>,
}
crate::authoring::clone_shape::struct_shape!(Owner {
    caster,
    fragment,
    generation,
    program,
    applied,
    count,
    next_gain,
    aim,
    pending,
    active
});
#[derive(Debug, Clone, Default)]
pub(super) struct CounterChannelState {
    next: u64,
    owners: BTreeMap<Uuid, Owner>,
}
crate::authoring::clone_shape::struct_shape!(CounterChannelState { next, owners });
impl CounterChannelState {
    pub(super) fn has_use_effect(
        &self,
        action: &ActionHandle,
        owns: impl Fn(&RuntimeFactSource) -> bool,
    ) -> bool {
        self.owners
            .values()
            .filter_map(|o| o.active.as_ref())
            .any(|a| &a.admitted.action == action && owns(&a.snapshot.fact_source))
    }
    pub(super) fn entity_count(&self) -> usize {
        self.owners.len()
            + self
                .owners
                .values()
                .map(|o| usize::from(o.pending.is_some()) + usize::from(o.active.is_some()))
                .sum::<usize>()
    }
    pub(super) fn retained_extent(&self) -> Option<usize> {
        let mut size = self
            .owners
            .len()
            .checked_mul(std::mem::size_of::<Owner>() + 80)?;
        for o in self.owners.values() {
            size = size
                .checked_add(o.fragment.retained_extent()?)?
                .checked_add(program::extent(&o.program)?)?
                .checked_add(
                    o.applied
                        .capacity()
                        .checked_mul(std::mem::size_of::<AppliedMarkModifier>())?,
                )?;
            for a in &o.applied {
                size = size
                    .checked_add(a.rule_id.capacity())?
                    .checked_add(a.relic_definition.capacity())?;
            }
            if let Some(a) = &o.active {
                size = size.checked_add(a.snapshot.fact_retained_extent()?)?;
            }
        }
        Some(size)
    }
}
impl BattleCore {
    fn counter_declaration(skill: &SkillDef) -> Option<(&Arc<CounterChannelDef>, &SkillId, usize)> {
        skill
            .steps
            .iter()
            .flat_map(|s| &s.effects)
            .find_map(|e| match &e.op {
                SkillEffectDef::CounterChannel {
                    program,
                    root,
                    entry,
                } => Some((program, root, *entry)),
                _ => None,
            })
    }
    pub(super) fn counter_channel_skill(skill: &SkillDef) -> bool {
        Self::counter_declaration(skill).is_some()
    }
    pub(super) fn fragment_owns_skill(&self, root: &SkillId, skill: &SkillId) -> bool {
        root == skill
            || self
                .game_data
                .skill_data
                .get_by_id(skill)
                .and_then(Self::counter_declaration)
                .is_some_and(|(program, declared, entry)| {
                    declared == root && program.entry_skill_id(root, entry) == *skill
                })
    }
    pub(super) fn counter_manual_entry(&self, root: &SkillId, requested: &SkillId) -> bool {
        self.game_data
            .skill_data
            .get_by_id(requested)
            .and_then(Self::counter_declaration)
            .is_some_and(|(p, r, i)| r == root && p.entries.get(i).is_some_and(|e| !e.automatic))
    }
    fn counter_key(&self, caster: UnitInstanceId) -> Option<Uuid> {
        self.units
            .get(&caster)?
            .fragment_source
            .as_ref()
            .map(|f| f.copy_uuid)
    }
    fn counter_owner_live(&self, o: &Owner) -> bool {
        self.units.get(&o.caster).is_some_and(|u| {
            u.is_active()
                && !u.corroded
                && u.fragment_source.as_deref() == Some(o.fragment.as_ref())
        })
    }
    fn next_counter_id(&mut self) -> Result<u64, GameError> {
        self.sustained.counters.next = self
            .sustained
            .counters
            .next
            .checked_add(1)
            .ok_or(GameError::InvalidAction)?;
        Ok(self.sustained.counters.next)
    }
    fn schedule_counter_gain(&mut self, key: Uuid, now: u64) -> Result<(), GameError> {
        self.event_queue
            .discard_counter_events(Some(key), true, false)?;
        let o = self
            .sustained
            .counters
            .owners
            .get_mut(&key)
            .ok_or(GameError::InvalidAction)?;
        o.next_gain = if o.count >= o.program.capacity {
            None
        } else {
            Some(
                now.checked_add(u64::from(o.program.interval_ms))
                    .ok_or(GameError::InvalidAction)?,
            )
        };
        if let Some(time_ms) = o.next_gain {
            self.event_queue.push_with_context(
                BattleEvent::CounterGain {
                    time_ms,
                    source: key,
                    generation: o.generation,
                },
                None,
                RuntimeFactSource::default(),
            );
        }
        Ok(())
    }
    pub(super) fn counter_gain(
        &mut self,
        key: Uuid,
        generation: u64,
        now: u64,
    ) -> Result<(), GameError> {
        let Some(o) = self
            .sustained
            .counters
            .owners
            .get_mut(&key)
            .filter(|o| o.generation == generation && o.next_gain == Some(now))
        else {
            return Ok(());
        };
        o.count = o
            .count
            .saturating_add(program::gain(
                &o.program,
                CounterGainTarget::Periodic,
                u64::from(o.program.periodic_gain),
            )?)
            .min(o.program.capacity);
        self.schedule_counter_gain(key, now)
    }
    pub(super) fn validate_counter_start(
        &self,
        caster: UnitInstanceId,
        skill: &SkillDef,
    ) -> Result<(), GameError> {
        use crate::game::behavior::{ActionDisabledReasonCode as Reason, ActionKind};
        let unavailable = |reason| GameError::ActionUnavailable {
            action: ActionKind::ActivateSkill,
            reason,
        };
        let Some((_, root, entry)) = Self::counter_declaration(skill) else {
            return Ok(());
        };
        let o = self
            .counter_key(caster)
            .and_then(|k| self.sustained.counters.owners.get(&k))
            .ok_or_else(|| unavailable(Reason::SkillUnavailable))?;
        let e = o
            .program
            .entries
            .get(entry)
            .ok_or(GameError::InvalidAction)?;
        if &o.fragment.skill_id != root || !self.counter_owner_live(o) {
            return Err(unavailable(Reason::SkillUnavailable));
        }
        if o.pending.is_some()
            || o.active.is_some()
            || (!e.automatic && o.count >= o.program.capacity)
        {
            return Err(unavailable(Reason::ActionLocked));
        }
        if o.count < e.minimum || o.count > e.maximum {
            return Err(unavailable(Reason::InsufficientResources));
        }
        let consumed = if e.spend == CounterSpend::None {
            0
        } else {
            o.count
        };
        // Resolve every admitted amount before either currency is consumed.
        for query in e.phases.iter().flat_map(|p| &p.queries) {
            match &query.effect {
                PhaseEffect::Damage { amount, .. } => {
                    program::amount(&o.program, amount, consumed, false)?;
                }
                PhaseEffect::Heal { amount } => {
                    program::amount(&o.program, amount, consumed, true)?;
                }
                _ => (),
            }
        }
        Ok(())
    }
    pub(super) fn prepare_counter_channel(
        &mut self,
        caster: UnitInstanceId,
        skill: &SkillDef,
        action: &ActionHandle,
        target: Option<crate::game::battle::event_log::SkillCastTarget>,
        now: u64,
    ) -> Result<(), GameError> {
        use crate::game::battle::event_log::SkillCastTarget;
        let Some((_, _, entry)) = Self::counter_declaration(skill) else {
            return Ok(());
        };
        self.validate_counter_start(caster, skill)?;
        let Some(SkillCastTarget::Tile { position: tile }) = target else {
            return Err(GameError::InvalidAction);
        };
        let unit = &self.units[&caster];
        let origin = unit.body.projected_tile();
        let facing =
            FacingDirection::from_delta((tile.x - origin.x) as f32, (tile.y - origin.y) as f32)
                .or(unit.facing_direction)
                .unwrap_or(FacingDirection::Up);
        let key = self.counter_key(caster).ok_or(GameError::InvalidAction)?;
        let o = self
            .sustained
            .counters
            .owners
            .get_mut(&key)
            .ok_or(GameError::InvalidAction)?;
        let e = &o.program.entries[entry];
        let consumed = if e.spend == CounterSpend::None {
            0
        } else {
            o.count
        };
        if e.remember_aim {
            o.aim = Some(tile);
        }
        let reset = e.spend == CounterSpend::AdmissionAll;
        if reset {
            o.count = 0;
        }
        o.pending = Some(Pending {
            action: action.clone(),
            entry,
            consumed,
            tile,
            origin,
            facing,
        });
        if reset {
            self.schedule_counter_gain(key, now)?;
        }
        control::poll("counter_channel_admitted")
    }
    pub(super) fn counter_channel_started_at(&self, caster: UnitInstanceId) -> Option<u64> {
        let active = self.sustained.counters.owners.get(&self.counter_key(caster)?)?.active.as_ref()?;
        Some(active.channel_started_at)
    }
    pub(super) fn counter_channel_active(&self, caster: UnitInstanceId, pause_only: bool) -> bool {
        self.counter_key(caster)
            .and_then(|k| self.sustained.counters.owners.get(&k))
            .is_some_and(|o| {
                o.active
                    .as_ref()
                    .is_some_and(|a| !pause_only || o.program.entries[a.admitted.entry].pause_basic)
            })
    }
    pub(super) fn counter_channel_protected(&self, caster: UnitInstanceId) -> bool {
        self.counter_key(caster)
            .and_then(|k| self.sustained.counters.owners.get(&k))
            .is_some_and(|o| {
                o.active
                    .as_ref()
                    .is_some_and(|a| o.program.entries[a.admitted.entry].protected)
            })
    }
    pub(super) fn counter_channel_reinput(
        &self,
        caster: UnitInstanceId,
        skill: &SkillId,
    ) -> Option<u64> {
        let o = self
            .sustained
            .counters
            .owners
            .get(&self.counter_key(caster)?)?;
        let a = o.active.as_ref()?;
        let e = &o.program.entries[a.admitted.entry];
        (!e.automatic
            && o.program
                .entry_skill_id(&o.fragment.skill_id, a.admitted.entry)
                == *skill)
            .then_some(a.id)
    }
    pub(super) fn stop_counter_channel(
        &mut self,
        caster: UnitInstanceId,
        now: u64,
        manual: bool,
    ) -> Result<(), GameError> {
        let Some(key) = self.counter_key(caster) else {
            return Ok(());
        };
        let Some(o) = self.sustained.counters.owners.get(&key) else {
            return Ok(());
        };
        if let Some(a) = &o.active {
            if manual && o.program.entries[a.admitted.entry].automatic {
                return Err(GameError::InvalidAction);
            }
            self.finish_counter_channel(key, now, false, false)?;
        }
        Ok(())
    }
    pub(super) fn reconcile_counter_channels(&mut self, now: u64) -> Result<(), GameError> {
        if self.sustained.closed {
            return Ok(());
        }
        control::charge(
            (self.units.len() + self.sustained.counters.entity_count()) as u64,
            "counter_reconcile",
        )?;
        let removed: Vec<_> = self
            .sustained
            .counters
            .owners
            .iter()
            .filter(|(_, o)| !self.counter_owner_live(o))
            .map(|(k, _)| *k)
            .collect();
        for key in removed {
            self.finish_counter_channel(key, now, false, true)?;
            self.sustained.counters.owners.remove(&key);
            self.event_queue
                .discard_counter_events(Some(key), true, true)?;
        }
        let candidates: Vec<_> = self
            .units
            .values()
            .filter(|u| u.is_active() && !u.corroded)
            .filter_map(|u| {
                u.fragment_source
                    .as_ref()
                    .filter(|f| !self.sustained.counters.owners.contains_key(&f.copy_uuid))
                    .map(|f| (u.instance_id, f.clone()))
            })
            .collect();
        for (caster, fragment) in candidates {
            let Some((program, _, _)) = self
                .current_skill_view(&fragment.skill_id, caster, &Default::default())
                .and_then(Self::counter_declaration)
            else {
                continue;
            };
            let (program, applied) = self.counter_program(caster, program)?;
            let key = fragment.copy_uuid;
            let generation = self.next_counter_id()?;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                program::extent(&program)
                    .and_then(|n| n.checked_add(std::mem::size_of::<Owner>() + 128)),
                "counter_owner",
            )?;
            self.sustained.counters.owners.insert(
                key,
                Owner {
                    caster,
                    fragment,
                    generation,
                    program: Arc::new(program),
                    applied,
                    count: 0,
                    next_gain: None,
                    aim: None,
                    pending: None,
                    active: None,
                },
            );
            self.schedule_counter_gain(key, now)?;
        }
        let keys: Vec<_> = self.sustained.counters.owners.keys().copied().collect();
        for key in keys {
            let o = &self.sustained.counters.owners[&key];
            if let Some(p) = &o.pending {
                if self
                    .runtime_facts
                    .action_lifetime(&p.action)
                    .is_none_or(|(_, terminal, _)| terminal)
                {
                    let automatic = o.program.entries[p.entry].automatic;
                    let o = self.sustained.counters.owners.get_mut(&key).unwrap();
                    o.pending = None;
                    if automatic {
                        o.count = 0;
                        self.schedule_counter_gain(key, now)?;
                    }
                }
            }
            let o = &self.sustained.counters.owners[&key];
            if o.count < o.program.capacity || o.pending.is_some() || o.active.is_some() {
                continue;
            }
            let caster = o.caster;
            let unit = &self.units[&caster];
            if unit.pending_skill_cast.is_some()
                || unit.next_action_time > now
                || self.skill_start_disabled_reason(caster, now).is_some()
                || self.resonance_use_has_obligations(caster)
            {
                continue;
            }
            let entry = o
                .program
                .entries
                .iter()
                .position(|e| e.automatic)
                .ok_or(GameError::InvalidAction)?;
            let skill_id = o.program.entry_skill_id(&o.fragment.skill_id, entry);
            let tile = o
                .aim
                .filter(|p| self.battlefield.is_valid_tile(*p))
                .unwrap_or_else(|| unit.body.projected_tile());
            let skill = self
                .cloned_current_skill(&skill_id, caster, &Default::default())?
                .ok_or(GameError::InvalidAction)?;
            self.start_admitted_skill_cast(
                now,
                caster,
                skill,
                Some(crate::game::battle::event_log::SkillCastTarget::Tile { position: tile }),
                false,
                true,
            )?;
        }
        Ok(())
    }
    pub(super) fn clear_counter_channels(&mut self, now: u64) -> Result<(), GameError> {
        let keys: Vec<_> = self.sustained.counters.owners.keys().copied().collect();
        for key in keys {
            self.finish_counter_channel(key, now, false, true)?;
        }
        self.sustained.counters.owners.clear();
        self.event_queue.discard_counter_events(None, true, true)
    }
}
