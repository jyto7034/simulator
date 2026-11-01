use std::collections::BTreeMap;

use uuid::Uuid;

use crate::game::{
    battle::{
        enums::BattleEvent,
        event_log::{BattleEventCause, BattleEventRootCause},
        ids::UnitInstanceId,
        scenario::{
            EnemyMovementPlan, ScenarioGroupId, ScenarioSpawnGroup, ScenarioUnitRef,
            ScenarioUnitSpawn,
        },
        types::{BattleUnitDraft, BattleUnitSource, BattleUnitThreatClass},
    },
    behavior::GameError,
    data::{
        boss_runtime_data::{
            BossSummonOccupancyClassData, BossSummonOccupancyPolicyData, BossSummonPlacementData,
            BossSummonRemovalModeData, BossTaskResultData, SummonBatchHandleId,
        },
        summoned_unit_data::{SummonedUnitDefinitionId, SummonerRemovalPolicyData},
    },
    determinism,
    enums::Side,
    growth::GrowthStack,
    resources::Position,
};

use super::{
    boss_runtime::{ActiveBossSummonInsufficientSpace, BossSummonTaskView},
    types::RuntimeUnitLifecycle,
    BattleCore, RemovalEventId, SpawnCancellationReason, SpawnOccupancyClass, SpawnOccupancyPolicy,
    SpawnOccupantKind, SpawnPlacement, SpawnRejection, SpawnRequest, SpawnRequestId,
    SpawnRequestSettlement, SpawnRequestSource, SpawnTriggerId,
};

const SUMMON_BATCH_UUID_NS: u64 = 0x5355_4d4d_4241_5443; // "SUMMBATC"
const SUMMON_UNIT_UUID_NS: u64 = 0x5355_4d4d_554e_4954; // "SUMMUNIT"

#[derive(Debug, Clone, Default)]
pub(super) struct SummonRuntimeState {
    pending: BTreeMap<String, PendingSummonRequest>,
    counters: BTreeMap<(UnitInstanceId, SummonedUnitDefinitionId), SummonCounter>,
    batches: BTreeMap<Uuid, SummonBatchRuntime>,
    next_uuid_index: u64,
}

#[derive(Debug, Clone)]
struct PendingSummonRequest {
    view: BossSummonTaskView,
    trigger_id: SpawnTriggerId,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct SummonCounter {
    alive: u32,
    total: u32,
}

#[derive(Debug, Clone)]
struct SummonBatchRuntime {
    batch_uuid: Uuid,
    summoner_unit_id: UnitInstanceId,
    definition_id: SummonedUnitDefinitionId,
    pattern_activation_id: u64,
    batch_handle_id: SummonBatchHandleId,
    members: Vec<SummonBatchMember>,
    on_summoner_removed: SummonerRemovalPolicyData,
    lifetime: Option<(u64, BossSummonRemovalModeData)>,
    removal_signal: Option<(
        crate::game::data::boss_runtime_data::BossSignalId,
        u64,
        BossSummonRemovalModeData,
    )>,
    removal_cause: Option<SummonBatchRemovalCause>,
    cleared_emitted: bool,
}

#[derive(Debug, Clone)]
struct SummonBatchMember {
    unit_id: UnitInstanceId,
    alive_counted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SummonBatchRemovalCause {
    SummonerRemoved,
    Lifetime,
    Signal,
}

crate::authoring::clone_shape::struct_shape!(SummonRuntimeState {
    pending,
    counters,
    batches,
    next_uuid_index
});
crate::authoring::clone_shape::struct_shape!(PendingSummonRequest { view, trigger_id });
crate::authoring::clone_shape::copy_shape!(SummonCounter, SummonBatchRemovalCause);
crate::authoring::clone_shape::struct_shape!(SummonBatchRuntime {
    batch_uuid,
    summoner_unit_id,
    definition_id,
    pattern_activation_id,
    batch_handle_id,
    members,
    on_summoner_removed,
    lifetime,
    removal_signal,
    removal_cause,
    cleared_emitted,
});
crate::authoring::clone_shape::struct_shape!(SummonBatchMember {
    unit_id,
    alive_counted
});

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SummonRuntimeDebugSnapshot {
    pub pending_count: usize,
    pub batch_count: usize,
    pub counters: Vec<(UnitInstanceId, String, u32, u32)>,
    pub batches: Vec<(Uuid, Vec<UnitInstanceId>, bool)>,
    pub next_uuid_index: u64,
}

impl BattleCore {
    pub(super) fn settle_boss_summons(&mut self, now_ms: u64) -> Result<bool, GameError> {
        self.capture_boss_summon_requests()?;
        let mut changed = self.reconcile_summon_batches(now_ms)?;

        let mut request_ids = self
            .summon_runtime
            .pending
            .values()
            .map(|pending| {
                (
                    pending.view.task.admitted_at_ms,
                    pending.view.task.request_id.clone(),
                )
            })
            .collect::<Vec<_>>();
        request_ids.sort();
        for (_, request_id) in request_ids {
            changed |= self.settle_one_summon_request(&request_id, now_ms)?;
        }
        Ok(changed)
    }

    pub(super) fn capture_boss_summon_requests(&mut self) -> Result<(), GameError> {
        let views = self
            .boss_runtime
            .as_ref()
            .map(|runtime| runtime.summon_task_views())
            .transpose()?
            .unwrap_or_default();
        for view in views {
            if self
                .summon_runtime
                .pending
                .contains_key(&view.task.request_id)
            {
                continue;
            }
            let trigger_id = SpawnTriggerId(format!("summon_trigger:{}", view.task.request_id));
            self.register_spawn_trigger(trigger_id.clone())
                .map_err(summon_kernel_error)?;
            self.summon_runtime.pending.insert(
                view.task.request_id.clone(),
                PendingSummonRequest { view, trigger_id },
            );
        }
        Ok(())
    }

