use crate::authoring::control;
use crate::{
    game::resources::Position,
    game::{
        ability::{SkillDef, SkillStepDef, SkillTarget, UnitTargetRule},
        battle::{
            core::{
                movement::types::WorldVec2,
                types::{
                    DeferredSkillStep, SkillImpactContext, SkillStepProgress, SkillStepResult,
                },
                BattleCore,
            },
            enums::BattleEvent,
            event_log::SkillCastTarget,
            ids::UnitInstanceId,
            tile_range::{TileRangePattern, TileRangePolicy},
        },
        enums::Side,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::game::battle::core) enum PreviousStepDamageGate {
    Satisfied,
    Unsatisfied,
    Pending,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::game::battle::core) enum SkillTargetUsefulness<'a> {
    Cast {
        time_ms: u64,
        skill: &'a SkillDef,
    },
    DeliveryStep {
        time_ms: u64,
        step_index: usize,
        step: &'a SkillStepDef,
    },
}

impl BattleCore {
    pub(in crate::game::battle::core) fn stored_skill_cast_anchor_position(
        &self,
        cast_seq: u64,
    ) -> Option<Position> {
        self.active_skill_casts
            .get(&cast_seq)
            .and_then(|cast_state| cast_state.cast_target_anchor_position)
    }

    pub(in crate::game::battle::core) fn stored_skill_cast_anchor_world_position(
        &self,
        cast_seq: u64,
    ) -> Option<WorldVec2> {
        self.active_skill_casts
            .get(&cast_seq)
            .and_then(|cast_state| cast_state.cast_target_anchor_world_position)
    }

    pub(in crate::game::battle::core) fn cleanup_finished_skill_cast(&mut self, cast_seq: u64) {
        if let Some(cast) = self.active_skill_casts.get_mut(&cast_seq) {
            if let Some(flow) = cast
                .flow
                .as_ref()
                .filter(|flow| !flow.plan.legacy || !flow.plan.side_branches.is_empty())
            {
                let work = cast
                    .step_progress
                    .len()
                    .saturating_mul(flow.nodes.len())
                    .saturating_add(cast.impact_contexts_by_step.len());
                if control::charge(work as u64, "flow_completed_occurrence_prune").is_err() {
                    return;
                }
                let before = cast.step_progress.len();
                cast.step_progress.retain(|occurrence, progress| {
                    !progress.is_terminal() || flow.retains_occurrence(*occurrence)
                });
                cast.total_steps -= before - cast.step_progress.len();
                // One latest impact remains an anchor for a following authored
                // leaf; completed repeat history does not grow with iterations.
                let latest = cast.impact_contexts_by_step.keys().next_back().copied();
                cast.impact_contexts_by_step.retain(|index, _| {
                    Some(*index) == latest || cast.step_progress.contains_key(index)
                });
            }
        }
        let size = self
            .active_skill_casts
            .get(&cast_seq)
            .map_or(0, |cast| cast.step_progress.len());
        if control::charge(size as u64 + 1, "cast_cleanup_scan").is_err() {
            return;
        }
        let should_remove = self
            .active_skill_casts
            .get(&cast_seq)
            .is_some_and(|cast_state| {
                cast_state
                    .flow
                    .as_ref()
                    .is_none_or(|flow| flow.complete && !flow.has_pending_sides())
                    && cast_state.deferred_steps.is_empty()
                    && cast_state.active_area_ids.is_empty()
                    && cast_state.step_progress.len() == cast_state.total_steps
                    && cast_state
                        .step_progress
                        .values()
                        .all(SkillStepProgress::is_terminal)
            });

        if should_remove {
            self.active_skill_casts.remove(&cast_seq);
        }
    }

    pub(in crate::game::battle::core) fn begin_skill_step_delivery(
        &mut self,
        cast_seq: u64,
        step_index: usize,
        delivery_count: usize,
    ) {
        let growth = usize::from(
            self.active_skill_casts
                .get(&cast_seq)
                .is_some_and(|cast| !cast.step_progress.contains_key(&step_index)),
        );
        if self
            .admit_authoring_runtime_growth(growth, "cast_child_record")
            .is_err()
        {
            return;
        }
        let Some(cast_state) = self.active_skill_casts.get_mut(&cast_seq) else {
            return;
        };

        let progress = cast_state
            .step_progress
            .entry(step_index)
            .or_insert_with(|| SkillStepProgress {
                use_owned: false,
                result: SkillStepResult::default(),
                started: true,
                pending_delivery_count: 0,
            });
        progress.started = true;
        progress.pending_delivery_count = progress
            .pending_delivery_count
            .saturating_add(delivery_count);
    }

