//! Per-effect spatial resolution. The original carrier and sibling target
//! paths remain unchanged; persistent hit history is indexed inside its exact
//! immutable step/generation owner.
use super::{
    types::{AreaInstanceId, AreaRuntime, EffectHitHistory},
    BattleCore,
};
use crate::{
    authoring::control,
    game::{
        ability::{
            authoring::AuthoredSkillEffect, DeliveryDef, SkillAreaTickPolicy, SkillAreaTracking,
            SkillHitTargetFilter, SkillStepDef,
        },
        battle::{ids::UnitInstanceId, runtime_facts::RuntimeFactSource},
        behavior::GameError,
        relic::structural::authoring::StructuralRecipeDef,
    },
};
use std::collections::BTreeMap;

pub(super) struct EffectiveEffectTargets {
    pub targets: Vec<UnitInstanceId>,
    history: Option<(AreaInstanceId, usize, EffectHitHistory)>,
}
impl EffectHitHistory {
    pub(super) fn retained_extent(&self) -> Option<usize> {
        self.hit_unit_ids
            .capacity()
            .checked_add(self.previous_tick_unit_ids.capacity())?
            .checked_mul(std::mem::size_of::<UnitInstanceId>())?
            .checked_add(std::mem::size_of::<Self>())
    }
}
impl AreaRuntime {
    pub(super) fn effective_history_extent(&self) -> Option<usize> {
        let Some(history) = &self.effective_history else {
            return Some(0);
        };
        control::charge(history.len() as u64, "effect_area_history_extent").ok()?;
        history.values().try_fold(
            std::mem::size_of::<BTreeMap<usize, EffectHitHistory>>(),
            |sum, row| sum.checked_add(row.retained_extent()?)?.checked_add(64),
        )
    }
    pub(super) fn admit_effective_history_copy(&self) -> Result<(), GameError> {
        if self.effective_history.is_none() {
            return Ok(());
        }
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.effective_history_extent(),
            "effect_area_history_copy",
        )?;
        if let Some(history) = &self.effective_history {
            for row in history.values() {
                control::charge(
                    row.hit_unit_ids.len() as u64 + row.previous_tick_unit_ids.len() as u64,
                    "effect_area_history_clone",
                )?;
            }
        }
        Ok(())
    }
}
impl BattleCore {
    pub(super) fn effective_effect_targets(
        &self,
        effect_index: usize,
        effect: &AuthoredSkillEffect,
        step: &SkillStepDef,
        source: &RuntimeFactSource,
        time_ms: u64,
    ) -> Result<Option<EffectiveEffectTargets>, GameError> {
        let Some(binding) = source.relic_effective() else {
            return Ok(None);
        };
        let site = &effect.source().ok_or(GameError::InvalidAction)?.site;
        control::charge(
            binding.execution().effects.len() as u64,
            "effect_footprint_descriptor",
        )?;
        let descriptor = binding.effect(site).ok_or(GameError::InvalidAction)?;
        let Some(footprint) = descriptor.recipes.iter().find_map(|recipe| match recipe {
            StructuralRecipeDef::DamageFootprint { footprint } => Some(footprint),
            _ => None,
        }) else {
            return Ok(None);
        };
        let cast = source.cast.as_ref().ok_or(GameError::InvalidAction)?;
        let runtime = if let DeliveryDef::TileArea { area, .. } = &step.delivery {
            if area.is_independent() {
                control::charge(self.active_areas.len() as u64, "effect_area_owner_lookup")?;
                Some(
                    self.active_areas
                        .values()
                        .find(|runtime| {
                            runtime.cast_seq == cast.cast_seq
                                && runtime.step_id == step.id
                                && source
                                    .flow
                                    .as_ref()
                                    .is_none_or(|flow| runtime.step_index == flow.occurrence)
                        })
                        .ok_or(GameError::InvalidAction)?,
                )
            } else {
                None
            }
        } else {
            None
        };
        let (side, tile, facing, filter, include_caster) = if let Some(runtime) = runtime {
            let (_, _, _, tile) = self
                .persistent_tile_area_geometry_at(time_ms, runtime)
                .ok_or(GameError::InvalidAction)?;
            let facing = if runtime.tracking == SkillAreaTracking::GroundFixed {
                runtime.facing
            } else {
                self.tile_area_projection_facing(cast.caster_instance_id, footprint)
                    .ok_or(GameError::InvalidAction)?
            };
            (
                runtime.caster_owner,
                tile,
                facing,
                runtime.hit_targets,
                runtime.include_caster,
            )
        } else {
            let owner = self
                .active_skill_casts
                .get(&cast.cast_seq)
                .ok_or(GameError::InvalidAction)?;
            let (side, tile) = self
                .resolve_cast_origin_context(Some(cast.cast_seq), cast.caster_instance_id, false)
                .ok_or(GameError::InvalidAction)?;
            let tile = if let DeliveryDef::TileArea { area, .. } = &step.delivery {
                self.resolve_tile_area_geometry(
                    time_ms,
                    cast.cast_seq,
                    source.flow.as_ref().map_or(0, |flow| flow.occurrence),
                    cast.caster_instance_id,
                    owner.flow.as_ref().and_then(|flow| flow.accepted_target()),
                    area,
                )
                .ok_or(GameError::InvalidAction)?
                .4
            } else {
                self.sample_unit_world_position_at(cast.caster_instance_id, time_ms)
                    .map_or(tile, |position| position.project_to_tile())
            };
            let facing = match &step.delivery {
                DeliveryDef::TileArea { area, .. } => self.tile_area_facing(cast.caster_instance_id, footprint, area),
                _ => self.tile_area_projection_facing(cast.caster_instance_id, footprint),
            }.ok_or(GameError::InvalidAction)?;
            let (filter, include) = match &step.delivery {
                DeliveryDef::TileArea { area, .. } => (area.hit_targets, area.include_caster),
                DeliveryDef::Instant => (SkillHitTargetFilter::Enemies, false),
                DeliveryDef::Projectile { .. } => return Err(GameError::InvalidAction),
            };
            (side, tile, facing, filter, include)
        };
        let tiles = self
            .valid_tile_area_tiles(footprint, tile, facing)
            .ok_or(GameError::InvalidAction)?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.units
                .len()
                .checked_mul(std::mem::size_of::<UnitInstanceId>()),
            "effect_footprint_targets",
        )?;
        let raw = self.collect_tile_area_targets_at(
            time_ms,
            side,
            cast.caster_instance_id,
            &tiles,
            filter,
            include_caster,
        );
        control::poll("effect_footprint_resolved")?;
        let Some(runtime) = runtime else {
            return Ok(Some(EffectiveEffectTargets {
                targets: raw,
                history: None,
            }));
        };
        if runtime.tick_policy == SkillAreaTickPolicy::EveryTick {
            return Ok(Some(EffectiveEffectTargets {
                targets: raw,
                history: None,
            }));
        }
        let prior = runtime
            .effective_history
            .as_ref()
            .and_then(|rows| rows.get(&effect_index));
        let old = prior
            .map(|row| {
                if runtime.tick_policy == SkillAreaTickPolicy::OncePerArea {
                    row.hit_unit_ids.as_slice()
                } else {
                    row.previous_tick_unit_ids.as_slice()
                }
            })
            .unwrap_or(&[]);
        control::charge(
            (raw.len() as u64)
                .checked_mul(old.len() as u64 + 1)
                .ok_or(GameError::InvalidAction)?,
            "effect_area_tick_filter",
        )?;
        let slots = raw
            .len()
            .checked_mul(2)
            .and_then(|n| n.checked_add(old.len()))
            .ok_or(GameError::InvalidAction)?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            slots
                .checked_mul(std::mem::size_of::<UnitInstanceId>())
                .and_then(|bytes| bytes.checked_add(std::mem::size_of::<EffectHitHistory>() + 64)),
            "effect_area_next_history",
        )?;
        let targets = raw.iter().copied().filter(|id| !old.contains(id)).collect();
        let history = if runtime.tick_policy == SkillAreaTickPolicy::OncePerArea {
            let mut hit_unit_ids = Vec::with_capacity(old.len() + raw.len());
            hit_unit_ids.extend_from_slice(old);
            hit_unit_ids.extend_from_slice(&raw);
            control::charge(
                (hit_unit_ids.len() as u64)
                    .checked_mul(u64::from(usize::BITS - hit_unit_ids.len().leading_zeros()) + 1)
                    .ok_or(GameError::InvalidAction)?,
                "effect_area_history_sort",
            )?;
            hit_unit_ids.sort_unstable();
            hit_unit_ids.dedup();
            EffectHitHistory {
                hit_unit_ids,
                previous_tick_unit_ids: Vec::new(),
            }
        } else {
            EffectHitHistory {
                hit_unit_ids: Vec::new(),
                previous_tick_unit_ids: raw,
            }
        };
        Ok(Some(EffectiveEffectTargets {
            targets,
            history: Some((runtime.area_id, effect_index, history)),
        }))
    }
    pub(super) fn commit_effect_target_history(
        &mut self,
        selection: Option<EffectiveEffectTargets>,
    ) -> Result<(), GameError> {
        let Some((id, index, row)) = selection.and_then(|selection| selection.history) else {
            return Ok(());
        };
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            row.retained_extent().and_then(|bytes| {
                bytes.checked_add(64 + std::mem::size_of::<BTreeMap<usize, EffectHitHistory>>())
            }),
            "effect_area_history_commit",
        )?;
        let runtime = self
            .active_areas
            .get_mut(&id)
            .ok_or(GameError::InvalidAction)?;
        runtime
            .effective_history
            .get_or_insert_with(|| Box::new(BTreeMap::new()))
            .insert(index, row);
        Ok(())
    }
}