    fn settle_one_summon_request(
        &mut self,
        request_id: &str,
        now_ms: u64,
    ) -> Result<bool, GameError> {
        let Some(pending) = self
            .summon_runtime
            .pending
            .get(request_id)
            .map(crate::authoring::clone_shape::controlled_clone)
            .transpose()?
        else {
            return Ok(false);
        };
        let view = &pending.view;
        let state_active = self
            .boss_runtime
            .as_ref()
            .is_some_and(|runtime| runtime.summon_request_is_active(view));
        let summoner_active = self
            .units
            .get(&view.summoner_unit_id)
            .is_some_and(|unit| unit.is_active());
        if !state_active || !summoner_active {
            let request = self.summon_spawn_request(view, Vec::new());
            self.bind_spawn_trigger(&pending.trigger_id, request.clone())
                .map_err(summon_kernel_error)?;
            let reason = if !summoner_active {
                SpawnCancellationReason::SummonerRemoved {
                    removal_event_id: RemovalEventId(format!(
                        "summoner_removed:{}:{now_ms}",
                        view.summoner_unit_id
                    )),
                }
            } else {
                SpawnCancellationReason::OwnerStateExited {
                    state_activation_id: view.state_activation_id,
                }
            };
            let settlement = self
                .cancel_spawn_request(request, reason)
                .map_err(summon_kernel_error)?;
            self.finish_summon_trigger(&pending.trigger_id, &settlement)?;
            if state_active {
                self.set_boss_summon_task_result(view, BossTaskResultData::Canceled, None)?;
            }
            self.summon_runtime.pending.remove(request_id);
            return Ok(true);
        }

        let definition = self
            .game_data
            .boss_runtime_data
            .get_summoned_unit(&view.task.definition_id)
            .cloned()
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "missing summoned-unit definition '{}'",
                    view.task.definition_id.as_str()
                ))
            })?;
        let counter_key = (view.summoner_unit_id, view.task.definition_id.clone());
        let counter = self
            .summon_runtime
            .counters
            .get(&counter_key)
            .copied()
            .unwrap_or_default();
        let next_alive = counter.alive.checked_add(view.task.count);
        let next_total = counter.total.checked_add(view.task.count);
        if next_alive.is_none_or(|value| value > definition.max_alive)
            || next_total.is_none_or(|value| value > definition.max_total)
        {
            let request = self.summon_spawn_request(view, Vec::new());
            self.bind_spawn_trigger(&pending.trigger_id, request.clone())
                .map_err(summon_kernel_error)?;
            let settlement = self
                .reject_spawn_request(
                    request,
                    SpawnRejection::LimitReached {
                        max_alive: definition.max_alive,
                        max_total: definition.max_total,
                    },
                )
                .map_err(summon_kernel_error)?;
            self.finish_summon_trigger(&pending.trigger_id, &settlement)?;
            self.set_boss_summon_task_result(view, BossTaskResultData::LimitReached, None)?;
            self.summon_runtime.pending.remove(request_id);
            return Ok(true);
        }

        let placements = self.summon_candidate_placements(view)?;
        if placements.len() < view.task.count as usize {
            let must_fail = match view.task.insufficient_space {
                ActiveBossSummonInsufficientSpace::Fail => true,
                ActiveBossSummonInsufficientSpace::WaitUntilAvailable { deadline_ms } => {
                    now_ms >= deadline_ms
                }
            };
            if !must_fail {
                return Ok(false);
            }
            let request = self.summon_spawn_request(view, placements);
            self.bind_spawn_trigger(&pending.trigger_id, request.clone())
                .map_err(summon_kernel_error)?;
            let settlement = self
                .reject_spawn_request(
                    request,
                    SpawnRejection::InsufficientSpace {
                        required: view.task.count,
                        available: u32::try_from(self.summon_candidate_placements(view)?.len())
                            .unwrap_or(u32::MAX),
                    },
                )
                .map_err(summon_kernel_error)?;
            self.finish_summon_trigger(&pending.trigger_id, &settlement)?;
            self.set_boss_summon_task_result(view, BossTaskResultData::NoSpawnSpace, None)?;
            self.summon_runtime.pending.remove(request_id);
            return Ok(true);
        }

        self.commit_summon_batch(pending, definition, counter, placements, now_ms)?;
        Ok(true)
    }

    fn commit_summon_batch(
        &mut self,
        pending: PendingSummonRequest,
        definition: std::sync::Arc<
            crate::game::data::summoned_unit_data::CompiledSummonedUnitDefinition,
        >,
        counter: SummonCounter,
        mut placements: Vec<SpawnPlacement>,
        now_ms: u64,
    ) -> Result<(), GameError> {
        let view = pending.view;
        placements.truncate(view.task.count as usize);
        self.preflight_authoring_clone()?;
        let before = self.clone();
        crate::authoring::control::poll("battle_core_clone_complete")?;
        if let Err(error) = self.event_log.prepare_authoring_write() {
            *self = before;
            return Err(error);
        }
        let batch_index = self.summon_runtime.next_uuid_index;
        let batch_uuid =
            determinism::uuid_v4_from_seed(self.seed, SUMMON_BATCH_UUID_NS, batch_index);
        let next_uuid_index = batch_index
            .checked_add(u64::from(view.task.count) + 1)
            .ok_or(GameError::InvalidAction)?;
        let route = view
            .task
            .route_id
            .as_deref()
            .map(|route_id| {
                self.boss_runtime
                    .as_ref()
                    .and_then(|runtime| runtime.summon_route(route_id))
                    .map(|route| EnemyMovementPlan::PathAlongCells {
                        cells: route.cells.clone(),
                    })
                    .ok_or_else(|| {
                        GameError::InvalidStaticData(format!(
                            "missing resolved summon route '{route_id}'"
                        ))
                    })
            })
            .transpose()?;
        let stat_scale = self
            .boss_runtime
            .as_ref()
            .map(|runtime| runtime.summon_enemy_stat_scale())
            .ok_or(GameError::InvalidAction)?;

        let mut spawns = Vec::with_capacity(view.task.count as usize);
        for (member_index, placement) in placements.iter().enumerate() {
            let unit_uuid_index = batch_index
                .checked_add(member_index as u64 + 1)
                .ok_or(GameError::InvalidAction)?;
            let owned_uuid =
                determinism::uuid_v4_from_seed(self.seed, SUMMON_UNIT_UUID_NS, unit_uuid_index);
            let instance_salt = u32::try_from(unit_uuid_index).map_err(|_| {
                GameError::InvalidStaticData(
                    "summon instance-salt continuation exceeded u32".to_string(),
                )
            })?;
            spawns.push(ScenarioUnitSpawn {
                unit_ref: ScenarioUnitRef::new(format!("summon_{batch_uuid}_{member_index}")),
                side: Side::Opponent,
                draft: BattleUnitDraft {
                    owned_uuid,
                    source: BattleUnitSource::SummonedUnit {
                        definition_id: definition.id.as_str().to_string(),
                        base_uuid: definition.base_uuid,
                        profile: definition.combat_profile.clone(),
                        summoner_instance_id: view.summoner_unit_id,
                        summon_sequence: u64::from(counter.total) + member_index as u64 + 1,
                        batch_uuid,
                    },
                    threat_class: BattleUnitThreatClass::Normal,
                    level: crate::game::enums::Tier::I,
                    stat_scale,
                    growth_stacks: GrowthStack::new(),
                    equipment_layout: None,
                    equipped_items: Vec::new(),
                    equipped_item_enhancements: Vec::new(),
                    employee_start_condition: None,
                },
                position: placement.position,
                instance_salt,
            });
        }
        let group = ScenarioSpawnGroup {
            id: ScenarioGroupId::new(format!("summon_batch_{batch_uuid}")),
            side: Side::Opponent,
            required_for_victory: false,
            enemy_movement_plan: route,
            spawns,
        };
        let request = self.summon_spawn_request(&view, placements);
        if let Err(error) = self.bind_spawn_trigger(&pending.trigger_id, request.clone()) {
            *self = before;
            return Err(summon_kernel_error(error));
        }
        let settlement = match self.execute_spawn_request(request, |core, placement| {
            core.spawn_scenario_group_member(&group, placement.order as usize)
        }) {
            Ok(settlement) => settlement,
            Err(error) => {
                *self = before;
                return Err(summon_kernel_error(error));
            }
        };
        let unit_ids = match &settlement {
            SpawnRequestSettlement::Spawned { unit_ids } => unit_ids.clone(),
            other => {
                *self = before;
                return Err(GameError::InvalidStaticData(format!(
                    "validated summon batch failed Kernel commit: {other:?}"
                )));
            }
        };
        if let Err(error) = self.finish_summon_trigger(&pending.trigger_id, &settlement) {
            *self = before;
            return Err(error);
        }

        self.summon_runtime.next_uuid_index = next_uuid_index;
        self.summon_runtime.counters.insert(
            (view.summoner_unit_id, view.task.definition_id.clone()),
            SummonCounter {
                alive: counter.alive + view.task.count,
                total: counter.total + view.task.count,
            },
        );
        let signal = view.task.removal_signal.as_ref().map(|signal| {
            let epoch = self
                .boss_runtime
                .as_ref()
                .map(|runtime| runtime.signal_epoch(&signal.signal_id))
                .unwrap_or(0);
            (signal.signal_id.clone(), epoch, signal.mode)
        });
        let lifetime = view
            .task
            .lifetime
            .as_ref()
            .map(|lifetime| (now_ms.saturating_add(lifetime.duration_ms), lifetime.mode));
        self.summon_runtime.batches.insert(
            batch_uuid,
            SummonBatchRuntime {
                batch_uuid,
                summoner_unit_id: view.summoner_unit_id,
                definition_id: view.task.definition_id.clone(),
                pattern_activation_id: view.pattern_activation_id,
                batch_handle_id: view.task.batch_handle_id.clone(),
                members: unit_ids
                    .iter()
                    .copied()
                    .map(|unit_id| SummonBatchMember {
                        unit_id,
                        alive_counted: true,
                    })
                    .collect(),
                on_summoner_removed: definition.on_summoner_removed,
                lifetime,
                removal_signal: signal,
                removal_cause: None,
                cleared_emitted: false,
            },
        );
        if !self.set_boss_summon_task_result(
            &view,
            BossTaskResultData::Spawned,
            Some(batch_uuid),
        )? {
            *self = before;
            return Err(GameError::InvalidAction);
        }
        let behavior = self
            .game_data
            .boss_runtime_data
            .get_actor_behavior(&definition.actor_behavior_id)
            .cloned()
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "missing summoned-unit behavior '{}'",
                    definition.actor_behavior_id.as_str()
                ))
            })?;
        if let Some(runtime) = self.boss_runtime.as_mut() {
            for unit_id in &unit_ids {
                runtime.attach_summoned_actor(*unit_id, behavior.clone())?;
            }
        }
        self.record_spawned_units(
            now_ms,
            BattleEventCause::Root {
                kind: BattleEventRootCause::System,
            },
            &unit_ids,
        )?;
        for unit_id in &unit_ids {
            self.schedule_initial_attack_for_unit(*unit_id, now_ms);
        }
        if let Some((expires_at_ms, _)) = lifetime {
            self.event_queue.push(BattleEvent::BossSummonWake {
                time_ms: expires_at_ms,
                batch_uuid,
            });
        }
        self.summon_runtime.pending.remove(&view.task.request_id);
        Ok(())
    }

    fn reconcile_summon_batches(&mut self, now_ms: u64) -> Result<bool, GameError> {
        let batch_ids = self
            .summon_runtime
            .batches
            .keys()
            .copied()
            .collect::<Vec<_>>();
        let mut changed = false;
        for batch_uuid in batch_ids {
            let Some(snapshot) = self.summon_runtime.batches.get(&batch_uuid).cloned() else {
                continue;
            };
            let summoner_active = self
                .units
                .get(&snapshot.summoner_unit_id)
                .is_some_and(|unit| unit.is_active());
            let signal_fired =
                snapshot
                    .removal_signal
                    .as_ref()
                    .is_some_and(|(signal_id, bind_epoch, _)| {
                        self.boss_runtime
                            .as_ref()
                            .map(|runtime| runtime.signal_epoch(signal_id))
                            .unwrap_or(0)
                            > *bind_epoch
                    });
            let removal = if snapshot.removal_cause.is_some() {
                None
            } else if !summoner_active
                && snapshot.on_summoner_removed == SummonerRemovalPolicyData::Despawn
            {
                Some((
                    SummonBatchRemovalCause::SummonerRemoved,
                    BossSummonRemovalModeData::Despawn,
                ))
            } else if signal_fired {
                snapshot
                    .removal_signal
                    .as_ref()
                    .map(|(_, _, mode)| (SummonBatchRemovalCause::Signal, *mode))
            } else {
                snapshot
                    .lifetime
                    .filter(|(deadline_ms, _)| now_ms >= *deadline_ms)
                    .map(|(_, mode)| (SummonBatchRemovalCause::Lifetime, mode))
            };
            if let Some((cause, mode)) = removal {
                for member in &snapshot.members {
                    if self
                        .units
                        .get(&member.unit_id)
                        .is_some_and(|unit| unit.is_active())
                    {
                        self.remove_summoned_unit(member.unit_id, mode, now_ms)?;
                    }
                }
                if let Some(batch) = self.summon_runtime.batches.get_mut(&batch_uuid) {
                    batch.removal_cause = Some(cause);
                }
                changed = true;
            }

            let member_ids = self.summon_runtime.batches[&batch_uuid]
                .members
                .iter()
                .map(|member| member.unit_id)
                .collect::<Vec<_>>();
            for member_id in member_ids {
                let inactive = self
                    .units
                    .get(&member_id)
                    .is_none_or(|unit| !unit.is_active());
                if !inactive {
                    continue;
                }
                let release = self
                    .summon_runtime
                    .batches
                    .get_mut(&batch_uuid)
                    .and_then(|batch| {
                        batch
                            .members
                            .iter_mut()
                            .find(|member| member.unit_id == member_id)
                    })
                    .is_some_and(|member| {
                        if member.alive_counted {
                            member.alive_counted = false;
                            true
                        } else {
                            false
                        }
                    });
                if release {
                    let key = (snapshot.summoner_unit_id, snapshot.definition_id.clone());
                    if let Some(counter) = self.summon_runtime.counters.get_mut(&key) {
                        counter.alive = counter.alive.saturating_sub(1);
                    }
                    changed = true;
                }
            }

            let should_emit_clear =
                self.summon_runtime
                    .batches
                    .get(&batch_uuid)
                    .is_some_and(|batch| {
                        !batch.cleared_emitted
                            && batch.members.iter().all(|member| !member.alive_counted)
                    });
            if should_emit_clear {
                let batch = self
                    .summon_runtime
                    .batches
                    .get_mut(&batch_uuid)
                    .expect("batch exists");
                batch.cleared_emitted = true;
                if let Some(runtime) = self.boss_runtime.as_mut() {
                    runtime.note_summon_batch_cleared(
                        batch.pattern_activation_id,
                        &batch.batch_handle_id,
                        batch.batch_uuid,
                    );
                }
                changed = true;
            }
        }
        Ok(changed)
    }

    fn remove_summoned_unit(
        &mut self,
        unit_id: UnitInstanceId,
        mode: BossSummonRemovalModeData,
        now_ms: u64,
    ) -> Result<(), GameError> {
        let owner = self
            .units
            .get(&unit_id)
            .map(|unit| unit.owner)
            .ok_or(GameError::UnitNotFound)?;
        match mode {
            BossSummonRemovalModeData::Kill => {
                self.finalize_unit_death(None, unit_id, owner, now_ms);
            }
            BossSummonRemovalModeData::Despawn => {
                self.withdraw_unit(unit_id, now_ms)?;
                if let Some(unit) = self.units.get_mut(&unit_id) {
                    unit.lifecycle = RuntimeUnitLifecycle::Despawned;
                }
            }
        }
        Ok(())
    }

    fn summon_candidate_placements(
        &self,
        view: &BossSummonTaskView,
    ) -> Result<Vec<SpawnPlacement>, GameError> {
        let summoner = self
            .units
            .get(&view.summoner_unit_id)
            .filter(|unit| unit.is_active())
            .ok_or(GameError::InvalidAction)?;
        let mut candidates = match &view.task.placement {
            BossSummonPlacementData::AroundSummoner { radius_tiles } => {
                let origin = summoner.body.position;
                let radius_squared = (*radius_tiles as f32) * (*radius_tiles as f32);
                let mut cells = Vec::new();
                for y in 0..i32::from(self.battlefield.height()) {
                    for x in 0..i32::from(self.battlefield.width()) {
                        let position = Position::new(x, y);
                        if self.battlefield.ensure_walkable_tile(position).is_err() {
                            continue;
                        }
                        let center = super::movement::types::WorldVec2::from_tile_center(position);
                        let dx = center.x - origin.x;
                        let dy = center.y - origin.y;
                        let distance_squared = dx * dx + dy * dy;
                        if distance_squared <= radius_squared {
                            cells.push((distance_squared, y, x, position));
                        }
                    }
                }
                cells.sort_by(|left, right| {
                    left.0
                        .total_cmp(&right.0)
                        .then_with(|| left.1.cmp(&right.1))
                        .then_with(|| left.2.cmp(&right.2))
                });
                cells
                    .into_iter()
                    .map(|(_, _, _, position)| position)
                    .collect::<Vec<_>>()
            }
            BossSummonPlacementData::SpawnZone { zone_id } => self
                .boss_runtime
                .as_ref()
                .and_then(|runtime| runtime.summon_spawn_zone(zone_id))
                .map(|zone| zone.cells.clone())
                .ok_or_else(|| {
                    GameError::InvalidStaticData(format!(
                        "missing resolved summon spawn zone '{zone_id}'"
                    ))
                })?,
        };
        let policy = summon_occupancy_policy(&view.task.occupancy_policy);
        candidates.retain(|position| self.battlefield.ensure_walkable_tile(*position).is_ok());
        let mut placements = Vec::new();
        for position in candidates {
            let occupants = self.summon_occupants_at(position, view.summoner_unit_id);
            if !policy.allows(&occupants) {
                continue;
            }
            placements.push(SpawnPlacement {
                order: placements.len() as u32,
                position,
                terrain_walkable: true,
                occupants,
            });
        }
        Ok(placements)
    }

    fn summon_occupants_at(
        &self,
        position: Position,
        summoner_unit_id: UnitInstanceId,
    ) -> Vec<SpawnOccupantKind> {
        let summoner_owner = self
            .units
            .get(&summoner_unit_id)
            .map(|unit| unit.owner)
            .unwrap_or(Side::Opponent);
        let mut occupants = self
            .units
            .values()
            .filter(|unit| unit.is_active() && unit.body.projected_tile() == position)
            .map(|unit| {
                if unit.instance_id == summoner_unit_id {
                    SpawnOccupantKind::Summoner
                } else if unit.role == crate::game::battle::types::BattleUnitRole::DefenseObject {
                    if unit.owner == Side::Player {
                        SpawnOccupantKind::ProtectedObjective
                    } else {
                        SpawnOccupantKind::StaticDefenseObject
                    }
                } else if unit.owner == summoner_owner {
                    SpawnOccupantKind::SameFaction
                } else {
                    SpawnOccupantKind::OpposingCombatant
                }
            })
            .collect::<Vec<_>>();
        occupants.sort();
        occupants
    }

    fn summon_spawn_request(
        &self,
        view: &BossSummonTaskView,
        placements: Vec<SpawnPlacement>,
    ) -> SpawnRequest {
        SpawnRequest {
            id: SpawnRequestId(view.task.request_id.clone()),
            source: SpawnRequestSource::Summon {
                summoner: view.summoner_unit_id,
                definition_id: view.task.definition_id.as_str().to_string(),
                count: view.task.count,
            },
            owner: Side::Opponent,
            occupancy: summon_occupancy_policy(&view.task.occupancy_policy),
            placements,
        }
    }

    fn finish_summon_trigger(
        &mut self,
        trigger_id: &SpawnTriggerId,
        expected: &SpawnRequestSettlement,
    ) -> Result<(), GameError> {
        let actual = self
            .settle_spawn_trigger(trigger_id)
            .map_err(summon_kernel_error)?;
        if &actual != expected {
            return Err(GameError::InvalidStaticData(
                "summon task and Spawn Kernel settlement diverged".to_string(),
            ));
        }
        Ok(())
    }

    fn set_boss_summon_task_result(
        &mut self,
        view: &BossSummonTaskView,
        result: BossTaskResultData,
        batch_uuid: Option<Uuid>,
    ) -> Result<bool, GameError> {
        self.boss_runtime
            .as_mut()
            .ok_or(GameError::InvalidAction)?
            .settle_summon_task(view, result, batch_uuid)
    }

    #[cfg(test)]
    pub(crate) fn summon_runtime_debug_snapshot(&self) -> SummonRuntimeDebugSnapshot {
        SummonRuntimeDebugSnapshot {
            pending_count: self.summon_runtime.pending.len(),
            batch_count: self.summon_runtime.batches.len(),
            counters: self
                .summon_runtime
                .counters
                .iter()
                .map(|((summoner, definition), counter)| {
                    (
                        *summoner,
                        definition.as_str().to_string(),
                        counter.alive,
                        counter.total,
                    )
                })
                .collect(),
            batches: self
                .summon_runtime
                .batches
                .values()
                .map(|batch| {
                    (
                        batch.batch_uuid,
                        batch.members.iter().map(|member| member.unit_id).collect(),
                        batch.cleared_emitted,
                    )
                })
                .collect(),
            next_uuid_index: self.summon_runtime.next_uuid_index,
        }
    }
}

