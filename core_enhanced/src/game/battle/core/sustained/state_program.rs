use super::*;
#[cfg(test)]
mod tests;
use crate::game::{
    ability::{flow::FlowComparison, state_program::*},
    battle::{
        damage::{BattleCommand, DamageSource, DamageSourceSnapshot},
        runtime_control,
    },
    enums::Side,
    value_expr::{
        self as expr, CompiledAmountPlan, EvalFrame, ExprMeter, HitPointField, StatSelector,
        TimeValue, TypedInteger, ValueExprFault, ValueExprFaultCode, ValueExprPhase, ValueRef,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StateEventKey {
    Action(u64),
    Death(UnitInstanceId),
}
crate::authoring::clone_shape::copy_shape!(StateEventKey);

#[derive(Debug, Clone)]
pub(super) struct StateRecord {
    pub program: Arc<CompiledStateProgram>,
    pub fields: Vec<StateScalar>,
    pub seen: Vec<(usize, StateEventKey)>,
    pub events: u64,
}
crate::authoring::clone_shape::struct_shape!(StateRecord {
    program,
    fields,
    seen,
    events
});
impl StateRecord {
    pub(super) fn new(program: Arc<CompiledStateProgram>) -> Result<Self, GameError> {
        runtime_control::admit_retained_growth(
            0,
            program
                .retained_bytes
                .checked_add(program.fields.len() * std::mem::size_of::<StateScalar>()),
            "state_activation",
        )?;
        let fields = program.fields.iter().map(|f| f.initial).collect();
        Ok(Self {
            program,
            fields,
            seen: Vec::new(),
            events: 0,
        })
    }
    pub(super) fn retained_extent(&self) -> Option<usize> {
        self.program
            .retained_bytes
            .checked_add(
                self.fields
                    .capacity()
                    .checked_mul(std::mem::size_of::<StateScalar>())?,
            )?
            .checked_add(
                self.seen
                    .capacity()
                    .checked_mul(std::mem::size_of::<(usize, StateEventKey)>())?,
            )
    }
    pub(super) fn view(&self) -> StateView {
        StateView {
            definition: self.program.id.clone(),
            events: self.events,
            fields: self
                .program
                .fields
                .iter()
                .zip(&self.fields)
                .map(|(f, value)| StateFieldView {
                    name: f.name.clone(),
                    unit: f.unit.clone(),
                    value: *value,
                })
                .collect(),
        }
    }
}
#[derive(Clone, Copy)]
pub(super) enum StateEventInput {
    HpLost(bool, Option<u64>, u32, bool),
    BasicAttackKill(bool, u64, UnitInstanceId, Side),
    OwnerKill {
        victim: UnitInstanceId,
        had_owner_dot: bool,
        direct: Option<DamageSource>,
    },
    OwnerSkillCast,
    OwnerStunEnded { target: UnitInstanceId },
    Periodic,
    Expired,
    ManualInput,
}

enum ReactionEffect {
    Resonance {
        amount: i32,
        authored: Arc<crate::game::ability::authoring::EffectSource>,
        immediate: bool,
    },
    Heal(i32, Arc<crate::game::ability::authoring::EffectSource>),
    OwnerState {
        duration_ms: u32,
        profile: Arc<crate::game::ability::sustained::TargetStateProfile>,
        replacement: StateReplacement,
        source: Arc<crate::game::ability::authoring::EffectSource>,
    },
    EventTargetStatus {
        target: UnitInstanceId,
        duration_ms: u32,
        profile: Arc<crate::game::ability::sustained::HitStatusProfile>,
        source: Arc<crate::game::ability::authoring::EffectSource>,
    },
    Resources(
        Arc<Vec<crate::game::resource::ResourceOperation>>,
        Arc<crate::game::ability::authoring::EffectSource>,
    ),
}
struct Reaction {
    roll_seed: u64,
    cast_damage: Vec<(u32, Arc<crate::game::ability::authoring::EffectSource>)>,
    fields: Vec<StateScalar>,
    effects: Vec<ReactionEffect>,
    end: bool,
    invoke: Option<crate::game::ability::SkillId>,
}
fn field_fault(plan: &CompiledAmountPlan, cause: impl Into<String>) -> GameError {
    runtime_control::reject_expression(ValueExprFault {
        code: ValueExprFaultCode::Unavailable,
        site: plan.source().site.clone(),
        node_index: 0,
        expected_unit: None,
        actual_unit: None,
        phase: ValueExprPhase::Evaluate,
        cause: cause.into(),
    })
}
impl BattleCore {
    fn apply_state_reward(
        &mut self,
        caster: UnitInstanceId,
        target: UnitInstanceId,
        duration_ms: u32,
        profile: &crate::game::ability::sustained::TargetStateProfile,
        replacement: StateReplacement,
        now: u64,
    ) -> Result<(), GameError> {
        let Some(source_unit) = self.units.get(&caster) else {
            return Ok(());
        };
        if (profile.end_on_source_exit && !source_unit.is_active())
            || !self.units.get(&target).is_some_and(|u| u.is_active())
            || !self.stress_allied_effect_allowed(Some(caster), target)
        {
            return Ok(());
        }
        let side = source_unit.owner;
        if replacement == StateReplacement::ReplaceSameSource {
            loop {
                control::charge(
                    self.sustained.records.len() as u64,
                    "state_reward_replace_scan",
                )?;
                let source = &self.runtime_fact_source;
                let old = self
                    .sustained
                    .records
                    .iter()
                    .find(|(_, r)| {
                        r.caster == caster
                            && r.target == target
                            && r.target_state
                                .as_ref()
                                .is_some_and(|p| p.definition == profile.definition)
                            && r.source.fragment.as_ref().map(|f| f.copy_uuid)
                                == source.fragment.as_ref().map(|f| f.copy_uuid)
                            && r.source.equipment == source.equipment
                    })
                    .map(|(id, _)| *id);
                let Some(id) = old else {
                    break;
                };
                self.end_sustained(id, now, SustainedEndReason::Replaced)?;
            }
        }
        self.begin_sustained_owned(
            caster,
            target,
            SustainedEffectDef::TargetState {
                duration_ms,
                profile: crate::authoring::clone_shape::controlled_clone(profile)?,
            },
            now,
            None,
            Some(side),
            None,
        )
    }
    pub(in crate::game::battle::core) fn capture_state_basic(
        &self,
        snapshot: &mut DamageSourceSnapshot,
        primary: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        if snapshot.source != DamageSource::BasicAttack
            || snapshot.fact_source.reaction
            || snapshot.fact_source.action.is_none()
        {
            return Ok(());
        }
        control::charge(
            self.sustained.records.len() as u64,
            "state_basic_capture_scan",
        )?;
        let eligible = |r: &&Record| {
            r.target == snapshot.source_id
                && r.expires_at > now
                && r.state.as_ref().is_some_and(|s| {
                    s.program
                        .handlers
                        .iter()
                        .any(|h| matches!(h.event, StateEvent::BasicAttackKill { .. }))
                })
        };
        for record in self.sustained.records.values() {
            if let Some(state) = &record.state {
                control::charge(
                    (state.program.handlers.len() * 2) as u64,
                    "state_basic_handler_scan",
                )?;
            }
        }
        let count = self.sustained.records.values().filter(eligible).count();
        if count == 0 {
            return Ok(());
        }
        runtime_control::admit_retained_growth(
            0,
            count
                .checked_mul(std::mem::size_of::<u64>())
                .and_then(|bytes| {
                    bytes.checked_add(
                        std::mem::size_of::<crate::game::battle::damage::StateBasicCapture>() + 64,
                    )
                }),
            "state_basic_capture",
        )?;
        let mut activations = Vec::with_capacity(count);
        for (id, record) in &self.sustained.records {
            if eligible(&record) {
                activations.push(*id);
            }
        }
        snapshot.state_basic_capture =
            Some(Arc::new(crate::game::battle::damage::StateBasicCapture {
                primary,
                activations,
            }));
        Ok(())
    }
    pub(in crate::game::battle::core) fn authored_state_basic_kill(
        &mut self,
        incoming: &DamageSourceSnapshot,
        victim: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        if incoming.source != DamageSource::BasicAttack
            || incoming.fact_source.reaction
            || !self
                .units
                .get(&incoming.source_id)
                .is_some_and(|u| u.is_active())
            || !self
                .units
                .get(&victim)
                .is_some_and(|u| u.lifecycle.is_dead() && u.owner != incoming.source_side)
        {
            return Ok(());
        }
        let (Some(capture), Some(action)) =
            (&incoming.state_basic_capture, &incoming.fact_source.action)
        else {
            return Ok(());
        };
        for &id in &capture.activations {
            control::charge(1, "state_basic_kill_dispatch")?;
            if self
                .sustained
                .records
                .get(&id)
                .is_some_and(|r| r.target == incoming.source_id && r.expires_at > now)
            {
                self.run_state_event(
                    id,
                    StateEventInput::BasicAttackKill(
                        victim == capture.primary,
                        action.ordinal().0,
                        victim,
                        incoming.source_side,
                    ),
                    now,
                )?;
            }
        }
        Ok(())
    }
    pub(super) fn replace_authored_state(
        &mut self,
        caster: UnitInstanceId,
        target: UnitInstanceId,
        program: &CompiledStateProgram,
        source: &RuntimeFactSource,
        now: u64,
    ) -> Result<(), GameError> {
        if program.replacement == StateReplacement::Stack {
            return Ok(());
        }
        loop {
            control::charge(self.sustained.records.len() as u64, "state_replace_scan")?;
            let old = self
                .sustained
                .records
                .iter()
                .find(|(_, r)| {
                    r.caster == caster
                        && r.target == target
                        && r.state.as_ref().is_some_and(|s| s.program.id == program.id)
                        && r.source.fragment.as_ref().map(|f| f.copy_uuid)
                            == source.fragment.as_ref().map(|f| f.copy_uuid)
                        && r.source.equipment == source.equipment
                })
                .map(|(id, _)| *id);
            let Some(id) = old else {
                return Ok(());
            };
            self.end_sustained(id, now, SustainedEndReason::Replaced)?;
        }
    }
    pub(in crate::game::battle::core) fn authored_state_after_damage(
        &mut self,
        target: UnitInstanceId,
        incoming: &DamageSourceSnapshot,
        hp_lost: u32,
        now: u64,
    ) -> Result<(), GameError> {
        let action = incoming.fact_source.action.as_ref().map(|a| a.ordinal().0);
        if hp_lost == 0
            || !self
                .units
                .get(&target)
                .is_some_and(|u| u.is_active() && u.stats.current_health > 0)
        {
            return Ok(());
        }
        let direct = !incoming.fact_source.reaction
            && incoming.fact_source.guard_release().is_none()
            && incoming
                .fact_source
                .cast
                .as_ref()
                .is_none_or(|c| c.proc_origin.is_none())
            && incoming.source_id != target
            && matches!(
                incoming.source,
                DamageSource::BasicAttack | DamageSource::Ability
            );
        // Bound iteration to activations present at dispatch. A reaction cannot
        // cause a new subscription to consume the event currently being emitted.
        let last = self.sustained.next;
        let mut cursor = 0;
        loop {
            control::charge(self.sustained.records.len() as u64, "state_event_scan")?;
            let id = self
                .sustained
                .records
                .range((
                    std::ops::Bound::Excluded(cursor),
                    std::ops::Bound::Included(last),
                ))
                .find(|(_, r)| r.state.is_some() && r.target == target && r.expires_at > now)
                .map(|(id, _)| *id);
            let Some(id) = id else {
                break;
            };
            cursor = id;
            let hostile = self.units[&target].owner != incoming.source_side;
            self.run_state_event(
                id,
                StateEventInput::HpLost(hostile, action, hp_lost, direct),
                now,
            )?;
        }
        self.equipment_states_after_damage(target, incoming, hp_lost, direct, now)?;
        Ok(())
    }
    // Finite, prepared and equipped records share field/reaction semantics.
    // Only finite records supply a deadline to expression evaluation.
    fn state_event_context(
        &self,
        activation: u64,
    ) -> Option<(
        &StateRecord,
        &RuntimeFactSource,
        UnitInstanceId,
        UnitInstanceId,
        Option<u64>,
    )> {
        if let Some(record) = self.sustained.records.get(&activation) {
            return Some((
                record.state.as_ref()?,
                &record.source,
                record.caster,
                record.target,
                Some(record.expires_at),
            ));
        }
        if let Some(record) = self.sustained.equipped.get(&activation) {
            return Some((
                &record.state,
                &record.source,
                record.owner,
                record.owner,
                None,
            ));
        }
        let record = self.sustained.prepared.get(&activation)?;
        Some((
            record.state.as_ref()?,
            &record.source,
            record.caster,
            record.caster,
            None,
        ))
    }
    pub(super) fn run_state_event(
        &mut self, activation: u64, input: StateEventInput, now: u64,
    ) -> Result<(), GameError> {
        self.run_state_event_output(activation, input, now).map(|_| ())
    }
    pub(super) fn run_state_event_output(
        &mut self,
        activation: u64,
        input: StateEventInput,
        now: u64,
    ) -> Result<Vec<(u32, Arc<crate::game::ability::authoring::EffectSource>)>, GameError> {
        let mut output = Vec::new();
        let Some((state, ..)) = self.state_event_context(activation) else {
            return Ok(output);
        };
        let program = Arc::clone(&state.program);
        for (index, handler) in program.handlers.iter().enumerate() {
            control::charge(1, "state_event_handler")?;
            let Some((state, source, caster, target, expires)) =
                self.state_event_context(activation)
            else {
                break;
            };
            let dedup = match (handler.event, input) {
                (StateEvent::OwnerKill | StateEvent::OwnerDirectKill { .. }, StateEventInput::OwnerKill { victim, direct, .. }) => {
                    if let StateEvent::OwnerDirectKill { source: filter } = handler.event {
                        let matches = matches!((filter, direct),
                            (StateDirectKillSource::BasicAttack | StateDirectKillSource::BasicOrActiveSkill, Some(DamageSource::BasicAttack))
                            | (StateDirectKillSource::ActiveSkill | StateDirectKillSource::BasicOrActiveSkill, Some(DamageSource::Ability)));
                        if !matches { continue; }
                    }
                    let key = (index, StateEventKey::Death(victim));
                    control::charge(state.seen.len() as u64, "state_kill_dedup")?;
                    if state.seen.contains(&key) {
                        continue;
                    }
                    Some(key)
                }
                (StateEvent::OwnerStunEnded, StateEventInput::OwnerStunEnded { .. }) => None,
                (StateEvent::OwnerSkillCast, StateEventInput::OwnerSkillCast) => None,
                (StateEvent::Periodic { .. }, StateEventInput::Periodic)
                | (StateEvent::Expired, StateEventInput::Expired)
                | (StateEvent::ManualInput, StateEventInput::ManualInput) => None,
                (
                    StateEvent::BasicAttackKill {
                        primary_only,
                        once_per_action,
                        eligibility,
                        threat,
                    },
                    StateEventInput::BasicAttackKill(primary, action, victim, source_side),
                ) => {
                    if primary_only && !primary {
                        continue;
                    }
                    if threat.is_some_and(|class| {
                        !self
                            .units
                            .get(&victim)
                            .is_some_and(|u| u.threat_class == class)
                    }) || (eligibility == StateKillEligibility::EncounterCombatant
                        && !self.eligible_encounter_death(victim, source_side)?)
                    {
                        continue;
                    }
                    if once_per_action {
                        control::charge(state.seen.len() as u64, "state_event_dedup")?;
                        if state.seen.contains(&(index, StateEventKey::Action(action))) {
                            continue;
                        }
                        Some((index, StateEventKey::Action(action)))
                    } else {
                        None
                    }
                }
                (
                    StateEvent::HpLost {
                        relation,
                        once_per_action,
                    }
                    | StateEvent::DirectHpLost {
                        relation,
                        once_per_action,
                    },
                    StateEventInput::HpLost(hostile, action, _, direct),
                ) => {
                    if matches!(handler.event, StateEvent::DirectHpLost { .. }) && !direct {
                        continue;
                    }
                    if (relation == StateRelation::Hostile && !hostile)
                        || (relation == StateRelation::Friendly && hostile)
                    {
                        continue;
                    }
                    if once_per_action {
                        let Some(action) = action else { continue; };
                        control::charge(state.seen.len() as u64, "state_event_dedup")?;
                        if state.seen.contains(&(index, StateEventKey::Action(action))) {
                            continue;
                        }
                        Some((index, StateEventKey::Action(action)))
                    } else {
                        None
                    }
                }
                _ => continue,
            };
            if !self
                .units
                .get(&target)
                .is_some_and(|u| u.is_active() && u.stats.current_health > 0)
            {
                return Ok(output);
            }
            source.admit_owner("state_reaction_source")?;
            let source = source.clone();
            runtime_control::admit_retained_growth(
                0,
                Some(program.reaction_bytes),
                "state_reaction_plan",
            )?;
            let tag = |id: uuid::Uuid| { let n=id.as_u128(); (n as u64) ^ ((n >> 64) as u64).rotate_left(23) };
            let victim = match input { StateEventInput::OwnerKill { victim, .. } => tag(victim.as_uuid()), _ => 0 };
            let copy = source.equipment.map_or(activation, |e| tag(e.persistent_copy_uuid.unwrap_or(e.runtime_item_instance_id)));
            let mut reaction = Reaction {
                roll_seed: self.seed ^ tag(target.as_uuid()).rotate_left(7) ^ copy.rotate_left(19) ^ victim.rotate_left(37) ^ (index as u64),
                cast_damage: Vec::new(),
                fields: crate::authoring::clone_shape::controlled_clone(&state.fields)?,
                effects: Vec::new(),
                end: false,
                invoke: None,
            };
            self.prepare_state_ops(
                &program,
                &source,
                &handler.body,
                target,
                expires,
                Some(input),
                now,
                &mut reaction,
            )?;
            // Child qualification and capture happen before committing fields
            // or heals. Rejected targeting leaves this activation untouched.
            let child = reaction
                .invoke
                .as_ref()
                .map(|skill| self.prepare_sustained_child(activation, skill, None, now))
                .transpose()?;
            if dedup.is_some() {
                runtime_control::admit_retained_growth(
                    0,
                    Some(std::mem::size_of::<(usize, StateEventKey)>()),
                    "state_event_history",
                )?;
                self.admit_authoring_runtime_growth(1, "state_event_history")?;
            }
            let state = if let Some(record) = self.sustained.records.get_mut(&activation) {
                record.state.as_mut()
            } else if let Some(record) = self.sustained.equipped.get_mut(&activation) {
                Some(&mut record.state)
            } else {
                self.sustained
                    .prepared
                    .get_mut(&activation)
                    .and_then(|r| r.state.as_mut())
            }
            .expect("state unchanged during prepare");
            let events = state
                .events
                .checked_add(1)
                .ok_or_else(|| overflow("state_event_count"))?;
            state.fields = reaction.fields;
            state.events = events;
            if let Some(key) = dedup {
                state.seen.reserve_exact(1);
                state.seen.push(key);
            }
            output.extend(reaction.cast_damage);
            self.apply_reaction_effects(reaction.effects, &source, caster, target, now)?;
            if let Some(child) = child {
                self.commit_sustained_child(activation, child, now)?;
                break;
            }
            if reaction.end {
                // Natural expiry already owns cleanup; do not recursively
                // invoke the Expired handler or change its termination reason.
                match input {
                    StateEventInput::HpLost(..) | StateEventInput::BasicAttackKill(..) => {
                        if self.sustained.prepared.contains_key(&activation) {
                            self.end_prepared_basic(activation, now, SustainedEndReason::Depleted)?;
                        } else {
                            self.end_sustained(activation, now, SustainedEndReason::Depleted)?;
                        }
                    }
                    StateEventInput::ManualInput => {
                        self.end_sustained(activation, now, SustainedEndReason::Stopped)?
                    }
                    StateEventInput::Expired
                    | StateEventInput::Periodic
                    | StateEventInput::OwnerSkillCast
                    | StateEventInput::OwnerKill { .. }
                    | StateEventInput::OwnerStunEnded { .. } => (),
                }
                break;
            }
        }
        Ok(output)
    }
    fn apply_reaction_effects(
        &mut self,
        effects: Vec<ReactionEffect>,
        source: &RuntimeFactSource,
        caster: UnitInstanceId,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        for effect in effects {
            let (mut effect_source, authored) = match &effect {
                ReactionEffect::Heal(_, authored) | ReactionEffect::Resonance { authored, .. } => {
                    (source.observation(), authored)
                }
                ReactionEffect::Resources(_, authored) => (source.clone(), authored),
                ReactionEffect::OwnerState { source: authored, .. }
                | ReactionEffect::EventTargetStatus { source: authored, .. } => (source.independent_origin()?, authored),
            };
            effect_source.authored = Some(Arc::clone(authored));
            let prior = std::mem::replace(&mut self.runtime_fact_source, effect_source);
            let result = match effect {
                ReactionEffect::Resonance { amount, authored, immediate } => {
                    if immediate {
                        self.process_skill_commands(
                            vec![(BattleCommand::ModifyResonance {
                                target_id: target,
                                amount,
                                allow_autocast_when_full: false,
                            }, Some(authored))],
                            now,
                            caster,
                            self.runtime_fact_source.cast.clone(),
                        );
                        runtime_control::poll()
                    } else {
                        self.queue_equipment_resonance_reward(target, source, amount, authored)
                    }
                }
                ReactionEffect::Heal(amount, authored) => {
                    if !self
                        .units
                        .get(&target)
                        .is_some_and(|u| u.is_active() && u.stats.current_health > 0)
                    {
                        self.runtime_fact_source = prior;
                        continue;
                    }
                    let origin = self.runtime_fact_source.cast.clone();
                    self.process_skill_commands(
                        vec![(
                            BattleCommand::ApplyHeal {
                                target_id: target,
                                flat: amount,
                                percent: 0,
                                source_id: Some(caster),
                            },
                            Some(authored),
                        )],
                        now,
                        caster,
                        origin,
                    );
                    runtime_control::poll()
                }
                ReactionEffect::EventTargetStatus { target: event_target, duration_ms, profile, .. } => {
                    self.apply_event_target_status(caster, event_target, duration_ms, &profile, now)
                }
                ReactionEffect::Resources(operations, _) => {
                    self.apply_typed_resource_operations(&operations, &[target], now)
                }
                ReactionEffect::OwnerState {
                    duration_ms,
                    profile,
                    replacement,
                    ..
                } => {
                    self.apply_state_reward(caster, target, duration_ms, &profile, replacement, now)
                }
            };
            self.runtime_fact_source = prior;
            result?;
            control::poll("state_reaction_applied")?;
            runtime_control::poll()?;
        }
        Ok(())
    }
    pub(in crate::game::battle::core) fn run_equipment_reaction(
        &mut self,
        program: &CompiledStateProgram,
        source: &RuntimeFactSource,
        owner: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        runtime_control::admit_retained_growth(
            0,
            Some(program.reaction_bytes),
            "equipment_reaction_plan",
        )?;
        let mut reaction = Reaction {
            roll_seed: 0,
            cast_damage: Vec::new(),
            fields: Vec::new(),
            effects: Vec::new(),
            end: false,
            invoke: None,
        };
        self.prepare_state_ops(
            program,
            source,
            &program.handlers[0].body,
            owner,
            None,
            None,
            now,
            &mut reaction,
        )?;
        self.apply_reaction_effects(reaction.effects, source, owner, owner, now)
    }
    fn state_condition(
        &self,
        program: &CompiledStateProgram,
        source: &RuntimeFactSource,
        condition: &CompiledStateCondition,
        fields: &[StateScalar],
        target: UnitInstanceId,
        expires: Option<u64>,
        event: Option<StateEventInput>,
        now: u64,
    ) -> Result<bool, GameError> {
        control::charge(1, "state_condition")?;
        Ok(match condition {
            CompiledStateCondition::Always(value) => *value,
            CompiledStateCondition::EventVictimHadOwnerDot => match event {
                Some(StateEventInput::OwnerKill { had_owner_dot, .. }) => had_owner_dot,
                _ => return Err(GameError::InvalidAction),
            },
            CompiledStateCondition::Logic(logic) => logic.evaluate(|child| {
                self.state_condition(program, source, child, fields, target, expires, event, now)
            })?,
            CompiledStateCondition::Bool { field, equals } => {
                fields[*field] == StateScalar::Bool(*equals)
            }
            CompiledStateCondition::Compare {
                left,
                comparison,
                right,
            } => {
                let left = self
                    .state_amount(program, source, &fields, left, target, expires, event, now)?
                    .value;
                let right = self
                    .state_amount(program, source, &fields, right, target, expires, event, now)?
                    .value;
                match comparison {
                    FlowComparison::Equal => left == right,
                    FlowComparison::Greater => left > right,
                    FlowComparison::AtLeast => left >= right,
                    FlowComparison::Less => left < right,
                }
            }
        })
    }
    fn prepare_state_ops(
        &self,
        program: &CompiledStateProgram,
        source: &RuntimeFactSource,
        ops: &[CompiledStateOp],
        target: UnitInstanceId,
        expires: Option<u64>,
        event: Option<StateEventInput>,
        now: u64,
        reaction: &mut Reaction,
    ) -> Result<(), GameError> {
        for op in ops {
            control::charge(1, "state_operation")?;
            if reaction.end {
                break;
            }
            match op {
                CompiledStateOp::AddCastDamage { percent } => {
                    let value = self.state_amount(program, source, &reaction.fields, percent, target, expires, event, now)?.value;
                    if !(0..=10_000).contains(&value) { return Err(field_fault(percent, "cast damage must be from0 to10000 percent")); }
                    if value > 0 { reaction.cast_damage.push((value as u32, Arc::new(percent.source().clone()))); }
                }
                CompiledStateOp::Chance { percent, roll_site, then_ops, else_ops } => {
                    let value = self.state_amount(program, source, &reaction.fields, percent, target, expires, event, now)?.value;
                    if !(0..=100).contains(&value) { return Err(field_fault(percent, "chance must be from0 to100 percent")); }
                    let roll = crate::game::determinism::uuid_v4_from_seed(reaction.roll_seed, 0x5354415445434841, *roll_site).as_u128() % 100;
                    self.prepare_state_ops(program, source, if roll < value as u128 { then_ops } else { else_ops }, target, expires, event, now, reaction)?;
                }
                CompiledStateOp::Set { field, value } => {
                    let amount = self.state_amount(
                        program,
                        source,
                        &reaction.fields,
                        value,
                        target,
                        expires,
                        event,
                        now,
                    )?;
                    let schema = &program.fields[*field];
                    if amount.value < schema.min || amount.value > schema.max {
                        return Err(field_fault(
                            value,
                            format!("field {} outside declared bounds", schema.name),
                        ));
                    }
                    reaction.fields[*field] = StateScalar::Integer(amount.value);
                }
                CompiledStateOp::SetBool { field, value } => {
                    reaction.fields[*field] = StateScalar::Bool(*value)
                }
                CompiledStateOp::HealOwner { amount }
                | CompiledStateOp::GrantOwnerResonance { amount } => {
                    let value = self.state_amount(
                        program,
                        source,
                        &reaction.fields,
                        amount,
                        target,
                        expires,
                        event,
                        now,
                    )?;
                    let value = expr::narrow_heal(&amount.source().site, &value)
                        .map_err(runtime_control::reject_expression)?;
                    let authored = Arc::new(amount.source().clone());
                    reaction.effects.push(
                        if matches!(op, CompiledStateOp::GrantOwnerResonance { .. }) {
                            if value < 0 {
                                return Err(field_fault(
                                    amount,
                                    "resonance reward must be nonnegative",
                                ));
                            }
                            ReactionEffect::Resonance {
                                amount: value,
                                authored,
                                immediate: matches!(event, Some(StateEventInput::BasicAttackKill(..))),
                            }
                        } else {
                            ReactionEffect::Heal(value, authored)
                        },
                    );
                }
                CompiledStateOp::ApplyOwnerState {
                    duration,
                    profile,
                    replacement,
                } => {
                    let value = self.state_amount(
                        program,
                        source,
                        &reaction.fields,
                        duration,
                        target,
                        expires,
                        event,
                        now,
                    )?;
                    let duration_ms = u32::try_from(value.value)
                        .ok()
                        .filter(|v| *v > 0)
                        .ok_or_else(|| {
                            field_fault(
                                duration,
                                "state duration must be positive u32 milliseconds",
                            )
                        })?;
                    reaction.effects.push(ReactionEffect::OwnerState {
                        duration_ms,
                        profile: Arc::clone(profile),
                        replacement: *replacement,
                        source: Arc::new(duration.source().clone()),
                    });
                }
                CompiledStateOp::ApplyEventTargetStatus { duration, profile } => {
                    let Some(StateEventInput::OwnerStunEnded { target: event_target }) = event else {
                        return Err(runtime_control::invariant("state_event_target_unavailable"));
                    };
                    let value = self.state_amount(program, source, &reaction.fields, duration, target, expires, event, now)?;
                    let duration_ms = u32::try_from(value.value).ok().filter(|v| *v > 0)
                        .ok_or_else(|| field_fault(duration, "status duration must be positive u32 milliseconds"))?;
                    reaction.effects.push(ReactionEffect::EventTargetStatus {
                        target: event_target, duration_ms, profile: Arc::clone(profile),
                        source: Arc::new(duration.source().clone()),
                    });
                }
                CompiledStateOp::Resources { operations, source } => {
                    reaction.effects.push(ReactionEffect::Resources(
                        Arc::clone(operations),
                        Arc::clone(source),
                    ));
                }
                CompiledStateOp::If {
                    condition,
                    then_ops,
                    else_ops,
                } => {
                    let pass = self.state_condition(
                        program,
                        source,
                        condition,
                        &reaction.fields,
                        target,
                        expires,
                        event,
                        now,
                    )?;
                    self.prepare_state_ops(
                        program,
                        source,
                        if pass { then_ops } else { else_ops },
                        target,
                        expires,
                        event,
                        now,
                        reaction,
                    )?;
                }
                CompiledStateOp::End => reaction.end = true,
                CompiledStateOp::EndAndInvoke { skill } => {
                    reaction.end = true;
                    reaction.invoke = Some(skill.clone());
                }
            }
        }
        Ok(())
    }
    fn state_amount(
        &self,
        program: &CompiledStateProgram,
        source: &RuntimeFactSource,
        fields: &[StateScalar],
        plan: &CompiledAmountPlan,
        target: UnitInstanceId,
        expires: Option<u64>,
        event: Option<StateEventInput>,
        now: u64,
    ) -> Result<TypedInteger, GameError> {
        let refs = &plan.requirements().references;
        runtime_control::admit_retained_growth(
            0,
            refs.len()
                .checked_mul(std::mem::size_of::<(ValueRef, TypedInteger)>() + 128),
            "state_expression_frame",
        )?;
        let mut frame = EvalFrame::with_value_capacity(refs.len());
        if plan.requirements().state_owner_dot_targets {
            frame.set_state_owner_dot_targets(self.state_owner_dot_members(target, now, plan)?);
        }
        let owner = self
            .units
            .get(&target)
            .ok_or_else(|| field_fault(plan, "state owner unavailable"))?;
        for reference in refs {
            control::charge(program.fields.len() as u64 + 1, "state_expression_read")?;
            let value = match reference {
                ValueRef::StateField { name, .. } => {
                    let index = program
                        .fields
                        .iter()
                        .position(|f| &f.name == name)
                        .ok_or_else(|| field_fault(plan, "state schema mismatch"))?;
                    let StateScalar::Integer(value) = fields[index] else {
                        return Err(field_fault(plan, "non-numeric state read"));
                    };
                    value
                }
                ValueRef::Resource { resource, owner, field } => {
                    let lease = self.resolve_typed_resource_lease(resource, *owner, source, Some(target))
                        .map_err(|e| field_fault(plan, format!("state resource binding: {e:?}")))?;
                    let runtime = self.runtime_facts.resources.as_ref()
                        .ok_or_else(|| field_fault(plan, "state resources unavailable"))?;
                    let value = runtime.store.value(&lease)
                        .map_err(|e| field_fault(plan, format!("state resource read: {e:?}")))?;
                    use crate::game::value_expr::ResourceValueField;
                    i128::from(match field {
                        ResourceValueField::Balance => value.balance,
                        ResourceValueField::Reserved => value.reserved,
                        ResourceValueField::Available => value.available(),
                    })
                }
                ValueRef::EventHpLost => i128::from(match event {
                    Some(StateEventInput::HpLost(_, _, amount, _)) => amount,
                    _ => return Err(field_fault(plan, "missing HP event")),
                }),
                ValueRef::HitPoints { field, .. } => i128::from(match field {
                    HitPointField::Current => owner.stats.current_health,
                    HitPointField::Max => owner.stats.max_health,
                    HitPointField::Missing => owner
                        .stats
                        .max_health
                        .checked_sub(owner.stats.current_health)
                        .ok_or_else(|| field_fault(plan, "invalid owner HP"))?,
                }),
                ValueRef::RelationCount { relation, .. } => i128::from(self.relation_count(target, *relation)?),
                ValueRef::Stat { stat, .. } => match stat {
                    StatSelector::Attack => i128::from(owner.stats.attack),
                    StatSelector::SpellPower => i128::from(owner.stats.spell_power),
                    StatSelector::Defense => i128::from(owner.stats.defense),
                    StatSelector::AttackSpeedPoints => i128::from(owner.basic_attack_speed_percent),
                },
                ValueRef::Time(TimeValue::BattleElapsed) => i128::from(now),
                ValueRef::Time(TimeValue::RemainingTime) => i128::from(
                    expires
                        .ok_or_else(|| field_fault(plan, "state has no deadline"))?
                        .saturating_sub(now),
                ),
                _ => return Err(field_fault(plan, "unsupported state context")),
            };
            frame
                .insert(
                    reference.clone(),
                    TypedInteger::new(value, reference.unit()),
                )
                .map_err(|e| field_fault(plan, e))?;
        }
        let mut meter = ExprMeter::new(plan.evaluation_limits())
            .map_err(|e| field_fault(plan, e))?
            .with_host_admission(super::super::value_expr_runtime::evaluation_host_admission);
        expr::evaluate(plan, &frame, &mut meter)
            .map(|v| v.value)
            .map_err(runtime_control::reject_expression)
    }
}
