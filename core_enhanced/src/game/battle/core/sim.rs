use crate::authoring::control::{self, Resource};
use uuid::Uuid;

use crate::game::battle::event_log::BattleEventLogEntry;
use crate::{
    game::resources::Position,
    game::{
        ability::{
            DeliveryDef, SkillActivationMode, SkillCastSelection, SkillDef, SkillEffectDef,
            SkillId, SkillStepCondition, SkillStepDef, SkillStepRepeat, SkillTarget,
            SkillTargetReach, SkillUnitReference, StepTargetingMode,
        },
        battle::{
            cooldown::CooldownSource,
            damage::{BattleCommand, DamageSource, ProcRollIdentity},
            enums::BattleEvent,
            event_log::{
                BattleEventCause, BattleEventLog, BattleEventRootCause, BattleLogEvent,
                BuffExpireReason, SkillCastTarget,
            },
            ids::UnitInstanceId,
            scenario::{ScenarioAction, ScenarioTrigger, TerminalCondition, TerminalOutcome},
            types::{
                BattleOutcomeId, BattleResult, BattleUnitDraft, BattleUnitSourceIdentity,
                BattleVerdict, BattleWinner, EngagementStance, ParticipantBattleResult,
                TacticalTileKind,
            },
        },
        behavior::{
            ActionDisabledReasonCode, BattlePayloadEligibilityDto, GameError,
            LiveBattleSkillReadinessDto,
        },
        determinism,
        enums::Side,
        stats::UnitStats,
    },
};

fn side_sort_key(side: Side) -> u8 {
    match side {
        Side::Player => 0,
        Side::Opponent => 1,
    }
}

fn cast_start_seq_from_cause(cause: BattleEventCause) -> Option<u64> {
    match cause {
        BattleEventCause::Parent { seq } => Some(seq),
        BattleEventCause::Root { .. } => None,
    }
}

use super::{
    skill_runtime::cast::{PreviousStepDamageGate, SkillTargetUsefulness},
    triggers::TriggerOccurrenceContext,
    types::{AbilityProcKey, PendingSkillCast, SkillStepResult},
    ActiveBuff, BattleCore, BuffInstanceKey, CompendiumCandidate, CompendiumCandidateKind,
};

pub(super) fn allocate_evaluated_amounts(
    count: usize,
) -> Result<Vec<super::value_expr_runtime::RuntimeAmount>, GameError> {
    super::super::runtime_control::admit_retained_growth(
        0,
        count.checked_mul(std::mem::size_of::<super::value_expr_runtime::RuntimeAmount>()),
        "value_expr_evaluated_amounts",
    )?;
    Ok(Vec::with_capacity(count))
}

#[derive(Debug, Clone)]
pub struct BattleExecutionState {
    last_event_time_ms: u64,
    battle_start_hooks_ran: bool,
    terminal_outcome: Option<BattleFinishSignal>,
    finalized: bool,
}

impl BattleExecutionState {
    pub fn last_event_time_ms(&self) -> u64 {
        self.last_event_time_ms
    }

    pub fn is_finished(&self) -> bool {
        self.terminal_outcome.is_some()
    }

    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub fn is_finalized(&self) -> bool {
        self.finalized
    }

    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub fn terminal_outcome(&self) -> Option<BattleFinishSignal> {
        self.terminal_outcome.clone()
    }

    pub fn next_time_after(&self, delta_ms: u64) -> u64 {
        self.last_event_time_ms.saturating_add(delta_ms)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleFinishSignal {
    pub time_ms: u64,
    pub outcome_id: BattleOutcomeId,
    pub verdict: BattleVerdict,
    pub winner: BattleWinner,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleStepOutcome {
    Running,
    Finished(BattleFinishSignal),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleDeployCurrentHpPolicy {
    FixedCurrentHp(u32),
}

impl BattleDeployCurrentHpPolicy {
    pub const WITHDRAW_REDEPLOY_RECOVERY_PERCENT: u32 = 30;

    pub fn withdraw_redeploy(current_hp: u32, max_hp: u32) -> Self {
        let recovery =
            (u64::from(max_hp) * u64::from(Self::WITHDRAW_REDEPLOY_RECOVERY_PERCENT) / 100) as u32;
        let recovered_hp = current_hp.saturating_add(recovery);
        BattleDeployCurrentHpPolicy::FixedCurrentHp(recovered_hp.min(max_hp))
    }

    pub fn current_hp_for_max(self, max_hp: u32) -> u32 {
        if max_hp == 0 {
            return 0;
        }
        let current_hp = match self {
            BattleDeployCurrentHpPolicy::FixedCurrentHp(current_hp) => current_hp,
        };
        current_hp.max(1).min(max_hp)
    }
}

#[derive(Debug, Clone)]
pub enum BattleLiveCommand {
    DeployPlayerUnit {
        draft: BattleUnitDraft,
        position: Position,
        tile_kind: TacticalTileKind,
        facing: crate::game::battle::tile_range::FacingDirection,
        instance_salt: u32,
        time_ms: u64,
        current_hp_policy: Option<BattleDeployCurrentHpPolicy>,
    },
    WithdrawUnit {
        unit_id: UnitInstanceId,
        time_ms: u64,
    },
    SetEngagementStance {
        unit_id: UnitInstanceId,
        stance: EngagementStance,
        time_ms: u64,
    },
    TacticalRelocateUnit {
        unit_id: UnitInstanceId,
        destination: Position,
        tile_kind: TacticalTileKind,
        facing: crate::game::battle::tile_range::FacingDirection,
        time_ms: u64,
    },
    TacticalSwapUnits {
        first_unit_id: UnitInstanceId,
        second_unit_id: UnitInstanceId,
        first_destination_tile_kind: TacticalTileKind,
        second_destination_tile_kind: TacticalTileKind,
        first_facing: crate::game::battle::tile_range::FacingDirection,
        second_facing: crate::game::battle::tile_range::FacingDirection,
        time_ms: u64,
    },
    ReinputSustainedSkill {
        unit_id: UnitInstanceId,
        skill_id: SkillId,
        activation: u64,
        target: Option<SkillCastTarget>,
        time_ms: u64,
    },
    CommitInputPlan {unit_id:UnitInstanceId,input:crate::game::ability::input_plan::ShotCommitInput,time_ms:u64},
    CancelInputPlan {unit_id:UnitInstanceId,preparation:u64,disconnected:bool,time_ms:u64},
    SelectRelation { unit_id:UnitInstanceId, entry:String, target:UnitInstanceId, time_ms:u64 },
    ActivateSkill {
        unit_id: UnitInstanceId,
        skill_id: SkillId,
        target: Option<SkillCastTarget>,
        time_ms: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleLiveCommandOutcome {
    UnitDeployed {
        unit_id: UnitInstanceId,
    },
    UnitWithdrawn {
        unit_id: UnitInstanceId,
    },
    EngagementStanceChanged {
        unit_id: UnitInstanceId,
        stance: EngagementStance,
    },
    UnitTacticallyRelocated {
        unit_id: UnitInstanceId,
    },
    UnitsTacticallySwapped {
        first_unit_id: UnitInstanceId,
        second_unit_id: UnitInstanceId,
    },
    InputPlanPrepared {unit_id:UnitInstanceId,preparation:u64},
    InputPlanCommitted {unit_id:UnitInstanceId,receipt:crate::game::ability::input_plan::ShotCommitReceipt},
    InputPlanCancelled {unit_id:UnitInstanceId},
    RelationChanged { unit_id:UnitInstanceId, target:Option<UnitInstanceId> },
    SkillActivated {
        unit_id: UnitInstanceId,
    },
}

#[derive(Debug, Clone)]
pub enum BattleLiveSignal {
    TacticalResourceDelta {
        fact_source: crate::game::battle::runtime_facts::RuntimeFactSource,
        time_ms: u64,
        retained_extent: usize,
        source_unit_id: UnitInstanceId,
        skill_id: SkillId,
        amount: i32,
    },
}

impl PartialEq for BattleLiveSignal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::TacticalResourceDelta {
                    source_unit_id: a,
                    skill_id: b,
                    amount: c,
                    ..
                },
                Self::TacticalResourceDelta {
                    source_unit_id: x,
                    skill_id: y,
                    amount: z,
                    ..
                },
            ) => (a, b, c) == (x, y, z),
        }
    }
}
impl Eq for BattleLiveSignal {}

crate::authoring::clone_shape::struct_shape!(BattleExecutionState {
    last_event_time_ms,
    battle_start_hooks_ran,
    terminal_outcome,
    finalized,
});
crate::authoring::clone_shape::struct_shape!(BattleFinishSignal {
    time_ms,
    outcome_id,
    verdict,
    winner
});
crate::authoring::clone_shape::copy_shape!(BattleWinner);
crate::authoring::clone_shape::enum_shape!(BattleLiveSignal {
    units: [],
    tuples: [],
    records: [TacticalResourceDelta {
        fact_source,
        time_ms,
        retained_extent,
        source_unit_id,
        skill_id,
        amount
    }]
});

#[derive(Debug, Clone)]
pub(super) struct TriggeredAbilityProcContext<'a> {
    pub(super) proc_roll_identity: &'a ProcRollIdentity,
    pub(super) current_time_ms: u64,
    pub(super) proc_chance_percent: u8,
    pub(super) internal_cooldown_ms: u64,
    pub(super) max_triggers_per_battle: Option<u32>,
}

#[derive(Debug, Clone)]
struct SkillStepExecution<'a> {
    time_ms: u64,
    cast_seq: u64,
    step_index: usize,
    caster_instance_id: UnitInstanceId,
    skill: &'a SkillDef,
    step: &'a SkillStepDef,
    cast_target: Option<SkillCastTarget>,
    cause: BattleEventCause,
}

impl BattleCore {
    pub(super) fn movement_tick_ms(&self) -> u64 {
        self.game_data
            .run_policy
            .battle_runtime
            .movement_tick_ms
            .max(1)
    }

    fn has_pending_attack_resolve_at(&self, time_ms: u64) -> bool {
        self.event_queue.iter().any(|event| {
            matches!(
                event.as_ref(),
                BattleEvent::AttackResolve { time_ms: event_time_ms, .. }
                | BattleEvent::BasicHealResolve { time_ms: event_time_ms, .. } if *event_time_ms == time_ms
            )
        })
    }

