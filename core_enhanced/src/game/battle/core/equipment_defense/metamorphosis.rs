//! A terminal revival consumes one employee use and creates an independent enemy.
use super::*;
use crate::game::{
    battle::{
        core::{spawn_kernel::*, types::RuntimeUnitLifecycle},
        event_log::{BattleEventCause, BattleLogEvent},
        runtime_facts::DeathCause,
        scenario::{ScenarioGroupId, ScenarioSpawnGroup, ScenarioUnitRef, ScenarioUnitSpawn},
        types::{BattleUnitDraft, BattleUnitSource, BattleUnitThreatClass, UnitCombatProfile},
    },
    resources::Position,
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct EmployeeTransformationWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub actor: UnitInstanceId,
    pub employee_uuid: uuid::Uuid,
    pub successor: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub time_ms: u64,
    pub position: Position,
    pub form: String,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
crate::authoring::clone_shape::struct_shape!(EmployeeTransformationWitness {
    generation,
    actor,
    employee_uuid,
    successor,
    equipment,
    time_ms,
    position,
    form,
    definition_id,
    source_file,
    field_path
});
impl EmployeeTransformationWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>()
            .checked_add(self.form.capacity())?
            .checked_add(self.definition_id.capacity())?
            .checked_add(self.source_file.capacity())?
            .checked_add(self.field_path.capacity())?;
        if let Some(g) = &self.generation {
            bytes = bytes
                .checked_add(size_of::<crate::game::data::content_pack::ContentIdentity>())?
                .checked_add(g.semantic_sha256.capacity())?;
        }
        Some(bytes)
    }
}

impl BattleCore {
    pub(in crate::game::battle::core) fn required_transformation_alive(&self) -> bool {
        self.equipment_defense
            .required_transformations
            .iter()
            .any(|id| self.units.get(id).is_some_and(|u| u.is_active()))
    }

