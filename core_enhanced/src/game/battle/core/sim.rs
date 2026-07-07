use uuid::Uuid;

use crate::game::battle::event_log::BattleEventLogEntry;
use crate::{
    game::resources::Position,
    game::{
        ability::{
            DeliveryDef, SkillActivationMode, SkillDef, SkillEffectDef, SkillId,
            SkillStepCondition, SkillStepDef, SkillStepRepeat, SkillTarget, SkillUnitReference,
            StepTargetingMode,
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
            scenario::{ScenarioAction, ScenarioTrigger, WinCondition},
            types::{BattleResult, BattleUnitDraft, BattleWinner, ParticipantBattleResult},
        },
        behavior::{GameError, LiveBattleSkillReadinessDto},
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
    movement::ActionState,
    skill_runtime::cast::PreviousStepDamageGate,
    types::{AbilityProcKey, PendingSkillCast, SkillStepResult},
    ActiveBuff, BattleCore, BuffInstanceKey,
};

#[derive(Debug, Clone)]
pub struct BattleExecutionState {
    last_event_time_ms: u64,
    battle_start_hooks_ran: bool,
    finished: bool,
    finalized: bool,
}

impl BattleExecutionState {
    pub fn last_event_time_ms(&self) -> u64 {
        self.last_event_time_ms
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn is_finalized(&self) -> bool {
        self.finalized
    }

    pub fn next_time_after(&self, delta_ms: u64) -> u64 {
        self.last_event_time_ms.saturating_add(delta_ms)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BattleFinishSignal {
    pub time_ms: u64,
    pub winner: BattleWinner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleStepOutcome {
    Running,
    Finished(BattleFinishSignal),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleDeployCurrentHpPolicy {
    FixedCurrentHp(u32),
    PercentOfMax(u32),
}

impl BattleDeployCurrentHpPolicy {
    pub const WITHDRAW_REDEPLOY_RECOVERY_PERCENT: u32 = 30;
    pub const DEATH_REDEPLOY_PERCENT: u32 = 60;

    pub fn withdraw_redeploy(current_hp: u32, max_hp: u32) -> Self {
        let recovered_hp = current_hp
            .saturating_add(max_hp.saturating_mul(Self::WITHDRAW_REDEPLOY_RECOVERY_PERCENT) / 100);
        BattleDeployCurrentHpPolicy::FixedCurrentHp(recovered_hp.min(max_hp))
    }

    pub fn death_redeploy() -> Self {
        BattleDeployCurrentHpPolicy::PercentOfMax(Self::DEATH_REDEPLOY_PERCENT)
    }

    pub fn current_hp_for_max(self, max_hp: u32) -> u32 {
        if max_hp == 0 {
            return 0;
        }
        let current_hp = match self {
            BattleDeployCurrentHpPolicy::FixedCurrentHp(current_hp) => current_hp,
            BattleDeployCurrentHpPolicy::PercentOfMax(percent) => {
                max_hp.saturating_mul(percent) / 100
            }
        };
        current_hp.max(1).min(max_hp)
    }
}

#[derive(Debug, Clone)]
pub enum BattleLiveCommand {
    DeployPlayerUnit {
        draft: BattleUnitDraft,
        position: Position,
        facing: crate::game::battle::tile_range::FacingDirection,
        instance_salt: u32,
        time_ms: u64,
        current_hp_policy: Option<BattleDeployCurrentHpPolicy>,
    },
    WithdrawUnit {
        unit_id: UnitInstanceId,
        time_ms: u64,
    },
    ActivateSkill {
        unit_id: UnitInstanceId,
        skill_id: SkillId,
        target: Option<SkillCastTarget>,
        time_ms: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleLiveCommandOutcome {
    UnitDeployed { unit_id: UnitInstanceId },
    UnitWithdrawn { unit_id: UnitInstanceId },
    SkillActivated { unit_id: UnitInstanceId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleLiveSignal {
    StabilizationDelta {
        source_unit_id: UnitInstanceId,
        skill_id: SkillId,
        amount: i32,
    },
}

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
    fn movement_tick_ms(&self) -> u64 {
        self.game_data
            .run_policy
            .battle_runtime
            .movement_tick_ms
            .max(1)
    }

    fn max_battle_time_ms(&self) -> u64 {
        self.game_data
            .run_policy
            .battle_runtime
            .max_battle_time_ms
            .max(1)
    }

    fn has_pending_attack_resolve_at(&self, time_ms: u64) -> bool {
        self.event_queue.iter().any(|event| {
            matches!(
                event,
                BattleEvent::AttackResolve {
                    time_ms: event_time_ms,
                    ..
                } if *event_time_ms == time_ms
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
            CooldownSource::Item { item_instance_id }
            | CooldownSource::Artifact {
                artifact_instance_id: item_instance_id,
            } => Self::proc_uuid_tag(item_instance_id),
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
        let Some(skill) = self.game_data.skill_data.get_by_id(skill_id).cloned() else {
            return;
        };

        let Some((caster_owner, caster_pos)) =
            self.resolve_cast_origin_context(None, caster_instance_id, allow_dead_caster)
        else {
            return;
        };
        let cast_target = explicit_target.or_else(|| {
            self.resolve_skill_cast_target(&skill, caster_instance_id, caster_owner, caster_pos)
        });

        let target_instance_id = match cast_target {
            Some(SkillCastTarget::Unit { unit_instance_id }) => Some(unit_instance_id),
            _ => None,
        };
        let cast_target_anchor_position = match cast_target {
            Some(SkillCastTarget::Tile { position }) => Some(position),
            Some(SkillCastTarget::Unit { unit_instance_id }) => {
                self.live_unit_projected_tile(unit_instance_id)
            }
            None => None,
        };
        let cast_target_anchor_world_position = match cast_target {
            Some(SkillCastTarget::Tile { position }) => Some(
                crate::game::battle::core::movement::types::WorldVec2::from_tile_center(position),
            ),
            Some(SkillCastTarget::Unit { unit_instance_id }) => {
                self.unit_world_position_or_tile_center(unit_instance_id)
            }
            None => None,
        };

        let ability_seq = self.with_recording_context(cause, |core| {
            core.record_event_log(
                time_ms,
                BattleLogEvent::AbilityCast {
                    skill_id: skill.id.clone(),
                    caster_instance_id,
                    target_instance_id,
                },
            )
        });

        self.active_skill_casts.insert(
            ability_seq,
            super::types::ActiveSkillCast {
                caster_instance_id,
                skill_id: skill.id.clone(),
                caster_owner,
                anchor_position: caster_pos,
                cast_target_anchor_position,
                cast_target_anchor_world_position,
                allow_dead_caster,
                total_steps: skill.steps.len(),
                step_progress: Default::default(),
                deferred_steps: Default::default(),
                impact_contexts_by_step: Default::default(),
                last_impact_context: None,
                active_area_ids: Vec::new(),
            },
        );

        self.with_recording_cause(ability_seq, |core| {
            for (step_index, step) in skill.steps.iter().enumerate() {
                core.event_queue.push(BattleEvent::SkillStep {
                    time_ms: time_ms.saturating_add(step.delay_ms as u64),
                    cast_seq: ability_seq,
                    step_index,
                    caster_instance_id,
                    skill_id: skill.id.clone(),
                    step_id: step.id.clone(),
                    cast_target,
                    cause: BattleEventCause::Parent { seq: ability_seq },
                });
            }
        });
    }

    fn explicit_skill_cast_target_is_valid(
        &self,
        skill: &SkillDef,
        caster_instance_id: UnitInstanceId,
        caster_owner: Side,
        _caster_pos: Position,
        target: SkillCastTarget,
    ) -> bool {
        let Some((target_def, range_policy, defense_tile_range, air_capable)) =
            skill.cast_target_definition()
        else {
            return false;
        };

        match (target_def, target) {
            (SkillTarget::SelfUnit, SkillCastTarget::Unit { unit_instance_id }) => {
                unit_instance_id == caster_instance_id
            }
            (SkillTarget::EnemySingle { .. }, SkillCastTarget::Unit { unit_instance_id }) => {
                if !self.is_alive_enemy(unit_instance_id, caster_owner) {
                    return false;
                }
                if !self.single_target_can_target_unit(unit_instance_id, air_capable) {
                    return false;
                }
                self.is_target_in_tile_range_policy(
                    caster_instance_id,
                    unit_instance_id,
                    range_policy,
                    defense_tile_range,
                )
            }
            (SkillTarget::CastTarget, SkillCastTarget::Tile { position }) => {
                self.battlefield.is_valid_tile(position)
            }
            _ => false,
        }
    }

    pub(super) fn start_manual_skill_cast(
        &mut self,
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        requested_skill_id: SkillId,
        explicit_target: Option<SkillCastTarget>,
    ) -> Result<(), GameError> {
        let (skill, cast_target) = self.resolve_manual_skill_cast_request(
            time_ms,
            caster_instance_id,
            &requested_skill_id,
            explicit_target,
        )?;

        let cast_end_ms = if skill.focus_time_ms == 0 {
            time_ms.saturating_add(1)
        } else {
            time_ms.saturating_add(skill.focus_time_ms as u64)
        };
        if let Some(caster) = self.units.get_mut(&caster_instance_id) {
            caster.next_action_time = cast_end_ms;
            caster.action_locks.lock_resonance_gain_until(cast_end_ms);
            if !skill.focus_permissions.allows_basic_attack {
                caster.action_locks.lock_basic_attack_until(cast_end_ms);
            }
            if !skill.focus_permissions.allows_move {
                caster.action_locks.lock_movement_until(cast_end_ms);
            }
        }

        if !skill.focus_permissions.allows_move {
            let interrupted = self.interrupt_movement(
                time_ms,
                caster_instance_id,
                crate::game::battle::event_log::MovementStopReason::CastStarted,
                Some(cast_end_ms),
                ActionState::Idle,
            );
            if interrupted {
                self.schedule_continuous_movement_tick(cast_end_ms);
            }
        }

        let start_seq = self.record_event_log(
            time_ms,
            BattleLogEvent::ManualCastStart {
                skill_id: skill.id.clone(),
                caster_instance_id,
                target: cast_target.clone(),
            },
        );
        if let Some(caster) = self.units.get_mut(&caster_instance_id) {
            caster.pending_skill_cast = Some(PendingSkillCast {
                skill_id: skill.id,
                cast_target,
                start_seq,
            });
        }
        self.event_queue.push(BattleEvent::ManualCastEnd {
            time_ms: cast_end_ms,
            caster_instance_id,
            cause: BattleEventCause::Parent { seq: start_seq },
        });

        Ok(())
    }

    pub fn validate_manual_skill_activation(
        &self,
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        requested_skill_id: &SkillId,
        explicit_target: Option<SkillCastTarget>,
    ) -> Result<(), GameError> {
        self.resolve_manual_skill_cast_request(
            time_ms,
            caster_instance_id,
            requested_skill_id,
            explicit_target,
        )
        .map(|_| ())
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
        let mut target_required = false;
        let mut target_available = false;
        let mut target_block_reason = None;

        let reason = if unit.owner != Side::Player {
            Some("not_player_unit")
        } else if !unit.is_active() || !unit.is_combatant() {
            Some("unit_unavailable")
        } else if skill_id.is_none() {
            Some("no_skill")
        } else if activation_mode != SkillActivationMode::Manual {
            Some("activation_mode_not_manual")
        } else if self
            .active_hard_cc_release_time_ms(unit_id, time_ms)
            .is_some()
        {
            Some("hard_cc")
        } else if self.has_buff_kind(unit_id, crate::game::battle::buffs::BuffKind::Silence) {
            Some("silenced")
        } else if time_ms < unit.next_action_time {
            Some("action_locked")
        } else if unit.resonance_current < resonance_max {
            Some("resonance_not_full")
        } else if let Some(skill_id) = &skill_id {
            let Some(skill) = self.game_data.skill_data.get_by_id(skill_id.as_str()) else {
                return Some(LiveBattleSkillReadinessDto {
                    skill_id: Some(skill_id.clone()),
                    activation_mode,
                    resonance_current: unit.resonance_current.min(resonance_max),
                    resonance_max,
                    manual_activation_allowed: false,
                    target_required: false,
                    target_available: false,
                    can_activate_reason: Some("invalid_static_data".to_string()),
                    target_block_reason: None,
                });
            };
            let Some(caster_pos) = self.live_unit_projected_tile(unit_id) else {
                return Some(LiveBattleSkillReadinessDto {
                    skill_id: Some(skill_id.clone()),
                    activation_mode,
                    resonance_current: unit.resonance_current.min(resonance_max),
                    resonance_max,
                    manual_activation_allowed: false,
                    target_required: false,
                    target_available: false,
                    can_activate_reason: Some("unit_unavailable".to_string()),
                    target_block_reason: None,
                });
            };

            match skill.cast_target_definition() {
                Some((SkillTarget::SelfUnit, _, _, _)) => {
                    target_required = false;
                    target_available = true;
                }
                Some(_) => {
                    target_required = true;
                    target_available = self
                        .resolve_skill_cast_target(skill, unit_id, unit.owner, caster_pos)
                        .is_some();
                    if !target_available {
                        target_block_reason = Some("no_valid_target".to_string());
                    }
                }
                None => {
                    target_required = false;
                    target_available = false;
                    target_block_reason = Some("invalid_static_data".to_string());
                }
            }
            None
        } else {
            Some("no_skill")
        };

        Some(LiveBattleSkillReadinessDto {
            skill_id,
            activation_mode,
            resonance_current: unit.resonance_current.min(resonance_max),
            resonance_max,
            manual_activation_allowed: reason.is_none(),
            target_required,
            target_available,
            can_activate_reason: reason.map(str::to_string),
            target_block_reason,
        })
    }

    fn resolve_manual_skill_cast_request(
        &self,
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        requested_skill_id: &SkillId,
        explicit_target: Option<SkillCastTarget>,
    ) -> Result<(SkillDef, Option<SkillCastTarget>), GameError> {
        let Some(caster) = self.units.get(&caster_instance_id) else {
            return Err(GameError::UnitNotFound);
        };
        if caster.owner != Side::Player
            || caster.skill_activation_mode != SkillActivationMode::Manual
            || !caster.is_combatant()
            || !caster.is_active()
        {
            return Err(GameError::InvalidAction);
        }
        if self
            .active_hard_cc_release_time_ms(caster_instance_id, time_ms)
            .is_some()
        {
            return Err(GameError::InvalidAction);
        }
        if self.has_buff_kind(
            caster_instance_id,
            crate::game::battle::buffs::BuffKind::Silence,
        ) {
            return Err(GameError::InvalidAction);
        }
        if time_ms < caster.next_action_time {
            return Err(GameError::InvalidAction);
        }

        let Some(unit_skill_id) = caster.skill_id.clone() else {
            return Err(GameError::InvalidAction);
        };
        if &unit_skill_id != requested_skill_id {
            return Err(GameError::InvalidAction);
        }
        let max = caster.resonance_max.max(1);
        if caster.resonance_current < max {
            return Err(GameError::InsufficientResources);
        }
        let caster_owner = caster.owner;
        let caster_pos = self
            .live_unit_projected_tile(caster_instance_id)
            .ok_or(GameError::InvalidAction)?;

        let skill = self
            .game_data
            .skill_data
            .get_by_id(requested_skill_id.as_str())
            .cloned()
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "manual skill '{}' is missing from skill database",
                    requested_skill_id
                ))
            })?;

        let cast_target = match explicit_target {
            Some(target)
                if self.explicit_skill_cast_target_is_valid(
                    &skill,
                    caster_instance_id,
                    caster_owner,
                    caster_pos,
                    target,
                ) =>
            {
                Some(target)
            }
            Some(_) => return Err(GameError::InvalidAction),
            None => {
                self.resolve_skill_cast_target(&skill, caster_instance_id, caster_owner, caster_pos)
            }
        };
        if cast_target.is_none()
            && !matches!(
                skill.cast_target_definition(),
                Some((SkillTarget::CastTarget, _, _, _))
            )
        {
            return Err(GameError::InvalidAction);
        }

        Ok((skill, cast_target))
    }

    pub(super) fn resolve_skill_cast_target(
        &self,
        skill: &SkillDef,
        caster_instance_id: UnitInstanceId,
        caster_owner: Side,
        caster_pos: Position,
    ) -> Option<SkillCastTarget> {
        let (target, range_policy, defense_tile_range, air_capable) =
            skill.cast_target_definition()?;
        let usefulness_step = self.skill_cast_target_usefulness_step(skill, target);
        self.resolve_skill_target_definition(
            caster_instance_id,
            caster_owner,
            caster_pos,
            range_policy,
            defense_tile_range,
            target,
            air_capable,
            usefulness_step,
        )
    }

    fn resolve_skill_target_definition(
        &self,
        caster_instance_id: UnitInstanceId,
        caster_owner: Side,
        caster_pos: Position,
        range_policy: crate::game::battle::tile_range::TileRangePolicy,
        defense_tile_range: Option<&crate::game::battle::tile_range::TileRangePattern>,
        target: &SkillTarget,
        air_capable: bool,
        usefulness_step: Option<&SkillStepDef>,
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
                    range_policy,
                    defense_tile_range,
                    *rule,
                    air_capable,
                    usefulness_step,
                )
                .map(|id| SkillCastTarget::Unit {
                    unit_instance_id: id,
                }),
            SkillTarget::CastTarget => None,
        }
    }

    pub(in crate::game::battle::core) fn resolve_skill_step_context(
        &self,
        cast_seq: u64,
        caster_instance_id: UnitInstanceId,
        step: &SkillStepDef,
        cast_target: Option<SkillCastTarget>,
    ) -> Option<SkillCastTarget> {
        let (caster_owner, caster_pos) =
            self.resolve_cast_origin_context(Some(cast_seq), caster_instance_id, false)?;
        let cast_target_anchor_position = self
            .active_skill_casts
            .get(&cast_seq)
            .and_then(|cast_state| cast_state.cast_target_anchor_position);

        match step.targeting {
            StepTargetingMode::ReuseCastTarget => match &step.target {
                SkillTarget::SelfUnit => Some(SkillCastTarget::Unit {
                    unit_instance_id: caster_instance_id,
                }),
                SkillTarget::EnemySingle { .. } => match cast_target {
                    Some(SkillCastTarget::Unit { unit_instance_id })
                        if self.is_alive_enemy(unit_instance_id, caster_owner)
                            && self.single_target_can_target_unit(
                                unit_instance_id,
                                step.air_capable,
                            )
                            && self.is_target_in_tile_range_policy(
                                caster_instance_id,
                                unit_instance_id,
                                step.range_policy,
                                step.defense_tile_range.as_ref(),
                            ) =>
                    {
                        Some(SkillCastTarget::Unit { unit_instance_id })
                    }
                    _ => None,
                },
                SkillTarget::CastTarget => match cast_target {
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
                step.range_policy,
                step.defense_tile_range.as_ref(),
                &step.target,
                step.air_capable,
                Some(step),
            ),
        }
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

    fn compute_winner(&mut self, current_time_ms: u64) -> Option<BattleWinner> {
        if let Some(winner) = self.scenario_runtime.forced_winner {
            return Some(winner);
        }

        let mut player_alive = false;
        let mut opponent_alive = false;

        for unit in self.units.values() {
            if !unit.is_active() || !unit.is_combatant() {
                continue;
            }

            match unit.owner {
                Side::Player => player_alive = true,
                Side::Opponent => opponent_alive = true,
            }
        }

        #[allow(deprecated)]
        match self.scenario.win_condition.clone() {
            WinCondition::AllRequiredEnemyGroupsDefeated => {
                if !player_alive {
                    return Some(if opponent_alive {
                        BattleWinner::Opponent
                    } else {
                        BattleWinner::Draw
                    });
                }

                let required_enemy_group_count = self.required_enemy_group_count();
                if required_enemy_group_count == 0 {
                    return Some(if opponent_alive {
                        BattleWinner::Opponent
                    } else {
                        BattleWinner::Player
                    });
                }

                if self.all_required_enemy_groups_defeated() {
                    return Some(BattleWinner::Player);
                }

                return None;
            }
            WinCondition::DefeatUnit { unit_ref } => {
                let defeated = self
                    .scenario_runtime
                    .unit_refs
                    .get(&unit_ref)
                    .and_then(|unit_id| self.units.get(unit_id))
                    .is_some_and(|unit| unit.is_dead());
                if defeated {
                    return Some(BattleWinner::Player);
                }
                if !player_alive {
                    return Some(if opponent_alive {
                        BattleWinner::Opponent
                    } else {
                        BattleWinner::Draw
                    });
                }
                return None;
            }
            WinCondition::ProtectUnit { unit_ref } => {
                let protected_destroyed = self
                    .scenario_runtime
                    .unit_refs
                    .get(&unit_ref)
                    .and_then(|unit_id| self.units.get(unit_id))
                    .is_some_and(|unit| unit.is_dead());
                if protected_destroyed {
                    return Some(BattleWinner::Opponent);
                }
                if self.required_enemy_group_count() > 0
                    && self.all_required_enemy_groups_defeated()
                {
                    return Some(BattleWinner::Player);
                }
                return None;
            }
            WinCondition::ProtectUnitUntil { unit_ref, time_ms } => {
                let protected_destroyed = self
                    .scenario_runtime
                    .unit_refs
                    .get(&unit_ref)
                    .and_then(|unit_id| self.units.get(unit_id))
                    .is_some_and(|unit| unit.is_dead());
                if protected_destroyed {
                    return Some(BattleWinner::Opponent);
                }
                if current_time_ms >= time_ms {
                    return Some(BattleWinner::Player);
                }
                return None;
            }
            WinCondition::SurviveUntil { time_ms } => {
                if current_time_ms >= time_ms {
                    return Some(BattleWinner::Player);
                }
                if !player_alive {
                    return Some(if opponent_alive {
                        BattleWinner::Opponent
                    } else {
                        BattleWinner::Draw
                    });
                }
                return None;
            }
        }
    }

    fn required_enemy_group_count(&self) -> usize {
        self.scenario
            .groups
            .iter()
            .filter(|group| group.side == Side::Opponent && group.required_for_victory)
            .count()
    }

    fn all_required_enemy_groups_defeated(&self) -> bool {
        self.scenario
            .groups
            .iter()
            .filter(|group| group.side == Side::Opponent && group.required_for_victory)
            .all(|group| {
                self.scenario_runtime.spawned_groups.contains(&group.id)
                    && group.spawns.iter().all(|spawn| {
                        self.scenario_runtime
                            .unit_refs
                            .get(&spawn.unit_ref)
                            .and_then(|unit_id| self.units.get(unit_id))
                            .is_none_or(|unit| unit.is_dead())
                    })
            })
    }

    fn finish_battle(&mut self, time_ms: u64, winner: BattleWinner) -> BattleResult {
        self.record_event_log(time_ms, BattleLogEvent::BattleEnd { winner });
        let participant_results = self.participant_results();
        BattleResult {
            winner,
            event_log: self.event_log.clone(),
            participant_results,
        }
    }

    fn finish_signal(
        state: &mut BattleExecutionState,
        time_ms: u64,
        winner: BattleWinner,
    ) -> BattleStepOutcome {
        state.finished = true;
        BattleStepOutcome::Finished(BattleFinishSignal { time_ms, winner })
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
    ) {
        if spawned_unit_ids.is_empty() {
            return;
        }

        self.with_recording_context(cause, |core| {
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
                core.record_event_log(
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
        });
    }

    fn enqueue_scenario_events(&mut self) {
        for event in self.scenario.events.clone() {
            let time_ms = match event.trigger {
                ScenarioTrigger::AtBattleStart => 0,
                ScenarioTrigger::AtTimeMs(time_ms) => time_ms,
            };
            match event.action {
                ScenarioAction::SpawnGroup { group_id } => {
                    self.event_queue.push(BattleEvent::SpawnGroup {
                        time_ms,
                        group_id,
                        cause: BattleEventCause::Root {
                            kind: BattleEventRootCause::Init,
                        },
                    });
                }
                ScenarioAction::EndBattle { winner } => {
                    self.event_queue.push(BattleEvent::EndBattle {
                        time_ms,
                        winner,
                        cause: BattleEventCause::Root {
                            kind: BattleEventRootCause::System,
                        },
                    });
                }
            }
        }
    }

    fn has_queued_spawn_group_at(&self, time_ms: u64) -> bool {
        self.event_queue.iter().any(
            |event| matches!(event, BattleEvent::SpawnGroup { time_ms: queued, .. } if *queued == time_ms),
        )
    }

    fn run_on_battle_start_hooks(&mut self) {
        let mut on_battle_start_commands = Vec::new();
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
            on_battle_start_commands.extend(Self::activation_commands_from_bindings(
                self.collect_all_trigger_activations(
                    unit_id,
                    crate::game::stats::TriggerType::OnBattleStart,
                ),
                unit_id,
                crate::game::battle::core::commands::TriggerAbilityContext {
                    trigger_type: crate::game::stats::TriggerType::OnBattleStart,
                    trigger_unit_id: unit_id,
                    counterpart_unit_id: None,
                    target_id: None,
                    occurrence_id,
                    occurrence_index: 0,
                },
            ));
        }
        if !on_battle_start_commands.is_empty() {
            self.with_recording_root(BattleEventRootCause::Init, |core| {
                core.process_commands(on_battle_start_commands, 0);
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
            |event| matches!(event, BattleEvent::ContinuousMovementTick { time_ms: queued } if *queued == time_ms),
        ) {
            return;
        }

        self.event_queue
            .push(BattleEvent::ContinuousMovementTick { time_ms });
    }

    pub fn run_battle(&mut self) -> Result<BattleResult, GameError> {
        self.run_battle_with_setup(|_| {})
    }

    pub fn run_battle_with_setup<F>(&mut self, setup: F) -> Result<BattleResult, GameError>
    where
        F: FnOnce(&mut Self),
    {
        self.run_battle_with_hooks(setup, None::<fn(&mut Self)>)
    }

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
        self.units.clear();
        self.artifacts.clear();
        self.items.clear();
        self.graveyard.clear();
        self.buffs.clear();
        self.active_skill_casts.clear();
        self.ability_proc_states.clear();
        self.projectiles.clear();
        self.active_projectiles.clear();
        self.active_areas.clear();
        self.active_movement_segments.clear();
        self.last_continuous_movement_tick_ms = None;
        self.movement_backend.reset_for_battle();
        self.battlefield.clear();
        self.event_log = BattleEventLog::new();
        self.event_log_seq = 1;
        self.projectile_seq = 0;
        self.area_seq = 0;
        self.damage_source_seq = 0;
        self.live_signals.clear();
        self.recording_cause_stack.clear();
        self.recording_source_command_stack.clear();
        self.event_queue.clear();
        self.scenario_runtime = Default::default();
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
        std::mem::take(&mut self.live_signals)
    }

    pub fn finalize_battle_execution(
        &mut self,
        state: &mut BattleExecutionState,
        finish: BattleFinishSignal,
    ) -> Result<BattleResult, GameError> {
        if !state.finished || state.finalized {
            return Err(GameError::InvalidAction);
        }

        state.finalized = true;
        Ok(self.finish_battle(finish.time_ms, finish.winner))
    }

    pub fn apply_live_command(
        &mut self,
        command: BattleLiveCommand,
    ) -> Result<BattleLiveCommandOutcome, GameError> {
        match command {
            BattleLiveCommand::DeployPlayerUnit {
                draft,
                position,
                facing,
                instance_salt,
                time_ms,
                current_hp_policy,
            } => {
                let unit_id = self.deploy_player_unit(
                    draft,
                    position,
                    facing,
                    instance_salt,
                    time_ms,
                    current_hp_policy,
                )?;
                Ok(BattleLiveCommandOutcome::UnitDeployed { unit_id })
            }
            BattleLiveCommand::WithdrawUnit { unit_id, time_ms } => {
                self.withdraw_unit(unit_id, time_ms)?;
                Ok(BattleLiveCommandOutcome::UnitWithdrawn { unit_id })
            }
            BattleLiveCommand::ActivateSkill {
                unit_id,
                skill_id,
                target,
                time_ms,
            } => {
                self.start_manual_skill_cast(time_ms, unit_id, skill_id, target)?;
                Ok(BattleLiveCommandOutcome::SkillActivated { unit_id })
            }
        }
    }

    pub fn apply_live_command_with_source_command_id(
        &mut self,
        command: BattleLiveCommand,
        source_command_id: Option<&str>,
    ) -> Result<BattleLiveCommandOutcome, GameError> {
        match source_command_id {
            Some(source_command_id) => self
                .with_recording_source_command_id(source_command_id, |core| {
                    core.apply_live_command(command)
                }),
            None => self.apply_live_command(command),
        }
    }

    fn start_battle_execution_with_setup<PreSetup>(
        &mut self,
        setup: PreSetup,
    ) -> Result<BattleExecutionState, GameError>
    where
        PreSetup: FnOnce(&mut Self),
    {
        self.scenario.validate()?;

        self.reset_runtime_state_for_battle();

        self.build_runtime_artifacts_from_scenario()?;
        for obstacle in self.scenario.battlefield.obstacles.clone() {
            self.battlefield.add_static_obstacle(obstacle)?;
        }
        setup(self);
        self.build_runtime_field()?;

        self.with_recording_root(BattleEventRootCause::Init, |core| {
            core.record_event_log(
                0,
                BattleLogEvent::BattleStart {
                    width: core.battlefield.width(),
                    height: core.battlefield.height(),
                },
            );

            let mut artifact_records: Vec<(Uuid, Side, Uuid)> = core
                .artifacts
                .values()
                .map(|a| (a.instance_id, a.owner, a.base_uuid))
                .collect();
            artifact_records.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
            for (artifact_instance_id, owner, base_uuid) in artifact_records {
                core.record_event_log(
                    0,
                    BattleLogEvent::ArtifactSpawned {
                        artifact_instance_id,
                        owner,
                        base_uuid,
                    },
                );
            }
        });

        self.enqueue_scenario_events();

        Ok(BattleExecutionState {
            last_event_time_ms: 0,
            battle_start_hooks_ran: false,
            finished: false,
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
        if state.finished {
            return Err(GameError::InvalidAction);
        }

        let Some(next_time_ms) = self.event_queue.peek().map(|e| e.time_ms()) else {
            let end_time_ms = state.last_event_time_ms.min(self.max_battle_time_ms());
            let winner = self
                .compute_winner(end_time_ms)
                .unwrap_or(BattleWinner::Draw);
            return Ok(Self::finish_signal(state, end_time_ms, winner));
        };

        let current_time_ms = next_time_ms;
        state.last_event_time_ms = current_time_ms;

        let max_battle_time_ms = self.max_battle_time_ms();
        if current_time_ms > max_battle_time_ms {
            return Ok(Self::finish_signal(
                state,
                max_battle_time_ms,
                BattleWinner::Draw,
            ));
        }

        let mut bucket: Vec<BattleEvent> = Vec::new();
        while matches!(self.event_queue.peek(), Some(e) if e.time_ms() == current_time_ms) {
            bucket.push(self.event_queue.pop().unwrap());
        }

        loop {
            if bucket.is_empty() {
                let mut new_bucket: Vec<BattleEvent> = Vec::new();
                while matches!(self.event_queue.peek(), Some(e) if e.time_ms() == current_time_ms) {
                    new_bucket.push(self.event_queue.pop().unwrap());
                }

                if new_bucket.is_empty() {
                    break;
                }

                bucket = new_bucket;
                continue;
            }

            bucket.sort_by(|a, b| b.cmp(a));
            for event in bucket.drain(..) {
                self.process_event(event, current_time_ms)?;
            }

            if current_time_ms == 0
                && !state.battle_start_hooks_ran
                && !self.has_queued_spawn_group_at(0)
            {
                if let Some(setup) = post_spawn_setup.take() {
                    setup(self);
                }
                state.battle_start_hooks_ran = true;
                self.run_on_battle_start_hooks();
                self.schedule_continuous_movement_tick(0);
            }

            if let Some(winner) = self.compute_winner(current_time_ms) {
                if self.has_pending_attack_resolve_at(current_time_ms) {
                    continue;
                }
                return Ok(Self::finish_signal(state, current_time_ms, winner));
            }
        }

        if let Some(winner) = self.compute_winner(current_time_ms) {
            return Ok(Self::finish_signal(state, current_time_ms, winner));
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
        if state.finished {
            return Err(GameError::InvalidAction);
        }

        let target_time_ms = target_time_ms.min(self.max_battle_time_ms());
        if target_time_ms < state.last_event_time_ms {
            return Ok(BattleStepOutcome::Running);
        }

        loop {
            let Some(next_time_ms) = self.event_queue.peek().map(|event| event.time_ms()) else {
                state.last_event_time_ms = target_time_ms;
                let winner = self
                    .compute_winner(target_time_ms)
                    .unwrap_or(BattleWinner::Draw);
                return Ok(Self::finish_signal(state, target_time_ms, winner));
            };

            if next_time_ms > target_time_ms {
                state.last_event_time_ms = target_time_ms;
                if let Some(winner) = self.compute_winner(target_time_ms) {
                    return Ok(Self::finish_signal(state, target_time_ms, winner));
                }
                return Ok(BattleStepOutcome::Running);
            }

            match self.step_battle_execution_with_post_spawn_setup(state, post_spawn_setup)? {
                BattleStepOutcome::Running => {
                    if state.last_event_time_ms >= target_time_ms {
                        return Ok(BattleStepOutcome::Running);
                    }
                }
                finished @ BattleStepOutcome::Finished(_) => return Ok(finished),
            }
        }
    }

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
                BattleStepOutcome::Finished(finish) => {
                    return self.finalize_battle_execution(&mut state, finish);
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

    pub(super) fn build_skill_step_commands(
        &mut self,
        caster_instance_id: UnitInstanceId,
        step: &SkillStepDef,
        targets: &[UnitInstanceId],
        committed_at_ms: u64,
        source_template: Option<&crate::game::battle::damage::DamageSourceSnapshot>,
    ) -> (Vec<BattleCommand>, SkillStepResult) {
        let mut commands: Vec<BattleCommand> = Vec::new();
        let mut result = SkillStepResult {
            resolved_target_count: targets.len(),
            ..SkillStepResult::default()
        };

        let hinted_target_id = if targets.len() == 1 {
            Some(targets[0])
        } else {
            None
        };

        let step_damage_modifiers = step
            .effects
            .iter()
            .filter_map(|effect| match effect {
                SkillEffectDef::ModifyDamage { modifiers } => Some(*modifiers),
                _ => None,
            })
            .fold(
                Default::default(),
                crate::game::battle::damage::DamageModifiers::merge,
            );

        for effect in &step.effects {
            match effect {
                SkillEffectDef::Damage {
                    amount,
                    damage_type,
                } => {
                    if *amount == 0 {
                        continue;
                    }
                    let damage_amount = amount.unsigned_abs();
                    for (hit_index, target_id) in targets.iter().enumerate() {
                        let source_snapshot = if let Some(template) = source_template {
                            let mut snapshot = template.clone();
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
                        commands.push(BattleCommand::ApplyDamage {
                            target_id: *target_id,
                            source_snapshot,
                        });
                    }
                    if !targets.is_empty() {
                        result.applied_effect_count =
                            result.applied_effect_count.saturating_add(targets.len());
                    }
                }
                SkillEffectDef::Heal { amount } => {
                    for target_id in targets {
                        commands.push(BattleCommand::ApplyHeal {
                            target_id: *target_id,
                            flat: *amount,
                            percent: 0,
                            source_id: Some(caster_instance_id),
                        });
                    }
                    if !targets.is_empty() {
                        result.applied_effect_count =
                            result.applied_effect_count.saturating_add(targets.len());
                    }
                }
                SkillEffectDef::ModifyResonance { amount } => {
                    for target_id in targets {
                        commands.push(BattleCommand::ModifyResonance {
                            target_id: *target_id,
                            amount: *amount,
                            allow_autocast_when_full: *amount > 0,
                        });
                    }
                    if !targets.is_empty() {
                        result.applied_effect_count =
                            result.applied_effect_count.saturating_add(targets.len());
                    }
                }
                SkillEffectDef::ModifyStabilization { .. } => {}
                SkillEffectDef::ModifyStats { modifier } => {
                    for target_id in targets {
                        commands.push(BattleCommand::ApplyModifier {
                            target_id: *target_id,
                            modifier: *modifier,
                        });
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
                        commands.push(BattleCommand::ApplyBuff {
                            caster_id: caster_instance_id,
                            target_id: *target_id,
                            buff_id,
                            duration_ms: u64::from(*duration_ms),
                        });
                    }
                    if !targets.is_empty() {
                        result.applied_effect_count =
                            result.applied_effect_count.saturating_add(targets.len());
                    }
                }
                SkillEffectDef::InterruptCast => {
                    for target_id in targets {
                        commands.push(BattleCommand::InterruptCast {
                            source_id: caster_instance_id,
                            target_id: *target_id,
                        });
                    }
                    if !targets.is_empty() {
                        result.applied_effect_count =
                            result.applied_effect_count.saturating_add(targets.len());
                    }
                }
                SkillEffectDef::ExtraAttack { count } => {
                    for _ in 0..(*count as usize) {
                        commands.push(BattleCommand::ScheduleAttack {
                            attacker_id: caster_instance_id,
                            target_id: hinted_target_id,
                            time_ms: 0,
                        });
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

        (commands, result)
    }

    pub(super) fn record_skill_step_live_signals(
        &mut self,
        caster_instance_id: UnitInstanceId,
        skill_id: &SkillId,
        step: &SkillStepDef,
        targets: &[UnitInstanceId],
    ) -> SkillStepResult {
        let mut result = SkillStepResult::default();
        if targets.is_empty() {
            return result;
        }

        for effect in &step.effects {
            if let SkillEffectDef::ModifyStabilization { amount } = effect {
                if *amount == 0 {
                    continue;
                }
                self.live_signals
                    .push(BattleLiveSignal::StabilizationDelta {
                        source_unit_id: caster_instance_id,
                        skill_id: skill_id.clone(),
                        amount: *amount,
                    });
                result.applied_effect_count = result.applied_effect_count.saturating_add(1);
            }
        }

        result
    }

    pub(super) fn resolve_skill_step<'a>(
        skill: &'a SkillDef,
        step_index: usize,
        step_id: &str,
    ) -> Option<&'a SkillStepDef> {
        let step = skill.steps.get(step_index)?;
        debug_assert_eq!(
            step.id, step_id,
            "skill step invariant broken: index {} resolved to '{}' but event carried '{}'",
            step_index, step.id, step_id
        );
        Some(step)
    }

    fn execute_skill_step(&mut self, execution: SkillStepExecution<'_>) {
        let step_target = self.resolve_skill_step_context(
            execution.cast_seq,
            execution.caster_instance_id,
            execution.step,
            execution.cast_target,
        );
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
                return;
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
                return;
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
            return;
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

        self.with_recording_cause(step_seq, |core| match &execution.step.delivery {
            DeliveryDef::Instant => {
                let mut aggregated_result = SkillStepResult::default();

                for _ in 0..repeat_count {
                    let iteration_target = core.resolve_skill_step_context(
                        execution.cast_seq,
                        execution.caster_instance_id,
                        execution.step,
                        execution.cast_target,
                    );
                    let targets = core.resolve_skill_step_targets(
                        execution.cast_seq,
                        execution.caster_instance_id,
                        execution.step,
                        iteration_target,
                    );
                    let (commands, result) = core.build_skill_step_commands(
                        execution.caster_instance_id,
                        execution.step,
                        &targets,
                        execution.time_ms,
                        None,
                    );
                    if !commands.is_empty() {
                        let summary = core.process_commands(commands, execution.time_ms);
                        aggregated_result.actual_damage_target_count = aggregated_result
                            .actual_damage_target_count
                            .saturating_add(summary.actual_damage_target_count);
                    }
                    aggregated_result.merge(&result);
                    let signal_result = core.record_skill_step_live_signals(
                        execution.caster_instance_id,
                        &execution.skill.id,
                        execution.step,
                        &targets,
                    );
                    aggregated_result.merge(&signal_result);
                }

                core.finalize_skill_step(
                    execution.cast_seq,
                    execution.step_index,
                    aggregated_result,
                    execution.time_ms,
                );
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
                );
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
            }
            DeliveryDef::TileArea { area } => {
                let Some(tile_range) = execution.step.defense_tile_range.as_ref() else {
                    core.finalize_skill_step(
                        execution.cast_seq,
                        execution.step_index,
                        SkillStepResult::default(),
                        execution.time_ms,
                    );
                    return;
                };

                if area.duration_ms == 0 {
                    let mut aggregated_result = SkillStepResult::default();

                    for _ in 0..repeat_count {
                        let iteration_target = core.resolve_skill_step_context(
                            execution.cast_seq,
                            execution.caster_instance_id,
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
                            tile_range,
                            area,
                        )
                        else {
                            continue;
                        };
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

                        let (commands, result) = core.build_skill_step_commands(
                            execution.caster_instance_id,
                            execution.step,
                            &targets,
                            execution.time_ms,
                            None,
                        );
                        if !commands.is_empty() {
                            let summary = core.with_recording_cause(area_declared_seq, |core| {
                                core.process_commands(commands, execution.time_ms)
                            });
                            aggregated_result.actual_damage_target_count = aggregated_result
                                .actual_damage_target_count
                                .saturating_add(summary.actual_damage_target_count);
                        }
                        aggregated_result.merge(&result);
                        let signal_result = core.record_skill_step_live_signals(
                            execution.caster_instance_id,
                            &execution.skill.id,
                            execution.step,
                            &targets,
                        );
                        aggregated_result.merge(&signal_result);
                    }

                    core.finalize_skill_step(
                        execution.cast_seq,
                        execution.step_index,
                        aggregated_result,
                        execution.time_ms,
                    );
                    return;
                }

                let mut spawned_areas = 0usize;
                for _ in 0..repeat_count {
                    let iteration_target = core.resolve_skill_step_context(
                        execution.cast_seq,
                        execution.caster_instance_id,
                        execution.step,
                        execution.cast_target,
                    );
                    if core.register_persistent_tile_area(
                        execution.time_ms,
                        execution.cast_seq,
                        execution.step_index,
                        execution.caster_instance_id,
                        execution.skill.id.clone(),
                        execution.step.id.clone(),
                        execution.step,
                        iteration_target,
                        tile_range.clone(),
                        area.clone(),
                    ) {
                        spawned_areas = spawned_areas.saturating_add(1);
                    }
                }
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
            }
        });
    }

    pub(super) fn process_event(
        &mut self,
        event: BattleEvent,
        current_time_ms: u64,
    ) -> Result<(), GameError> {
        match event {
            BattleEvent::SpawnGroup {
                time_ms,
                group_id,
                cause,
            } => {
                let spawned_unit_ids = self.spawn_scenario_group(&group_id)?;
                self.record_spawned_units(time_ms, cause, &spawned_unit_ids);

                for unit_id in spawned_unit_ids {
                    self.schedule_initial_attack_for_unit(unit_id, time_ms);
                }
                Ok(())
            }
            BattleEvent::EndBattle { winner, cause, .. } => {
                self.with_recording_context(cause, |core| {
                    core.scenario_runtime.forced_winner = Some(winner);
                });
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
            BattleEvent::AttackResolve {
                time_ms,
                attacker_instance_id,
                target_instance_id,
                source_snapshot,
                kind,
                delivery,
                cause,
            } => {
                self.handle_basic_attack_resolve_event(
                    time_ms,
                    attacker_instance_id,
                    target_instance_id,
                    source_snapshot,
                    kind,
                    delivery,
                    cause,
                );

                Ok(())
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
            } => {
                self.with_recording_context(cause, |core| {
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
                    );
                });

                Ok(())
            }
            BattleEvent::SkillProjectileAdvance {
                time_ms,
                delivery_id,
                ..
            } => {
                self.advance_skill_projectile(time_ms, delivery_id);
                Ok(())
            }
            BattleEvent::SkillAreaTick {
                time_ms,
                area_id,
                cause,
                ..
            } => {
                self.with_recording_context(cause, |core| {
                    core.apply_skill_area_tick(time_ms, area_id);
                });
                Ok(())
            }
            BattleEvent::SkillAreaExpire { area_id, cause, .. } => {
                self.with_recording_context(cause, |core| {
                    core.expire_skill_area(area_id);
                });
                Ok(())
            }
            BattleEvent::AutoCastStart {
                time_ms,
                caster_instance_id,
                cause,
            } => {
                let Some(caster) = self.units.get(&caster_instance_id) else {
                    return Ok(());
                };
                if !caster.is_active() {
                    return Ok(());
                }
                if self.has_buff_kind(
                    caster_instance_id,
                    crate::game::battle::buffs::BuffKind::Silence,
                ) {
                    if let Some(unit) = self.units.get_mut(&caster_instance_id) {
                        unit.pending_cast = true;
                        unit.pending_cast_cause.get_or_insert(cause);
                    }
                    return Ok(());
                }
                let blocked_until = caster.next_action_time.max(
                    self.active_hard_cc_release_time_ms(caster_instance_id, time_ms)
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
                    .and_then(|id| self.game_data.skill_data.get_by_id(id))
                {
                    Some(skill) => skill.clone(),
                    None => return Ok(()),
                };

                let target = self.resolve_skill_cast_target(
                    &skill,
                    caster_instance_id,
                    caster_owner,
                    caster_pos,
                );
                if !self.skill_has_useful_automatic_cast_target(
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

                // "캐스트/집중"이 진행되는 동안 새로운 AutoCastStart를 막기 위한 캐스트 락.
                // (공격/이동은 ActionLocks로 개별 게이트)
                let cast_end_ms = if skill.focus_time_ms == 0 {
                    time_ms.saturating_add(1)
                } else {
                    time_ms.saturating_add(skill.focus_time_ms as u64)
                };
                if let Some(caster) = self.units.get_mut(&caster_instance_id) {
                    caster.next_action_time = cast_end_ms;

                    // 집중/시전 중 공명 획득은 항상 금지
                    caster.action_locks.lock_resonance_gain_until(cast_end_ms);

                    if !skill.focus_permissions.allows_basic_attack {
                        caster.action_locks.lock_basic_attack_until(cast_end_ms);
                    }

                    if !skill.focus_permissions.allows_move {
                        caster.action_locks.lock_movement_until(cast_end_ms);
                    }
                }

                // 이동 금지 스킬이면 continuous movement 상태를 즉시 정지한다.
                if !skill.focus_permissions.allows_move {
                    let interrupted = self.interrupt_movement(
                        time_ms,
                        caster_instance_id,
                        crate::game::battle::event_log::MovementStopReason::CastStarted,
                        Some(cast_end_ms),
                        ActionState::Idle,
                    );
                    if interrupted {
                        self.schedule_continuous_movement_tick(cast_end_ms);
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

                if let Some(caster) = self.units.get_mut(&caster_instance_id) {
                    caster.pending_skill_cast = Some(PendingSkillCast {
                        skill_id: skill.id.clone(),
                        cast_target: target,
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
                        self.active_hard_cc_release_time_ms(caster_instance_id, time_ms)
                            .unwrap_or(0),
                    );
                if time_ms < blocked_until {
                    self.event_queue.push(BattleEvent::AutoCastEnd {
                        time_ms: blocked_until,
                        caster_instance_id,
                        cause,
                    });
                    return Ok(());
                }

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
                self.with_recording_context(cause, |core| {
                    if let Some(pending) = pending {
                        core.invoke_ability(
                            time_ms,
                            caster_instance_id,
                            &pending.skill_id,
                            pending.cast_target,
                            cause,
                            false,
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
                caster.next_basic_attack_ms = caster.next_basic_attack_ms.max(recovery_ends_at);
                caster.resonance_current = 0;
                caster
                    .action_locks
                    .lock_resonance_gain_until(time_ms.saturating_add(caster.resonance_lock_ms));
                if caster.next_action_time <= time_ms {
                    caster.next_action_time = 0;
                }
                caster.pending_cast = false;

                Ok(())
            }
            BattleEvent::ManualCastEnd {
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
                        self.active_hard_cc_release_time_ms(caster_instance_id, time_ms)
                            .unwrap_or(0),
                    );
                if time_ms < blocked_until {
                    self.event_queue.push(BattleEvent::ManualCastEnd {
                        time_ms: blocked_until,
                        caster_instance_id,
                        cause,
                    });
                    return Ok(());
                }

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
                self.with_recording_context(cause, |core| {
                    if let Some(pending) = pending {
                        core.invoke_ability(
                            time_ms,
                            caster_instance_id,
                            &pending.skill_id,
                            pending.cast_target,
                            cause,
                            false,
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
                caster.next_basic_attack_ms = caster.next_basic_attack_ms.max(recovery_ends_at);
                caster.resonance_current = 0;
                caster
                    .action_locks
                    .lock_resonance_gain_until(time_ms.saturating_add(caster.resonance_lock_ms));
                if caster.next_action_time <= time_ms {
                    caster.next_action_time = 0;
                }
                caster.pending_cast = false;
                caster.pending_cast_cause = None;

                Ok(())
            }
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
                let Some(skill) = self.game_data.skill_data.get_by_id(&skill_id).cloned() else {
                    return Ok(());
                };
                let Some(step) = Self::resolve_skill_step(&skill, step_index, &step_id).cloned()
                else {
                    return Ok(());
                };

                self.execute_skill_step(SkillStepExecution {
                    time_ms,
                    cast_seq,
                    step_index,
                    caster_instance_id,
                    skill: &skill,
                    step: &step,
                    cast_target,
                    cause,
                });
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
                let Some(def) = self.game_data.buff_data.get(buff_id).cloned() else {
                    return Ok(());
                };

                if duration_ms == 0 {
                    return Ok(());
                }

                if !matches!(
                    self.units.get(&target_instance_id),
                    Some(unit) if unit.is_active()
                ) {
                    return Ok(());
                }

                let key = BuffInstanceKey {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                };

                let expires_at_ms = time_ms.saturating_add(duration_ms);
                let max_stacks = def.max_stacks.max(1);
                let is_hard_cc = Self::is_hard_cc_kind(def.kind);

                // Hard CC is exclusive per target. Reapplying the same active
                // buff refreshes it; applying a different hard CC explicitly
                // ends the old one before the new BuffApplied event.
                if is_hard_cc {
                    let mut replaced_keys = self
                        .buffs
                        .keys()
                        .copied()
                        .filter(|active_key| {
                            active_key.target_instance_id == target_instance_id
                                && *active_key != key
                                && self
                                    .game_data
                                    .buff_data
                                    .get(active_key.buff_id)
                                    .is_some_and(|active_def| {
                                        Self::is_hard_cc_kind(active_def.kind)
                                    })
                        })
                        .collect::<Vec<_>>();
                    replaced_keys.sort_by(|left, right| {
                        left.target_instance_id
                            .cmp(&right.target_instance_id)
                            .then_with(|| left.caster_instance_id.cmp(&right.caster_instance_id))
                            .then_with(|| left.buff_id.as_u64().cmp(&right.buff_id.as_u64()))
                    });
                    for replaced_key in replaced_keys {
                        self.buffs.remove(&replaced_key);
                        self.with_recording_context(cause, |core| {
                            core.record_event_log(
                                time_ms,
                                BattleLogEvent::BuffExpired {
                                    caster_instance_id: replaced_key.caster_instance_id,
                                    target_instance_id: replaced_key.target_instance_id,
                                    buff_id: replaced_key.buff_id,
                                    reason: BuffExpireReason::Replaced,
                                },
                            )
                        });
                    }
                }

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

                let entry = self.buffs.entry(key).or_insert(ActiveBuff {
                    stacks: 0,
                    expires_at_ms,
                    next_tick_ms: None,
                });

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
                        self.event_queue.push(BattleEvent::BuffTick {
                            time_ms: tick_time_ms,
                            caster_instance_id,
                            target_instance_id,
                            buff_id,
                            cause: BattleEventCause::Parent { seq: applied_seq },
                        });
                    }
                }

                self.event_queue.push(BattleEvent::BuffExpire {
                    time_ms: effective_expires_at_ms,
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                    cause: BattleEventCause::Parent { seq: applied_seq },
                });

                if is_hard_cc {
                    let lock_until = effective_expires_at_ms.saturating_add(1);
                    if let Some(unit) = self.units.get_mut(&target_instance_id) {
                        unit.current_target = None;
                    }

                    self.interrupt_movement(
                        time_ms,
                        target_instance_id,
                        crate::game::battle::event_log::MovementStopReason::HardCC,
                        Some(lock_until),
                        ActionState::Idle,
                    );
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
                let Some(def) = self.game_data.buff_data.get(buff_id).cloned() else {
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
                        self.event_queue.push(BattleEvent::BuffTick {
                            time_ms: next_tick_ms,
                            caster_instance_id,
                            target_instance_id,
                            buff_id,
                            cause,
                        });
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
                let Some(def) = self.game_data.buff_data.get(buff_id).cloned() else {
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

                if matches!(
                    def.kind,
                    crate::game::battle::buffs::BuffKind::Stun
                        | crate::game::battle::buffs::BuffKind::Freeze
                        | crate::game::battle::buffs::BuffKind::Silence
                ) {
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
                self.try_start_pending_basic_attacks(time_ms);
                let next_time_ms = time_ms.saturating_add(movement_tick_ms);
                if next_time_ms <= self.max_battle_time_ms() {
                    self.schedule_continuous_movement_tick(next_time_ms);
                }
                Ok(())
            }
        }
    }
}
