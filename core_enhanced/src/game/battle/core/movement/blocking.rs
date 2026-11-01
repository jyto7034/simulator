use std::collections::{HashMap, HashSet};

use crate::game::{
    battle::{
        core::{BattleCore, BlockRuntimeState},
        ids::UnitInstanceId,
        scenario::{BattleObjective, EnemyMovementPlan, PlayerMovementPlan},
        types::BattleUnitRole,
    },
    enums::Side,
};

use super::{
    engine::MovementOutput,
    path::{route_progress_along_cells, route_total_length},
    types::WorldVec2,
};

// One millionth of a squared tile only absorbs f32 contact-point rounding.
const CONTACT_DISTANCE_SQ_EPSILON: f32 = 0.000001;

fn block_refresh_overflow(site: &'static str) -> crate::game::behavior::GameError {
    use crate::game::battle::runtime_control::{
        self as native, BattleRuntimeFault, RuntimeFaultCode,
    };
    native::reject(BattleRuntimeFault {
        code: RuntimeFaultCode::CounterOverflow,
        site,
        limit: None,
        requested: None,
    })
}

fn segment_circle_contact(
    from: WorldVec2,
    to: WorldVec2,
    center: WorldVec2,
    radius: f32,
) -> Option<f32> {
    let x = f64::from(from.x) - f64::from(center.x);
    let y = f64::from(from.y) - f64::from(center.y);
    let dx = f64::from(to.x) - f64::from(from.x);
    let dy = f64::from(to.y) - f64::from(from.y);
    let c = x * x + y * y - f64::from(radius) * f64::from(radius);
    if c <= 0.0 {
        return Some(0.0);
    }
    let a = dx * dx + dy * dy;
    if a <= f64::EPSILON {
        return None;
    }
    let b = x * dx + y * dy;
    let discriminant = b * b - a * c;
    if discriminant < 0.0 {
        return None;
    }
    let t = (-b - discriminant.sqrt()) / a;
    (0.0..=1.0).contains(&t).then_some(t as f32)
}

fn output_unit(output: &MovementOutput) -> UnitInstanceId {
    match *output {
        MovementOutput::BodyMoved { unit_id, .. }
        | MovementOutput::MovementStopped { unit_id }
        | MovementOutput::TargetReached { unit_id, .. } => unit_id,
    }
}

// Swept disk against an occupied tile. This changes only the actual movement
// segment; it does not add a static obstacle or rewrite the authored route.
fn segment_wall_contact(
    from: WorldVec2,
    to: WorldVec2,
    center: WorldVec2,
    body_radius: f32,
) -> Option<f32> {
    let half = f64::from(0.5 + body_radius.max(0.0));
    let mut enter = 0.0f64;
    let mut exit = 1.0f64;
    for (start, end, middle) in [(from.x, to.x, center.x), (from.y, to.y, center.y)] {
        let start = f64::from(start) - f64::from(middle);
        let delta = f64::from(end) - f64::from(middle) - start;
        if delta.abs() <= f64::EPSILON {
            if start.abs() > half {
                return None;
            }
        } else {
            let a = (-half - start) / delta;
            let b = (half - start) / delta;
            enter = enter.max(a.min(b));
            exit = exit.min(a.max(b));
            if enter > exit {
                return None;
            }
        }
    }
    Some(enter as f32)
}

impl BattleCore {
    fn enemy_block_demand(&self,id:UnitInstanceId)->u32 {self.shot_cohort_block_demand(id).unwrap_or(1)}
    fn occupied_block_capacity(&self,id:UnitInstanceId)->u32 {self.block_state.blocker_to_enemies.get(&id).map_or(0,|enemies|enemies.iter().fold(0u32,|total,enemy|total.saturating_add(self.enemy_block_demand(*enemy))))}