    fn proc_unit_tag(unit_id: UnitInstanceId) -> u64 {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&unit_id.as_bytes()[..8]);
        u64::from_be_bytes(bytes)
    }

    fn proc_uuid_tag(uuid: Uuid) -> u64 {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&uuid.as_bytes()[..8]);
        u64::from_be_bytes(bytes)
    }

    fn proc_cooldown_source_tag(source: CooldownSource) -> u64 {
        match source {
            CooldownSource::Unit { unit_instance_id } => Self::proc_unit_tag(unit_instance_id),
            CooldownSource::Item { item_instance_id } => Self::proc_uuid_tag(item_instance_id),
        }
    }

    fn proc_trigger_type_tag(trigger_type: crate::game::stats::TriggerType) -> u64 {
        match trigger_type {
            crate::game::stats::TriggerType::Permanent => 0x5045_524D_414E_454Eu64,
            crate::game::stats::TriggerType::OnAttack => 0x4F4E_4154_5441_434Bu64,
            crate::game::stats::TriggerType::OnHit => 0x4F4E_4849_545F_5F5Fu64,
            crate::game::stats::TriggerType::OnKill => 0x4F4E_4B49_4C4C_5F5Fu64,
            crate::game::stats::TriggerType::OnDeath => 0x4F4E_4445_4154_48u64,
            crate::game::stats::TriggerType::OnBattleStart => 0x4241_5454_4C45_5354u64,
            crate::game::stats::TriggerType::OnAllyDeath => 0x414C_4C59_4445_4144u64,
        }
    }

    fn proc_skill_id_tag(skill_id: &SkillId) -> u64 {
        skill_id.as_str().bytes().fold(0u64, |acc, b| {
            acc.wrapping_mul(131).wrapping_add(u64::from(b))
        })
    }

    pub(in crate::game::battle::core) fn proc_roll_percent(
        &self,
        identity: &ProcRollIdentity,
    ) -> u8 {
        const PROC_ROLL_NS: u64 = 0x5052_4F43_524F_4C4Cu64; // "PROCROLL"

        let seed = self.seed
            ^ Self::proc_trigger_type_tag(identity.trigger_type).rotate_left(3)
            ^ Self::proc_cooldown_source_tag(identity.activation_source).rotate_left(13)
            ^ Self::proc_skill_id_tag(&identity.ability_id).rotate_left(29)
            ^ (identity.binding_index as u64).rotate_left(7)
            ^ Self::proc_unit_tag(identity.caster_id).rotate_left(11)
            ^ Self::proc_unit_tag(identity.trigger_unit_id).rotate_left(17)
            ^ Self::proc_uuid_tag(identity.occurrence_id).rotate_left(41)
            ^ u64::from(identity.occurrence_index).wrapping_mul(0xD1B5_4A32_D192_ED03);
        let mut seed = seed;
        if let Some(counterpart_unit_id) = identity.counterpart_unit_id {
            seed ^= Self::proc_unit_tag(counterpart_unit_id).rotate_left(23);
        }
        if let Some(target_id) = identity.target_id {
            seed ^= Self::proc_unit_tag(target_id).rotate_left(37);
        }

        determinism::uuid_v4_from_seed(seed, PROC_ROLL_NS, u64::from(identity.occurrence_index))
            .as_bytes()[0]
            % 100
    }

    pub(super) fn should_fire_triggered_ability(
        &mut self,
        context: TriggeredAbilityProcContext<'_>,
    ) -> bool {
        let clamped_chance = context.proc_chance_percent.min(100);
        if clamped_chance == 0 {
            return false;
        }

        let key = AbilityProcKey {
            source: context.proc_roll_identity.activation_source,
            ability_id: context.proc_roll_identity.ability_id.clone(),
            binding_index: context.proc_roll_identity.binding_index,
        };
        let current_count = self
            .ability_proc_states
            .get(&key)
            .map(|state| state.trigger_count)
            .unwrap_or(0);
        let next_ready_ms = self
            .ability_proc_states
            .get(&key)
            .map(|state| state.next_ready_ms)
            .unwrap_or(0);

        if context.current_time_ms < next_ready_ms {
            return false;
        }
        if context
            .max_triggers_per_battle
            .is_some_and(|max| current_count >= max)
        {
            return false;
        }

        let roll = self.proc_roll_percent(context.proc_roll_identity);
        if roll >= clamped_chance {
            return false;
        }

        let state = self.ability_proc_states.entry(key).or_default();
        state.trigger_count = state.trigger_count.saturating_add(1);
        state.next_ready_ms = context
            .current_time_ms
            .saturating_add(context.internal_cooldown_ms);
        true
    }

    pub(super) fn invoke_ability(
        &mut self,
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        skill_id: &str,
        explicit_target: Option<SkillCastTarget>,
        cause: BattleEventCause,
        allow_dead_caster: bool,
    ) {
        self.invoke_ability_with_accepted_target_tile(
            time_ms,
            caster_instance_id,
            skill_id,
            (explicit_target, None, None),
            cause,
            allow_dead_caster,
            None,
            None,
        );
    }

    pub(super) fn invoke_reinput_child(
        &mut self, time_ms: u64, caster: UnitInstanceId, skill: &str,
        target: Option<SkillCastTarget>, permit: super::sustained::ReinputChildPermit,
    ) {
        self.invoke_ability_with_accepted_target_tile(time_ms, caster, skill, (target, None, None), Default::default(), false, None, Some(&permit));
    }

    fn invoke_ability_with_accepted_target_tile(
        &mut self,
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        skill_id: &str,
        target_snapshot: (Option<SkillCastTarget>, Option<Position>, Option<crate::game::battle::tile_range::FacingDirection>),
        cause: BattleEventCause,
        allow_dead_caster: bool,
        accepted_action: Option<crate::game::battle::runtime_facts::ActionHandle>,
        reinput_permit: Option<&super::sustained::ReinputChildPermit>,
    ) {
        let (explicit_target, accepted_target_tile, accepted_facing) = target_snapshot;
        let inherited_source = accepted_action
            .as_ref()
            .and_then(|action| self.runtime_facts.query(action).ok())
            .map(|snapshot| snapshot.source.as_ref().clone())
            .unwrap_or_else(|| self.runtime_fact_source.clone());
        let Ok(effective_source) = self.bind_effective_skill_source(
            &SkillId::from(skill_id),
            caster_instance_id,
            &inherited_source,
        ) else {
            return;
        };
        let Ok(Some(skill)) =
            self.cloned_skill_for_source(&SkillId::from(skill_id), &effective_source)
        else {
            if let Some(action) = &accepted_action {
                let _ = self.runtime_facts.terminal(action,
                    crate::game::battle::runtime_facts::ActionTerminalKind::Failed(crate::game::battle::runtime_facts::ActionFailureReason::DefinitionUnavailable), time_ms);
            }
            return;
        };

        if skill.resonance_admission.is_some()
            && !reinput_permit.is_some_and(|permit| permit.permits(caster_instance_id, &skill, time_ms))
            && !accepted_action.as_ref().is_some_and(|action|
                self.has_resonance_use_credential(caster_instance_id, &skill.id, action)) {
            return;
        }

        if explicit_target.is_none() {
            match self.requires_explicit_structural_selection(
                &skill,
                caster_instance_id,
                &effective_source,
            ) {
                Ok(false) => {}
                Ok(true) => {
                    if let Some(action) = &accepted_action {
                        let _ = self.runtime_facts.terminal(action,
                            crate::game::battle::runtime_facts::ActionTerminalKind::Failed(
                                crate::game::battle::runtime_facts::ActionFailureReason::OriginUnavailable), time_ms);
                    }
                    return;
                }
                Err(_) => return,
            }
        }

        let flow_plan =
            if let Some(binding) = effective_source.causal().and_then(|c| c.program.as_ref()) {
                binding.execution().flow.clone()
            } else {
                match effective_source.relic_effective() {
                    Some(binding) => binding.execution().flow.clone(),
                    None => match self.game_data.skill_data.compiled_flow(skill_id) {
                        Ok(plan) => plan,
                        Err(_) => return,
                    },
                }
            };
        if time_ms
            .checked_add(flow_plan.authored_delay_bound_ms)
            .is_none()
        {
            let _ = crate::game::battle::runtime_control::reject(
                crate::game::battle::runtime_control::BattleRuntimeFault {
                    code: crate::game::battle::runtime_control::RuntimeFaultCode::CounterOverflow,
                    site: "flow_schedule_horizon",
                    limit: None,
                    requested: None,
                },
            );
            return;
        }
        if !flow_plan.legacy
            && crate::game::battle::runtime_control::limit(
                crate::game::battle::runtime_control::RuntimeFaultCode::ControlStepLimit,
                "flow_control_nodes",
                self.runtime_facts.limits.max_control_steps,
                Some(flow_plan.nodes.len()),
            )
            .is_err()
        {
            return;
        }
        if self
            .admit_authoring_runtime_growth(flow_plan.nodes.len(), "flow_state_admission")
            .is_err()
        {
            return;
        }

        let Some((caster_owner, caster_pos)) =
            self.resolve_cast_origin_context(None, caster_instance_id, allow_dead_caster)
        else {
            if let Some(action) = &accepted_action {
                let _ = self.runtime_facts.terminal(
                    action,
                    crate::game::battle::runtime_facts::ActionTerminalKind::Failed(
                        crate::game::battle::runtime_facts::ActionFailureReason::CasterUnavailable,
                    ),
                    time_ms,
                );
            }
            return;
        };
        let cast_target = explicit_target.or_else(|| {
            self.resolve_skill_cast_target_for_origin(
                time_ms,
                &skill,
                caster_instance_id,
                caster_owner,
                caster_pos,
                allow_dead_caster,
            )
        });

        let Ok(flow) = super::skill_runtime::flow::FlowState::new(flow_plan.clone(), cast_target)
        else {
            return;
        };
        let target_instance_id = match cast_target {
            Some(SkillCastTarget::Unit { unit_instance_id }) => Some(unit_instance_id),
            _ => None,
        };
        let cast_target_anchor_position = accepted_target_tile.or_else(|| match cast_target {
            Some(SkillCastTarget::Tile { position }) => Some(position),
            Some(SkillCastTarget::Unit { unit_instance_id }) => {
                self.live_unit_projected_tile(unit_instance_id)
            }
            None | Some(SkillCastTarget::TilePair { .. } | SkillCastTarget::Direction { .. }) => None,
        });
        let cast_target_anchor_world_position = accepted_target_tile
            .map(crate::game::battle::core::movement::types::WorldVec2::from_tile_center)
            .or_else(|| match cast_target {
                Some(SkillCastTarget::Tile { position }) => Some(
                    crate::game::battle::core::movement::types::WorldVec2::from_tile_center(
                        position,
                    ),
                ),
                Some(SkillCastTarget::Unit { unit_instance_id }) => {
                    self.unit_world_position_or_tile_center(unit_instance_id)
                }
                None | Some(SkillCastTarget::TilePair { .. } | SkillCastTarget::Direction { .. }) => None,
            });

        if self
            .admit_authoring_runtime_growth(1, "skill_cast_spawn")
            .is_err()
        {
            return;
        }
        let parent_resource_context = self.resource_context_for_source(&self.runtime_fact_source);
        let action = match accepted_action {
            Some(action) => action,
            None => {
                let source = crate::game::battle::runtime_facts::RuntimeFactSource {
                    extended_observation: match self.independent_action_origin(caster_instance_id).and_then(|origin| crate::game::battle::runtime_facts::RuntimeFactSource::extended_observation(origin, self.runtime_fact_source.reinput().cloned())) { Ok(value) => value, Err(_) => return },
                    actor: Some(caster_instance_id),
                    equipment: self.runtime_fact_source.equipment,
                    reaction: self.runtime_fact_source.reaction,
                    fragment: if self.runtime_fact_source.reinput().is_some() || self.runtime_fact_source.reaction { self.runtime_fact_source.fragment.clone() } else { self.units.get(&caster_instance_id).and_then(|unit| unit.fragment_source.as_ref()).filter(|source| self.fragment_owns_skill(&source.skill_id, &skill.id) && self.runtime_fact_source.equipment.is_none()).cloned() },
                    resource_capabilities: match crate::game::battle::runtime_facts::RuntimeFactSource::resource_capabilities(parent_resource_context, None) { Ok(value) => value, Err(_) => return },
                    ..effective_source.clone()
                };
                let source = if effective_source.causal().is_some() {
                    match source.with_causal(effective_source.causal().cloned()) {
                        Ok(source) => source,
                        Err(_) => return,
                    }
                } else {
                    source
                };
                let source = match effective_source.relic_effective() {
                    Some(binding) => match source.with_relic_effective(binding.clone()) {
                        Ok(value) => value,
                        Err(_) => return,
                    },
                    None => source,
                };
                let source = match source.with_cast_damage(self.runtime_fact_source.cast_damage().cloned()) {
                    Ok(source) => source, Err(_) => return,
                };
                match self.runtime_facts.accept(
                    caster_instance_id,
                    Some(&skill.id),
                    source,
                    time_ms,
                    false,
                ) {
                    Ok(action) => action,
                    Err(_) => return,
                }
            }
        };
        if Self::tile_burst_skill(&skill) && !self.has_prepared_tile_burst(&action) {
            if self.prepare_tile_burst(caster_instance_id, &skill, &action, time_ms).is_err() { return; }
        }
        if Self::presence_burst_skill(&skill) && !self.has_prepared_presence_burst(&action) {
            if self.prepare_presence_burst(caster_instance_id, &skill, &action, time_ms).is_err() { return; }
        }
        if self
            .runtime_facts
            .release(
                &action,
                if flow_plan.legacy {
                    skill.steps.len()
                } else {
                    1
                },
                time_ms,
            )
            .is_err()
        {
            return;
        }
        let cast_seq = self.skill_cast_seq;
        self.skill_cast_seq = self.skill_cast_seq.wrapping_add(1);
        let ability_event_seq = self.with_recording_context(cause, |core| {
            core.record_event_log(
                time_ms,
                BattleLogEvent::AbilityCast {
                    skill_id: skill.id.clone(),
                    caster_instance_id,
                    target_instance_id,
                },
            )
        });

        if crate::game::battle::runtime_control::admit_retained_growth(
            0,
            std::mem::size_of::<crate::game::battle::skill_source::SkillCastExecutionSource>()
                .checked_add(skill.id.as_str().len()),
            "cast_source_owner",
        )
        .is_err()
        {
            return;
        }
        let execution_source = std::sync::Arc::new(
            crate::game::battle::skill_source::SkillCastExecutionSource {
                skill_id: skill.id.clone(),
                caster_instance_id,
                cast_seq,
                ability_event_seq,
                cause,
                proc_origin: self.recording_skill_proc_origin.clone(),
            },
        );
        if self
            .runtime_facts
            .bind_cast(&action, &execution_source)
            .is_err()
        {
            return;
        }
        let accepted_cast_damage = match self.runtime_facts.accepted_source(&action) {
            Ok(source) => source.cast_damage().cloned(), Err(_) => return,
        };
        let inherited_fragment = (self.runtime_fact_source.reinput().is_some()
            || self.runtime_fact_source.reaction)
            .then(|| self.runtime_fact_source.fragment.clone())
            .flatten();
        let fact_source = crate::game::battle::runtime_facts::RuntimeFactSource {
            extended_observation:
                match crate::game::battle::runtime_facts::RuntimeFactSource::extended_observation(
                    self.runtime_facts.independent_origin_for_action(&action),
                    self.runtime_fact_source.reinput().cloned(),
                ) {
                    Ok(value) => value,
                    Err(_) => return,
                },
            actor: Some(caster_instance_id),
            reaction: self.runtime_fact_source.reaction,
            equipment: effective_source.equipment,
            fragment: effective_source
                .fragment
                .clone()
                .or(inherited_fragment)
                .or_else(|| {
                    self.units
                        .get(&caster_instance_id)
                        .and_then(|unit| unit.fragment_source.as_ref())
                        .filter(|source| {
                            self.fragment_owns_skill(&source.skill_id, &skill.id)
                                && self.runtime_fact_source.equipment.is_none()
                        })
                        .cloned()
                }),
            action: Some(action.clone()),
            cast: Some(execution_source.clone()),
            allows_finite_children: true,
            resource_capabilities:
                match crate::game::battle::runtime_facts::RuntimeFactSource::resource_capabilities(
                    self.runtime_facts
                        .resources
                        .as_ref()
                        .and_then(|resources| resources.context(&action).cloned()),
                    None,
                ) {
                    Ok(value) => value,
                    Err(_) => return,
                },
            ..Default::default()
        };
        let fact_source = match fact_source.with_cast_damage(accepted_cast_damage) { Ok(source) => source, Err(_) => return };
        let fact_source = if let Some(binding) = effective_source.relic_effective() {
            match fact_source.with_relic_effective(binding.clone()) {
                Ok(value) => value,
                Err(_) => return,
            }
        } else {
            fact_source
        };
        let accepted_causal = self
            .runtime_facts
            .accepted_source(&action)
            .ok()
            .and_then(|s| s.causal().cloned());
        let fact_source = if accepted_causal.is_some() {
            match fact_source.with_causal(accepted_causal) {
                Ok(source) => source,
                Err(_) => return,
            }
        } else {
            fact_source
        };
        if fact_source.admit_owner("active_cast_fact_source").is_err() {
            return;
        }
        self.active_skill_casts.insert(
            cast_seq,
            super::types::ActiveSkillCast {
                control_cancelled: false,
                flow: Some(flow),
                fact_source,
                execution_source: Some(execution_source),
                caster_instance_id,
                ability_event_seq,
                skill_id: skill.id.clone(),
                caster_owner,
                anchor_position: caster_pos,
                accepted_facing: accepted_facing.or_else(|| self.units.get(&caster_instance_id).and_then(|u| u.facing_direction)),
                cast_target_anchor_position,
                cast_target_anchor_world_position,
                allow_dead_caster,
                total_steps: 0,
                step_progress: Default::default(),
                deferred_steps: Default::default(),
                impact_contexts_by_step: Default::default(),
                last_impact_context: None,
                active_area_ids: Vec::new(),
            },
        );

        let _ = self.drive_skill_flow(cast_seq, time_ms, None);
    }

    pub(super) fn reinput_cast_target(
        &self,
        now: u64,
        caster: UnitInstanceId,
        skill: &SkillDef,
        target: Option<SkillCastTarget>,
    ) -> Result<Option<SkillCastTarget>, GameError> {
        let unit = self.units.get(&caster).ok_or(GameError::UnitNotFound)?;
        let position = self
            .live_unit_projected_tile(caster)
            .ok_or(GameError::InvalidAction)?;
        let resolved = target.or_else(|| {
            self.resolve_skill_cast_target_at(now, skill, caster, unit.owner, position)
        });
        let resolved = resolved.ok_or(GameError::InvalidAction)?;
        if !self
            .explicit_skill_cast_target_is_valid(now, skill, caster, unit.owner, position, resolved)
        {
            return Err(GameError::InvalidAction);
        }
        Ok(Some(resolved))
    }
    fn explicit_skill_cast_target_is_valid(
        &self,
        time_ms: u64,
        skill: &SkillDef,
        caster_instance_id: UnitInstanceId,
        caster_owner: Side,
        _caster_pos: Position,
        target: SkillCastTarget,
    ) -> bool {
        if !self.approach_taunt_skill_allowed(caster_instance_id, skill, Some(target)) { return false; }
        if self.validate_fusion_start(caster_instance_id, skill, Some(target), time_ms).is_err() { return false; }
        let Some((target_def, reach, target_range, air_capable)) = skill.cast_target_definition()
        else {
            return false;
        };

        if let Some(plan) =
            self.current_effective_skill(&skill.id, caster_instance_id, &self.runtime_fact_source)
        {
            if let Some(reach) = &plan.selected_recipient_reach {
                let SkillCastTarget::Unit { unit_instance_id } = target else {
                    return false;
                };
                if (plan.selected_recipient_excludes_caster
                    && unit_instance_id == caster_instance_id)
                    || !self.selected_support_in_reach(
                        caster_instance_id,
                        unit_instance_id,
                        reach,
                    )
                {
                    return false;
                }
            }
        }
        match (target_def, target) {
            (SkillTarget::SelfUnit, SkillCastTarget::Unit { unit_instance_id }) => {
                unit_instance_id == caster_instance_id
                    && self.skill_has_useful_automatic_cast_target(
                        time_ms,
                        skill,
                        caster_instance_id,
                        Some(target),
                    )
            }
            (SkillTarget::AllySingle, SkillCastTarget::Unit { unit_instance_id }) => {
                self.is_alive_ally(unit_instance_id, caster_owner)
                    && self.single_target_can_target_unit(unit_instance_id, air_capable)
                    && self.is_target_in_skill_reach(
                        caster_instance_id,
                        unit_instance_id,
                        reach,
                        target_range,
                    )
                    && self.skill_has_useful_automatic_cast_target(
                        time_ms,
                        skill,
                        caster_instance_id,
                        Some(target),
                    )
            }
            (SkillTarget::EnemySingle { .. }, SkillCastTarget::Unit { unit_instance_id }) => {
                if !self.is_alive_enemy(unit_instance_id, caster_owner) {
                    return false;
                }
                if !self.single_target_can_target_unit(unit_instance_id, air_capable) {
                    return false;
                }
                self.is_target_in_skill_reach(
                    caster_instance_id,
                    unit_instance_id,
                    reach,
                    target_range,
                ) && self.skill_has_useful_automatic_cast_target(
                    time_ms,
                    skill,
                    caster_instance_id,
                    Some(target),
                )
            }
            (SkillTarget::CastTarget, SkillCastTarget::Direction { .. }) => skill.cast_selection() == SkillCastSelection::PlayerDirection,
            (SkillTarget::CastTarget, SkillCastTarget::TilePair { first, second }) => {
                skill.cast_selection() == SkillCastSelection::PlayerTilePair
                    && [first, second].into_iter().all(|position| self.is_tile_in_skill_reach(caster_instance_id, position, reach, target_range))
            }
            (SkillTarget::CastTarget, SkillCastTarget::Tile { position }) => {
                self.is_tile_in_skill_reach(caster_instance_id, position, reach, target_range)
                    && self
                        .validate_skill_object_placements(
                            time_ms,
                            caster_instance_id,
                            skill,
                            Some(target),
                        )
                        .is_ok()
            }
            _ => false,
        }
    }

    pub(super) fn admitted_skill_target_is_valid(
        &self,
        caster: UnitInstanceId,
        skill: &SkillDef,
        target: Option<SkillCastTarget>,
    ) -> bool {
        if !self.approach_taunt_skill_allowed(caster, skill, target) { return false; }
        if skill.cast_selection() == SkillCastSelection::SelfUnit {
            return target
                == Some(SkillCastTarget::Unit {
                    unit_instance_id: caster,
                })
                && self.units.get(&caster).is_some_and(|unit| unit.is_active());
        }
        let required_unit = skill.steps.iter().any(|step| {
            matches!(
                step.target,
                SkillTarget::EnemySingle { .. } | SkillTarget::AllySingle | SkillTarget::CastTarget
            ) && matches!(
                step.delivery,
                DeliveryDef::Instant
                    | DeliveryDef::Projectile {
                        hit_policy: crate::game::ability::ProjectileHitPolicy::TargetLocked,
                        ..
                    }
            )
        });
        if !required_unit {
            return true;
        }
        let Some(SkillCastTarget::Unit { unit_instance_id }) = target else {
            return matches!(skill.cast_selection(), SkillCastSelection::PlayerTile | SkillCastSelection::PlayerTilePair | SkillCastSelection::PlayerDirection);
        };
        let Some(unit) = self.units.get(&caster) else {
            return false;
        };
        let Some((target_def, reach, range, air_capable)) = skill.cast_target_definition() else {
            return false;
        };
        (if matches!(target_def, SkillTarget::AllySingle) {
            self.is_alive_ally(unit_instance_id, unit.owner)
        } else {
            self.is_alive_enemy(unit_instance_id, unit.owner)
        }) && self.single_target_can_target_unit(unit_instance_id, air_capable)
            && self.is_target_in_skill_reach(caster, unit_instance_id, reach, range)
    }

    pub(super) fn start_manual_skill_cast(
        &mut self,
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        requested_skill_id: SkillId,
        explicit_target: Option<SkillCastTarget>,
    ) -> Result<(), GameError> {
        if self
            .channel_reinput(caster_instance_id, &requested_skill_id)
            .is_some()
        {
            if explicit_target.is_some() {
                return Err(GameError::InvalidAction);
            }
            return self.stop_channel(caster_instance_id, time_ms);
        }
        if let Some(activation) = self.sustained_followup_on_input(caster_instance_id,&requested_skill_id)? {
            return self.reinput_sustained_skill(caster_instance_id,&requested_skill_id,activation,explicit_target,time_ms);
        }
        self.start_manual_skill_cast_with_retention(
            time_ms,
            caster_instance_id,
            requested_skill_id,
            explicit_target,
            false,
        )
        .map(|_| ())
    }

    pub fn request_skill_action(
        &mut self,
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        requested_skill_id: SkillId,
        explicit_target: Option<SkillCastTarget>,
    ) -> Result<crate::game::battle::runtime_facts::ActionHandle, GameError> {
        self.ensure_not_forced_paused()?;
        self.runtime_transaction(|core| {
            core.start_manual_skill_cast_with_retention(
                time_ms,
                caster_instance_id,
                requested_skill_id,
                explicit_target,
                true,
            )
        })
    }

    fn start_manual_skill_cast_with_retention(
        &mut self,
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        requested_skill_id: SkillId,
        explicit_target: Option<SkillCastTarget>,
        public_reserved: bool,
    ) -> Result<crate::game::battle::runtime_facts::ActionHandle, GameError> {
        let (skill, cast_target) = self.resolve_manual_skill_cast_request(
            time_ms,
            caster_instance_id,
            &requested_skill_id,
            explicit_target,
        )?;

        self.start_admitted_skill_cast(
            time_ms,
            caster_instance_id,
            skill,
            cast_target,
            public_reserved,
            true,
        )
    }

    pub(super) fn start_boss_skill_action(
        &mut self,
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        requested_skill_id: &SkillId,
        target: SkillCastTarget,
        facing: crate::game::battle::tile_range::FacingDirection,
    ) -> Result<crate::game::battle::runtime_facts::ActionHandle, GameError> {
        let unit = self
            .units
            .get(&caster_instance_id)
            .ok_or(GameError::UnitNotFound)?;
        if unit.owner != Side::Opponent
            || !unit.is_combatant()
            || !unit.is_active()
            || unit.pending_skill_cast.is_some()
            || time_ms < unit.next_action_time
            || self
                .skill_start_disabled_reason(caster_instance_id, time_ms)
                .is_some()
        {
            return Err(GameError::InvalidAction);
        }
        let pos = self
            .live_unit_projected_tile(caster_instance_id)
            .ok_or(GameError::InvalidAction)?;
        let owner = unit.owner;
        let skill = self
            .cloned_current_skill(
                requested_skill_id,
                caster_instance_id,
                &self.runtime_fact_source,
            )?
            .ok_or(GameError::InvalidAction)?;
        // Direction is part of the candidate; a rejection never publishes it.
        let previous_facing = self.units[&caster_instance_id].facing_direction;
        self.units
            .get_mut(&caster_instance_id)
            .unwrap()
            .facing_direction = Some(facing);
        let valid = self.explicit_skill_cast_target_is_valid(
            time_ms,
            &skill,
            caster_instance_id,
            owner,
            pos,
            target,
        );
        self.units
            .get_mut(&caster_instance_id)
            .unwrap()
            .facing_direction = previous_facing;
        if !valid {
            return Err(GameError::InvalidAction);
        }
        let action = self.start_admitted_skill_cast(
            time_ms,
            caster_instance_id,
            skill,
            Some(target),
            true,
            false,
        )?;
        self.units
            .get_mut(&caster_instance_id)
            .unwrap()
            .facing_direction = Some(facing);
        if let Some(pending) = self.units.get_mut(&caster_instance_id).and_then(|u| u.pending_skill_cast.as_mut()) { pending.accepted_facing = Some(facing); }
        Ok(action)
    }

    pub(super) fn start_admitted_skill_cast(
        &mut self,
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        skill: SkillDef,
        cast_target: Option<SkillCastTarget>,
        public_reserved: bool,
        reset_resonance: bool,
    ) -> Result<crate::game::battle::runtime_facts::ActionHandle, GameError> {
        if Self::input_plan_skill(&skill) && !self.input_plan_commit_pending(caster_instance_id) {return Err(GameError::InvalidAction);}
        self.validate_sustained_start(caster_instance_id, &skill)?;
        self.validate_prepared_volley_start(caster_instance_id,&skill)?;
        self.validate_paired_area_start(caster_instance_id,&skill,time_ms)?;
        self.validate_fusion_start(caster_instance_id, &skill, cast_target, time_ms)?;
        self.reconcile_ready_wait(time_ms)?;
        let accepted_source = crate::game::battle::runtime_facts::RuntimeFactSource {
            extended_observation:
                crate::game::battle::runtime_facts::RuntimeFactSource::extended_observation(
                    self.independent_action_origin(caster_instance_id)?,
                    None,
                )?,
            actor: Some(caster_instance_id),
            fragment: self
                .units
                .get(&caster_instance_id)
                .and_then(|unit| unit.fragment_source.as_ref())
                .filter(|source| self.fragment_owns_skill(&source.skill_id, &skill.id))
                .cloned(),
            ..Default::default()
        };
        let accepted_source =
            self.bind_effective_skill_source(&skill.id, caster_instance_id, &accepted_source)?;
        let cast_damage = self.consume_equipment_cast_damage(caster_instance_id, time_ms)?;
        let accepted_source = accepted_source.with_cast_damage(cast_damage)?;
        let action = self.runtime_facts.accept(
            caster_instance_id,
            Some(&skill.id),
            accepted_source,
            time_ms,
            public_reserved,
        )?;

        let cast_end_ms = time_ms
            .checked_add(if (Self::fusion_placement_skill(&skill) || Self::status_strike_declaration(&skill).is_some()) || Self::paired_area_skill(&skill) || Self::presence_burst_skill(&skill) || Self::tile_burst_skill(&skill) || Self::counter_channel_skill(&skill) || Self::relation_action_skill(&skill) || Self::input_plan_skill(&skill) || Self::prepared_volley_skill(&skill) { 0 } else { u64::from(skill.focus_time_ms).max(1) })
            .ok_or_else(|| {
                crate::game::battle::runtime_control::reject(
                    crate::game::battle::runtime_control::BattleRuntimeFault {
                        code:
                            crate::game::battle::runtime_control::RuntimeFaultCode::CounterOverflow,
                        site: "admitted_cast_deadline",
                        limit: None,
                        requested: None,
                    },
                )
            })?;
        let employee_use = reset_resonance
            && self.admit_employee_resonance_use(
                caster_instance_id,
                &skill,
                &action,
                time_ms,
            )?;
        self.prepare_counter_channel(caster_instance_id, &skill, &action, cast_target, time_ms)?;
        self.prepare_relation_action(caster_instance_id, &skill, &action, cast_target, time_ms)?;
        self.prepare_presence_burst(caster_instance_id, &skill, &action, time_ms)?;
                self.prepare_tile_burst(caster_instance_id, &skill, &action, time_ms)?;
        self.activate_ready_wait(caster_instance_id, &skill.id, time_ms)?;
        if let Some(caster) = self.units.get_mut(&caster_instance_id) {
            caster.next_action_time = cast_end_ms;
            if !employee_use {
                caster.action_locks.lock_resonance_gain_until(cast_end_ms);
            }
            if !skill.focus_permissions.allows_basic_attack {
                caster.action_locks.lock_basic_attack_until(cast_end_ms);
            }
            if !skill.focus_permissions.allows_move {
                caster.action_locks.lock_movement_until(cast_end_ms);
            }
        }

        let start_seq = self.record_event_log(
            time_ms,
            BattleLogEvent::ManualCastStart {
                skill_id: skill.id.clone(),
                caster_instance_id,
                target: cast_target,
            },
        );
        let accepted_target_tile = match cast_target {
            Some(SkillCastTarget::Unit { unit_instance_id }) => {
                self.live_unit_projected_tile(unit_instance_id)
            }
            Some(SkillCastTarget::Tile { position }) => Some(position),
            _ => None,
        };
        if let Some(caster) = self.units.get_mut(&caster_instance_id) {
            caster.pending_skill_cast = Some(PendingSkillCast {
                start_gate_pending: reset_resonance,
                reset_resonance: reset_resonance && !employee_use,
                action: Some(action.clone()),
                skill_id: skill.id,
                cast_target,
                accepted_target_tile,
                accepted_facing: caster.facing_direction,
                start_seq,
            });
        }
        if reset_resonance {
            self.event_queue.push(BattleEvent::SkillStartGate {
                time_ms,
                caster_instance_id,
                cause: BattleEventCause::Parent { seq: start_seq },
            });
        }
        self.event_queue.push(BattleEvent::ManualCastEnd {
            time_ms: cast_end_ms,
            caster_instance_id,
            cause: BattleEventCause::Parent { seq: start_seq },
        });

        Ok(action)
    }

    pub fn validate_manual_skill_activation(
        &self,
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        requested_skill_id: &SkillId,
        explicit_target: Option<SkillCastTarget>,
    ) -> Result<(), GameError> {
        if self
            .channel_reinput(caster_instance_id, requested_skill_id)
            .is_some()
        {
            return if explicit_target.is_none() {
                Ok(())
            } else {
                Err(GameError::InvalidAction)
            };
        }
        if let Some(activation) = self.sustained_followup_on_input(caster_instance_id, requested_skill_id)? {
            self.validate_action_reinput(caster_instance_id, requested_skill_id, activation, time_ms)?;
            return self.validate_sustained_input_target(activation, explicit_target, time_ms);
        }
        self.resolve_manual_skill_cast_request(
            time_ms,
            caster_instance_id,
            requested_skill_id,
            explicit_target,
        )
        .map(|_| ())
    }

    pub fn validate_manual_skill_targeting_focus(
        &self,
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        requested_skill_id: &SkillId,
    ) -> Result<SkillCastSelection, GameError> {
        let (skill, _, _) = self.resolve_manual_skill_base_request(
            time_ms,
            caster_instance_id,
            requested_skill_id,
        )?;
        let selection = skill.cast_selection();
        if !matches!(
            selection,
            SkillCastSelection::PlayerUnit | SkillCastSelection::PlayerTile | SkillCastSelection::PlayerTilePair | SkillCastSelection::PlayerDirection
        ) {
            return Err(GameError::ActionUnavailable {
                action: crate::game::behavior::ActionKind::BeginSkillTargeting,
                reason: ActionDisabledReasonCode::NoEligiblePayload,
            });
        }
        Ok(selection)
    }

    /// Fresh admission eligibility, without accepting an action or treating an
    /// active skill's control/reinput button as a new ready skill.
    pub(super) fn ready_wait_eligible_skill(
        &self, now: u64, actor: UnitInstanceId,
    ) -> Result<Option<SkillId>, GameError> {
        let Some(unit) = self.units.get(&actor) else { return Ok(None); };
        let Some(id) = unit.skill_id.as_ref() else { return Ok(None); };
        let (skill, side, position) = match self.resolve_employee_skill_base_request(now, actor, id, false) {
            Ok(value) => value,
            Err(GameError::ActionUnavailable { .. } | GameError::InvalidAction | GameError::UnitNotFound) => {
                control::poll("ready_wait_admission")?;
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        if unit.skill_activation_mode == SkillActivationMode::Auto
            && (skill.resonance_admission.is_some()
                || self.requires_explicit_structural_selection(&skill, actor, &self.runtime_fact_source)?) {
            return Ok(None);
        }
        let valid = |target| self.explicit_skill_cast_target_is_valid(now, &skill, actor, side, position, target);
        let available = match skill.cast_selection() {
            SkillCastSelection::SelfUnit => valid(SkillCastTarget::Unit { unit_instance_id: actor }),
            SkillCastSelection::AutomaticUnit => self.resolve_skill_cast_target_at(now, &skill, actor, side, position).is_some_and(valid),
            SkillCastSelection::PlayerUnit => {
                control::charge(self.units.len() as u64, "ready_wait_unit_candidates")?;
                self.units.keys().any(|id| valid(SkillCastTarget::Unit { unit_instance_id: *id }))
            }
            SkillCastSelection::PlayerDirection => {
                use crate::game::battle::tile_range::FacingDirection::*;
                [Up, Right, Down, Left].into_iter().any(|facing| valid(SkillCastTarget::Direction { facing }))
            }
            SkillCastSelection::PlayerTile | SkillCastSelection::PlayerTilePair => {
                if let Some((_, reach, range, _)) = skill.cast_target_definition() {
                    let policy = match reach {
                        SkillTargetReach::Pattern => crate::game::battle::tile_range::TileRangePolicy::Pattern,
                        SkillTargetReach::Global => crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles,
                    };
                    let cells = if reach == SkillTargetReach::Global {
                        (self.battlefield.width() as usize).checked_mul(self.battlefield.height() as usize)
                    } else { range.map(|r| r.rows.iter().map(|r| r.len()).sum()) };
                    crate::game::battle::runtime_control::admit_retained_growth(0,
                        cells.and_then(|n| n.checked_mul(std::mem::size_of::<Position>() * 2)), "ready_wait_tile_candidates")?;
                    control::charge(cells.unwrap_or(usize::MAX) as u64, "ready_wait_tile_candidates")?;
                    self.tile_reach_query_for_unit(actor, policy, range).is_some_and(|query| {
                        query.materialize_valid_cells().into_iter().any(|position| valid(
                            if skill.cast_selection() == SkillCastSelection::PlayerTilePair {
                                SkillCastTarget::TilePair { first: position, second: position }
                            } else { SkillCastTarget::Tile { position } }
                        ))
                    })
                } else { false }
            }
        };
        control::poll("ready_wait_admission")?;
        Ok(available.then_some(skill.id))
    }

    pub fn live_skill_readiness(
        &self,
        time_ms: u64,
        unit_id: UnitInstanceId,
    ) -> Option<LiveBattleSkillReadinessDto> {
        let unit = self.units.get(&unit_id)?;
        let skill_id = unit.skill_id.clone();
        let activation_mode = unit.skill_activation_mode;
        let resonance_max = unit.resonance_max.max(1);
        let mut selection = None;
        let mut target_required = false;
        let mut target_available = false;
        if let Some(skill) = skill_id
            .as_ref()
            .and_then(|id| self.current_skill_view(id, unit_id, &Default::default()))
        {
            selection = Some(skill.cast_selection());
            match skill.cast_selection() {
                SkillCastSelection::SelfUnit => {
                    target_available = true;
                }
                SkillCastSelection::AutomaticUnit | SkillCastSelection::PlayerUnit => {
                    target_required =
                        matches!(skill.cast_selection(), SkillCastSelection::PlayerUnit);
                    target_available = self
                        .live_unit_projected_tile(unit_id)
                        .and_then(|caster_pos| {
                            self.resolve_skill_cast_target_at(
                                time_ms, skill, unit_id, unit.owner, caster_pos,
                            )
                        })
                        .is_some();
                }
                SkillCastSelection::PlayerDirection => { target_required = true; target_available = true; },
                SkillCastSelection::PlayerTile | SkillCastSelection::PlayerTilePair => {
                    target_required = true;
                    target_available = skill
                        .cast_target_definition()
                        .and_then(|(_, reach, target_range, _)| {
                            let policy = match reach {
                                SkillTargetReach::Pattern => {
                                    crate::game::battle::tile_range::TileRangePolicy::Pattern
                                }
                                SkillTargetReach::Global => {
                                    crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles
                                }
                            };
                            self.tile_reach_query_for_unit(unit_id, policy, target_range)
                        })
                        .is_some_and(|query| {
                            query
                            .materialize_valid_cells()
                            .into_iter()
                            .any(|position| {
                                self.explicit_skill_cast_target_is_valid(
                                    time_ms,
                                    skill,
                                    unit_id,
                                    unit.owner,
                                    unit.body.projected_tile(),
                                    if skill.cast_selection() == SkillCastSelection::PlayerTilePair { SkillCastTarget::TilePair { first: position, second: position } } else { SkillCastTarget::Tile { position } },
                                )
                            })
                        });
                }
            }
        }

        let eligibility = match skill_id.as_ref() {
            Some(skill_id) => {
                let validation = match selection {
                    Some(SkillCastSelection::PlayerUnit | SkillCastSelection::PlayerTile | SkillCastSelection::PlayerTilePair | SkillCastSelection::PlayerDirection) => self
                        .validate_manual_skill_targeting_focus(time_ms, unit_id, skill_id)
                        .map(|_| ()),
                    _ => self.validate_manual_skill_activation(time_ms, unit_id, skill_id, None),
                };
                match validation {
                    Ok(()) => BattlePayloadEligibilityDto::enabled(),
                    Err(GameError::ActionUnavailable { reason, .. }) => {
                        BattlePayloadEligibilityDto::disabled(reason)
                    }
                    Err(GameError::InvalidStaticData(_)) => BattlePayloadEligibilityDto::disabled(
                        ActionDisabledReasonCode::SkillUnavailable,
                    ),
                    Err(_) => BattlePayloadEligibilityDto::disabled(
                        ActionDisabledReasonCode::UnitUnavailable,
                    ),
                }
            }
            None => {
                BattlePayloadEligibilityDto::disabled(ActionDisabledReasonCode::SkillUnavailable)
            }
        };

        Some(LiveBattleSkillReadinessDto {
            resonance_use: self.resonance_use_projection(unit_id),
            resonance_gain_block_reasons: self.resonance_gain_block_reasons(unit_id, time_ms),
            corrosion_ready: target_available
                && skill_id
                    .as_ref()
                    .is_some_and(|skill| self.corrosion_ready(unit_id, skill, time_ms)),
            skill_id,
            activation_mode,
            selection,
            resonance_current: unit.resonance_current.min(resonance_max),
            resonance_max,
            manual_activation_allowed: eligibility.enabled,
            target_required,
            target_available,
            eligibility,
        })
    }

    fn resolve_manual_skill_cast_request(
        &self,
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        requested_skill_id: &SkillId,
        explicit_target: Option<SkillCastTarget>,
    ) -> Result<(SkillDef, Option<SkillCastTarget>), GameError> {
        let unavailable = |reason| GameError::ActionUnavailable {
            action: crate::game::behavior::ActionKind::ActivateSkill,
            reason,
        };
        let (skill, caster_owner, caster_pos) = self.resolve_manual_skill_base_request(
            time_ms,
            caster_instance_id,
            requested_skill_id,
        )?;

        let cast_target = match (skill.cast_selection(), explicit_target) {
            (SkillCastSelection::SelfUnit, None) => Some(SkillCastTarget::Unit {
                unit_instance_id: caster_instance_id,
            }),
            (SkillCastSelection::AutomaticUnit, None) => self.resolve_skill_cast_target_at(
                time_ms,
                &skill,
                caster_instance_id,
                caster_owner,
                caster_pos,
            ),
            (SkillCastSelection::PlayerUnit, Some(target @ SkillCastTarget::Unit { .. }))
            | (SkillCastSelection::PlayerTile, Some(target @ SkillCastTarget::Tile { .. }))
            | (SkillCastSelection::PlayerTilePair, Some(target @ SkillCastTarget::TilePair { .. }))
            | (SkillCastSelection::PlayerDirection, Some(target @ SkillCastTarget::Direction { .. })) => {
                if self.explicit_skill_cast_target_is_valid(
                    time_ms,
                    &skill,
                    caster_instance_id,
                    caster_owner,
                    caster_pos,
                    target,
                ) {
                    Some(target)
                } else {
                    return Err(unavailable(ActionDisabledReasonCode::NoValidTarget));
                }
            }
            (SkillCastSelection::PlayerUnit | SkillCastSelection::PlayerTile | SkillCastSelection::PlayerTilePair | SkillCastSelection::PlayerDirection, None) => {
                return Err(unavailable(ActionDisabledReasonCode::SkillTargetRequired))
            }
            _ => {
                return Err(unavailable(
                    ActionDisabledReasonCode::SkillTargetTypeMismatch,
                ))
            }
        };
        if cast_target.is_none() {
            return Err(unavailable(ActionDisabledReasonCode::NoValidTarget));
        }
        if !matches!(skill.cast_selection(), SkillCastSelection::PlayerTile | SkillCastSelection::PlayerTilePair | SkillCastSelection::PlayerDirection)
            && !self.skill_has_useful_automatic_cast_target(
                time_ms,
                &skill,
                caster_instance_id,
                cast_target,
            )
        {
            return Err(unavailable(ActionDisabledReasonCode::NoValidTarget));
        }

        self.validate_fusion_start(caster_instance_id, &skill, cast_target, time_ms)?;
        Ok((skill, cast_target))
    }

    fn resolve_manual_skill_base_request(
        &self,
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        requested_skill_id: &SkillId,
    ) -> Result<(SkillDef, Side, Position), GameError> {
        self.resolve_employee_skill_base_request(time_ms, caster_instance_id, requested_skill_id, true)
    }

    fn resolve_employee_skill_base_request(
        &self, time_ms: u64, caster_instance_id: UnitInstanceId,
        requested_skill_id: &SkillId, manual_only: bool,
    ) -> Result<(SkillDef, Side, Position), GameError> {
        let unavailable = |reason| GameError::ActionUnavailable {
            action: crate::game::behavior::ActionKind::ActivateSkill,
            reason,
        };
        let Some(caster) = self.units.get(&caster_instance_id) else {
            return Err(GameError::UnitNotFound);
        };
        if self
            .independent_objects
            .objects
            .contains_key(&caster_instance_id)
            || caster.owner != Side::Player
            || !caster.is_combatant()
            || !caster.is_active()
        {
            return Err(unavailable(ActionDisabledReasonCode::UnitUnavailable));
        }
        if (manual_only && caster.skill_activation_mode != SkillActivationMode::Manual)
            || caster.skill_activation_mode == SkillActivationMode::Triggered {
            return Err(unavailable(
                ActionDisabledReasonCode::ActivationModeNotManual,
            ));
        }
        if let Some(reason) = self.skill_start_disabled_reason(caster_instance_id, time_ms) {
            return Err(unavailable(reason));
        }
        if time_ms < caster.next_action_time {
            return Err(unavailable(ActionDisabledReasonCode::ActionLocked));
        }
        if caster.pending_skill_cast.is_some()
            || self.resonance_use_has_obligations(caster_instance_id)
        {
            return Err(unavailable(ActionDisabledReasonCode::ActionLocked));
        }

        let Some(unit_skill_id) = caster.skill_id.clone() else {
            return Err(unavailable(ActionDisabledReasonCode::SkillUnavailable));
        };
        if &unit_skill_id != requested_skill_id && !self.counter_manual_entry(&unit_skill_id, requested_skill_id) {
            return Err(unavailable(ActionDisabledReasonCode::SkillUnavailable));
        }
        let caster_owner = caster.owner;
        let caster_pos = self
            .live_unit_projected_tile(caster_instance_id)
            .ok_or_else(|| unavailable(ActionDisabledReasonCode::UnitUnavailable))?;

        let skill = self
            .cloned_current_skill(
                requested_skill_id,
                caster_instance_id,
                &self.runtime_fact_source,
            )?
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "manual skill '{}' is missing from skill database",
                    requested_skill_id
                ))
            })?;

        if self.resonance_admission_payment(caster_instance_id, &skill).is_none() {
            return Err(unavailable(if skill.resonance_admission.is_some() {
                ActionDisabledReasonCode::InsufficientResources
            } else {
                ActionDisabledReasonCode::ResonanceNotFull
            }));
        }
        self.validate_sustained_start(caster_instance_id, &skill)?;
        self.validate_prepared_volley_start(caster_instance_id,&skill)?;
        self.validate_paired_area_start(caster_instance_id,&skill,time_ms)?;
        Ok((skill, caster_owner, caster_pos))
    }

    #[cfg(test)]
    pub(super) fn resolve_skill_cast_target(
        &self,
        skill: &SkillDef,
        caster_instance_id: UnitInstanceId,
        caster_owner: Side,
        caster_pos: Position,
    ) -> Option<SkillCastTarget> {
        self.resolve_skill_cast_target_at(0, skill, caster_instance_id, caster_owner, caster_pos)
    }

    fn resolve_skill_cast_target_at(
        &self,
        time_ms: u64,
        skill: &SkillDef,
        caster_instance_id: UnitInstanceId,
        caster_owner: Side,
        caster_pos: Position,
    ) -> Option<SkillCastTarget> {
        self.resolve_skill_cast_target_for_origin(
            time_ms,
            skill,
            caster_instance_id,
            caster_owner,
            caster_pos,
            false,
        )
    }

    fn resolve_skill_cast_target_for_origin(
        &self,
        time_ms: u64,
        skill: &SkillDef,
        caster_instance_id: UnitInstanceId,
        caster_owner: Side,
        caster_pos: Position,
        allow_dead_caster: bool,
    ) -> Option<SkillCastTarget> {
        let (target, reach, target_range, air_capable) = skill.cast_target_definition()?;
        if !allow_dead_caster && matches!(target, SkillTarget::EnemySingle { .. }) {
            if let Some((source, _)) = self.approach_taunt(caster_instance_id).filter(|_| Self::hostile_attack_skill(skill)) {
                return (self.approach_taunt_attack_allowed(caster_instance_id, source)
                    && self.single_target_can_target_unit(source, air_capable)
                    && self.is_target_in_skill_reach(caster_instance_id, source, reach, target_range))
                    .then_some(SkillCastTarget::Unit { unit_instance_id: source });
            }
        }
        self.resolve_skill_target_definition(
            caster_instance_id,
            caster_owner,
            caster_pos,
            reach,
            target_range,
            target,
            air_capable,
            if allow_dead_caster {
                SkillTargetUsefulness::AllowedDeadCasterCast { time_ms, skill }
            } else {
                SkillTargetUsefulness::Cast { time_ms, skill }
            },
        )
    }

    fn resolve_skill_target_definition(
        &self,
        caster_instance_id: UnitInstanceId,
        caster_owner: Side,
        caster_pos: Position,
        reach: SkillTargetReach,
        target_range: Option<&crate::game::battle::tile_range::TileRangePattern>,
        target: &SkillTarget,
        air_capable: bool,
        usefulness: SkillTargetUsefulness<'_>,
    ) -> Option<SkillCastTarget> {
        match target {
            SkillTarget::SelfUnit => Some(SkillCastTarget::Unit {
                unit_instance_id: caster_instance_id,
            }),
            SkillTarget::EnemySingle { rule } => self
                .choose_skill_target_by_rule(
                    caster_instance_id,
                    caster_owner,
                    caster_pos,
                    match reach {
                        SkillTargetReach::Pattern => {
                            crate::game::battle::tile_range::TileRangePolicy::Pattern
                        }
                        SkillTargetReach::Global => {
                            crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles
                        }
                    },
                    target_range,
                    *rule,
                    air_capable,
                    usefulness,
                )
                .map(|id| SkillCastTarget::Unit {
                    unit_instance_id: id,
                }),
            SkillTarget::AllySingle => self
                .choose_ally_skill_target(
                    caster_instance_id,
                    reach,
                    target_range,
                    air_capable,
                    usefulness,
                )
                .map(|unit_instance_id| SkillCastTarget::Unit { unit_instance_id }),
            SkillTarget::CastTarget => None,
        }
    }

    pub(in crate::game::battle::core) fn resolve_skill_step_context(
        &self,
        time_ms: u64,
        cast_seq: u64,
        step_index: usize,
        caster_instance_id: UnitInstanceId,
        skill: &SkillDef,
        step: &SkillStepDef,
        cast_target: Option<SkillCastTarget>,
    ) -> Option<SkillCastTarget> {
        if let Some(binding) = self
            .active_skill_casts
            .get(&cast_seq)
            .and_then(|c| c.fact_source.causal())
            .and_then(|c| c.program.as_ref())
        {
            let target = binding.role(step_index)?;
            return self.units.get(&target).filter(|u| u.is_active()).map(|_| {
                SkillCastTarget::Unit {
                    unit_instance_id: target,
                }
            });
        }
        let (caster_owner, caster_pos) =
            self.resolve_cast_origin_context(Some(cast_seq), caster_instance_id, false)?;
        let cast_target_anchor_position = self
            .active_skill_casts
            .get(&cast_seq)
            .and_then(|cast_state| cast_state.cast_target_anchor_position);
        let (_, cast_reach, cast_target_range, cast_air_capable) =
            skill.cast_target_definition()?;

        match step.targeting {
            StepTargetingMode::ReuseCastTarget => match &step.target {
                SkillTarget::SelfUnit => Some(SkillCastTarget::Unit {
                    unit_instance_id: caster_instance_id,
                }),
                SkillTarget::AllySingle => match cast_target {
                    Some(SkillCastTarget::Unit { unit_instance_id })
                        if self.is_alive_ally(unit_instance_id, caster_owner)
                            && self.single_target_can_target_unit(
                                unit_instance_id,
                                cast_air_capable,
                            )
                            && self.is_target_in_skill_reach(
                                caster_instance_id,
                                unit_instance_id,
                                cast_reach,
                                cast_target_range,
                            ) =>
                    {
                        Some(SkillCastTarget::Unit { unit_instance_id })
                    }
                    _ => None,
                },
                SkillTarget::EnemySingle { .. } => match cast_target {
                    Some(SkillCastTarget::Unit { .. })
                        if matches!(step.delivery, DeliveryDef::TileArea { .. }) =>
                    {
                        cast_target_anchor_position
                            .map(|position| SkillCastTarget::Tile { position })
                    }
                    Some(SkillCastTarget::Unit { unit_instance_id })
                        if self.is_alive_enemy(unit_instance_id, caster_owner)
                            && self.single_target_can_target_unit(
                                unit_instance_id,
                                cast_air_capable,
                            ) =>
                    {
                        Some(SkillCastTarget::Unit { unit_instance_id })
                    }
                    _ => None,
                },
                SkillTarget::CastTarget => match cast_target {
                    pair @ Some(SkillCastTarget::TilePair { .. } | SkillCastTarget::Direction { .. }) => pair,
                    Some(SkillCastTarget::Unit { unit_instance_id }) => {
                        Some(SkillCastTarget::Unit { unit_instance_id })
                    }
                    Some(SkillCastTarget::Tile { position }) => {
                        Some(SkillCastTarget::Tile { position })
                    }
                    None => cast_target_anchor_position
                        .map(|position| SkillCastTarget::Tile { position }),
                },
            },
            StepTargetingMode::RetargetOnStep => self.resolve_skill_target_definition(
                caster_instance_id,
                caster_owner,
                caster_pos,
                cast_reach,
                cast_target_range,
                &step.target,
                cast_air_capable,
                SkillTargetUsefulness::DeliveryStep {
                    time_ms,
                    step_index,
                    step,
                },
            ),
        }
    }

    pub(in crate::game::battle::core) fn is_target_in_skill_reach(
        &self,
        caster_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        reach: SkillTargetReach,
        target_range: Option<&crate::game::battle::tile_range::TileRangePattern>,
    ) -> bool {
        let policy = match reach {
            SkillTargetReach::Pattern => crate::game::battle::tile_range::TileRangePolicy::Pattern,
            SkillTargetReach::Global => {
                crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles
            }
        };
        self.is_target_in_tile_range_policy(
            caster_instance_id,
            target_instance_id,
            policy,
            target_range,
        )
    }

    pub(super) fn is_tile_in_skill_reach(
        &self,
        caster_instance_id: UnitInstanceId,
        target_position: Position,
        reach: SkillTargetReach,
        target_range: Option<&crate::game::battle::tile_range::TileRangePattern>,
    ) -> bool {
        let policy = match reach {
            SkillTargetReach::Pattern => crate::game::battle::tile_range::TileRangePolicy::Pattern,
            SkillTargetReach::Global => {
                crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles
            }
        };
        self.tile_reach_query_for_unit(caster_instance_id, policy, target_range)
            .is_some_and(|query| query.contains(target_position))
    }

    pub(in crate::game::battle::core) fn resolve_cast_origin_context(
        &self,
        cast_seq: Option<u64>,
        caster_instance_id: UnitInstanceId,
        allow_dead_caster: bool,
    ) -> Option<(Side, Position)> {
        if let Some(caster) = self.units.get(&caster_instance_id) {
            if caster.is_active() {
                if let Some(position) = self.live_unit_projected_tile(caster_instance_id) {
                    return Some((caster.owner, position));
                }
            } else if allow_dead_caster {
                if let Some(snapshot) = self.graveyard.get(&caster_instance_id) {
                    return Some((snapshot.owner, snapshot.position));
                }
            }
        }

        cast_seq.and_then(|seq| {
            self.active_skill_casts.get(&seq).and_then(|cast| {
                if allow_dead_caster || cast.allow_dead_caster {
                    Some((cast.caster_owner, cast.anchor_position))
                } else {
                    None
                }
            })
        })
    }

    fn buff_stacks_on_unit(
        &self,
        target_instance_id: UnitInstanceId,
        buff_id: crate::game::battle::buffs::BuffId,
    ) -> u8 {
        self.buffs
            .iter()
            .filter(|(key, _)| {
                key.target_instance_id == target_instance_id && key.buff_id == buff_id
            })
            .fold(0u8, |acc, (_, active)| acc.saturating_add(active.stacks))
    }

    fn resolve_skill_unit_reference(
        &self,
        caster_instance_id: UnitInstanceId,
        step_target: Option<SkillCastTarget>,
        unit: SkillUnitReference,
    ) -> Option<UnitInstanceId> {
        match unit {
            SkillUnitReference::SelfUnit => Some(caster_instance_id),
            SkillUnitReference::StepTarget => match step_target {
                Some(SkillCastTarget::Unit { unit_instance_id }) => Some(unit_instance_id),
                _ => None,
            },
        }
    }

    fn evaluate_step_condition(
        &self,
        cast_seq: u64,
        step_index: usize,
        caster_instance_id: UnitInstanceId,
        step_target: Option<SkillCastTarget>,
        condition: &SkillStepCondition,
    ) -> PreviousStepDamageGate {
        match condition {
            SkillStepCondition::Always => PreviousStepDamageGate::Satisfied,
            SkillStepCondition::IfPreviousStepDealtDamage => {
                self.previous_step_damage_gate(cast_seq, step_index)
            }
            SkillStepCondition::IfCasterHasBuff {
                buff_id,
                min_stacks,
            } => self
                .resolve_skill_unit_reference(
                    caster_instance_id,
                    step_target,
                    SkillUnitReference::SelfUnit,
                )
                .is_some_and(|unit_id| {
                    self.buff_stacks_on_unit(
                        unit_id,
                        crate::game::battle::buffs::BuffId::from_name(buff_id),
                    ) >= *min_stacks
                })
                .then_some(PreviousStepDamageGate::Satisfied)
                .unwrap_or(PreviousStepDamageGate::Unsatisfied),
        }
    }

    fn evaluate_step_repeat_count(
        &self,
        caster_instance_id: UnitInstanceId,
        step_target: Option<SkillCastTarget>,
        repeat: &SkillStepRepeat,
    ) -> usize {
        match repeat {
            SkillStepRepeat::Once => 1,
            SkillStepRepeat::Times { count } => usize::from(*count),
            SkillStepRepeat::ByBuffStacks { unit, buff_id, max } => {
                let Some(unit_id) =
                    self.resolve_skill_unit_reference(caster_instance_id, step_target, *unit)
                else {
                    return 0;
                };
                let stacks = self.buff_stacks_on_unit(
                    unit_id,
                    crate::game::battle::buffs::BuffId::from_name(buff_id),
                );
                let stacks = match max {
                    Some(max_stacks) => stacks.min(*max_stacks),
                    None => stacks,
                };
                usize::from(stacks)
            }
        }
    }

    fn terminal_condition_matches(
        &self,
        condition: &TerminalCondition,
        current_time_ms: u64,
    ) -> bool {
        match condition {
            TerminalCondition::AllRequiredEnemyGroupsDefeated => {
                self.all_required_enemy_groups_defeated()
            }
            TerminalCondition::UnitDefeated { unit_ref } => self
                .scenario_runtime
                .spawned
                .unit_refs
                .get(unit_ref)
                .and_then(|unit_id| self.units.get(unit_id))
                .is_some_and(|unit| unit.is_dead()),
            TerminalCondition::AllPlayerCombatantsDefeated => {
                let mut combatants = self
                    .units
                    .values()
                    .filter(|unit| unit.owner == Side::Player && unit.is_combatant())
                    .peekable();
                combatants.peek().is_some() && combatants.all(|unit| unit.is_dead())
            }
            TerminalCondition::TimeReached { time_ms } => current_time_ms >= *time_ms,
            TerminalCondition::BossSignalLatched { signal_id } => self
                .boss_runtime
                .as_ref()
                .is_some_and(|runtime| runtime.is_signal_latched(signal_id)),
        }
    }

    pub(super) fn compute_terminal_outcome(&self, current_time_ms: u64) -> Option<TerminalOutcome> {
        self.scenario
            .terminal_outcomes
            .iter()
            .filter(|outcome| (outcome.verdict != BattleVerdict::Victory || !self.required_transformation_alive())
                && self.terminal_condition_matches(&outcome.condition, current_time_ms))
            .min_by(|left, right| {
                let left_verdict = match left.verdict {
                    BattleVerdict::Defeat => 0_u8,
                    BattleVerdict::Victory => 1_u8,
                };
                let right_verdict = match right.verdict {
                    BattleVerdict::Defeat => 0_u8,
                    BattleVerdict::Victory => 1_u8,
                };
                left_verdict
                    .cmp(&right_verdict)
                    .then_with(|| right.priority.cmp(&left.priority))
                    .then_with(|| left.id.cmp(&right.id))
            })
            .cloned()
    }

    pub(crate) fn finish_stress_extraction(&mut self, state: &mut BattleExecutionState) -> Result<BattleResult, GameError> {
        if !self.stress_round_status().is_some_and(|(_, _, waiting)| waiting) || state.is_finished() || self.required_transformation_alive() {
            return Err(GameError::InvalidAction);
        }
        let outcome = self.scenario.terminal_outcomes.iter().find(|o| o.verdict == BattleVerdict::Victory).cloned().ok_or(GameError::InvalidAction)?;
        Self::finish_signal(state, state.last_event_time_ms, outcome);
        self.finalize_battle_execution_in_candidate(state)
    }

    fn all_required_enemy_groups_defeated(&self) -> bool {
        !self.required_transformation_alive() && !self.shot_cohort_alive() && self.scenario
            .groups
            .iter()
            .filter(|group| group.side == Side::Opponent && group.required_for_victory)
            .all(|group| {
                !group.spawns.is_empty()
                    && self
                        .scenario_runtime
                        .spawned
                        .spawned_groups
                        .contains(&group.id)
                    && group.spawns.iter().all(|spawn| {
                        self.scenario_runtime
                            .spawned
                            .unit_refs
                            .get(&spawn.unit_ref)
                            .and_then(|unit_id| self.units.get(unit_id))
                            .is_none_or(|unit| !unit.is_active())
                    })
            })
    }

    fn finish_battle(
        &mut self,
        time_ms: u64,
        outcome_id: BattleOutcomeId,
        verdict: BattleVerdict,
    ) -> Result<BattleResult, GameError> {
        self.settle_cohort_bounties(verdict, time_ms)?;
        self.settle_equipment_forms(verdict, time_ms)?;
        self.clear_sustained(time_ms)?;
        self.clear_skill_areas(time_ms)?;
        self.clear_basic_tile_impacts(time_ms)?;
        self.clear_basic_projectile_batches(time_ms)?;
        self.clear_independent_objects(time_ms)?;
        let winner = verdict.winner();
        self.record_event_log(time_ms, BattleLogEvent::BattleEnd { winner });
        let participant_results = self.participant_results();
        let result_stats = self.result_stats_accumulator.clone().finish(
            winner,
            &participant_results,
            &self.game_data.run_policy.battle_result_stats,
        );
        Ok(BattleResult {
            runtime_facts: self.runtime_fact_snapshot()?,
            outcome_id,
            verdict,
            winner,
            result_stats,
            #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
            event_log: self.event_log.clone(),
            participant_results,
        })
    }

    fn finish_signal(
        state: &mut BattleExecutionState,
        time_ms: u64,
        outcome: TerminalOutcome,
    ) -> BattleStepOutcome {
        let signal = BattleFinishSignal {
            time_ms,
            outcome_id: outcome.id,
            verdict: outcome.verdict,
            winner: outcome.verdict.winner(),
        };
        state.last_event_time_ms = time_ms;
        state.terminal_outcome = Some(signal.clone());
        BattleStepOutcome::Finished(signal)
    }

    fn participant_results(&self) -> Vec<ParticipantBattleResult> {
        let mut results = self
            .units
            .values()
            .map(|unit| ParticipantBattleResult {
                unit_instance_id: unit.instance_id,
                owned_uuid: unit.source_owned_uuid,
                side: unit.owner,
                survived: !unit.is_dead(),
                final_hp: unit.stats.current_health,
                max_hp: unit.stats.max_health,
                became_incapacitated: unit.stats.current_health == 0,
            })
            .collect::<Vec<_>>();
        results.sort_by(|left, right| {
            side_sort_key(left.side)
                .cmp(&side_sort_key(right.side))
                .then_with(|| left.unit_instance_id.cmp(&right.unit_instance_id))
        });
        results
    }

    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub fn init_intial_events(&mut self) {
        let mut unit_ids: Vec<UnitInstanceId> = self.units.keys().copied().collect();
        unit_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        for unit_id in unit_ids {
            self.schedule_initial_attack_for_unit(unit_id, 0);
        }
    }

    pub(super) fn schedule_initial_attack_for_unit(
        &mut self,
        unit_id: UnitInstanceId,
        time_ms: u64,
    ) {
        self.schedule_resonance_pulse(unit_id, time_ms);
        let Some(unit) = self.units.get_mut(&unit_id) else {
            return;
        };
        if !unit.can_basic_attack() {
            return;
        }
        unit.next_basic_attack_ms = time_ms;
        unit.pending_basic_attack = false;
        self.event_queue.push(BattleEvent::AttackStart {
            time_ms,
            attacker_instance_id: unit_id,
            target_instance_id: None,
            schedule_next: true,
            cause: BattleEventCause::Root {
                kind: BattleEventRootCause::Period,
            },
        });
    }

    pub(super) fn record_spawned_units(
        &mut self,
        time_ms: u64,
        cause: BattleEventCause,
        spawned_unit_ids: &[UnitInstanceId],
    ) -> Result<(), GameError> {
        if spawned_unit_ids.is_empty() {
            return Ok(());
        }

        self.with_recording_context(cause, |core| -> Result<(), GameError> {
            let mut unit_records: Vec<(
                UnitInstanceId,
                Side,
                crate::game::battle::types::BattleUnitRole,
                crate::game::battle::types::BattleUnitThreatClass,
                crate::game::battle::types::MobilityKind,
                Uuid,
                crate::game::battle::types::BattleUnitSourceIdentity,
                _,
                UnitStats,
            )> = spawned_unit_ids
                .iter()
                .filter_map(|unit_id| {
                    core.units.get(unit_id).map(|unit| {
                        (
                            unit.instance_id,
                            unit.owner,
                            unit.role,
                            unit.threat_class,
                            unit.mobility_kind,
                            unit.base_uuid,
                            unit.source_identity.clone(),
                            unit.world_position().quantized_milli(),
                            unit.stats,
                        )
                    })
                })
                .collect();
            unit_records.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
            for (
                unit_instance_id,
                owner,
                role,
                threat_class,
                mobility_kind,
                base_uuid,
                unit_source,
                world_position,
                stats,
            ) in unit_records
            {
                let compendium_candidate = if owner == Side::Opponent {
                    match &unit_source {
                        BattleUnitSourceIdentity::Abnormality { abnormality_id, .. } => {
                            Some(CompendiumCandidate {
                                id: abnormality_id.clone(),
                                kind: CompendiumCandidateKind::Abnormality,
                                base_uuid,
                            })
                        }
                        BattleUnitSourceIdentity::CorrodedEmployee { profile_id, .. } => {
                            Some(CompendiumCandidate {
                                id: profile_id.clone(),
                                kind: CompendiumCandidateKind::CorrodedEmployee,
                                base_uuid,
                            })
                        }
                        BattleUnitSourceIdentity::Employee { .. }
                        | BattleUnitSourceIdentity::FacilityEntity { .. }
                        | BattleUnitSourceIdentity::DefenseObject { .. }
                        | BattleUnitSourceIdentity::TestFixture { .. }
                        | BattleUnitSourceIdentity::IndependentObject { .. }
                        | BattleUnitSourceIdentity::SummonedUnit { .. } => None,
                    }
                } else {
                    None
                };
                let spawn_seq = core.record_event_log(
                    time_ms,
                    BattleLogEvent::UnitSpawned {
                        unit_instance_id,
                        owner,
                        role,
                        threat_class,
                        mobility_kind,
                        base_uuid,
                        unit_source,
                        world_position,
                        stats,
                    },
                );
                if let Some(candidate) = compendium_candidate {
                    core.record_compendium_candidate(spawn_seq, candidate)
                        .map_err(|error| {
                            GameError::InvalidStaticData(format!(
                                "invalid encounter compendium spawn candidate: {error:?}"
                            ))
                        })?;
                }
            }

            let mut item_records: Vec<(Uuid, Side, UnitInstanceId, Uuid)> = core
                .items
                .values()
                .filter(|item| spawned_unit_ids.contains(&item.owner_unit_instance))
                .map(|i| (i.instance_id, i.owner, i.owner_unit_instance, i.base_uuid))
                .collect();
            item_records.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
            for (item_instance_id, owner, owner_unit_instance_id, base_uuid) in item_records {
                core.record_event_log(
                    time_ms,
                    BattleLogEvent::ItemSpawned {
                        item_instance_id,
                        owner,
                        owner_unit_instance_id,
                        base_uuid,
                    },
                );
            }
            Ok(())
        })
    }

    fn enqueue_scenario_events(&mut self) -> Result<(), GameError> {
        control::charge(self.scenario.events.len() as u64, "scenario_event_copy")?;
        for event in self.scenario.events.clone() {
            control::poll("scenario_event_enqueue")?;
            match (event.trigger, event.action) {
                (ScenarioTrigger::Wave(activation), ScenarioAction::SpawnGroup { group_id }) => {
                    self.register_wave_trigger(group_id, activation.clone())?;
                    self.enqueue_wave_wakes(&activation)?;
                }
                (trigger, ScenarioAction::SpawnGroup { group_id }) => {
                    let time_ms = match trigger {
                        ScenarioTrigger::AtBattleStart => 0,
                        ScenarioTrigger::AtTimeMs(time_ms) => time_ms,
                        ScenarioTrigger::Wave(_) => unreachable!(),
                    };
                    self.event_queue.push(BattleEvent::spawn_group(
                        time_ms,
                        group_id,
                        BattleEventCause::Root { kind: BattleEventRootCause::Init },
                    )?);
                }
            }
            control::poll("scenario_event_enqueued")?;
        }
        Ok(())
    }

    fn has_queued_spawn_group_at(&self, time_ms: u64) -> bool {
        self.event_queue.iter().any(
            |event| matches!(event.as_ref(), BattleEvent::SpawnGroup { time_ms: queued, .. } if *queued == time_ms),
        )
    }

    fn run_on_battle_start_hooks(&mut self) {
        let mut on_battle_start_command_batches = Vec::new();
        let mut unit_ids: Vec<_> = self.units.keys().copied().collect();
        unit_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        for unit_id in unit_ids {
            let occurrence_id = self.proc_occurrence_id(
                crate::game::stats::TriggerType::OnBattleStart,
                unit_id,
                None,
                0,
                0,
            );
            on_battle_start_command_batches.extend(
                self.dispatch_item_trigger_occurrence(TriggerOccurrenceContext {
                    trigger_type: crate::game::stats::TriggerType::OnBattleStart,
                    trigger_unit_id: unit_id,
                    counterpart_unit_id: None,
                    target_id: None,
                    occurrence_id,
                    occurrence_index: 0,
                    allow_dead_caster: false,
                    committed_hit: None,
                    time_ms: 0,
                })
                .into_command_batches(),
            );
        }
        if !on_battle_start_command_batches.is_empty() {
            self.with_recording_root(BattleEventRootCause::Init, |core| {
                core.process_command_batches(on_battle_start_command_batches, 0);
            });
        }
    }

    pub(super) fn schedule_continuous_movement_tick(&mut self, time_ms: u64) {
        let movement_tick_ms = self.movement_tick_ms();
        if self
            .last_continuous_movement_tick_ms
            .is_some_and(|last_time_ms| time_ms < last_time_ms.saturating_add(movement_tick_ms))
        {
            return;
        }

        if self.event_queue.iter().any(
            |event| matches!(event.as_ref(), BattleEvent::ContinuousMovementTick { time_ms: queued } if *queued == time_ms),
        ) {
            return;
        }

        self.event_queue
            .push(BattleEvent::ContinuousMovementTick { time_ms });
    }

    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub fn run_battle(&mut self) -> Result<BattleResult, GameError> {
        self.run_battle_with_setup(|_| {})
    }

    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub fn run_battle_with_setup<F>(&mut self, setup: F) -> Result<BattleResult, GameError>
    where
        F: FnOnce(&mut Self),
    {
        self.run_battle_with_hooks(setup, None::<fn(&mut Self)>)
    }

    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub fn run_battle_with_post_spawn_setup<F>(
        &mut self,
        post_spawn_setup: F,
    ) -> Result<BattleResult, GameError>
    where
        F: FnOnce(&mut Self),
    {
        self.run_battle_with_hooks(|_| {}, Some(post_spawn_setup))
    }

    pub(in crate::game::battle::core) fn reset_runtime_state_for_battle(&mut self) {
        self.independent_objects = Default::default();
        self.spawn_kernel.reset_independent_requests();
        self.corrosion_freeze = None;
        self.runtime_facts = super::runtime_facts::RuntimeFactState::new(self.runtime_facts.limits);
        self.runtime_fact_source = Default::default();
        self.runtime_strike_scope = None;
        if let Some(amount_observation_enabled) = self
            .authoring_observation
            .as_ref()
            .map(|owner| owner.amount_observation_enabled)
        {
            self.enable_authoring_observation_with_options(
                self.authoring_trace.options(),
                amount_observation_enabled,
            );
        }
        self.units.clear();
        self.tactical_states.clear();
        self.items.clear();
        self.graveyard.clear();
        self.buffs.clear();
        self.active_skill_casts.clear();
        self.ability_proc_states.clear();
        self.projectiles.clear();
        self.basic_projectile_batches.clear();
        self.active_projectiles.clear();
        self.active_areas.clear();
        self.active_movement_segments.clear();
        self.sustained = Default::default();
        self.field_work_objects = super::field_work::runtime_from_scenario(&self.scenario);
        self.last_continuous_movement_tick_ms = None;
        self.movement_backend =
            std::sync::Arc::new(super::movement::RapierMovementWorld::default());
        self.battlefield.clear();
        self.event_log = BattleEventLog::with_runtime_safety(self.game_data.runtime_safety());
        self.event_log_seq = 1;
        self.result_stats_accumulator = Default::default();
        self.skill_cast_seq = 0;
        self.projectile_seq = 0;
        self.area_seq = 0;
        self.damage_source_seq = 0;
        self.live_signals.clear();
        self.recording_cause_stack.clear();
        self.recording_source_command_stack.clear();
        self.event_queue.clear();
        self.scenario_runtime = Default::default();
        self.boss_runtime = self
            .scenario
            .boss_runtime
            .as_ref()
            .map(super::boss_runtime::BossEncounterRuntime::from_setup);
    }

    pub fn start_battle_execution(&mut self) -> Result<BattleExecutionState, GameError> {
        self.start_battle_execution_with_setup(|_| {})
    }

    pub fn step_battle_execution(
        &mut self,
        state: &mut BattleExecutionState,
    ) -> Result<BattleStepOutcome, GameError> {
        self.step_battle_execution_with_post_spawn_setup(state, &mut None::<fn(&mut Self)>)
    }

    pub(crate) fn step_battle_execution_in_candidate(
        &mut self,
        state: &mut BattleExecutionState,
    ) -> Result<BattleStepOutcome, GameError> {
        self.run_profile_budget(|core| {
            core.step_battle_execution_in_candidate_with_setup(state, &mut None::<fn(&mut Self)>)
        })
    }

    pub fn step_battle_execution_until(
        &mut self,
        state: &mut BattleExecutionState,
        target_time_ms: u64,
    ) -> Result<BattleStepOutcome, GameError> {
        self.step_battle_execution_until_with_post_spawn_setup(
            state,
            target_time_ms,
            &mut None::<fn(&mut Self)>,
        )
    }

    pub fn step_battle_execution_by(
        &mut self,
        state: &mut BattleExecutionState,
        delta_ms: u64,
    ) -> Result<BattleStepOutcome, GameError> {
        let target_time_ms = state.next_time_after(delta_ms);
        self.step_battle_execution_until(state, target_time_ms)
    }

    pub fn event_log_entries_after_seq(&self, last_seen_seq: u64) -> Vec<BattleEventLogEntry> {
        self.event_log.entries_after_seq(last_seen_seq)
    }

    pub fn drain_live_signals(&mut self) -> Vec<BattleLiveSignal> {
        let signals = std::mem::take(&mut self.live_signals);
        for signal in &signals {
            let BattleLiveSignal::TacticalResourceDelta {
                retained_extent, ..
            } = signal;
            self.runtime_facts.release_fact_storage(*retained_extent);
        }
        signals
    }

    pub fn finalize_battle_execution(
        &mut self,
        state: &mut BattleExecutionState,
    ) -> Result<BattleResult, GameError> {
        self.runtime_timestamp(state, |core, state| {
            core.finalize_battle_execution_in_candidate(state)
        })
    }

    pub(crate) fn finalize_battle_execution_in_candidate(
        &mut self,
        state: &mut BattleExecutionState,
    ) -> Result<BattleResult, GameError> {
        self.ensure_not_forced_paused()?;
        let Some(finish) = state.terminal_outcome.clone() else {
            return Err(GameError::InvalidAction);
        };
        if state.finalized {
            return Err(GameError::InvalidAction);
        }

        let result = self.finish_battle(finish.time_ms, finish.outcome_id, finish.verdict)?;
        self.ensure_not_forced_paused()?;
        control::poll("battle_finalize_commit")?;
        state.finalized = true;
        Ok(result)
    }

    pub fn apply_live_command(
        &mut self,
        command: BattleLiveCommand,
    ) -> Result<BattleLiveCommandOutcome, GameError> {
        self.runtime_transaction(|core| core.apply_live_command_in_candidate(command))
    }

    pub(crate) fn apply_live_command_in_candidate(
        &mut self,
        command: BattleLiveCommand,
    ) -> Result<BattleLiveCommandOutcome, GameError> {
        self.run_profile_budget(|core| core.apply_live_command_inner(command))
    }

    fn apply_live_command_inner(
        &mut self,
        command: BattleLiveCommand,
    ) -> Result<BattleLiveCommandOutcome, GameError> {
        self.ensure_not_forced_paused()?;
        control::charge(1, "battle_live_command")?;
        let command_time = match &command {
            BattleLiveCommand::DeployPlayerUnit { time_ms, .. }
            | BattleLiveCommand::WithdrawUnit { time_ms, .. }
            | BattleLiveCommand::SetEngagementStance { time_ms, .. }
            | BattleLiveCommand::TacticalRelocateUnit { time_ms, .. }
            | BattleLiveCommand::TacticalSwapUnits { time_ms, .. }
            | BattleLiveCommand::ReinputSustainedSkill { time_ms, .. }
            | BattleLiveCommand::CommitInputPlan {time_ms,..} | BattleLiveCommand::CancelInputPlan {time_ms,..}
            | BattleLiveCommand::SelectRelation { time_ms, .. }
            | BattleLiveCommand::ActivateSkill { time_ms, .. } => *time_ms,
        };
        if self.input_plan_paused() && !(matches!(&command,BattleLiveCommand::CommitInputPlan{..}|BattleLiveCommand::CancelInputPlan{..}) || (self.input_plan_reprepare_allowed() && matches!(&command,BattleLiveCommand::ActivateSkill{unit_id,skill_id,..} if self.current_skill_view(skill_id,*unit_id,&Default::default()).is_some_and(Self::input_plan_skill)))){return Err(GameError::InvalidAction);}
        if matches!(&command,BattleLiveCommand::SelectRelation{..}|BattleLiveCommand::CommitInputPlan{..}|BattleLiveCommand::CancelInputPlan{..}|BattleLiveCommand::ActivateSkill{..}) && self.sustained_time_is_stale(command_time) {return Err(GameError::InvalidAction);}
        self.reconcile_sustained(command_time)?;
        let result = match command {
            BattleLiveCommand::DeployPlayerUnit {
                draft,
                position,
                tile_kind,
                facing,
                instance_salt,
                time_ms,
                current_hp_policy,
            } => {
                let unit_id = self.deploy_player_unit(
                    draft,
                    position,
                    tile_kind,
                    facing,
                    instance_salt,
                    time_ms,
                    current_hp_policy,
                )?;
                Ok(BattleLiveCommandOutcome::UnitDeployed { unit_id })
            }
            BattleLiveCommand::WithdrawUnit { unit_id, time_ms } => {
                self.withdraw_unit(unit_id, time_ms)?;
                self.reconcile_independent_object_removals(time_ms)?;
                self.wake_boss_action_waiters(time_ms)?;
                Ok(BattleLiveCommandOutcome::UnitWithdrawn { unit_id })
            }
            BattleLiveCommand::SetEngagementStance {
                unit_id,
                stance,
                time_ms,
            } => {
                self.set_engagement_stance_in_candidate(unit_id, stance, time_ms)?;
                Ok(BattleLiveCommandOutcome::EngagementStanceChanged { unit_id, stance })
            }
            BattleLiveCommand::TacticalRelocateUnit {
                unit_id,
                destination,
                tile_kind,
                facing,
                time_ms,
            } => {
                self.tactically_relocate_unit_in_candidate(
                    unit_id,
                    destination,
                    tile_kind,
                    facing,
                    time_ms,
                )?;
                Ok(BattleLiveCommandOutcome::UnitTacticallyRelocated { unit_id })
            }
            BattleLiveCommand::TacticalSwapUnits {
                first_unit_id,
                second_unit_id,
                first_destination_tile_kind,
                second_destination_tile_kind,
                first_facing,
                second_facing,
                time_ms,
            } => {
                self.tactically_swap_units_in_candidate(
                    first_unit_id,
                    second_unit_id,
                    first_destination_tile_kind,
                    second_destination_tile_kind,
                    first_facing,
                    second_facing,
                    time_ms,
                )?;
                Ok(BattleLiveCommandOutcome::UnitsTacticallySwapped {
                    first_unit_id,
                    second_unit_id,
                })
            }
            BattleLiveCommand::ReinputSustainedSkill {
                unit_id,
                skill_id,
                activation,
                target,
                time_ms,
            } => {
                self.reinput_sustained_skill(unit_id, &skill_id, activation, target, time_ms)?;
                Ok(BattleLiveCommandOutcome::SkillActivated { unit_id })
            }
            BattleLiveCommand::CommitInputPlan {unit_id,input,time_ms} => {let receipt=self.commit_input_plan(unit_id,input,time_ms)?;Ok(BattleLiveCommandOutcome::InputPlanCommitted{unit_id,receipt})},
            BattleLiveCommand::CancelInputPlan {unit_id,preparation,disconnected,time_ms:_} => {self.cancel_input_plan(unit_id,preparation,disconnected)?;Ok(BattleLiveCommandOutcome::InputPlanCancelled{unit_id})},
            BattleLiveCommand::SelectRelation { unit_id, entry, target, time_ms } => {
                self.select_relation_target(unit_id,&entry,target,time_ms)?;
                Ok(BattleLiveCommandOutcome::RelationChanged{unit_id,target:self.selected_relation_target(unit_id)})
            },
            BattleLiveCommand::ActivateSkill {
                unit_id,
                skill_id,
                target,
                time_ms,
            } => {
                if self.current_skill_view(&skill_id,unit_id,&Default::default()).is_some_and(Self::input_plan_skill){
                    if target.is_some(){return Err(GameError::InvalidAction);}let preparation=self.prepare_input_plan(unit_id,&skill_id,time_ms)?;return Ok(BattleLiveCommandOutcome::InputPlanPrepared{unit_id,preparation});
                }
                self.start_manual_skill_cast(time_ms, unit_id, skill_id, target)?;
                Ok(BattleLiveCommandOutcome::SkillActivated { unit_id })
            }
        };
        if result.is_ok() {
            self.reconcile_sustained(command_time)?;
        }
        control::poll("battle_live_command_complete")?;
        result
    }

    pub fn apply_live_command_with_source_command_id(
        &mut self,
        command: BattleLiveCommand,
        source_command_id: Option<&str>,
    ) -> Result<BattleLiveCommandOutcome, GameError> {
        self.runtime_transaction(|core| {
            core.apply_live_command_with_source_command_id_in_candidate(command, source_command_id)
        })
    }

    pub(crate) fn apply_live_command_with_source_command_id_in_candidate(
        &mut self,
        command: BattleLiveCommand,
        source_command_id: Option<&str>,
    ) -> Result<BattleLiveCommandOutcome, GameError> {
        self.run_profile_budget(|core| match source_command_id {
            Some(id) => core.with_recording_source_command_id(id, |core| {
                core.apply_live_command_in_candidate(command)
            }),
            None => core.apply_live_command_in_candidate(command),
        })
    }

    fn start_battle_execution_with_setup<PreSetup>(
        &mut self,
        setup: PreSetup,
    ) -> Result<BattleExecutionState, GameError>
    where
        PreSetup: FnOnce(&mut Self),
    {
        self.runtime_transaction(|core| core.start_battle_execution_with_setup_in_candidate(setup))
    }

    pub(crate) fn start_battle_execution_in_candidate(
        &mut self,
    ) -> Result<BattleExecutionState, GameError> {
        self.run_profile_budget(|core| core.start_battle_execution_with_setup_in_candidate(|_| {}))
    }

    fn start_battle_execution_with_setup_in_candidate<PreSetup>(
        &mut self,
        setup: PreSetup,
    ) -> Result<BattleExecutionState, GameError>
    where
        PreSetup: FnOnce(&mut Self),
    {
        self.ensure_not_forced_paused()?;
        crate::game::battle::runtime_control::seed_retained_extent(
            self.retained_runtime_fact_extent()?,
        )?;
        control::charge(1, "battle_startup")?;
        self.scenario.validate()?;

        self.reset_runtime_state_for_battle();
        self.start_causal_periodics(Some(0))?;
        self.prepare_runtime_resources()?;

        for obstacle in self.scenario.battlefield.obstacles.clone() {
            control::charge(1, "startup_obstacle")?;
            self.battlefield.add_static_obstacle(obstacle)?;
        }
        setup(self);
        control::poll("startup_setup")?;
        self.build_runtime_field()?;
        self.reconcile_nearby_defense()?;
        self.reconcile_ready_wait(0)?;
        self.start_causal_periodics(Some(0))?;

        self.with_recording_root(BattleEventRootCause::Init, |core| {
            core.record_event_log(
                0,
                BattleLogEvent::BattleStart {
                    width: core.battlefield.width(),
                    height: core.battlefield.height(),
                },
            );
        });

        self.enqueue_scenario_events()?;
        control::poll("battle_startup_complete")?;

        Ok(BattleExecutionState {
            last_event_time_ms: 0,
            battle_start_hooks_ran: false,
            terminal_outcome: None,
            finalized: false,
        })
    }

    fn step_battle_execution_with_post_spawn_setup<PostSpawnSetup>(
        &mut self,
        state: &mut BattleExecutionState,
        post_spawn_setup: &mut Option<PostSpawnSetup>,
    ) -> Result<BattleStepOutcome, GameError>
    where
        PostSpawnSetup: FnOnce(&mut Self),
    {
        self.runtime_timestamp(state, |core, state| {
            core.step_battle_execution_in_candidate_with_setup(state, post_spawn_setup)
        })
    }

    fn step_battle_execution_in_candidate_with_setup<PostSpawnSetup>(
        &mut self,
        state: &mut BattleExecutionState,
        post_spawn_setup: &mut Option<PostSpawnSetup>,
    ) -> Result<BattleStepOutcome, GameError>
    where
        PostSpawnSetup: FnOnce(&mut Self),
    {
        self.ensure_not_forced_paused()?;
        if self.input_plan_paused(){return Ok(BattleStepOutcome::Running);}
        control::charge(1, "battle_bucket_entry")?;
        if self.stress_round_status().is_some_and(|(_, _, waiting)| waiting) { return Ok(BattleStepOutcome::Running); }
        if state.is_finished() {
            return Err(GameError::InvalidAction);
        }

        let Some(next_time_ms) = self.event_queue.peek_time_ms() else {
            return Ok(
                match self.compute_terminal_outcome(state.last_event_time_ms) {
                    Some(outcome) => Self::finish_signal(state, state.last_event_time_ms, outcome),
                    None => BattleStepOutcome::Running,
                },
            );
        };

        let current_time_ms = next_time_ms;
        state.last_event_time_ms = current_time_ms;

        let mut bucket: Vec<super::queue::QueuedEvent> = Vec::new();
        while matches!(self.event_queue.peek_time_ms(), Some(time_ms) if time_ms == current_time_ms)
        {
            control::admit(Resource::BucketEvents, bucket.len(), 1, "same_time_bucket")?;
            control::charge(1, "same_time_pop")?;
            super::queue::reserve_timestamp_bucket_slot(&mut bucket, "same_time_bucket_storage")?;
            bucket.push(self.event_queue.pop().unwrap());
        }

        loop {
            control::charge(1, "same_time_iteration")?;
            if bucket.is_empty() {
                let mut new_bucket: Vec<super::queue::QueuedEvent> = Vec::new();
                while matches!(self.event_queue.peek_time_ms(), Some(time_ms) if time_ms == current_time_ms)
                {
                    control::admit(
                        Resource::BucketEvents,
                        new_bucket.len(),
                        1,
                        "same_time_refill",
                    )?;
                    control::charge(1, "same_time_pop")?;
                    super::queue::reserve_timestamp_bucket_slot(
                        &mut new_bucket,
                        "same_time_refill_storage",
                    )?;
                    new_bucket.push(self.event_queue.pop().unwrap());
                }

                if new_bucket.is_empty() {
                    self.reconcile_independent_objects(current_time_ms)?;
                    if self.event_queue.peek_time_ms() == Some(current_time_ms) {
                        continue;
                    }
                    break;
                }

                bucket = new_bucket;
                continue;
            }

            bucket.sort_by(|a, b| b.cmp(a));
            let mut boss_boundary_requested = false;
            let mut pending_events = bucket.drain(..);
            while let Some(event) = pending_events.next() {
                boss_boundary_requested |=
                    self.battle_event_requests_boss_boundary(event.event.view().as_ref());
                self.process_event_with_pending_bucket(
                    event,
                    current_time_ms,
                    pending_events.as_slice(),
                )?;
                if self.forced_pause_remaining_ms().is_some() {
                    for pending in pending_events {
                        self.event_queue.requeue_existing(pending)?;
                    }
                    return Ok(BattleStepOutcome::Running);
                }
            }
            drop(pending_events);

            boss_boundary_requested |= self.settle_wave_triggers(current_time_ms)?;

            if current_time_ms == 0
                && !state.battle_start_hooks_ran
                && !self.has_queued_spawn_group_at(0)
            {
                if let Some(setup) = post_spawn_setup.take() {
                    setup(self);
                }
                state.battle_start_hooks_ran = true;
                self.run_on_battle_start_hooks();
                control::poll("battle_start_hooks")?;
                self.schedule_continuous_movement_tick(0);
                control::poll("initial_movement_enqueue")?;
                boss_boundary_requested = true;
                boss_boundary_requested |= self.settle_wave_triggers(current_time_ms)?;
            }

            if boss_boundary_requested {
                self.run_boss_runtime_boundary(current_time_ms)?;
                if self.settle_wave_triggers(current_time_ms)? {
                    self.run_boss_runtime_boundary(current_time_ms)?;
                }
            }

            if self.has_pending_attack_resolve_at(current_time_ms) {
                continue;
            }
        }

        self.settle_stress_round();
        self.runtime_facts.release_empty_action_storage();
        control::poll("battle_bucket_complete")?;
        if let Some(outcome) = self.compute_terminal_outcome(current_time_ms) {
            return Ok(Self::finish_signal(state, current_time_ms, outcome));
        }

        Ok(BattleStepOutcome::Running)
    }

    fn step_battle_execution_until_with_post_spawn_setup<PostSpawnSetup>(
        &mut self,
        state: &mut BattleExecutionState,
        target_time_ms: u64,
        post_spawn_setup: &mut Option<PostSpawnSetup>,
    ) -> Result<BattleStepOutcome, GameError>
    where
        PostSpawnSetup: FnOnce(&mut Self),
    {
        self.run_profile_budget(|core| {
            core.step_battle_execution_until_inner(state, target_time_ms, post_spawn_setup, true)
        })
    }

    pub(crate) fn step_battle_execution_until_in_candidate(
        &mut self,
        state: &mut BattleExecutionState,
        target_time_ms: u64,
    ) -> Result<BattleStepOutcome, GameError> {
        self.run_profile_budget(|core| {
            core.step_battle_execution_until_inner(
                state,
                target_time_ms,
                &mut None::<fn(&mut Self)>,
                false,
            )
        })
    }

    pub(crate) fn step_battle_execution_by_in_candidate(
        &mut self,
        state: &mut BattleExecutionState,
        delta_ms: u64,
    ) -> Result<BattleStepOutcome, GameError> {
        let target_time_ms = state.next_time_after(delta_ms);
        self.step_battle_execution_until_in_candidate(state, target_time_ms)
    }

    fn advance_battle_clock_in_candidate(
        &self,
        state: &mut BattleExecutionState,
        target_time_ms: u64,
    ) -> Result<BattleStepOutcome, GameError> {
        state.last_event_time_ms = target_time_ms;
        let outcome = match self.compute_terminal_outcome(target_time_ms) {
            Some(outcome) => Self::finish_signal(state, target_time_ms, outcome),
            None => BattleStepOutcome::Running,
        };
        control::poll("battle_clock_commit")?;
        Ok(outcome)
    }

    fn step_battle_execution_until_inner<PostSpawnSetup>(
        &mut self,
        state: &mut BattleExecutionState,
        target_time_ms: u64,
        post_spawn_setup: &mut Option<PostSpawnSetup>,
        stage_timestamps: bool,
    ) -> Result<BattleStepOutcome, GameError>
    where
        PostSpawnSetup: FnOnce(&mut Self),
    {
        self.ensure_not_forced_paused()?;
        if self.input_plan_paused(){return Ok(BattleStepOutcome::Running);}
        control::charge(1, "battle_clock_entry")?;
        if state.is_finished() {
            return Err(GameError::InvalidAction);
        }

        if target_time_ms < state.last_event_time_ms {
            return Ok(BattleStepOutcome::Running);
        }

        // A user callback can replace the public unit map without its private
        // insertion history. Exclude its entire invocation from scratch reuse.
        let reuse_units = stage_timestamps
            && post_spawn_setup.is_none()
            && !crate::authoring::control::is_active();
        let mut timestamp_units = None;
        loop {
            if self.input_plan_paused() || self.forced_pause_remaining_ms().is_some() || self.stress_round_status().is_some_and(|(_, _, waiting)| waiting) {
                return Ok(BattleStepOutcome::Running);
            }
            control::charge(1, "battle_clock_iteration")?;
            let next_time_ms = self.event_queue.peek_time_ms();
            if next_time_ms.is_none_or(|time| time > target_time_ms) {
                return if stage_timestamps {
                    // No event is consumed and the core is only borrowed. Stage
                    // the execution clock/terminal signal, then commit after
                    // polling; a full gameplay candidate has no changed owner.
                    let mut candidate = state.clone();
                    let outcome =
                        self.advance_battle_clock_in_candidate(&mut candidate, target_time_ms)?;
                    *state = candidate;
                    Ok(outcome)
                } else {
                    self.advance_battle_clock_in_candidate(state, target_time_ms)
                };
            }

            let outcome = if reuse_units {
                self.runtime_timestamp_reusing(state, &mut timestamp_units, |core, state| {
                    core.step_battle_execution_in_candidate_with_setup(state, post_spawn_setup)
                })?
            } else if stage_timestamps {
                self.step_battle_execution_with_post_spawn_setup(state, post_spawn_setup)?
            } else {
                self.step_battle_execution_in_candidate_with_setup(state, post_spawn_setup)?
            };
            match outcome {
                BattleStepOutcome::Running => {
                    if state.last_event_time_ms >= target_time_ms {
                        return Ok(BattleStepOutcome::Running);
                    }
                }
                finished @ BattleStepOutcome::Finished(_) => return Ok(finished),
            }
        }
    }

    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    fn run_battle_with_hooks<PreSetup, PostSpawnSetup>(
        &mut self,
        setup: PreSetup,
        mut post_spawn_setup: Option<PostSpawnSetup>,
    ) -> Result<BattleResult, GameError>
    where
        PreSetup: FnOnce(&mut Self),
        PostSpawnSetup: FnOnce(&mut Self),
    {
        let mut state = self.start_battle_execution_with_setup(setup)?;
        loop {
            match self
                .step_battle_execution_with_post_spawn_setup(&mut state, &mut post_spawn_setup)?
            {
                BattleStepOutcome::Running => {}
                BattleStepOutcome::Finished(_) => {
                    return self.finalize_battle_execution(&mut state);
                }
            }
        }
    }

    pub(in crate::game::battle::core) fn is_alive_enemy(
        &self,
        unit_id: UnitInstanceId,
        owner: Side,
    ) -> bool {
        match self.units.get(&unit_id) {
            Some(unit) => unit.owner != owner && unit.is_active(),
            None => false,
        }
    }

    #[cfg(test)]
    pub(super) fn build_skill_step_commands(
        &mut self,
        caster_instance_id: UnitInstanceId,
        step: &SkillStepDef,
        targets: &[UnitInstanceId],
        committed_at_ms: u64,
        source_template: Option<&crate::game::battle::damage::DamageSourceSnapshot>,
    ) -> Result<(Vec<super::types::SkillCommand>, SkillStepResult), GameError> {
        control::charge(step.effects.len() as u64, "skill_effect_scan")?;
        let mut commands: Vec<super::types::SkillCommand> = Vec::new();
        let mut result = SkillStepResult {
            resolved_target_count: targets.len(),
            ..SkillStepResult::default()
        };

        let hinted_target_id = if targets.len() == 1 {
            Some(targets[0])
        } else {
            None
        };

        let step_damage_modifiers = super::skill_runtime::skill_step_damage_modifiers(step);

        for effect in &step.effects {
            control::charge(1, "skill_effect")?;
            let mut emit = |command| {
                control::admit(
                    control::Resource::EffectCommands,
                    commands.len(),
                    1,
                    "skill_effect_commands",
                )?;
                commands.push((command, effect.source().cloned()));
                Ok::<(), GameError>(())
            };
            match &effect.op {
                SkillEffectDef::PersonalShieldGrant { .. } => {
                    return Err(GameError::InvalidStaticData(
                        "PersonalShieldGrant requires the ordered occurrence executor".into(),
                    ))
                }
                SkillEffectDef::GuardCounter { .. } | SkillEffectDef::StatusStrike { .. } | SkillEffectDef::FusionPlacement { .. } | SkillEffectDef::PairedAreas { .. } | SkillEffectDef::PreparedVolley { .. } | SkillEffectDef::InputPlan { .. } | SkillEffectDef::RelationAction { .. } | SkillEffectDef::CounterChannel { .. } | SkillEffectDef::TileBurst { .. } | SkillEffectDef::PresenceBurst { .. } | SkillEffectDef::CreateObject { .. } => {
                    return Err(GameError::InvalidStaticData(
                        "Owned mark burst and object creation require the ordered occurrence executor".into(),
                    ))
                }
                SkillEffectDef::Damage {
                    amount,
                    damage_type,
                } => {
                    if *amount == 0 {
                        continue;
                    }
                    let damage_amount = amount.unsigned_abs();
                    for (hit_index, target_id) in targets.iter().enumerate() {
                        control::charge(1, "skill_effect_target")?;
                        let source_snapshot = if let Some(template) = source_template {
                            let mut snapshot = template.try_clone_fact_source()?;
                            snapshot.source = DamageSource::Ability;
                            snapshot.damage_type = *damage_type;
                            snapshot.base_damage = damage_amount;
                            snapshot.modifiers = step_damage_modifiers;
                            snapshot.minimum_damage = 0;
                            self.materialize_damage_source_snapshot_for_target_hit(
                                snapshot,
                                *target_id,
                                hit_index as u32,
                            )
                        } else if let Some(snapshot) = self.damage_source_snapshot_for_unit(
                            caster_instance_id,
                            *target_id,
                            DamageSource::Ability,
                            *damage_type,
                            damage_amount,
                            step_damage_modifiers,
                            0,
                            committed_at_ms,
                            false,
                        ) {
                            snapshot
                        } else {
                            continue;
                        };
                        emit(BattleCommand::ApplyDamage {
                            target_id: *target_id,
                            source_snapshot,
                        })?;
                    }
                    if !targets.is_empty() {
                        result.applied_effect_count =
                            result.applied_effect_count.saturating_add(targets.len());
                    }
                }
                SkillEffectDef::Heal { amount } => {
                    for target_id in targets {
                        control::charge(1, "skill_effect_target")?;
                        emit(BattleCommand::ApplyHeal {
                            target_id: *target_id,
                            flat: *amount,
                            percent: 0,
                            source_id: Some(caster_instance_id),
                        })?;
                    }
                    if !targets.is_empty() {
                        result.applied_effect_count =
                            result.applied_effect_count.saturating_add(targets.len());
                    }
                }
                SkillEffectDef::DamageExpr { .. }
                | SkillEffectDef::HealExpr { .. }
                | SkillEffectDef::ShieldExpr { .. } => {
                    unreachable!(
                        "authored value expressions cannot enter battle execution before WP02"
                    )
                }
                SkillEffectDef::ModifyResonance { amount } => {
                    for target_id in targets {
                        control::charge(1, "skill_effect_target")?;
                        emit(BattleCommand::ModifyResonance {
                            target_id: *target_id,
                            amount: *amount,
                            allow_autocast_when_full: *amount > 0,
                        })?;
                    }
                    if !targets.is_empty() {
                        result.applied_effect_count =
                            result.applied_effect_count.saturating_add(targets.len());
                    }
                }
                SkillEffectDef::ModifyTacticalResource { .. } => {}
                SkillEffectDef::Resources { operations } => {
                    control::charge_serialized(operations, "resource_command_copy")?;
                    control::charge(targets.len() as u64, "resource_command_targets")?;
                    emit(BattleCommand::Resources {
                        operations: operations.clone(),
                        targets: targets.to_vec(),
                    })?;
                    result.applied_effect_count = result.applied_effect_count.saturating_add(1);
                }
                SkillEffectDef::Pull { max_distance_tiles } => {
                    for target_id in targets {
                        control::charge(1, "pull_effect_target")?;
                        emit(BattleCommand::Pull {
                            caster_id: caster_instance_id,
                            target_id: *target_id,
                            max_distance_tiles: *max_distance_tiles,
                        })?;
                    }
                    result.applied_effect_count =
                        result.applied_effect_count.saturating_add(targets.len());
                }
                SkillEffectDef::HealthCost {
                    amount,
                    allow_lethal,
                } => {
                    emit(BattleCommand::PayHealthCost {
                        caster_id: caster_instance_id,
                        amount: *amount,
                        allow_lethal: *allow_lethal,
                    })?;
                    result.applied_effect_count = result.applied_effect_count.saturating_add(1);
                }
                SkillEffectDef::RechargeShield {
                    source_skill,
                    amount,
                } => {
                    control::charge_serialized(source_skill, "shield_recharge_command")?;
                    emit(BattleCommand::RechargeShield {
                        caster_id: caster_instance_id,
                        source_skill: source_skill.clone(),
                        amount: *amount,
                    })?;
                    result.applied_effect_count = result.applied_effect_count.saturating_add(1);
                }
                SkillEffectDef::Sustained { definition } => {
                    control::charge_serialized(definition, "sustained_command_copy")?;
                    for target_id in targets {
                        control::charge(1, "sustained_target")?;
                        emit(BattleCommand::BeginSustained {
                            caster_id: caster_instance_id,
                            target_id: *target_id,
                            definition: definition.clone(),
                        })?;
                    }
                    result.applied_effect_count =
                        result.applied_effect_count.saturating_add(targets.len());
                }
                SkillEffectDef::ModifyStats { modifier } => {
                    for target_id in targets {
                        control::charge(1, "skill_effect_target")?;
                        emit(BattleCommand::ApplyModifier {
                            target_id: *target_id,
                            modifier: *modifier,
                        })?;
                    }
                    if !targets.is_empty() {
                        result.applied_effect_count =
                            result.applied_effect_count.saturating_add(targets.len());
                    }
                }
                SkillEffectDef::ApplyBuff {
                    buff_id,
                    duration_ms,
                } => {
                    let buff_id = crate::game::battle::buffs::BuffId::from_name(buff_id);
                    for target_id in targets {
                        control::charge(1, "skill_effect_target")?;
                        emit(BattleCommand::ApplyBuff {
                            caster_id: caster_instance_id,
                            target_id: *target_id,
                            buff_id,
                            duration_ms: u64::from(*duration_ms),
                        })?;
                    }
                    if !targets.is_empty() {
                        result.applied_effect_count =
                            result.applied_effect_count.saturating_add(targets.len());
                    }
                }
                SkillEffectDef::InterruptCast => {
                    for target_id in targets {
                        control::charge(1, "skill_effect_target")?;
                        emit(BattleCommand::InterruptCast {
                            source_id: caster_instance_id,
                            target_id: *target_id,
                        })?;
                    }
                    if !targets.is_empty() {
                        result.applied_effect_count =
                            result.applied_effect_count.saturating_add(targets.len());
                    }
                }
                SkillEffectDef::ExtraAttack { count } => {
                    for _ in 0..(*count as usize) {
                        control::charge(1, "skill_effect_target")?;
                        emit(BattleCommand::ScheduleAttack {
                            attacker_id: caster_instance_id,
                            target_id: hinted_target_id,
                            time_ms: 0,
                        })?;
                    }
                    if hinted_target_id.is_some() && *count > 0 {
                        result.scheduled_attack_count = result
                            .scheduled_attack_count
                            .saturating_add(usize::from(*count));
                        result.applied_effect_count = result
                            .applied_effect_count
                            .saturating_add(usize::from(*count));
                    }
                }
                SkillEffectDef::ModifyDamage { .. } => {}
            }
        }

        Ok((commands, result))
    }

    fn build_ordered_skill_effect_commands(
        &mut self,
        caster_instance_id: UnitInstanceId,
        effect: &crate::game::ability::authoring::AuthoredSkillEffect,
        targets: &[UnitInstanceId],
        evaluated_amounts: Option<&[super::value_expr_runtime::RuntimeAmount]>,
        step_damage_modifiers: crate::game::battle::damage::DamageModifiers,
        committed_at_ms: u64,
        source_template: Option<&crate::game::battle::damage::DamageSourceSnapshot>,
    ) -> Result<(Vec<super::types::SkillCommand>, SkillStepResult), GameError> {
        let mut commands: Vec<super::types::SkillCommand> = Vec::new();
        // Target resolution belongs to the delivery, not each effect.
        let mut result = SkillStepResult::default();
        let hinted_target_id = if targets.len() == 1 {
            targets.first().copied()
        } else {
            None
        };
        let mut emit = |command| {
            control::admit(
                control::Resource::EffectCommands,
                commands.len(),
                1,
                "skill_effect_commands",
            )?;
            commands.push((command, effect.source().cloned()));
            Ok::<(), GameError>(())
        };
        match &effect.op {
            SkillEffectDef::PersonalShieldGrant { .. } => {
                return Err(GameError::InvalidStaticData(
                    "personal shield grant runtime binding is pending".into(),
                ));
            }
            SkillEffectDef::GuardCounter { .. } | SkillEffectDef::StatusStrike { .. } | SkillEffectDef::FusionPlacement { .. } | SkillEffectDef::PairedAreas { .. } | SkillEffectDef::PreparedVolley { .. } | SkillEffectDef::InputPlan { .. } | SkillEffectDef::RelationAction { .. } | SkillEffectDef::CounterChannel { .. } | SkillEffectDef::TileBurst { .. } | SkillEffectDef::PresenceBurst { .. } | SkillEffectDef::CreateObject { .. } => {
                return Err(GameError::InvalidStaticData(
                    "Owned mark burst and object creation must execute once outside the target command loop".into(),
                ))
            }
            SkillEffectDef::Damage {
                amount: _,
                damage_type,
            }
            | SkillEffectDef::DamageExpr { damage_type, .. } => {
                let literal = match (&effect.op, evaluated_amounts) {
                    (SkillEffectDef::Damage { amount, .. }, None) => Some(amount.unsigned_abs()),
                    _ => None,
                };
                if literal == Some(0) {
                    return Ok((commands, result));
                }
                for (hit_index, target_id) in targets.iter().enumerate() {
                    control::charge(1, "skill_effect_target")?;
                    let damage_amount = match (literal, evaluated_amounts) {
                        (Some(value), _) => value,
                        (None, Some(values)) => match values.get(hit_index) {
                            Some(super::value_expr_runtime::RuntimeAmount::Damage(value)) => *value,
                            _ => {
                                return Err(GameError::InvalidStaticData(
                                    "damage expression produced a missing or non-damage amount"
                                        .to_string(),
                                ));
                            }
                        },
                        (None, None) => {
                            return Err(GameError::InvalidStaticData(
                                "damage expression was not evaluated before command construction"
                                    .to_string(),
                            ));
                        }
                    };
                    if damage_amount == 0 {
                        continue;
                    }
                    let source_snapshot = if let Some(template) = source_template {
                        let mut snapshot = template.try_clone_fact_source()?;
                        snapshot.source = DamageSource::Ability;
                        snapshot.damage_type = *damage_type;
                        snapshot.base_damage = damage_amount;
                        snapshot.modifiers = step_damage_modifiers;
                        snapshot.minimum_damage = 0;
                        self.materialize_damage_source_snapshot_for_target_hit(
                            snapshot,
                            *target_id,
                            hit_index as u32,
                        )
                    } else if let Some(snapshot) = self.damage_source_snapshot_for_unit(
                        caster_instance_id,
                        *target_id,
                        DamageSource::Ability,
                        *damage_type,
                        damage_amount,
                        step_damage_modifiers,
                        0,
                        committed_at_ms,
                        false,
                    ) {
                        snapshot
                    } else {
                        continue;
                    };
                    emit(BattleCommand::ApplyDamage {
                        target_id: *target_id,
                        source_snapshot,
                    })?;
                }
                result.applied_effect_count =
                    result.applied_effect_count.saturating_add(commands.len());
            }
            SkillEffectDef::Heal { amount: _ } | SkillEffectDef::HealExpr { amount: _ } => {
                for (target_index, target_id) in targets.iter().enumerate() {
                    control::charge(1, "skill_effect_target")?;
                    let amount = match evaluated_amounts.and_then(|values| values.get(target_index))
                    {
                        Some(super::value_expr_runtime::RuntimeAmount::Heal(value)) => *value,
                        Some(_) => {
                            return Err(GameError::InvalidStaticData(
                                "heal evaluation produced a non-heal amount".to_string(),
                            ));
                        }
                        None => match &effect.op {
                            SkillEffectDef::Heal { amount } => *amount,
                            SkillEffectDef::HealExpr { .. } => {
                                return Err(GameError::InvalidStaticData(
                                    "heal expression was not evaluated before command construction"
                                        .to_string(),
                                ));
                            }
                            _ => unreachable!(),
                        },
                    };
                    emit(BattleCommand::ApplyHeal {
                        target_id: *target_id,
                        flat: amount,
                        percent: 0,
                        source_id: Some(caster_instance_id),
                    })?;
                }
                result.applied_effect_count =
                    result.applied_effect_count.saturating_add(targets.len());
            }
            SkillEffectDef::ModifyResonance { amount } => {
                for target_id in targets {
                    control::charge(1, "skill_effect_target")?;
                    emit(BattleCommand::ModifyResonance {
                        target_id: *target_id,
                        amount: *amount,
                        allow_autocast_when_full: *amount > 0,
                    })?;
                }
                result.applied_effect_count =
                    result.applied_effect_count.saturating_add(targets.len());
            }
            SkillEffectDef::ModifyTacticalResource { .. } => {}
            SkillEffectDef::Resources { operations } => {
                control::charge_serialized(operations, "resource_command_copy")?;
                control::charge(targets.len() as u64, "resource_command_targets")?;
                emit(BattleCommand::Resources {
                    operations: operations.clone(),
                    targets: targets.to_vec(),
                })?;
                result.applied_effect_count = result.applied_effect_count.saturating_add(1);
            }
            SkillEffectDef::ShieldExpr {
                duration_ms,
                priority,
                shared_radius_units,
                ..
            } => {
                for (index, target_id) in targets.iter().enumerate() {
                    control::charge(1, "shield_effect_target")?;
                    let Some(super::value_expr_runtime::RuntimeAmount::Shield(amount)) =
                        evaluated_amounts.and_then(|v| v.get(index))
                    else {
                        return Err(GameError::InvalidStaticData(
                            "shield expression requires an evaluated shield amount".into(),
                        ));
                    };
                    if *amount == 0 {
                        continue;
                    }
                    emit(BattleCommand::BeginSustained {
                        caster_id: caster_instance_id,
                        target_id: *target_id,
                        definition: crate::game::ability::sustained::SustainedEffectDef::Shield {
                            duration_ms: *duration_ms,
                            amount: *amount,
                            priority: *priority,
                            shared_radius_units: *shared_radius_units,
                        },
                    })?;
                }
                result.applied_effect_count =
                    result.applied_effect_count.saturating_add(commands.len());
            }
            SkillEffectDef::Pull { max_distance_tiles } => {
                for target_id in targets {
                    control::charge(1, "pull_effect_target")?;
                    emit(BattleCommand::Pull {
                        caster_id: caster_instance_id,
                        target_id: *target_id,
                        max_distance_tiles: *max_distance_tiles,
                    })?;
                }
                result.applied_effect_count =
                    result.applied_effect_count.saturating_add(targets.len());
            }
            SkillEffectDef::HealthCost {
                amount,
                allow_lethal,
            } => {
                emit(BattleCommand::PayHealthCost {
                    caster_id: caster_instance_id,
                    amount: *amount,
                    allow_lethal: *allow_lethal,
                })?;
                result.applied_effect_count = result.applied_effect_count.saturating_add(1);
            }
            SkillEffectDef::RechargeShield {
                source_skill,
                amount,
            } => {
                control::charge_serialized(source_skill, "shield_recharge_command")?;
                emit(BattleCommand::RechargeShield {
                    caster_id: caster_instance_id,
                    source_skill: source_skill.clone(),
                    amount: *amount,
                })?;
                result.applied_effect_count = result.applied_effect_count.saturating_add(1);
            }
            SkillEffectDef::Sustained { definition } => {
                control::charge_serialized(definition, "sustained_command_copy")?;
                for target_id in targets {
                    control::charge(1, "sustained_target")?;
                    emit(BattleCommand::BeginSustained {
                        caster_id: caster_instance_id,
                        target_id: *target_id,
                        definition: definition.clone(),
                    })?;
                }
                result.applied_effect_count =
                    result.applied_effect_count.saturating_add(targets.len());
            }
            SkillEffectDef::ModifyStats { modifier } => {
                for target_id in targets {
                    control::charge(1, "skill_effect_target")?;
                    emit(BattleCommand::ApplyModifier {
                        target_id: *target_id,
                        modifier: *modifier,
                    })?;
                }
                result.applied_effect_count =
                    result.applied_effect_count.saturating_add(targets.len());
            }
            SkillEffectDef::ApplyBuff {
                buff_id,
                duration_ms,
            } => {
                let buff_id = crate::game::battle::buffs::BuffId::from_name(buff_id);
                for target_id in targets {
                    control::charge(1, "skill_effect_target")?;
                    emit(BattleCommand::ApplyBuff {
                        caster_id: caster_instance_id,
                        target_id: *target_id,
                        buff_id,
                        duration_ms: u64::from(*duration_ms),
                    })?;
                }
                result.applied_effect_count =
                    result.applied_effect_count.saturating_add(targets.len());
            }
            SkillEffectDef::InterruptCast => {
                for target_id in targets {
                    control::charge(1, "skill_effect_target")?;
                    emit(BattleCommand::InterruptCast {
                        source_id: caster_instance_id,
                        target_id: *target_id,
                    })?;
                }
                result.applied_effect_count =
                    result.applied_effect_count.saturating_add(targets.len());
            }
            SkillEffectDef::ExtraAttack { count } => {
                for _ in 0..usize::from(*count) {
                    control::charge(1, "skill_effect_target")?;
                    emit(BattleCommand::ScheduleAttack {
                        attacker_id: caster_instance_id,
                        target_id: hinted_target_id,
                        time_ms: 0,
                    })?;
                }
                if hinted_target_id.is_some() && *count > 0 {
                    result.scheduled_attack_count = result
                        .scheduled_attack_count
                        .saturating_add(usize::from(*count));
                    result.applied_effect_count = result
                        .applied_effect_count
                        .saturating_add(usize::from(*count));
                }
            }
            SkillEffectDef::ModifyDamage { .. } => {}
        }
        Ok((commands, result))
    }

    fn record_ordered_skill_effect_signal(
        &mut self,
        caster_instance_id: UnitInstanceId,
        skill_id: &SkillId,
        effect: &crate::game::ability::authoring::AuthoredSkillEffect,
        targets: &[UnitInstanceId],
        time_ms: u64,
    ) -> Result<SkillStepResult, GameError> {
        let mut result = SkillStepResult::default();
        let SkillEffectDef::ModifyTacticalResource { amount } = &effect.op else {
            return Ok(result);
        };
        if targets.is_empty() || *amount == 0 {
            return Ok(result);
        }
        self.admit_authoring_runtime_growth(1, "skill_live_signal")?;
        let source = crate::game::battle::runtime_facts::RuntimeFactSource {
            actor: Some(caster_instance_id),
            authored: effect.source().cloned(),
            ..self.runtime_fact_source.clone()
        };
        let fact_source = self.reserve_finite_source(source)?;
        let retained_extent = self.runtime_facts.reserve_fact_storage(
            fact_source
                .retained_extent()
                .and_then(|value| value.checked_add(32)),
        )?;
        self.live_signals
            .push(BattleLiveSignal::TacticalResourceDelta {
                fact_source,
                time_ms,
                retained_extent,
                source_unit_id: caster_instance_id,
                skill_id: skill_id.clone(),
                amount: *amount,
            });
        result.applied_effect_count = 1;
        Ok(result)
    }

    pub(in crate::game::battle::core) fn execute_ordered_skill_effects(
        &mut self,
        caster_instance_id: UnitInstanceId,
        skill_id: &SkillId,
        step: &SkillStepDef,
        targets: &[UnitInstanceId],
        consume_time_ms: u64,
        committed_at_ms: u64,
        source_template: Option<&crate::game::battle::damage::DamageSourceSnapshot>,
        cast_origin: Option<
            std::sync::Arc<crate::game::battle::skill_source::SkillCastExecutionSource>,
        >,
    ) -> Result<SkillStepResult, GameError> {
        let step_damage_modifiers = super::skill_runtime::skill_step_damage_modifiers(step);
        let base_source = source_template
            .map(|template| template.fact_source.clone())
            .unwrap_or_else(|| self.runtime_fact_source.clone());
        self.ensure_value_expr_action_witness(skill_id, &base_source)?;
        let action_resource_ids = base_source
            .action
            .as_ref()
            .map(|action| self.runtime_facts.action_result_witness(action))
            .transpose()
            .map_err(|error| {
                GameError::InvalidStaticData(format!(
                    "action result occurrence is invalid: {error:?}"
                ))
            })?
            .flatten()
            .map(super::value_expr_runtime::ActionResultWitness::resource_ids);
        let owned_capture;
        let carrier = if let Some(capture) =
            source_template.and_then(|template| template.value_expr_capture.as_deref())
        {
            Some(capture)
        } else {
            owned_capture = self.build_value_expr_delivery_capture(
                step,
                &base_source,
                Some(targets),
                committed_at_ms,
                None,
            )?;
            owned_capture.as_deref()
        };
        let mut prefix = carrier
            .map(|capture| {
                super::value_expr_runtime::EffectResultPrefix::from_capture(
                    capture,
                    &base_source,
                    action_resource_ids
                        .clone()
                        .unwrap_or_else(|| capture.resource_ids()),
                )
            })
            .transpose()?;
        let prior_strike_scope = self.begin_strike_scope();
        let operation = (|| -> Result<SkillStepResult, GameError> {
            let mut aggregate = SkillStepResult {
                resolved_target_count: targets.len(),
                ..SkillStepResult::default()
            };
            for (effect_index, effect) in step.effects.iter().enumerate() {
                control::charge(1, "ordered_skill_effect")?;
                let selection = self.effective_effect_targets(
                    effect_index,
                    effect,
                    step,
                    &base_source,
                    consume_time_ms,
                )?;
                let targets = selection
                    .as_ref()
                    .map_or(targets, |selection| selection.targets.as_slice());
                let attachment_targets;
                let targets = if effect.attached_to.is_some() {
                    control::charge(targets.len() as u64, "attachment_target_filter")?;
                    crate::game::battle::runtime_control::admit_retained_growth(
                        0,
                        targets
                            .len()
                            .checked_mul(std::mem::size_of::<UnitInstanceId>()),
                        "attachment_targets",
                    )?;
                    attachment_targets = targets
                        .iter()
                        .copied()
                        .filter(|t| !self.strike_deflected(*t))
                        .collect::<Vec<_>>();
                    attachment_targets.as_slice()
                } else {
                    targets
                };
                let mut effect_source = base_source.clone();
                effect_source.authored = effect.source().cloned();
                let mut effect_damage_modifiers = step_damage_modifiers;
                if matches!(&effect.op, SkillEffectDef::Damage { .. } | SkillEffectDef::DamageExpr { .. })
                    && (matches!(&step.delivery, DeliveryDef::TileArea { .. }) || selection.is_some()) {
                    let side = source_template.map(|s| s.source_side)
                        .or_else(|| self.units.get(&caster_instance_id).map(|u| u.owner))
                        .ok_or(GameError::InvalidAction)?;
                    let percent = self.area_damage_percent(caster_instance_id, side,
                        &effect_source, targets, consume_time_ms)?;
                    effect_damage_modifiers.damage_amp_percent = effect_damage_modifiers.damage_amp_percent.saturating_add(percent);
                }
                if let SkillEffectDef::PersonalShieldGrant { definition } = &effect.op {
                    // Source-only amount is evaluated once for this release,
                    // before one creation program and the complete recipient set.
                    let capture = carrier
                        .and_then(|owner| owner.capture_for(effect_index, targets.first().copied()))
                        .ok_or_else(|| {
                            GameError::InvalidStaticData(
                                "personal grant has no source amount capture".into(),
                            )
                        })?;
                    let value = self
                        .evaluate_value_expr_capture_with_resolved(
                            capture,
                            targets.first().copied(),
                            consume_time_ms,
                            prefix.as_ref(),
                            effect_index,
                            None,
                        )
                        .map_err(GameError::ValueExpressionFault)?;
                    let super::value_expr_runtime::RuntimeAmount::Shield(amount) = value.amount
                    else {
                        return Err(GameError::InvalidStaticData(
                            "personal grant amount has wrong unit".into(),
                        ));
                    };
                    let candidate =
                        super::amount_observation::PendingAmountEvaluation::from_runtime(value)?;
                    let prior_projection = self.begin_effect_result_projection(
                        effect_source.clone(),
                        action_resource_ids.clone(),
                    )?;
                    let operation = self.begin_personal_shield_grant(
                        caster_instance_id,
                        definition,
                        amount,
                        &effect_source,
                        consume_time_ms,
                    );
                    let projection = self.finish_effect_result_projection(prior_projection)?;
                    let count = operation?;
                    if action_resource_ids.is_some() {
                        self.settle_skill_effect_result(
                            &base_source,
                            &effect.source().ok_or(GameError::InvalidAction)?.site.action,
                            &projection,
                            action_resource_ids
                                .clone()
                                .expect("selected result resources"),
                        )?;
                    }
                    if let Some(prefix) = &mut prefix {
                        prefix.settle(effect_index, projection)?;
                    }
                    self.commit_amount_evaluation(candidate)?;
                    let result = SkillStepResult {
                        applied_effect_count: count,
                        ..Default::default()
                    };
                    if let (Some(origin), Some(occurrence), Some(effect_source)) = (
                        cast_origin.as_ref(),
                        base_source.flow.as_ref(),
                        effect.source(),
                    ) {
                        if let Some(flow) = self
                            .active_skill_casts
                            .get_mut(&origin.cast_seq)
                            .and_then(|cast| cast.flow.as_mut())
                        {
                            let mut scoped = result.clone();
                            scoped.resolved_target_count = count;
                            flow.record_effect_result(
                                occurrence.occurrence,
                                &effect_source.site,
                                &scoped,
                            )?;
                        }
                    }
                    self.record_lab_site(&base_source,effect.source().map(|s|&s.site),crate::mod_validation::CoverageStage::Triggered)?;
                    self.record_lab_site(&base_source,effect.source().map(|s|&s.site),if count>0{crate::mod_validation::CoverageStage::ConsumedOrEffect}else{crate::mod_validation::CoverageStage::LegalNoop})?;
                    aggregate.merge(&result);
                    continue;
                }
                let mut pre_numeric_amounts = Vec::new();
                let (evaluated_amounts, amount_record_candidate) = if matches!(
                    effect.op,
                    SkillEffectDef::DamageExpr { .. }
                        | SkillEffectDef::HealExpr { .. }
                        | SkillEffectDef::ShieldExpr { .. }
                ) || (matches!(
                    effect.op,
                    SkillEffectDef::Damage { .. }
                ) && carrier
                    .is_some_and(|owner| owner.has_effect_capture(effect_index)))
                {
                    let capture_owner = carrier.ok_or_else(|| {
                        GameError::InvalidStaticData(
                            "value-expression effect has no release carrier".to_string(),
                        )
                    })?;
                    if let Some(first_target) = targets.first() {
                        let first_capture = capture_owner
                            .capture_for(effect_index, Some(*first_target))
                            .ok_or_else(|| {
                                GameError::InvalidStaticData(
                                    "value-expression effect has no capture for its resolved set"
                                        .to_string(),
                                )
                            })?;
                        let members = if first_capture.plan().moment()
                            == crate::game::value_expr::EvalMoment::Live
                        {
                            self.prepare_resolved_target_members(
                                first_capture.plan(),
                                Some(targets),
                            )
                            .map_err(GameError::ValueExpressionFault)?
                        } else {
                            None
                        };
                        let mut values = allocate_evaluated_amounts(targets.len())?;
                        let mut latest = None;
                        for target in targets {
                            control::charge(1, "value_expr_effect_target")?;
                            let capture = capture_owner
                                .capture_for(effect_index, Some(*target))
                                .ok_or_else(|| {
                                    GameError::InvalidStaticData(
                                        "value-expression carrier has no capture for the resolved target"
                                            .to_string(),
                                    )
                                })?;
                            let value = self
                                .evaluate_value_expr_capture_with_resolved(
                                    capture,
                                    Some(*target),
                                    consume_time_ms,
                                    prefix.as_ref(),
                                    effect_index,
                                    members.as_ref(),
                                )
                                .map_err(GameError::ValueExpressionFault)?;
                            if let Some(pre) = value.pre_numeric.clone() {
                                crate::game::battle::runtime_control::admit_retained_growth(
                                    0,
                                    Some(
                                        std::mem::size_of::<(
                                            UnitInstanceId,
                                            super::relic_wedge::PreNumericAmount,
                                        )>() + 64,
                                    ),
                                    "wedge_evaluated_target",
                                )?;
                                pre_numeric_amounts.push((*target, pre));
                            }
                            values.push(value.amount);
                            if let Some(mut candidate) =
                                super::amount_observation::PendingAmountEvaluation::from_runtime(
                                    value,
                                )?
                            {
                                candidate.set_evaluation_advance(values.len())?;
                                latest = Some(candidate);
                            }
                        }
                        (Some(values), latest)
                    } else {
                        if !capture_owner.has_effect_capture(effect_index) {
                            return Err(GameError::InvalidStaticData(
                                "value-expression effect has no release capture".to_string(),
                            ));
                        }
                        (Some(Vec::new()), None)
                    }
                } else if self.amount_observation_enabled()
                    && matches!(
                        effect.op,
                        SkillEffectDef::Damage { .. } | SkillEffectDef::Heal { .. }
                    )
                    && !targets.is_empty()
                {
                    let entry = effect.source().and_then(|source| {
                        self.amount_entry_for_source(&effect_source, &source.site)
                    });
                    if let Some(entry) = entry {
                        let mut values = allocate_evaluated_amounts(targets.len())?;
                        let mut latest = None;
                        for target in targets {
                            control::charge(1, "legacy_amount_observation_target")?;
                            let value = self
                                .evaluate_legacy_amount_for_observation(
                                    entry,
                                    &effect_source,
                                    *target,
                                    committed_at_ms,
                                    consume_time_ms,
                                    effect_index,
                                )
                                .map_err(GameError::ValueExpressionFault)?;
                            values.push(value.amount);
                            if let Some(mut candidate) =
                                super::amount_observation::PendingAmountEvaluation::from_runtime(
                                    value,
                                )?
                            {
                                candidate.set_evaluation_advance(values.len())?;
                                latest = Some(candidate);
                            }
                        }
                        (Some(values), latest)
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };
                if let Some(source) = carrier
                    .and_then(|owner| owner.capture_for(effect_index, targets.first().copied()))
                    .map(|capture| capture.source())
                    .filter(|source| source.relic_numeric().is_some())
                {
                    effect_source.extended_observation = source.extended_observation.clone();
                }
                let resource_ids = action_resource_ids.clone().or_else(|| {
                    prefix
                        .as_ref()
                        .map(super::value_expr_runtime::EffectResultPrefix::resource_ids)
                });
                let direct_source = if matches!(&effect.op, SkillEffectDef::GuardCounter{..} | SkillEffectDef::StatusStrike{..} | SkillEffectDef::FusionPlacement{..} | SkillEffectDef::PairedAreas{..} | SkillEffectDef::PreparedVolley{..} | SkillEffectDef::InputPlan{..} | SkillEffectDef::RelationAction { .. } | SkillEffectDef::CounterChannel { .. } | SkillEffectDef::TileBurst { .. } | SkillEffectDef::PresenceBurst { .. }) {
                    Some(crate::authoring::clone_shape::controlled_clone(&effect_source)?)
                } else { None };
                let prior_projection =
                    self.begin_effect_result_projection(effect_source, resource_ids)?;
                let effect_operation = (|| -> Result<SkillStepResult, GameError> {
                    if let Some(source) = direct_source {
                        let prior = std::mem::replace(&mut self.runtime_fact_source, source);
                        let result = match &effect.op {
                            SkillEffectDef::GuardCounter { program } => self.execute_guard_counter(caster_instance_id, targets, program, consume_time_ms),
                            SkillEffectDef::StatusStrike { program } => self.execute_status_strike(caster_instance_id,program,consume_time_ms),
                            SkillEffectDef::FusionPlacement { program } => self.install_fusion(caster_instance_id,program,consume_time_ms),
                            SkillEffectDef::PairedAreas { program } => self.install_paired_areas(caster_instance_id,program,consume_time_ms),
                            SkillEffectDef::PreparedVolley { .. } => self.release_prepared_volley(caster_instance_id,consume_time_ms),
                            SkillEffectDef::InputPlan { .. } => self.release_input_plan(caster_instance_id,consume_time_ms),
                            SkillEffectDef::RelationAction { .. } => self.release_relation_action(caster_instance_id, consume_time_ms),
                            SkillEffectDef::CounterChannel { .. } => self.release_counter_channel(caster_instance_id, consume_time_ms),
                            SkillEffectDef::TileBurst { .. } => self.release_tile_burst(caster_instance_id, consume_time_ms),
                            SkillEffectDef::PresenceBurst { program } => self.execute_presence_burst(caster_instance_id, program, consume_time_ms),
                            _ => unreachable!("direct source belongs to a typed operation"),
                        };
                        self.runtime_fact_source = prior;
                        return result;
                    }
                    if let SkillEffectDef::CreateObject { definition } = &effect.op {
                        let cast = cast_origin.as_deref().ok_or_else(|| {
                            GameError::InvalidStaticData(
                                "CreateObject has no admitted cast occurrence".into(),
                            )
                        })?;
                        return self.execute_object_creation_effect(
                            caster_instance_id,
                            &step.id,
                            effect,
                            definition,
                            cast,
                            &base_source,
                            consume_time_ms,
                        );
                    }
                    let (mut commands, mut result) = self.build_ordered_skill_effect_commands(
                        caster_instance_id,
                        effect,
                        targets,
                        evaluated_amounts.as_deref(),
                        effect_damage_modifiers,
                        committed_at_ms,
                        source_template,
                    )?;
                    if carrier
                        .and_then(|owner| owner.capture_for(effect_index, targets.first().copied()))
                        .is_some_and(|capture| capture.source().relic_numeric().is_some())
                    {
                        for (command, _) in &mut commands {
                            if let BattleCommand::ApplyDamage {
                                target_id,
                                source_snapshot,
                            } = command
                            {
                                if let Some(source) = carrier
                                    .and_then(|owner| {
                                        owner.capture_for(effect_index, Some(*target_id))
                                    })
                                    .map(|capture| capture.source())
                                    .filter(|source| source.relic_numeric().is_some())
                                {
                                    source_snapshot.fact_source.extended_observation =
                                        source.extended_observation.clone();
                                }
                            }
                        }
                    }
                    for (command, _) in &mut commands {
                        if let BattleCommand::ApplyDamage {
                            target_id,
                            source_snapshot,
                        } = command
                        {
                            if let Some((_, pre)) = pre_numeric_amounts
                                .iter()
                                .find(|(target, _)| target == target_id)
                            {
                                source_snapshot.fact_source = self
                                    .bind_pre_numeric(&source_snapshot.fact_source, pre.clone())?;
                            }
                        }
                    }
                    if !commands.is_empty() {
                        let summary = self.process_skill_commands_in_scope(
                            commands,
                            consume_time_ms,
                            caster_instance_id,
                            cast_origin.clone(),
                        );
                        control::poll("ordered_skill_effect_commands")?;
                        result.actual_hp_loss =
                            result.actual_hp_loss.saturating_add(summary.actual_hp_loss);
                        result.actual_damage_target_count = result
                            .actual_damage_target_count
                            .saturating_add(summary.actual_damage_target_count);
                        result.effective_hit_count = result.effective_hit_count.saturating_add(summary.effective_hit_count);
                        result.killed_target_count = result
                            .killed_target_count
                            .saturating_add(summary.killed_target_count);
                    }
                    result.merge(&self.record_ordered_skill_effect_signal(
                        caster_instance_id,
                        skill_id,
                        effect,
                        targets,
                        consume_time_ms,
                    )?);
                    control::poll("ordered_skill_effect_complete")?;
                    Ok(result)
                })();
                let projection_result = self.finish_effect_result_projection(prior_projection);
                let effect_result = match (effect_operation, projection_result) {
                    (Err(error), _) => return Err(error),
                    (Ok(_), Err(error)) => return Err(error),
                    (Ok(result), Ok(projection)) => {
                        if action_resource_ids.is_some() {
                            let site = effect.source().ok_or_else(|| {
                                GameError::InvalidStaticData(
                                    "completed action effect has no authored site".to_string(),
                                )
                            })?;
                            self.settle_skill_effect_result(
                                &base_source,
                                &site.site.action,
                                &projection,
                                action_resource_ids
                                    .clone()
                                    .expect("selected result resources"),
                            )?;
                        }
                        if let Some(prefix) = &mut prefix {
                            prefix.settle(effect_index, projection)?;
                        }
                        self.commit_amount_evaluation(amount_record_candidate)?;
                        result
                    }
                };
                if let (Some(origin), Some(occurrence), Some(effect_source)) = (
                    cast_origin.as_ref(),
                    base_source.flow.as_ref(),
                    effect.source(),
                ) {
                    if let Some(flow) = self
                        .active_skill_casts
                        .get_mut(&origin.cast_seq)
                        .and_then(|cast| cast.flow.as_mut())
                    {
                        let mut scoped = effect_result.clone();
                        scoped.resolved_target_count = targets.len();
                        flow.record_effect_result(
                            occurrence.occurrence,
                            &effect_source.site,
                            &scoped,
                        )?;
                    }
                }
                self.record_lab_site(&base_source,effect.source().map(|s|&s.site),crate::mod_validation::CoverageStage::Triggered)?;
                // Damage and healing facts own their terminal witness, including
                // expression amounts. A zero-damage command omitted by the
                // producer has no fact, so record that actual legal branch here.
                if !matches!(effect.op,
                    SkillEffectDef::Damage { .. } | SkillEffectDef::DamageExpr { .. }
                    | SkillEffectDef::Heal { .. } | SkillEffectDef::HealExpr { .. })
                    && effect_result.applied_effect_count > 0
                {
                    self.record_lab_site(&base_source,effect.source().map(|s|&s.site),crate::mod_validation::CoverageStage::ConsumedOrEffect)?;
                }else if matches!(effect.op, SkillEffectDef::Damage { .. } | SkillEffectDef::DamageExpr { .. })
                    && effect_result.applied_effect_count == 0
                    && effect_result.scheduled_attack_count == 0 && !targets.is_empty(){
                    self.record_lab_site(&base_source,effect.source().map(|s|&s.site),crate::mod_validation::CoverageStage::LegalNoop)?;
                }else if targets.is_empty(){
                    self.record_lab_site(&base_source,effect.source().map(|s|&s.site),crate::mod_validation::CoverageStage::Ineligible)?;
                }
                self.commit_effect_target_history(selection)?;
                aggregate.merge(&effect_result);
            }
            Ok(aggregate)
        })();
        self.end_strike_scope(prior_strike_scope);
        operation
    }

    pub(super) fn resolve_skill_step<'a>(
        skill: &'a SkillDef,
        step_index: usize,
        step_id: &str,
    ) -> Option<&'a SkillStepDef> {
        if skill.flow.is_some()
            || skill
                .steps
                .iter()
                .flat_map(|step| &step.effects)
                .any(|effect| {
                    effect
                        .source
                        .as_ref()
                        .is_some_and(|source| source.source_file == "effective:relic_structural")
                })
        {
            control::charge(skill.steps.len() as u64, "flow_authored_step_lookup").ok()?;
            return skill.steps.iter().find(|step| step.id == step_id);
        }
        let step = skill.steps.get(step_index)?;
        debug_assert_eq!(
            step.id, step_id,
            "skill step invariant broken: index {} resolved to '{}' but event carried '{}'",
            step_index, step.id, step_id
        );
        Some(step)
    }

    fn execute_skill_step(&mut self, execution: SkillStepExecution<'_>) -> Result<(), GameError> {
        let owns_delivery = !self.is_side_occurrence(execution.cast_seq, execution.step_index)
            && (matches!(&execution.step.delivery, DeliveryDef::TileArea {area,..} if matches!(area.mode,crate::game::ability::SkillTileAreaMode::ChannelGround { .. }))
                || execution
                .skill
                .use_lifetime
                .effects()
                .iter()
                .any(|reference| reference.step == execution.step.id)
                || (execution
                    .step
                    .effects
                    .iter()
                    .any(|effect| matches!(effect.op, SkillEffectDef::ExtraAttack { .. }))
                    && !matches!(&execution.step.delivery, DeliveryDef::TileArea { area, .. } if area.is_independent())));
        if owns_delivery {
            self.admit_authoring_runtime_growth(1, "use_effect_occurrence")?;
            if let Some(cast) = self.active_skill_casts.get_mut(&execution.cast_seq) {
                cast.step_progress
                    .entry(execution.step_index)
                    .or_insert_with(|| super::types::SkillStepProgress {
                        use_owned: true,
                        result: Default::default(),
                        started: false,
                        pending_delivery_count: 0,
                    })
                    .use_owned = true;
            }
        }
        let mut source = self
            .skill_flow_source(execution.cast_seq, execution.step_index)?
            .unwrap_or_else(|| crate::game::battle::runtime_facts::RuntimeFactSource {
                actor: Some(execution.caster_instance_id),
                ..Default::default()
            });
        if let Some(flow) = self.skill_flow_occurrence(execution.cast_seq, execution.step_index) {
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(std::mem::size_of_val(&flow) + 64),
                "flow_occurrence_source",
            )?;
            source.flow = Some(std::sync::Arc::new(flow));
        } else {
            source.flow = None;
        }
        let action = source.action.clone();
        let prior = std::mem::replace(&mut self.runtime_fact_source, source);
        let (cast_seq, step_index, time_ms) =
            (execution.cast_seq, execution.step_index, execution.time_ms);
        let result = if matches!(&execution.step.delivery, DeliveryDef::Instant)
            || matches!(&execution.step.delivery, DeliveryDef::TileArea { area, .. } if !area.is_independent())
        {
            self.with_instant_resource_delivery(|core| core.execute_skill_step_inner(execution))
        } else {
            self.execute_skill_step_inner(execution)
        };
        self.runtime_fact_source = prior;
        result?;
        control::poll("action_step_dispatch")?;
        let deferred = self
            .active_skill_casts
            .get(&cast_seq)
            .is_some_and(|cast| cast.deferred_steps.contains_key(&step_index));
        if !deferred {
            let legacy = self
                .active_skill_casts
                .get(&cast_seq)
                .and_then(|cast| cast.flow.as_ref())
                .is_none_or(|flow| flow.plan.legacy);
            if legacy {
                if let Some(action) = action {
                    self.runtime_facts
                        .dispatched(&action, step_index, time_ms)?;
                }
            }
            self.flow_step_dispatched(cast_seq, step_index, time_ms)?;
            self.cancel_uses_with_unavailable_targets(time_ms)?;
        }
        Ok(())
    }

    fn execute_skill_step_inner(
        &mut self,
        execution: SkillStepExecution<'_>,
    ) -> Result<(), GameError> {
        control::charge(1, "skill_step")?;
        let step_target = self.resolve_skill_step_context(
            execution.time_ms,
            execution.cast_seq,
            execution.step_index,
            execution.caster_instance_id,
            execution.skill,
            execution.step,
            execution.cast_target,
        );
        control::poll("skill_step_context_resolved")?;
        if step_target.is_none()
            && self
                .runtime_fact_source
                .causal()
                .and_then(|c| c.program.as_ref())
                .is_some()
        {
            let source = self.runtime_fact_source.clone();
            self.causal_state()?.cancel_participants(&source)?;
            self.finalize_skill_step(
                execution.cast_seq,
                execution.step_index,
                SkillStepResult::default(),
                execution.time_ms,
            );
            return Ok(());
        }
        match self.evaluate_step_condition(
            execution.cast_seq,
            execution.step_index,
            execution.caster_instance_id,
            step_target,
            &execution.step.when,
        ) {
            PreviousStepDamageGate::Satisfied => {}
            PreviousStepDamageGate::Unsatisfied => {
                self.finalize_skill_step(
                    execution.cast_seq,
                    execution.step_index,
                    SkillStepResult::default(),
                    execution.time_ms,
                );
                return Ok(());
            }
            PreviousStepDamageGate::Pending => {
                self.defer_skill_step(
                    execution.cast_seq,
                    execution.step_index,
                    super::types::DeferredSkillStep {
                        caster_instance_id: execution.caster_instance_id,
                        skill_id: execution.skill.id.clone(),
                        step_id: execution.step.id.clone(),
                        cast_target: execution.cast_target,
                        cause: execution.cause,
                    },
                );
                return Ok(());
            }
        }

        let repeat_count = self.evaluate_step_repeat_count(
            execution.caster_instance_id,
            step_target,
            &execution.step.repeat,
        );
        if repeat_count == 0 {
            self.finalize_skill_step(
                execution.cast_seq,
                execution.step_index,
                SkillStepResult::default(),
                execution.time_ms,
            );
            return Ok(());
        }

        let target_instance_id = match step_target {
            Some(SkillCastTarget::Unit { unit_instance_id }) => Some(unit_instance_id),
            _ => None,
        };
        let presentation =
            (!execution.step.presentation.is_empty()).then(|| execution.step.presentation.clone());

        let step_seq = self.with_recording_context(execution.cause, |core| {
            core.record_event_log(
                execution.time_ms,
                BattleLogEvent::AbilityStepTriggered {
                    skill_id: execution.skill.id.clone(),
                    step_id: execution.step.id.clone(),
                    caster_instance_id: execution.caster_instance_id,
                    target_instance_id,
                    presentation,
                },
            )
        });

        let cast_origin = self
            .active_skill_casts
            .get(&execution.cast_seq)
            .and_then(|cast| cast.execution_source.clone());

        self.with_recording_cause(step_seq, |core| -> Result<(), GameError> {
            match &execution.step.delivery {
                DeliveryDef::Instant => {
                    let mut aggregated_result = SkillStepResult::default();

                    for _ in 0..repeat_count {
                        control::charge(1, "skill_step_repeat")?;
                        let iteration_target = core.resolve_skill_step_context(
                            execution.time_ms,
                            execution.cast_seq,
                            execution.step_index,
                            execution.caster_instance_id,
                            execution.skill,
                            execution.step,
                            execution.cast_target,
                        );
                        let targets = core.resolve_skill_step_targets(
                            execution.cast_seq,
                            execution.caster_instance_id,
                            execution.step,
                            iteration_target,
                        );
                        let result = core.execute_ordered_skill_effects(
                            execution.caster_instance_id,
                            &execution.skill.id,
                            execution.step,
                            &targets,
                            execution.time_ms,
                            execution.time_ms,
                            None,
                            cast_origin.clone(),
                        )?;
                        aggregated_result.merge(&result);
                    }

                    core.finalize_skill_step(
                        execution.cast_seq,
                        execution.step_index,
                        aggregated_result,
                        execution.time_ms,
                    );
                    Ok(())
                }
                delivery @ DeliveryDef::Projectile { .. } => {
                    let launched_count = core.dispatch_skill_projectile_delivery(
                        execution.time_ms,
                        execution.cast_seq,
                        execution.step_index,
                        execution.caster_instance_id,
                        execution.skill,
                        execution.step,
                        execution.cast_target,
                        delivery,
                        repeat_count,
                    )?;
                    control::poll("skill_projectiles_dispatched")?;
                    if launched_count == 0 {
                        core.finalize_skill_step(
                            execution.cast_seq,
                            execution.step_index,
                            SkillStepResult::default(),
                            execution.time_ms,
                        );
                    } else {
                        core.begin_skill_step_delivery(
                            execution.cast_seq,
                            execution.step_index,
                            launched_count,
                        );
                    }
                    Ok(())
                }
                DeliveryDef::TileArea { footprint, area } => {
                    if !area.is_independent() {
                        let mut aggregated_result = SkillStepResult::default();

                        for _ in 0..repeat_count {
                            control::charge(1, "skill_step_repeat")?;
                            let iteration_target = core.resolve_skill_step_context(
                                execution.time_ms,
                                execution.cast_seq,
                                execution.step_index,
                                execution.caster_instance_id,
                                execution.skill,
                                execution.step,
                                execution.cast_target,
                            );
                            let Some((
                                origin,
                                impact_position,
                                direction_hint,
                                affected_tiles,
                                targets,
                            )) = core.resolve_instant_tile_area_targets(
                                execution.time_ms,
                                execution.cast_seq,
                                execution.step_index,
                                execution.caster_instance_id,
                                iteration_target,
                                footprint,
                                area,
                            )
                            else {
                                continue;
                            };
                            control::poll("instant_area_targets_resolved")?;
                            if core.skill_step_needs_hostile_usefulness_gate(
                                execution.caster_instance_id,
                                execution.step,
                                area.hit_targets,
                                &targets,
                            ) && !core.skill_step_has_useful_hostile_target(
                                execution.caster_instance_id,
                                execution.step,
                                &targets,
                            ) {
                                continue;
                            }

                            let delivery_id = core.allocate_skill_area_delivery_id(
                                execution.cast_seq,
                                execution.caster_instance_id,
                                execution.time_ms,
                            );
                            let area_declared_seq = core.record_skill_tile_area_declared(
                                execution.time_ms,
                                delivery_id,
                                execution.skill.id.clone(),
                                execution.step.id.clone(),
                                execution.caster_instance_id,
                                iteration_target,
                                origin,
                                impact_position,
                                direction_hint,
                                affected_tiles,
                                area.clone(),
                            );
                            core.update_skill_cast_impact_context(
                                execution.cast_seq,
                                execution.step_index,
                                super::types::SkillImpactContext {
                                    delivery_id,
                                    impact_time_ms: execution.time_ms,
                                    impact_position,
                                    direction_hint: Some(direction_hint),
                                    first_hit_unit_id: targets.first().copied(),
                                    hit_unit_ids: targets.clone(),
                                    spawned_area_id: None,
                                },
                            );

                            let result = core.with_recording_cause(area_declared_seq, |core| {
                                core.execute_ordered_skill_effects(
                                    execution.caster_instance_id,
                                    &execution.skill.id,
                                    execution.step,
                                    &targets,
                                    execution.time_ms,
                                    execution.time_ms,
                                    None,
                                    cast_origin.clone(),
                                )
                            })?;
                            core.apply_area_hit_reaction(execution.caster_instance_id,
                                area.on_hit_caster_heal_percent, &result, None, execution.time_ms)?;
                            aggregated_result.merge(&result);
                        }

                        core.finalize_skill_step(
                            execution.cast_seq,
                            execution.step_index,
                            aggregated_result,
                            execution.time_ms,
                        );
                        return Ok(());
                    }

                    let mut spawned_areas = 0usize;
                    for _ in 0..repeat_count {
                        control::charge(1, "skill_step_repeat")?;
                        let iteration_target = core.resolve_skill_step_context(
                            execution.time_ms,
                            execution.cast_seq,
                            execution.step_index,
                            execution.caster_instance_id,
                            execution.skill,
                            execution.step,
                            execution.cast_target,
                        );
                        let registered = core.register_persistent_tile_area(
                            execution.time_ms,
                            execution.cast_seq,
                            execution.step_index,
                            execution.caster_instance_id,
                            execution.skill.id.clone(),
                            execution.step.id.clone(),
                            execution.step,
                            iteration_target,
                            footprint.clone(),
                            area.clone(),
                        )?;
                        control::poll("skill_area_registration_result")?;
                        if registered {
                            spawned_areas = spawned_areas.saturating_add(1);
                        }
                    }
                    control::poll("skill_areas_registered")?;
                    if spawned_areas == 0 {
                        core.finalize_skill_step(
                            execution.cast_seq,
                            execution.step_index,
                            SkillStepResult::default(),
                            execution.time_ms,
                        );
                    } else {
                        core.begin_skill_step_delivery(
                            execution.cast_seq,
                            execution.step_index,
                            spawned_areas,
                        );
                    }
                    Ok(())
                }
            }
        })?;
        Ok(())
    }

    pub(super) fn process_event(
        &mut self,
        event: impl Into<super::queue::DispatchEvent>,
        current_time_ms: u64,
    ) -> Result<(), GameError> {
        self.process_event_with_pending_bucket(event, current_time_ms, &[])
    }

    fn process_event_with_pending_bucket(
        &mut self,
        event: impl Into<super::queue::DispatchEvent>,
        current_time_ms: u64,
        pending_events: &[super::queue::QueuedEvent],
    ) -> Result<(), GameError> {
        self.process_owned_event(event, current_time_ms, pending_events)
            .map_err(|error| {
                crate::game::battle::runtime_control::trusted_failure(error, "owned_event_dispatch")
            })
    }

    fn process_owned_event(
        &mut self,
        event: impl Into<super::queue::DispatchEvent>,
        current_time_ms: u64,
        pending_events: &[super::queue::QueuedEvent],
    ) -> Result<(), GameError> {
        control::charge(1, "battle_event_dispatch")?;
        self.reconcile_sustained(current_time_ms)?;
        self.settle_field_work_progress(current_time_ms);
        control::poll("field_work_settlement")?;
        let (event, source, fact_source, close_delivery_on_dispatch) = match event.into() {
            super::queue::DispatchEvent::Raw(event) => (event, None, Default::default(), false),
            super::queue::DispatchEvent::Queued(event) => (
                event.event.into_owned()?,
                event.source,
                event.fact_source.into_owned()?,
                event.close_delivery_on_dispatch,
            ),
        };
        let completion = if close_delivery_on_dispatch {
            fact_source.delivery.clone()
        } else {
            None
        };
        let resource_completion = if close_delivery_on_dispatch {
            fact_source.resource_delivery().cloned()
        } else {
            None
        };
        let prior_fact_source = std::mem::replace(&mut self.runtime_fact_source, fact_source);
        let result = self.process_event_inner(event, source, current_time_ms);
        self.runtime_fact_source = prior_fact_source;
        control::poll("battle_event_inner_complete")?;
        if result.is_ok() {
            self.close_source_resource_delivery(resource_completion.as_deref())?;
            if let Some(ticket) = completion {
                self.runtime_facts
                    .close_delivery(&ticket, current_time_ms)?;
            }
        }
        if result.is_ok() {
            self.settle_causal_runtime(current_time_ms, pending_events)?;
            self.reconcile_sustained(current_time_ms)?;
            self.settle_equipment_resonance_rewards(current_time_ms)?;
            self.reconcile_field_work(current_time_ms);
            self.reconcile_ready_wait(current_time_ms)?;
            control::poll("field_work_reconciliation")?;
        }
        result
    }

    fn resolve_skill_start_gate(
        &mut self,
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        cause: BattleEventCause,
    ) -> Result<(), GameError> {
        let Some(start_seq) = cast_start_seq_from_cause(cause) else {
            return Ok(());
        };
        let pending = self
            .units
            .get(&caster_instance_id)
            .and_then(|unit| unit.pending_skill_cast.as_ref())
            .filter(|pending| pending.start_seq == start_seq && pending.start_gate_pending)
            .cloned();
        let Some(pending) = pending else {
            return Ok(());
        };
        let employee = self
            .units
            .get(&caster_instance_id)
            .is_some_and(Self::is_employee_resonance_unit);
        let credential = pending.action.as_ref().is_some_and(|action| {
            self.has_resonance_use_credential(caster_instance_id, &pending.skill_id, action)
        });
        let target_valid = self
            .pending_skill_view(&pending, caster_instance_id)
            .is_some_and(|skill| {
                self.admitted_skill_target_is_valid(caster_instance_id, skill, pending.cast_target)
            });
        let start_blocked = if self.current_action_interrupt_protected(caster_instance_id) {
            false
        } else if employee {
            self.active_hard_cc_release_time_ms(caster_instance_id, time_ms)
                .is_some()
        } else {
            self.skill_start_disabled_reason(caster_instance_id, time_ms)
                .is_some()
        };
        if !target_valid
            || start_blocked
            || !self.units.get(&caster_instance_id).is_some_and(|unit| {
                unit.is_active()
                    && unit.skill_id.as_ref().is_some_and(|root| self.fragment_owns_skill(root, &pending.skill_id))
                    && (if employee {
                        credential
                    } else {
                        unit.resonance_current >= unit.resonance_max.max(1)
                    })
                    && unit.employee_trauma.is_none_or(|trauma| trauma < 100)
            })
        {
            self.cancel_resonance_use_control(caster_instance_id);
            self.cancel_boss_owned_work(time_ms, caster_instance_id, true, false);
            return Ok(());
        }
        if let Some(active) = self
            .units
            .get_mut(&caster_instance_id)
            .and_then(|unit| unit.pending_skill_cast.as_mut())
        {
            active.start_gate_pending = false;
        }
        if let Some(action) = pending.action.as_ref() {
            self.try_start_corrosion(caster_instance_id, &pending.skill_id, action, time_ms)?;
        }
        Ok(())
    }

    fn process_event_inner(
        &mut self,
        event: BattleEvent,
        event_source: Option<
            std::sync::Arc<crate::game::battle::skill_source::SkillEffectExecutionSource>,
        >,
        current_time_ms: u64,
    ) -> Result<(), GameError> {
        match event {
            BattleEvent::CausalInvoke { time_ms } => self.invoke_causal_program(time_ms),
            BattleEvent::CausalPeriodic { time_ms } => self.process_causal_periodic(time_ms),
            BattleEvent::ResonancePulse {
                time_ms,
                unit_instance_id,
                spawn_order,
            } => {
                self.handle_resonance_pulse(unit_instance_id, spawn_order, time_ms);
                Ok(())
            }
            BattleEvent::SpawnGroup {
                time_ms,
                group_id,
                cause,
            } => {
                let spawned_unit_ids = self.spawn_scenario_group(&group_id)?;
                self.record_spawned_units(time_ms, cause, &spawned_unit_ids)?;
                self.start_causal_periodics(Some(time_ms))?;

                for unit_id in spawned_unit_ids {
                    self.schedule_initial_attack_for_unit(unit_id, time_ms);
                }
                Ok(())
            }
            BattleEvent::AttackStart {
                time_ms,
                attacker_instance_id,
                target_instance_id,
                schedule_next,
                cause,
            } => {
                self.handle_basic_attack_start_event(
                    time_ms,
                    current_time_ms,
                    attacker_instance_id,
                    target_instance_id,
                    schedule_next,
                    cause,
                );

                Ok(())
            }
            BattleEvent::BasicHealResolve {
                time_ms,
                attacker_instance_id,
                target_instance_id,
                attack_epoch,
                kind,
                cause,
            } => {
                self.handle_basic_heal_resolve_event(
                    time_ms,
                    attacker_instance_id,
                    target_instance_id,
                    attack_epoch,
                    kind,
                    cause,
                );
                Ok(())
            }
            BattleEvent::AttackResolve {
                time_ms,
                attacker_instance_id,
                target_instance_id,
                attack_epoch,
                source_snapshot,
                kind,
                delivery,
                cause,
            } => {
                self.handle_basic_attack_resolve_event(
                    time_ms,
                    attacker_instance_id,
                    target_instance_id,
                    attack_epoch,
                    source_snapshot,
                    kind,
                    delivery,
                    cause,
                );

                Ok(())
            }
            BattleEvent::BasicAttackTileImpact { time_ms, release_id, source_snapshot, tile, air_capable, cause } => {
                self.with_recording_context(cause, |core| core.impact_basic_delayed_tile(release_id, source_snapshot, tile, air_capable, time_ms))
            }
            BattleEvent::BasicAttackProjectileAdvance {
                time_ms,
                projectile_id,
                cause,
            } => {
                self.with_recording_context(cause, |core| {
                    core.advance_basic_attack_projectile(time_ms, projectile_id);
                });

                Ok(())
            }
            BattleEvent::SkillProjectileImpact {
                time_ms,
                delivery_id,
                cast_seq,
                step_index,
                skill_id,
                step_id,
                caster_instance_id,
                source_snapshot,
                impact_position,
                first_hit_unit_id,
                impact_vfx_id,
                terminal,
                cause,
            } => self.with_recording_context(cause, |core| {
                core.apply_skill_projectile_impact(
                    time_ms,
                    delivery_id,
                    cast_seq,
                    step_index,
                    skill_id,
                    step_id,
                    caster_instance_id,
                    source_snapshot,
                    impact_position,
                    first_hit_unit_id,
                    impact_vfx_id,
                    terminal,
                )
            }),
            BattleEvent::SkillProjectileAdvance {
                time_ms,
                delivery_id,
                ..
            } => self.advance_skill_projectile(time_ms, delivery_id),
            BattleEvent::SkillAreaTick {
                time_ms,
                area_id,
                cause,
                ..
            } => self
                .with_recording_context(cause, |core| core.apply_skill_area_tick(time_ms, area_id)),
            BattleEvent::SkillAreaExpire { area_id, cause, .. } => {
                self.with_recording_context(cause, |core| core.expire_skill_area(area_id))
            }
            BattleEvent::SkillStartGate { time_ms, caster_instance_id, cause } => self.resolve_skill_start_gate(time_ms,caster_instance_id,cause),
            BattleEvent::AutoCastStart {
                time_ms,
                caster_instance_id,
                cause,
            } => {
                if self.channel_active(caster_instance_id)
                    || !self.boss_runtime_allows_combat_action(caster_instance_id)
                {
                    if let Some(unit) = self.units.get_mut(&caster_instance_id) {
                        unit.pending_cast = false;
                        unit.pending_cast_cause = None;
                    }
                    return Ok(());
                }
                if self.unit_is_in_defense_stance(caster_instance_id) {
                    if let Some(unit) = self.units.get_mut(&caster_instance_id) {
                        unit.pending_cast = true;
                        unit.pending_cast_cause.get_or_insert(cause);
                    }
                    return Ok(());
                }
                let Some(caster) = self.units.get(&caster_instance_id) else {
                    return Ok(());
                };
                if !caster.is_active() {
                    return Ok(());
                }
                let control_state = self
                    .unit_control_state(caster_instance_id, time_ms)
                    .unwrap_or_default();
                if control_state.silenced {
                    if let Some(unit) = self.units.get_mut(&caster_instance_id) {
                        unit.pending_cast = true;
                        unit.pending_cast_cause.get_or_insert(cause);
                    }
                    return Ok(());
                }
                let blocked_until = caster.next_action_time.max(
                    control_state
                        .hard_cc_release_time_ms
                        .map(|expires_at_ms| expires_at_ms.saturating_add(1))
                        .unwrap_or(0),
                );

                // CC 등 행동이 막힌 상태라면 이벤트 연기
                if time_ms < blocked_until {
                    if let Some(unit) = self.units.get_mut(&caster_instance_id) {
                        unit.pending_cast = true;
                        unit.pending_cast_cause.get_or_insert(cause);
                    }
                    self.event_queue.push(BattleEvent::AutoCastStart {
                        time_ms: blocked_until,
                        caster_instance_id,
                        cause,
                    });
                    return Ok(());
                }

                let (caster_owner, skill_id, caster_base_uuid) = {
                    let Some(caster) = self.units.get(&caster_instance_id) else {
                        return Ok(());
                    };
                    (caster.owner, caster.skill_id.clone(), caster.base_uuid)
                };

                let skill_id = skill_id
                    .or_else(|| {
                        self.game_data
                            .abnormality_data
                            .get_by_uuid(&caster_base_uuid)
                            .and_then(|meta| meta.skill_id.clone())
                    })
                    .filter(|id| self.game_data.skill_data.get_by_id(id).is_some());

                if skill_id.as_deref().unwrap_or("").is_empty() {
                    // TODO: 기록
                    return Ok(());
                }

                let caster_pos = match self.live_unit_projected_tile(caster_instance_id) {
                    Some(pos) => pos,
                    None => return Ok(()),
                };

                let skill = match skill_id
                    .as_deref()
                    .map(|id| {
                        self.cloned_current_skill(
                            &SkillId::from(id),
                            caster_instance_id,
                            &self.runtime_fact_source,
                        )
                    })
                    .transpose()?
                    .flatten()
                {
                    Some(skill) => skill,
                    None => return Ok(()),
                };

                if skill.resonance_admission.is_some() {
                    // This portable target requires an explicit employee request.
                    return Ok(());
                }

                // A new selected-recipient program requires accepted player
                // input. This automatic path cannot manufacture that choice.
                if self.requires_explicit_structural_selection(
                    &skill,
                    caster_instance_id,
                    &self.runtime_fact_source,
                )? {
                    return Ok(());
                }

                let target = self.resolve_skill_cast_target_at(
                    time_ms,
                    &skill,
                    caster_instance_id,
                    caster_owner,
                    caster_pos,
                );
                if !self.approach_taunt_skill_allowed(caster_instance_id, &skill, target) || !self.skill_has_useful_automatic_cast_target(
                    time_ms,
                    &skill,
                    caster_instance_id,
                    target,
                ) {
                    if let Some(caster) = self.units.get_mut(&caster_instance_id) {
                        caster.pending_cast = true;
                        caster.pending_cast_cause.get_or_insert(cause);
                    }
                    return Ok(());
                }

                self.reconcile_ready_wait(time_ms)?;
                let cast_damage = self.consume_equipment_cast_damage(caster_instance_id, time_ms)?;
                let action = self.runtime_facts.accept(
                    caster_instance_id,
                    Some(&skill.id),
                    crate::game::battle::runtime_facts::RuntimeFactSource {
                        extended_observation: crate::game::battle::runtime_facts::RuntimeFactSource::extended_observation(self.independent_action_origin(caster_instance_id)?, None)?,
                        actor: Some(caster_instance_id),
                        fragment: self
                            .units
                            .get(&caster_instance_id)
                            .and_then(|unit| unit.fragment_source.as_ref())
                            .filter(|source| self.fragment_owns_skill(&source.skill_id, &skill.id))
                            .cloned(),
                        ..Default::default()
                    }.with_cast_damage(cast_damage)?,
                    time_ms,
                    false,
                )?;

                let employee_use = self.admit_employee_resonance_use(
                    caster_instance_id,
                    &skill,
                    &action,
                    time_ms,
                )?;
                if self.try_start_corrosion(caster_instance_id, &skill.id, &action, time_ms)? {
                    return Ok(());
                }

                // "캐스트/집중"이 진행되는 동안 새로운 AutoCastStart를 막기 위한 캐스트 락.
                // (공격/이동은 ActionLocks로 개별 게이트)
                self.prepare_presence_burst(caster_instance_id, &skill, &action, time_ms)?;
                self.prepare_tile_burst(caster_instance_id, &skill, &action, time_ms)?;
                self.activate_ready_wait(caster_instance_id, &skill.id, time_ms)?;
                let cast_end_ms = if (Self::fusion_placement_skill(&skill) || Self::status_strike_declaration(&skill).is_some()) || Self::paired_area_skill(&skill) || Self::presence_burst_skill(&skill) || Self::tile_burst_skill(&skill) || Self::counter_channel_skill(&skill) || Self::relation_action_skill(&skill) { time_ms } else if skill.focus_time_ms == 0 {
                    time_ms.saturating_add(1)
                } else {
                    time_ms.saturating_add(skill.focus_time_ms as u64)
                };
                if let Some(caster) = self.units.get_mut(&caster_instance_id) {
                    caster.next_action_time = cast_end_ms;

                    // 집중/시전 중 공명 획득은 항상 금지
                    if !employee_use {
                        caster.action_locks.lock_resonance_gain_until(cast_end_ms);
                    }

                    if !skill.focus_permissions.allows_basic_attack {
                        caster.action_locks.lock_basic_attack_until(cast_end_ms);
                    }

                    if !skill.focus_permissions.allows_move {
                        caster.action_locks.lock_movement_until(cast_end_ms);
                    }
                }

                let start_seq = self.with_recording_context(cause, |core| {
                    core.record_event_log(
                        time_ms,
                        BattleLogEvent::AutoCastStart {
                            skill_id,
                            caster_instance_id,
                            target,
                        },
                    )
                });
                let accepted_target_tile = match target {
                    Some(SkillCastTarget::Unit { unit_instance_id }) => {
                        self.live_unit_projected_tile(unit_instance_id)
                    }
                    Some(SkillCastTarget::Tile { position }) => Some(position),
                    None | Some(SkillCastTarget::TilePair { .. } | SkillCastTarget::Direction { .. }) => None,
                };

                if let Some(caster) = self.units.get_mut(&caster_instance_id) {
                    caster.pending_skill_cast = Some(PendingSkillCast {
                        start_gate_pending: false,
                        reset_resonance: !employee_use,
                        action: Some(action),
                        skill_id: skill.id.clone(),
                        cast_target: target,
                        accepted_target_tile,
                        accepted_facing: caster.facing_direction,
                        start_seq,
                    });
                }

                self.event_queue.push(BattleEvent::AutoCastEnd {
                    time_ms: cast_end_ms,
                    caster_instance_id,
                    cause: BattleEventCause::Parent { seq: start_seq },
                });

                Ok(())
            }
            BattleEvent::RestTick {
                time_ms,
                unit_instance_id,
                rest_epoch,
            } => {
                self.handle_rest_tick(time_ms, unit_instance_id, rest_epoch);
                Ok(())
            }
            BattleEvent::IndependentObjectExpiry {
                time_ms,
                object_id,
                epoch,
            } => self.expire_independent_object(object_id, epoch, time_ms),
            BattleEvent::ForcedDefenseEnd {
                time_ms,
                unit_instance_id,
                epoch,
            } => {
                self.handle_forced_defense_end(time_ms, unit_instance_id, epoch);
                Ok(())
            }
            BattleEvent::FieldWorkCompletion {
                time_ms,
                object_id,
                generation,
            } => {
                self.handle_field_work_completion(time_ms, object_id, generation);
                Ok(())
            }
            BattleEvent::AutoCastEnd {
                time_ms,
                caster_instance_id,
                cause,
            } => {
                let Some(expected_start_seq) = cast_start_seq_from_cause(cause) else {
                    return Ok(());
                };
                let has_matching_pending = self
                    .units
                    .get(&caster_instance_id)
                    .and_then(|unit| unit.pending_skill_cast.as_ref())
                    .is_some_and(|pending| pending.start_seq == expected_start_seq);
                if !has_matching_pending {
                    return Ok(());
                }

                let blocked_until = self
                    .units
                    .get(&caster_instance_id)
                    .map(|unit| unit.next_action_time)
                    .unwrap_or(0)
                    .max(
                        if self.current_action_interrupt_protected(caster_instance_id) {0} else {self.active_hard_cc_release_time_ms(caster_instance_id, time_ms)
                            .unwrap_or(0)},
                    );
                if time_ms < blocked_until {
                    self.event_queue.push(BattleEvent::AutoCastEnd {
                        time_ms: blocked_until,
                        caster_instance_id,
                        cause,
                    });
                    return Ok(());
                }

                let target_valid = self
                    .units
                    .get(&caster_instance_id)
                    .and_then(|unit| unit.pending_skill_cast.as_ref())
                    .and_then(|pending| {
                        self.pending_skill_view(pending, caster_instance_id)
                            .map(|skill| {
                                self.admitted_skill_target_is_valid(
                                    caster_instance_id,
                                    skill,
                                    pending.cast_target,
                                )
                            })
                    })
                    .unwrap_or(false);
                if !target_valid
                    && self
                        .units
                        .get(&caster_instance_id)
                        .is_some_and(Self::is_employee_resonance_unit)
                {
                    self.cancel_resonance_use_control(caster_instance_id);
                    self.cancel_boss_owned_work(time_ms, caster_instance_id, true, false);
                    return Ok(());
                }

                let preserve_basic_clock = self.units.get(&caster_instance_id)
                    .and_then(|u|u.pending_skill_cast.as_ref())
                    .and_then(|p|self.pending_skill_view(p,caster_instance_id))
                    .is_some_and(Self::skill_preserves_basic_clock);
                let pending = self.units.get_mut(&caster_instance_id).and_then(|unit| {
                    if unit
                        .pending_skill_cast
                        .as_ref()
                        .is_some_and(|pending| pending.start_seq == expected_start_seq)
                    {
                        unit.pending_skill_cast.take()
                    } else {
                        None
                    }
                });
                let reset_resonance = pending
                    .as_ref()
                    .is_some_and(|pending| pending.reset_resonance);
                let finishing_action = pending.as_ref().and_then(|pending| pending.action.clone());
                let completion_ticket = match &finishing_action {
                    Some(action) => self.runtime_facts.delivery(action)?,
                    None => None,
                };
                self.runtime_fact_source = crate::game::battle::runtime_facts::RuntimeFactSource {
                    actor: Some(caster_instance_id),
                    action: finishing_action,
                    ..Default::default()
                };
                self.with_recording_context(cause, |core| {
                    if let Some(pending) = pending {
                        core.invoke_ability_with_accepted_target_tile(
                            time_ms,
                            caster_instance_id,
                            &pending.skill_id,
                            (pending.cast_target, pending.accepted_target_tile, pending.accepted_facing),
                            cause,
                            false,
                            pending.action,
                            None,
                        );
                        core.schedule_pending_autocasts(time_ms);
                    }

                    core.record_event_log(
                        time_ms,
                        BattleLogEvent::AutoCastEnd { caster_instance_id },
                    );
                });

                let Some(caster) = self.units.get_mut(&caster_instance_id) else {
                    return Ok(());
                };
                let recovery_ends_at =
                    time_ms.saturating_add(caster.stats.attack_interval_ms.max(1));
                if !preserve_basic_clock {
                    caster.next_basic_attack_ms = caster.next_basic_attack_ms.max(recovery_ends_at);
                }
                let resonance_before_reset = caster.resonance_current;
                if reset_resonance {
                    caster.resonance_current = 0;
                    caster.action_locks.lock_resonance_gain_until(
                        time_ms.saturating_add(caster.resonance_lock_ms),
                    );
                }
                if caster.next_action_time <= time_ms {
                    caster.next_action_time = 0;
                }
                caster.pending_cast = false;

                if reset_resonance {
                    self.record_resource_fact(
                        crate::game::battle::runtime_facts::ResourceOwner::Unit(caster_instance_id),
                        crate::game::battle::runtime_facts::FactResource::Resonance,
                        Some(crate::game::battle::runtime_facts::CostReason::CastReset),
                        self.runtime_fact_source.clone(),
                        -i64::from(resonance_before_reset),
                        resonance_before_reset,
                        0,
                        time_ms,
                    )?;
                }
                if let Some(ticket) = completion_ticket {
                    self.runtime_facts.close_delivery(&ticket, time_ms)?;
                }

                Ok(())
            }
            BattleEvent::ManualCastEnd {
                time_ms,
                caster_instance_id,
                cause,
            } => {
                // An immediate action can finish in the same time bucket as
                // its admission. Its start gate still precedes release.
                self.resolve_skill_start_gate(time_ms,caster_instance_id,cause)?;
                let Some(expected_start_seq) = cast_start_seq_from_cause(cause) else {
                    return Ok(());
                };
                let has_matching_pending = self
                    .units
                    .get(&caster_instance_id)
                    .and_then(|unit| unit.pending_skill_cast.as_ref())
                    .is_some_and(|pending| pending.start_seq == expected_start_seq);
                if !has_matching_pending {
                    return Ok(());
                }

                let blocked_until = self
                    .units
                    .get(&caster_instance_id)
                    .map(|unit| unit.next_action_time)
                    .unwrap_or(0)
                    .max(
                        if self.current_action_interrupt_protected(caster_instance_id) {0} else {self.active_hard_cc_release_time_ms(caster_instance_id, time_ms)
                            .unwrap_or(0)},
                    );
                if time_ms < blocked_until {
                    self.event_queue.push(BattleEvent::ManualCastEnd {
                        time_ms: blocked_until,
                        caster_instance_id,
                        cause,
                    });
                    return Ok(());
                }

                let target_valid = self
                    .units
                    .get(&caster_instance_id)
                    .and_then(|unit| unit.pending_skill_cast.as_ref())
                    .and_then(|pending| {
                        self.pending_skill_view(pending, caster_instance_id)
                            .map(|skill| {
                                self.admitted_skill_target_is_valid(
                                    caster_instance_id,
                                    skill,
                                    pending.cast_target,
                                )
                            })
                    })
                    .unwrap_or(false);
                if !target_valid
                    && self
                        .units
                        .get(&caster_instance_id)
                        .is_some_and(Self::is_employee_resonance_unit)
                {
                    self.cancel_resonance_use_control(caster_instance_id);
                    self.cancel_boss_owned_work(time_ms, caster_instance_id, true, false);
                    return Ok(());
                }

                let preserve_basic_clock = self.units.get(&caster_instance_id)
                    .and_then(|u|u.pending_skill_cast.as_ref())
                    .and_then(|p|self.pending_skill_view(p,caster_instance_id))
                    .is_some_and(Self::skill_preserves_basic_clock);
                let pending = self.units.get_mut(&caster_instance_id).and_then(|unit| {
                    if unit
                        .pending_skill_cast
                        .as_ref()
                        .is_some_and(|pending| pending.start_seq == expected_start_seq)
                    {
                        unit.pending_skill_cast.take()
                    } else {
                        None
                    }
                });
                let reset_resonance = pending
                    .as_ref()
                    .is_some_and(|pending| pending.reset_resonance);
                let finishing_action = pending.as_ref().and_then(|pending| pending.action.clone());
                let completion_ticket = match &finishing_action {
                    Some(action) => self.runtime_facts.delivery(action)?,
                    None => None,
                };
                self.runtime_fact_source = crate::game::battle::runtime_facts::RuntimeFactSource {
                    actor: Some(caster_instance_id),
                    action: finishing_action,
                    ..Default::default()
                };
                self.with_recording_context(cause, |core| {
                    if let Some(pending) = pending {
                        core.invoke_ability_with_accepted_target_tile(
                            time_ms,
                            caster_instance_id,
                            &pending.skill_id,
                            (pending.cast_target, pending.accepted_target_tile, pending.accepted_facing),
                            cause,
                            false,
                            pending.action,
                            None,
                        );
                    }

                    core.record_event_log(
                        time_ms,
                        BattleLogEvent::ManualCastEnd { caster_instance_id },
                    );
                });

                let Some(caster) = self.units.get_mut(&caster_instance_id) else {
                    return Ok(());
                };
                let recovery_ends_at =
                    time_ms.saturating_add(caster.stats.attack_interval_ms.max(1));
                if !preserve_basic_clock {
                    caster.next_basic_attack_ms = caster.next_basic_attack_ms.max(recovery_ends_at);
                }
                let resonance_before_reset = caster.resonance_current;
                if reset_resonance {
                    caster.resonance_current = 0;
                }
                if reset_resonance {
                    caster.action_locks.lock_resonance_gain_until(
                        time_ms.saturating_add(caster.resonance_lock_ms),
                    );
                }
                if caster.next_action_time <= time_ms {
                    caster.next_action_time = 0;
                }
                caster.pending_cast = false;
                caster.pending_cast_cause = None;

                if reset_resonance {
                    self.record_resource_fact(
                        crate::game::battle::runtime_facts::ResourceOwner::Unit(caster_instance_id),
                        crate::game::battle::runtime_facts::FactResource::Resonance,
                        Some(crate::game::battle::runtime_facts::CostReason::CastReset),
                        self.runtime_fact_source.clone(),
                        -i64::from(resonance_before_reset),
                        resonance_before_reset,
                        0,
                        time_ms,
                    )?;
                }
                if let Some(ticket) = completion_ticket {
                    self.runtime_facts.close_delivery(&ticket, time_ms)?;
                }

                Ok(())
            }
            BattleEvent::TileBurstWave { time_ms, id } => self.tile_burst_wave(id,time_ms),
            BattleEvent::ShotStockTick{source,time_ms}=>self.shot_stock_tick(source,time_ms),
            BattleEvent::HostileCohortTick{source,time_ms}=>self.hostile_cohort_tick(source,time_ms),
            BattleEvent::StatusMobileTick{time_ms}=>self.status_mobile_tick(time_ms),
            BattleEvent::MobileEffectsTick{time_ms}=>self.mobile_effects_tick(time_ms),
            BattleEvent::PairedAreaTick{time_ms,id}=>self.paired_area_tick(id,time_ms),
            BattleEvent::PreparedVolleyTick{source,generation,time_ms}=>self.drive_prepared_volley(source,generation,time_ms),
            BattleEvent::VolleyFlightTick{time_ms}=>self.volley_flight_tick(time_ms),
            BattleEvent::ShotFlightTick{time_ms}=>self.shot_flight_tick(time_ms),
            BattleEvent::RelationGain { source, generation, time_ms } => self.relation_gain(source, generation, time_ms),
            BattleEvent::RelationActionTick { source, generation, time_ms } => self.drive_relation_action(source, generation, time_ms),
            BattleEvent::CounterGain { source, generation, time_ms } => self.counter_gain(source, generation, time_ms),
            BattleEvent::CounterChannelTick { source, generation, time_ms } => self.counter_channel_tick(source, generation, time_ms),
            BattleEvent::PresenceTick { source, host, generation, time_ms } => self.presence_tick(source, host, generation, time_ms),
            BattleEvent::SustainedExpiry { activation, .. } => {
                self.expire_sustained(activation, current_time_ms)
            }
            BattleEvent::EquipmentStateTick { activation, time_ms } => self.equipment_state_tick(activation, time_ms),
            BattleEvent::EquipmentDeathBurst { id, time_ms } => self.equipment_death_burst_tick(id, time_ms),
            BattleEvent::DeferredDamageTick { debt, time_ms } => self.deferred_damage_tick(debt, time_ms),
            BattleEvent::MagazineReload { equipment, time_ms } => self.finish_equipment_reload(equipment, time_ms),
            BattleEvent::ChannelUpkeep { activation, time_ms } => self.channel_upkeep(activation, time_ms),
            BattleEvent::SkillFlowWake {
                time_ms,
                cast_seq,
                generation,
            } => self.drive_skill_flow(cast_seq, time_ms, Some(generation)),
            BattleEvent::SkillStep {
                time_ms,
                cast_seq,
                step_index,
                caster_instance_id,
                skill_id,
                step_id,
                cast_target,
                cause,
            } => {
                if !self
                    .active_skill_casts
                    .get(&cast_seq)
                    .is_some_and(|cast| !cast.control_cancelled)
                {
                    return Ok(());
                }
                let source = self
                    .skill_flow_source(cast_seq, step_index)?
                    .ok_or(GameError::InvalidAction)?;
                let Some(skill) = self.cloned_skill_for_source(&skill_id, &source)? else {
                    return Ok(());
                };
                let Some(step) = Self::resolve_skill_step(&skill, step_index, &step_id) else {
                    return Ok(());
                };
                // This is the release boundary. Previously released carriers bypass
                // it and retain their original target and lifetime.
                if !self.approach_taunt_skill_allowed(caster_instance_id, &skill, cast_target) {
                    self.cancel_resonance_use_control(caster_instance_id);
                    self.cancel_hostile_structured_flows(caster_instance_id, time_ms, crate::game::battle::runtime_facts::ActionCancelReason::Interrupted);
                    return Ok(());
                }
                // A preleased structural child resolves its own step target;
                // the original use's accepted target does not govern it.
                // Original steps retain their target-lifecycle cancellation.
                if !self.is_side_occurrence(cast_seq, step_index)
                    && self
                        .units
                        .get(&caster_instance_id)
                        .and_then(|unit| unit.resonance_use.as_ref())
                        .is_some_and(|use_state| {
                            self.active_skill_casts.get(&cast_seq).is_some_and(|cast| {
                                cast.fact_source.action.as_ref() == Some(&use_state.action)
                            })
                        })
                    && !self.admitted_skill_target_is_valid(caster_instance_id, &skill, cast_target)
                {
                    self.cancel_resonance_use_control(caster_instance_id);
                    self.cancel_structured_flows(
                        caster_instance_id,
                        time_ms,
                        crate::game::battle::runtime_facts::ActionCancelReason::Interrupted,
                    );
                    return Ok(());
                }

                self.execute_skill_step(SkillStepExecution {
                    time_ms,
                    cast_seq,
                    step_index,
                    caster_instance_id,
                    skill: &skill,
                    step,
                    cast_target,
                    cause,
                })?;
                self.schedule_pending_autocasts(time_ms);
                Ok(())
            }
            BattleEvent::ApplyBuff {
                time_ms,
                caster_instance_id,
                target_instance_id,
                buff_id,
                duration_ms,
                cause,
            } => {
                let Some(def) = self
                    .game_data
                    .buff_data
                    .get(buff_id)
                    .map(crate::authoring::clone_shape::controlled_clone)
                    .transpose()?
                else {
                    return Ok(());
                };

                if duration_ms == 0 {
                    return Ok(());
                }
                if def.kind == crate::game::battle::buffs::BuffKind::Stun && self.units.get(&target_instance_id).is_some_and(|u|u.target_traits.contains(&crate::game::battle::types::UnitTargetTrait::StunImmune)) { return Ok(()); }
                if def.kind.control_capability().is_some()
                    && self.sustained_invulnerable(target_instance_id, time_ms)? {
                    return Ok(());
                }

                if !matches!(
                    self.units.get(&target_instance_id),
                    Some(unit) if unit.is_active()
                ) {
                    return Ok(());
                }

                let duration_ms = if let Some(kind) = super::equipment_support::buff_control_kind(def.kind) {
                    self.received_control_duration(caster_instance_id, target_instance_id, None, kind, duration_ms, time_ms)?
                } else { duration_ms };

                let key = BuffInstanceKey {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                };

                let expires_at_ms = time_ms.saturating_add(duration_ms);
                let max_stacks = def.max_stacks.max(1);
                let is_hard_cc = matches!(
                    def.kind.control_capability(),
                    Some(crate::game::battle::buffs::BuffControlCapability::HardCrowdControl)
                );

                let before = self
                    .buffs
                    .get(&key)
                    .map(super::authoring_observation::BuffApplicationState::from);
                if before.is_none() {
                    self.admit_authoring_runtime_growth(1, "buff_apply")?;
                }
                let reservation =
                    self.reserve_authoring_observation_with_source(event_source.as_ref())?;

                let applied_seq = self.with_recording_context(cause, |core| {
                    core.record_event_log(
                        time_ms,
                        BattleLogEvent::BuffApplied {
                            caster_instance_id,
                            target_instance_id,
                            buff_id,
                            duration_ms,
                        },
                    )
                });

                self.runtime_fact_source
                    .admit_owner("buff_application_source")?;
                let entry = self.buffs.entry(key).or_insert(ActiveBuff {
                    latest_application_source: self.runtime_fact_source.observation(),
                    stacks: 0,
                    expires_at_ms,
                    next_tick_ms: None,
                });

                entry.latest_application_source = self.runtime_fact_source.observation();
                match def.reapply_policy {
                    crate::game::battle::buffs::BuffReapplyPolicy::StackRefreshDurationKeepCadence => {
                        entry.expires_at_ms = entry.expires_at_ms.max(expires_at_ms);
                        entry.stacks = entry.stacks.saturating_add(1).min(max_stacks);
                    }
                    crate::game::battle::buffs::BuffReapplyPolicy::RefreshDuration => {
                        entry.expires_at_ms = expires_at_ms;
                        entry.stacks = max_stacks.min(1);
                    }
                }

                let effective_expires_at_ms = entry.expires_at_ms;

                if def.tick_interval_ms > 0 && entry.next_tick_ms.is_none() {
                    let tick_time_ms = time_ms.saturating_add(def.tick_interval_ms);
                    if tick_time_ms < entry.expires_at_ms {
                        entry.next_tick_ms = Some(tick_time_ms);
                        self.event_queue.push_with_context(
                            BattleEvent::BuffTick {
                                time_ms: tick_time_ms,
                                caster_instance_id,
                                target_instance_id,
                                buff_id,
                                cause: BattleEventCause::Parent { seq: applied_seq },
                            },
                            None,
                            self.runtime_fact_source.observation(),
                        );
                    }
                }

                let after = super::authoring_observation::BuffApplicationState::from(&*entry);
                self.commit_authoring_observation(
                    reservation,
                    time_ms,
                    Some(caster_instance_id),
                    target_instance_id,
                    super::authoring_observation::AppliedChange::Buff {
                        buff_id,
                        kind: def.kind,
                        before,
                        after,
                    },
                );

                self.event_queue.push(BattleEvent::BuffExpire {
                    time_ms: effective_expires_at_ms,
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                    cause: BattleEventCause::Parent { seq: applied_seq },
                });

                if is_hard_cc {
                    let lock_until = effective_expires_at_ms.saturating_add(1);
                    self.cancel_unreleased_actions_for_hard_cc(
                        time_ms,
                        caster_instance_id,
                        target_instance_id,
                    );

                    self.refresh_block_state();
                    self.schedule_continuous_movement_tick(lock_until);
                }

                Ok(())
            }
            BattleEvent::BuffTick {
                time_ms,
                caster_instance_id,
                target_instance_id,
                buff_id,
                cause,
            } => {
                let Some(def) = self
                    .game_data
                    .buff_data
                    .get(buff_id)
                    .map(crate::authoring::clone_shape::controlled_clone)
                    .transpose()?
                else {
                    return Ok(());
                };

                let key = BuffInstanceKey {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                };

                let Some(active) = self.buffs.get(&key) else {
                    return Ok(());
                };

                if Some(time_ms) != active.next_tick_ms {
                    return Ok(());
                }

                let (stacks, expires_at_ms) = (active.stacks, active.expires_at_ms);

                let tick_seq = self.with_recording_context(cause, |core| {
                    core.record_event_log(
                        time_ms,
                        BattleLogEvent::BuffTick {
                            caster_instance_id,
                            target_instance_id,
                            buff_id,
                        },
                    )
                });

                if let crate::game::battle::buffs::BuffKind::PeriodicDamage {
                    damage_per_tick,
                    damage_type,
                } = def.kind
                {
                    let stacks = stacks.max(1) as i32;
                    let dmg = (damage_per_tick as i32).saturating_mul(stacks);
                    if dmg > 0 {
                        self.with_recording_cause(tick_seq, |core| {
                            let Some(source_snapshot) = core.damage_source_snapshot_for_unit(
                                caster_instance_id,
                                target_instance_id,
                                DamageSource::BuffTick,
                                damage_type,
                                dmg as u32,
                                Default::default(),
                                0,
                                time_ms,
                                false,
                            ) else {
                                return;
                            };
                            core.process_commands(
                                vec![BattleCommand::ApplyDamage {
                                    target_id: target_instance_id,
                                    source_snapshot,
                                }],
                                time_ms,
                            );
                            core.schedule_pending_autocasts(time_ms);
                        });
                    }
                }

                let next_tick_ms = time_ms.saturating_add(def.tick_interval_ms);
                if let Some(active) = self.buffs.get_mut(&key) {
                    if next_tick_ms < expires_at_ms {
                        active.next_tick_ms = Some(next_tick_ms);
                        self.event_queue.push_with_context(
                            BattleEvent::BuffTick {
                                time_ms: next_tick_ms,
                                caster_instance_id,
                                target_instance_id,
                                buff_id,
                                cause,
                            },
                            None,
                            self.runtime_fact_source.observation(),
                        );
                    } else {
                        active.next_tick_ms = None;
                    }
                }

                Ok(())
            }
            BattleEvent::BuffExpire {
                time_ms,
                caster_instance_id,
                target_instance_id,
                buff_id,
                cause,
            } => {
                let Some(def) = self
                    .game_data
                    .buff_data
                    .get(buff_id)
                    .map(crate::authoring::clone_shape::controlled_clone)
                    .transpose()?
                else {
                    return Ok(());
                };

                let key = BuffInstanceKey {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                };

                let Some(active) = self.buffs.get(&key) else {
                    return Ok(());
                };

                if time_ms < active.expires_at_ms {
                    return Ok(());
                }

                self.with_recording_context(cause, |core| {
                    core.record_event_log(
                        time_ms,
                        BattleLogEvent::BuffExpired {
                            caster_instance_id,
                            target_instance_id,
                            buff_id,
                            reason: BuffExpireReason::Natural,
                        },
                    )
                });

                self.buffs.remove(&key);
                if def.kind == crate::game::battle::buffs::BuffKind::Stun {
                    self.dispatch_equipment_stun_end(caster_instance_id, target_instance_id, time_ms)?;
                }

                if def.kind.control_capability().is_some() {
                    if matches!(
                        def.kind.control_capability(),
                        Some(crate::game::battle::buffs::BuffControlCapability::HardCrowdControl)
                    ) {
                        self.refresh_block_state();
                    }
                    self.schedule_pending_autocasts(time_ms.saturating_add(1));
                }
                Ok(())
            }
            BattleEvent::ContinuousMovementTick { time_ms } => {
                let movement_tick_ms = self.movement_tick_ms();
                if self
                    .last_continuous_movement_tick_ms
                    .is_some_and(|last_time_ms| {
                        time_ms < last_time_ms.saturating_add(movement_tick_ms)
                    })
                {
                    return Ok(());
                }

                self.last_continuous_movement_tick_ms = Some(time_ms);
                self.run_continuous_attack_movement_tick(time_ms, movement_tick_ms);
                self.settle_cohort_escapes(time_ms)?;
                self.try_start_pending_basic_attacks(time_ms);
                let next_time_ms = time_ms.saturating_add(movement_tick_ms);
                if next_time_ms > time_ms {
                    self.schedule_continuous_movement_tick(next_time_ms);
                }
                Ok(())
            }
            BattleEvent::BossRuntimeWake { .. }
            | BattleEvent::BossSummonWake { .. }
            | BattleEvent::CorrosionBoundary { .. }
            | BattleEvent::WaveTriggerWake { .. } => Ok(()),
        }
    }
}

#[cfg(test)]
impl BattleExecutionState {
    pub(crate) fn u03_exhaust_simulation_time_fixture(&mut self) {
        self.last_event_time_ms = u64::MAX;
    }
}
