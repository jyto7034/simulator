use crate::game::{
    ability::SkillAreaTracking,
    battle::{
        event_log::{
            BattleEventCause, BattleEventRootCause, BattleLogEvent,
            SkillCastCancelReason,
        },
        ids::UnitInstanceId,
        scenario::{ScenarioGroupId, ScenarioSpawnGroup, ScenarioUnitRef, ScenarioUnitSpawn},
        types::{BattleEquipmentBinding, BattleUnitDraft, TacticalTileKind},
    },
    behavior::GameError,
    enums::Side,
    resources::{item_slot::EquipmentSlotAddress, Position},
};

use super::{
    movement::{
        types::{FiniteWorldRadius, FiniteWorldSpeed, UnitBody},
        ActionState,
    },
    types::{BattleEquipmentSlots, RuntimeEquipmentSlot},
    BattleCore, RuntimeItem, RuntimeUnit, RuntimeUnitLifecycle, UnitBuffRemovalCause,
};

impl BattleCore {
    pub(super) fn spawn_scenario_group(
        &mut self,
        group_id: &crate::game::battle::scenario::ScenarioGroupId,
    ) -> Result<Vec<UnitInstanceId>, GameError> {
        if self
            .scenario_runtime
            .spawned
            .spawned_groups
            .contains(group_id)
        {
            return Ok(Vec::new());
        }
        self.prepare_runtime_resources()?;
        let group = self
            .scenario
            .group(group_id)
            .cloned()
            .ok_or(GameError::InvalidAction)?;

        let mut spawned_unit_ids = Vec::new();
        for spawn in &group.spawns {
            spawned_unit_ids.push(self.spawn_scenario_unit(&group, spawn)?);
        }

        for instance_id in &spawned_unit_ids {
            if let Some(resources) = self.runtime_facts.resources.as_mut() {
                resources.deploy(*instance_id)?;
            }
        }
        self.scenario_runtime
            .spawned_for_write()?
            .spawned_groups
            .insert(group_id.clone());
        Ok(spawned_unit_ids)
    }

    /// Single-member commit primitive used only by the shared atomic Spawn
    /// Kernel. Legacy non-Wave scenario groups keep the batch helper above.
    pub(in crate::game::battle::core) fn spawn_scenario_group_member(
        &mut self,
        group: &ScenarioSpawnGroup,
        spawn_index: usize,
    ) -> Result<UnitInstanceId, GameError> {
        self.prepare_runtime_resources()?;
        let spawn = group
            .spawns
            .get(spawn_index)
            .ok_or(GameError::InvalidAction)?;
        let instance_id = self.spawn_scenario_unit(group, spawn)?;
        if let Some(resources) = self.runtime_facts.resources.as_mut() {
            resources.deploy(instance_id)?;
        }
        Ok(instance_id)
    }