fn summon_occupancy_policy(policy: &BossSummonOccupancyPolicyData) -> SpawnOccupancyPolicy {
    match policy {
        BossSummonOccupancyPolicyData::RequireUnoccupied => SpawnOccupancyPolicy::RequireUnoccupied,
        BossSummonOccupancyPolicyData::AllowOccupied { allowed } => {
            SpawnOccupancyPolicy::AllowOccupied(
                allowed
                    .iter()
                    .map(|class| match class {
                        BossSummonOccupancyClassData::Summoner => SpawnOccupancyClass::Summoner,
                        BossSummonOccupancyClassData::SameFaction => {
                            SpawnOccupancyClass::SameFaction
                        }
                        BossSummonOccupancyClassData::OpposingCombatant => {
                            SpawnOccupancyClass::OpposingCombatant
                        }
                        BossSummonOccupancyClassData::ProtectedObjective => {
                            SpawnOccupancyClass::ProtectedObjective
                        }
                        BossSummonOccupancyClassData::StaticDefenseObject => {
                            SpawnOccupancyClass::StaticDefenseObject
                        }
                        BossSummonOccupancyClassData::Any => SpawnOccupancyClass::Any,
                    })
                    .collect(),
            )
        }
    }
}

fn summon_kernel_error(error: super::SpawnKernelError) -> GameError {
    GameError::InvalidStaticData(format!("summon Spawn Kernel error: {error:?}"))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::game::{
        ability::SkillActivationMode,
        battle::{
            core::{
                boss_runtime::{
                    ActiveBossSummonInsufficientSpace, ActiveBossSummonTask, BossSummonTaskView,
                },
                sim::BattleStepOutcome,
            },
            event_log::{BattleLogEvent, BATTLE_EVENT_LOG_VERSION},
            scenario::{
                BattleFieldSpec, BattleScenario, ScenarioAction, ScenarioEvent, ScenarioEventId,
                ScenarioTrigger, TacticalPlan,
            },
            types::{BattleUnitSourceIdentity, BattleUnitStatScale, UnitCombatProfile},
        },
        data::{
            boss_runtime_data::{
                tests::{summon_catalog_fixture, summon_task_mut},
                BossActorBehaviorDefinitionData, BossActorBehaviorId, BossActorSlotId,
                BossConditionData, BossControlNodeData, BossControlNodeId,
                BossControlTransitionData, BossDurationValueData, BossEncounterConditionData,
                BossEncounterPhaseData, BossEncounterPhaseId, BossEncounterPhaseTransitionData,
                BossEncounterRuntimeId, BossPhaseActorBehaviorData, BossPhaseTransitionTimingData,
                BossSignalDefinitionData, BossSignalId, BossSummonInsufficientSpaceData,
                BossSummonLifetimeData, BossSummonRemovalSignalData, BossTaskData, BossTaskId,
                BossTransitionId, ResolvedBossActorSetup, ResolvedBossEncounterSetup,
                ResolvedBossSummonRoute, ResolvedBossSummonSpawnZone,
            },
            summoned_unit_data::{SummonedUnitCatalogData, SummonerRemovalPolicyData},
            GameDataBuilder,
        },
        enums::{Side, Tier},
        growth::GrowthStack,
        stats::UnitStats,
    };

    fn fixture_draft(id: u128, threat: BattleUnitThreatClass) -> BattleUnitDraft {
        let mut profile = UnitCombatProfile::employee_default();
        profile.weapon_profile = Some(Default::default());
        profile.stats = UnitStats::with_values(1_000, 1_000, 1, 0, 1_000);
        profile.stats.move_speed_units_per_ms = 1_000;
        profile.basic_attack.interval_ms = 1_000;
        profile.basic_attack.windup_ms = 1;
        profile.skill_activation_mode = SkillActivationMode::Auto;
        BattleUnitDraft {
            owned_uuid: Uuid::from_u128(id),
            source: BattleUnitSource::TestFixture {
                base_uuid: Uuid::from_u128(id + 100),
                profile,
            },
            threat_class: threat,
            level: Tier::I,
            stat_scale: Default::default(),
            growth_stacks: GrowthStack::new(),
            equipment_layout: None,
            equipped_items: vec![],
            equipped_item_enhancements: vec![],
            employee_start_condition: None,
        }
    }

    fn fixture_core(
        actor_count: usize,
        player_position: Position,
        zone_cells: Vec<Position>,
        mutate: impl FnOnce(
            &mut crate::game::data::boss_runtime_data::BossRuntimeCatalogData,
            &mut SummonedUnitCatalogData,
        ),
    ) -> BattleCore {
        let (mut catalog, mut summoned) = summon_catalog_fixture();
        for actor_index in 1..actor_count {
            let slot = BossActorSlotId::new(format!("secondary_{actor_index}"));
            catalog.encounters[0].actor_slots.push(slot.clone());
            catalog.encounters[0].phases[0]
                .actor_behaviors
                .push(BossPhaseActorBehaviorData {
                    actor_slot_id: slot,
                    behavior_id: BossActorBehaviorId::new("summoning_behavior"),
                });
        }
        mutate(&mut catalog, &mut summoned);
        let database = catalog
            .compile_with_summoned_units(&summoned)
            .expect("summon runtime fixture must compile");
        let encounter = database
            .get_encounter(&BossEncounterRuntimeId::new("tracking_encounter"))
            .unwrap()
            .clone();
        let actors = (0..actor_count)
            .map(|actor_index| ResolvedBossActorSetup {
                actor_slot_index: actor_index,
                unit_ref: ScenarioUnitRef::new(format!("boss_{actor_index}")),
            })
            .collect::<Vec<_>>();
        let setup = ResolvedBossEncounterSetup {
            definition: encounter,
            actors,
            enemy_stat_scale: BattleUnitStatScale {
                max_health_percent: 150,
                attack_percent: 200,
                defense_percent: 125,
                exact_override: None,
                encounter_health_percent: 100,
                encounter_attack_percent: 100,
            },
            summon_spawn_zones: vec![ResolvedBossSummonSpawnZone {
                id: "fixture_zone".to_string(),
                cells: zone_cells,
            }],
            summon_routes: vec![ResolvedBossSummonRoute {
                id: "fixture_route".to_string(),
                cells: vec![
                    Position::new(5, 2),
                    Position::new(4, 2),
                    Position::new(3, 2),
                ],
            }],
        };
        let boss_group_id = ScenarioGroupId::new("boss_group");
        let player_group_id = ScenarioGroupId::new("player_group");
        let boss_spawns = (0..actor_count)
            .map(|actor_index| ScenarioUnitSpawn {
                unit_ref: ScenarioUnitRef::new(format!("boss_{actor_index}")),
                side: Side::Opponent,
                draft: fixture_draft(10 + actor_index as u128, BattleUnitThreatClass::Boss),
                position: Position::new(5, 1 + actor_index as i32),
                instance_salt: actor_index as u32,
            })
            .collect();
        let scenario = BattleScenario {
            battlefield: BattleFieldSpec {
            walkable_paths: Vec::new(),
                object_placement: None,
                width: 8,
                height: 6,
                valid_tiles: vec![],
                obstacles: vec![],
            },
            groups: vec![
                ScenarioSpawnGroup {
                    id: boss_group_id.clone(),
                    side: Side::Opponent,
                    required_for_victory: true,
                    enemy_movement_plan: Some(EnemyMovementPlan::PathAlongCells {
                        cells: vec![
                            Position::new(5, 2),
                            Position::new(4, 2),
                            Position::new(3, 2),
                            Position::new(2, 2),
                        ],
                    }),
                    spawns: boss_spawns,
                },
                ScenarioSpawnGroup {
                    id: player_group_id.clone(),
                    side: Side::Player,
                    required_for_victory: false,
                    enemy_movement_plan: None,
                    spawns: vec![ScenarioUnitSpawn {
                        unit_ref: ScenarioUnitRef::new("player_0"),
                        side: Side::Player,
                        draft: fixture_draft(100, BattleUnitThreatClass::Normal),
                        position: player_position,
                        instance_salt: 0,
                    }],
                },
            ],
            events: vec![
                ScenarioEvent {
                    id: ScenarioEventId::new("spawn_bosses"),
                    trigger: ScenarioTrigger::AtBattleStart,
                    action: ScenarioAction::SpawnGroup {
                        group_id: boss_group_id,
                    },
                },
                ScenarioEvent {
                    id: ScenarioEventId::new("spawn_player"),
                    trigger: ScenarioTrigger::AtBattleStart,
                    action: ScenarioAction::SpawnGroup {
                        group_id: player_group_id,
                    },
                },
            ],
            terminal_outcomes: BattleScenario::fixture_required_group_outcomes(),
            tactical_plan: TacticalPlan::default(),
            boss_runtime: Some(setup),
        };
        BattleCore::new_from_scenario(
            scenario,
            GameDataBuilder::empty()
                .with_boss_runtime_data(Arc::new(database))
                .build_arc(),
            77,
        )
    }

    fn start_fixture(mut core: BattleCore) -> BattleCore {
        let mut execution = core.start_battle_execution().unwrap();
        assert!(matches!(
            core.step_battle_execution(&mut execution),
            Ok(BattleStepOutcome::Running)
        ));
        core
    }

    fn boss_ids(core: &BattleCore) -> Vec<UnitInstanceId> {
        let mut ids = core
            .units
            .values()
            .filter(|unit| {
                unit.owner == Side::Opponent
                    && matches!(
                        unit.source_identity,
                        BattleUnitSourceIdentity::TestFixture { .. }
                    )
            })
            .map(|unit| unit.instance_id)
            .collect::<Vec<_>>();
        ids.sort();
        ids
    }

    fn player_id(core: &BattleCore) -> UnitInstanceId {
        core.units
            .values()
            .find(|unit| unit.owner == Side::Player)
            .unwrap()
            .instance_id
    }

    fn summoned_ids(core: &BattleCore) -> Vec<UnitInstanceId> {
        let mut ids = core
            .units
            .values()
            .filter(|unit| {
                matches!(
                    unit.source_identity,
                    BattleUnitSourceIdentity::SummonedUnit { .. }
                )
            })
            .map(|unit| unit.instance_id)
            .collect::<Vec<_>>();
        ids.sort_by_key(|unit_id| core.units[unit_id].spawn_order);
        ids
    }

    fn configure_zone_task(
        task: &mut BossTaskData,
        count: u32,
        insufficient: BossSummonInsufficientSpaceData,
    ) {
        let BossTaskData::Summon {
            count: task_count,
            placement,
            on_insufficient_space,
            ..
        } = task
        else {
            unreachable!()
        };
        *task_count = count;
        *placement = BossSummonPlacementData::SpawnZone {
            zone_id: "fixture_zone".to_string(),
        };
        *on_insufficient_space = insufficient;
    }

    fn configure_bound_removal_signal(
        catalog: &mut crate::game::data::boss_runtime_data::BossRuntimeCatalogData,
        signal_id: BossSignalId,
    ) {
        catalog.signals.push(BossSignalDefinitionData {
            id: signal_id.clone(),
        });
        let behavior = catalog
            .actor_behaviors
            .iter_mut()
            .find(|behavior| behavior.id.as_str() == "summoning_behavior")
            .unwrap();
        let BossControlNodeData::PatternCall { outcome_routes, .. } = &mut behavior.nodes[0] else {
            unreachable!()
        };
        outcome_routes
            .iter_mut()
            .find(|route| route.outcome_id.as_str() == "failed")
            .unwrap()
            .target_node_id = BossControlNodeId::new("signal_producer");
        behavior.nodes.push(BossControlNodeData::State {
            id: BossControlNodeId::new("signal_producer"),
            tasks: vec![BossTaskData::EmitSignal {
                id: BossTaskId::new("emit_removal_signal"),
                signal_id: signal_id.clone(),
            }],
            transitions: vec![BossControlTransitionData {
                id: BossTransitionId::new("removal_signal_emitted"),
                priority: 10,
                condition: BossConditionData::TaskResult {
                    task_id: BossTaskId::new("emit_removal_signal"),
                    result: BossTaskResultData::Completed,
                },
                target_node_id: BossControlNodeId::new("summoner_route_combat"),
            }],
        });
        let BossTaskData::Summon { removal_signal, .. } = summon_task_mut(catalog) else {
            unreachable!()
        };
        *removal_signal = Some(BossSummonRemovalSignalData {
            signal_id,
            mode: BossSummonRemovalModeData::Despawn,
        });
    }

    #[test]
    fn successful_batch_has_independent_scaled_stats_provenance_and_no_authority() {
        let core = start_fixture(fixture_core(
            1,
            Position::new(0, 0),
            vec![Position::new(6, 1), Position::new(6, 2)],
            |_, _| {},
        ));
        let boss_id = boss_ids(&core)[0];
        let summoned = summoned_ids(&core);
        assert_eq!(summoned.len(), 2);
        let mut sequences = vec![];
        let mut batches = vec![];
        for unit_id in summoned {
            let unit = &core.units[&unit_id];
            assert_eq!(unit.stats.max_health, 150);
            assert_eq!(unit.stats.attack, 24);
            assert_eq!(unit.stats.defense, 4);
            let BattleUnitSourceIdentity::SummonedUnit {
                definition_id,
                summoner_instance_id,
                summon_sequence,
                batch_uuid,
                ..
            } = &unit.source_identity
            else {
                unreachable!()
            };
            assert_eq!(definition_id, "fixture_minion");
            assert_eq!(*summoner_instance_id, boss_id);
            sequences.push(*summon_sequence);
            batches.push(*batch_uuid);
        }
        sequences.sort();
        assert_eq!(sequences, vec![1, 2]);
        assert_eq!(batches[0], batches[1]);
        assert_eq!(core.summon_runtime_debug_snapshot().counters[0].2, 2);
        assert_eq!(core.summon_runtime_debug_snapshot().counters[0].3, 2);
        assert_eq!(
            core.boss_runtime.as_ref().unwrap().authority_debug_counts(),
            (1, 3, 0),
            "actors: {:?}",
            core.boss_runtime
                .as_ref()
                .unwrap()
                .actor_unit_ids_for_test()
        );
        assert_eq!(core.event_log.version, BATTLE_EVENT_LOG_VERSION);
    }

    #[test]
    fn limit_and_space_failure_are_atomic_and_do_not_consume_uuid_or_total() {
        for (label, mut core) in [
            (
                "limit",
                fixture_core(
                    1,
                    Position::new(0, 0),
                    vec![Position::new(6, 1)],
                    |_, summoned| summoned.definitions[0].max_alive = 1,
                ),
            ),
            (
                "space",
                fixture_core(
                    1,
                    Position::new(6, 1),
                    vec![Position::new(6, 1)],
                    |catalog, _| {
                        configure_zone_task(
                            summon_task_mut(catalog),
                            2,
                            BossSummonInsufficientSpaceData::Fail,
                        );
                    },
                ),
            ),
        ] {
            core = start_fixture(core);
            let snapshot = core.summon_runtime_debug_snapshot();
            assert_eq!(summoned_ids(&core).len(), 0, "{label}");
            assert_eq!(snapshot.pending_count, 0, "{label}");
            assert_eq!(snapshot.batch_count, 0, "{label}");
            assert!(snapshot.counters.is_empty(), "{label}");
            assert_eq!(snapshot.next_uuid_index, 0, "{label}");
        }
    }

    #[test]
    fn borrowed_active_summon_identity_and_pending_copy_preserve_waiter() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut core = start_fixture(fixture_core(
            2,
            Position::new(6, 1),
            vec![Position::new(6, 1)],
            |catalog, _| {
                configure_zone_task(
                    summon_task_mut(catalog),
                    1,
                    BossSummonInsufficientSpaceData::WaitUntilAvailable {
                        timeout_ms: BossDurationValueData::Literal(100),
                    },
                );
            },
        ));
        let runtime = core.boss_runtime.as_ref().unwrap();
        let views = runtime.summon_task_views().unwrap();
        assert_eq!(views.len(), 2);
        for view in &views {
            assert!(runtime.summon_request_is_active(view));
            for field in 0..6 {
                let mut stale = view.clone();
                match field {
                    0 => stale.task.request_id.push_str("_stale"),
                    1 => stale.actor_runtime_key = usize::MAX,
                    2 => stale.behavior_activation_id = u64::MAX,
                    3 => stale.pattern_activation_id = u64::MAX,
                    4 => stale.state_activation_id = u64::MAX,
                    5 => stale.task_activation_id = u64::MAX,
                    _ => unreachable!(),
                }
                assert!(!runtime.summon_request_is_active(&stale), "field {field}");
            }
            // Historical matching does not compare these descriptive payloads.
            let mut descriptive = view.clone();
            descriptive.summoner_unit_id = Uuid::nil().into();
            descriptive.task.count += 1;
            assert!(runtime.summon_request_is_active(&descriptive));
        }
        let id = &views[0].task.request_id;
        let before = format!("{:?}", core.summon_runtime);
        let mut measured = core.clone();
        let (waiting, usage) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || measured.settle_one_summon_request(id, 0),
        )
        .unwrap();
        assert!(!waiting.unwrap());
        let failure = control::run(
            &ExecutionLimits {
                max_work: usage.work - 1,
                ..Default::default()
            },
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || core.settle_one_summon_request(id, 0),
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_work_limit");
        assert_eq!(failure.site, "owned_clone_copy");
        assert_eq!(format!("{:?}", core.summon_runtime), before);
        assert!(!core.settle_one_summon_request(id, 0).unwrap());
        core.withdraw_unit(player_id(&core), 10).unwrap();
        core.run_boss_runtime_boundary(10).unwrap();
        assert!(!core
            .boss_runtime
            .as_ref()
            .unwrap()
            .summon_request_is_active(&views[0]));
    }

    #[test]
    fn authoring_summon_view_projection_prices_complete_collection_and_empty_scans() {
        use crate::authoring::{
            clone_shape::CloneShape,
            control::{self, CancellationToken, CloneMeter, ExecutionLimits, ExecutionPhase},
        };
        let core = start_fixture(fixture_core(
            3,
            Position::new(6, 1),
            vec![Position::new(6, 1)],
            |catalog, _| {
                configure_zone_task(
                    summon_task_mut(catalog),
                    1,
                    BossSummonInsufficientSpaceData::WaitUntilAvailable {
                        timeout_ms: BossDurationValueData::Literal(100),
                    },
                );
            },
        ));
        let runtime = core.boss_runtime.as_ref().unwrap();
        let ordinary = runtime.summon_task_views().unwrap();
        assert_eq!(ordinary.len(), 3);
        let source = format!("{runtime:?}");
        let (actual, usage) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || runtime.summon_task_views(),
        )
        .unwrap();
        assert_eq!(format!("{:?}", actual.unwrap()), format!("{ordinary:?}"));
        // Independent completed-output visitor: no projection implementation is
        // reused. Traversal/order work is additional to native owned-copy work.
        let (_, native_usage) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                let mut meter = CloneMeter::default();
                ordinary.visit_clone_shape(&mut meter)?;
                meter.finish("view_owner_storage", "view_owner_copy")
            },
        )
        .unwrap();
        assert_eq!(usage.clone_slots_peak, native_usage.clone_slots_peak);
        assert!(usage.work > native_usage.work);
        let (exact, exact_usage) = control::run(
            &ExecutionLimits {
                max_work: usage.work,
                max_clone_slots: usage.clone_slots_peak,
                ..Default::default()
            },
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || runtime.summon_task_views(),
        )
        .unwrap();
        assert_eq!(format!("{:?}", exact.unwrap()), format!("{ordinary:?}"));
        assert_eq!(usage, exact_usage);
        for limits in [
            ExecutionLimits {
                max_work: usage.work - 1,
                ..Default::default()
            },
            ExecutionLimits {
                max_clone_slots: usage.clone_slots_peak - 1,
                ..Default::default()
            },
        ] {
            let failure = control::run(
                &limits,
                &CancellationToken::default(),
                ExecutionPhase::TrialExecution,
                || runtime.summon_task_views(),
            )
            .unwrap_err();
            assert!(matches!(
                failure.code,
                "authoring_work_limit" | "authoring_clone_limit"
            ));
            assert_eq!(format!("{runtime:?}"), source);
        }
        // Even with no selected view, eligibility scans must still consume fuel.
        let mut settled = runtime.clone();
        for view in &ordinary {
            assert!(settled
                .settle_summon_task(view, BossTaskResultData::Canceled, None)
                .unwrap());
        }
        let (empty, empty_usage) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || settled.summon_task_views(),
        )
        .unwrap();
        assert!(empty.unwrap().is_empty());
        assert_eq!(empty_usage.clone_slots_peak, 1);
        assert!(empty_usage.work > 2);
        let failure = control::run(
            &ExecutionLimits {
                max_work: 1,
                ..Default::default()
            },
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || settled.summon_task_views(),
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_work_limit");
        assert_eq!(failure.site, "boss_summon_view_actor_scan");
        assert!(settled.summon_task_views().unwrap().is_empty());
    }

    #[test]
    fn same_deadline_space_wins_and_multiple_waiters_use_stable_fifo() {
        let mut core = start_fixture(fixture_core(
            2,
            Position::new(6, 1),
            vec![Position::new(6, 1)],
            |catalog, summoned| {
                summoned.definitions[0].max_alive = 1;
                summoned.definitions[0].max_total = 1;
                configure_zone_task(
                    summon_task_mut(catalog),
                    1,
                    BossSummonInsufficientSpaceData::WaitUntilAvailable {
                        timeout_ms: BossDurationValueData::Literal(10),
                    },
                );
            },
        ));
        assert_eq!(core.summon_runtime_debug_snapshot().pending_count, 2);
        let expected_first = boss_ids(&core)[0];
        core.withdraw_unit(player_id(&core), 10).unwrap();
        core.run_boss_runtime_boundary(10).unwrap();

        let summoned = summoned_ids(&core);
        assert_eq!(summoned.len(), 1);
        let BattleUnitSourceIdentity::SummonedUnit {
            summoner_instance_id,
            ..
        } = core.units[&summoned[0]].source_identity
        else {
            unreachable!()
        };
        assert_eq!(summoner_instance_id, expected_first);
        assert_eq!(core.summon_runtime_debug_snapshot().pending_count, 0);
    }

    #[test]
    fn pending_request_cancels_on_owner_state_exit_without_consuming_identity_or_total() {
        let mut core = start_fixture(fixture_core(
            1,
            Position::new(6, 1),
            vec![Position::new(6, 1)],
            |catalog, _| {
                configure_zone_task(
                    summon_task_mut(catalog),
                    1,
                    BossSummonInsufficientSpaceData::WaitUntilAvailable {
                        timeout_ms: BossDurationValueData::Literal(100),
                    },
                );
                let idle_behavior = BossActorBehaviorDefinitionData {
                    id: BossActorBehaviorId::new("phase_two_idle"),
                    entry_node_id: BossControlNodeId::new("route_combat"),
                    nodes: vec![BossControlNodeData::State {
                        id: BossControlNodeId::new("route_combat"),
                        tasks: vec![BossTaskData::RouteCombat {
                            id: BossTaskId::new("phase_two_route_combat"),
                        }],
                        transitions: vec![],
                    }],
                };
                catalog.actor_behaviors.push(idle_behavior.clone());
                let phase_two_id = BossEncounterPhaseId::new("phase_two");
                catalog.encounters[0].phases[0].transitions =
                    vec![BossEncounterPhaseTransitionData {
                        id: BossTransitionId::new("exit_summon_state"),
                        priority: 10,
                        condition: BossEncounterConditionData::ActorHealthAtOrBelowPercent {
                            actor_slot_id: BossActorSlotId::new("primary"),
                            percent: 50,
                        },
                        timing: BossPhaseTransitionTimingData::Immediate,
                        target_phase_id: phase_two_id.clone(),
                    }];
                catalog.encounters[0].phases.push(BossEncounterPhaseData {
                    id: phase_two_id,
                    actor_behaviors: vec![BossPhaseActorBehaviorData {
                        actor_slot_id: BossActorSlotId::new("primary"),
                        behavior_id: idle_behavior.id,
                    }],
                    transitions: vec![],
                });
            },
        ));
        assert_eq!(core.summon_runtime_debug_snapshot().pending_count, 1);
        let summoner = boss_ids(&core)[0];
        core.units.get_mut(&summoner).unwrap().stats.current_health = 400;
        core.run_boss_runtime_boundary(1).unwrap();

        let snapshot = core.summon_runtime_debug_snapshot();
        assert_eq!(snapshot.pending_count, 0);
        assert_eq!(snapshot.batch_count, 0);
        assert!(snapshot.counters.is_empty());
        assert_eq!(snapshot.next_uuid_index, 0);
        assert!(core.units[&summoner].is_active());
        assert_eq!(
            core.boss_runtime
                .as_ref()
                .unwrap()
                .authority_debug_counts()
                .2,
            1
        );
    }

    #[test]
    fn every_occupant_is_checked_and_around_order_is_distance_then_y_then_x() {
        let mut core = start_fixture(fixture_core(
            1,
            Position::new(0, 0),
            vec![Position::new(5, 1)],
            |_, _| {},
        ));
        let summoner = boss_ids(&core)[0];
        let player = player_id(&core);
        for unit_id in summoned_ids(&core) {
            core.units.get_mut(&unit_id).unwrap().lifecycle = RuntimeUnitLifecycle::Despawned;
        }
        let summoner_tile = core.units[&summoner].body.projected_tile();
        core.units.get_mut(&player).unwrap().set_world_position(
            super::super::movement::types::WorldVec2::from_tile_center(summoner_tile),
        );
        let view = BossSummonTaskView {
            actor_runtime_key: 0,
            summoner_unit_id: summoner,
            behavior_activation_id: 1,
            pattern_activation_id: 1,
            state_activation_id: 1,
            task_activation_id: 1,
            task: ActiveBossSummonTask {
                request_id: "candidate_probe".to_string(),
                definition_id: SummonedUnitDefinitionId::new("fixture_minion"),
                count: 1,
                batch_handle_id: SummonBatchHandleId::new("candidate_probe"),
                placement: BossSummonPlacementData::SpawnZone {
                    zone_id: "fixture_zone".to_string(),
                },
                occupancy_policy: BossSummonOccupancyPolicyData::AllowOccupied {
                    allowed: vec![BossSummonOccupancyClassData::Summoner],
                },
                insufficient_space: ActiveBossSummonInsufficientSpace::Fail,
                route_id: Some("fixture_route".to_string()),
                lifetime: None,
                removal_signal: None,
                admitted_at_ms: 0,
            },
        };
        assert!(core.summon_candidate_placements(&view).unwrap().is_empty());
        let mut any = view.clone();
        any.task.occupancy_policy = BossSummonOccupancyPolicyData::AllowOccupied {
            allowed: vec![BossSummonOccupancyClassData::Any],
        };
        assert_eq!(core.summon_candidate_placements(&any).unwrap().len(), 1);

        let mut around = view;
        around.task.placement = BossSummonPlacementData::AroundSummoner { radius_tiles: 1 };
        around.task.occupancy_policy = BossSummonOccupancyPolicyData::RequireUnoccupied;
        let positions = core
            .summon_candidate_placements(&around)
            .unwrap()
            .into_iter()
            .map(|placement| placement.position)
            .collect::<Vec<_>>();
        assert_eq!(
            &positions[..3],
            &[
                Position::new(summoner_tile.x, summoner_tile.y - 1),
                Position::new(summoner_tile.x - 1, summoner_tile.y),
                Position::new(summoner_tile.x + 1, summoner_tile.y),
            ]
        );
    }

    #[test]
    fn per_summoner_limits_are_isolated_and_total_never_frees_after_clear() {
        let mut core = start_fixture(fixture_core(
            2,
            Position::new(0, 0),
            vec![Position::new(6, 1), Position::new(6, 2)],
            |catalog, _| {
                let behavior = catalog
                    .actor_behaviors
                    .iter_mut()
                    .find(|behavior| behavior.id.as_str() == "summoning_behavior")
                    .unwrap();
                let BossControlNodeData::PatternCall {
                    selector,
                    outcome_routes,
                    ..
                } = &mut behavior.nodes[0]
                else {
                    unreachable!()
                };
                let crate::game::data::boss_runtime_data::BossPatternSelectorData::FixedSequence {
                    repeat,
                    ..
                } = selector
                else {
                    unreachable!()
                };
                *repeat = true;
                let complete = outcome_routes
                    .iter_mut()
                    .find(|route| route.outcome_id.as_str() == "complete")
                    .unwrap();
                complete.target_node_id = BossControlNodeId::new("repeat_delay");
                behavior.nodes.push(BossControlNodeData::State {
                    id: BossControlNodeId::new("repeat_delay"),
                    tasks: vec![BossTaskData::Wait {
                        id: BossTaskId::new("repeat_delay_wait"),
                        duration_ms: BossDurationValueData::Literal(1),
                    }],
                    transitions: vec![BossControlTransitionData {
                        id: BossTransitionId::new("repeat_after_delay"),
                        priority: 10,
                        condition: BossConditionData::TaskResult {
                            task_id: BossTaskId::new("repeat_delay_wait"),
                            result: BossTaskResultData::Completed,
                        },
                        target_node_id: BossControlNodeId::new("summon_pattern_call"),
                    }],
                });
            },
        ));
        let first = core.summon_runtime_debug_snapshot();
        assert_eq!(first.counters.len(), 2);
        assert!(first
            .counters
            .iter()
            .all(|entry| (entry.2, entry.3) == (2, 2)));

        for round in 0..2 {
            let kill_time = 1 + round * 2;
            let active = summoned_ids(&core)
                .into_iter()
                .filter(|unit_id| core.units[unit_id].is_active())
                .collect::<Vec<_>>();
            for unit_id in active {
                core.finalize_unit_death(None, unit_id, Side::Opponent, kill_time);
            }
            core.run_boss_runtime_boundary(kill_time).unwrap();
            core.run_boss_runtime_boundary(kill_time + 1).unwrap();
        }
        let final_snapshot = core.summon_runtime_debug_snapshot();
        assert_eq!(final_snapshot.counters.len(), 2);
        assert!(final_snapshot
            .counters
            .iter()
            .all(|entry| (entry.2, entry.3) == (0, 4)));
        assert_eq!(final_snapshot.batch_count, 4);
        assert_eq!(final_snapshot.next_uuid_index, 12);
    }

    #[test]
    fn summoner_removal_respects_persist_and_despawn_without_death_hooks() {
        for (policy, should_persist) in [
            (SummonerRemovalPolicyData::Persist, true),
            (SummonerRemovalPolicyData::Despawn, false),
        ] {
            let mut core = start_fixture(fixture_core(
                1,
                Position::new(0, 0),
                vec![Position::new(6, 1), Position::new(6, 2)],
                |_, summoned| summoned.definitions[0].on_summoner_removed = policy,
            ));
            let minions = summoned_ids(&core);
            core.withdraw_unit(boss_ids(&core)[0], 1).unwrap();
            core.run_boss_runtime_boundary(1).unwrap();
            for unit_id in &minions {
                let unit = &core.units[unit_id];
                if should_persist {
                    assert!(unit.is_active());
                } else {
                    assert_eq!(unit.lifecycle, RuntimeUnitLifecycle::Despawned);
                    assert!(!core.graveyard.contains_key(unit_id));
                    assert!(!core.event_log.entries.iter().any(|entry| matches!(
                        entry.event,
                        BattleLogEvent::UnitDied { unit_instance_id, .. } if unit_instance_id == *unit_id
                    )));
                }
            }
        }
    }

    #[test]
    fn lifetime_modes_and_signal_binding_use_first_nonretroactive_cause() {
        for mode in [
            BossSummonRemovalModeData::Despawn,
            BossSummonRemovalModeData::Kill,
        ] {
            let mut core = start_fixture(fixture_core(
                1,
                Position::new(0, 0),
                vec![Position::new(6, 1), Position::new(6, 2)],
                |catalog, _| {
                    let BossTaskData::Summon { lifetime, .. } = summon_task_mut(catalog) else {
                        unreachable!()
                    };
                    *lifetime = Some(BossSummonLifetimeData {
                        lifetime_ms: BossDurationValueData::Literal(5),
                        mode,
                    });
                },
            ));
            let minions = summoned_ids(&core);
            core.run_boss_runtime_boundary(5).unwrap();
            for unit_id in minions {
                let unit = &core.units[&unit_id];
                match mode {
                    BossSummonRemovalModeData::Despawn => {
                        assert_eq!(unit.lifecycle, RuntimeUnitLifecycle::Despawned);
                        assert!(!core.graveyard.contains_key(&unit_id));
                    }
                    BossSummonRemovalModeData::Kill => {
                        assert_eq!(unit.lifecycle, RuntimeUnitLifecycle::Dead);
                        assert!(core.graveyard.contains_key(&unit_id));
                        assert!(core.event_log.entries.iter().any(|entry| matches!(
                            entry.event,
                            BattleLogEvent::UnitDied {
                                unit_instance_id,
                                killer_instance_id: None,
                                ..
                            } if unit_instance_id == unit_id
                        )));
                    }
                }
            }
        }

        let signal_id = BossSignalId::new("expire_minions");
        let mut core = start_fixture(fixture_core(
            1,
            Position::new(0, 0),
            vec![Position::new(6, 1), Position::new(6, 2)],
            |catalog, _| {
                configure_bound_removal_signal(catalog, signal_id.clone());
            },
        ));
        let minions = summoned_ids(&core);
        assert!(minions
            .iter()
            .all(|unit_id| core.units[unit_id].is_active()));
        core.boss_runtime
            .as_mut()
            .unwrap()
            .emit_signal_for_test(signal_id.clone());
        core.run_boss_runtime_boundary(1).unwrap();
        assert!(minions
            .iter()
            .all(|unit_id| { core.units[unit_id].lifecycle == RuntimeUnitLifecycle::Despawned }));

        let prebind_signal = BossSignalId::new("prebind_expire_minions");
        let mut core = start_fixture(fixture_core(
            1,
            Position::new(6, 1),
            vec![Position::new(6, 1)],
            |catalog, _| {
                configure_bound_removal_signal(catalog, prebind_signal.clone());
                configure_zone_task(
                    summon_task_mut(catalog),
                    1,
                    BossSummonInsufficientSpaceData::WaitUntilAvailable {
                        timeout_ms: BossDurationValueData::Literal(10),
                    },
                );
            },
        ));
        assert_eq!(core.summon_runtime_debug_snapshot().pending_count, 1);
        core.boss_runtime
            .as_mut()
            .unwrap()
            .emit_signal_for_test(prebind_signal.clone());
        core.withdraw_unit(player_id(&core), 2).unwrap();
        core.run_boss_runtime_boundary(2).unwrap();
        let minions = summoned_ids(&core);
        assert_eq!(minions.len(), 1);
        assert!(core.units[&minions[0]].is_active());

        core.boss_runtime
            .as_mut()
            .unwrap()
            .emit_signal_for_test(prebind_signal);
        core.run_boss_runtime_boundary(3).unwrap();
        assert_eq!(
            core.units[&minions[0]].lifecycle,
            RuntimeUnitLifecycle::Despawned
        );
    }

    #[test]
    fn batch_clear_is_emitted_once_for_the_exact_pattern_handle() {
        let mut core = start_fixture(fixture_core(
            1,
            Position::new(0, 0),
            vec![Position::new(6, 1), Position::new(6, 2)],
            |_, _| {},
        ));
        for unit_id in summoned_ids(&core) {
            core.finalize_unit_death(None, unit_id, Side::Opponent, 1);
        }
        core.run_boss_runtime_boundary(1).unwrap();
        let once = core.summon_runtime_debug_snapshot();
        assert_eq!(once.batches.len(), 1);
        assert!(once.batches[0].2);
        assert_eq!(once.counters[0].2, 0);
        assert_eq!(once.counters[0].3, 2);

        core.run_boss_runtime_boundary(1).unwrap();
        assert_eq!(core.summon_runtime_debug_snapshot(), once);
    }

    #[test]
    fn persisted_summons_do_not_gate_required_group_victory() {
        let mut core = fixture_core(
            1,
            Position::new(0, 0),
            vec![Position::new(6, 1), Position::new(6, 2)],
            |_, _| {},
        );
        let mut execution = core.start_battle_execution().unwrap();
        assert!(matches!(
            core.step_battle_execution(&mut execution),
            Ok(BattleStepOutcome::Running)
        ));
        let minions = summoned_ids(&core);
        assert!(minions
            .iter()
            .all(|unit_id| core.units[unit_id].is_active()));
        let boss = boss_ids(&core)[0];
        core.finalize_unit_death(None, boss, Side::Opponent, 1);

        let outcome = core.step_battle_execution_until(&mut execution, 1).unwrap();
        assert!(matches!(
            outcome,
            BattleStepOutcome::Finished(signal)
                if signal.winner == crate::game::battle::types::BattleWinner::Player
        ));
        assert!(minions
            .iter()
            .all(|unit_id| core.units[unit_id].is_active()));
    }
}