    // Only the new HP-form evolution boundary opts into this admission. The
    // generic reconciler retains its existing callers and authority semantics.
    pub(in crate::game::battle::core) fn refresh_independent_form_block_state(
        &mut self,
    ) -> Result<(), crate::game::behavior::GameError> {
        use crate::{authoring::control, game::battle::runtime_control as native};
        // Price the allocation-free route-length discovery before its table walk.
        let discovery = self
            .units_clone_history
            .bucket_bound()
            .ok_or_else(|| block_refresh_overflow("object_block_refresh_discovery"))?;
        control::charge(discovery as u64, "object_block_refresh_discovery")?;
        let (work, workspace) = self.independent_form_block_refresh_bounds()?;
        control::charge(work, "object_block_refresh_work")?;
        native::admit_retained_growth(0, Some(workspace), "object_block_refresh_workspace")?;
        self.refresh_block_state();
        // Segmented capacity lookup already charges through shared control.
        // Its sticky refusal must roll back the enclosing evolution candidate.
        control::poll("object_block_refresh_complete")?;
        native::poll()
    }

    pub(in crate::game::battle::core) fn independent_form_block_refresh_bounds(
        &self,
    ) -> Result<(u64, usize), crate::game::behavior::GameError> {
        use crate::owned_collections::MapCloneHistory;
        let bound = || -> Option<(u64, usize)> {
            let n = self.units.len();
            let m = self.block_state.enemy_to_blocker.len();
            let units = self.units_clone_history.bucket_bound()?;
            let old_pairs = self.block_state.enemy_to_blocker.clone_bucket_bound()?;
            let old_blockers = self.block_state.blocker_to_enemies.clone_bucket_bound()?;
            let buffs = self.buffs.clone_bucket_bound()?;
            let fresh = MapCloneHistory::from_tombstone_free_capacity(n).bucket_bound()?;
            let EnemyMovementPlan::PathAlongCells { cells } =
                &self.scenario.tactical_plan.enemy_plan;
            let mut route = cells.len();
            for unit in self.units.values() {
                if let Some(EnemyMovementPlan::PathAlongCells { cells }) = &unit.enemy_movement_plan
                {
                    route = route.max(cells.len());
                }
            }
            let square = n.checked_mul(n)?;
            let old_square = m.checked_mul(m)?;
            // Stable sorts use at most k² comparisons: enemy order, previous
            // engagements and all per-blocker lists (sum k² <= n²).
            let route_comparisons = square.checked_mul(2)?.checked_add(old_square)?;
            let route_work = route.checked_mul(4)?.checked_add(1)?;
            let stages = [
                units.checked_mul(2)?,                      // blocker/enemy collection
                n,                                          // initial capacities
                old_pairs,                                  // previous engagement collection
                square.checked_add(old_square)?,            // ID ordering
                route_comparisons.checked_mul(route_work)?, // centers + segments
                n.checked_add(m)?.checked_mul(2)?.checked_mul(buffs)?, // control state
                m,                                          // preserve valid existing reservations
                n.checked_add(square.checked_mul(2)?)?,     // new candidate search/min
                fresh.checked_add(n)?,                      // per-blocker list sort walk
                old_blockers
                    .checked_add(old_pairs)?
                    .checked_add(n)?
                    .checked_add(m)?, // equality
                fresh.checked_add(n)?,                      // active-pair set
                old_pairs.checked_add(m)?,                  // released preferences
                fresh.checked_mul(2)?.checked_add(n.checked_mul(4)?)?, // target collect/write
            ];
            let work = stages.into_iter().try_fold(0usize, usize::checked_add)?;

            // Cumulative allocation, not just final live extent. A filtered Vec
            // starts at <=4 elements and doubles: all its allocations sum to
            // <=8*k slots. Exact-size collects and stable-sort scratch use k.
            let ids = n.checked_mul(8 + 8 + 8 + 1 + 1 + 1)?;
            let pairs = m.checked_mul(2)?.checked_add(n.checked_mul(8 + 1)?)?;
            let mut bytes = ids
                .checked_mul(size_of::<UnitInstanceId>())?
                .checked_add(pairs.checked_mul(size_of::<(UnitInstanceId, UnitInstanceId)>())?)?;
            // Route-progress creates two exact-size center vectors per priority
            // comparison. Admit their cumulative allocations before any sort.
            bytes = bytes.checked_add(
                route_comparisons
                    .checked_mul(2)?
                    .checked_mul(route)?
                    .checked_mul(size_of::<WorldVec2>())?,
            )?;
            // Fresh insert-only tables: the history bound covers load-factor
            // rounding; geometric growth sums to <=2*final buckets. Every
            // allocation has one pinned hashbrown control group plus alignment
            // padding, and there are no more allocations than insertions.
            for (entry, alignment) in [
                (
                    size_of::<(UnitInstanceId, u32)>(),
                    align_of::<(UnitInstanceId, u32)>(),
                ),
                (
                    size_of::<(UnitInstanceId, Vec<UnitInstanceId>)>(),
                    align_of::<(UnitInstanceId, Vec<UnitInstanceId>)>(),
                ),
                (
                    size_of::<(UnitInstanceId, UnitInstanceId)>(),
                    align_of::<(UnitInstanceId, UnitInstanceId)>(),
                ),
                (
                    size_of::<(UnitInstanceId, UnitInstanceId)>(),
                    align_of::<(UnitInstanceId, UnitInstanceId)>(),
                ),
            ] {
                let table = fresh
                    .checked_mul(entry.checked_add(1)?)?
                    .checked_mul(2)?
                    .checked_add(
                        n.checked_mul(size_of::<[u8; 16]>().checked_add(alignment - 1)?)?,
                    )?;
                bytes = bytes.checked_add(table)?;
            }
            Some((u64::try_from(work).ok()?, bytes))
        };
        bound().ok_or_else(|| block_refresh_overflow("object_block_refresh_bounds"))
    }

