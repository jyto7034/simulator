use std::collections::{HashMap, HashSet};

use crate::game::{
    battle::{
        core::{BattleCore, MeleeSlotReservation},
        ids::UnitInstanceId,
        scenario::{EnemyMovementPlan, PlayerMovementPlan},
        scenario::{FormationKind, GroupObjective, TacticalGroupPlan, TacticalGroupPlanId},
    },
    enums::Side,
};

use super::types::{MovementGoal, WorldVec2};

const MELEE_ENGAGEMENT_RANGE_THRESHOLD: f32 = 1.0;
const MELEE_ENGAGEMENT_SPACING_MULTIPLIER: f32 = 1.15;
const MELEE_ENGAGEMENT_TARGET_DRIFT_TOLERANCE: f32 = 0.35;

enum TacticalGoalDecision {
    Handled(Option<MovementGoal>),
    NotApplicable,
}

impl BattleCore {
    pub fn build_continuous_attack_goals(&mut self) -> HashMap<UnitInstanceId, MovementGoal> {
        let mut unit_ids: Vec<UnitInstanceId> = self.units.keys().copied().collect();
        unit_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        let mut goals = HashMap::new();
        let previous_reservations = self.melee_slot_reservations.clone();
        self.melee_slot_reservations.clear();
        let mut occupied_slots: HashSet<(UnitInstanceId, usize)> = HashSet::new();
        let mut reserved_positions: Vec<MeleeSlotReservation> = Vec::new();

        for unit_id in unit_ids {
            let Some(unit) = self.units.get(&unit_id) else {
                continue;
            };
            if unit.is_dead() || !unit.can_move() {
                continue;
            }

            let goal = match self.group_objective_goal(unit_id) {
                TacticalGoalDecision::Handled(goal) => goal,
                TacticalGoalDecision::NotApplicable => match unit.owner {
                    Side::Player => match self.scenario.tactical_plan.player_plan.clone() {
                        PlayerMovementPlan::FreeEngage => self.free_engage_goal(
                            unit_id,
                            &previous_reservations,
                            &mut occupied_slots,
                            &mut reserved_positions,
                        ),
                        PlayerMovementPlan::HoldDeployment {
                            guard_radius,
                            leash_radius,
                            chase_radius,
                            return_to_anchor,
                        } => self.hold_deployment_goal(
                            unit_id,
                            guard_radius,
                            leash_radius,
                            chase_radius,
                            return_to_anchor,
                        ),
                        PlayerMovementPlan::CautiousEngage {
                            leash_radius,
                            chase_radius,
                        } => self.cautious_engage_goal(unit_id, leash_radius, chase_radius),
                    },
                    Side::Opponent => match self.scenario.tactical_plan.enemy_plan.clone() {
                        EnemyMovementPlan::AssaultPlayer => self.free_engage_goal(
                            unit_id,
                            &previous_reservations,
                            &mut occupied_slots,
                            &mut reserved_positions,
                        ),
                        EnemyMovementPlan::PathToPoint { point_id } => {
                            self.path_to_point_goal(unit_id, &point_id)
                        }
                        EnemyMovementPlan::PathAlongPath { point_ids } => {
                            self.path_along_path_goal(unit_id, &point_ids)
                        }
                    },
                },
            };

            if let Some(goal) = goal {
                goals.insert(unit_id, goal);
            }
        }

        goals
    }

    fn free_engage_goal(
        &mut self,
        unit_id: UnitInstanceId,
        previous_reservations: &HashMap<UnitInstanceId, MeleeSlotReservation>,
        occupied_slots: &mut HashSet<(UnitInstanceId, usize)>,
        reserved_positions: &mut Vec<MeleeSlotReservation>,
    ) -> Option<MovementGoal> {
        let unit = self.units.get(&unit_id)?;
        let target_id = self
            .continuous_sticky_target(unit_id)
            .or_else(|| self.nearest_continuous_enemy(unit_id))?;

        let desired_range = unit.basic_attack.range_units.max(0.0);
        let already_in_range = self
            .units
            .get(&target_id)
            .is_some_and(|target| unit.body.can_reach(&target.body, desired_range));
        let engagement_point =
            if !already_in_range && desired_range <= MELEE_ENGAGEMENT_RANGE_THRESHOLD {
                self.reserve_melee_engagement_point(
                    unit_id,
                    target_id,
                    desired_range,
                    previous_reservations.get(&unit_id).copied(),
                    occupied_slots,
                    reserved_positions,
                )
            } else {
                None
            };

        Some(MovementGoal::AttackUnit {
            target_id,
            desired_range,
            approach_point: engagement_point,
        })
    }

    fn hold_deployment_goal(
        &self,
        unit_id: UnitInstanceId,
        guard_radius: f32,
        leash_radius: f32,
        chase_radius: f32,
        return_to_anchor: bool,
    ) -> Option<MovementGoal> {
        let unit = self.units.get(&unit_id)?;
        let anchor = unit.tactical_anchor.unwrap_or(unit.body.position);
        let distance_from_anchor = unit.body.position.distance(anchor);
        let desired_range = unit.basic_attack.range_units.max(0.0);
        let leash_radius = leash_radius.max(0.0);

        if distance_from_anchor > leash_radius {
            return return_to_anchor.then_some(MovementGoal::MoveToPoint {
                point: anchor,
                stop_radius: 0.1,
            });
        }

        if let Some(target_id) = self.closest_enemy_in_attack_range(unit_id) {
            return Some(MovementGoal::AttackUnit {
                target_id,
                desired_range,
                approach_point: None,
            });
        }

        if let Some((target_id, approach_point)) =
            self.closest_enemy_reachable_from_anchor(unit_id, anchor, guard_radius, chase_radius)
        {
            return Some(MovementGoal::AttackUnit {
                target_id,
                desired_range,
                approach_point: Some(approach_point),
            });
        }

        if return_to_anchor && distance_from_anchor > 0.1 {
            return Some(MovementGoal::MoveToPoint {
                point: anchor,
                stop_radius: 0.1,
            });
        }

        None
    }

    fn cautious_engage_goal(
        &self,
        unit_id: UnitInstanceId,
        leash_radius: f32,
        chase_radius: f32,
    ) -> Option<MovementGoal> {
        let unit = self.units.get(&unit_id)?;
        let anchor = unit.tactical_anchor.unwrap_or(unit.body.position);
        let distance_from_anchor = unit.body.position.distance(anchor);
        let leash_radius = leash_radius.max(0.0);
        let desired_range = unit.basic_attack.range_units.max(0.0);

        if distance_from_anchor > leash_radius {
            return Some(MovementGoal::MoveToPoint {
                point: anchor,
                stop_radius: 0.1,
            });
        }

        if let Some(target_id) = self.closest_enemy_in_attack_range(unit_id) {
            return Some(MovementGoal::AttackUnit {
                target_id,
                desired_range,
                approach_point: None,
            });
        }

        let (target_id, approach_point) =
            self.closest_enemy_with_limited_chase(unit_id, anchor, leash_radius, chase_radius)?;
        Some(MovementGoal::AttackUnit {
            target_id,
            desired_range,
            approach_point: Some(approach_point),
        })
    }

