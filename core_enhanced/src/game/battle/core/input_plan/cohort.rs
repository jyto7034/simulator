use super::super::spawn_kernel::*;
use super::*;
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
impl BattleCore {
    pub(in crate::game::battle::core) fn shot_cohort_block_demand(
        &self,
        id: UnitInstanceId,
    ) -> Option<u32> {
        self.sustained
            .input_plans
            .cohorts
            .get(&id)
            .and_then(|copy| self.sustained.input_plans.owners.get(copy))
            .map(|o| o.program.cohort.block)
    }
    pub(in crate::game::battle::core) fn settle_cohort_escapes(
        &mut self,
        now: u64,
    ) -> Result<(), GameError> {
        control::charge(
            self.sustained.input_plans.cohorts.len() as u64,
            "cohort_escape_scan",
        )?;
        let escaped: Vec<_> = self
            .sustained
            .input_plans
            .cohorts
            .iter()
            .filter_map(|(id, copy)| {
                let u = self.units.get(id)?;
                let EnemyMovementPlan::PathAlongCells { cells } = u.enemy_movement_plan.as_ref()?;
                (u.is_active()
                    && self.blocked_by(*id).is_none()
                    && cells.last().is_some_and(|tile| {
                        u.body.position.distance(WorldVec2::from_tile_center(*tile)) <= 0.25
                    }))
                .then_some((*id, *copy))
            })
            .collect();
        for (id, copy) in escaped {
            let leak = self.sustained.input_plans.owners[&copy].program.cohort.leak;
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
            self.sustained.input_plans.cohorts.remove(&id);
            self.withdraw_unit(id, now)?;
            if let (Some(target_id), Some(source_snapshot)) = (target, source) {
                self.execute_committed_damage(super::super::damage_runtime::CommittedDamageInput {
                    target_id,
                    source_snapshot,
                    committed_at_ms: now,
                    policy: super::super::damage_runtime::CommittedDamagePolicy::BypassShields,
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
            || routes
                .iter()
                .any(|route| route.is_empty() || route.iter().any(|p| !self.projectile_tile_valid(*p)))
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
            .input_plans
            .owners
            .get(&copy)
            .filter(|o| o.next_cohort == now)
        else {
            return Ok(());
        };
        let h = controlled_clone(&o.program.cohort)?;
        let sequence = o.spawn_index;
        let caster = o
            .caster
            .unwrap_or_else(|| crate::game::battle::ids::UnitInstanceId(o.employee));
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
                definition_id: h.definition,
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
        self.sustained.input_plans.cohorts.insert(id, copy);
        self.event_queue.push(BattleEvent::AttackStart {
            time_ms: now,
            attacker_instance_id: id,
            target_instance_id: None,
            schedule_next: true,
            cause: BattleEventCause::default(),
        });
        self.schedule_continuous_movement_tick(now);
        let o = self.sustained.input_plans.owners.get_mut(&copy).unwrap();
        o.spawn_index = sequence.checked_add(1).ok_or(GameError::InvalidAction)?;
        o.next_cohort = now
            .checked_add(u64::from(o.program.cohort.interval_ms))
            .ok_or(GameError::InvalidAction)?;
        self.event_queue.push(BattleEvent::HostileCohortTick {
            source: copy,
            time_ms: o.next_cohort,
        });
        control::poll("input_plan_hostile_spawn")
    }
    pub(in crate::game::battle::core) fn shot_cohort_death(
        &mut self,
        target: UnitInstanceId,
        killer_side: Side,
        now: u64,
    ) -> Result<(), GameError> {
        let Some(copy) = self.sustained.input_plans.cohorts.remove(&target) else {
            return Ok(());
        };
        if killer_side != Side::Player {
            return Ok(());
        }
        let Some(o) = self.sustained.input_plans.owners.get_mut(&copy) else {
            return Ok(());
        };
        let mut gain = o.program.cohort.kill_gain;
        for m in &o.program.modifiers {
            if let ShotChange::KillGain { factor } = m.change {
                gain = scale_floor(gain, factor)?;
            }
        }
        o.enhanced = add_stock(o.enhanced, gain, o.program.enhanced.capacity)?;
        let _ = now;
        control::poll("input_plan_kill_credit")
    }
    pub(in crate::game::battle::core) fn shot_cohort_alive(&self) -> bool {
        self.sustained
            .input_plans
            .cohorts
            .keys()
            .any(|id| self.units.get(id).is_some_and(|u| u.is_active()))
    }
    pub(in crate::game::battle::core) fn shot_cohort_damage_bonus(
        &self,
        target: UnitInstanceId,
        source: &DamageSourceSnapshot,
    ) -> Result<i32, GameError> {
        let Some(o) = self
            .sustained
            .input_plans
            .cohorts
            .get(&target)
            .and_then(|copy| self.sustained.input_plans.owners.get(copy))
        else {
            return Ok(0);
        };
        let family = &o.program.cohort.weakness_family;
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
            u128::from(o.program.cohort.weakness_percent.numerator)
                .div_ceil(u128::from(o.program.cohort.weakness_percent.denominator)),
        )
        .map_err(|_| GameError::InvalidAction)
    }
}