    pub(in crate::game::battle::core) fn refresh_block_state(&mut self) {
        if !matches!(
            self.scenario.tactical_plan.player_plan,
            PlayerMovementPlan::FixedDefense
        ) {
            self.replace_block_state(BlockRuntimeState::default());
            return;
        }

        let mut blocker_ids = self
            .units
            .values()
            .filter(|unit| self.is_valid_blocker(unit.instance_id))
            .map(|unit| unit.instance_id)
            .collect::<Vec<_>>();
        blocker_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        if blocker_ids.is_empty() {
            self.replace_block_state(BlockRuntimeState::default());
            return;
        }

        let mut enemy_ids = self
            .units
            .values()
            .filter(|unit| self.is_valid_blockable_enemy(unit.instance_id))
            .map(|unit| unit.instance_id)
            .collect::<Vec<_>>();
        enemy_ids.sort_by(|a, b| self.compare_enemy_block_priority(*a, *b));
        if enemy_ids.is_empty() {
            self.replace_block_state(BlockRuntimeState::default());
            return;
        }

        let mut remaining_capacity = blocker_ids
            .iter()
            .filter_map(|blocker_id| {
                self.units.get(blocker_id).map(|unit| {
                    (
                        *blocker_id,
                        if unit.role == BattleUnitRole::IndependentWall {
                            u32::MAX
                        } else {
                            self.effective_block_capacity(unit.instance_id)
                        },
                    )
                })
            })
            .collect::<HashMap<_, _>>();
        let mut block_state = BlockRuntimeState::default();

        let mut existing_engagements = self
            .block_state
            .enemy_to_blocker
            .iter()
            .map(|(enemy_id, blocker_id)| (*enemy_id, *blocker_id))
            .collect::<Vec<_>>();
        existing_engagements.sort_by(|(left_enemy, left_blocker), (right_enemy, right_blocker)| {
            left_blocker
                .as_bytes()
                .cmp(right_blocker.as_bytes())
                .then_with(|| self.compare_enemy_block_priority(*left_enemy, *right_enemy))
        });

        for (enemy_id, blocker_id) in existing_engagements {
            if !self.is_valid_blocker(blocker_id)
                || !self.is_valid_blockable_enemy(enemy_id)
                || !self.is_enemy_inside_block_radius(enemy_id, blocker_id)
            {
                continue;
            }
            let Some(capacity) = remaining_capacity.get_mut(&blocker_id) else {
                continue;
            };
            let demand=self.enemy_block_demand(enemy_id);
            if *capacity < demand {
                continue;
            }
            *capacity = capacity.saturating_sub(self.enemy_block_demand(enemy_id));
            block_state
                .blocker_to_enemies
                .entry(blocker_id)
                .or_default()
                .push(enemy_id);
            block_state.enemy_to_blocker.insert(enemy_id, blocker_id);
        }

        for enemy_id in enemy_ids {
            if block_state.enemy_to_blocker.contains_key(&enemy_id) {
                continue;
            }
            let Some(blocker_id) =
                self.closest_available_blocker(enemy_id, &blocker_ids, &remaining_capacity)
            else {
                continue;
            };
            if let Some(capacity) = remaining_capacity.get_mut(&blocker_id) {
                *capacity = capacity.saturating_sub(self.enemy_block_demand(enemy_id));
            }
            block_state
                .blocker_to_enemies
                .entry(blocker_id)
                .or_default()
                .push(enemy_id);
            block_state.enemy_to_blocker.insert(enemy_id, blocker_id);
        }

        for enemy_ids in block_state.blocker_to_enemies.values_mut() {
            enemy_ids.sort_by(|a, b| self.compare_enemy_block_priority(*a, *b));
        }

        self.replace_block_state(block_state);
    }

