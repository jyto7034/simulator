use super::spawn_kernel::*;
use super::BattleCore;
use crate::game::{
    battle::{
        event_log::BattleEventCause,
        scenario::{
            EnemyMovementPlan, ScenarioGroupId, ScenarioSpawnGroup, ScenarioUnitRef,
            ScenarioUnitSpawn,
        },
        types::{BattleUnitDraft, BattleUnitSource, BattleUnitThreatClass, UnitCombatProfile},
    },
    data::skill_fragment_data::SkillFragmentOrigin,
    determinism,
};
use crate::{
    authoring::{clone_shape::controlled_clone, control},
    game::{
        ability::hostile_cohort::*,
        battle::{
            core::movement::types::WorldVec2, damage::DamageSourceSnapshot, enums::BattleEvent,
            ids::UnitInstanceId, types::BattleUnitSourceIdentity,
        },
        behavior::GameError,
        enums::Side,
        resources::Position,
    },
};
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub(super) struct CohortOwner {
    pub(super) employee: Uuid,
    pub(super) profile: Arc<HostileCohortDef>,
    pub(super) fragment: Option<Uuid>,
    pub(super) providers: BTreeMap<Uuid, HostileCohortProvider>,
    pub(super) next_cohort: u64,
    spawn_index: u64,
}
crate::authoring::clone_shape::struct_shape!(CohortOwner {
    employee,
    profile,
    fragment,
    providers,
    next_cohort,
    spawn_index
});
#[derive(Debug, Clone)]
pub(super) struct CohortMember {
    pub(super) spawner: Uuid,
    pub(super) fragment: Option<Uuid>,
    pub(super) profile: Arc<HostileCohortDef>,
}
crate::authoring::clone_shape::struct_shape!(CohortMember {
    spawner,
    fragment,
    profile
});
#[derive(Debug, Clone, Default)]
pub(super) struct SharedCohortState {
    pub(super) owners: BTreeMap<Uuid, CohortOwner>,
    pub(super) members: BTreeMap<UnitInstanceId, CohortMember>,
}
crate::authoring::clone_shape::struct_shape!(SharedCohortState { owners, members });
impl SharedCohortState {
    pub(super) fn entity_count(&self) -> usize {
        self.owners.len() + self.members.len()
    }
    pub(super) fn retained_extent(&self) -> Option<usize> {
        let mut n =
            std::mem::size_of::<Self>().checked_add(self.entity_count().checked_mul(256)?)?;
        for o in self.owners.values() {
            n = n
                .checked_add(o.profile.retained_extent()?)?
                .checked_add(o.providers.len().checked_mul(96)?)?;
        }
        for m in self.members.values() {
            n = n.checked_add(m.profile.retained_extent()?)?;
        }
        Some(n)
    }
    pub(super) fn next_for_fragment(&self, copy: Uuid) -> u64 {
        self.owners
            .values()
            .find(|o| o.fragment == Some(copy))
            .map_or(0, |o| o.next_cohort)
    }
}
impl BattleCore {
    pub(super) fn clear_shared_cohorts(&mut self) -> Result<(), GameError> {
        self.sustained.shared_cohorts = Default::default();
        self.event_queue.discard_shared_cohort_events()
    }
    pub fn shared_cohorts_snapshot(&self) -> Result<SharedCohortsSnapshot, GameError> {
        let state = &self.sustained.shared_cohorts;
        control::charge(state.entity_count() as u64, "shared_cohort_snapshot")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            state.retained_extent(),
            "shared_cohort_snapshot_storage",
        )?;
        Ok(SharedCohortsSnapshot {
            owners: state
                .owners
                .iter()
                .map(|(id, o)| {
                    Ok(SharedCohortOwnerView {
                        id: *id,
                        employee: o.employee,
                        definition: controlled_clone(&o.profile.definition)?,
                        providers: o.providers.values().copied().collect(),
                        next_spawn_ms: o.next_cohort,
                    })
                })
                .collect::<Result<_, GameError>>()?,
            units: state
                .members
                .iter()
                .map(|(unit, m)| SharedCohortUnitView {
                    unit: *unit,
                    spawner: m.spawner,
                    fragment_recipient: m.fragment,
                })
                .collect(),
        })
    }
    pub(crate) fn register_equipment_cohort(
        &mut self,
        employee: Uuid,
        copy: Uuid,
        definition: Uuid,
        now: u64,
    ) -> Result<(), GameError> {
        let program = self
            .game_data
            .equipment_attack_catalog()
            .by_equipment
            .get(&definition)
            .cloned();
        if let Some(cohort) = program.as_ref().and_then(|p| p.cohort.as_ref()) {
            self.register_shared_cohort(
                employee,
                HostileCohortProvider::Equipment { copy, definition },
                &cohort.spawn,
                now,
            )?;
        }
        Ok(())
    }
    pub(super) fn reconcile_equipment_cohorts(&mut self, now: u64) -> Result<(), GameError> {
        if self.sustained.closed {
            return Ok(());
        }
        control::charge(self.items.len() as u64, "shared_cohort_equipped_scan")?;
        let mut providers = Vec::new();
        for item in self.items.values() {
            let Some(unit) = self
                .units
                .get(&item.owner_unit_instance)
                .filter(|u| u.is_active() && u.owner == Side::Player)
            else {
                continue;
            };
            let BattleUnitSourceIdentity::Employee { employee_uuid, .. } = unit.source_identity
            else {
                continue;
            };
            if unit.equipment_slots.as_ref().is_some_and(|slots| {
                !slots
                    .slots
                    .iter()
                    .flatten()
                    .any(|slot| slot.runtime_item_instance_id == item.instance_id)
            }) {
                continue;
            }
            if self
                .game_data
                .equipment_attack_catalog()
                .by_equipment
                .get(&item.base_uuid)
                .is_some_and(|p| p.cohort.is_some())
            {
                providers.push((
                    employee_uuid,
                    item.persistent_copy_uuid.unwrap_or(item.instance_id),
                    item.base_uuid,
                ));
            }
        }
        for (employee, copy, definition) in providers {
            self.register_equipment_cohort(employee, copy, definition, now)?;
        }
        Ok(())
    }
    pub(super) fn equipment_prioritizes_cohort(
        &self,
        actor: UnitInstanceId,
        target: UnitInstanceId,
    ) -> bool {
        let Some(member) = self.sustained.shared_cohorts.members.get(&target) else {
            return false;
        };
        let Some(unit) = self
            .units
            .get(&actor)
            .filter(|u| u.owner == Side::Player && !u.corroded)
        else {
            return false;
        };
        if control::charge(self.items.len() as u64, "shared_cohort_priority").is_err() {
            return false;
        }
        self.items
            .values()
            .filter(|item| item.owner_unit_instance == actor)
            .filter(|item| {
                unit.equipment_slots.as_ref().is_none_or(|slots| {
                    slots
                        .slots
                        .iter()
                        .flatten()
                        .any(|slot| slot.runtime_item_instance_id == item.instance_id)
                })
            })
            .any(|item| {
                self.game_data
                    .equipment_attack_catalog()
                    .by_equipment
                    .get(&item.base_uuid)
                    .and_then(|p| p.cohort.as_ref())
                    .is_some_and(|c| {
                        c.prioritize && c.spawn.definition == member.profile.definition
                    })
            })
    }
    pub(super) fn register_shared_cohort(
        &mut self,
        employee: Uuid,
        provider: HostileCohortProvider,
        profile: &HostileCohortDef,
        now: u64,
    ) -> Result<(), GameError> {
        if self.sustained.closed {
            return Ok(());
        }
        let copy = provider.copy();
        let fragment = provider.fragment();
        control::charge(
            self.sustained.shared_cohorts.owners.len() as u64 + 1,
            "shared_cohort_registration",
        )?;
        let existing = self
            .sustained
            .shared_cohorts
            .owners
            .iter()
            .find(|(_, o)| o.employee == employee && o.profile.definition == profile.definition)
            .map(|(k, _)| *k);
        if let Some(key) = existing {
            let o = &self.sustained.shared_cohorts.owners[&key];
            if o.profile.as_ref() != profile
                || o.providers.get(&copy).is_some_and(|old| old != &provider)
                || (fragment.is_some() && o.fragment.is_some() && fragment != o.fragment)
            {
                return Err(GameError::InvalidStaticData(
                    "conflicting shared hostile cohort providers".into(),
                ));
            }
            if !o.providers.contains_key(&copy) {
                crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    Some(96),
                    "shared_cohort_provider",
                )?;
            }
            let o = self.sustained.shared_cohorts.owners.get_mut(&key).unwrap();
            o.providers.insert(copy, provider);
            if fragment.is_some() {
                o.fragment = fragment;
            }
            return Ok(());
        }
        if self.sustained.shared_cohorts.owners.contains_key(&copy) {
            return Err(GameError::InvalidStaticData(
                "shared cohort provider identity collision".into(),
            ));
        }
        profile.validate().map_err(GameError::InvalidStaticData)?;
        self.input_plan_routes()?;
        let next_cohort = now
            .checked_add(u64::from(profile.interval_ms))
            .ok_or(GameError::InvalidAction)?;
        self.admit_authoring_runtime_growth(1, "shared_cohort_owner")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            profile.retained_extent().and_then(|n| n.checked_add(512)),
            "shared_cohort_owner_storage",
        )?;
        let profile = Arc::new(controlled_clone(profile)?);
        self.sustained.shared_cohorts.owners.insert(
            copy,
            CohortOwner {
                employee,
                profile,
                fragment,
                providers: BTreeMap::from([(copy, provider)]),
                next_cohort,
                spawn_index: 0,
            },
        );
        self.event_queue.push(BattleEvent::HostileCohortTick {
            source: copy,
            time_ms: next_cohort,
        });
        Ok(())
    }
}
impl BattleCore {
    pub(in crate::game::battle::core) fn shot_cohort_block_demand(
        &self,
        id: UnitInstanceId,
    ) -> Option<u32> {
        self.sustained
            .shared_cohorts
            .members
            .get(&id)
            .map(|m| m.profile.block)
    }
    pub(in crate::game::battle::core) fn settle_cohort_escapes(
        &mut self,
        now: u64,
    ) -> Result<(), GameError> {
        control::charge(
            self.sustained.shared_cohorts.members.len() as u64,
            "cohort_escape_scan",
        )?;
        let escaped: Vec<_> = self
            .sustained
            .shared_cohorts
            .members
            .iter()
            .filter_map(|(id, copy)| {
                let u = self.units.get(id)?;
                let EnemyMovementPlan::PathAlongCells { cells } = u.enemy_movement_plan.as_ref()?;
                (u.is_active()
                    && self.blocked_by(*id).is_none()
                    && cells.last().is_some_and(|tile| {
                        u.body.position.distance(WorldVec2::from_tile_center(*tile)) <= 0.25
                    }))
                .then_some((*id, copy.profile.leak))
            })
            .collect();
        for (id, leak) in escaped {
            let target = self.protected_unit_id();
            let source = target.and_then(|target| {
                self.damage_source_snapshot_for_unit(
                    id,
                    target,
                    crate::game::battle::damage::DamageSource::Environment,
                    crate::game::battle::damage::DamageType::True,
                    leak,
                    Default::default(),
                    0,
                    now,
                    false,
                )
            });
            self.sustained.shared_cohorts.members.remove(&id);
            self.withdraw_unit(id, now)?;
            if let (Some(target_id), Some(source_snapshot)) = (target, source) {
                self.execute_committed_damage(super::damage_runtime::CommittedDamageInput {
                    target_id,
                    source_snapshot,
                    committed_at_ms: now,
                    policy: super::damage_runtime::CommittedDamagePolicy::BypassShields,
                });
                control::poll("cohort_escape_damage")?;
            }
        }
        Ok(())
    }

