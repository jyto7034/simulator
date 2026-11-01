//! Same-owner form changes share the creation transaction and request ledger.
use std::sync::Arc;

use super::{
    independent_objects::ObjectFamilyKey, spawn_kernel::IndependentSpawnIdentity, BattleCore,
};
use crate::game::{
    battle::{enums::BattleEvent, ids::UnitInstanceId, types::BattleUnitSourceIdentity},
    behavior::GameError,
    data::independent_object_data::{CompiledIndependentObject, IndependentObjectKind},
    resources::Position,
    stats::UnitStats,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectEvolutionReceipt {
    pub object_id: UnitInstanceId,
    pub creator: UnitInstanceId,
    pub previous_definition: crate::game::data::independent_object_data::IndependentObjectId,
    pub definition: crate::game::data::independent_object_data::IndependentObjectId,
    pub epoch: u64,
    pub time_ms: u64,
    pub previous_max_health: Option<u32>,
    pub max_health: Option<u32>,
}
crate::authoring::clone_shape::struct_shape!(ObjectEvolutionReceipt {
    object_id,
    creator,
    previous_definition,
    definition,
    epoch,
    time_ms,
    previous_max_health,
    max_health
});

impl BattleCore {
    pub(super) fn independent_evolution_candidate(
        &self,
        definition: &CompiledIndependentObject,
        family: &ObjectFamilyKey,
        position: Position,
        enabled: bool,
    ) -> Result<Option<(UnitInstanceId, Arc<CompiledIndependentObject>)>, GameError> {
        if !enabled {
            return Ok(None);
        }
        let next = definition
            .definition
            .evolves_to
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        let next = self
            .game_data
            .boss_runtime_data
            .independent_objects()
            .get(next)
            .ok_or(GameError::InvalidAction)?;
        let count = self.independent_objects.objects.len() as u64;
        crate::authoring::control::charge(count, "object_evolution_match")?;
        let object = self.independent_objects.objects.values().find(|object| {
            object.definition == definition.definition.id
                && &object.family == family
                && self
                    .units
                    .get(&object.id)
                    .map_or(object.position == position, |unit| {
                        unit.is_active() && unit.body.projected_tile() == position
                    })
        });
        Ok(object.map(|object| (object.id, Arc::clone(next))))
    }

    pub(super) fn evolve_independent_object(
        &mut self,
        id: UnitInstanceId,
        definition: &CompiledIndependentObject,
        input: &IndependentSpawnIdentity,
        source_stats: UnitStats,
    ) -> Result<(), GameError> {
        let old = self
            .independent_objects
            .objects
            .get(&id)
            .ok_or(GameError::InvalidAction)?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            old.retained_extent()
                .and_then(|n| n.checked_add(old.definition.0.len())),
            "object_evolution_snapshot",
        )?;
        input.source.admit_owner("object_evolution_new_source")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(definition.definition.id.0.len() * 2),
            "object_evolution_definition",
        )?;
        let old_hp = self.units.get(&id).map(|unit| unit.stats.current_health);
        let old_max = self.units.get(&id).map(|unit| unit.stats.max_health);
        let previous_definition = crate::authoring::clone_shape::controlled_clone(&old.definition)?;
        let mut object = crate::authoring::clone_shape::controlled_clone(old)?;
        object.epoch = object
            .epoch
            .checked_add(1)
            .ok_or(GameError::InvalidAction)?;
        object.created_at_ms = input.time_ms;
        object.expires_at_ms = input
            .time_ms
            .checked_add(u64::from(definition.definition.ttl_ms))
            .ok_or(GameError::InvalidAction)?;
        object.definition =
            crate::authoring::clone_shape::controlled_clone(&definition.definition.id)?;
        object.source = crate::authoring::clone_shape::controlled_clone(&input.source)?;
        object.creation_action = input.creation_action;
        object.source_attack = source_stats.attack;
        object.source_defense = source_stats.defense;
        object.source_magic_resistance = source_stats.magic_resist;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            object.retained_extent(),
            "object_evolution_owner",
        )?;
        if !matches!(definition.definition.kind, IndependentObjectKind::Trap(_)) {
            let old_hp = self
                .units
                .get(&id)
                .filter(|unit| unit.is_active())
                .ok_or(GameError::InvalidAction)?
                .stats
                .current_health;
            let (mut profile, kind, movement) = self.independent_adapter_profile(
                definition,
                source_stats.defense,
                source_stats.magic_resist,
            )?;
            profile.stats.current_health = old_hp.min(profile.stats.max_health);
            self.validate_independent_form_overlays(id, &profile.basic_attack)?;
            // This shared cancellation invalidates unreleased attack/cast/move
            // epochs. Released carriers keep their own snapshots and leases.
            self.cancel_boss_owned_work(input.time_ms, id, true, true);
            self.stop_channel_with_reason(
                id,
                input.time_ms,
                crate::game::ability::sustained::SustainedEndReason::Interrupted,
            )?;
            self.event_queue.discard_independent_form_starts(id)?;
            let cast_count = self.active_skill_casts.len() as u64;
            crate::authoring::control::charge(cast_count, "object_evolution_cast_scan")?;
            let mut cursor = None;
            loop {
                crate::authoring::control::charge(cast_count, "object_evolution_cast_next")?;
                let seq = self
                    .active_skill_casts
                    .iter()
                    .filter(|(seq, cast)| {
                        cast.caster_instance_id == id && cursor.is_none_or(|last| **seq > last)
                    })
                    .map(|(seq, _)| *seq)
                    .min();
                let Some(seq) = seq else {
                    break;
                };
                self.event_queue.discard_skill_continuations(seq)?;
                cursor = Some(seq);
            }
            self.rebase_independent_form_sustained(id, &profile)?;
            let unit = self.units.get_mut(&id).ok_or(GameError::InvalidAction)?;
            unit.base_uuid = definition.definition.base_uuid;
            unit.source_identity = BattleUnitSourceIdentity::IndependentObject {
                definition_id: crate::authoring::clone_shape::controlled_clone(
                    &definition.definition.id.0,
                )?,
                base_uuid: definition.definition.base_uuid,
                creator_instance_id: input.creator,
                kind,
            };
            // Rebase has installed effective stats/basic action while preserving
            // external effects; the other fields belong to the new form.
            unit.basic_attack_speed_percent = profile.basic_attack_speed_percent;
            unit.incoming_damage_modifiers = profile.incoming_damage_modifiers;
            unit.skill_id = profile.skill_id;
            unit.skill_activation_mode = profile.skill_activation_mode;
            unit.enemy_movement_plan = movement;
            unit.block_capacity = profile.block_capacity;
            unit.block_radius_units = profile.block_radius_units;
            unit.blockable = profile.blockable;
            unit.mobility_kind = profile.mobility_kind;
            unit.target_traits = profile.target_traits;
            unit.body.radius =
                profile.movement.radius_units as f32 / super::movement::types::DATA_UNITS_PER_WORLD;
            unit.body.move_speed = unit.stats.move_speed_units_per_ms as f32 * 1000.0
                / super::movement::types::DATA_UNITS_PER_WORLD;
            unit.resonance_max = profile.resonance.max;
            unit.resonance_current = unit.resonance_current.min(unit.resonance_max);
            unit.resonance_lock_ms = profile.resonance.gain_lock_ms;
            unit.next_basic_attack_ms = input.time_ms;
            unit.next_action_time = input.time_ms;
            object.position = unit.body.projected_tile();
        }
        self.event_queue.discard_independent_object_expiry(id)?;
        let expires_at_ms = object.expires_at_ms;
        let epoch = object.epoch;
        let receipt = ObjectEvolutionReceipt {
            object_id: id,
            creator: input.creator,
            previous_definition,
            definition: crate::authoring::clone_shape::controlled_clone(&object.definition)?,
            epoch,
            time_ms: input.time_ms,
            previous_max_health: old_max,
            max_health: self.units.get(&id).map(|unit| unit.stats.max_health),
        };
        self.independent_objects.objects.insert(id, object);
        self.record_event_log(
            input.time_ms,
            crate::game::battle::event_log::BattleLogEvent::IndependentObjectEvolved { receipt },
        );
        if let Some(before) = old_hp {
            let after = self
                .units
                .get(&id)
                .ok_or(GameError::InvalidAction)?
                .stats
                .current_health;
            self.record_runtime_stat_hp_change_and_finalize(
                Some(input.creator),
                id,
                before,
                after,
                input.time_ms,
                None,
            );
        }
        if old_hp.is_some() {
            // Validate against the final form/HP state while retaining the
            // incumbent order and swept contacts of every other live owner.
            self.refresh_independent_form_block_state()?;
        }
        self.event_queue.push(BattleEvent::IndependentObjectExpiry {
            time_ms: expires_at_ms,
            object_id: id,
            epoch,
        });
        if self.units.contains_key(&id) {
            self.schedule_initial_attack_for_unit(id, input.time_ms);
        }
        crate::authoring::control::poll("object_evolution_complete")?;
        crate::game::battle::runtime_control::poll()
    }
}
