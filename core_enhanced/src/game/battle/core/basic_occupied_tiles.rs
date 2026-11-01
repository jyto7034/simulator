//! One release-time snapshot of the enemy-occupied tiles in the weapon range.
use super::{targeting::BasicAttackRangePolicy, BattleCore};
use crate::{
    authoring::control,
    game::{
        ability::SkillHitTargetFilter,
        battle::{ids::UnitInstanceId, runtime_control, tile_range::TileReachQuery},
        behavior::GameError,
        resources::Position,
    },
};

impl BattleCore {
    fn occupied_tile_query<'a>(
        &'a self,
        actor: UnitInstanceId,
        policy: &'a BasicAttackRangePolicy<'_>,
        now: Option<u64>,
    ) -> Option<TileReachQuery<'a>> {
        let origin = match now {
            Some(now) => self
                .sample_unit_body_at(actor, now)?
                .position
                .project_to_tile(),
            None => self.live_unit_projected_tile(actor)?,
        };
        if let Some(pattern) = policy.tile_range.as_deref() {
            Self::charge_authoring_tile_pattern(pattern, 2).ok()?;
        }
        TileReachQuery::try_new(
            &self.battlefield,
            policy.range_policy,
            policy.tile_range.as_deref(),
            Some(origin),
            self.units.get(&actor)?.facing_direction,
        )
        .ok()
    }

    pub(super) fn basic_occupied_tile_in_range(
        &self,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        policy: &BasicAttackRangePolicy<'_>,
        now: Option<u64>,
    ) -> bool {
        let tile = match now {
            Some(now) => self
                .sample_unit_body_at(target, now)
                .map(|body| body.position.project_to_tile()),
            None => self.live_unit_projected_tile(target),
        };
        tile.is_some_and(|tile| {
            self.occupied_tile_query(actor, policy, now)
                .is_some_and(|query| query.contains(tile))
        })
    }

    pub(super) fn collect_basic_occupied_tiles(
        &self,
        actor: UnitInstanceId,
        now: u64,
    ) -> Result<Option<(Vec<Position>, Vec<UnitInstanceId>, bool)>, GameError> {
        let count = self.units.len();
        control::charge(
            (count as u64).saturating_mul(16 + 2 * u64::from(usize::BITS - count.leading_zeros())),
            "basic_occupied_tiles_scan",
        )?;
        self.admit_authoring_runtime_growth(0, "basic_occupied_tiles")?;
        runtime_control::admit_retained_growth(
            0,
            count.checked_mul(
                std::mem::size_of::<Position>() + std::mem::size_of::<UnitInstanceId>(),
            ),
            "basic_occupied_tiles",
        )?;
        let Some(source) = self.units.get(&actor).filter(|unit| unit.is_active()) else {
            return Ok(None);
        };
        let policy = self.basic_attack_range_policy_for_unit(source);
        let Some(query) = self.occupied_tile_query(actor, &policy, Some(now)) else {
            return Ok(None);
        };
        // Each membership test scans the authored mask, including validation.
        if let Some(pattern) = policy.tile_range.as_deref() {
            Self::charge_authoring_tile_pattern(pattern, count as u64)?;
        }
        let mut tiles = Vec::with_capacity(count);
        let mut targets = Vec::with_capacity(count);
        for unit in self.units.values() {
            if !self.skill_delivery_accepts_unit(
                source.owner,
                actor,
                unit.instance_id,
                SkillHitTargetFilter::Enemies,
                false,
            ) || !self.single_target_can_target_unit(unit.instance_id, policy.air_capable)
            {
                continue;
            }
            let Some(body) = self.sample_unit_body_at(unit.instance_id, now) else {
                continue;
            };
            let tile = body.position.project_to_tile();
            if query.contains(tile) {
                tiles.push(tile);
                targets.push(unit.instance_id);
            }
        }
        // Capture both before applying any damage/reaction. A shared tile is one
        // visual strike, while each eligible occupant receives one full hit.
        tiles.sort_unstable_by_key(|p| (p.x, p.y));
        tiles.dedup();
        targets.sort_unstable();
        Ok(Some((tiles, targets, policy.air_capable)))
    }
}