    pub(in crate::game::battle::core) fn input_plan_routes(
        &self,
    ) -> Result<Vec<Vec<Position>>, GameError> {
        let mut routes = Vec::new();
        for group in self
            .scenario
            .groups
            .iter()
            .filter(|g| g.side == Side::Opponent)
        {
            let EnemyMovementPlan::PathAlongCells { cells } = group
                .enemy_movement_plan
                .as_ref()
                .unwrap_or(&self.scenario.tactical_plan.enemy_plan);
            control::charge(
                (cells.len() as u64).saturating_mul(routes.len() as u64 + 1),
                "cohort_route_scan",
            )?;
            if !cells.is_empty() && !routes.contains(cells) {
                routes.push(controlled_clone(cells)?);
            }
        }
        if routes.is_empty() {
            let EnemyMovementPlan::PathAlongCells { cells } =
                &self.scenario.tactical_plan.enemy_plan;
            if !cells.is_empty() {
                routes.push(controlled_clone(cells)?);
            }
        }
        if routes.is_empty()
            || routes.iter().any(|route| {
                route.is_empty() || route.iter().any(|p| !self.projectile_tile_valid(*p))
            })
        {
            return Err(GameError::InvalidStaticData(
                "Equipped hostile spawner requires a valid entry route".into(),
            ));
        }
        Ok(routes)
    }
    pub(in crate::game::battle::core) fn hostile_cohort_tick(
        &mut self,
        copy: Uuid,
        now: u64,
    ) -> Result<(), GameError> {
        if self.compute_terminal_outcome(now).is_some() {
            return Ok(());
        }
        let Some(o) = self
            .sustained
            .shared_cohorts
            .owners
            .get(&copy)
            .filter(|o| o.next_cohort == now)
        else {
            return Ok(());
        };
        let h = Arc::clone(&o.profile);
        let fragment = o.fragment;
        let sequence = o.spawn_index;
        let caster = self.units.values().find(|u| u.is_active() && matches!(
            u.source_identity, BattleUnitSourceIdentity::Employee {employee_uuid, ..} if employee_uuid == o.employee
        )).map_or(UnitInstanceId(o.employee), |u| u.instance_id);
        let routes = self.input_plan_routes()?;
        let route = controlled_clone(&routes[(sequence % routes.len() as u64) as usize])?;
        let uuid =
            determinism::uuid_v4_from_seed(self.seed, 0x53484f54434f484f, self.next_shot_id()?);
        let salt = u32::try_from(sequence).map_err(|_| GameError::InvalidAction)?;
        let speed =
            u32::try_from(u128::from(h.speed.numerator) * 1000 / u128::from(h.speed.denominator))
                .map_err(|_| GameError::InvalidAction)?;
        if speed == 0 {
            return Err(GameError::InvalidStaticData(
                "Hostile speed is below native precision".into(),
            ));
        }
        let mut profile = UnitCombatProfile::for_employee(
            h.hp,
            h.power,
            i32::try_from(h.physical_defense).map_err(|_| GameError::InvalidAction)?,
            i32::try_from(h.magic_defense).map_err(|_| GameError::InvalidAction)?,
            100,
        );
        profile.basic_attack.interval_ms = u64::from(h.attack_interval_ms);
        profile.basic_attack.windup_ms = 200;
        profile.basic_attack.damage_type = h.damage_type;
        profile.basic_attack.defense_tile_range = Some(h.attack_range.clone());
        profile.stats.attack_interval_ms = u64::from(h.attack_interval_ms);
        profile.movement.speed_units_per_ms = speed;
        profile.stats.move_speed_units_per_ms = speed;
        profile.block_capacity = h.block;
        let group = ScenarioSpawnGroup {
            id: ScenarioGroupId::new(format!("cohort:{copy}:{sequence}")),
            side: Side::Opponent,
            required_for_victory: false,
            enemy_movement_plan: Some(EnemyMovementPlan::PathAlongCells {
                cells: route.clone(),
            }),
            spawns: vec![ScenarioUnitSpawn {
                unit_ref: ScenarioUnitRef::new(format!("cohort:{copy}:{sequence}")),
                side: Side::Opponent,
                draft: BattleUnitDraft {
                    owned_uuid: uuid,
                    source: BattleUnitSource::SummonedUnit {
                        definition_id: h.definition.clone(),
                        base_uuid: uuid,
                        profile,
                        summoner_instance_id: caster,
                        summon_sequence: sequence,
                        batch_uuid: uuid,
                    },
                    threat_class: BattleUnitThreatClass::Normal,
                    level: crate::game::enums::Tier::I,
                    stat_scale: Default::default(),
                    growth_stacks: crate::game::growth::GrowthStack::new(),
                    equipment_layout: None,
                    equipped_items: Vec::new(),
                    equipped_item_enhancements: Vec::new(),
                    employee_start_condition: None,
                },
                position: route[0],
                instance_salt: salt,
            }],
        };
        let request = SpawnRequest {
            id: SpawnRequestId(format!("cohort:{copy}:{sequence}")),
            source: SpawnRequestSource::Summon {
                summoner: caster,
                definition_id: h.definition.clone(),
                count: 1,
            },
            owner: Side::Opponent,
            occupancy: SpawnOccupancyPolicy::AllowOccupied(vec![SpawnOccupancyClass::Any]),
            placements: vec![SpawnPlacement {
                order: 0,
                position: route[0],
                terrain_walkable: true,
                occupants: Vec::new(),
            }],
        };
        let result = self
            .execute_spawn_request(request, |c, _| c.spawn_scenario_group_member(&group, 0))
            .map_err(|e| GameError::InvalidStaticData(format!("cohort spawn failed: {e:?}")))?;
        let SpawnRequestSettlement::Spawned { unit_ids } = result else {
            return Err(GameError::InvalidAction);
        };
        let id = unit_ids[0];
        self.record_spawned_units(now, BattleEventCause::default(), &unit_ids)?;
        self.admit_authoring_runtime_growth(1, "cohort_owner_entity")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(192),
            "cohort_owner_storage",
        )?;
        self.sustained.shared_cohorts.members.insert(
            id,
            CohortMember {
                spawner: copy,
                fragment,
                profile: h,
            },
        );
        self.event_queue.push(BattleEvent::AttackStart {
            time_ms: now,
            attacker_instance_id: id,
            target_instance_id: None,
            schedule_next: true,
            cause: BattleEventCause::default(),
        });
        self.schedule_continuous_movement_tick(now);
        let o = self.sustained.shared_cohorts.owners.get_mut(&copy).unwrap();
        o.spawn_index = sequence.checked_add(1).ok_or(GameError::InvalidAction)?;
        o.next_cohort = now
            .checked_add(u64::from(o.profile.interval_ms))
            .ok_or(GameError::InvalidAction)?;
        self.event_queue.push(BattleEvent::HostileCohortTick {
            source: copy,
            time_ms: o.next_cohort,
        });
        self.observe_shared_cohort_spawn(copy, id, now)?;
        control::poll("input_plan_hostile_spawn")
    }
    pub(in crate::game::battle::core) fn shot_cohort_death(
        &mut self,
        target: UnitInstanceId,
        source: &DamageSourceSnapshot,
        now: u64,
    ) -> Result<(), GameError> {
        let Some(member) = self.sustained.shared_cohorts.members.remove(&target) else {
            return Ok(());
        };
        if source.source_side != Side::Player {
            return Ok(());
        }
        self.reward_cohort_magazine(source.source_id, &member.profile.weakness_family, now)?;
        self.reward_cohort_bounty(source, &member.profile.definition, now)?;
        if let Some(copy) = member.fragment {
            self.credit_input_plan_cohort_kill(copy)?;
        }
        control::poll("shared_cohort_kill_credit")
    }
    pub(in crate::game::battle::core) fn shot_cohort_alive(&self) -> bool {
        self.sustained
            .shared_cohorts
            .members
            .keys()
            .any(|id| self.units.get(id).is_some_and(|u| u.is_active()))
    }
    pub(in crate::game::battle::core) fn shot_cohort_damage_bonus(
        &self,
        target: UnitInstanceId,
        source: &DamageSourceSnapshot,
    ) -> Result<i32, GameError> {
        let Some(o) = self.sustained.shared_cohorts.members.get(&target) else {
            return Ok(0);
        };
        let family = &o.profile.weakness_family;
        let fragment = source
            .fact_source
            .fragment
            .as_ref()
            .and_then(|f| {
                self.game_data
                    .skill_fragment_data
                    .get_by_id(&f.definition_id)
            })
            .and_then(|m| m.origin.as_ref())
            .is_some_and(|origin| match origin {
                SkillFragmentOrigin::Abnormality { abnormality_id } => abnormality_id == family,
                SkillFragmentOrigin::Concept { concept_id } => concept_id == family,
            });
        let weapon = source
            .fact_source
            .equipment
            .and_then(|e| {
                self.game_data
                    .equipment_data
                    .get_by_uuid(&e.definition_uuid)
            })
            .is_some_and(|e| {
                e.equipment_type == crate::game::data::equipment_data::EquipmentType::Weapon
                    && e.content_family.as_ref() == Some(family)
            });
        if !fragment && !weapon {
            return Ok(0);
        }
        i32::try_from(
            u128::from(o.profile.weakness_percent.numerator)
                .div_ceil(u128::from(o.profile.weakness_percent.denominator)),
        )
        .map_err(|_| GameError::InvalidAction)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SharedCohortWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub employee: Uuid,
    pub definition: String,
    pub unit: UnitInstanceId,
    pub time_ms: u64,
    pub next_spawn_ms: u64,
    pub provider_count: usize,
    pub fragment_recipient: Option<Uuid>,
    pub equipment_copy: Option<Uuid>,
    pub initial_hp: u32,
    pub source_file: Option<String>,
    pub field_path: Option<String>,
}
crate::authoring::clone_shape::struct_shape!(SharedCohortWitness {
    generation,
    employee,
    definition,
    unit,
    time_ms,
    next_spawn_ms,
    provider_count,
    fragment_recipient,
    equipment_copy,
    initial_hp,
    source_file,
    field_path
});
impl SharedCohortWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut n = std::mem::size_of::<Self>()
            .checked_add(self.definition.capacity())?
            .checked_add(self.source_file.as_ref().map_or(0, String::capacity))?
            .checked_add(self.field_path.as_ref().map_or(0, String::capacity))?;
        if let Some(g) = &self.generation {
            n = n
                .checked_add(std::mem::size_of_val(g.as_ref()))?
                .checked_add(g.semantic_sha256.capacity())?;
        }
        Some(n)
    }
}
impl BattleCore {
    fn observe_shared_cohort_spawn(
        &mut self,
        spawner: Uuid,
        unit: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        if self.authoring_observation.is_none() {
            return Ok(());
        }
        let o = &self.sustained.shared_cohorts.owners[&spawner];
        let source = o.providers.values().find_map(|p| match p {
            HostileCohortProvider::Equipment { copy, definition } => self
                .game_data
                .equipment_attack_catalog()
                .by_equipment
                .get(definition)
                .map(|p| (*copy, &p.cohort_source)),
            HostileCohortProvider::Fragment { .. } => None,
        });
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            o.profile.retained_extent(),
            "shared_cohort_witness_copy",
        )?;
        let witness = SharedCohortWitness {
            generation: self.content_identity.clone(),
            employee: o.employee,
            definition: controlled_clone(&o.profile.definition)?,
            unit,
            time_ms: now,
            next_spawn_ms: o.next_cohort,
            provider_count: o.providers.len(),
            fragment_recipient: o.fragment,
            equipment_copy: source.map(|(copy, _)| copy),
            initial_hp: self.units[&unit].stats.current_health,
            source_file: source
                .map(|(_, s)| controlled_clone(&s.source_file))
                .transpose()?,
            field_path: source
                .map(|(_, s)| controlled_clone(&s.field_path))
                .transpose()?,
        };
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            witness.retained_extent(),
            "shared_cohort_observation",
        )?;
        if let Some(o) = &mut self.authoring_observation {
            o.latest_shared_cohort = Some(witness);
        }
        Ok(())
    }
}