    pub(in crate::game::battle::core) fn record_skill_step_result(
        &mut self,
        cast_seq: u64,
        step_index: usize,
        result: SkillStepResult,
    ) {
        let growth = usize::from(
            self.active_skill_casts
                .get(&cast_seq)
                .is_some_and(|cast| !cast.step_progress.contains_key(&step_index)),
        );
        if self
            .admit_authoring_runtime_growth(growth, "cast_child_record")
            .is_err()
        {
            return;
        }
        let Some(cast_state) = self.active_skill_casts.get_mut(&cast_seq) else {
            return;
        };

        let progress = cast_state
            .step_progress
            .entry(step_index)
            .or_insert_with(|| SkillStepProgress {
                use_owned: false,
                result: SkillStepResult::default(),
                started: true,
                pending_delivery_count: 0,
            });
        progress.started = true;
        progress.result.merge(&result);
    }

    pub(in crate::game::battle::core) fn finalize_skill_step(
        &mut self,
        cast_seq: u64,
        step_index: usize,
        result: SkillStepResult,
        time_ms: u64,
    ) {
        self.record_skill_step_result(cast_seq, step_index, result);
        if control::poll("skill_result_recorded").is_err() {
            return;
        }
        let sustained_children = self.sustained_flow_children(cast_seq, step_index);
        if let Some(cast_state) = self.active_skill_casts.get_mut(&cast_seq) {
            let progress = cast_state
                .step_progress
                .entry(step_index)
                .or_insert_with(|| SkillStepProgress {
                    use_owned: false,
                    result: SkillStepResult::default(),
                    started: true,
                    pending_delivery_count: 0,
                });
            progress.started = true;
            progress.pending_delivery_count = sustained_children;
        }
        self.wake_skill_flow(cast_seq, time_ms);
        self.release_deferred_skill_steps(cast_seq, time_ms);
        self.cleanup_finished_skill_cast(cast_seq);
    }

    pub(in crate::game::battle::core) fn resolve_skill_step_delivery(
        &mut self,
        cast_seq: u64,
        step_index: usize,
        result: SkillStepResult,
        terminal: bool,
        time_ms: u64,
    ) {
        self.record_skill_step_result(cast_seq, step_index, result);
        if control::poll("skill_result_recorded").is_err() {
            return;
        }

        if let Some(cast_state) = self.active_skill_casts.get_mut(&cast_seq) {
            let progress = cast_state
                .step_progress
                .entry(step_index)
                .or_insert_with(|| SkillStepProgress {
                    use_owned: false,
                    result: SkillStepResult::default(),
                    started: true,
                    pending_delivery_count: usize::from(!terminal),
                });
            progress.started = true;
            if terminal && progress.pending_delivery_count > 0 {
                progress.pending_delivery_count -= 1;
            }
        }

        self.wake_skill_flow(cast_seq, time_ms);
        self.release_deferred_skill_steps(cast_seq, time_ms);
        self.cleanup_finished_skill_cast(cast_seq);
    }

    pub(in crate::game::battle::core) fn finish_skill_step_delivery(
        &mut self,
        cast_seq: u64,
        step_index: usize,
        time_ms: u64,
    ) {
        if let Some(progress) = self
            .active_skill_casts
            .get_mut(&cast_seq)
            .and_then(|cast| cast.step_progress.get_mut(&step_index))
        {
            progress.pending_delivery_count = progress.pending_delivery_count.saturating_sub(1);
        }
        self.wake_skill_flow(cast_seq, time_ms);
        self.release_deferred_skill_steps(cast_seq, time_ms);
        self.cleanup_finished_skill_cast(cast_seq);
    }

    pub(in crate::game::battle::core) fn defer_skill_step(
        &mut self,
        cast_seq: u64,
        step_index: usize,
        deferred: DeferredSkillStep,
    ) {
        let growth = usize::from(
            self.active_skill_casts
                .get(&cast_seq)
                .is_some_and(|cast| !cast.deferred_steps.contains_key(&step_index)),
        );
        if self
            .admit_authoring_runtime_growth(growth, "cast_child_record")
            .is_err()
        {
            return;
        }
        let Some(cast_state) = self.active_skill_casts.get_mut(&cast_seq) else {
            return;
        };
        cast_state.deferred_steps.insert(step_index, deferred);
    }