    fn path_to_point_goal(
        &self,
        unit_id: UnitInstanceId,
        point_id: &crate::game::battle::scenario::TacticalPointId,
    ) -> Option<MovementGoal> {
        let unit = self.units.get(&unit_id)?;
        if let Some(target_id) = self.closest_enemy_in_attack_range(unit_id) {
            return Some(MovementGoal::AttackUnit {
                target_id,
                desired_range: unit.basic_attack.range_units.max(0.0),
                approach_point: None,
            });
        }

        let point = self.scenario.tactical_plan.point(point_id)?;
        Some(MovementGoal::MoveToPoint {
            point: WorldVec2::from_tile_center(point.position),
            stop_radius: 0.1,
        })
    }

    fn path_along_path_goal(
        &self,
        unit_id: UnitInstanceId,
        point_ids: &[crate::game::battle::scenario::TacticalPointId],
    ) -> Option<MovementGoal> {
        let unit = self.units.get(&unit_id)?;
        if let Some(target_id) = self.closest_enemy_in_attack_range(unit_id) {
            return Some(MovementGoal::AttackUnit {
                target_id,
                desired_range: unit.basic_attack.range_units.max(0.0),
                approach_point: None,
            });
        }

        let target = self.active_path_target(unit.body.position, point_ids, 0.25)?;
        Some(MovementGoal::MoveToPoint {
            point: target,
            stop_radius: 0.1,
        })
    }

    fn group_objective_goal(&self, unit_id: UnitInstanceId) -> TacticalGoalDecision {
        let Some(unit) = self.units.get(&unit_id) else {
            return TacticalGoalDecision::NotApplicable;
        };
        let Some(group_id) = unit.tactical_group_id.as_ref() else {
            return TacticalGoalDecision::NotApplicable;
        };
        let Some(group_plan) = self.scenario.tactical_plan.group_plan(group_id) else {
            return TacticalGoalDecision::NotApplicable;
        };

        let objective_center = match &group_plan.objective {
            GroupObjective::FollowBattleObjective => return TacticalGoalDecision::NotApplicable,
            GroupObjective::HoldArea { point_id } => {
                let Some(point) = self.scenario.tactical_plan.point(point_id) else {
                    return TacticalGoalDecision::Handled(None);
                };
                WorldVec2::from_tile_center(point.position)
            }
            GroupObjective::AdvanceToPoint { point_id } => {
                let Some(point) = self.scenario.tactical_plan.point(point_id) else {
                    return TacticalGoalDecision::Handled(None);
                };
                let target = WorldVec2::from_tile_center(point.position);
                self.group_live_center(&group_plan.id)
                    .map(|center| {
                        advance_center_toward(center, target, group_plan.cohesion_radius.max(0.5))
                    })
                    .unwrap_or(target)
            }
            GroupObjective::AdvanceAlongPath { point_ids } => {
                let Some(target) = self.active_group_path_target(
                    &group_plan.id,
                    point_ids,
                    group_plan.cohesion_radius.max(0.5),
                ) else {
                    return TacticalGoalDecision::Handled(None);
                };
                self.group_live_center(&group_plan.id)
                    .map(|center| {
                        advance_center_toward(center, target, group_plan.cohesion_radius.max(0.5))
                    })
                    .unwrap_or(target)
            }
            GroupObjective::ReconnectToGroup { group_id } => {
                let Some(center) = self.group_live_center(group_id) else {
                    return TacticalGoalDecision::Handled(None);
                };
                center
            }
        };

        if let Some(target_id) = self.closest_enemy_in_attack_range(unit_id) {
            return TacticalGoalDecision::Handled(Some(MovementGoal::AttackUnit {
                target_id,
                desired_range: unit.basic_attack.range_units.max(0.0),
                approach_point: None,
            }));
        }

        let Some(formation_point) =
            self.group_formation_point(group_plan, unit_id, objective_center)
        else {
            return TacticalGoalDecision::Handled(None);
        };
        let cohesion_radius = group_plan.cohesion_radius.max(0.0);
        let distance_from_slot = unit.body.position.distance(formation_point);

        if distance_from_slot > cohesion_radius.max(0.2) {
            return TacticalGoalDecision::Handled(Some(MovementGoal::MoveToPoint {
                point: formation_point,
                stop_radius: 0.2,
            }));
        }

        if let Some((target_id, approach_point)) =
            self.closest_enemy_reachable_from_group_slot(unit_id, formation_point, group_plan)
        {
            return TacticalGoalDecision::Handled(Some(MovementGoal::AttackUnit {
                target_id,
                desired_range: unit.basic_attack.range_units.max(0.0),
                approach_point: Some(approach_point),
            }));
        }

        if distance_from_slot <= 0.2 {
            return TacticalGoalDecision::Handled(None);
        }

        TacticalGoalDecision::Handled(Some(MovementGoal::MoveToPoint {
            point: formation_point,
            stop_radius: 0.2,
        }))
    }

    fn group_formation_point(
        &self,
        group_plan: &TacticalGroupPlan,
        unit_id: UnitInstanceId,
        center: WorldVec2,
    ) -> Option<WorldVec2> {
        let mut members = self
            .scenario_runtime
            .tactical_groups
            .get(&group_plan.id)?
            .iter()
            .copied()
            .filter(|member_id| {
                self.units
                    .get(member_id)
                    .is_some_and(|unit| !unit.is_dead())
            })
            .collect::<Vec<_>>();
        members.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        let index = members.iter().position(|member_id| *member_id == unit_id)?;
        let offset = formation_offset(&group_plan.formation, index, members.len(), 0.9);
        let unit_radius = self.units.get(&unit_id)?.body.radius;
        Some(project_goal_point_to_battlefield(
            center + offset,
            unit_radius,
            self,
        ))
    }

    fn active_group_path_target(
        &self,
        group_id: &TacticalGroupPlanId,
        point_ids: &[crate::game::battle::scenario::TacticalPointId],
        reach_radius: f32,
    ) -> Option<WorldVec2> {
        let center = self.group_live_center(group_id);
        let mut last_valid = None;

        for point_id in point_ids {
            let point = self.scenario.tactical_plan.point(point_id)?;
            let target = WorldVec2::from_tile_center(point.position);
            last_valid = Some(target);
            if center.is_none_or(|center| center.distance(target) > reach_radius) {
                return Some(target);
            }
        }

        last_valid
    }

    fn active_path_target(
        &self,
        current: WorldVec2,
        point_ids: &[crate::game::battle::scenario::TacticalPointId],
        reach_radius: f32,
    ) -> Option<WorldVec2> {
        let mut last_valid = None;

        for point_id in point_ids {
            let point = self.scenario.tactical_plan.point(point_id)?;
            let target = WorldVec2::from_tile_center(point.position);
            last_valid = Some(target);
            if current.distance(target) > reach_radius {
                return Some(target);
            }
        }

        last_valid
    }

    fn group_live_center(&self, group_id: &TacticalGroupPlanId) -> Option<WorldVec2> {
        let members = self.scenario_runtime.tactical_groups.get(group_id)?;
        let mut center = WorldVec2::ZERO;
        let mut count = 0.0;

        for member_id in members {
            let Some(unit) = self.units.get(member_id) else {
                continue;
            };
            if unit.is_dead() {
                continue;
            }
            center += unit.body.position;
            count += 1.0;
        }

        if count <= 0.0 {
            None
        } else {
            Some(center * (1.0 / count))
        }
    }

