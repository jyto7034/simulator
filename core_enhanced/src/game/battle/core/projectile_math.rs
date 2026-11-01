use super::movement::types::{WorldVec2, DATA_UNITS_PER_WORLD};
use super::spatial::moving_circle_sweep_hit_fraction;

pub(in crate::game::battle::core) const PROJECTILE_REEVALUATION_TICK_MS: u64 = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::game::battle::core) struct ProjectileMotionWindow {
    pub(in crate::game::battle::core) start_time_ms: u64,
    pub(in crate::game::battle::core) end_time_ms: u64,
    pub(in crate::game::battle::core) start_position: WorldVec2,
    pub(in crate::game::battle::core) end_position: WorldVec2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::game::battle::core) struct ProjectileCollisionCandidate<I> {
    pub(in crate::game::battle::core) id: I,
    pub(in crate::game::battle::core) target_start: WorldVec2,
    pub(in crate::game::battle::core) target_end: WorldVec2,
    pub(in crate::game::battle::core) reach: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::game::battle::core) struct ProjectileCollisionImpact<I> {
    pub(in crate::game::battle::core) id: I,
    pub(in crate::game::battle::core) time_ms: u64,
    pub(in crate::game::battle::core) position: WorldVec2,
}

#[derive(Debug, Clone, PartialEq)]
pub(in crate::game::battle::core) struct ProjectileMotionResult<I> {
    pub(in crate::game::battle::core) end_position: WorldVec2,
    pub(in crate::game::battle::core) next_reevaluation_ms: u64,
    pub(in crate::game::battle::core) ordered_impacts: Vec<ProjectileCollisionImpact<I>>,
}

impl ProjectileMotionWindow {
    pub(in crate::game::battle::core) fn speed_limited(
        start_time_ms: u64,
        end_time_ms: u64,
        start_position: WorldVec2,
        aim: WorldVec2,
        speed_units_per_ms: u32,
    ) -> Self {
        let duration_ms = end_time_ms.saturating_sub(start_time_ms);
        let direction = aim - start_position;
        let distance = direction.length();
        let speed_world_per_ms = speed_units_per_ms as f32 / DATA_UNITS_PER_WORLD;
        let max_step = speed_world_per_ms * duration_ms as f32;
        let end_position =
            if distance <= f32::EPSILON || speed_units_per_ms == 0 || max_step >= distance {
                aim
            } else {
                start_position + direction * (max_step / distance)
            };

        Self {
            start_time_ms,
            end_time_ms,
            start_position,
            end_position,
        }
    }

    pub(in crate::game::battle::core) fn fixed_trajectory(
        start_time_ms: u64,
        end_time_ms: u64,
        trajectory_start: WorldVec2,
        trajectory_aim: WorldVec2,
        from_elapsed_ms: u64,
        to_elapsed_ms: u64,
        total_ms: u64,
    ) -> Self {
        Self {
            start_time_ms,
            end_time_ms,
            start_position: sample_projectile_position_at(
                trajectory_start,
                trajectory_aim,
                from_elapsed_ms,
                total_ms,
            ),
            end_position: sample_projectile_position_at(
                trajectory_start,
                trajectory_aim,
                to_elapsed_ms,
                total_ms,
            ),
        }
    }

    pub(in crate::game::battle::core) fn resolve<I: Ord>(
        self,
        candidates: impl IntoIterator<Item = ProjectileCollisionCandidate<I>>,
    ) -> ProjectileMotionResult<I> {
        let duration_ms = self.end_time_ms.saturating_sub(self.start_time_ms);
        let mut impacts = candidates
            .into_iter()
            .filter_map(|candidate| {
                let hit_fraction = moving_circle_sweep_hit_fraction(
                    self.start_position,
                    self.end_position,
                    candidate.reach,
                    candidate.target_start,
                    candidate.target_end,
                )?;
                let elapsed_delta = ((duration_ms as f32) * hit_fraction).ceil() as u64;
                let time_ms = self
                    .start_time_ms
                    .saturating_add(elapsed_delta.min(duration_ms));
                let position = interpolate_at_hit_fraction(
                    self.start_position,
                    self.end_position,
                    hit_fraction,
                );
                let target_position = interpolate_at_hit_fraction(
                    candidate.target_start,
                    candidate.target_end,
                    hit_fraction,
                );
                Some((
                    ProjectileCollisionImpact {
                        id: candidate.id,
                        time_ms,
                        position,
                    },
                    position.distance_squared(target_position),
                ))
            })
            .collect::<Vec<_>>();

        impacts.sort_by(|(a, a_distance), (b, b_distance)| {
            a.time_ms
                .cmp(&b.time_ms)
                .then_with(|| a_distance.total_cmp(b_distance))
                .then_with(|| a.id.cmp(&b.id))
        });

        ProjectileMotionResult {
            end_position: self.end_position,
            next_reevaluation_ms: self
                .end_time_ms
                .saturating_add(PROJECTILE_REEVALUATION_TICK_MS),
            ordered_impacts: impacts.into_iter().map(|(impact, _)| impact).collect(),
        }
    }
}