    // The caller owns deployment scope timing: ordinary groups open scopes
    // after the whole group is built; Spawn Kernel callbacks open each member's.
    fn spawn_scenario_unit(
        &mut self,
        group: &ScenarioSpawnGroup,
        spawn: &ScenarioUnitSpawn,
    ) -> Result<UnitInstanceId, GameError> {
        let unit = &spawn.draft;
        let combat_profile = unit.combat_profile(&self.game_data)?;
        let stats = self.relic_penalized_stats(unit)?;
        let base_uuid = unit.base_uuid();
        let source_identity = unit
            .source
            .source_identity(unit.owned_uuid, &self.game_data)?;
        let instance_id = Self::make_instance_id(base_uuid, group.side, spawn.instance_salt);
        let equipment_slots = self.prepare_battle_equipment_slots(unit, group.side, instance_id)?;
        let move_speed_units_per_ms = stats.move_speed_units_per_ms;
        let finite_radius = FiniteWorldRadius::try_new(
            combat_profile.movement.radius_units as f32
                / super::movement::types::DATA_UNITS_PER_WORLD,
        )
        .map_err(|error| {
            GameError::InvalidStaticData(format!("battle unit movement input is invalid: {error}"))
        })?;
        let finite_speed = FiniteWorldSpeed::try_new(
            move_speed_units_per_ms as f32 * 1_000.0 / super::movement::types::DATA_UNITS_PER_WORLD,
        )
        .map_err(|error| {
            GameError::InvalidStaticData(format!("battle unit movement input is invalid: {error}"))
        })?;
        let spawn_order = self.scenario_runtime.next_spawn_order;
        self.scenario_runtime.next_spawn_order =
            self.scenario_runtime.next_spawn_order.saturating_add(1);
        let resonance_start = combat_profile.resonance.start;
        let resonance_max = combat_profile.resonance.max.max(1);
        let resonance_lock_ms = combat_profile.resonance.gain_lock_ms;
        let resonance_current = resonance_start.min(resonance_max);
        let placed_position = self.place_scenario_spawn(instance_id, spawn.position, group.side)?;
        let spawn_position = super::movement::types::WorldVec2::from_tile_center(placed_position);
        let body = UnitBody::from_finite(spawn_position, finite_radius, finite_speed);
        let (block_capacity, block_radius_units) = runtime_blocking_stats(&combat_profile);
        if self
            .insert_runtime_unit(
                instance_id,
                RuntimeUnit {
                    employee_trauma: unit
                        .employee_start_condition
                        .map(|condition| condition.trauma),
                    corroded: false,
                    instance_id,
                    lifecycle: RuntimeUnitLifecycle::Active,
                    spawn_order,
                    source_owned_uuid: unit.owned_uuid,
                    owner: group.side,
                    role: unit.source.role(),
                    threat_class: unit.threat_class,
                    base_uuid,
                    source_identity,
                    equipment_slots: equipment_slots.clone(),
                    stats,
                    basic_attack_speed_percent: combat_profile.basic_attack_speed_percent,
                    incoming_damage_modifiers: combat_profile.incoming_damage_modifiers,
                    basic_attack: combat_profile.basic_attack,
                    fragment_source: combat_profile
                        .fragment_source
                        .map(|source| source.into_shared())
                        .transpose()?,
                    skill_id: combat_profile.skill_id,
                    skill_activation_mode: combat_profile.skill_activation_mode,
                    body,
                    tactical_anchor: Some(spawn_position),
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
                    basic_attack_epoch: 0,
                    basic_attack_windup_until_ms: None,
                    ranged_reposition_until_ms: 0,
                    resonance_current,
                    resonance_max,
                    resonance_lock_ms,
                    resonance_use: None,
                    next_resonance_pulse_ms: None,
                    equipment_resonance_gain: None,
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
            .spawned_for_write()?
            .unit_refs
            .insert(spawn.unit_ref.clone(), instance_id);

        for (index, binding) in unit.equipped_items.iter().enumerate() {
            let item_instance_id = Self::prepared_item_instance_id(
                equipment_slots.as_deref(),
                binding,
                group.side,
                instance_id,
                index,
            )?;
            self.runtime_facts.register_provider(
                crate::game::battle::runtime_facts::EquipmentSourceSnapshot {
                    definition_uuid: binding.base_uuid,
                    runtime_item_instance_id: item_instance_id,
                    persistent_copy_uuid: binding.persistent_copy_uuid,
                },
                Some(instance_id),
            )?;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(std::mem::size_of::<Option<uuid::Uuid>>()),
                "item_copy_binding",
            )?;
            self.items.insert(
                item_instance_id,
                RuntimeItem {
            weapon_stage: binding.weapon_stage,
                    instance_id: item_instance_id,
                    owner: group.side,
                    owner_unit_instance: instance_id,
                    base_uuid: binding.base_uuid,
                    persistent_copy_uuid: binding.persistent_copy_uuid,
                },
            );
        }
        Ok(instance_id)
    }

    pub(super) fn prepare_battle_equipment_slots(
        &self,
        draft: &BattleUnitDraft,
        side: Side,
        owner_unit_instance: UnitInstanceId,
    ) -> Result<Option<std::sync::Arc<BattleEquipmentSlots>>, GameError> {
        let mut copy_uuids = std::collections::HashSet::new();
        for binding in &draft.equipped_items {
            if let Some(copy_uuid) = binding.persistent_copy_uuid {
                if copy_uuid.is_nil() {
                    return Err(GameError::InvalidStaticData(
                        "battle equipment binding has a nil actual-copy UUID".to_string(),
                    ));
                }
                if !copy_uuids.insert(copy_uuid) {
                    return Err(GameError::InvalidStaticData(format!(
                        "battle equipment actual-copy UUID {copy_uuid} is duplicated"
                    )));
                }
            }
        }

        let Some(layout) = draft.equipment_layout else {
            if draft
                .equipped_items
                .iter()
                .any(|binding| binding.slot.is_some())
            {
                return Err(GameError::InvalidStaticData(
                    "battle equipment slot address requires an equipment layout".to_string(),
                ));
            }
            return Ok(None);
        };

        let mut slots: [Option<RuntimeEquipmentSlot>; 3] = [None; 3];
        for (equipped_index, binding) in draft.equipped_items.iter().enumerate() {
            let address = binding.slot.ok_or_else(|| {
                GameError::InvalidStaticData(
                    "battle equipment binding is missing its slot address".to_string(),
                )
            })?;
            if !address.is_valid() || !address.belongs_to(layout) {
                return Err(GameError::InvalidStaticData(format!(
                    "battle equipment slot {address:?} is invalid for layout {layout:?}"
                )));
            }
            let index = equipment_slot_index(address);
            let salt = u32::try_from(equipped_index).map_err(|_| {
                GameError::InvalidStaticData(
                    "battle equipment source index does not fit the runtime item ID salt"
                        .to_string(),
                )
            })?;
            let runtime_item_instance_id =
                Self::make_item_instance_id(binding.base_uuid, side, owner_unit_instance, salt);
            if slots[index]
                .replace(RuntimeEquipmentSlot {
                    binding: *binding,
                    runtime_item_instance_id,
                })
                .is_some()
            {
                return Err(GameError::InvalidStaticData(format!(
                    "battle equipment slot {address:?} is duplicated"
                )));
            }
        }

        let current = self.retained_runtime_fact_extent()?;
        let next = current.checked_add(BattleEquipmentSlots::retained_extent());
        crate::game::battle::runtime_control::admit_retained_growth(
            current,
            next,
            "battle_equipment_slots",
        )?;
        Ok(Some(std::sync::Arc::new(BattleEquipmentSlots {
            layout,
            slots,
        })))
    }

    fn prepared_item_instance_id(
        equipment_slots: Option<&BattleEquipmentSlots>,
        binding: &BattleEquipmentBinding,
        side: Side,
        owner_unit_instance: UnitInstanceId,
        equipped_index: usize,
    ) -> Result<uuid::Uuid, GameError> {
        let salt = u32::try_from(equipped_index).map_err(|_| {
            GameError::InvalidStaticData(
                "battle equipment source index does not fit the runtime item ID salt".to_string(),
            )
        })?;
        let Some(slots) = equipment_slots else {
            return Ok(Self::make_item_instance_id(
                binding.base_uuid,
                side,
                owner_unit_instance,
                salt,
            ));
        };
        let address = binding.slot.ok_or_else(|| {
            GameError::InvalidStaticData(
                "prepared battle equipment binding is missing its slot address".to_string(),
            )
        })?;
        let entry = slots.slots[equipment_slot_index(address)]
            .as_ref()
            .ok_or_else(|| {
                GameError::InvalidStaticData(
                    "prepared battle equipment slot entry is missing".to_string(),
                )
            })?;
        if entry.binding != *binding {
            return Err(GameError::InvalidStaticData(
                "prepared battle equipment slot entry does not match its source binding"
                    .to_string(),
            ));
        }
        Ok(entry.runtime_item_instance_id)
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
        tile_kind: TacticalTileKind,
        facing: crate::game::battle::tile_range::FacingDirection,
        instance_salt: u32,
        time_ms: u64,
        current_hp_policy: Option<super::sim::BattleDeployCurrentHpPolicy>,
    ) -> Result<UnitInstanceId, GameError> {
        if self.units.values().any(|u| u.source_owned_uuid == draft.owned_uuid
            && u.lifecycle == RuntimeUnitLifecycle::PermanentlyDead) {
            return Err(GameError::InvalidAction);
        }
        self.validate_player_tactical_destination(position, None)?;
        self.validate_field_work_cell_destination(position, tile_kind)?;
        let employee_uuid = draft.owned_uuid;
        let weapon_profile = draft
            .combat_profile(&self.game_data)?
            .weapon_profile
            .ok_or(GameError::WeaponRequired)?;
        let group_id = ScenarioGroupId::new(format!("live_player_deploy_{instance_salt}"));
        let unit_ref = ScenarioUnitRef::new(format!("{}_0", group_id.0));
        self.scenario_for_deployment()?;
        self.admit_scenario_binding_growth(draft.equipped_items.len())?;
        self.scenario_for_deployment()?
            .groups
            .push(ScenarioSpawnGroup {
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
        self.initialize_deployed_tactical_state(unit_id, tile_kind, weapon_profile, time_ms)?;
        let actual_position = self
            .unit_projected_tile(unit_id)
            .ok_or(GameError::InvalidAction)?;
        self.record_spawned_units(
            time_ms,
            BattleEventCause::Root {
                kind: BattleEventRootCause::System,
            },
            &spawned_unit_ids,
        )?;
        self.record_event_log(
            time_ms,
            BattleLogEvent::UnitDeployed {
                employee_uuid,
                unit_instance_id: unit_id,
                position: actual_position,
                facing,
            },
        );
        self.reconcile_field_work(time_ms);
        self.start_causal_periodics(Some(time_ms))?;
        self.schedule_initial_attack_for_unit(unit_id, time_ms);
        Ok(unit_id)
    }

    pub(in crate::game::battle::core) fn withdraw_unit(
        &mut self,
        unit_id: UnitInstanceId,
        time_ms: u64,
    ) -> Result<(), GameError> {
        let Some(unit) = self.units.get(&unit_id) else {
            return Err(GameError::UnitNotFound);
        };
        if !unit.lifecycle.is_active() {
            return Err(GameError::InvalidAction);
        }
        self.settle_field_work_progress(time_ms);

        let Some(unit) = self.units.get_mut(&unit_id) else {
            return Err(GameError::UnitNotFound);
        };

        unit.lifecycle = RuntimeUnitLifecycle::Withdrawn;
        unit.move_epoch = unit.move_epoch.wrapping_add(1);
        unit.action_state = ActionState::Idle;
        unit.action_locks = Default::default();
        unit.current_target = None;
        unit.pending_basic_attack = false;
        unit.basic_attack_epoch = unit.basic_attack_epoch.wrapping_add(1);
        unit.basic_attack_windup_until_ms = None;
        unit.pending_cast = false;
        unit.pending_cast_cause = None;
        let pending_skill_cast = unit.pending_skill_cast.take();
        unit.ranged_reposition_until_ms = 0;
        let world_position = unit.body.position;
        let position = unit.body.projected_tile();

        self.cancel_causal_periodics_for(unit_id)?;

        self.runtime_facts.cancel_unreleased_basic(
            unit_id,
            crate::game::battle::runtime_facts::ActionCancelReason::Withdrawn,
            time_ms,
        )?;

        self.items
            .retain(|_, item| item.owner_unit_instance != unit_id);
        self.active_movement_segments.remove(&unit_id);
        self.tactical_states.remove(&unit_id);
        for unit in self.units.values_mut() {
            if unit.current_target == Some(unit_id) {
                unit.current_target = None;
            }
        }
        // Keep incumbent/contact authority until final field-work reconciliation
        // refreshes the relations after all withdrawal cleanup/capacity changes.
        if !self.independent_objects.objects.contains_key(&unit_id) {
            self.record_event_log(
                time_ms,
                BattleLogEvent::UnitWithdrawn {
                    unit_instance_id: unit_id,
                    world_position: world_position.quantized_milli(),
                    position,
                },
            );
        }
        self.clear_active_buffs_for_removed_unit(time_ms, unit_id, UnitBuffRemovalCause::Withdrawn);
        self.record_pending_skill_cast_cancelled_for_withdrawn_unit(
            time_ms,
            unit_id,
            pending_skill_cast,
        );
        self.cancel_active_skill_runtime_for_withdrawn_unit(time_ms, unit_id)?;
        self.cancel_resonance_use_control(unit_id);
        if let Some(unit) = self.units.get_mut(&unit_id) {
            unit.resonance_use = None;
            unit.next_resonance_pulse_ms = None;
            unit.equipment_resonance_gain = None;
        }
        self.cancel_uses_with_unavailable_targets(time_ms)?;
        if let Some(resources) = self.runtime_facts.resources.as_mut() {
            resources.end_deployment(unit_id)?;
        }
        self.reconcile_field_work(time_ms);
        self.reconcile_paired_areas(time_ms)?;
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

    fn cancel_active_skill_runtime_for_withdrawn_unit(
        &mut self,
        time_ms: u64,
        withdrawn_unit_id: UnitInstanceId,
    ) -> Result<(), GameError> {
        let mut cancelled_casts = self
            .active_skill_casts
            .iter()
            .filter_map(|(cast_seq, cast)| {
                (cast.caster_instance_id == withdrawn_unit_id).then_some((
                    *cast_seq,
                    cast.ability_event_seq,
                    cast.skill_id.clone(),
                ))
            })
            .collect::<Vec<_>>();
        cancelled_casts.sort_by_key(|(cast_seq, _, _)| *cast_seq);

        for (cast_seq, ability_event_seq, skill_id) in cancelled_casts {
            if !self.cancel_structural_side_cast_for_withdrawal(cast_seq, time_ms)? {
                let Some(cast) = self.active_skill_casts.remove(&cast_seq) else {
                    continue;
                };
                self.cancel_pending_action(
                    cast.fact_source.action.as_ref(),
                    time_ms,
                    crate::game::battle::runtime_facts::ActionCancelReason::Withdrawn,
                );
            }
            self.record_event_log(
                time_ms,
                BattleLogEvent::SkillCastCancelled {
                    caster_instance_id: withdrawn_unit_id,
                    interrupted_skill_id: skill_id,
                    interrupted_cast_seq: ability_event_seq,
                    reason: SkillCastCancelReason::Withdrawn,
                },
            );
        }

        let facts = &mut self.runtime_facts;
        self.active_areas.retain(|_, area| {
            let keep = area.caster_instance_id != withdrawn_unit_id
                || matches!(area.tracking, SkillAreaTracking::GroundFixed);
            if !keep {
                if let Some(ticket) = &area.source_snapshot.fact_source.delivery {
                    let _ = facts.close_delivery(ticket, time_ms);
                }
            }
            keep
        });
        Ok(())
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
        self.cancel_pending_action(
            pending.action.as_ref(),
            time_ms,
            crate::game::battle::runtime_facts::ActionCancelReason::Withdrawn,
        );
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

fn equipment_slot_index(address: EquipmentSlotAddress) -> usize {
    match address {
        EquipmentSlotAddress::Weapon | EquipmentSlotAddress::Any3(0) => 0,
        EquipmentSlotAddress::Armor | EquipmentSlotAddress::Any3(1) => 1,
        EquipmentSlotAddress::Accessory | EquipmentSlotAddress::Any3(2) => 2,
        EquipmentSlotAddress::Any3(_) => unreachable!("validated equipment slot address"),
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