    pub(in crate::game::battle::core) fn previous_step_damage_gate(
        &self,
        cast_seq: u64,
        step_index: usize,
    ) -> PreviousStepDamageGate {
        if step_index == 0 {
            return PreviousStepDamageGate::Unsatisfied;
        }

        let Some(cast_state) = self.active_skill_casts.get(&cast_seq) else {
            return PreviousStepDamageGate::Unsatisfied;
        };
        let Some(progress) = cast_state.step_progress.get(&(step_index - 1)) else {
            return PreviousStepDamageGate::Pending;
        };

        if progress.result.dealt_damage() {
            PreviousStepDamageGate::Satisfied
        } else if progress.is_terminal() {
            PreviousStepDamageGate::Unsatisfied
        } else {
            PreviousStepDamageGate::Pending
        }
    }

    pub(in crate::game::battle::core) fn release_deferred_skill_steps(
        &mut self,
        cast_seq: u64,
        time_ms: u64,
    ) {
        let size = self
            .active_skill_casts
            .get(&cast_seq)
            .map_or(0, |cast| cast.deferred_steps.len());
        if control::charge(size as u64 + 1, "deferred_step_scan").is_err() {
            return;
        }
        let ready_steps = self
            .active_skill_casts
            .get(&cast_seq)
            .map(|cast_state| {
                cast_state
                    .deferred_steps
                    .keys()
                    .copied()
                    .filter(|step_index| {
                        !matches!(
                            self.previous_step_damage_gate(cast_seq, *step_index),
                            PreviousStepDamageGate::Pending
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        for step_index in ready_steps {
            if control::charge(1, "deferred_step_release").is_err() {
                return;
            }
            let deferred = self
                .active_skill_casts
                .get_mut(&cast_seq)
                .and_then(|cast_state| cast_state.deferred_steps.remove(&step_index));
            let Some(deferred) = deferred else {
                continue;
            };
            self.event_queue.push(BattleEvent::SkillStep {
                time_ms,
                cast_seq,
                step_index,
                caster_instance_id: deferred.caster_instance_id,
                skill_id: deferred.skill_id,
                step_id: deferred.step_id,
                cast_target: deferred.cast_target,
                cause: deferred.cause,
            });
        }

        self.cleanup_finished_skill_cast(cast_seq);
    }

    pub(in crate::game::battle::core) fn impact_context_before_step(
        &self,
        cast_seq: u64,
        step_index: usize,
    ) -> Option<SkillImpactContext> {
        let cast_state = self.active_skill_casts.get(&cast_seq)?;
        cast_state
            .impact_contexts_by_step
            .range(..step_index)
            .next_back()
            .and_then(|(_, context)| {
                control::charge(
                    context.hit_unit_ids.len() as u64 + 1,
                    "prior_impact_context_copy",
                )
                .ok()?;
                Some(context.clone())
            })
    }

    pub(in crate::game::battle::core) fn update_skill_cast_impact_context(
        &mut self,
        cast_seq: u64,
        step_index: usize,
        context: SkillImpactContext,
    ) {
        if control::charge(context.hit_unit_ids.len() as u64 + 1, "impact_context_copy").is_err() {
            return;
        }
        let Some(cast) = self.active_skill_casts.get(&cast_seq) else {
            return;
        };
        let prior = cast.impact_contexts_by_step.get(&step_index);
        let last = cast.last_impact_context.as_ref();
        let removed = prior
            .map_or(0, |context| context.hit_unit_ids.len())
            .saturating_add(last.map_or(0, |context| context.hit_unit_ids.len()));
        let additional = context
            .hit_unit_ids
            .len()
            .saturating_mul(2)
            .saturating_add(usize::from(prior.is_none()))
            .saturating_add(usize::from(last.is_none()));
        if self
            .admit_authoring_runtime_replacement(removed, additional, "impact_context_storage")
            .is_err()
        {
            return;
        }
        let Some(cast_state) = self.active_skill_casts.get_mut(&cast_seq) else {
            return;
        };
        cast_state
            .impact_contexts_by_step
            .insert(step_index, context.clone());
        cast_state.last_impact_context = Some(context);
    }

    pub(in crate::game::battle::core) fn is_alive_ally(
        &self,
        id: UnitInstanceId,
        owner: Side,
    ) -> bool {
        self.units
            .get(&id)
            .is_some_and(|unit| unit.owner == owner && unit.is_active())
    }

    pub(in crate::game::battle::core) fn choose_ally_skill_target(
        &self,
        caster: UnitInstanceId,
        reach: crate::game::ability::SkillTargetReach,
        range: Option<&TileRangePattern>,
        air_capable: bool,
        usefulness: SkillTargetUsefulness<'_>,
    ) -> Option<UnitInstanceId> {
        control::charge(self.units.len() as u64 + 1, "ally_skill_candidates").ok()?;
        let owner = self.units.get(&caster)?.owner;
        self.authoring_target_candidates()
            .ok()?
            .filter(|unit| {
                if !self.is_alive_ally(unit.instance_id, owner)
                    || !self.single_target_can_target_unit(unit.instance_id, air_capable)
                    || !self.is_target_in_skill_reach(caster, unit.instance_id, reach, range)
                {
                    return false;
                }
                match usefulness {
                    SkillTargetUsefulness::Cast { time_ms, skill } => self
                        .skill_has_useful_automatic_cast_target(
                            time_ms,
                            skill,
                            caster,
                            Some(SkillCastTarget::Unit {
                                unit_instance_id: unit.instance_id,
                            }),
                        ),
                    SkillTargetUsefulness::DeliveryStep { step, .. } => {
                        self.is_skill_step_useful_ally_target(caster, step, unit.instance_id)
                    }
                }
            })
            .min_by(|a, b| {
                (u64::from(a.stats.current_health) * u64::from(b.stats.max_health.max(1)))
                    .cmp(
                        &(u64::from(b.stats.current_health) * u64::from(a.stats.max_health.max(1))),
                    )
                    .then_with(|| {
                        self.basic_attack_target_distance_sq_world(caster, a.instance_id)
                            .unwrap_or(f32::INFINITY)
                            .total_cmp(
                                &self
                                    .basic_attack_target_distance_sq_world(caster, b.instance_id)
                                    .unwrap_or(f32::INFINITY),
                            )
                    })
                    .then_with(|| a.instance_id.cmp(&b.instance_id))
            })
            .map(|unit| unit.instance_id)
    }

    pub(in crate::game::battle::core) fn choose_skill_target_by_rule(
        &self,
        caster_instance_id: UnitInstanceId,
        caster_owner: Side,
        caster_pos: Position,
        range_policy: TileRangePolicy,
        defense_tile_range: Option<&TileRangePattern>,
        rule: UnitTargetRule,
        air_capable: bool,
        usefulness: SkillTargetUsefulness<'_>,
    ) -> Option<UnitInstanceId> {
        if control::charge(self.units.len() as u64 + 1, "skill_target_candidates").is_err() {
            return None;
        }
        let in_range = |unit_id: UnitInstanceId| {
            if !self.single_target_can_target_unit(unit_id, air_capable) {
                return false;
            }
            self.is_target_in_tile_range_policy(
                caster_instance_id,
                unit_id,
                range_policy,
                defense_tile_range,
            )
        };
        let is_useful = |unit_id: UnitInstanceId| {
            let target = Some(SkillCastTarget::Unit {
                unit_instance_id: unit_id,
            });
            match usefulness {
                SkillTargetUsefulness::Cast { time_ms, skill } => self
                    .skill_has_useful_automatic_cast_target(
                        time_ms,
                        skill,
                        caster_instance_id,
                        target,
                    ),
                SkillTargetUsefulness::DeliveryStep {
                    time_ms,
                    step_index,
                    step,
                } => self.skill_step_has_useful_actual_delivery_target(
                    time_ms,
                    step_index,
                    caster_instance_id,
                    step,
                    target,
                ),
            }
        };

        match rule {
            UnitTargetRule::CurrentTarget => self
                .units
                .get(&caster_instance_id)
                .and_then(|caster| {
                    caster.current_target.filter(|id| {
                        self.is_alive_enemy(*id, caster_owner) && in_range(*id) && is_useful(*id)
                    })
                })
                .or_else(|| {
                    control::poll("skill_target_fallback").ok()?;
                    self.choose_skill_target_by_rule(
                        caster_instance_id,
                        caster_owner,
                        caster_pos,
                        range_policy,
                        defense_tile_range,
                        UnitTargetRule::Nearest,
                        air_capable,
                        usefulness,
                    )
                }),
            UnitTargetRule::LowestHealthEnemy => {
                let Some(caster_body) = self.unit_body_view(caster_instance_id) else {
                    return None;
                };
                let mut best: Option<(u32, f32, UnitInstanceId)> = None;
                for unit in self.authoring_target_candidates().ok()? {
                    control::poll("skill_target_candidate").ok()?;
                    if !unit.is_active() || unit.owner == caster_owner {
                        continue;
                    }
                    let Some(target_body) = self.unit_body_view(unit.instance_id) else {
                        continue;
                    };
                    if !in_range(unit.instance_id) || !is_useful(unit.instance_id) {
                        continue;
                    }
                    let distance = caster_body.position.distance_squared(target_body.position);

                    match best {
                        None => {
                            best = Some((unit.stats.current_health, distance, unit.instance_id))
                        }
                        Some((best_hp, best_distance, best_id))
                            if unit.stats.current_health < best_hp
                                || (unit.stats.current_health == best_hp
                                    && distance < best_distance)
                                || (unit.stats.current_health == best_hp
                                    && distance.total_cmp(&best_distance).is_eq()
                                    && unit.instance_id.as_bytes() < best_id.as_bytes()) =>
                        {
                            best = Some((unit.stats.current_health, distance, unit.instance_id));
                        }
                        _ => {}
                    }
                }
                best.map(|(_, _, id)| id)
            }
            UnitTargetRule::Nearest => self
                .units
                .get(&caster_instance_id)
                .and_then(|caster| {
                    caster.current_target.filter(|id| {
                        self.is_alive_enemy(*id, caster_owner) && in_range(*id) && is_useful(*id)
                    })
                })
                .or_else(|| {
                    control::poll("skill_target_fallback").ok()?;
                    let Some(caster_body) = self.unit_body_view(caster_instance_id) else {
                        return None;
                    };
                    let mut best: Option<(f32, UnitInstanceId)> = None;
                    for unit in self.authoring_target_candidates().ok()? {
                        control::poll("skill_target_candidate").ok()?;
                        if !unit.is_active()
                            || unit.owner == caster_owner
                            || !in_range(unit.instance_id)
                            || !is_useful(unit.instance_id)
                        {
                            continue;
                        }
                        let Some(target_body) = self.unit_body_view(unit.instance_id) else {
                            continue;
                        };
                        let distance = caster_body.position.distance_squared(target_body.position);
                        match best {
                            None => best = Some((distance, unit.instance_id)),
                            Some((best_distance, _)) if distance < best_distance => {
                                best = Some((distance, unit.instance_id));
                            }
                            Some((best_distance, best_id))
                                if distance.total_cmp(&best_distance).is_eq()
                                    && unit.instance_id.as_bytes() < best_id.as_bytes() =>
                            {
                                best = Some((distance, unit.instance_id));
                            }
                            _ => {}
                        }
                    }
                    best.map(|(_, id)| id)
                }),
        }
        .filter(|id| {
            control::poll("skill_target_result").is_ok() && in_range(*id) && is_useful(*id)
        })
    }

    pub(in crate::game::battle::core) fn resolve_skill_step_targets(
        &self,
        cast_seq: u64,
        caster_instance_id: UnitInstanceId,
        step: &SkillStepDef,
        step_target: Option<SkillCastTarget>,
    ) -> Vec<UnitInstanceId> {
        if control::charge(1, "skill_step_target").is_err() {
            return Vec::new();
        }
        let Some((caster_owner, _caster_pos)) =
            self.resolve_cast_origin_context(Some(cast_seq), caster_instance_id, false)
        else {
            return Vec::new();
        };

        let mut targets: Vec<UnitInstanceId> = Vec::new();

        match &step.target {
            SkillTarget::SelfUnit => targets.push(caster_instance_id),
            SkillTarget::AllySingle => {
                if let Some(SkillCastTarget::Unit { unit_instance_id }) = step_target {
                    if self.is_alive_ally(unit_instance_id, caster_owner)
                        && self.is_skill_step_useful_ally_target(
                            caster_instance_id,
                            step,
                            unit_instance_id,
                        )
                    {
                        targets.push(unit_instance_id);
                    }
                }
            }
            SkillTarget::EnemySingle { .. } => {
                if let Some(SkillCastTarget::Unit { unit_instance_id }) = step_target {
                    if self.is_alive_enemy(unit_instance_id, caster_owner)
                        && self.is_skill_step_useful_hostile_target(
                            caster_instance_id,
                            step,
                            unit_instance_id,
                        )
                    {
                        targets.push(unit_instance_id);
                    }
                }
            }
            SkillTarget::CastTarget => {
                if let Some(SkillCastTarget::Unit { unit_instance_id }) = step_target {
                    if self
                        .units
                        .get(&unit_instance_id)
                        .is_some_and(|unit| unit.is_active())
                    {
                        targets.push(unit_instance_id);
                    }
                }
            }
        }

        targets.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        targets
    }
}
