//! Stateless, release-time footprint for native directional basic attacks.
use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        ability::SkillHitTargetFilter,
        battle::{ids::UnitInstanceId, runtime_control},
        behavior::GameError,
        resources::Position,
    },
};

fn delta(origin: Position, point: Position) -> (f64, f64) {
    // Convert before subtracting: authored map coordinates are signed i32.
    (
        f64::from(point.x) - f64::from(origin.x),
        f64::from(point.y) - f64::from(origin.y),
    )
}

fn in_length(origin: Position, target: Position, length: u32) -> bool {
    let (x, y) = delta(origin, target);
    let distance_sq = x * x + y * y;
    distance_sq > 0.0 && distance_sq <= f64::from(length).powi(2)
}

fn contains(
    origin: Position,
    aim: Position,
    point: Position,
    length: u32,
    half_width: u32,
) -> bool {
    let (ax, ay) = delta(origin, aim);
    let (px, py) = delta(origin, point);
    let axis_sq = ax * ax + ay * ay;
    let dot = px * ax + py * ay;
    let cross = px * ay - py * ax;
    // No normalization, angle rounding, trigonometry, or cardinal snapping.
    axis_sq > 0.0
        && dot > 0.0
        && dot * dot <= f64::from(length).powi(2) * axis_sq
        && cross * cross * f64::from(length).powi(2) <= dot * dot * f64::from(half_width).powi(2)
}

impl BattleCore {
    pub(super) fn basic_cone_primary_in_range(
        &self,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        length: u32,
        policy: &super::targeting::BasicAttackRangePolicy<'_>,
        time_ms: Option<u64>,
    ) -> bool {
        use crate::game::battle::tile_range::TileReachQuery;
        let tile = |id| match time_ms {
            Some(now) => self
                .sample_unit_body_at(id, now)
                .map(|body| body.position.project_to_tile()),
            None => self.live_unit_projected_tile(id),
        };
        let Some((origin, aim)) = tile(actor).zip(tile(target)) else {
            return false;
        };
        if !in_length(origin, aim, length) {
            return false;
        }
        if let Some(pattern) = policy.tile_range.as_deref() {
            if Self::charge_authoring_tile_pattern(pattern, 2).is_err() {
                return false;
            }
        }
        TileReachQuery::try_new(
            &self.battlefield,
            policy.range_policy,
            policy.tile_range.as_deref(),
            Some(origin),
            self.units
                .get(&actor)
                .and_then(|unit| unit.facing_direction),
        )
        .is_ok_and(|query| query.contains(aim))
    }

    pub(super) fn collect_basic_cone_targets(
        &self,
        actor: UnitInstanceId,
        primary: UnitInstanceId,
        length: u32,
        half_width: u32,
        source_side: crate::game::enums::Side,
        now: u64,
    ) -> Result<Option<Vec<UnitInstanceId>>, GameError> {
        let count = self.units.len();
        control::charge(
            (count as u64).saturating_mul(16 + u64::from(usize::BITS - count.leading_zeros())),
            "basic_cone_target_scan",
        )?;
        self.admit_authoring_runtime_growth(0, "basic_cone_targets")?;
        runtime_control::admit_retained_growth(
            0,
            count.checked_mul(std::mem::size_of::<UnitInstanceId>()),
            "basic_cone_targets",
        )?;
        let mut targets = Vec::with_capacity(count);
        let Some(source) = self.units.get(&actor).filter(|unit| unit.is_active()) else {
            return Ok(None);
        };
        let Some((origin, aim)) = self
            .sample_unit_body_at(actor, now)
            .zip(self.sample_unit_body_at(primary, now))
            .map(|(source, target)| {
                (
                    source.position.project_to_tile(),
                    target.position.project_to_tile(),
                )
            })
        else {
            return Ok(None);
        };
        if !in_length(origin, aim, length) {
            return Ok(None);
        }
        for unit in self.units.values() {
            if !self.skill_delivery_accepts_unit(
                source_side,
                actor,
                unit.instance_id,
                SkillHitTargetFilter::Enemies,
                false,
            ) || !self
                .single_target_can_target_unit(unit.instance_id, source.basic_attack.air_capable)
            {
                continue;
            }
            let Some(body) = self.sample_unit_body_at(unit.instance_id, now) else {
                continue;
            };
            let tile = body.position.project_to_tile();
            if self.battlefield.is_valid_tile(tile)
                && contains(origin, aim, tile, length, half_width)
            {
                targets.push(unit.instance_id);
            }
        }
        targets.sort_unstable();
        Ok(Some(targets))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_cone_geometry_widens_and_rotates_without_cardinal_snapping() {
        let p = Position::new;
        let origin = p(0, 0);
        for (point, expected) in [
            (p(1, 0), true),
            (p(1, 1), false),
            (p(3, 1), true),
            (p(5, 2), true),
            (p(5, -2), true),
            (p(5, 3), false),
            (p(6, 0), false),
            (p(0, 0), false),
            (p(-1, 0), false),
        ] {
            assert_eq!(
                contains(origin, p(1, 0), point, 5, 2),
                expected,
                "{point:?}"
            );
            assert_eq!(
                contains(origin, p(0, 1), p(-point.y, point.x), 5, 2),
                expected
            );
        }
        assert!(contains(origin, p(2, 2), p(3, 3), 5, 2));
        assert!(contains(origin, p(2, 2), p(2, 4), 5, 2));
        assert!(!contains(origin, p(2, 2), p(4, 4), 5, 2));
        assert!(!contains(origin, p(2, 2), p(1, 4), 5, 2));
        assert!(!in_length(origin, p(4, 4), 5));
        assert!(in_length(origin, p(3, 4), 5));
    }
}
