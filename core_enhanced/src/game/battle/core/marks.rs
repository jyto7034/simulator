//! Deployment-owned presence marks. Hosts are records, never battle units.
mod power;
use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        ability::{marks::*, SkillEffectDef},
        battle::{
            damage::{DamageModifiers, DamageSource, DamageSourceSnapshot},
            enums::BattleEvent,
            ids::UnitInstanceId,
            runtime_facts::{ActionHandle, RuntimeFactSource},
            types::{BattleUnitSourceIdentity, BattleUnitThreatClass, MobilityKind},
        },
        behavior::GameError,
        skill_fragment::FragmentSourceSnapshot,
    },
};
use power::PowerClock;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PresenceCapture {
    pub fragment: Arc<FragmentSourceSnapshot>,
    pub program: Arc<PresenceBurstProgram>,
    pub applied: Vec<AppliedMarkModifier>,
}
crate::authoring::clone_shape::struct_shape!(PresenceCapture {
    fragment,
    program,
    applied
});
impl PresenceCapture {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut n = std::mem::size_of::<Self>()
            .checked_add(128)?
            .checked_add(self.fragment.retained_extent()?)?
            .checked_add(self.program.marks.definition.capacity())?;
        n = n.checked_add(crate::game::relic::structural::support::pattern_extent(
            &self.program.explosion.range,
        )?)?;
        n = n.checked_add(
            self.program
                .modifiers
                .capacity()
                .checked_mul(std::mem::size_of::<MarkModifier>())?,
        )?;
        for m in &self.program.modifiers {
            n = n
                .checked_add(m.id.capacity())?
                .checked_add(m.relic_definition.capacity())?;
            if let MarkChange::Range { pattern } = &m.change {
                n = n.checked_add(crate::game::relic::structural::support::pattern_extent(
                    pattern,
                )?)?;
            }
        }
        n = n.checked_add(
            self.applied
                .capacity()
                .checked_mul(std::mem::size_of::<AppliedMarkModifier>())?,
        )?;
        for m in &self.applied {
            n = n
                .checked_add(m.rule_id.capacity())?
                .checked_add(m.relic_definition.capacity())?;
        }
        Some(n)
    }
}
#[derive(Debug, Clone)]
struct Host {
    generation: u64,
    next_damage_at: u64,
    born_action: Option<ActionHandle>,
}
crate::authoring::clone_shape::struct_shape!(Host {
    generation,
    next_damage_at,
    born_action
});
#[derive(Debug, Clone)]
struct Network {
    caster: UnitInstanceId,
    capture: Arc<PresenceCapture>,
    hosts: BTreeMap<UnitInstanceId, Host>,
    power: PowerClock,
}
crate::authoring::clone_shape::struct_shape!(Network {
    caster,
    capture,
    hosts,
    power
});
#[derive(Debug, Clone)]
struct PendingBurst {
    action: ActionHandle,
    caster: UnitInstanceId,
    capture: Arc<PresenceCapture>,
    centers: Vec<(UnitInstanceId, crate::game::resources::Position)>,
    ordinary_damage: u32,
    host_damage: u32,
}
crate::authoring::clone_shape::struct_shape!(PendingBurst {
    action,
    caster,
    capture,
    centers,
    ordinary_damage,
    host_damage
});
#[derive(Debug, Clone, Default)]
pub(super) struct PresenceState {
    next: u64,
    networks: BTreeMap<uuid::Uuid, Network>,
    pending: Vec<PendingBurst>,
    attempts: Vec<(ActionHandle, UnitInstanceId, uuid::Uuid)>,
    suppress_spread: bool,
}
crate::authoring::clone_shape::struct_shape!(PresenceState {
    next,
    networks,
    pending,
    attempts,
    suppress_spread
});
impl PresenceState {
    pub(super) fn retained_extent(&self) -> Option<usize> {
        let mut n = 0usize;
        for r in self.networks.values() {
            n = n
                .checked_add(std::mem::size_of::<Network>() + 80)?
                .checked_add(r.capture.retained_extent()?)?
                .checked_add(
                    r.hosts
                        .len()
                        .checked_mul(std::mem::size_of::<Host>() + 96)?,
                )?;
        }
        n = n.checked_add(
            self.pending
                .capacity()
                .checked_mul(std::mem::size_of::<PendingBurst>())?,
        )?;
        for p in &self.pending {
            n = n.checked_add(p.capture.retained_extent()?)?.checked_add(
                p.centers.capacity().checked_mul(std::mem::size_of::<(
                    UnitInstanceId,
                    crate::game::resources::Position,
                )>())?,
            )?;
        }
        n = n.checked_add(self.attempts.capacity().checked_mul(std::mem::size_of::<(
            ActionHandle,
            UnitInstanceId,
            uuid::Uuid,
        )>())?)?;
        Some(n)
    }
    pub(super) fn entity_count(&self) -> usize {
        self.attempts.len()
            + self.pending.len()
            + self.networks.len()
            + self.networks.values().map(|r| r.hosts.len()).sum::<usize>()
    }
}
pub(super) fn ratio_product(a: MarkRatio, b: MarkRatio) -> Result<MarkRatio, GameError> {
    fn gcd(mut a: u64, mut b: u64) -> u64 {
        while b != 0 {
            let r = a % b;
            a = b;
            b = r;
        }
        a
    }
    let x = gcd(a.numerator, b.denominator);
    let y = gcd(b.numerator, a.denominator);
    Ok(MarkRatio {
        numerator: (a.numerator / x)
            .checked_mul(b.numerator / y)
            .ok_or(GameError::InvalidAction)?,
        denominator: (a.denominator / y)
            .checked_mul(b.denominator / x)
            .ok_or(GameError::InvalidAction)?,
    })
}
pub(super) fn ratio_sum(a: MarkRatio, b: MarkRatio) -> Result<MarkRatio, GameError> {
    fn gcd(mut a: u128, mut b: u128) -> u128 {
        while b != 0 {
            let r = a % b;
            a = b;
            b = r;
        }
        a
    }
    let d = u128::from(a.denominator) * u128::from(b.denominator);
    let n = (u128::from(a.numerator) * u128::from(b.denominator))
        .checked_add(u128::from(b.numerator) * u128::from(a.denominator))
        .ok_or(GameError::InvalidAction)?;
    let g = gcd(n, d);
    Ok(MarkRatio {
        numerator: u64::try_from(n / g).map_err(|_| GameError::InvalidAction)?,
        denominator: u64::try_from(d / g).map_err(|_| GameError::InvalidAction)?,
    })
}
/// Add changes of the same time value before the single final millisecond ceiling.
pub(super) fn modified_period(base: u32, changes: &[(bool, MarkRatio)]) -> Result<u32, GameError> {
    let mut increase = MarkRatio { numerator: 100, denominator: 1 };
    let mut decrease = MarkRatio { numerator: 0, denominator: 1 };
    for &(negative, ratio) in changes {
        if negative { decrease = ratio_sum(decrease, ratio)?; }
        else { increase = ratio_sum(increase, ratio)?; }
    }
    let n = (u128::from(increase.numerator) * u128::from(decrease.denominator))
        .checked_sub(u128::from(decrease.numerator) * u128::from(increase.denominator))
        .filter(|n| *n > 0).ok_or(GameError::InvalidAction)?;
    let d = (u128::from(increase.denominator) * u128::from(decrease.denominator))
        .checked_mul(100).ok_or(GameError::InvalidAction)?;
    u32::try_from(u128::from(base).checked_mul(n).ok_or(GameError::InvalidAction)?.div_ceil(d))
        .ok().filter(|n| *n > 0).ok_or(GameError::InvalidAction)
}
impl BattleCore {
    pub(super) fn bind_presence(
        &self,
        snapshot: &mut DamageSourceSnapshot,
    ) -> Result<(), GameError> {
        if snapshot.source != DamageSource::BasicAttack || snapshot.fact_source.reaction {
            return Ok(());
        }
        let Some(fragment) = self
            .units
            .get(&snapshot.source_id)
            .filter(|u| u.is_active() && !u.corroded)
            .and_then(|u| u.fragment_source.clone())
        else {
            return Ok(());
        };
        let Some(skill) = self
            .game_data
            .skill_data
            .get_by_id(fragment.skill_id.as_str())
        else {
            return Ok(());
        };
        control::charge_serialized(skill, "presence_provider_scan")?;
        let mut selected = None;
        for effect in skill.steps.iter().flat_map(|s| &s.effects) {
            if let SkillEffectDef::PresenceBurst { program } = &effect.op {
                if selected.is_some() {
                    return Err(GameError::InvalidAction);
                }
                selected = Some(program);
            }
        }
        let Some(program) = selected else {
            return Ok(());
        };
        let mut program = (**program).clone();
        let mut applied = Vec::new();
        let mut period_changes = Vec::new();
        for modifier in &program.modifiers {
            let Some(copy) =
                self.equipped_normal_relic_copy(snapshot.source_id, &modifier.relic_definition)
            else {
                continue;
            };
            applied.push(AppliedMarkModifier {
                rule_id: modifier.id.clone(),
                relic_definition: modifier.relic_definition.clone(),
                relic_copy_uuid: copy,
            });
            match &modifier.change {
                MarkChange::Period { decrease, percent } => {
                    period_changes.push((*decrease, *percent));
                }
                MarkChange::Growth { percent } => {
                    let d = percent
                        .denominator
                        .checked_mul(100)
                        .ok_or(GameError::InvalidAction)?;
                    program.marks.power.growth_per_second = ratio_product(
                        program.marks.power.growth_per_second,
                        MarkRatio {
                            numerator: d
                                .checked_add(percent.numerator)
                                .ok_or(GameError::InvalidAction)?,
                            denominator: d,
                        },
                    )?;
                }
                MarkChange::PreservePower => program.marks.power.prevent_decay = true,
                MarkChange::Range { pattern } => program.explosion.range = pattern.clone(),
                MarkChange::ExplosionPower { percent } => {
                    program.explosion.bonus_percent =
                        ratio_sum(program.explosion.bonus_percent, *percent)?;
                }
                MarkChange::ExplosionOverlap => program.explosion.dedup = ExplosionDedup::Explosion,
            }
        }
        program.marks.periodic_interval_ms = modified_period(program.marks.periodic_interval_ms, &period_changes)?;
        program.validate().map_err(GameError::InvalidStaticData)?;
        let capture = PresenceCapture {
            fragment,
            program: Arc::new(program),
            applied,
        };
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            capture.retained_extent(),
            "presence_capture",
        )?;
        snapshot.presence_capture = Some(Arc::new(capture));
        Ok(())
    }
    fn presence_source_live(&self, r: &Network) -> bool {
        self.units.get(&r.caster).is_some_and(|u| {
            u.is_active()
                && !u.corroded
                && u.fragment_source
                    .as_ref()
                    .is_some_and(|f| f.as_ref() == r.capture.fragment.as_ref())
        })
    }
    fn presence_eligible(&self, r: &Network, target: UnitInstanceId) -> bool {
        let Some(c) = self.units.get(&r.caster) else { return false; };
        self.mark_target_eligible(c.owner, r.capture.program.marks.filter, target)
    }
    pub(super) fn mark_target_eligible(&self, owner:crate::game::enums::Side, f:MarkFilter, target:UnitInstanceId) -> bool {
        if control::charge(self.scenario_runtime.spawned.unit_refs.len() as u64,"mark_encounter_membership").is_err() {return false;}
        let Some(u) = self.units.get(&target) else { return false; };
        u.is_active() && u.stats.current_health > 0 && u.owner != owner
            && (f.air || u.mobility_kind != MobilityKind::Airborne)
            && (f.boss || u.threat_class != BattleUnitThreatClass::Boss)
            && (f.abnormality || !matches!(u.source_identity,BattleUnitSourceIdentity::Abnormality { .. }))
            && (f.summon || !matches!(u.source_identity,BattleUnitSourceIdentity::SummonedUnit { .. }))
            && (f.temporary || self.scenario_runtime.spawned.unit_refs.values().any(|id| *id == target))
            && !matches!(u.source_identity,BattleUnitSourceIdentity::IndependentObject { .. } | BattleUnitSourceIdentity::DefenseObject { .. })
    }
    pub(super) fn reconcile_presence(&mut self, now: u64) -> Result<(), GameError> {
        control::charge(
            self.sustained.presence.entity_count() as u64,
            "presence_reconcile",
        )?;
        let invalid: Vec<_> = self
            .sustained
            .presence
            .networks
            .iter()
            .filter(|(_, r)| !self.presence_source_live(r))
            .map(|(id, _)| *id)
            .collect();
        for id in invalid {
            self.sustained.presence.networks.remove(&id);
            self.sustained
                .presence
                .attempts
                .retain(|(_, _, copy)| *copy != id);
            self.event_queue.discard_presence_ticks(Some(id), None)?;
        }
        self.sustained.presence.pending.retain(|p| {
            self.runtime_facts
                .action_lifetime(&p.action)
                .is_some_and(|(_, terminal, _)| !terminal)
                && self
                    .units
                    .get(&p.caster)
                    .is_some_and(|u| u.is_active() && !u.corroded)
        });
        self.sustained
            .presence
            .attempts
            .retain(|(a, _, _)| self.runtime_facts.action_lifetime(a).is_some());
        for r in self.sustained.presence.networks.values_mut() {
            let old = r.hosts.len();
            if r.hosts
                .keys()
                .any(|id| !self.units.get(id).is_some_and(|u| u.is_active()))
            {
                r.power.settle(&r.capture.program.marks.power, old, now)?;
                r.hosts
                    .retain(|id, _| self.units.get(id).is_some_and(|u| u.is_active()));
                r.power.membership_changed(old, r.hosts.len(), now);
            }
        }
        Ok(())
    }
    fn infect_presence(
        &mut self,
        key: uuid::Uuid,
        target: UnitInstanceId,
        action: Option<ActionHandle>,
        now: u64,
    ) -> Result<(), GameError> {
        let r = self
            .sustained
            .presence
            .networks
            .get(&key)
            .ok_or(GameError::InvalidAction)?;
        if r.hosts.contains_key(&target) || !self.presence_eligible(r, target) {
            return Ok(());
        }
        let generation = self
            .sustained
            .presence
            .next
            .checked_add(1)
            .ok_or(GameError::InvalidAction)?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<Host>() + 96),
            "presence_host",
        )?;
        let r = self.sustained.presence.networks.get_mut(&key).unwrap();
        let old = r.hosts.len();
        r.power.settle(&r.capture.program.marks.power, old, now)?;
        let time_ms = now
            .checked_add(u64::from(r.capture.program.marks.periodic_interval_ms))
            .ok_or(GameError::InvalidAction)?;
        r.hosts.insert(
            target,
            Host {
                generation,
                next_damage_at: time_ms,
                born_action: action,
            },
        );
        r.power.membership_changed(old, old + 1, now);
        self.sustained.presence.next = generation;
        self.event_queue.push(BattleEvent::PresenceTick {
            time_ms,
            source: key,
            host: target,
            generation,
        });
        Ok(())
    }
    fn spread_presence(
        &mut self,
        key: uuid::Uuid,
        host: UnitInstanceId,
        action: Option<ActionHandle>,
        now: u64,
    ) -> Result<(), GameError> {
        let r = self
            .sustained
            .presence
            .networks
            .get(&key)
            .ok_or(GameError::InvalidAction)?;
        let Some(center) = self.sample_unit_world_position_at(host, now) else {
            return Ok(());
        };
        let radius = f64::from(r.capture.program.marks.spread_radius_units)
            / f64::from(super::movement::types::DATA_UNITS_PER_WORLD);
        control::charge(self.units.len() as u64, "presence_spread_candidates")?;
        let mut candidates: Vec<_> = self
            .units
            .values()
            .filter(|u| {
                u.instance_id != host
                    && !r.hosts.contains_key(&u.instance_id)
                    && self.presence_eligible(r, u.instance_id)
            })
            .filter_map(|u| {
                let position = self.sample_unit_world_position_at(u.instance_id, now)?;
                let dx = f64::from(position.x - center.x);
                let dy = f64::from(position.y - center.y);
                let d = dx * dx + dy * dy;
                (d <= radius * radius).then_some((
                    d,
                    self.enemy_route_progress_at(u.instance_id, now),
                    u.instance_id,
                ))
            })
            .collect();
        candidates.sort_by(|a, b| {
            a.0.total_cmp(&b.0)
                .then_with(|| b.1.total_cmp(&a.1))
                .then_with(|| a.2.cmp(&b.2))
        });
        let limit = r.capture.program.marks.spread_limit as usize;
        for (_, _, id) in candidates.into_iter().take(limit) {
            self.infect_presence(key, id, action.clone(), now)?;
        }
        Ok(())
    }
    pub(super) fn presence_hit(
        &mut self,
        snapshot: &DamageSourceSnapshot,
        target: UnitInstanceId,
        killed: bool,
        now: u64,
    ) -> Result<(), GameError> {
        if killed || snapshot.source != DamageSource::BasicAttack || snapshot.fact_source.reaction {
            return Ok(());
        }
        let (Some(capture), Some(action)) =
            (&snapshot.presence_capture, &snapshot.fact_source.action)
        else {
            return Ok(());
        };
        self.reconcile_presence(now)?;
        let key = capture.fragment.copy_uuid;
        if self
            .sustained
            .presence
            .attempts
            .iter()
            .any(|(a, t, f)| a == action && *t == target && *f == key)
        {
            return Ok(());
        }
        let new = Network {
            caster: snapshot.source_id,
            capture: capture.clone(),
            hosts: BTreeMap::new(),
            power: PowerClock::new(&capture.program.marks.power, now)?,
        };
        let r = self.sustained.presence.networks.get(&key).unwrap_or(&new);
        if !self.presence_source_live(r) || !self.presence_eligible(r, target) {
            return Ok(());
        }
        let already = r
            .hosts
            .get(&target)
            .map(|h| h.born_action.as_ref() == Some(action));
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(2 * std::mem::size_of::<(ActionHandle, UnitInstanceId, uuid::Uuid)>()),
            "presence_hit_dedup",
        )?;
        self.sustained
            .presence
            .attempts
            .push((action.clone(), target, key));
        if let Some(new_this_action) = already {
            if !new_this_action {
                self.spread_presence(key, target, Some(action.clone()), now)?;
            }
            return Ok(());
        }
        // Counter-based roll: observation/log traffic never advances this stream.
        let mut seed = self.seed ^ action.ordinal().0.rotate_left(17);
        for b in key.as_bytes().iter().chain(target.as_bytes()) {
            seed = seed.wrapping_mul(0x100000001b3) ^ u64::from(*b);
        }
        seed = (seed ^ (seed >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        seed = (seed ^ (seed >> 27)).wrapping_mul(0x94d049bb133111eb);
        seed ^= seed >> 31;
        let p = capture.program.marks.chance_percent;
        let succeeds = u128::from(seed)
            < (u128::from(p.numerator) * (u128::from(u64::MAX) + 1))
                / (100 * u128::from(p.denominator));
        if !succeeds {
            return Ok(());
        }
        if !self.sustained.presence.networks.contains_key(&key) {
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(std::mem::size_of::<Network>() + 80),
                "presence_network",
            )?;
            self.sustained.presence.networks.insert(key, new);
        }
        self.infect_presence(key, target, Some(action.clone()), now)
    }
    pub(super) fn presence_death(
        &mut self,
        host: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        let keys: Vec<_> = self
            .sustained
            .presence
            .networks
            .iter()
            .filter(|(_, r)| self.presence_source_live(r) && r.hosts.contains_key(&host))
            .map(|(id, _)| *id)
            .collect();
        let action = self.runtime_fact_source.action.clone();
        for key in keys {
            let r = &self.sustained.presence.networks[&key];
            let born = r.hosts[&host].born_action.as_ref();
            let spread = !self.sustained.presence.suppress_spread
                && (action.is_none() || born != action.as_ref());
            // Settle while the dying host still belongs to the old population.
            let r = self.sustained.presence.networks.get_mut(&key).unwrap();
            let old = r.hosts.len();
            r.power.settle(&r.capture.program.marks.power, old, now)?;
            r.hosts.remove(&host);
            r.power.membership_changed(old, old - 1, now);
            self.event_queue
                .discard_presence_ticks(Some(key), Some(host))?;
            if spread {
                self.spread_presence(key, host, action.clone(), now)?;
            }
        }
        Ok(())
    }
    pub(super) fn presence_tick(
        &mut self,
        key: uuid::Uuid,
        host: UnitInstanceId,
        generation: u64,
        now: u64,
    ) -> Result<(), GameError> {
        self.reconcile_presence(now)?;
        let Some(r) = self.sustained.presence.networks.get(&key).filter(|r| {
            r.hosts
                .get(&host)
                .is_some_and(|h| h.generation == generation)
        }) else {
            return Ok(());
        };
        let p = &r.capture.program.marks;
        let amount = r.power.amount(
            r.power.value(&p.power, r.hosts.len(), now)?,
            p.periodic_coefficient,
        )?;
        let caster = r.caster;
        let fragment = r.capture.fragment.clone();
        let damage_type = p.periodic_damage_type;
        let interval = p.periodic_interval_ms;
        let prior = std::mem::replace(
            &mut self.runtime_fact_source,
            RuntimeFactSource {
                actor: Some(caster),
                fragment: Some(fragment),
                reaction: true,
                ..Default::default()
            },
        );
        let result = (|| {
            if amount > 0 {
                if let Some(snapshot) = self.damage_source_snapshot_for_unit(
                    caster,
                    host,
                    DamageSource::BuffTick,
                    damage_type,
                    amount,
                    DamageModifiers::default(),
                    0,
                    now,
                    false,
                ) {
                    self.execute_committed_damage(super::damage_runtime::CommittedDamageInput {
                        target_id: host,
                        source_snapshot: snapshot,
                        committed_at_ms: now,
                        policy: super::damage_runtime::CommittedDamagePolicy::GeneralCommand,
                    });
                }
            }
            control::poll("presence_periodic_damage")
        })();
        self.runtime_fact_source = prior;
        result?;
        if self.sustained.presence.networks.get(&key).is_some_and(|r| {
            r.hosts
                .get(&host)
                .is_some_and(|h| h.generation == generation)
        }) {
            let next_damage_at = now
                .checked_add(u64::from(interval))
                .ok_or(GameError::InvalidAction)?;
            self.sustained
                .presence
                .networks
                .get_mut(&key)
                .unwrap()
                .hosts
                .get_mut(&host)
                .unwrap()
                .next_damage_at = next_damage_at;
            self.event_queue.push(BattleEvent::PresenceTick {
                time_ms: next_damage_at,
                source: key,
                host,
                generation,
            });
        }
        Ok(())
    }
}

