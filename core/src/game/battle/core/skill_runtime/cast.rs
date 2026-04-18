use crate::{
    ecs::resources::Position,
    game::{
        ability::{SkillArea, SkillStepDef, SkillTarget, UnitTargetRule},
        battle::{
            core::{
                types::{
                    DeferredSkillStep, SkillImpactContext, SkillStepProgress, SkillStepResult,
                },
                BattleCore,
            },
            enums::BattleEvent,
            ids::UnitInstanceId,
            timeline::SkillCastTarget,
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

impl BattleCore {
    pub(in crate::game::battle::core) fn stored_skill_cast_anchor_position(
        &self,
        cast_seq: u64,
    ) -> Option<Position> {
        self.active_skill_casts
            .get(&cast_seq)
            .and_then(|cast_state| cast_state.cast_target_anchor_position)
    }

    fn cleanup_finished_skill_cast(&mut self, cast_seq: u64) {
        let should_remove = self
            .active_skill_casts
            .get(&cast_seq)
            .is_some_and(|cast_state| {
                cast_state.deferred_steps.is_empty()
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
        let Some(cast_state) = self.active_skill_casts.get_mut(&cast_seq) else {
            return;
        };

        let progress = cast_state
            .step_progress
            .entry(step_index)
            .or_insert_with(|| SkillStepProgress {
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
        let Some(cast_state) = self.active_skill_casts.get_mut(&cast_seq) else {
            return;
        };

        let progress = cast_state
            .step_progress
            .entry(step_index)
            .or_insert_with(|| SkillStepProgress {
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
        if let Some(cast_state) = self.active_skill_casts.get_mut(&cast_seq) {
            let progress = cast_state
                .step_progress
                .entry(step_index)
                .or_insert_with(|| SkillStepProgress {
                    result: SkillStepResult::default(),
                    started: true,
                    pending_delivery_count: 0,
                });
            progress.started = true;
            progress.pending_delivery_count = 0;
        }
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

        if let Some(cast_state) = self.active_skill_casts.get_mut(&cast_seq) {
            let progress = cast_state
                .step_progress
                .entry(step_index)
                .or_insert_with(|| SkillStepProgress {
                    result: SkillStepResult::default(),
                    started: true,
                    pending_delivery_count: usize::from(!terminal),
                });
            progress.started = true;
            if terminal && progress.pending_delivery_count > 0 {
                progress.pending_delivery_count -= 1;
            }
        }

        self.release_deferred_skill_steps(cast_seq, time_ms);
        self.cleanup_finished_skill_cast(cast_seq);
    }

    pub(in crate::game::battle::core) fn defer_skill_step(
        &mut self,
        cast_seq: u64,
        step_index: usize,
        deferred: DeferredSkillStep,
    ) {
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
            .map(|(_, context)| context.clone())
    }

    pub(in crate::game::battle::core) fn update_skill_cast_impact_context(
        &mut self,
        cast_seq: u64,
        step_index: usize,
        context: SkillImpactContext,
    ) {
        let Some(cast_state) = self.active_skill_casts.get_mut(&cast_seq) else {
            return;
        };
        cast_state
            .impact_contexts_by_step
            .insert(step_index, context.clone());
        cast_state.last_impact_context = Some(context);
    }

    pub(in crate::game::battle::core) fn choose_skill_target_by_rule(
        &self,
        caster_instance_id: UnitInstanceId,
        caster_owner: Side,
        caster_pos: Position,
        range_tiles: u8,
        rule: UnitTargetRule,
    ) -> Option<UnitInstanceId> {
        let in_range = |unit_id: UnitInstanceId| {
            self.battlefield
                .position_of(unit_id)
                .is_some_and(|pos| caster_pos.chebyshev(&pos) <= range_tiles as i32)
        };

        match rule {
            UnitTargetRule::CurrentTarget => {
                self.units
                    .get(&caster_instance_id)
                    .and_then(|caster| {
                        caster.current_target.filter(|id| {
                            self.is_alive_enemy(*id, caster_owner)
                                && self.battlefield.position_of(*id).is_some_and(|pos| {
                                    caster_pos.chebyshev(&pos) <= range_tiles as i32
                                })
                        })
                    })
                    .or_else(|| {
                        self.choose_skill_target_by_rule(
                            caster_instance_id,
                            caster_owner,
                            caster_pos,
                            range_tiles,
                            UnitTargetRule::Nearest,
                        )
                    })
            }
            UnitTargetRule::LowestHealthEnemy => {
                let mut best: Option<(u32, i32, UnitInstanceId)> = None;
                for unit in self.units.values() {
                    if unit.is_dead() || unit.owner == caster_owner {
                        continue;
                    }
                    let Some(unit_pos) = self.battlefield.position_of(unit.instance_id) else {
                        continue;
                    };
                    let distance = caster_pos.chebyshev(&unit_pos);
                    if distance > range_tiles as i32 {
                        continue;
                    }

                    match best {
                        None => {
                            best = Some((unit.stats.current_health, distance, unit.instance_id))
                        }
                        Some((best_hp, best_distance, best_id))
                            if unit.stats.current_health < best_hp
                                || (unit.stats.current_health == best_hp
                                    && distance < best_distance)
                                || (unit.stats.current_health == best_hp
                                    && distance == best_distance
                                    && unit.instance_id.as_bytes() < best_id.as_bytes()) =>
                        {
                            best = Some((unit.stats.current_health, distance, unit.instance_id));
                        }
                        _ => {}
                    }
                }
                best.map(|(_, _, id)| id)
            }
            UnitTargetRule::Nearest => {
                self.choose_enemy_target_in_tile_range(caster_owner, caster_pos, range_tiles)
            }
        }
        .filter(|id| in_range(*id))
    }

    pub(in crate::game::battle::core) fn resolve_skill_anchor_position(
        &self,
        caster_instance_id: UnitInstanceId,
        caster_owner: Side,
        caster_pos: Position,
        area: &SkillArea,
        range_tiles: u8,
        wants_allies: bool,
    ) -> Option<Position> {
        match area {
            SkillArea::All | SkillArea::RadiusChebyshev { .. } | SkillArea::Line { .. } => {
                if wants_allies {
                    Some(caster_pos)
                } else {
                    self.choose_skill_target_by_rule(
                        caster_instance_id,
                        caster_owner,
                        caster_pos,
                        range_tiles,
                        UnitTargetRule::Nearest,
                    )
                    .and_then(|id| self.battlefield.position_of(id))
                }
            }
        }
    }

    pub(in crate::game::battle::core) fn resolve_skill_step_targets(
        &self,
        cast_seq: u64,
        caster_instance_id: UnitInstanceId,
        step: &SkillStepDef,
        step_target: Option<SkillCastTarget>,
    ) -> Vec<UnitInstanceId> {
        let Some((caster_owner, caster_pos)) =
            self.resolve_cast_origin_context(Some(cast_seq), caster_instance_id, false)
        else {
            return Vec::new();
        };
        let cast_target_anchor_position = self.stored_skill_cast_anchor_position(cast_seq);

        let mut targets: Vec<UnitInstanceId> = Vec::new();

        match &step.target {
            SkillTarget::SelfUnit => targets.push(caster_instance_id),
            SkillTarget::EnemySingle { .. } => {
                if let Some(SkillCastTarget::Unit { unit_instance_id }) = step_target {
                    if self.is_alive_enemy(unit_instance_id, caster_owner) {
                        targets.push(unit_instance_id);
                    }
                }
            }
            SkillTarget::Allies { area } | SkillTarget::Enemies { area } => {
                let wants_allies = matches!(step.target, SkillTarget::Allies { .. });
                let anchor = match step_target {
                    Some(SkillCastTarget::Tile { position }) => Some(position),
                    Some(SkillCastTarget::Unit { unit_instance_id }) => self
                        .battlefield
                        .position_of(unit_instance_id)
                        .or(cast_target_anchor_position),
                    _ => cast_target_anchor_position,
                };
                let Some(anchor) = anchor else {
                    return Vec::new();
                };

                for unit in self.units.values() {
                    if unit.is_dead() {
                        continue;
                    }

                    let is_ally = unit.owner == caster_owner;
                    if wants_allies != is_ally {
                        continue;
                    }

                    let Some(pos) = self.battlefield.position_of(unit.instance_id) else {
                        continue;
                    };

                    match area {
                        SkillArea::All => {}
                        SkillArea::RadiusChebyshev { radius_tiles } => {
                            if anchor.chebyshev(&pos) > *radius_tiles as i32 {
                                continue;
                            }
                        }
                        SkillArea::Line { length_tiles } => {
                            if !Self::is_point_on_skill_line(
                                caster_pos,
                                anchor,
                                pos,
                                *length_tiles as i32,
                                caster_owner,
                            ) {
                                continue;
                            }
                        }
                    }

                    targets.push(unit.instance_id);
                }
            }
        }

        targets.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        targets
    }
}