    /// Constrain the actual backend segment, reserving capacity in route priority order.
    pub(super) fn constrain_block_contacts(&mut self, outputs: &mut [MovementOutput]) {
        if !matches!(
            self.scenario.tactical_plan.player_plan,
            PlayerMovementPlan::FixedDefense
        ) {
            return;
        }
        outputs.sort_unstable_by(|a, b| {
            self.compare_enemy_block_priority(output_unit(a), output_unit(b))
        });
        for output in outputs.iter_mut() {
            let MovementOutput::BodyMoved { unit_id, from, to } = output else {
                continue;
            };
            if !self.is_valid_blockable_enemy(*unit_id) || self.blocked_by(*unit_id).is_some() {
                continue;
            }
            let contact = self
                .units
                .values()
                .filter(|blocker| self.is_valid_blocker(blocker.instance_id))
                .filter(|blocker| self.approach_taunt(*unit_id).is_none_or(|(source, _)| source == blocker.instance_id))
                .filter(|blocker| {
                    self.units
                        .get(unit_id)
                        .is_some_and(|mover| mover.owner != blocker.owner)
                })
                .filter(|blocker| {
                    blocker.role == BattleUnitRole::IndependentWall
                        || self.units.get(unit_id).is_some_and(|mover| mover.blockable)
                })
                .filter(|blocker| {
                    blocker.role == BattleUnitRole::IndependentWall
                        || self.occupied_block_capacity(blocker.instance_id).saturating_add(self.enemy_block_demand(*unit_id))
                            <= self.effective_block_capacity(blocker.instance_id)
                })
                .filter_map(|blocker| {
                    let contact = if blocker.role == BattleUnitRole::IndependentWall {
                        segment_wall_contact(
                            *from,
                            *to,
                            blocker.body.position,
                            self.units.get(unit_id)?.body.radius,
                        )
                    } else {
                        segment_circle_contact(
                            *from,
                            *to,
                            blocker.body.position,
                            blocker.block_radius_units,
                        )
                    };
                    contact.map(|t| {
                        (
                            t,
                            from.distance_squared(blocker.body.position),
                            blocker.instance_id,
                        )
                    })
                })
                .min_by(|a, b| {
                    a.0.total_cmp(&b.0)
                        .then_with(|| a.1.total_cmp(&b.1))
                        .then_with(|| a.2.cmp(&b.2))
                });
            if let Some((t, _, blocker_id)) = contact {
                *to = *from + (*to - *from) * t;
                self.block_state
                    .enemy_to_blocker
                    .insert(*unit_id, blocker_id);
                self.block_state
                    .blocker_to_enemies
                    .entry(blocker_id)
                    .or_default()
                    .push(*unit_id);
            }
        }
        // Preserve the backend's canonical output/log order.
        outputs.sort_unstable_by_key(output_unit);
        self.apply_block_target_preferences();
    }