    fn closest_enemy_reachable_from_group_slot(
        &self,
        unit_id: UnitInstanceId,
        formation_point: WorldVec2,
        group_plan: &TacticalGroupPlan,
    ) -> Option<(UnitInstanceId, WorldVec2)> {
        let unit = self.units.get(&unit_id)?;
        let desired_range = unit.basic_attack.range_units.max(0.0);
        let engage_radius = group_plan.engage_radius.max(0.0);
        let cohesion_radius = group_plan.cohesion_radius.max(0.0);

        self.units
            .values()
            .filter(|candidate| !candidate.is_dead() && candidate.owner != unit.owner)
            .filter_map(|candidate| {
                let desired_center_distance =
                    desired_range + unit.body.radius + candidate.body.radius;
                if formation_point.distance(candidate.body.position)
                    > engage_radius + desired_center_distance
                {
                    return None;
                }

                let approach_point = project_goal_point_to_battlefield(
                    bounded_approach_point(
                        unit.body.position,
                        candidate.body.position,
                        formation_point,
                        cohesion_radius,
                        desired_center_distance,
                    ),
                    unit.body.radius,
                    self,
                );
                let score = unit.body.position.distance_squared(approach_point);
                Some((candidate.instance_id, approach_point, score))
            })
            .min_by(|a, b| {
                a.2.total_cmp(&b.2)
                    .then_with(|| a.0.as_bytes().cmp(b.0.as_bytes()))
            })
            .map(|(target_id, approach_point, _)| (target_id, approach_point))
    }

    fn reserve_melee_engagement_point(
        &mut self,
        unit_id: UnitInstanceId,
        target_id: UnitInstanceId,
        desired_range: f32,
        previous: Option<MeleeSlotReservation>,
        occupied_slots: &mut HashSet<(UnitInstanceId, usize)>,
        reserved_positions: &mut Vec<MeleeSlotReservation>,
    ) -> Option<WorldVec2> {
        let unit = self.units.get(&unit_id)?;
        let target = self.units.get(&target_id)?;
        if unit.is_dead() || target.is_dead() || unit.owner == target.owner {
            return None;
        }

        let orbit_radius = unit.body.radius + target.body.radius + desired_range;
        let slot_count =
            melee_engagement_point_count(unit.body.radius, target.body.radius, desired_range);

        if let Some(previous) = previous {
            if previous.target_id == target_id
                && previous.slot_index < slot_count
                && !occupied_slots.contains(&(target_id, previous.slot_index))
                && !melee_engagement_point_conflicts(
                    previous.position,
                    unit.body.radius,
                    reserved_positions,
                )
                && melee_engagement_point_tracks_target(
                    previous.position,
                    target.body.position,
                    orbit_radius,
                )
                && melee_engagement_point_is_valid(previous.position, unit.body.radius, self)
            {
                occupied_slots.insert((target_id, previous.slot_index));
                self.melee_slot_reservations.insert(unit_id, previous);
                reserved_positions.push(previous);
                return Some(previous.position);
            }
        }

        let mut best: Option<(usize, WorldVec2, f32)> = None;
        for slot_index in 0..slot_count {
            if occupied_slots.contains(&(target_id, slot_index)) {
                continue;
            }

            let position = melee_engagement_point(
                unit.body.position,
                target.body.position,
                slot_index,
                slot_count,
                orbit_radius,
            );
            if !melee_engagement_point_is_valid(position, unit.body.radius, self) {
                continue;
            }
            if melee_engagement_point_conflicts(position, unit.body.radius, reserved_positions) {
                continue;
            }

            let score = self.melee_engagement_point_score(unit_id, target_id, position);
            if best
                .as_ref()
                .is_none_or(|(_, _, best_score)| score < *best_score)
            {
                best = Some((slot_index, position, score));
            }
        }

        let (slot_index, position, _) = best?;
        occupied_slots.insert((target_id, slot_index));
        let reservation = MeleeSlotReservation {
            target_id,
            slot_index,
            position,
            unit_radius: unit.body.radius,
        };
        self.melee_slot_reservations.insert(unit_id, reservation);
        reserved_positions.push(reservation);
        Some(position)
    }

    fn melee_engagement_point_score(
        &self,
        unit_id: UnitInstanceId,
        target_id: UnitInstanceId,
        position: WorldVec2,
    ) -> f32 {
        let Some(unit) = self.units.get(&unit_id) else {
            return f32::MAX;
        };

        let mut score = unit.body.position.distance(position);
        for other in self.units.values() {
            if other.instance_id == unit_id || other.instance_id == target_id || other.is_dead() {
                continue;
            }

            let min_distance =
                (unit.body.radius + other.body.radius) * MELEE_ENGAGEMENT_SPACING_MULTIPLIER;
            let distance = position.distance(other.body.position);
            if distance < min_distance {
                score += (min_distance - distance) * 4.0;
            }
        }

        score
    }

    fn continuous_sticky_target(&self, unit_id: UnitInstanceId) -> Option<UnitInstanceId> {
        let unit = self.units.get(&unit_id)?;
        let target_id = unit.current_target?;
        let target = self.units.get(&target_id)?;
        if target.is_dead() || target.owner == unit.owner {
            None
        } else {
            Some(target_id)
        }
    }

    fn nearest_continuous_enemy(&self, unit_id: UnitInstanceId) -> Option<UnitInstanceId> {
        let unit = self.units.get(&unit_id)?;
        if unit.is_dead() {
            return None;
        }
        let body = unit.movement_body_view();

        self.units
            .values()
            .filter(|candidate| !candidate.is_dead() && candidate.owner != unit.owner)
            .min_by(|a, b| {
                let a_body = a.movement_body_view();
                let b_body = b.movement_body_view();
                body.position
                    .distance_squared(a_body.position)
                    .total_cmp(&body.position.distance_squared(b_body.position))
                    .then_with(|| a.instance_id.as_bytes().cmp(b.instance_id.as_bytes()))
            })
            .map(|target| target.instance_id)
    }

    fn closest_enemy_in_attack_range(&self, unit_id: UnitInstanceId) -> Option<UnitInstanceId> {
        let unit = self.units.get(&unit_id)?;
        let desired_range = unit.basic_attack.range_units.max(0.0);
        self.units
            .values()
            .filter(|candidate| {
                !candidate.is_dead()
                    && candidate.owner != unit.owner
                    && unit.body.can_reach(&candidate.body, desired_range)
            })
            .min_by(|a, b| {
                unit.body
                    .position
                    .distance_squared(a.body.position)
                    .total_cmp(&unit.body.position.distance_squared(b.body.position))
                    .then_with(|| a.instance_id.as_bytes().cmp(b.instance_id.as_bytes()))
            })
            .map(|target| target.instance_id)
    }

