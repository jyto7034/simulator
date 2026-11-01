//! Immutable release geometry; target membership is resolved at projectile impact.
use super::{
    damage_runtime::{CommittedDamageInput, CommittedDamagePolicy},
    movement::types::WorldVec2,
    types::ProjectileRecord,
    BattleCore,
};
use crate::{
    authoring::{clone_shape::controlled_clone, control},
    game::{
        ability::SkillHitTargetFilter,
        battle::{
            damage::DamageSourceSnapshot,
            event_log::BattleLogEvent,
            ids::UnitInstanceId,
            runtime_control,
            tile_range::{FacingDirection, TileRangePattern},
        },
        behavior::GameError,
        data::abnormality_data::BasicAttackAction,
    },
};
use std::{mem::size_of, sync::Arc};
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub struct BasicProjectileArea {
    footprint: TileRangePattern,
    facing: FacingDirection,
    secondary_percent: u32,
}
impl BasicProjectileArea {
    pub(super) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = (size_of::<Self>() + 64).checked_add(
            self.footprint
                .rows
                .capacity()
                .checked_mul(size_of::<String>())?,
        )?;
        for row in &self.footprint.rows {
            bytes = bytes.checked_add(row.capacity())?;
        }
        Some(bytes)
    }
}

impl BattleCore {
    pub(super) fn capture_basic_projectile_area(
        &self,
        actor: UnitInstanceId,
        action: &BasicAttackAction,
    ) -> Result<Option<Arc<BasicProjectileArea>>, GameError> {
        let BasicAttackAction::DamageArea {
            footprint,
            secondary_percent,
        } = action
        else {
            return Ok(None);
        };
        let facing = self
            .tile_area_projection_facing(actor, footprint)
            .ok_or(GameError::InvalidAction)?;
        let area = BasicProjectileArea {
            footprint: controlled_clone(footprint)?,
            facing,
            secondary_percent: *secondary_percent,
        };
        runtime_control::admit_retained_growth(
            0,
            area.retained_extent(),
            "basic_projectile_area_capture",
        )?;
        Ok(Some(Arc::new(area)))
    }

    /// Shared instant/impact damage loop: one committed basic hit per selected enemy.
    pub(super) fn apply_basic_area_targets(
        &mut self,
        targets: Vec<UnitInstanceId>,
        primary: UnitInstanceId,
        source: &DamageSourceSnapshot,
        secondary_percent: u32,
        air_capable: bool,
        occurrence_id: Uuid,
        now: u64,
    ) -> Result<u64, GameError> {
        let mut actual_hp_loss = 0;
        for target in targets {
            if !self.single_target_can_target_unit(target, air_capable) {
                continue;
            }
            let mut snapshot = controlled_clone(source)?;
            if target != primary {
                snapshot.base_damage =
                    (u64::from(snapshot.base_damage) * u64::from(secondary_percent) / 100)
                        .min(u64::from(u32::MAX)) as u32;
            }
            let summary = self
                .execute_committed_damage(CommittedDamageInput {
                    target_id: target,
                    source_snapshot: snapshot,
                    committed_at_ms: now,
                    policy: CommittedDamagePolicy::BasicAttack {
                        occurrence_id,
                        occurrence_time_ms: now,
                    },
                })
                .ok_or(GameError::InvalidAction)?;
            Self::add_basic_hp_loss(&mut actual_hp_loss, summary.actual_hp_loss)?;
        }
        Ok(actual_hp_loss)
    }

    pub(super) fn resolve_basic_projectile_area(
        &mut self,
        projectile: &ProjectileRecord,
        area: &BasicProjectileArea,
        now: u64,
        projectile_id: Uuid,
        primary: UnitInstanceId,
        mut source: DamageSourceSnapshot,
        impact_position: WorldVec2,
    ) -> Result<u64, GameError> {
        let anchor = self
            .sample_unit_body_at(primary, now)
            .ok_or(GameError::InvalidAction)?
            .position
            .project_to_tile();
        let tiles = self
            .valid_tile_area_tiles(&area.footprint, anchor, area.facing)
            .ok_or(GameError::InvalidAction)?;
        let mut targets = self.collect_tile_area_targets_at(
            now,
            projectile.attacker_owner_at_launch,
            projectile.attacker_instance_id,
            &tiles,
            SkillHitTargetFilter::Enemies,
            false,
        );
        targets.retain(|target| {
            self.single_target_can_target_unit(*target, projectile.air_capable_at_launch)
        });
        let percent = self.area_damage_percent(
            projectile.attacker_instance_id,
            projectile.attacker_owner_at_launch,
            &source.fact_source,
            &targets,
            now,
        )?;
        source.modifiers.damage_amp_percent =
            source.modifiers.damage_amp_percent.saturating_add(percent);
        Self::scale_magazine_collision(&mut source, true);
        self.record_event_log(
            now,
            BattleLogEvent::BasicAttackProjectileImpacted {
                projectile_id,
                attacker_instance_id: projectile.attacker_instance_id,
                target_instance_id: primary,
                impact_position: impact_position.quantized_milli(),
                hit: true,
            },
        );
        let actual = self.apply_basic_area_targets(
            targets,
            primary,
            &source,
            area.secondary_percent,
            projectile.air_capable_at_launch,
            projectile_id,
            now,
        )?;
        control::poll("basic_projectile_area_resolved")?;
        self.schedule_pending_autocasts(now);
        Ok(actual)
    }
}