    fn replace_block_state(&mut self, next_state: BlockRuntimeState) {
        if self.block_state == next_state {
            return;
        }
        let previous_state = std::mem::replace(&mut self.block_state, next_state);
        self.clear_released_block_target_preferences(&previous_state);
        self.apply_block_target_preferences();
    }

    /// Read the unique current enemy-to-blocker authority; capacity demand is not a count.
    pub(in crate::game::battle::core) fn relation_count(
        &self, subject: UnitInstanceId, relation: crate::game::value_expr::RelationCountKind,
    ) -> Result<u32, crate::game::behavior::GameError> {
        use crate::game::value_expr::RelationCountKind;
        let Some(owner) = self.units.get(&subject).filter(|u| u.is_active() && u.is_combatant() && u.stats.current_health > 0) else { return Ok(0); };
        match relation { RelationCountKind::BlockingHostiles => {
            crate::authoring::control::charge(self.block_state.enemy_to_blocker.len() as u64, "blocking_hostiles_relation")?;
            let count = self.block_state.enemy_to_blocker.iter().filter(|(enemy, blocker)| **blocker == subject
                && self.units.get(*enemy).is_some_and(|u| u.is_active() && u.is_combatant() && u.stats.current_health > 0 && u.owner != owner.owner)).count();
            u32::try_from(count).map_err(|_| crate::game::battle::runtime_control::invariant("blocking_hostiles_count_overflow"))
        } }
    }

    pub(in crate::game::battle::core) fn blocked_by(
        &self,
        enemy_id: UnitInstanceId,
    ) -> Option<UnitInstanceId> {
        self.block_state.enemy_to_blocker.get(&enemy_id).copied()
    }

    pub(in crate::game::battle::core) fn first_blocked_enemy(
        &self,
        blocker_id: UnitInstanceId,
    ) -> Option<UnitInstanceId> {
        self.block_state
            .blocker_to_enemies
            .get(&blocker_id)
            .and_then(|enemy_ids| enemy_ids.first().copied())
    }

    pub(in crate::game::battle::core) fn fixed_defense_route_end_target_for_enemy(
        &self,
        enemy_id: UnitInstanceId,
    ) -> Option<UnitInstanceId> {
        if !self.is_fixed_defense_route_enemy(enemy_id) || self.blocked_by(enemy_id).is_some() {
            return None;
        }
        let endpoint = self.fixed_defense_route_end_point_for_enemy(enemy_id)?;
        let enemy = self.units.get(&enemy_id)?;
        if enemy.body.position.distance(endpoint) > 0.25 {
            return None;
        }
        self.protected_unit_id()
    }

    pub(in crate::game::battle::core) fn protected_unit_id(&self) -> Option<UnitInstanceId> {
        let unit_ref = match &self.scenario.tactical_plan.objective {
            BattleObjective::ProtectUnit { unit_ref } => Some(unit_ref),
            _ => None,
        };

        unit_ref
            .and_then(|unit_ref| {
                self.scenario_runtime
                    .spawned
                    .unit_refs
                    .get(unit_ref)
                    .copied()
            })
            .or_else(|| {
                self.units
                    .values()
                    .filter(|unit| unit.role == BattleUnitRole::DefenseObject && unit.is_active())
                    .map(|unit| unit.instance_id)
                    .min_by(|a, b| a.as_bytes().cmp(b.as_bytes()))
            })
    }

    pub(in crate::game::battle::core) fn is_fixed_defense_route_enemy(
        &self,
        unit_id: UnitInstanceId,
    ) -> bool {
        matches!(
            self.scenario.tactical_plan.player_plan,
            PlayerMovementPlan::FixedDefense
        ) && self
            .units
            .get(&unit_id)
            .is_some_and(|unit| unit.owner == Side::Opponent)
    }

    fn fixed_defense_route_end_point_for_enemy(
        &self,
        enemy_id: UnitInstanceId,
    ) -> Option<super::types::WorldVec2> {
        let plan = self
            .units
            .get(&enemy_id)
            .and_then(|unit| unit.enemy_movement_plan.as_ref())
            .unwrap_or(&self.scenario.tactical_plan.enemy_plan);
        match plan {
            EnemyMovementPlan::PathAlongCells { cells } => cells
                .last()
                .copied()
                .map(super::types::WorldVec2::from_tile_center),
        }
    }