    fn closest_enemy_reachable_from_anchor(
        &self,
        unit_id: UnitInstanceId,
        anchor: WorldVec2,
        guard_radius: f32,
        chase_radius: f32,
    ) -> Option<(UnitInstanceId, WorldVec2)> {
        let unit = self.units.get(&unit_id)?;
        let desired_range = unit.basic_attack.range_units.max(0.0);
        let max_reaction_radius = guard_radius.max(0.0).max(chase_radius.max(0.0));

        self.units
            .values()
            .filter(|candidate| !candidate.is_dead() && candidate.owner != unit.owner)
            .filter_map(|candidate| {
                let reach_from_anchor =
                    max_reaction_radius + desired_range + unit.body.radius + candidate.body.radius;
                if anchor.distance(candidate.body.position) > reach_from_anchor {
                    return None;
                }

                let approach_point = project_goal_point_to_battlefield(
                    bounded_approach_point(
                        unit.body.position,
                        candidate.body.position,
                        anchor,
                        max_reaction_radius,
                        desired_range + unit.body.radius + candidate.body.radius,
                    ),
                    unit.body.radius,
                    self,
                );
                let score = unit.body.position.distance_squared(approach_point);
                Some((candidate.instance_id, approach_point, score))
            })
            .min_by(|a, b| {
                a.2.total_cmp(&b.2)
                    .then_with(|| a.0.as_bytes().cmp(b.0.as_bytes()))
            })
            .map(|(target_id, approach_point, _)| (target_id, approach_point))
    }

    fn closest_enemy_with_limited_chase(
        &self,
        unit_id: UnitInstanceId,
        anchor: WorldVec2,
        leash_radius: f32,
        chase_radius: f32,
    ) -> Option<(UnitInstanceId, WorldVec2)> {
        let unit = self.units.get(&unit_id)?;
        let desired_range = unit.basic_attack.range_units.max(0.0);
        let chase_radius = chase_radius.max(0.0);

        self.units
            .values()
            .filter(|candidate| !candidate.is_dead() && candidate.owner != unit.owner)
            .filter_map(|candidate| {
                let desired_center_distance =
                    desired_range + unit.body.radius + candidate.body.radius;
                let current_reaction_distance = desired_center_distance + chase_radius;
                if unit.body.position.distance(candidate.body.position) > current_reaction_distance
                {
                    return None;
                }

                if anchor.distance(candidate.body.position) > leash_radius + desired_center_distance
                {
                    return None;
                }

                let approach_point = project_goal_point_to_battlefield(
                    bounded_approach_point(
                        unit.body.position,
                        candidate.body.position,
                        anchor,
                        leash_radius,
                        desired_center_distance,
                    ),
                    unit.body.radius,
                    self,
                );
                let score = unit.body.position.distance_squared(approach_point);
                Some((candidate.instance_id, approach_point, score))
            })
            .min_by(|a, b| {
                a.2.total_cmp(&b.2)
                    .then_with(|| a.0.as_bytes().cmp(b.0.as_bytes()))
            })
            .map(|(target_id, approach_point, _)| (target_id, approach_point))
    }
}

fn bounded_approach_point(
    unit_position: WorldVec2,
    target_position: WorldVec2,
    anchor: WorldVec2,
    max_anchor_distance: f32,
    desired_center_distance: f32,
) -> WorldVec2 {
    let from_target = (unit_position - target_position).normalized_or_zero();
    let raw_approach = if from_target.length_squared() <= f32::EPSILON {
        anchor
    } else {
        target_position + from_target * desired_center_distance.max(0.0)
    };
    let from_anchor = raw_approach - anchor;
    let distance = from_anchor.length();
    if distance <= max_anchor_distance || distance <= f32::EPSILON {
        raw_approach
    } else {
        anchor + from_anchor.normalized_or_zero() * max_anchor_distance
    }
}

fn formation_offset(
    formation: &FormationKind,
    index: usize,
    member_count: usize,
    spacing: f32,
) -> WorldVec2 {
    if member_count <= 1 {
        return WorldVec2::ZERO;
    }

    let centered_index = index as f32 - (member_count.saturating_sub(1) as f32 / 2.0);
    match formation {
        FormationKind::Line => WorldVec2::new(centered_index * spacing, 0.0),
        FormationKind::Column => WorldVec2::new(0.0, centered_index * spacing),
        FormationKind::Wedge => {
            if index == 0 {
                return WorldVec2::ZERO;
            }
            let rank = ((index + 1) / 2) as f32;
            let side = if index % 2 == 0 { 1.0 } else { -1.0 };
            WorldVec2::new(side * rank * spacing, rank * spacing)
        }
        FormationKind::Loose => {
            let columns = (member_count as f32).sqrt().ceil() as usize;
            let row = index / columns;
            let column = index % columns;
            let rows = member_count.div_ceil(columns);
            let x = column as f32 - (columns.saturating_sub(1) as f32 / 2.0);
            let y = row as f32 - (rows.saturating_sub(1) as f32 / 2.0);
            WorldVec2::new(x * spacing, y * spacing)
        }
    }
}

fn clamp_point_to_board(point: WorldVec2, unit_radius: f32, core: &BattleCore) -> WorldVec2 {
    let min = unit_radius.max(0.0);
    let max_x = (f32::from(core.battlefield.width()) - min).max(min);
    let max_y = (f32::from(core.battlefield.height()) - min).max(min);
    WorldVec2::new(point.x.clamp(min, max_x), point.y.clamp(min, max_y))
}

fn project_goal_point_to_battlefield(
    point: WorldVec2,
    unit_radius: f32,
    core: &BattleCore,
) -> WorldVec2 {
    let clamped = clamp_point_to_board(point, unit_radius, core);
    let projected_tile = clamped.project_to_tile();
    if core.battlefield.is_walkable_tile(projected_tile) {
        return clamped;
    }

    nearest_walkable_tile_center(clamped, core).unwrap_or(clamped)
}

fn nearest_walkable_tile_center(point: WorldVec2, core: &BattleCore) -> Option<WorldVec2> {
    let mut best: Option<(f32, WorldVec2)> = None;

    for y in 0..core.battlefield.height() as i32 {
        for x in 0..core.battlefield.width() as i32 {
            let tile = crate::game::resources::Position::new(x, y);
            if !core.battlefield.is_walkable_tile(tile) {
                continue;
            }
            let center = WorldVec2::from_tile_center(tile);
            let score = point.distance_squared(center);
            if best
                .as_ref()
                .is_none_or(|(best_score, _)| score < *best_score)
            {
                best = Some((score, center));
            }
        }
    }

    best.map(|(_, center)| center)
}

fn advance_center_toward(current: WorldVec2, target: WorldVec2, max_step: f32) -> WorldVec2 {
    let offset = target - current;
    let distance = offset.length();
    if distance <= max_step || distance <= f32::EPSILON {
        target
    } else {
        current + offset.normalized_or_zero() * max_step
    }
}

fn melee_engagement_point_count(
    attacker_radius: f32,
    target_radius: f32,
    desired_range: f32,
) -> usize {
    let orbit_radius = attacker_radius + target_radius + desired_range;
    let min_spacing = (attacker_radius * 2.0 * MELEE_ENGAGEMENT_SPACING_MULTIPLIER).max(0.1);
    let circumference = std::f32::consts::TAU * orbit_radius.max(min_spacing);
    ((circumference / min_spacing).floor() as usize).clamp(6, 12)
}