pub(in crate::game::battle::core) fn sample_projectile_position_at(
    start: WorldVec2,
    aim: WorldVec2,
    elapsed_ms: u64,
    total_ms: u64,
) -> WorldVec2 {
    if total_ms == 0 || elapsed_ms >= total_ms {
        return aim;
    }

    let t = elapsed_ms as f32 / total_ms.max(1) as f32;
    start + (aim - start) * t
}

fn interpolate_at_hit_fraction(start: WorldVec2, end: WorldVec2, hit_fraction: f32) -> WorldVec2 {
    start + (end - start) * hit_fraction.clamp(0.0, 1.0)
}

pub(in crate::game::battle::core) fn projectile_flight_ms(
    distance_units: u64,
    speed_units_per_ms: u32,
) -> u64 {
    if distance_units == 0 {
        return 0;
    }
    let speed_units_per_ms = speed_units_per_ms as u64;
    if speed_units_per_ms == 0 {
        // Published data rejects zero speed. Preserve the established defensive computation for
        // programmatic callers without treating it as a valid publication fallback.
        return 0;
    }
    distance_units.saturating_add(speed_units_per_ms.saturating_sub(1)) / speed_units_per_ms
}

pub(in crate::game::battle::core) fn projectile_flight_ms_between_world_points(
    start: WorldVec2,
    aim: WorldVec2,
    speed_units_per_ms: u32,
) -> u64 {
    let distance_units = (start.distance(aim) * DATA_UNITS_PER_WORLD).ceil().max(0.0) as u64;
    projectile_flight_ms(distance_units, speed_units_per_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motion_window_orders_same_time_impacts_by_distance_then_id() {
        let window = ProjectileMotionWindow::fixed_trajectory(
            10,
            20,
            WorldVec2::ZERO,
            WorldVec2::new(10.0, 0.0),
            0,
            10,
            10,
        );
        let result = window.resolve([
            ProjectileCollisionCandidate {
                id: 3_u8,
                target_start: WorldVec2::new(5.0, 0.20),
                target_end: WorldVec2::new(5.0, 0.20),
                reach: 0.25,
            },
            ProjectileCollisionCandidate {
                id: 2_u8,
                target_start: WorldVec2::new(5.0, 0.10),
                target_end: WorldVec2::new(5.0, 0.10),
                reach: 0.25,
            },
            ProjectileCollisionCandidate {
                id: 1_u8,
                target_start: WorldVec2::new(5.0, 0.10),
                target_end: WorldVec2::new(5.0, 0.10),
                reach: 0.25,
            },
        ]);

        assert_eq!(
            result
                .ordered_impacts
                .iter()
                .map(|impact| impact.id)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(result.next_reevaluation_ms, 21);
    }
}

pub(in crate::game::battle::core) fn rect_entry(a: WorldVec2, b: WorldVec2, min: WorldVec2, max: WorldVec2) -> Option<f32> {
    let mut enter = 0.0f32;
    let mut leave = 1.0f32;
    for (start, delta, lo, hi) in [
        (a.x, b.x - a.x, min.x, max.x),
        (a.y, b.y - a.y, min.y, max.y),
    ] {
        if delta.abs() < 1e-9 {
            if start < lo || start > hi {
                return None;
            }
        } else {
            let x = (lo - start) / delta;
            let y = (hi - start) / delta;
            enter = enter.max(x.min(y));
            leave = leave.min(x.max(y));
            if enter > leave {
                return None;
            }
        }
    }
    Some(enter)
}

impl super::BattleCore {
    pub(super) fn projectile_tile_valid(&self, tile: crate::game::resources::Position) -> bool {
        let f = &self.scenario.battlefield;
        tile.x >= 0
            && tile.y >= 0
            && tile.x < i32::from(f.width)
            && tile.y < i32::from(f.height)
            && (f.valid_tiles.is_empty() || f.valid_tiles.contains(&tile))
            && !f.obstacles.contains(&tile)
    }
}