    fn closest_available_blocker(
        &self,
        enemy_id: UnitInstanceId,
        blocker_ids: &[UnitInstanceId],
        remaining_capacity: &HashMap<UnitInstanceId, u32>,
    ) -> Option<UnitInstanceId> {
        blocker_ids
            .iter()
            .copied()
            .filter(|id| {
                self.units
                    .get(id)
                    .is_some_and(|unit| unit.role != BattleUnitRole::IndependentWall)
            })
            .filter(|blocker_id| remaining_capacity.get(blocker_id).copied().unwrap_or(0) >= self.enemy_block_demand(enemy_id))
            .filter(|blocker_id| self.is_enemy_inside_block_radius(enemy_id, *blocker_id))
            .min_by(|a, b| {
                let distance_order = self
                    .block_distance_sq(*a, enemy_id)
                    .unwrap_or(f32::MAX)
                    .total_cmp(&self.block_distance_sq(*b, enemy_id).unwrap_or(f32::MAX));
                distance_order.then_with(|| a.as_bytes().cmp(b.as_bytes()))
            })
    }

    fn is_valid_blocker(&self, blocker_id: UnitInstanceId) -> bool {
        let Some(unit) = self.units.get(&blocker_id) else {
            return false;
        };
        if unit.role == BattleUnitRole::IndependentWall {
            return unit.is_active();
        }
        unit.owner == Side::Player
            && unit.is_combatant()
            && unit.is_active()
            && self.effective_block_capacity(blocker_id) > 0
            && unit.block_radius_units > 0.0
            && !self.current_control_state(blocker_id).hard_cc_active()
    }

    fn is_valid_blockable_enemy(&self, enemy_id: UnitInstanceId) -> bool {
        let Some(unit) = self.units.get(&enemy_id) else {
            return false;
        };
        (unit.owner == Side::Opponent
            || matches!(
                unit.source_identity,
                crate::game::battle::types::BattleUnitSourceIdentity::IndependentObject { .. }
            ))
            && unit.is_combatant()
            && unit.is_active()
            && !unit.is_airborne()
            && !self.current_control_state(enemy_id).hard_cc_active()
    }

    fn compare_enemy_block_priority(
        &self,
        left: UnitInstanceId,
        right: UnitInstanceId,
    ) -> std::cmp::Ordering {
        let progress_order = self
            .enemy_route_progress(right)
            .total_cmp(&self.enemy_route_progress(left));
        progress_order
            .then_with(|| {
                let left_spawn_order = self
                    .units
                    .get(&left)
                    .map(|unit| unit.spawn_order)
                    .unwrap_or(u64::MAX);
                let right_spawn_order = self
                    .units
                    .get(&right)
                    .map(|unit| unit.spawn_order)
                    .unwrap_or(u64::MAX);
                left_spawn_order.cmp(&right_spawn_order)
            })
            .then_with(|| left.as_bytes().cmp(right.as_bytes()))
    }

    pub(in crate::game::battle::core) fn enemy_route_progress(
        &self,
        enemy_id: UnitInstanceId,
    ) -> f32 {
        let Some(enemy) = self.units.get(&enemy_id) else {
            return 0.0;
        };
        self.enemy_route_progress_from(enemy_id, enemy.body.position)
    }

    pub(in crate::game::battle::core) fn enemy_route_progress_at(
        &self,
        enemy_id: UnitInstanceId,
        time_ms: u64,
    ) -> f32 {
        self.sample_unit_world_position_at(enemy_id, time_ms)
            .map_or(0.0, |position| self.enemy_route_progress_from(enemy_id, position))
    }

    fn enemy_route_progress_from(&self, enemy_id: UnitInstanceId, position: WorldVec2) -> f32 {
        let Some(enemy) = self.units.get(&enemy_id) else {
            return 0.0;
        };
        let plan = enemy
            .enemy_movement_plan
            .as_ref()
            .unwrap_or(&self.scenario.tactical_plan.enemy_plan);
        match plan {
            EnemyMovementPlan::PathAlongCells { cells } => {
                route_progress_along_cells(cells, position).unwrap_or(0.0)
            }
        }
    }

