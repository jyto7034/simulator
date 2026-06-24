use std::collections::HashSet;

use uuid::Uuid;

use crate::game::{
    battle::{
        event_log::{
            BattleEventCause, BattleEventRootCause, BattleLogEvent, BuffExpireReason,
            SkillCastCancelReason,
        },
        ids::UnitInstanceId,
        scenario::{ScenarioGroupId, ScenarioSpawnGroup, ScenarioUnitRef, ScenarioUnitSpawn},
        types::BattleUnitDraft,
    },
    behavior::GameError,
    enums::Side,
    resources::Position,
};

use super::{
    movement::{types::UnitBody, ActionState},
    BattleCore, RuntimeArtifact, RuntimeItem, RuntimeUnit, RuntimeUnitLifecycle,
};

impl BattleCore {
    pub(super) fn build_runtime_artifacts_from_scenario(&mut self) -> Result<(), GameError> {
        let mut seen_artifacts: HashSet<(crate::game::enums::Side, Uuid)> = HashSet::new();
        for artifact in &self.scenario.artifacts {
            if !seen_artifacts.insert((artifact.side, artifact.base_uuid)) {
                return Err(GameError::InvalidAction);
            }
            let instance_id = Self::make_artifact_instance_id(
                artifact.base_uuid,
                artifact.side,
                artifact.instance_salt,
            );
            if self
                .artifacts
                .insert(
                    instance_id,
                    RuntimeArtifact {
                        instance_id,
                        owner: artifact.side,
                        base_uuid: artifact.base_uuid,
                    },
                )
                .is_some()
            {
                return Err(GameError::InvalidAction);
            }
        }

        Ok(())
    }

    pub(super) fn spawn_scenario_group(
        &mut self,
        group_id: &crate::game::battle::scenario::ScenarioGroupId,
    ) -> Result<Vec<UnitInstanceId>, GameError> {
        if self.scenario_runtime.spawned_groups.contains(group_id) {
            return Ok(Vec::new());
        }
        let group = self
            .scenario
            .group(group_id)
            .cloned()
            .ok_or(GameError::InvalidAction)?;

        let artifact_base_uuids: Vec<Uuid> = self
            .scenario
            .artifacts
            .iter()
            .filter(|artifact| artifact.side == group.side)
            .map(|artifact| artifact.base_uuid)
            .collect();

        let mut spawned_unit_ids = Vec::new();
        for spawn in &group.spawns {
            let unit = &spawn.draft;
            let combat_profile = unit.combat_profile(&self.game_data)?;
            let stats = unit.effective_stats(&self.game_data, &artifact_base_uuids)?;

            let base_uuid = unit.base_uuid();
            let source_identity = unit
                .source
                .source_identity(unit.owned_uuid, &self.game_data)?;
            let instance_id = Self::make_instance_id(base_uuid, group.side, spawn.instance_salt);
            let spawn_order = self.scenario_runtime.next_spawn_order;
            self.scenario_runtime.next_spawn_order =
                self.scenario_runtime.next_spawn_order.saturating_add(1);
            let resonance_start = combat_profile.resonance.start;
            let resonance_max = combat_profile.resonance.max.max(1);
            let resonance_lock_ms = combat_profile.resonance.gain_lock_ms;
            let resonance_current = resonance_start.min(resonance_max);
            let move_speed_units_per_ms = stats.move_speed_units_per_ms;
            let placed_position =
                self.place_scenario_spawn(instance_id, spawn.position, group.side)?;
            let spawn_position =
                super::movement::types::WorldVec2::from_tile_center(placed_position);
            let radius = combat_profile.movement.radius_units as f32
                / super::movement::types::DATA_UNITS_PER_WORLD;
            let body = UnitBody::new_at(
                spawn_position,
                radius,
                move_speed_units_per_ms as f32 * 1_000.0
                    / super::movement::types::DATA_UNITS_PER_WORLD,
            );
            let tactical_anchor = Some(spawn_position);
            let (block_capacity, block_radius_units) = runtime_blocking_stats(&combat_profile);
            if self
                .units
                .insert(
                    instance_id,
                    RuntimeUnit {
                        instance_id,
                        lifecycle: RuntimeUnitLifecycle::Active,
                        spawn_order,
                        source_owned_uuid: unit.owned_uuid,
                        owner: group.side,
                        role: unit.source.role(),
                        threat_class: unit.threat_class,
                        base_uuid,
                        source_identity,
                        stats,
                        incoming_damage_modifiers: combat_profile.incoming_damage_modifiers,
                        basic_attack: combat_profile.basic_attack,
                        skill_id: combat_profile.skill_id,
                        skill_activation_mode: combat_profile.skill_activation_mode,
                        body,
                        tactical_anchor,
                        enemy_movement_plan: group.enemy_movement_plan.clone(),
                        block_capacity,
                        block_radius_units,
                        blockable: combat_profile.blockable,
                        mobility_kind: combat_profile.mobility_kind,
                        target_traits: combat_profile.target_traits.clone(),
                        facing_direction: (group.side == Side::Player)
                            .then_some(crate::game::battle::tile_range::FacingDirection::Right),
                        move_epoch: 0,
                        action_state: ActionState::Idle,
                        action_locks: Default::default(),
                        current_target: None,
                        next_basic_attack_ms: 0,
                        pending_basic_attack: false,
                        ranged_reposition_until_ms: 0,
                        resonance_current,
                        resonance_max,
                        resonance_lock_ms,
                        next_action_time: 0,
                        pending_cast: false,
                        pending_cast_cause: None,
                        pending_skill_cast: None,
                    },
                )
                .is_some()
            {
                return Err(GameError::InvalidAction);
            }
            self.scenario_runtime
                .unit_refs
                .insert(spawn.unit_ref.clone(), instance_id);
            spawned_unit_ids.push(instance_id);

            for (idx, equipment_uuid) in unit.equipped_items.iter().enumerate() {
                let item_instance_id = Self::make_item_instance_id(
                    *equipment_uuid,
                    group.side,
                    instance_id,
                    idx as u32,
                );
                self.items.insert(
                    item_instance_id,
                    RuntimeItem {
                        instance_id: item_instance_id,
                        owner: group.side,
                        owner_unit_instance: instance_id,
                        base_uuid: *equipment_uuid,
                    },
                );
            }
        }

        self.scenario_runtime
            .spawned_groups
            .insert(group_id.clone());
        Ok(spawned_unit_ids)
    }