fn melee_engagement_point(
    attacker_position: WorldVec2,
    target_position: WorldVec2,
    slot_index: usize,
    slot_count: usize,
    orbit_radius: f32,
) -> WorldVec2 {
    let base_direction = (attacker_position - target_position).normalized_or_zero();
    let base_angle = if base_direction.length_squared() <= f32::EPSILON {
        0.0
    } else {
        base_direction.y.atan2(base_direction.x)
    };
    let angle = base_angle + std::f32::consts::TAU * (slot_index as f32 / slot_count as f32);
    target_position + WorldVec2::new(angle.cos(), angle.sin()) * orbit_radius
}

fn melee_engagement_point_is_valid(
    position: WorldVec2,
    unit_radius: f32,
    core: &BattleCore,
) -> bool {
    let projected = project_goal_point_to_battlefield(position, unit_radius, core);
    position.x.is_finite()
        && position.y.is_finite()
        && position.distance_squared(projected) <= f32::EPSILON
}

fn melee_engagement_point_tracks_target(
    position: WorldVec2,
    target_position: WorldVec2,
    orbit_radius: f32,
) -> bool {
    let drift = (position.distance(target_position) - orbit_radius).abs();
    drift <= MELEE_ENGAGEMENT_TARGET_DRIFT_TOLERANCE
}