    pub(in crate::game::battle::core) fn enemy_route_remaining(
        &self,
        enemy_id: UnitInstanceId,
    ) -> Option<f32> {
        let enemy = self.units.get(&enemy_id)?;
        let plan = enemy
            .enemy_movement_plan
            .as_ref()
            .unwrap_or(&self.scenario.tactical_plan.enemy_plan);
        match plan {
            EnemyMovementPlan::PathAlongCells { cells } => {
                let progress = route_progress_along_cells(cells, enemy.body.position)?;
                let total = route_total_length(cells)?;
                Some((total - progress).max(0.0))
            }
        }
    }

    fn is_enemy_inside_block_radius(
        &self,
        enemy_id: UnitInstanceId,
        blocker_id: UnitInstanceId,
    ) -> bool {
        if self.approach_taunt(enemy_id).is_some_and(|(source, _)| source != blocker_id) { return false; }
        let Some(distance_sq) = self.block_distance_sq(blocker_id, enemy_id) else {
            return false;
        };
        let Some(blocker) = self.units.get(&blocker_id) else {
            return false;
        };
        let Some(enemy) = self
            .units
            .get(&enemy_id)
            .filter(|unit| unit.owner != blocker.owner)
        else {
            return false;
        };
        if blocker.role == BattleUnitRole::IndependentWall {
            let half = 0.5 + enemy.body.radius.max(0.0) + CONTACT_DISTANCE_SQ_EPSILON;
            return (blocker.body.position.x - enemy.body.position.x).abs() <= half
                && (blocker.body.position.y - enemy.body.position.y).abs() <= half;
        }
        if !enemy.blockable {
            return false;
        }
        let radius = blocker.block_radius_units.max(0.0);
        distance_sq <= radius * radius + CONTACT_DISTANCE_SQ_EPSILON
    }

    fn block_distance_sq(
        &self,
        blocker_id: UnitInstanceId,
        enemy_id: UnitInstanceId,
    ) -> Option<f32> {
        let blocker = self.units.get(&blocker_id)?;
        let enemy = self.units.get(&enemy_id)?;
        Some(blocker.body.position.distance_squared(enemy.body.position))
    }

    fn clear_released_block_target_preferences(&mut self, previous_state: &BlockRuntimeState) {
        let active_pairs = self
            .block_state
            .enemy_to_blocker
            .iter()
            .map(|(enemy_id, blocker_id)| (*enemy_id, *blocker_id))
            .collect::<HashSet<_>>();

        for (enemy_id, blocker_id) in previous_state
            .enemy_to_blocker
            .iter()
            .map(|(enemy_id, blocker_id)| (*enemy_id, *blocker_id))
        {
            if active_pairs.contains(&(enemy_id, blocker_id)) {
                continue;
            }
            if let Some(enemy) = self.units.get_mut(&enemy_id) {
                if enemy.current_target == Some(blocker_id) {
                    enemy.current_target = None;
                }
            }
            if let Some(blocker) = self.units.get_mut(&blocker_id) {
                if blocker.current_target == Some(enemy_id) {
                    blocker.current_target = None;
                }
            }
        }
    }

    fn apply_block_target_preferences(&mut self) {
        let blocker_targets = self
            .block_state
            .blocker_to_enemies
            .iter()
            .filter_map(|(blocker_id, enemy_ids)| {
                enemy_ids.first().map(|enemy_id| (*blocker_id, *enemy_id))
            })
            .collect::<Vec<_>>();
        let enemy_targets = self
            .block_state
            .enemy_to_blocker
            .iter()
            .map(|(enemy_id, blocker_id)| (*enemy_id, *blocker_id))
            .collect::<Vec<_>>();

        for (unit_id, target_id) in blocker_targets.into_iter().chain(enemy_targets) {
            if let Some(unit) = self.units.get_mut(&unit_id) {
                unit.current_target = Some(target_id);
            }
        }
    }
}