    fn place_scenario_spawn(
        &mut self,
        instance_id: UnitInstanceId,
        preferred_position: Position,
        _side: Side,
    ) -> Result<Position, GameError> {
        let _ = instance_id;
        self.battlefield.ensure_walkable_tile(preferred_position)?;
        Ok(preferred_position)
    }

    pub(in crate::game::battle::core) fn deploy_player_unit(
        &mut self,
        draft: BattleUnitDraft,
        position: Position,
        facing: crate::game::battle::tile_range::FacingDirection,
        instance_salt: u32,
        time_ms: u64,
        current_hp_policy: Option<super::sim::BattleDeployCurrentHpPolicy>,
    ) -> Result<UnitInstanceId, GameError> {
        let employee_uuid = draft.owned_uuid;
        let group_id = ScenarioGroupId::new(format!("live_player_deploy_{instance_salt}"));
        let unit_ref = ScenarioUnitRef::new(format!("{}_0", group_id.0));
        self.scenario.groups.push(ScenarioSpawnGroup {
            id: group_id.clone(),
            side: Side::Player,
            required_for_victory: false,
            enemy_movement_plan: None,
            spawns: vec![ScenarioUnitSpawn {
                unit_ref,
                side: Side::Player,
                draft,
                position,
                instance_salt,
            }],
        });

        let spawned_unit_ids = self.spawn_scenario_group(&group_id)?;
        let unit_id = spawned_unit_ids
            .first()
            .copied()
            .ok_or(GameError::InvalidAction)?;
        if let Some(unit) = self.units.get_mut(&unit_id) {
            unit.facing_direction = Some(facing);
            if let Some(current_hp_policy) = current_hp_policy {
                unit.stats.current_health =
                    current_hp_policy.current_hp_for_max(unit.stats.max_health);
            }
        }
        let actual_position = self
            .unit_projected_tile(unit_id)
            .ok_or(GameError::InvalidAction)?;
        self.record_spawned_units(
            time_ms,
            BattleEventCause::Root {
                kind: BattleEventRootCause::System,
            },
            &spawned_unit_ids,
        );
        self.record_event_log(
            time_ms,
            BattleLogEvent::UnitDeployed {
                employee_uuid,
                unit_instance_id: unit_id,
                position: actual_position,
                facing,
            },
        );
        self.schedule_initial_attack_for_unit(unit_id, time_ms);
        Ok(unit_id)
    }

    pub(in crate::game::battle::core) fn withdraw_unit(
        &mut self,
        unit_id: UnitInstanceId,
        time_ms: u64,
    ) -> Result<(), GameError> {
        let Some(unit) = self.units.get_mut(&unit_id) else {
            return Err(GameError::UnitNotFound);
        };
        if !unit.lifecycle.is_active() {
            return Err(GameError::InvalidAction);
        }

        unit.lifecycle = RuntimeUnitLifecycle::Withdrawn;
        unit.move_epoch = unit.move_epoch.wrapping_add(1);
        unit.action_state = ActionState::Idle;
        unit.action_locks = Default::default();
        unit.current_target = None;
        unit.pending_basic_attack = false;
        unit.pending_cast = false;
        unit.pending_cast_cause = None;
        let pending_skill_cast = unit.pending_skill_cast.take();
        unit.ranged_reposition_until_ms = 0;
        let world_position = unit.body.position;
        let position = unit.body.projected_tile();

        self.items
            .retain(|_, item| item.owner_unit_instance != unit_id);
        self.active_movement_segments.remove(&unit_id);
        for unit in self.units.values_mut() {
            if unit.current_target == Some(unit_id) {
                unit.current_target = None;
            }
        }
        self.block_state = Default::default();
        self.record_event_log(
            time_ms,
            BattleLogEvent::UnitWithdrawn {
                unit_instance_id: unit_id,
                world_position: world_position.quantized_milli(),
                position,
            },
        );
        self.clear_active_buffs_for_withdrawn_unit(time_ms, unit_id);
        self.record_pending_skill_cast_cancelled_for_withdrawn_unit(
            time_ms,
            unit_id,
            pending_skill_cast,
        );
        self.cancel_active_skill_runtime_for_withdrawn_unit(time_ms, unit_id);
        Ok(())
    }