impl BattleCore {
    pub(super) fn has_prepared_presence_burst(&self, action: &ActionHandle) -> bool {
        self.sustained
            .presence
            .pending
            .iter()
            .any(|p| p.action == *action)
    }
    pub(super) fn presence_burst_skill(skill: &crate::game::ability::SkillDef) -> bool {
        skill
            .steps
            .iter()
            .flat_map(|s| &s.effects)
            .any(|e| matches!(e.op, SkillEffectDef::PresenceBurst { .. }))
    }
    pub(super) fn validate_presence_burst(
        &self,
        caster: UnitInstanceId,
        skill: &crate::game::ability::SkillDef,
    ) -> Result<(), GameError> {
        if !Self::presence_burst_skill(skill) {
            return Ok(());
        }
        if self
            .sustained
            .presence
            .pending
            .iter()
            .any(|p| p.caster == caster && p.capture.fragment.skill_id == skill.id)
        {
            return Ok(());
        }
        let key = self
            .units
            .get(&caster)
            .and_then(|u| u.fragment_source.as_ref())
            .filter(|f| f.skill_id == skill.id)
            .map(|f| f.copy_uuid)
            .ok_or(GameError::InvalidAction)?;
        let r = self
            .sustained
            .presence
            .networks
            .get(&key)
            .ok_or(GameError::InvalidAction)?;
        if !self.presence_source_live(r)
            || !r.hosts.keys().any(|id| {
                self.units
                    .get(id)
                    .is_some_and(|u| u.is_active() && u.stats.current_health > 0)
            })
        {
            return Err(GameError::InvalidAction);
        }
        Ok(())
    }
    pub(super) fn prepare_presence_burst(
        &mut self,
        caster: UnitInstanceId,
        skill: &crate::game::ability::SkillDef,
        action: &ActionHandle,
        now: u64,
    ) -> Result<(), GameError> {
        if !Self::presence_burst_skill(skill) {
            return Ok(());
        }
        self.reconcile_presence(now)?;
        self.validate_presence_burst(caster, skill)?;
        let key = self.units[&caster]
            .fragment_source
            .as_ref()
            .ok_or(GameError::InvalidAction)?
            .copy_uuid;
        let r = self
            .sustained
            .presence
            .networks
            .get(&key)
            .ok_or(GameError::InvalidAction)?;
        let value = r
            .power
            .value(&r.capture.program.marks.power, r.hosts.len(), now)?;
        let e = &r.capture.program.explosion;
        let coefficient = ratio_product(
            e.coefficient,
            ratio_sum(
                MarkRatio {
                    numerator: 1,
                    denominator: 1,
                },
                ratio_product(
                    e.bonus_percent,
                    MarkRatio {
                        numerator: 1,
                        denominator: 100,
                    },
                )?,
            )?,
        )?;
        let ordinary_damage = r.power.amount(value, coefficient)?;
        let host_damage = r
            .power
            .amount(value, ratio_product(coefficient, e.host_multiplier)?)?;
        control::charge(r.hosts.len() as u64, "presence_burst_snapshot")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(
                std::mem::size_of::<PendingBurst>()
                    + 128
                    + r.hosts.len()
                        * std::mem::size_of::<(UnitInstanceId, crate::game::resources::Position)>(),
            ),
            "presence_burst_reservation",
        )?;
        let mut centers: Vec<_> = r
            .hosts
            .keys()
            .filter(|id| self.units.get(id).is_some_and(|u| u.is_active()))
            .filter_map(|id| {
                self.sample_unit_body_at(*id, now)
                    .map(|body| (*id, body.projected_tile()))
            })
            .collect();
        centers.sort_by_key(|(id, p)| (p.y, p.x, *id));
        if centers.is_empty() {
            return Err(GameError::InvalidAction);
        }
        let pending = PendingBurst {
            action: action.clone(),
            caster,
            capture: r.capture.clone(),
            centers,
            ordinary_damage,
            host_damage,
        };
        self.sustained.presence.pending.push(pending);
        // Admission owns this finite release; periodic generations become stale.
        self.sustained.presence.networks.remove(&key);
        self.event_queue.discard_presence_ticks(Some(key), None)?;
        Ok(())
    }
    pub(super) fn execute_presence_burst(
        &mut self,
        caster: UnitInstanceId,
        program: &PresenceBurstProgram,
        now: u64,
    ) -> Result<super::types::SkillStepResult, GameError> {
        let action = self
            .runtime_fact_source
            .action
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        let index = self
            .sustained
            .presence
            .pending
            .iter()
            .position(|p| {
                p.caster == caster
                    && p.action == *action
                    && p.capture.program.marks.definition == program.marks.definition
            })
            .ok_or(GameError::InvalidAction)?;
        let p = self.sustained.presence.pending.remove(index);
        let owner = self
            .units
            .get(&caster)
            .ok_or(GameError::InvalidAction)?
            .owner;
        let mut hits = Vec::new();
        let mut wave: BTreeMap<UnitInstanceId, u32> = BTreeMap::new();
        for (host, center) in &p.centers {
            let tiles = p
                .capture
                .program
                .explosion
                .range
                .affected_tiles(
                    *center,
                    crate::game::battle::tile_range::FacingDirection::Up,
                )
                .map_err(GameError::InvalidStaticData)?;
            let targets = self.collect_tile_area_targets_at(
                now,
                owner,
                caster,
                &tiles,
                crate::game::ability::SkillHitTargetFilter::Enemies,
                false,
            );
            let host_hit = targets.contains(host);
            for target in targets {
                let amount = if target == *host {
                    p.host_damage
                } else {
                    p.ordinary_damage
                };
                if p.capture.program.explosion.dedup == ExplosionDedup::Explosion {
                    hits.push((target, amount));
                } else {
                    wave.entry(target)
                        .and_modify(|n| *n = (*n).max(amount))
                        .or_insert(amount);
                }
            }
            // Own-host damage is guaranteed even if a large/moving host's
            // sampled center no longer occupies the captured burst tile.
            if p.capture.program.explosion.dedup == ExplosionDedup::Wave {
                wave.insert(*host, p.host_damage);
            } else {
                // Each center contributes its host exactly once.
                if !host_hit {
                    hits.push((*host, p.host_damage));
                }
            }
        }
        if p.capture.program.explosion.dedup == ExplosionDedup::Wave {
            hits.extend(wave);
        }
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            hits.capacity()
                .checked_mul(std::mem::size_of::<(UnitInstanceId, u32)>()),
            "presence_burst_hits",
        )?;
        let old_suppression = std::mem::replace(&mut self.sustained.presence.suppress_spread, true);
        let result = (|| {
            let mut result = super::types::SkillStepResult::default();
            result.resolved_target_count = hits.len();
            for (target, amount) in hits {
                if amount == 0 || !self.units.get(&target).is_some_and(|u| u.is_active()) {
                    continue;
                }
                if let Some(snapshot) = self.damage_source_snapshot_for_unit(
                    caster,
                    target,
                    DamageSource::Ability,
                    p.capture.program.explosion.damage_type,
                    amount,
                    DamageModifiers::default(),
                    0,
                    now,
                    false,
                ) {
                    if let Some(summary) =
                        self.execute_committed_damage(super::damage_runtime::CommittedDamageInput {
                            target_id: target,
                            source_snapshot: snapshot,
                            committed_at_ms: now,
                            policy: super::damage_runtime::CommittedDamagePolicy::GeneralCommand,
                        })
                    {
                        result.actual_hp_loss =
                            result.actual_hp_loss.saturating_add(summary.actual_hp_loss);
                        result.actual_damage_target_count += summary.actual_damage_target_count;
                        result.effective_hit_count += summary.effective_hit_count;
                        result.killed_target_count += summary.killed_target_count;
                        result.applied_effect_count += 1;
                    }
                }
                control::poll("presence_burst_damage")?;
            }
            Ok(result)
        })();
        self.sustained.presence.suppress_spread = old_suppression;
        result
    }
}

