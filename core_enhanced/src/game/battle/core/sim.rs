use bevy_ecs::world::World;
use uuid::Uuid;

use crate::{
    ecs::resources::Position,
    game::{
        ability::{
            DeliveryDef, SkillDef, SkillEffectDef, SkillStepCondition, SkillStepDef,
            SkillStepRepeat, SkillTarget, SkillUnitReference, StepTargetingMode,
        },
        battle::{
            cooldown::CooldownSource,
            damage::{BattleCommand, DamageSource},
            enums::BattleEvent,
            ids::UnitInstanceId,
            timeline::{
                AttackKind, SkillCastTarget, Timeline, TimelineCause, TimelineEvent,
                TimelineRootCause,
            },
            types::{BattleResult, BattleWinner},
        },
        behavior::GameError,
        determinism,
        enums::Side,
        stats::UnitStats,
    },
};

use super::{
    movement::{types::DEFAULT_MOVEMENT_TICK_MS, ActionState},
    skill_runtime::cast::PreviousStepDamageGate,
    types::{AbilityProcKey, PendingSkillCast, SkillStepResult},
    ActiveBuff, BattleCore, BuffInstanceKey,
};

const MAX_BATTLE_TIME_MS: u64 = 60_000;

#[derive(Debug, Clone)]
pub(super) struct TriggeredAbilityProcContext<'a> {
    pub(super) source: CooldownSource,
    pub(super) ability_id: &'a str,
    pub(super) binding_index: usize,
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
    cause: TimelineCause,
}