    pub(in crate::game::battle::core) fn resolve_employee_transformation(
        &mut self,
        actor: UnitInstanceId,
        program: Arc<CompiledEquipmentDefense>,
        source: RuntimeFactSource,
        now: u64,
    ) -> Result<(), GameError> {
        let base_uuid = program
            .transformation_uuid
            .ok_or(GameError::InvalidAction)?;
        let EquipmentDefenseProgram::EmployeeRevival {
            transform_into: Some(form),
            ..
        } = &program.program
        else {
            return Err(GameError::InvalidAction);
        };
        let body = self
            .sample_unit_body_at(actor, now)
            .ok_or(GameError::InvalidAction)?;
        let unit = self
            .units
            .get(&actor)
            .filter(|u| u.is_active())
            .ok_or(GameError::InvalidAction)?;
        let employee_uuid = unit.source_owned_uuid;
        let owner = unit.owner;
        let facing = unit
            .facing_direction
            .unwrap_or(crate::game::battle::tile_range::FacingDirection::Up);
        let position = body.projected_tile();
        let equipment = source.equipment.ok_or(GameError::InvalidAction)?;
        let meta = self
            .game_data
            .abnormality_data
            .get_by_uuid(&base_uuid)
            .ok_or(GameError::InvalidAction)?;
        // The profile constructor clones only these owned leaves. Admit those
        // copies before calling it; the Spawn Kernel owns the actual unit copy.
        let bytes = super::super::sustained::basic_extent(&meta.basic_attack)
            .and_then(|n| {
                n.checked_add(
                    meta.target_traits
                        .len()
                        .checked_mul(size_of::<crate::game::battle::types::UnitTargetTrait>())?,
                )
            })
            .and_then(|n| n.checked_add(meta.skill_id.as_ref().map_or(0, |id| id.as_str().len())))
            .and_then(|n| n.checked_add(size_of::<UnitCombatProfile>() + form.len() + 512));
        runtime_control::admit_retained_growth(0, bytes, "metamorphosis_profile")?;
        control::charge(
            bytes.ok_or(GameError::InvalidAction)? as u64,
            "metamorphosis_profile",
        )?;
        let profile = BattleUnitDraft::combat_profile_from_abnormality(base_uuid, &self.game_data)?;
        let identity = Self::make_instance_id(actor.0, Side::Opponent, 0x6d657461).0;
        let mut salt = u32::MAX;
        loop {
            control::charge(1, "metamorphosis_instance")?;
            if !self
                .units
                .contains_key(&Self::make_instance_id(base_uuid, Side::Opponent, salt))
            {
                break;
            }
            salt = salt.checked_sub(1).ok_or(GameError::InvalidAction)?;
        }
        let key = format!("employee-metamorphosis:{actor}");
        let group = ScenarioSpawnGroup {
            id: ScenarioGroupId::new(key.clone()),
            side: Side::Opponent,
            required_for_victory: false,
            enemy_movement_plan: None,
            spawns: vec![ScenarioUnitSpawn {
                unit_ref: ScenarioUnitRef::new(key.clone()),
                side: Side::Opponent,
                position,
                instance_salt: salt,
                draft: BattleUnitDraft {
                    owned_uuid: identity,
                    source: BattleUnitSource::SummonedUnit {
                        definition_id: form.clone(),
                        base_uuid,
                        profile,
                        summoner_instance_id: actor,
                        summon_sequence: 0,
                        batch_uuid: identity,
                    },
                    threat_class: BattleUnitThreatClass::Normal,
                    level: crate::game::enums::Tier::I,
                    stat_scale: Default::default(),
                    growth_stacks: Default::default(),
                    equipment_layout: None,
                    equipped_items: vec![],
                    equipped_item_enhancements: vec![],
                    employee_start_condition: None,
                },
            }],
        };
        let request = SpawnRequest {
            id: SpawnRequestId(key),
            owner: Side::Opponent,
            source: SpawnRequestSource::Summon {
                summoner: actor,
                definition_id: form.clone(),
                count: 1,
            },
            occupancy: SpawnOccupancyPolicy::AllowOccupied(vec![SpawnOccupancyClass::Any]),
            placements: vec![SpawnPlacement {
                order: 0,
                position,
                terrain_walkable: true,
                occupants: vec![],
            }],
        };
        let prior = std::mem::replace(&mut self.runtime_fact_source, source);
        let result = (|| {
            let mut prepared = self
                .prepare_death_fact(None, actor, now)?
                .ok_or(GameError::InvalidAction)?;
            prepared.cause = DeathCause::Transformation;
            self.finalize_prepared_death_with_dispatch(
                None, actor, owner, now, prepared, None, false,
            );
            runtime_control::poll()?;
            control::poll("metamorphosis_employee_exit")?;
            let unit = self
                .units
                .get_mut(&actor)
                .filter(|u| u.lifecycle.is_dead())
                .ok_or(GameError::InvalidAction)?;
            unit.lifecycle = RuntimeUnitLifecycle::PermanentlyDead;
            let settlement = self
                .execute_spawn_request(request, |core, _| {
                    core.spawn_scenario_group_member(&group, 0)
                })
                .map_err(|e| {
                    GameError::InvalidStaticData(format!(
                        "employee transformation Spawn Kernel: {e:?}"
                    ))
                })?;
            let SpawnRequestSettlement::Spawned { unit_ids } = settlement else {
                return Err(GameError::InvalidAction);
            };
            let successor = *unit_ids.first().ok_or(GameError::InvalidAction)?;
            // Tile validation is shared with every spawn; retain the exact
            // employee world position for the replacement, including offsets.
            let spawned = self
                .units
                .get_mut(&successor)
                .ok_or(GameError::InvalidAction)?;
            spawned.body.position = body.position;
            spawned.tactical_anchor = Some(body.position);
            // A stationary form must have a facing before its first attack.
            // Movement may later turn it using the ordinary enemy policy.
            spawned.facing_direction = Some(facing);
            runtime_control::admit_retained_growth(
                0,
                Some(128 + size_of::<UnitInstanceId>()),
                "metamorphosis_required_enemy",
            )?;
            self.equipment_defense
                .required_transformations
                .insert(successor);
            control::poll("metamorphosis_spawned")?;
            self.record_spawned_units(now, BattleEventCause::default(), &unit_ids)?;
            self.record_event_log(
                now,
                BattleLogEvent::EmployeeTransformed {
                    actor,
                    successor,
                    employee_uuid,
                    equipment_copy: equipment
                        .persistent_copy_uuid
                        .ok_or(GameError::InvalidAction)?,
                    world_position: body.position.quantized_milli(),
                },
            );
            self.schedule_initial_attack_for_unit(successor, now);
            self.schedule_continuous_movement_tick(now);
            self.record_lab_source(
                &self.runtime_fact_source.clone(),
                crate::mod_validation::CoverageStage::ConsumedOrEffect,
            )?;
            if self.authoring_observation.is_some() {
                let bytes = size_of::<EmployeeTransformationWitness>()
                    + form.len()
                    + program.definition_id.len()
                    + program.source_file.len()
                    + program.field_path.len();
                runtime_control::admit_retained_growth(
                    0,
                    Some(bytes),
                    "metamorphosis_witness_copy",
                )?;
                control::charge(bytes as u64, "metamorphosis_witness_copy")?;
                let witness = EmployeeTransformationWitness {
                    generation: self.content_identity.clone(),
                    actor,
                    employee_uuid,
                    successor,
                    equipment,
                    time_ms: now,
                    position,
                    form: form.clone(),
                    definition_id: program.definition_id.clone(),
                    source_file: program.source_file.clone(),
                    field_path: program.field_path.clone(),
                };
                runtime_control::admit_retained_growth(
                    0,
                    witness.retained_extent(),
                    "metamorphosis_observation",
                )?;
                self.authoring_observation
                    .as_mut()
                    .unwrap()
                    .latest_employee_transformation = Some(witness);
            }
            control::poll("metamorphosis_complete")
        })();
        self.runtime_fact_source = prior;
        result
    }
}