#[cfg(test)]
mod tests;

impl BattleCore {
    pub fn presence_marks_snapshot(&self, now: u64) -> Result<PresenceMarksSnapshot, GameError> {
        control::charge(
            self.sustained.presence.entity_count() as u64,
            "presence_snapshot",
        )?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.sustained.presence.retained_extent(),
            "presence_snapshot_storage",
        )?;
        let networks = self
            .sustained
            .presence
            .networks
            .values()
            .map(|r| {
                Ok(PresenceNetworkView {
                    caster: r.caster,
                    fragment: (*r.capture.fragment).clone(),
                    definition: r.capture.program.marks.definition.clone(),
                    power_units: r
                        .power
                        .value(&r.capture.program.marks.power, r.hosts.len(), now)?
                        .to_string(),
                    power_scale: r.power.scale,
                    empty_since_ms: r.power.empty_since,
                    hosts: r
                        .hosts
                        .iter()
                        .map(|(unit, h)| PresenceHostView {
                            unit: *unit,
                            next_damage_at_ms: h.next_damage_at,
                        })
                        .collect(),
                    applied_modifiers: r.capture.applied.clone(),
                })
            })
            .collect::<Result<Vec<_>, GameError>>()?;
        let pending_bursts = self
            .sustained
            .presence
            .pending
            .iter()
            .map(|p| PendingPresenceBurstView {
                caster: p.caster,
                fragment_copy_uuid: p.capture.fragment.copy_uuid,
                action_ordinal: p.action.ordinal().0,
                centers: p.centers.clone(),
                ordinary_damage: p.ordinary_damage,
                host_damage: p.host_damage,
            })
            .collect();
        Ok(PresenceMarksSnapshot {
            networks,
            pending_bursts,
        })
    }
}