impl BattleCore {
    fn proc_roll_percent(
        &self,
        source: CooldownSource,
        ability_id: &str,
        binding_index: usize,
        trigger_count: u32,
        time_ms: u64,
    ) -> u8 {
        const PROC_ROLL_NS: u64 = 0x5052_4F43_524F_4C4Cu64; // "PROCROLL"

        let source_tag = match source {
            CooldownSource::Unit { unit_instance_id } => {
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&unit_instance_id.as_bytes()[..8]);
                u64::from_be_bytes(bytes)
            }
            CooldownSource::Item { item_instance_id }
            | CooldownSource::Artifact {
                artifact_instance_id: item_instance_id,
            } => {
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&item_instance_id.as_bytes()[..8]);
                u64::from_be_bytes(bytes)
            }
        };
        let ability_tag = ability_id.bytes().fold(0u64, |acc, b| {
            acc.wrapping_mul(131).wrapping_add(u64::from(b))
        });
        let seed = self.seed
            ^ source_tag.rotate_left(13)
            ^ ability_tag.rotate_left(29)
            ^ (binding_index as u64).rotate_left(7)
            ^ time_ms.wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ u64::from(trigger_count).wrapping_mul(0xD1B5_4A32_D192_ED03);

        determinism::uuid_v4_from_seed(seed, PROC_ROLL_NS, u64::from(trigger_count)).as_bytes()[0]
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
            source: context.source,
            ability_id: context.ability_id.to_string(),
            binding_index: context.binding_index,
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

        let roll = self.proc_roll_percent(
            context.source,
            context.ability_id,
            context.binding_index,
            current_count,
            context.current_time_ms,
        );
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
        cause: TimelineCause,
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
                self.battlefield.position_of(unit_instance_id)
            }
            None => None,
        };

        let ability_seq = self.with_recording_context(cause, |core| {
            core.record_timeline(
                time_ms,
                TimelineEvent::AbilityCast {
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
                caster_owner,
                anchor_position: caster_pos,
                cast_target_anchor_position,
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
                    cause: TimelineCause::Parent { seq: ability_seq },
                });
            }
        });
    }

    pub(super) fn resolve_skill_cast_target(
        &self,
        skill: &SkillDef,
        caster_instance_id: UnitInstanceId,
        caster_owner: Side,
        caster_pos: Position,
    ) -> Option<SkillCastTarget> {
        let (range_units, target) = skill.cast_target_definition()?;
        self.resolve_skill_target_definition(
            caster_instance_id,
            caster_owner,
            caster_pos,
            range_units,
            target,
        )
    }

    fn resolve_skill_target_definition(
        &self,
        caster_instance_id: UnitInstanceId,
        caster_owner: Side,
        caster_pos: Position,
        range_units: f32,
        target: &SkillTarget,
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
                    range_units,
                    *rule,
                )
                .map(|id| SkillCastTarget::Unit {
                    unit_instance_id: id,
                }),
            SkillTarget::Allies { area } => Some(SkillCastTarget::Tile {
                position: self
                    .resolve_skill_anchor_position(
                        caster_instance_id,
                        caster_owner,
                        caster_pos,
                        area,
                        range_units,
                        true,
                    )
                    .unwrap_or(caster_pos),
            }),
            SkillTarget::Enemies { area } => self
                .resolve_skill_anchor_position(
                    caster_instance_id,
                    caster_owner,
                    caster_pos,
                    area,
                    range_units,
                    false,
                )
                .map(|position| SkillCastTarget::Tile { position }),
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
                        if self.is_alive_enemy(unit_instance_id, caster_owner) =>
                    {
                        Some(SkillCastTarget::Unit { unit_instance_id })
                    }
                    _ => None,
                },
                SkillTarget::Allies { .. } | SkillTarget::Enemies { .. } => match cast_target {
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
                step.range_units,
                &step.target,
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
            if !caster.is_dead() {
                if let Some(position) = self.battlefield.position_of(caster_instance_id) {
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

    fn compute_winner(&self) -> Option<BattleWinner> {
        let mut player_alive = false;
        let mut opponent_alive = false;

        for unit in self.units.values() {
            if unit.is_dead() {
                continue;
            }

            match unit.owner {
                Side::Player => player_alive = true,
                Side::Opponent => opponent_alive = true,
            }

            if player_alive && opponent_alive {
                return None;
            }
        }

        Some(match (player_alive, opponent_alive) {
            (true, false) => BattleWinner::Player,
            (false, true) => BattleWinner::Opponent,
            (false, false) => BattleWinner::Draw,
            (true, true) => unreachable!(),
        })
    }

    fn finish_battle(&mut self, time_ms: u64, winner: BattleWinner) -> BattleResult {
        self.record_timeline(time_ms, TimelineEvent::BattleEnd { winner });
        BattleResult {
            winner,
            timeline: self.timeline.clone(),
        }
    }

    pub fn init_intial_events(&mut self) {
        let mut unit_ids: Vec<UnitInstanceId> = self.units.keys().copied().collect();
        unit_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        for unit_id in unit_ids {
            let Some(unit) = self.units.get_mut(&unit_id) else {
                continue;
            };
            unit.next_basic_attack_ms = 0;
            unit.pending_basic_attack = false;
            self.event_queue.push(BattleEvent::AttackStart {
                time_ms: 0,
                attacker_instance_id: unit_id,
                target_instance_id: None,
                schedule_next: true,
                cause: TimelineCause::Root {
                    kind: TimelineRootCause::Period,
                },
            });
        }
    }

    pub(super) fn schedule_continuous_movement_tick(&mut self, time_ms: u64) {
        if self
            .last_continuous_movement_tick_ms
            .is_some_and(|last_time_ms| {
                time_ms < last_time_ms.saturating_add(DEFAULT_MOVEMENT_TICK_MS)
            })
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

    pub fn run_battle(&mut self, world: &mut World) -> Result<BattleResult, GameError> {
        self.run_battle_with_setup(world, |_| {})
    }

    pub fn run_battle_with_setup<F>(
        &mut self,
        _world: &mut World,
        setup: F,
    ) -> Result<BattleResult, GameError>
    where
        F: FnOnce(&mut Self),
    {
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
        self.battlefield.clear();
        self.timeline = Timeline::new();
        self.timeline_seq = 0;
        self.projectile_seq = 0;
        self.area_seq = 0;
        self.recording_cause_stack.clear();
        self.event_queue.clear();

        self.build_runtime_units_from_decks(Side::Player)?;
        self.build_runtime_units_from_decks(Side::Opponent)?;
        self.build_runtime_field()?;
        setup(self);

        self.with_recording_root(TimelineRootCause::Init, |core| {
            core.record_timeline(
                0,
                TimelineEvent::BattleStart {
                    width: core.battlefield.width(),
                    height: core.battlefield.height(),
                },
            );

            let mut unit_records: Vec<(UnitInstanceId, Side, Uuid, _, UnitStats)> = core
                .units
                .values()
                .map(|u| {
                    (
                        u.instance_id,
                        u.owner,
                        u.base_uuid,
                        u.world_position().quantized_milli(),
                        u.stats,
                    )
                })
                .collect();
            unit_records.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
            for (unit_instance_id, owner, base_uuid, world_position, stats) in unit_records {
                core.record_timeline(
                    0,
                    TimelineEvent::UnitSpawned {
                        unit_instance_id,
                        owner,
                        base_uuid,
                        world_position,
                        stats,
                    },
                );
            }

            let mut artifact_records: Vec<(Uuid, Side, Uuid)> = core
                .artifacts
                .values()
                .map(|a| (a.instance_id, a.owner, a.base_uuid))
                .collect();
            artifact_records.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
            for (artifact_instance_id, owner, base_uuid) in artifact_records {
                core.record_timeline(
                    0,
                    TimelineEvent::ArtifactSpawned {
                        artifact_instance_id,
                        owner,
                        base_uuid,
                    },
                );
            }

            let mut item_records: Vec<(Uuid, Side, UnitInstanceId, Uuid)> = core
                .items
                .values()
                .map(|i| (i.instance_id, i.owner, i.owner_unit_instance, i.base_uuid))
                .collect();
            item_records.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
            for (item_instance_id, owner, owner_unit_instance_id, base_uuid) in item_records {
                core.record_timeline(
                    0,
                    TimelineEvent::ItemSpawned {
                        item_instance_id,
                        owner,
                        owner_unit_instance_id,
                        base_uuid,
                    },
                );
            }
        });

        let mut on_battle_start_commands = Vec::new();
        let mut unit_ids: Vec<_> = self.units.keys().copied().collect();
        unit_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        for unit_id in unit_ids {
            on_battle_start_commands.extend(Self::activation_commands_from_bindings(
                self.collect_all_trigger_activations(
                    unit_id,
                    crate::game::stats::TriggerType::OnBattleStart,
                ),
                unit_id,
                None,
            ));
        }
        if !on_battle_start_commands.is_empty() {
            self.with_recording_root(TimelineRootCause::Init, |core| {
                core.process_commands(on_battle_start_commands, 0);
            });
        }

        self.init_intial_events();
        self.schedule_continuous_movement_tick(0);

        let mut last_event_time_ms = 0;

        while let Some(next_time_ms) = self.event_queue.peek().map(|e| e.time_ms()) {
            let current_time_ms = next_time_ms;
            last_event_time_ms = current_time_ms;

            if current_time_ms > MAX_BATTLE_TIME_MS {
                return Ok(self.finish_battle(MAX_BATTLE_TIME_MS, BattleWinner::Draw));
            }

            let mut bucket: Vec<BattleEvent> = Vec::new();
            while matches!(self.event_queue.peek(), Some(e) if e.time_ms() == current_time_ms) {
                bucket.push(self.event_queue.pop().unwrap());
            }

            loop {
                if bucket.is_empty() {
                    let mut new_bucket: Vec<BattleEvent> = Vec::new();
                    while matches!(self.event_queue.peek(), Some(e) if e.time_ms() == current_time_ms)
                    {
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

                if let Some(winner) = self.compute_winner() {
                    return Ok(self.finish_battle(current_time_ms, winner));
                }
            }

            if let Some(winner) = self.compute_winner() {
                return Ok(self.finish_battle(current_time_ms, winner));
            }

            if let Some(winner) = self.compute_winner() {
                return Ok(self.finish_battle(current_time_ms, winner));
            }
        }

        let end_time_ms = last_event_time_ms.min(MAX_BATTLE_TIME_MS);
        let winner = self.compute_winner().unwrap_or(BattleWinner::Draw);
        Ok(self.finish_battle(end_time_ms, winner))
    }

    pub(in crate::game::battle::core) fn is_alive_enemy(
        &self,
        unit_id: UnitInstanceId,
        owner: Side,
    ) -> bool {
        match self.units.get(&unit_id) {
            Some(unit) => unit.owner != owner && !unit.is_dead(),
            None => false,
        }
    }

    pub(in crate::game::battle::core) fn persisted_target_if_alive(
        &self,
        owner: Side,
        current_target: Option<UnitInstanceId>,
    ) -> Option<UnitInstanceId> {
        current_target.filter(|id| self.is_alive_enemy(*id, owner))
    }

    pub(in crate::game::battle::core) fn persisted_target_in_range(
        &self,
        attacker_instance_id: UnitInstanceId,
        current_target: Option<UnitInstanceId>,
    ) -> Option<UnitInstanceId> {
        let attacker = self.units.get(&attacker_instance_id)?;
        self.persisted_target_if_alive(attacker.owner, current_target)
            .filter(|id| self.is_basic_attack_target_in_range(attacker_instance_id, *id))
    }

    pub(in crate::game::battle::core) fn select_basic_attack_target(
        &self,
        attacker_instance_id: UnitInstanceId,
        current_target: Option<UnitInstanceId>,
        hinted_target: Option<UnitInstanceId>,
    ) -> Option<UnitInstanceId> {
        let attacker = self.units.get(&attacker_instance_id)?;
        let in_range = |id: UnitInstanceId| {
            self.is_alive_enemy(id, attacker.owner)
                && self.is_basic_attack_target_in_range(attacker_instance_id, id)
        };

        hinted_target
            .filter(|id| in_range(*id))
            .or_else(|| self.persisted_target_in_range(attacker_instance_id, current_target))
            .or_else(|| self.choose_attack_target_in_range(attacker_instance_id))
    }

    pub(super) fn try_start_pending_basic_attacks(&mut self, now_ms: u64) {
        let mut unit_ids: Vec<UnitInstanceId> = self.units.keys().copied().collect();
        unit_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        for unit_id in unit_ids {
            let Some(unit) = self.units.get(&unit_id) else {
                continue;
            };
            let pending = unit.pending_basic_attack;
            let next_ready_ms = unit.next_basic_attack_ms;
            let can_attack = unit.action_locks.can_basic_attack(now_ms);
            let lock_until = unit.action_locks.basic_attack_until_ms;
            if unit.is_dead() || !pending || now_ms < next_ready_ms {
                continue;
            }

            if !can_attack {
                if let Some(unit) = self.units.get_mut(&unit_id) {
                    unit.pending_basic_attack = false;
                }
                self.event_queue.push(BattleEvent::AttackStart {
                    time_ms: lock_until,
                    attacker_instance_id: unit_id,
                    target_instance_id: None,
                    schedule_next: true,
                    cause: TimelineCause::Root {
                        kind: TimelineRootCause::Period,
                    },
                });
                continue;
            }

            let target = self.select_basic_attack_target(unit_id, unit.current_target, None);
            let Some(target_id) = target else {
                if let Some(unit) = self.units.get_mut(&unit_id) {
                    unit.current_target = None;
                }
                continue;
            };

            if let Some(unit) = self.units.get_mut(&unit_id) {
                unit.pending_basic_attack = false;
            }

            self.event_queue.push(BattleEvent::AttackStart {
                time_ms: now_ms,
                attacker_instance_id: unit_id,
                target_instance_id: Some(target_id),
                schedule_next: true,
                cause: TimelineCause::Root {
                    kind: TimelineRootCause::Period,
                },
            });
        }
    }

    pub(super) fn build_skill_step_commands(
        caster_instance_id: UnitInstanceId,
        step: &SkillStepDef,
        targets: &[UnitInstanceId],
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
                    for target_id in targets {
                        commands.push(BattleCommand::ApplyDamage {
                            source_id: caster_instance_id,
                            target_id: *target_id,
                            amount: damage_amount,
                            damage_type: *damage_type,
                            modifiers: step_damage_modifiers,
                            source: DamageSource::Ability,
                            minimum_damage: 0,
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
            core.record_timeline(
                execution.time_ms,
                TimelineEvent::AbilityStepTriggered {
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
                    let (commands, result) = Self::build_skill_step_commands(
                        execution.caster_instance_id,
                        execution.step,
                        &targets,
                    );
                    if !commands.is_empty() {
                        let summary = core.process_commands(commands, execution.time_ms);
                        aggregated_result.actual_damage_target_count = aggregated_result
                            .actual_damage_target_count
                            .saturating_add(summary.actual_damage_target_count);
                    }
                    aggregated_result.merge(&result);
                }

                core.finalize_skill_step(
                    execution.cast_seq,
                    execution.step_index,
                    aggregated_result,
                    execution.time_ms,
                );
            }
            DeliveryDef::Projectile {
                speed_units_per_ms,
                collision,
            } => {
                let launched_count = core.dispatch_skill_projectile_delivery(
                    execution.time_ms,
                    execution.cast_seq,
                    execution.step_index,
                    execution.caster_instance_id,
                    execution.skill,
                    execution.step,
                    execution.cast_target,
                    *speed_units_per_ms,
                    *collision,
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
            DeliveryDef::Area { area } => {
                if area.duration_ms == 0 {
                    let mut aggregated_result = SkillStepResult::default();

                    for _ in 0..repeat_count {
                        let iteration_target = core.resolve_skill_step_context(
                            execution.cast_seq,
                            execution.caster_instance_id,
                            execution.step,
                            execution.cast_target,
                        );
                        let Some((origin, impact_position, direction_hint, targets)) = core
                            .resolve_instant_area_targets(
                                execution.time_ms,
                                execution.cast_seq,
                                execution.step_index,
                                execution.caster_instance_id,
                                iteration_target,
                                area,
                            )
                        else {
                            continue;
                        };

                        let delivery_id = core.allocate_skill_area_delivery_id(
                            execution.cast_seq,
                            execution.caster_instance_id,
                            execution.time_ms,
                        );
                        let area_declared_seq = core.record_skill_area_declared(
                            execution.time_ms,
                            delivery_id,
                            execution.skill.id.clone(),
                            execution.step.id.clone(),
                            execution.caster_instance_id,
                            iteration_target,
                            origin,
                            impact_position,
                            direction_hint,
                            *area,
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

                        let (commands, result) = Self::build_skill_step_commands(
                            execution.caster_instance_id,
                            execution.step,
                            &targets,
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
                    if core.register_persistent_area(
                        execution.time_ms,
                        execution.cast_seq,
                        execution.step_index,
                        execution.caster_instance_id,
                        execution.skill.id.clone(),
                        execution.step.id.clone(),
                        iteration_target,
                        *area,
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
            BattleEvent::AttackStart {
                time_ms,
                attacker_instance_id,
                target_instance_id,
                schedule_next,
                cause,
            } => {
                let (
                    is_dead,
                    next_ready_ms,
                    can_attack,
                    lock_until,
                    base_uuid,
                    current_target,
                    interval_ms,
                ) = {
                    let Some(attacker) = self.units.get(&attacker_instance_id) else {
                        return Ok(());
                    };
                    (
                        attacker.is_dead(),
                        attacker.next_basic_attack_ms,
                        attacker.action_locks.can_basic_attack(current_time_ms),
                        attacker.action_locks.basic_attack_until_ms,
                        attacker.base_uuid,
                        attacker.current_target,
                        attacker.stats.attack_interval_ms.max(1),
                    )
                };
                if is_dead {
                    return Ok(());
                }

                if schedule_next && current_time_ms < next_ready_ms {
                    self.event_queue.push(BattleEvent::AttackStart {
                        time_ms: next_ready_ms,
                        attacker_instance_id,
                        target_instance_id,
                        schedule_next,
                        cause,
                    });
                    return Ok(());
                }

                // 행동 락(하드 CC/집중 등)으로 공격이 지연됐을 때 재스케줄링
                if !can_attack {
                    self.event_queue.push(BattleEvent::AttackStart {
                        time_ms: if schedule_next {
                            lock_until.max(next_ready_ms)
                        } else {
                            lock_until
                        },
                        attacker_instance_id,
                        target_instance_id,
                        schedule_next,
                        cause,
                    });
                    return Ok(());
                }

                let hinted_target = if schedule_next {
                    None
                } else {
                    target_instance_id
                };
                let target = self.select_basic_attack_target(
                    attacker_instance_id,
                    current_target,
                    hinted_target,
                );

                if target.is_none() {
                    if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
                        attacker.current_target = None;
                    }
                    if schedule_next {
                        if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
                            attacker.pending_basic_attack = true;
                            attacker.next_basic_attack_ms = time_ms;
                        }
                    }
                    return Ok(());
                }

                let target_id = target.unwrap();
                let stopped_movement = self.interrupt_movement(
                    time_ms,
                    attacker_instance_id,
                    crate::game::battle::timeline::MovementStopReason::AttackStarted,
                    None,
                    ActionState::Idle,
                );
                if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
                    attacker.current_target = Some(target_id);
                    attacker.pending_basic_attack = false;
                }

                let _ = stopped_movement;

                let windup_ms = self
                    .game_data
                    .abnormality_data
                    .get_by_uuid(&base_uuid)
                    .map(|m| m.basic_attack.effective_windup_ms() as u64)
                    .unwrap_or(0);
                let resolve_time = time_ms.saturating_add(windup_ms);

                if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
                    attacker.action_locks.lock_basic_attack_until(resolve_time);
                    attacker.action_locks.lock_movement_until(resolve_time);
                }

                let attack_kind = if schedule_next {
                    AttackKind::Auto
                } else {
                    AttackKind::Triggered
                };
                let attack_delivery = self.basic_attack_delivery(base_uuid);

                let start_seq = self.with_recording_context(cause, |core| {
                    core.record_timeline(
                        time_ms,
                        TimelineEvent::AttackStart {
                            attacker_instance_id,
                            target_instance_id: target_id,
                            kind: Some(attack_kind),
                            delivery: Some(attack_delivery),
                        },
                    )
                });

                self.event_queue.push(BattleEvent::AttackResolve {
                    time_ms: resolve_time,
                    attacker_instance_id,
                    target_instance_id: target_id,
                    kind: attack_kind,
                    cause: TimelineCause::Parent { seq: start_seq },
                });

                if schedule_next {
                    let next_time = time_ms.saturating_add(interval_ms);
                    if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
                        attacker.next_basic_attack_ms = next_time;
                    }
                    self.event_queue.push(BattleEvent::AttackStart {
                        time_ms: next_time,
                        attacker_instance_id,
                        target_instance_id: None,
                        schedule_next: true,
                        cause: TimelineCause::Root {
                            kind: TimelineRootCause::Period,
                        },
                    });
                }

                Ok(())
            }
            BattleEvent::AttackResolve {
                time_ms,
                attacker_instance_id,
                target_instance_id,
                kind,
                cause,
            } => {
                // TODO: Resolve all AttackResolve events with the same time_ms as one batch.
                // Sequential queue order currently lets the first lethal resolve end the battle
                // before another simultaneous attack can apply its already-started damage.
                let attacker_alive = self
                    .units
                    .get(&attacker_instance_id)
                    .is_some_and(|unit| !unit.is_dead());
                if !attacker_alive {
                    return Ok(());
                }
                let attack_delivery = self
                    .units
                    .get(&attacker_instance_id)
                    .map(|unit| self.basic_attack_delivery(unit.base_uuid))
                    .unwrap_or(crate::game::battle::timeline::AttackDelivery::Instant);

                let resolve_seq = self.with_recording_context(cause, |core| {
                    core.record_timeline(
                        time_ms,
                        TimelineEvent::AttackResolve {
                            attacker_instance_id,
                            target_instance_id,
                            kind: Some(kind),
                            delivery: Some(attack_delivery),
                        },
                    )
                });

                let hit = self.with_recording_cause(resolve_seq, |core| {
                    core.resolve_basic_attack(attacker_instance_id, target_instance_id, time_ms)
                });

                if !hit {
                    self.with_recording_cause(resolve_seq, |core| {
                        core.record_timeline(
                            time_ms,
                            TimelineEvent::AttackMiss {
                                attacker_instance_id,
                                target_instance_id,
                                kind: Some(kind),
                                delivery: Some(attack_delivery),
                            },
                        )
                    });
                }

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
                if caster.is_dead() {
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
                let blocked_until = caster.next_action_time;

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

                let (caster_owner, caster_base_uuid) = {
                    let Some(caster) = self.units.get(&caster_instance_id) else {
                        return Ok(());
                    };
                    (caster.owner, caster.base_uuid)
                };

                let skill_id = self
                    .game_data
                    .abnormality_data
                    .get_by_uuid(&caster_base_uuid)
                    .and_then(|m| m.skill_id.as_deref())
                    .filter(|id| self.game_data.skill_data.get_by_id(id).is_some())
                    .map(str::to_string);

                if skill_id.as_deref().unwrap_or("").is_empty() {
                    // TODO: 기록
                    return Ok(());
                }

                let caster_pos = match self.battlefield.position_of(caster_instance_id) {
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
                        crate::game::battle::timeline::MovementStopReason::CastStarted,
                        Some(cast_end_ms),
                        ActionState::Idle,
                    );
                    if interrupted {
                        self.schedule_continuous_movement_tick(cast_end_ms);
                    }
                }

                // Current rule: autocast may begin even when no valid cast target is found.
                // We still enter focus/recovery and spend resonance at AutoCastEnd so
                // "bad timing" remains a possible AI failure mode.
                //
                // Revisit only if playtests show this feels excessively punishing:
                // the alternative contract is to abort before PendingSkillCast is stored
                // and preserve resonance when target resolution returns None.
                let target = self.resolve_skill_cast_target(
                    &skill,
                    caster_instance_id,
                    caster_owner,
                    caster_pos,
                );

                let start_seq = self.with_recording_context(cause, |core| {
                    core.record_timeline(
                        time_ms,
                        TimelineEvent::AutoCastStart {
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
                    });
                }

                self.event_queue.push(BattleEvent::AutoCastEnd {
                    time_ms: cast_end_ms,
                    caster_instance_id,
                    cause: TimelineCause::Parent { seq: start_seq },
                });

                Ok(())
            }
            BattleEvent::AutoCastEnd {
                time_ms,
                caster_instance_id,
                cause,
            } => {
                let blocked_until = self
                    .units
                    .get(&caster_instance_id)
                    .map(|unit| unit.next_action_time)
                    .unwrap_or(0);
                if time_ms < blocked_until {
                    self.event_queue.push(BattleEvent::AutoCastEnd {
                        time_ms: blocked_until,
                        caster_instance_id,
                        cause,
                    });
                    return Ok(());
                }

                let pending = self
                    .units
                    .get_mut(&caster_instance_id)
                    .and_then(|unit| unit.pending_skill_cast.take());
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

                    core.record_timeline(
                        time_ms,
                        TimelineEvent::AutoCastEnd { caster_instance_id },
                    );
                });

                let Some(caster) = self.units.get_mut(&caster_instance_id) else {
                    return Ok(());
                };
                let recovery_ends_at =
                    time_ms.saturating_add(caster.stats.attack_interval_ms.max(1));
                caster.next_basic_attack_ms = caster.next_basic_attack_ms.max(recovery_ends_at);
                // Intentionally consumes resonance even if the pending cast had no target and
                // invoke_ability became a functional no-op. Keep this coupled with the
                // AutoCastStart rule above so the "whiff still spends" behavior stays explicit
                // in code until we decide otherwise from playtest feedback.
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
                let Some(def) = crate::game::battle::buffs::get(buff_id) else {
                    return Ok(());
                };

                if duration_ms == 0 {
                    return Ok(());
                }

                if !matches!(
                    self.units.get(&target_instance_id),
                    Some(unit) if !unit.is_dead()
                ) {
                    return Ok(());
                }

                let applied_seq = self.with_recording_context(cause, |core| {
                    core.record_timeline(
                        time_ms,
                        TimelineEvent::BuffApplied {
                            caster_instance_id,
                            target_instance_id,
                            buff_id,
                            duration_ms,
                        },
                    )
                });

                let key = BuffInstanceKey {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                };

                let expires_at_ms = time_ms.saturating_add(duration_ms);
                let max_stacks = def.max_stacks.max(1);
                let is_hard_cc = matches!(
                    def.kind,
                    crate::game::battle::buffs::BuffKind::Stun
                        | crate::game::battle::buffs::BuffKind::Freeze
                );

                // Hard CC is exclusive per target: any existing hard CC on this target is replaced.
                if is_hard_cc {
                    self.buffs.retain(|k, _| {
                        if k.target_instance_id != target_instance_id {
                            return true;
                        }
                        let Some(kdef) = crate::game::battle::buffs::get(k.buff_id) else {
                            return true;
                        };
                        !matches!(
                            kdef.kind,
                            crate::game::battle::buffs::BuffKind::Stun
                                | crate::game::battle::buffs::BuffKind::Freeze
                        )
                    });
                }

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
                            cause: TimelineCause::Parent { seq: applied_seq },
                        });
                    }
                }

                self.event_queue.push(BattleEvent::BuffExpire {
                    time_ms: effective_expires_at_ms,
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                    cause: TimelineCause::Parent { seq: applied_seq },
                });

                if is_hard_cc {
                    let lock_until = effective_expires_at_ms.saturating_add(1);
                    if let Some(unit) = self.units.get_mut(&target_instance_id) {
                        unit.next_action_time = unit.next_action_time.max(lock_until);
                        unit.action_locks.lock_movement_until(lock_until);
                        unit.action_locks.lock_basic_attack_until(lock_until);
                        unit.action_locks.lock_resonance_gain_until(lock_until);
                        unit.current_target = None;
                    }

                    self.interrupt_movement(
                        time_ms,
                        target_instance_id,
                        crate::game::battle::timeline::MovementStopReason::HardCC,
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
                let Some(def) = crate::game::battle::buffs::get(buff_id) else {
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
                    core.record_timeline(
                        time_ms,
                        TimelineEvent::BuffTick {
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
                            core.process_commands(
                                vec![BattleCommand::ApplyDamage {
                                    source_id: caster_instance_id,
                                    target_id: target_instance_id,
                                    amount: dmg as u32,
                                    damage_type,
                                    modifiers: Default::default(),
                                    source: DamageSource::BuffTick,
                                    minimum_damage: 0,
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
                let Some(def) = crate::game::battle::buffs::get(buff_id) else {
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
                    core.record_timeline(
                        time_ms,
                        TimelineEvent::BuffExpired {
                            caster_instance_id,
                            target_instance_id,
                            buff_id,
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
                if self
                    .last_continuous_movement_tick_ms
                    .is_some_and(|last_time_ms| {
                        time_ms < last_time_ms.saturating_add(DEFAULT_MOVEMENT_TICK_MS)
                    })
                {
                    return Ok(());
                }

                self.last_continuous_movement_tick_ms = Some(time_ms);
                self.run_continuous_attack_movement_tick(time_ms, DEFAULT_MOVEMENT_TICK_MS);
                self.try_start_pending_basic_attacks(time_ms);
                let next_time_ms = time_ms.saturating_add(DEFAULT_MOVEMENT_TICK_MS);
                if next_time_ms <= MAX_BATTLE_TIME_MS {
                    self.schedule_continuous_movement_tick(next_time_ms);
                }
                Ok(())
            }
        }
    }

    pub(in crate::game::battle::core) fn is_point_on_skill_line(
        caster_pos: Position,
        anchor: Position,
        candidate: Position,
        length_tiles: i32,
        caster_owner: Side,
    ) -> bool {
        let step_x = (anchor.x - caster_pos.x).signum();
        let mut step_y = (anchor.y - caster_pos.y).signum();
        if step_x == 0 && step_y == 0 {
            step_y = match caster_owner {
                Side::Player => -1,
                Side::Opponent => 1,
            };
        }

        for step in 1..=length_tiles.max(0) {
            let point = Position::new(caster_pos.x + step_x * step, caster_pos.y + step_y * step);
            if point == candidate {
                return true;
            }
        }

        false
    }
}