    pub(super) fn build_runtime_field(&mut self) -> Result<(), GameError> {
        for unit in self.units.values() {
            if !unit.lifecycle.is_active() {
                continue;
            }
            self.battlefield
                .ensure_walkable_tile(unit.body.projected_tile())?;
        }
        Ok(())
    }

    fn clear_active_buffs_for_withdrawn_unit(
        &mut self,
        time_ms: u64,
        withdrawn_unit_id: UnitInstanceId,
    ) {
        let mut expired = self
            .buffs
            .keys()
            .copied()
            .filter_map(|key| {
                if key.target_instance_id == withdrawn_unit_id {
                    Some((key, BuffExpireReason::TargetWithdrawn))
                } else if key.caster_instance_id == withdrawn_unit_id {
                    Some((key, BuffExpireReason::CasterWithdrawn))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        expired.sort_by(|(left_key, left_reason), (right_key, right_reason)| {
            left_key
                .target_instance_id
                .cmp(&right_key.target_instance_id)
                .then_with(|| {
                    left_key
                        .caster_instance_id
                        .cmp(&right_key.caster_instance_id)
                })
                .then_with(|| left_key.buff_id.as_u64().cmp(&right_key.buff_id.as_u64()))
                .then_with(|| (*left_reason as u8).cmp(&(*right_reason as u8)))
        });
        for (key, reason) in expired {
            if self.buffs.remove(&key).is_none() {
                continue;
            }
            self.record_event_log(
                time_ms,
                BattleLogEvent::BuffExpired {
                    caster_instance_id: key.caster_instance_id,
                    target_instance_id: key.target_instance_id,
                    buff_id: key.buff_id,
                    reason,
                },
            );
        }
    }

    fn cancel_active_skill_runtime_for_withdrawn_unit(
        &mut self,
        time_ms: u64,
        withdrawn_unit_id: UnitInstanceId,
    ) {
        let mut cancelled_casts = self
            .active_skill_casts
            .iter()
            .filter_map(|(cast_seq, cast)| {
                (cast.caster_instance_id == withdrawn_unit_id)
                    .then_some((*cast_seq, cast.skill_id.clone()))
            })
            .collect::<Vec<_>>();
        cancelled_casts.sort_by_key(|(cast_seq, _)| *cast_seq);

        for (cast_seq, skill_id) in cancelled_casts {
            if self.active_skill_casts.remove(&cast_seq).is_none() {
                continue;
            }
            self.active_areas
                .retain(|_, area| area.cast_seq != cast_seq);
            self.active_projectiles
                .retain(|_, projectile| projectile.cast_seq != cast_seq);
            self.record_event_log(
                time_ms,
                BattleLogEvent::SkillCastCancelled {
                    caster_instance_id: withdrawn_unit_id,
                    interrupted_skill_id: skill_id,
                    interrupted_cast_seq: cast_seq,
                    reason: SkillCastCancelReason::Withdrawn,
                },
            );
        }
    }

    fn record_pending_skill_cast_cancelled_for_withdrawn_unit(
        &mut self,
        time_ms: u64,
        withdrawn_unit_id: UnitInstanceId,
        pending_skill_cast: Option<crate::game::battle::core::types::PendingSkillCast>,
    ) {
        let Some(pending) = pending_skill_cast else {
            return;
        };
        self.record_event_log(
            time_ms,
            BattleLogEvent::SkillCastCancelled {
                caster_instance_id: withdrawn_unit_id,
                interrupted_skill_id: pending.skill_id,
                interrupted_cast_seq: pending.start_seq,
                reason: SkillCastCancelReason::Withdrawn,
            },
        );
    }
}

fn runtime_blocking_stats(
    combat_profile: &crate::game::battle::types::UnitCombatProfile,
) -> (u32, f32) {
    if matches!(
        combat_profile.deployment_affinity,
        crate::game::battle::types::DeploymentAffinity::PlatformOnly
    ) {
        (0, 0.0)
    } else {
        (
            combat_profile.block_capacity,
            combat_profile.block_radius_units,
        )
    }
}