fn melee_engagement_point_conflicts(
    position: WorldVec2,
    unit_radius: f32,
    reserved_positions: &[MeleeSlotReservation],
) -> bool {
    reserved_positions.iter().any(|reserved| {
        position.distance(reserved.position)
            < (unit_radius + reserved.unit_radius) * MELEE_ENGAGEMENT_SPACING_MULTIPLIER
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use uuid::Uuid;

    use super::*;
    use crate::game::{
        battle::{
            core::{
                movement::{types::UnitBody, ActionState},
                RuntimeUnit,
            },
            ids::UnitInstanceId,
            scenario::{
                BattleScenario, EnemyMovementPlan, FormationKind, GroupObjective,
                PlayerMovementPlan, TacticalGroupMembers, TacticalGroupPlan, TacticalGroupPlanId,
                TacticalPlan, TacticalPoint, TacticalPointId,
            },
        },
        data::{abnormality_data::BasicAttackDef, GameDataBase, GameDataBuilder},
        resources::Position,
        stats::UnitStats,
    };

    fn empty_game_data() -> Arc<GameDataBase> {
        GameDataBuilder::empty().build_arc()
    }

    fn core_with_player_plan(player_plan: PlayerMovementPlan) -> BattleCore {
        let mut scenario = BattleScenario::empty((8, 4));
        scenario.tactical_plan = TacticalPlan {
            player_plan,
            ..TacticalPlan::default()
        };
        BattleCore::new_from_scenario(scenario, empty_game_data(), 123)
    }

    fn runtime_unit(
        id: u128,
        owner: Side,
        position: WorldVec2,
        anchor: Option<WorldVec2>,
    ) -> RuntimeUnit {
        let mut stats = UnitStats::with_values(10, 10, 1, 0, 1);
        stats.move_speed_units_per_ms = 1_000_000;
        let mut basic_attack = BasicAttackDef::default();
        basic_attack.range_units = 0.5;

        RuntimeUnit {
            instance_id: UnitInstanceId::from(Uuid::from_u128(id)),
            source_owned_uuid: Uuid::from_u128(id),
            owner,
            role: crate::game::battle::types::BattleUnitRole::Combatant,
            base_uuid: Uuid::nil(),
            stats,
            basic_attack,
            skill_id: None,
            body: UnitBody::new_at(position, 0.35, 1.0),
            tactical_anchor: anchor,
            tactical_group_id: None,
            move_epoch: 0,
            action_state: ActionState::Idle,
            action_locks: Default::default(),
            current_target: None,
            next_basic_attack_ms: 0,
            pending_basic_attack: false,
            resonance_current: 0,
            resonance_max: 100,
            resonance_lock_ms: 0,
            next_action_time: 0,
            pending_cast: false,
            pending_cast_cause: None,
            pending_skill_cast: None,
        }
    }

    #[test]
    fn free_engage_keeps_existing_nearest_enemy_chase_goal() {
        let mut core = core_with_player_plan(PlayerMovementPlan::FreeEngage);
        let player_id = UnitInstanceId::from(Uuid::from_u128(1));
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        core.units.insert(
            player_id,
            runtime_unit(1, Side::Player, WorldVec2::new(1.0, 1.0), None),
        );
        core.units.insert(
            enemy_id,
            runtime_unit(2, Side::Opponent, WorldVec2::new(6.0, 1.0), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&player_id),
            Some(MovementGoal::AttackUnit { target_id, .. }) if *target_id == enemy_id
        ));
    }

    #[test]
    fn hold_deployment_does_not_chase_enemy_outside_anchor_reaction_radius() {
        let mut core = core_with_player_plan(PlayerMovementPlan::HoldDeployment {
            guard_radius: 1.0,
            leash_radius: 1.5,
            chase_radius: 0.5,
            return_to_anchor: true,
        });
        let player_id = UnitInstanceId::from(Uuid::from_u128(1));
        let anchor = WorldVec2::new(1.0, 1.0);
        core.units.insert(
            player_id,
            runtime_unit(1, Side::Player, anchor, Some(anchor)),
        );
        core.units.insert(
            UnitInstanceId::from(Uuid::from_u128(2)),
            runtime_unit(2, Side::Opponent, WorldVec2::new(6.0, 1.0), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(
            !goals.contains_key(&player_id),
            "hold deployment should preserve placement instead of chasing far enemies"
        );
    }

    #[test]
    fn hold_deployment_returns_to_anchor_when_unit_exceeds_leash() {
        let mut core = core_with_player_plan(PlayerMovementPlan::HoldDeployment {
            guard_radius: 1.0,
            leash_radius: 1.5,
            chase_radius: 0.5,
            return_to_anchor: true,
        });
        let player_id = UnitInstanceId::from(Uuid::from_u128(1));
        let anchor = WorldVec2::new(1.0, 1.0);
        core.units.insert(
            player_id,
            runtime_unit(1, Side::Player, WorldVec2::new(4.0, 1.0), Some(anchor)),
        );
        core.units.insert(
            UnitInstanceId::from(Uuid::from_u128(2)),
            runtime_unit(2, Side::Opponent, WorldVec2::new(5.0, 1.0), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&player_id),
            Some(MovementGoal::MoveToPoint { point, .. }) if *point == anchor
        ));
    }

    #[test]
    fn cautious_engage_chases_near_enemy_without_exceeding_anchor_leash() {
        let mut core = core_with_player_plan(PlayerMovementPlan::CautiousEngage {
            leash_radius: 2.5,
            chase_radius: 1.0,
        });
        let player_id = UnitInstanceId::from(Uuid::from_u128(1));
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        let anchor = WorldVec2::new(1.0, 1.0);
        core.units.insert(
            player_id,
            runtime_unit(1, Side::Player, anchor, Some(anchor)),
        );
        core.units.insert(
            enemy_id,
            runtime_unit(2, Side::Opponent, WorldVec2::new(2.6, 1.0), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&player_id),
            Some(MovementGoal::AttackUnit {
                target_id,
                approach_point: Some(point),
                ..
            }) if *target_id == enemy_id && point.distance(anchor) <= 2.5
        ));
    }

    #[test]
    fn cautious_engage_refuses_far_enemy_even_when_free_engage_would_chase() {
        let mut core = core_with_player_plan(PlayerMovementPlan::CautiousEngage {
            leash_radius: 2.5,
            chase_radius: 1.0,
        });
        let player_id = UnitInstanceId::from(Uuid::from_u128(1));
        let anchor = WorldVec2::new(1.0, 1.0);
        core.units.insert(
            player_id,
            runtime_unit(1, Side::Player, anchor, Some(anchor)),
        );
        core.units.insert(
            UnitInstanceId::from(Uuid::from_u128(2)),
            runtime_unit(2, Side::Opponent, WorldVec2::new(7.0, 1.0), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(
            !goals.contains_key(&player_id),
            "cautious engage should not collapse into unlimited free engage"
        );
    }

    #[test]
    fn cautious_engage_returns_to_anchor_when_outside_leash() {
        let mut core = core_with_player_plan(PlayerMovementPlan::CautiousEngage {
            leash_radius: 2.5,
            chase_radius: 1.0,
        });
        let player_id = UnitInstanceId::from(Uuid::from_u128(1));
        let anchor = WorldVec2::new(1.0, 1.0);
        core.units.insert(
            player_id,
            runtime_unit(1, Side::Player, WorldVec2::new(4.0, 1.0), Some(anchor)),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&player_id),
            Some(MovementGoal::MoveToPoint { point, .. }) if *point == anchor
        ));
    }

    #[test]
    fn goal_projection_snaps_non_rectangular_void_tile_to_walkable_tile() {
        let scenario = BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                width: 3,
                height: 3,
                valid_tiles: vec![
                    Position::new(1, 0),
                    Position::new(1, 1),
                    Position::new(1, 2),
                ],
                obstacles: Vec::new(),
            },
            artifacts: Vec::new(),
            groups: Vec::new(),
            events: Vec::new(),
            win_condition:
                crate::game::battle::scenario::WinCondition::AllRequiredEnemyGroupsDefeated,
            tactical_plan: TacticalPlan::default(),
        };
        let core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);

        let projected = project_goal_point_to_battlefield(
            WorldVec2::from_tile_center(Position::new(0, 1)),
            0.35,
            &core,
        );

        assert_eq!(projected, WorldVec2::from_tile_center(Position::new(1, 1)));
    }

    #[test]
    fn goal_projection_avoids_static_obstacle_tile() {
        let mut core =
            BattleCore::new_from_scenario(BattleScenario::empty((3, 3)), empty_game_data(), 123);
        core.battlefield
            .add_static_obstacle(Position::new(1, 1))
            .unwrap();

        let projected = project_goal_point_to_battlefield(
            WorldVec2::from_tile_center(Position::new(1, 1)),
            0.35,
            &core,
        );

        assert_ne!(projected.project_to_tile(), Position::new(1, 1));
        assert!(core
            .battlefield
            .is_walkable_tile(projected.project_to_tile()));
    }

    #[test]
    fn enemy_path_to_point_moves_to_tactical_point_instead_of_chasing_far_player() {
        let mut scenario = BattleScenario::empty((8, 4));
        let point_id = TacticalPointId::new("exit");
        scenario.tactical_plan = TacticalPlan {
            enemy_plan: EnemyMovementPlan::PathToPoint {
                point_id: point_id.clone(),
            },
            points: vec![TacticalPoint {
                id: point_id,
                position: Position::new(7, 1),
            }],
            ..TacticalPlan::default()
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        core.units.insert(
            UnitInstanceId::from(Uuid::from_u128(1)),
            runtime_unit(1, Side::Player, WorldVec2::new(1.0, 1.0), None),
        );
        core.units.insert(
            enemy_id,
            runtime_unit(2, Side::Opponent, WorldVec2::new(6.0, 1.0), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&enemy_id),
            Some(MovementGoal::MoveToPoint { point, .. })
                if *point == WorldVec2::from_tile_center(Position::new(7, 1))
        ));
    }

    #[test]
    fn enemy_path_to_point_still_attacks_enemy_already_in_range() {
        let mut scenario = BattleScenario::empty((8, 4));
        let point_id = TacticalPointId::new("exit");
        scenario.tactical_plan = TacticalPlan {
            enemy_plan: EnemyMovementPlan::PathToPoint {
                point_id: point_id.clone(),
            },
            points: vec![TacticalPoint {
                id: point_id,
                position: Position::new(7, 1),
            }],
            ..TacticalPlan::default()
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let player_id = UnitInstanceId::from(Uuid::from_u128(1));
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        core.units.insert(
            player_id,
            runtime_unit(1, Side::Player, WorldVec2::new(6.7, 1.0), None),
        );
        core.units.insert(
            enemy_id,
            runtime_unit(2, Side::Opponent, WorldVec2::new(6.0, 1.0), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&enemy_id),
            Some(MovementGoal::AttackUnit { target_id, .. }) if *target_id == player_id
        ));
    }

    #[test]
    fn enemy_path_along_path_skips_reached_waypoints_and_moves_to_next() {
        let mut scenario = BattleScenario::empty((8, 4));
        let first_id = TacticalPointId::new("entry");
        let second_id = TacticalPointId::new("exit");
        scenario.tactical_plan = TacticalPlan {
            enemy_plan: EnemyMovementPlan::PathAlongPath {
                point_ids: vec![first_id.clone(), second_id.clone()],
            },
            points: vec![
                TacticalPoint {
                    id: first_id,
                    position: Position::new(2, 1),
                },
                TacticalPoint {
                    id: second_id,
                    position: Position::new(6, 1),
                },
            ],
            ..TacticalPlan::default()
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        core.units.insert(
            UnitInstanceId::from(Uuid::from_u128(1)),
            runtime_unit(1, Side::Player, WorldVec2::new(1.0, 3.0), None),
        );
        core.units.insert(
            enemy_id,
            runtime_unit(2, Side::Opponent, WorldVec2::new(2.5, 1.5), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&enemy_id),
            Some(MovementGoal::MoveToPoint { point, .. })
                if *point == WorldVec2::from_tile_center(Position::new(6, 1))
        ));
    }

    #[test]
    fn enemy_empty_path_does_not_fall_back_to_chasing_far_player() {
        let mut scenario = BattleScenario::empty((8, 4));
        scenario.tactical_plan = TacticalPlan {
            enemy_plan: EnemyMovementPlan::PathAlongPath {
                point_ids: Vec::new(),
            },
            ..TacticalPlan::default()
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        core.units.insert(
            UnitInstanceId::from(Uuid::from_u128(1)),
            runtime_unit(1, Side::Player, WorldVec2::new(1.0, 1.0), None),
        );
        core.units.insert(
            enemy_id,
            runtime_unit(2, Side::Opponent, WorldVec2::new(7.0, 1.0), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(
            !goals.contains_key(&enemy_id),
            "empty enemy path should not become unlimited free engage"
        );
    }

    #[test]
    fn group_objective_moves_member_toward_bounded_advance_center_before_free_engage() {
        let group_id = TacticalGroupPlanId::new("player_main");
        let point_id = TacticalPointId::new("rally");
        let mut scenario = BattleScenario::empty((8, 4));
        scenario.tactical_plan = TacticalPlan {
            points: vec![TacticalPoint {
                id: point_id.clone(),
                position: Position::new(6, 1),
            }],
            group_plans: vec![TacticalGroupPlan {
                id: group_id.clone(),
                side: Side::Player,
                members: TacticalGroupMembers::SideAll(Side::Player),
                objective: GroupObjective::AdvanceToPoint {
                    point_id: point_id.clone(),
                },
                formation: FormationKind::Loose,
                cohesion_radius: 2.0,
                engage_radius: 2.0,
            }],
            ..TacticalPlan::default()
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let player_id = UnitInstanceId::from(Uuid::from_u128(1));
        let mut player = runtime_unit(1, Side::Player, WorldVec2::new(1.0, 1.0), None);
        player.tactical_group_id = Some(group_id.clone());
        core.units.insert(player_id, player);
        core.scenario_runtime
            .tactical_groups
            .insert(group_id, vec![player_id]);
        core.units.insert(
            UnitInstanceId::from(Uuid::from_u128(2)),
            runtime_unit(2, Side::Opponent, WorldVec2::new(7.0, 3.0), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&player_id),
            Some(MovementGoal::MoveToPoint { point, .. })
                if point.distance(WorldVec2::new(2.991_803_2, 1.181_073)) < 0.001
        ));
    }

    #[test]
    fn hold_area_still_uses_final_tactical_point_as_group_center() {
        let group_id = TacticalGroupPlanId::new("player_main");
        let point_id = TacticalPointId::new("hold");
        let mut scenario = BattleScenario::empty((8, 4));
        scenario.tactical_plan = TacticalPlan {
            points: vec![TacticalPoint {
                id: point_id.clone(),
                position: Position::new(6, 1),
            }],
            group_plans: vec![TacticalGroupPlan {
                id: group_id.clone(),
                side: Side::Player,
                members: TacticalGroupMembers::SideAll(Side::Player),
                objective: GroupObjective::HoldArea {
                    point_id: point_id.clone(),
                },
                formation: FormationKind::Loose,
                cohesion_radius: 2.0,
                engage_radius: 2.0,
            }],
            ..TacticalPlan::default()
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let player_id = UnitInstanceId::from(Uuid::from_u128(1));
        let mut player = runtime_unit(1, Side::Player, WorldVec2::new(1.0, 1.0), None);
        player.tactical_group_id = Some(group_id.clone());
        core.units.insert(player_id, player);
        core.scenario_runtime
            .tactical_groups
            .insert(group_id, vec![player_id]);

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&player_id),
            Some(MovementGoal::MoveToPoint { point, .. })
                if *point == WorldVec2::from_tile_center(Position::new(6, 1))
        ));
    }

    #[test]
    fn advance_along_path_skips_reached_waypoints_and_advances_toward_next() {
        let group_id = TacticalGroupPlanId::new("player_main");
        let first_id = TacticalPointId::new("first");
        let second_id = TacticalPointId::new("second");
        let mut scenario = BattleScenario::empty((8, 4));
        scenario.tactical_plan = TacticalPlan {
            points: vec![
                TacticalPoint {
                    id: first_id.clone(),
                    position: Position::new(2, 1),
                },
                TacticalPoint {
                    id: second_id.clone(),
                    position: Position::new(6, 1),
                },
            ],
            group_plans: vec![TacticalGroupPlan {
                id: group_id.clone(),
                side: Side::Player,
                members: TacticalGroupMembers::SideAll(Side::Player),
                objective: GroupObjective::AdvanceAlongPath {
                    point_ids: vec![first_id, second_id],
                },
                formation: FormationKind::Loose,
                cohesion_radius: 1.0,
                engage_radius: 0.5,
            }],
            ..TacticalPlan::default()
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let player_id = UnitInstanceId::from(Uuid::from_u128(1));
        let mut player = runtime_unit(1, Side::Player, WorldVec2::new(2.5, 1.5), None);
        player.tactical_group_id = Some(group_id.clone());
        core.units.insert(player_id, player);
        core.scenario_runtime
            .tactical_groups
            .insert(group_id, vec![player_id]);

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&player_id),
            Some(MovementGoal::MoveToPoint { point, .. })
                if *point == WorldVec2::new(3.5, 1.5)
        ));
    }

    #[test]
    fn advance_along_empty_path_does_not_fall_back_to_free_engage() {
        let group_id = TacticalGroupPlanId::new("player_main");
        let mut scenario = BattleScenario::empty((8, 4));
        scenario.tactical_plan = TacticalPlan {
            group_plans: vec![TacticalGroupPlan {
                id: group_id.clone(),
                side: Side::Player,
                members: TacticalGroupMembers::SideAll(Side::Player),
                objective: GroupObjective::AdvanceAlongPath {
                    point_ids: Vec::new(),
                },
                formation: FormationKind::Loose,
                cohesion_radius: 1.0,
                engage_radius: 0.5,
            }],
            ..TacticalPlan::default()
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let player_id = UnitInstanceId::from(Uuid::from_u128(1));
        let mut player = runtime_unit(1, Side::Player, WorldVec2::new(1.0, 1.0), None);
        player.tactical_group_id = Some(group_id.clone());
        core.units.insert(player_id, player);
        core.scenario_runtime
            .tactical_groups
            .insert(group_id, vec![player_id]);
        core.units.insert(
            UnitInstanceId::from(Uuid::from_u128(2)),
            runtime_unit(2, Side::Opponent, WorldVec2::new(7.0, 1.0), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(
            !goals.contains_key(&player_id),
            "empty group path should not become unlimited free engage"
        );
    }

    #[test]
    fn group_objective_still_allows_immediate_self_defense() {
        let group_id = TacticalGroupPlanId::new("player_main");
        let point_id = TacticalPointId::new("rally");
        let mut scenario = BattleScenario::empty((8, 4));
        scenario.tactical_plan = TacticalPlan {
            points: vec![TacticalPoint {
                id: point_id.clone(),
                position: Position::new(6, 1),
            }],
            group_plans: vec![TacticalGroupPlan {
                id: group_id.clone(),
                side: Side::Player,
                members: TacticalGroupMembers::SideAll(Side::Player),
                objective: GroupObjective::AdvanceToPoint { point_id },
                formation: FormationKind::Loose,
                cohesion_radius: 4.0,
                engage_radius: 2.0,
            }],
            ..TacticalPlan::default()
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let player_id = UnitInstanceId::from(Uuid::from_u128(1));
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        let mut player = runtime_unit(1, Side::Player, WorldVec2::new(1.0, 1.0), None);
        player.tactical_group_id = Some(group_id.clone());
        core.units.insert(player_id, player);
        core.scenario_runtime
            .tactical_groups
            .insert(group_id, vec![player_id]);
        core.units.insert(
            enemy_id,
            runtime_unit(2, Side::Opponent, WorldVec2::new(1.7, 1.0), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&player_id),
            Some(MovementGoal::AttackUnit { target_id, .. }) if *target_id == enemy_id
        ));
    }

    #[test]
    fn group_objective_uses_engage_radius_without_falling_back_to_free_engage() {
        let group_id = TacticalGroupPlanId::new("player_main");
        let point_id = TacticalPointId::new("rally");
        let mut scenario = BattleScenario::empty((8, 4));
        scenario.tactical_plan = TacticalPlan {
            points: vec![TacticalPoint {
                id: point_id.clone(),
                position: Position::new(4, 1),
            }],
            group_plans: vec![TacticalGroupPlan {
                id: group_id.clone(),
                side: Side::Player,
                members: TacticalGroupMembers::SideAll(Side::Player),
                objective: GroupObjective::HoldArea { point_id },
                formation: FormationKind::Loose,
                cohesion_radius: 1.0,
                engage_radius: 0.5,
            }],
            ..TacticalPlan::default()
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let player_id = UnitInstanceId::from(Uuid::from_u128(1));
        let mut player = runtime_unit(1, Side::Player, WorldVec2::new(4.5, 1.5), None);
        player.tactical_group_id = Some(group_id.clone());
        core.units.insert(player_id, player);
        core.scenario_runtime
            .tactical_groups
            .insert(group_id, vec![player_id]);
        core.units.insert(
            UnitInstanceId::from(Uuid::from_u128(2)),
            runtime_unit(2, Side::Opponent, WorldVec2::new(7.5, 1.5), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(
            !goals.contains_key(&player_id),
            "group objective should not fall back to unlimited free engage"
        );
    }

    #[test]
    fn group_objective_chases_near_enemy_inside_engage_radius_bounded_by_cohesion() {
        let group_id = TacticalGroupPlanId::new("player_main");
        let point_id = TacticalPointId::new("rally");
        let mut scenario = BattleScenario::empty((8, 4));
        scenario.tactical_plan = TacticalPlan {
            points: vec![TacticalPoint {
                id: point_id.clone(),
                position: Position::new(4, 1),
            }],
            group_plans: vec![TacticalGroupPlan {
                id: group_id.clone(),
                side: Side::Player,
                members: TacticalGroupMembers::SideAll(Side::Player),
                objective: GroupObjective::HoldArea {
                    point_id: point_id.clone(),
                },
                formation: FormationKind::Loose,
                cohesion_radius: 1.0,
                engage_radius: 0.6,
            }],
            ..TacticalPlan::default()
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let player_id = UnitInstanceId::from(Uuid::from_u128(1));
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        let formation_point = WorldVec2::from_tile_center(Position::new(4, 1));
        let mut player = runtime_unit(1, Side::Player, formation_point, None);
        player.tactical_group_id = Some(group_id.clone());
        core.units.insert(player_id, player);
        core.scenario_runtime
            .tactical_groups
            .insert(group_id, vec![player_id]);
        core.units.insert(
            enemy_id,
            runtime_unit(2, Side::Opponent, WorldVec2::new(6.0, 1.5), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&player_id),
            Some(MovementGoal::AttackUnit {
                target_id,
                approach_point: Some(point),
                ..
            }) if *target_id == enemy_id && point.distance(formation_point) <= 1.0
        ));
    }

    #[test]
    fn reconnect_to_group_moves_member_toward_target_group_live_center() {
        let main_group_id = TacticalGroupPlanId::new("player_main");
        let scout_group_id = TacticalGroupPlanId::new("scout");
        let mut scenario = BattleScenario::empty((8, 5));
        scenario.tactical_plan = TacticalPlan {
            group_plans: vec![
                TacticalGroupPlan {
                    id: main_group_id.clone(),
                    side: Side::Player,
                    members: TacticalGroupMembers::ExplicitUnits(vec![]),
                    objective: GroupObjective::FollowBattleObjective,
                    formation: FormationKind::Loose,
                    cohesion_radius: 4.0,
                    engage_radius: 2.0,
                },
                TacticalGroupPlan {
                    id: scout_group_id.clone(),
                    side: Side::Player,
                    members: TacticalGroupMembers::ExplicitUnits(vec![]),
                    objective: GroupObjective::ReconnectToGroup {
                        group_id: main_group_id.clone(),
                    },
                    formation: FormationKind::Loose,
                    cohesion_radius: 1.0,
                    engage_radius: 0.5,
                },
            ],
            ..TacticalPlan::default()
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let main_a_id = UnitInstanceId::from(Uuid::from_u128(1));
        let main_b_id = UnitInstanceId::from(Uuid::from_u128(2));
        let scout_id = UnitInstanceId::from(Uuid::from_u128(3));

        let mut main_a = runtime_unit(1, Side::Player, WorldVec2::new(5.0, 1.0), None);
        main_a.tactical_group_id = Some(main_group_id.clone());
        let mut main_b = runtime_unit(2, Side::Player, WorldVec2::new(5.0, 3.0), None);
        main_b.tactical_group_id = Some(main_group_id.clone());
        let mut scout = runtime_unit(3, Side::Player, WorldVec2::new(1.0, 1.0), None);
        scout.tactical_group_id = Some(scout_group_id.clone());
        core.units.insert(main_a_id, main_a);
        core.units.insert(main_b_id, main_b);
        core.units.insert(scout_id, scout);
        core.scenario_runtime
            .tactical_groups
            .insert(main_group_id, vec![main_a_id, main_b_id]);
        core.scenario_runtime
            .tactical_groups
            .insert(scout_group_id, vec![scout_id]);

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&scout_id),
            Some(MovementGoal::MoveToPoint { point, .. })
                if *point == WorldVec2::new(5.0, 2.0)
        ));
    }

    #[test]
    fn reconnect_to_missing_group_does_not_fall_back_to_free_engage() {
        let missing_group_id = TacticalGroupPlanId::new("missing_main");
        let scout_group_id = TacticalGroupPlanId::new("scout");
        let mut scenario = BattleScenario::empty((8, 5));
        scenario.tactical_plan = TacticalPlan {
            group_plans: vec![TacticalGroupPlan {
                id: scout_group_id.clone(),
                side: Side::Player,
                members: TacticalGroupMembers::ExplicitUnits(vec![]),
                objective: GroupObjective::ReconnectToGroup {
                    group_id: missing_group_id,
                },
                formation: FormationKind::Loose,
                cohesion_radius: 1.0,
                engage_radius: 0.5,
            }],
            ..TacticalPlan::default()
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let scout_id = UnitInstanceId::from(Uuid::from_u128(1));
        let mut scout = runtime_unit(1, Side::Player, WorldVec2::new(1.0, 1.0), None);
        scout.tactical_group_id = Some(scout_group_id.clone());
        core.units.insert(scout_id, scout);
        core.scenario_runtime
            .tactical_groups
            .insert(scout_group_id, vec![scout_id]);
        core.units.insert(
            UnitInstanceId::from(Uuid::from_u128(2)),
            runtime_unit(2, Side::Opponent, WorldVec2::new(7.0, 1.0), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(
            !goals.contains_key(&scout_id),
            "reconnect objective should not become unlimited free engage when target group is missing"
        );
    }
}
