//! Relationship generations, battle-copy resources and finite admitted actions.
mod effects;
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
            marks::{AppliedMarkModifier, MarkRatio},
            relation_action::*,
            SkillDef, SkillEffectDef, SkillId,
        },
        battle::{
            damage::DamageSourceSnapshot,
            enums::BattleEvent,
            event_log::SkillCastTarget,
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
struct Relation {
    target: UnitInstanceId,
    generation: u64,
    next_gain: u64,
}
crate::authoring::clone_shape::struct_shape!(Relation {
    target,
    generation,
    next_gain
});
#[derive(Debug, Clone)]
struct Pending {
    action: ActionHandle,
    active_mode: bool,
    tile: Position,
    facing: FacingDirection,
    primary: Option<UnitInstanceId>,
}
crate::authoring::clone_shape::struct_shape!(Pending {
    action,
    active_mode,
    tile,
    facing,
    primary
});
#[derive(Debug, Clone)]
enum Continuation {
    Plan {
        index: usize,
        paid: u32,
    },
    AfterSpend {
        plan: usize,
        paid: u32,
        target: Option<UnitInstanceId>,
    },
    Repeat {
        plan: usize,
        paid: u32,
        start: u64,
        index: u32,
        count: u32,
    },
    Timed {
        plan: usize,
        paid: u32,
        end: u64,
        next: u64,
    },
    ClearStance,
    ResumeAt {
        time: u64,
    },
}
#[derive(Debug, Clone)]
struct Active {
    id: u64,
    admitted: Pending,
    source: RuntimeFactSource,
    snapshot: Option<DamageSourceSnapshot>,
    waiting_basic: bool,
    bound_basic: Option<ActionHandle>,
    bound_epoch: u64,
    start_stock: u32,
    primary_position: Option<Position>,
    last_strike_primary: Option<UnitInstanceId>,
    continuations: Vec<Continuation>,
    next_at: Option<u64>,
    next_self: Option<u64>,
    stance: MarkRatio,
    hits: BTreeMap<UnitInstanceId, u32>,
    ramps: BTreeMap<UnitInstanceId, MarkRatio>,
    actual_hp_loss: u64,
    hit_index: u32,
}
crate::authoring::clone_shape::struct_shape!(Active {
    id,
    admitted,
    source,
    snapshot,
    waiting_basic,
    bound_basic,
    bound_epoch,
    start_stock,
    primary_position,
    last_strike_primary,
    continuations,
    next_at,
    next_self,
    stance,
    hits,
    ramps,
    actual_hp_loss,
    hit_index
});
impl crate::authoring::clone_shape::CloneShape for Continuation {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), GameError> {
        m.scan_storage(Some(1))
    }
}
#[derive(Debug, Clone)]
struct Owner {
    caster: UnitInstanceId,
    fragment: Arc<FragmentSourceSnapshot>,
    program: Arc<RelationActionDef>,
    applied: Vec<AppliedMarkModifier>,
    stock: u32,
    mode: bool,
    reserved_mode: bool,
    shield_used: bool,
    relation: Option<Relation>,
    pending: Option<Pending>,
    active: Option<Active>,
}
crate::authoring::clone_shape::struct_shape!(Owner {
    caster,
    fragment,
    program,
    applied,
    stock,
    mode,
    reserved_mode,
    shield_used,
    relation,
    pending,
    active
});
#[derive(Debug, Clone, Default)]
pub(super) struct RelationActionState {
    next: u64,
    owners: BTreeMap<Uuid, Owner>,
}
crate::authoring::clone_shape::struct_shape!(RelationActionState { next, owners });
impl RelationActionState {
    pub(super) fn has_use_effect(
        &self,
        action: &ActionHandle,
        owns: impl Fn(&RuntimeFactSource) -> bool,
    ) -> bool {
        self.owners
            .values()
            .filter_map(|o| o.active.as_ref())
            .any(|a| &a.admitted.action == action && owns(&a.source))
    }
    pub(super) fn entity_count(&self) -> usize {
        self.owners.len()
            + self
                .owners
                .values()
                .map(|o| {
                    usize::from(o.relation.is_some())
                        + usize::from(o.pending.is_some())
                        + o.active.as_ref().map_or(0, |a| {
                            1 + a.continuations.len() + a.hits.len() + a.ramps.len()
                        })
                })
                .sum::<usize>()
    }
    pub(super) fn retained_extent(&self) -> Option<usize> {
        let mut n = self
            .owners
            .len()
            .checked_mul(std::mem::size_of::<Owner>() + 80)?;
        for o in self.owners.values() {
            n = n
                .checked_add(o.fragment.retained_extent()?)?
                .checked_add(program::extent(&o.program)?)?
                .checked_add(
                    o.applied
                        .capacity()
                        .checked_mul(std::mem::size_of::<AppliedMarkModifier>())?,
                )?;
            for m in &o.applied {
                n = n.checked_add(m.rule_id.capacity() + m.relic_definition.capacity())?;
            }
            if let Some(a) = &o.active {
                n = n
                    .checked_add(a.source.retained_extent()?)?
                    .checked_add(
                        a.snapshot
                            .as_ref()
                            .map_or(Some(0), DamageSourceSnapshot::fact_retained_extent)?,
                    )?
                    .checked_add(
                        a.continuations
                            .capacity()
                            .checked_mul(std::mem::size_of::<Continuation>())?,
                    )?
                    .checked_add((a.hits.len() + a.ramps.len()).checked_mul(128)?)?;
            }
        }
        Some(n)
    }
}
const ZERO: MarkRatio = MarkRatio {
    numerator: 0,
    denominator: 1,
};
impl BattleCore {
    fn relation_declaration(skill: &SkillDef) -> Option<&Arc<RelationActionDef>> {
        skill
            .steps
            .iter()
            .flat_map(|s| &s.effects)
            .find_map(|e| match &e.op {
                SkillEffectDef::RelationAction { program } => Some(program),
                _ => None,
            })
    }
    pub(super) fn relation_action_skill(skill: &SkillDef) -> bool {
        Self::relation_declaration(skill).is_some()
    }
    fn relation_key(&self, caster: UnitInstanceId) -> Option<Uuid> {
        self.units
            .get(&caster)?
            .fragment_source
            .as_ref()
            .map(|f| f.copy_uuid)
    }
    fn relation_live(&self, o: &Owner) -> bool {
        self.units.get(&o.caster).is_some_and(|u| {
            u.is_active()
                && !u.corroded
                && u.fragment_source.as_deref() == Some(o.fragment.as_ref())
        })
    }
    fn next_relation_id(&mut self) -> Result<u64, GameError> {
        let s = &mut self.sustained.relations;
        s.next = s.next.checked_add(1).ok_or(GameError::InvalidAction)?;
        Ok(s.next)
    }
    fn relation_entry<'b>(p: &'b RelationActionDef, a: &Pending) -> &'b RelationActionEntry {
        if a.active_mode {
            &p.active
        } else {
            &p.normal
        }
    }
    pub(super) fn relation_action_active(&self, caster: UnitInstanceId) -> bool {
        self.relation_key(caster)
            .and_then(|k| self.sustained.relations.owners.get(&k))
            .is_some_and(|o| o.active.as_ref().is_some_and(|a| !a.waiting_basic))
    }
    pub(super) fn relation_action_protected(&self, caster: UnitInstanceId) -> bool {
        self.relation_key(caster)
            .and_then(|k| self.sustained.relations.owners.get(&k))
            .is_some_and(|o| {
                o.active.as_ref().is_some_and(|a| {
                    !a.waiting_basic && Self::relation_entry(&o.program, &a.admitted).protected
                })
            })
    }
    pub(super) fn relation_action_reinput(
        &self,
        caster: UnitInstanceId,
        skill: &SkillId,
    ) -> Option<u64> {
        let o = self
            .sustained
            .relations
            .owners
            .get(&self.relation_key(caster)?)?;
        let a = o.active.as_ref()?;
        (&o.fragment.skill_id == skill
            && !a.waiting_basic
            && Self::relation_entry(&o.program, &a.admitted).manual_stop)
            .then_some(a.id)
    }
    pub(super) fn relation_target_died(
        &mut self,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        control::charge(
            self.sustained.relations.owners.len() as u64,
            "relation_death_scan",
        )?;
        for o in self.sustained.relations.owners.values_mut() {
            if o.relation.as_ref().is_some_and(|r| r.target == target)
                && o.caster != target
                && !o.mode
            {
                o.reserved_mode = true;
            }
        }
        let wakes: Vec<_> = self
            .sustained
            .relations
            .owners
            .iter()
            .filter_map(|(key, o)| {
                o.active
                    .as_ref()
                    .filter(|a| {
                        !a.waiting_basic
                            && a.admitted.primary == Some(target)
                            && a.next_at.is_some_and(|t| t > now)
                    })
                    .map(|a| (*key, a.id))
            })
            .collect();
        for (key, id) in wakes {
            self.event_queue
                .discard_relation_events(Some(key), false, true)?;
            let a = self
                .sustained
                .relations
                .owners
                .get_mut(&key)
                .unwrap()
                .active
                .as_mut()
                .unwrap();
            a.next_at = Some(now);
            self.event_queue.push_with_context(
                BattleEvent::RelationActionTick {
                    source: key,
                    generation: id,
                    time_ms: now,
                },
                None,
                controlled_clone(&a.source)?,
            );
        }
        Ok(())
    }
    pub(super) fn reconcile_relation_actions(&mut self, now: u64) -> Result<(), GameError> {
        if self.sustained.closed {
            return Ok(());
        }
        control::charge(
            (self.units.len() + self.sustained.relations.entity_count()) as u64,
            "relation_reconcile",
        )?;
        let keys: Vec<_> = self.sustained.relations.owners.keys().copied().collect();
        for key in keys {
            let stale = self.sustained.relations.owners[&key]
                .pending
                .as_ref()
                .is_some_and(|p| {
                    !self
                        .runtime_facts
                        .action_lifetime(&p.action)
                        .is_some_and(|(_, terminal, _)| !terminal)
                });
            if stale {
                self.sustained
                    .relations
                    .owners
                    .get_mut(&key)
                    .unwrap()
                    .pending = None;
            }
            let o = &self.sustained.relations.owners[&key];
            let live = self.relation_live(o);
            let relation_live = o
                .relation
                .as_ref()
                .is_none_or(|r| self.units.get(&r.target).is_some_and(|u| u.is_active()));
            if !live {
                self.finish_relation_action(key, now, false, true)?;
                self.sustained
                    .relations
                    .owners
                    .get_mut(&key)
                    .unwrap()
                    .pending = None;
            }
            if !live || !relation_live {
                self.sustained
                    .relations
                    .owners
                    .get_mut(&key)
                    .unwrap()
                    .relation = None;
                self.event_queue
                    .discard_relation_events(Some(key), true, !live)?;
            }
        }
        let candidates: Vec<_> = self
            .units
            .values()
            .filter(|u| u.is_active() && !u.corroded)
            .filter_map(|u| {
                u.fragment_source
                    .as_ref()
                    .map(|f| (u.instance_id, f.clone()))
            })
            .collect();
        for (caster, fragment) in candidates {
            if self
                .sustained
                .relations
                .owners
                .get(&fragment.copy_uuid)
                .is_some_and(|o| o.caster == caster && o.fragment == fragment)
            {
                continue;
            }
            let Some(def) = self
                .current_skill_view(&fragment.skill_id, caster, &Default::default())
                .and_then(Self::relation_declaration)
            else {
                continue;
            };
            let (program, applied) = self.relation_program(caster, def)?;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                program::extent(&program)
                    .and_then(|n| n.checked_add(std::mem::size_of::<Owner>() + 128)),
                "relation_owner",
            )?;
            self.admit_authoring_runtime_growth(1, "relation_owner_entity")?;
            if let Some(o) = self.sustained.relations.owners.get_mut(&fragment.copy_uuid) {
                o.caster = caster;
                o.fragment = fragment;
                o.program = Arc::new(program);
                o.applied = applied;
            } else {
                self.sustained.relations.owners.insert(
                    fragment.copy_uuid,
                    Owner {
                        caster,
                        fragment,
                        stock: program.initial_stock,
                        program: Arc::new(program),
                        applied,
                        mode: false,
                        reserved_mode: false,
                        shield_used: false,
                        relation: None,
                        pending: None,
                        active: None,
                    },
                );
            }
        }
        let keys: Vec<_> = self.sustained.relations.owners.keys().copied().collect();
        for key in keys {
            let o = &self.sustained.relations.owners[&key];
            if !self.relation_live(o) {
                continue;
            }
            let caster = o.caster;
            let interrupt = o.active.as_ref().is_some_and(|a| {
                !a.waiting_basic
                    && !Self::relation_entry(&o.program, &a.admitted).protected
                    && self.unit_control_state(caster, now).is_some_and(|s| {
                        s.hard_cc_active()
                            || (s.silenced
                                && Self::relation_entry(&o.program, &a.admitted)
                                    .interrupt_on_silence)
                    })
            });
            if interrupt {
                self.finish_relation_action(key, now, false, false)?;
            }
            self.check_relation_shield(key, now)?;
            self.enter_reserved_relation_mode(key, now)?;
        }
        Ok(())
    }
    fn enter_reserved_relation_mode(&mut self, key: Uuid, now: u64) -> Result<(), GameError> {
        let o = &self.sustained.relations.owners[&key];
        let caster = o.caster;
        if !o.reserved_mode
            || o.mode
            || o.active.is_some()
            || o.pending.is_some()
            || !self.relation_live(o)
            || self.resonance_use_has_obligations(caster)
            || self.runtime_facts.actor_has_unsettled_action(caster)?
            || self.units[&caster].pending_skill_cast.is_some()
            || self.units[&caster].basic_attack_windup_until_ms.is_some()
        {
            return Ok(());
        }
        let percent = o.program.mode_damage_percent;
        let stock = o.stock;
        let fragment = o.fragment.clone();
        let o = self.sustained.relations.owners.get_mut(&key).unwrap();
        o.mode = true;
        o.reserved_mode = false;
        o.relation = None;
        self.event_queue
            .discard_relation_events(Some(key), true, false)?;
        self.relation_self_damage(
            caster,
            &fragment,
            percent,
            stock,
            crate::game::battle::damage::DamageType::True,
            true,
            now,
        )?;
        control::poll("relation_mode_entered")
    }
    pub(super) fn select_relation_target(
        &mut self,
        caster: UnitInstanceId,
        entry: &str,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        self.reconcile_relation_actions(now)?;
        let key = self.relation_key(caster).ok_or(GameError::InvalidAction)?;
        let o = self
            .sustained
            .relations
            .owners
            .get(&key)
            .ok_or(GameError::InvalidAction)?;
        let a = self.units.get(&caster).ok_or(GameError::InvalidAction)?;
        let b = self.units.get(&target).ok_or(GameError::InvalidAction)?;
        let control = self
            .unit_control_state(caster, now)
            .ok_or(GameError::InvalidAction)?;
        if !self.normal_player_employee(caster)
            || !self.normal_player_employee(target)
            || !self.relation_live(o)
            || o.program.menu_entry != entry
            || o.mode
            || !b.is_active()
            || a.owner != b.owner
            || !b.is_combatant()
            || control.hard_cc_active()
            || control.silenced
            || self.unit_is_field_working(caster)
            || self.unit_is_in_defense_stance(caster)
        {
            return Err(GameError::InvalidAction);
        }
        let same = o.relation.as_ref().is_some_and(|r| r.target == target);
        let period = o.program.period_ms;
        self.event_queue
            .discard_relation_events(Some(key), true, false)?;
        let relation = if same {
            None
        } else {
            Some(Relation {
                target,
                generation: self.next_relation_id()?,
                next_gain: now
                    .checked_add(period.into())
                    .ok_or(GameError::InvalidAction)?,
            })
        };
        self.sustained
            .relations
            .owners
            .get_mut(&key)
            .unwrap()
            .relation = relation;
        self.schedule_relation_gain(key)?;
        self.check_relation_shield(key, now)?;
        control::poll("relation_target_changed")
    }
    fn schedule_relation_gain(&mut self, key: Uuid) -> Result<(), GameError> {
        if let Some(r) = self.sustained.relations.owners[&key].relation.as_ref() {
            self.event_queue.push_with_context(
                BattleEvent::RelationGain {
                    source: key,
                    generation: r.generation,
                    time_ms: r.next_gain,
                },
                None,
                RuntimeFactSource::default(),
            );
        }
        Ok(())
    }
    pub(super) fn relation_gain(
        &mut self,
        key: Uuid,
        generation: u64,
        now: u64,
    ) -> Result<(), GameError> {
        if !self
            .sustained
            .relations
            .owners
            .get(&key)
            .and_then(|o| o.relation.as_ref())
            .is_some_and(|r| r.generation == generation && r.next_gain == now)
        {
            return Ok(());
        }
        self.reconcile_relation_actions(now)?;
        let Some(o) = self.sustained.relations.owners.get_mut(&key) else {
            return Ok(());
        };
        let Some(r) = o
            .relation
            .as_mut()
            .filter(|r| r.generation == generation && r.next_gain == now)
        else {
            return Ok(());
        };
        o.stock = if let Some(cap) = o.program.stock_capacity {
            o.stock.saturating_add(o.program.periodic_gain).min(cap)
        } else {
            o.stock
                .checked_add(o.program.periodic_gain)
                .ok_or(GameError::InvalidAction)?
        };
        r.next_gain = now
            .checked_add(o.program.period_ms.into())
            .ok_or(GameError::InvalidAction)?;
        self.schedule_relation_gain(key)?;
        control::poll("relation_stock_gained")
    }
    pub(super) fn prepare_relation_action(
        &mut self,
        caster: UnitInstanceId,
        skill: &SkillDef,
        action: &ActionHandle,
        target: Option<SkillCastTarget>,
        now: u64,
    ) -> Result<(), GameError> {
        if Self::relation_declaration(skill).is_none() {
            return Ok(());
        }
        let key = self.relation_key(caster).ok_or(GameError::InvalidAction)?;
        self.enter_reserved_relation_mode(key, now)?;
        let o = self
            .sustained
            .relations
            .owners
            .get(&key)
            .ok_or(GameError::InvalidAction)?;
        if !self.relation_live(o) || o.active.is_some() || o.pending.is_some() {
            return Err(GameError::InvalidAction);
        }
        let entry = if o.mode {
            &o.program.active
        } else {
            &o.program.normal
        };
        let unit = &self.units[&caster];
        let origin = unit.body.projected_tile();
        let tile = match target {
            Some(SkillCastTarget::Tile { position }) => position,
            _ => origin,
        };
        let facing = unit.facing_direction.unwrap_or(FacingDirection::Up);
        let primary = if let Some(selector) = &entry.selector {
            Some(
                self.relation_primary(caster, &selector.pattern, now)
                    .ok_or(GameError::InvalidAction)?,
            )
        } else {
            None
        };
        let pending = Pending {
            action: action.clone(),
            active_mode: o.mode,
            tile,
            facing,
            primary,
        };
        self.sustained
            .relations
            .owners
            .get_mut(&key)
            .unwrap()
            .pending = Some(pending);
        control::poll("relation_action_admitted")
    }
    pub(super) fn clear_relation_actions(&mut self, now: u64) -> Result<(), GameError> {
        let keys: Vec<_> = self.sustained.relations.owners.keys().copied().collect();
        for key in keys {
            self.finish_relation_action(key, now, false, true)?;
        }
        self.sustained.relations = Default::default();
        self.event_queue.discard_relation_events(None, true, true)
    }
}
