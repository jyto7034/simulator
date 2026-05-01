use std::collections::{HashMap, HashSet};

use crate::game::battle::{
    core::{BattleCore, MeleeSlotReservation},
    ids::UnitInstanceId,
};

use super::types::{MovementGoal, WorldVec2};

const MELEE_ENGAGEMENT_RANGE_THRESHOLD: f32 = 1.0;
const MELEE_ENGAGEMENT_SPACING_MULTIPLIER: f32 = 1.15;
const MELEE_ENGAGEMENT_TARGET_DRIFT_TOLERANCE: f32 = 0.35;

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
            if unit.is_dead() || unit.stats.move_speed_units_per_ms == 0 {
                continue;
            }

            let Some(target_id) = self
                .continuous_sticky_target(unit_id)
                .or_else(|| self.nearest_continuous_enemy(unit_id))
            else {
                continue;
            };

            let desired_range = self.basic_attack_range_units(unit.base_uuid);
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
                        &mut occupied_slots,
                        &mut reserved_positions,
                    )
                } else {
                    None
                };

            goals.insert(
                unit_id,
                MovementGoal::AttackUnit {
                    target_id,
                    desired_range,
                    approach_point: engagement_point,
                },
            );
        }

        goals
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
        let board_width = f32::from(self.battlefield.width());
        let board_height = f32::from(self.battlefield.height());

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
                && melee_engagement_point_is_valid(
                    previous.position,
                    unit.body.radius,
                    board_width,
                    board_height,
                )
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
            if !melee_engagement_point_is_valid(
                position,
                unit.body.radius,
                board_width,
                board_height,
            ) {
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
    board_width: f32,
    board_height: f32,
) -> bool {
    position.x.is_finite()
        && position.y.is_finite()
        && position.x >= unit_radius
        && position.y >= unit_radius
        && position.x <= board_width - unit_radius
        && position.y <= board_height - unit_radius
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
