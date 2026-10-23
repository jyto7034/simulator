use super::super::{
    damage_runtime::{CommittedDamageInput, CommittedDamagePolicy},
    marks::{ratio_product, ratio_sum},
};
use super::*;
use crate::game::{
    ability::{
        sustained::{
            HitModifierSource, HitStatusKind, HitStatusProfile, HitVulnerability,
            SustainedEffectDef,
        },
        SkillHitTargetFilter,
    },
    battle::damage::{DamageModifiers, DamageSource, DamageType},
};
impl BattleCore {
    pub(super) fn relation_primary(
        &self,
        caster: UnitInstanceId,
        range: &crate::game::battle::tile_range::TileRangePattern,
        now: u64,
    ) -> Option<UnitInstanceId> {
        let u = self.units.get(&caster)?;
        let tiles = range
            .affected_tiles(
                self.sample_unit_body_at(caster, now)?
                    .position
                    .project_to_tile(),
                u.facing_direction.unwrap_or(FacingDirection::Up),
            )
            .ok()?;
        self.collect_tile_area_targets_at(
            now,
            u.owner,
            caster,
            &tiles,
            SkillHitTargetFilter::Enemies,
            false,
        )
        .into_iter()
        .filter(|t| self.single_target_can_target_unit(*t, false))
        .min_by(|a, b| {
            (self.blocked_by(*a) != Some(caster))
                .cmp(&(self.blocked_by(*b) != Some(caster)))
                .then_with(|| self.compare_default_forward_target(*a, *b))
        })
    }
    pub(super) fn relation_refresh_primary(
        &self,
        caster: UnitInstanceId,
        p: &RelationActionDef,
        a: &mut Active,
        now: u64,
    ) -> Result<bool, GameError> {
        let Some(primary) = a.admitted.primary else {
            return Ok(true);
        };
        if self.units.get(&primary).is_some_and(|u| u.is_active()) {
            a.primary_position = self
                .sample_unit_body_at(primary, now)
                .map(|u| u.position.project_to_tile());
            return Ok(true);
        }
        if !self
            .units
            .get(&primary)
            .is_some_and(|u| u.lifecycle.is_dead())
        {
            return Ok(false);
        }
        let entry = Self::relation_entry(p, &a.admitted);
        let distance = p
            .modifiers
            .iter()
            .filter_map(|m| match &m.change {
                RelationActionChange::Retarget {
                    entry: e,
                    distance_units,
                } if e == &entry.definition => Some(*distance_units),
                _ => None,
            })
            .max();
        let Some(distance) = distance else {
            return Ok(false);
        };
        let Some(origin) = self.sample_unit_body_at(primary, now).map(|b| b.position) else {
            return Ok(false);
        };
        let limit = distance as f32 / super::super::movement::types::DATA_UNITS_PER_WORLD;
        control::charge(self.units.len() as u64, "relation_retarget_scan")?;
        let next = self
            .units
            .values()
            .filter(|u| {
                u.is_active()
                    && u.owner != self.units[&caster].owner
                    && self.single_target_can_target_unit(u.instance_id, false)
            })
            .filter_map(|u| {
                self.sample_unit_body_at(u.instance_id, now)
                    .map(|b| (u.instance_id, origin.distance_squared(b.position)))
            })
            .filter(|(_, d)| *d <= limit * limit)
            .min_by(|(a, da), (b, db)| {
                da.total_cmp(db)
                    .then_with(|| self.compare_default_forward_target(*a, *b))
            })
            .map(|(id, _)| id);
        a.admitted.primary = next;
        a.primary_position = next.and_then(|t| {
            self.sample_unit_body_at(t, now)
                .map(|u| u.position.project_to_tile())
        });
        Ok(next.is_some())
    }
    pub(super) fn relation_strike(
        &mut self,
        key: Uuid,
        p: &RelationActionDef,
        index: usize,
        paid: u32,
        a: &mut Active,
        now: u64,
    ) -> Result<(), GameError> {
        let ActionPlan::Strike {
            target,
            power,
            damage_type,
            range,
            after,
            ramp,
        } = &p.plans[index].op
        else {
            return Err(GameError::InvalidAction);
        };
        let caster = self.sustained.relations.owners[&key].caster;
        a.last_strike_primary = a.admitted.primary;
        let primary_area=p.modifiers.iter().any(|m|matches!(&m.change,RelationActionChange::PrimaryArea{definition} if definition==&p.plans[index].definition));
        let targets = if *target == PlanTarget::Primary && !primary_area {
            a.admitted.primary.into_iter().collect::<Vec<_>>()
        } else {
            let (origin, facing) = match target {
                PlanTarget::Primary => (
                    a.primary_position.ok_or(GameError::InvalidAction)?,
                    FacingDirection::Up,
                ),
                PlanTarget::SelectedArea => (a.admitted.tile, FacingDirection::Up),
                PlanTarget::ForwardArea => (
                    self.sample_unit_body_at(caster, now)
                        .ok_or(GameError::InvalidAction)?
                        .position
                        .project_to_tile(),
                    a.admitted.facing,
                ),
                PlanTarget::SelectedDirectionArea => (
                    self.sample_unit_body_at(caster, now)
                        .ok_or(GameError::InvalidAction)?
                        .position
                        .project_to_tile(),
                    a.admitted.facing,
                ),
            };
            let tiles = if primary_area {
                vec![origin]
            } else {
                range
                    .as_ref()
                    .ok_or(GameError::InvalidAction)?
                    .pattern
                    .affected_tiles(origin, facing)
                    .map_err(GameError::InvalidStaticData)?
            };
            self.collect_tile_area_targets_at(
                now,
                self.units[&caster].owner,
                caster,
                &tiles,
                SkillHitTargetFilter::Enemies,
                false,
            )
        };
        for target in targets {
            if !self.relation_activation_exists(key, a.id) {
                break;
            }
            if !self.units.get(&target).is_some_and(|u| u.is_active())
                || !self.single_target_can_target_unit(target, false)
            {
                continue;
            }
            let mut bonus = *a.ramps.get(&target).unwrap_or(&ZERO);
            let entry = Self::relation_entry(p, &a.admitted);
            let prior_hits = a.hits.get(&target).copied().unwrap_or(0);
            for m in &p.modifiers {
                if let RelationActionChange::Multihit {
                    entry: e,
                    step,
                    cap,
                } = &m.change
                {
                    if e == &entry.definition {
                        let value = ratio_product(
                            *step,
                            MarkRatio {
                                numerator: u64::from(prior_hits) + 1,
                                denominator: 1,
                            },
                        )?;
                        let value = if u128::from(value.numerator) * u128::from(cap.denominator)
                            > u128::from(cap.numerator) * u128::from(value.denominator)
                        {
                            *cap
                        } else {
                            value
                        };
                        bonus = ratio_sum(bonus, value)?;
                    }
                }
            }
            let mut snapshot =
                controlled_clone(a.snapshot.as_ref().ok_or(GameError::InvalidAction)?)?;
            snapshot.base_damage = program::scale(
                u64::from(*power),
                ratio_sum(
                    MarkRatio {
                        numerator: 1,
                        denominator: 1,
                    },
                    ratio_product(
                        bonus,
                        MarkRatio {
                            numerator: 1,
                            denominator: 100,
                        },
                    )?,
                )?,
            )?;
            if let Some(prepared) = &snapshot.prepared_basic {
                snapshot.base_damage = prepared.apply(snapshot.base_damage)?;
            }
            if *power == 0 {
                snapshot.minimum_damage = 0;
            }
            snapshot.damage_type = *damage_type;
            snapshot.committed_at_ms = now;
            snapshot.crit_roll_percent = None;
            let policy = if snapshot.source == DamageSource::BasicAttack {
                CommittedDamagePolicy::BasicAttack {
                    occurrence_id: snapshot.crit_roll_identity.source_instance_id,
                    occurrence_time_ms: now,
                }
            } else {
                CommittedDamagePolicy::GeneralCommand
            };
            let snapshot = self.materialize_damage_source_snapshot_for_target_hit(
                snapshot,
                target,
                a.hit_index,
            );
            let summary = self.execute_committed_damage(CommittedDamageInput {
                target_id: target,
                source_snapshot: snapshot,
                committed_at_ms: now,
                policy,
            });
            control::poll("relation_strike_damage")?;
            if let Some(summary) = summary {
                a.actual_hp_loss = a
                    .actual_hp_loss
                    .checked_add(u64::from(summary.actual_hp_loss))
                    .ok_or(GameError::InvalidAction)?;
                if summary.effective_hit_count > 0 {
                    if !a.hits.contains_key(&target) {
                        self.admit_authoring_runtime_growth(2, "relation_hit_entities")?;
                        crate::game::battle::runtime_control::admit_retained_growth(
                            0,
                            Some(256),
                            "relation_hit_history",
                        )?;
                    }
                    a.hits.insert(
                        target,
                        prior_hits.checked_add(1).ok_or(GameError::InvalidAction)?,
                    );
                    a.ramps.insert(
                        target,
                        ratio_sum(*a.ramps.get(&target).unwrap_or(&ZERO), *ramp)?,
                    );
                    if self.relation_activation_exists(key, a.id) {
                        for status in after {
                            self.relation_apply_status(
                                caster,
                                target,
                                p,
                                status,
                                a.start_stock,
                                paid,
                                &a.source,
                                now,
                            )?;
                        }
                    }
                }
            }
            a.hit_index = a.hit_index.checked_add(1).ok_or(GameError::InvalidAction)?;
        }
        Ok(())
    }
    pub(super) fn relation_apply_status(
        &mut self,
        caster: UnitInstanceId,
        target: UnitInstanceId,
        p: &RelationActionDef,
        status: &PlanStatus,
        stock: u32,
        paid: u32,
        source: &RuntimeFactSource,
        now: u64,
    ) -> Result<(), GameError> {
        if !self.units.get(&target).is_some_and(|u| u.is_active()) {
            return Ok(());
        }
        let mut count = status
            .count
            .evaluate(stock, paid)
            .ok_or(GameError::InvalidAction)?;
        for m in &p.modifiers {
            if let RelationActionChange::StatusCount { definition, factor } = &m.change {
                if definition == &status.definition {
                    count = program::scale(count.into(), *factor)?;
                }
            }
        }
        if count == 0 {
            return Ok(());
        }
        let kind = match status.kind {
            PlanStatusKind::Stun => HitStatusKind::Stun,
            PlanStatusKind::BasicLock => HitStatusKind::BasicLock,
            PlanStatusKind::Vulnerability => HitStatusKind::StackDamageTaken {
                per_stack: status.percent,
                stacks: count,
            },
            PlanStatusKind::Root => HitStatusKind::Root,
        };
        let vulnerability = p.modifiers.iter().find_map(|m| match &m.change {
            RelationActionChange::ControlVulnerability {
                definition,
                percent,
                duration_ms,
            } if definition == &status.definition => Some(HitVulnerability {
                definition: m.id.clone(),
                percent: *percent,
                duration_ms: *duration_ms,
                modifier_source: self
                    .equipped_normal_relic_copy(caster, &m.relic_definition)
                    .map(|copy_uuid| HitModifierSource {
                        copy_uuid,
                        definition: m.relic_definition.clone(),
                        rule: m.id.clone(),
                    }),
            }),
            _ => None,
        });
        let prior = std::mem::replace(&mut self.runtime_fact_source, source.independent_origin()?);
        let result = self.begin_sustained(
            caster,
            target,
            SustainedEffectDef::HitStatus {
                duration_ms: status.duration_ms,
                profile: HitStatusProfile {
                    definition: status.definition.clone(),
                    kind,
                    modifier_source: None,
                    vulnerability,
                },
            },
            now,
        );
        self.runtime_fact_source = prior;
        result
    }
    pub(super) fn check_relation_shield(&mut self, key: Uuid, now: u64) -> Result<(), GameError> {
        let o = &self.sustained.relations.owners[&key];
        if o.shield_used || o.mode || !self.relation_live(o) {
            return Ok(());
        }
        let Some(target) = o.relation.as_ref().map(|r| r.target) else {
            return Ok(());
        };
        let Some(u) = self.units.get(&target).filter(|u| u.is_active()) else {
            return Ok(());
        };
        if u64::from(u.stats.current_health) * 100
            > u64::from(u.stats.max_health) * u64::from(o.program.shield_threshold)
        {
            return Ok(());
        }
        let amount = program::percentage(u.stats.max_health.into(), o.program.shield_percent)?;
        if amount == 0 {
            return Ok(());
        }
        let caster = o.caster;
        let duration_ms = o.program.shield_duration_ms;
        let source = RuntimeFactSource {
            actor: Some(caster),
            fragment: Some(o.fragment.clone()),
            ..Default::default()
        };
        self.sustained
            .relations
            .owners
            .get_mut(&key)
            .unwrap()
            .shield_used = true;
        let prior = std::mem::replace(&mut self.runtime_fact_source, source);
        let result = self.begin_sustained(
            caster,
            target,
            SustainedEffectDef::IndependentShield {
                duration_ms,
                amount,
                priority: 0,
            },
            now,
        );
        self.runtime_fact_source = prior;
        result?;
        control::poll("relation_emergency_shield")
    }
    pub(in crate::game::battle::core) fn relation_shields_after_damage(
        &mut self,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        let keys: Vec<_> = self
            .sustained
            .relations
            .owners
            .iter()
            .filter(|(_, o)| o.relation.as_ref().is_some_and(|r| r.target == target))
            .map(|(k, _)| *k)
            .collect();
        for key in keys {
            self.check_relation_shield(key, now)?;
        }
        Ok(())
    }
    pub(in crate::game::battle::core) fn relation_reduction(
        &self,
        target: UnitInstanceId,
    ) -> Result<i32, GameError> {
        control::charge(
            self.sustained.relations.owners.len() as u64,
            "relation_reduction",
        )?;
        let mut sum = ZERO;
        for o in self
            .sustained
            .relations
            .owners
            .values()
            .filter(|o| self.relation_live(o))
        {
            if o.relation.as_ref().is_some_and(|r| r.target == target) && !o.mode {
                sum = ratio_sum(sum, o.program.reduction)?;
            }
            if o.caster == target {
                if let Some(a) = &o.active {
                    sum = ratio_sum(sum, a.stance)?;
                }
            }
        }
        Ok(program::scale(1, sum)?.min(i32::MAX as u32) as i32)
    }
    pub(super) fn relation_self_damage(
        &mut self,
        caster: UnitInstanceId,
        fragment: &Arc<FragmentSourceSnapshot>,
        percent: MarkRatio,
        count: u32,
        damage_type: DamageType,
        bypass: bool,
        now: u64,
    ) -> Result<(), GameError> {
        let hp = self
            .units
            .get(&caster)
            .ok_or(GameError::InvalidAction)?
            .stats
            .max_health;
        let damage = program::percentage(u64::from(hp) * u64::from(count), percent)?;
        if damage == 0 {
            return Ok(());
        }
        let mut snapshot = self
            .damage_source_snapshot_template_for_unit(
                caster,
                DamageSource::Ability,
                damage_type,
                damage,
                DamageModifiers::default(),
                0,
                now,
                false,
            )
            .ok_or(GameError::InvalidAction)?;
        snapshot.fact_source = RuntimeFactSource {
            actor: Some(caster),
            fragment: Some(fragment.clone()),
            ..Default::default()
        };
        self.execute_committed_damage(CommittedDamageInput {
            target_id: caster,
            source_snapshot: snapshot,
            committed_at_ms: now,
            policy: if bypass {
                CommittedDamagePolicy::BypassShields
            } else {
                CommittedDamagePolicy::GeneralCommand
            },
        });
        control::poll("relation_self_damage")
    }
}
