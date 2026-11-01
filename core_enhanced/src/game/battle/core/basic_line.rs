use super::{
    damage_runtime::{CommittedDamageInput, CommittedDamagePolicy},
    movement::types::{WorldVec2, DATA_UNITS_PER_WORLD},
    BattleCore,
};
use crate::{
    authoring::control,
    game::{
        ability::sustained::BasicLineProfile,
        battle::{damage::DamageSourceSnapshot, ids::UnitInstanceId},
        behavior::GameError,
    },
};

impl BattleCore {
    pub(super) fn skill_preserves_basic_clock(skill: &crate::game::ability::SkillDef) -> bool {
        use crate::game::ability::{sustained::SustainedEffectDef, SkillEffectDef};
        Self::relation_action_skill(skill)
            || skill
                .steps
                .iter()
                .flat_map(|s| &s.effects)
                .any(|effect| match &effect.op {
                    SkillEffectDef::Sustained {
                        definition: SustainedEffectDef::BasicAttackOverlay { profile, .. },
                    } => profile.line.is_some(),
                    SkillEffectDef::Sustained {
                        definition: SustainedEffectDef::State { program, .. },
                    } => program
                        .basic_attack
                        .as_ref()
                        .is_some_and(|p| p.line.is_some()),
                    _ => false,
                })
    }

    pub(super) fn line_target_in_range(
        &self,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        line: &BasicLineProfile,
    ) -> bool {
        let Some(a) = self.unit_world_position_or_tile_center(actor) else {
            return false;
        };
        let Some(t) = self.unit_world_position_or_tile_center(target) else {
            return false;
        };
        let Some(unit) = self.units.get(&target) else {
            return false;
        };
        a.distance(t)
            <= line.length_units as f32 / DATA_UNITS_PER_WORLD
                + line.width_units as f32 / DATA_UNITS_PER_WORLD / 2.0
                + unit.body.radius
    }

    pub(super) fn resolve_basic_line(
        &mut self,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        mut source: DamageSourceSnapshot,
        line: BasicLineProfile,
        now: u64,
    ) -> Result<bool, GameError> {
        let origin = self
            .unit_world_position_or_tile_center(actor)
            .ok_or(GameError::InvalidAction)?;
        let aim = self
            .unit_world_position_or_tile_center(target)
            .ok_or(GameError::InvalidAction)?;
        let delta = aim - origin;
        let direction = if delta.length() > f32::EPSILON {
            delta * (1.0 / delta.length())
        } else {
            let (x, y) = self.units[&actor]
                .facing_direction
                .ok_or(GameError::InvalidAction)?
                .rotate_offset(0, -1);
            WorldVec2 {
                x: x as f32,
                y: y as f32,
            }
        };
        let end = origin + direction * (line.length_units as f32 / DATA_UNITS_PER_WORLD);
        let air = self.units[&actor].basic_attack.air_capable;
        control::charge(self.units.len() as u64, "basic_line_targets")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.units
                .len()
                .checked_mul(std::mem::size_of::<(f32, UnitInstanceId)>()),
            "basic_line_target_storage",
        )?;
        let mut hits = Vec::with_capacity(self.units.len());
        for (id, unit) in &self.units {
            if unit.owner == source.source_side || !self.single_target_can_target_unit(*id, air) {
                continue;
            }
            let Some(pos) = self.unit_world_position_or_tile_center(*id) else {
                continue;
            };
            let reach = line.width_units as f32 / DATA_UNITS_PER_WORLD / 2.0 + unit.body.radius;
            if let Some(fraction) =
                super::spatial::moving_circle_sweep_hit_fraction(origin, end, reach, pos, pos)
            {
                hits.push((fraction, *id));
            }
        }
        // Contact order, then stable identity for simultaneous contacts. A unit
        // contributes once even when it occupies multiple map cells.
        hits.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        let piercing = line.piercing.as_ref().filter(|p| {
            line.piercing_relics
                && self
                    .equipped_normal_relic_copy(actor, &p.relic_definition)
                    .is_some()
        });
        source.base_damage = line.power;
        if line.power == 0 {
            source.minimum_damage = 0;
        }
        if let Some(prepared) = &source.prepared_basic {
            source.base_damage = prepared.apply(source.base_damage)?;
        }
        self.grant_basic_attack_release_resonance(actor, now);
        self.release_prepared_basic(&source)?;
        self.consume_basic_overlay(actor, now)?;
        let occurrence_id = source.crit_roll_identity.source_instance_id;
        let mut total = 0;
        let mut prior_contacts = 0u64;
        for (_, id) in hits {
            control::poll("basic_line_before_hit")?;
            if !self.units.get(&id).is_some_and(|u| u.is_active()) {
                continue;
            }
            let mut snapshot = crate::authoring::clone_shape::controlled_clone(&source)?;
            if let Some(rule) = piercing {
                let bonus = (prior_contacts * u64::from(rule.increment_percent))
                    .min(rule.cap_percent.into());
                snapshot.base_damage = u32::try_from(
                    (u128::from(snapshot.base_damage) * u128::from(100 + bonus)) / 100,
                )
                .map_err(|_| {
                    crate::game::battle::runtime_control::invariant("basic_line_power_overflow")
                })?;
            }
            let result = self
                .execute_committed_damage(CommittedDamageInput {
                    target_id: id,
                    source_snapshot: snapshot,
                    committed_at_ms: now,
                    policy: CommittedDamagePolicy::BasicAttack {
                        occurrence_id,
                        occurrence_time_ms: now,
                    },
                })
                .ok_or(GameError::InvalidAction)?;
            Self::add_basic_hp_loss(&mut total, result.actual_hp_loss)?;
            // Geometric contact counts even when armor, immunity or a miss
            // prevents HP loss; successful contact is distinct from damage.
            prior_contacts += 1;
        }
        self.finish_basic_lifesteal(&source, total, now)?;
        control::poll("basic_line_after_settlement")?;
        self.schedule_pending_autocasts(now);
        Ok(true)
    }
}
