use super::super::{
    damage_runtime::{CommittedDamageInput, CommittedDamagePolicy},
    projectile_math::{rect_entry, ProjectileCollisionCandidate, ProjectileMotionWindow},
};
use super::*;
use crate::game::resources::Position;
use crate::game::{
    ability::{
        relation_action::{PlanStatus, PlanStatusKind},
        sustained::{HitModifierSource, HitStatusKind, HitStatusProfile},
        SkillHitTargetFilter,
    },
    battle::{
        damage::{DamageModifiers, DamageSource},
        tile_range::FacingDirection,
    },
};
impl BattleCore {
    fn volley_target(
        &self,
        key: Uuid,
        caster: UnitInstanceId,
        now: u64,
    ) -> Result<Option<UnitInstanceId>, GameError> {
        let p = &self.sustained.volleys.owners[&key].program;
        let origin = self
            .sample_unit_body_at(caster, now)
            .ok_or(GameError::InvalidAction)?
            .position
            .project_to_tile();
        let tiles = p
            .selection_range
            .affected_tiles(origin, FacingDirection::Up)
            .map_err(GameError::InvalidStaticData)?;
        let mut targets = self.collect_tile_area_targets_at(
            now,
            Side::Player,
            caster,
            &tiles,
            SkillHitTargetFilter::Enemies,
            false,
        );
        targets.retain(|t| self.single_target_can_target_unit(*t, p.selection_air));
        control::charge(
            (targets.len() as u64).saturating_mul((targets.len() + 1).ilog2() as u64 + 1),
            "volley_target_order",
        )?;
        targets.sort_by(|a, b| {
            let marked = |t| {
                self.sustained
                    .volleys
                    .marks
                    .get(&(key, t))
                    .is_some_and(|expires| now < *expires)
            };
            marked(*a)
                .cmp(&marked(*b))
                .then_with(|| {
                    self.enemy_route_progress(*b)
                        .total_cmp(&self.enemy_route_progress(*a))
                })
                .then_with(|| self.compare_enemy_spawn_order(*a, *b))
        });
        Ok(targets.first().copied())
    }
    pub(in crate::game::battle::core) fn release_next_volley_projectile(
        &mut self,
        key: Uuid,
        activation: u64,
        now: u64,
    ) -> Result<(), GameError> {
        let o = &self.sustained.volleys.owners[&key];
        let Some(a) = o.active.as_ref().filter(|a| a.id == activation) else {
            return Ok(());
        };
        if a.flight.is_some() {
            return Ok(());
        }
        let caster = a.caster;
        let continuous = continuous(&o.program);
        let interval = continuous.map_or(1, |(_, _, ms)| ms);
        if continuous.is_none() && a.prepared == 0 {
            return self.finish_prepared_volley(key, now, true, false);
        }
        if self
            .unit_control_state(caster, now)
            .is_some_and(|s| s.silenced)
        {
            self.sustained
                .volleys
                .owners
                .get_mut(&key)
                .unwrap()
                .active
                .as_mut()
                .unwrap()
                .next_at = Some(
                now.checked_add(interval.into())
                    .ok_or(GameError::InvalidAction)?,
            );
            return Ok(());
        }
        let Some(target) = self.volley_target(key, caster, now)? else {
            if continuous.is_none() {
                return self.finish_prepared_volley(key, now, true, false);
            }
            self.sustained
                .volleys
                .owners
                .get_mut(&key)
                .unwrap()
                .active
                .as_mut()
                .unwrap()
                .next_at = Some(
                now.checked_add(interval.into())
                    .ok_or(GameError::InvalidAction)?,
            );
            return Ok(());
        };
        let o = &self.sustained.volleys.owners[&key];
        let a = o.active.as_ref().unwrap();
        let program = o.program.clone();
        let program_bytes = o.program_bytes;
        let counts = controlled_clone(&a.hits)?;
        let applied = Arc::new(controlled_clone(&o.applied)?);
        let parent = controlled_clone(&a.source)?;
        let position = self
            .sample_unit_body_at(caster, now)
            .ok_or(GameError::InvalidAction)?
            .position;
        let mut source = self
            .damage_source_snapshot_template_for_unit(
                caster,
                DamageSource::Ability,
                program.projectile.damage_type,
                program.projectile.power,
                DamageModifiers::default(),
                0,
                now,
                false,
            )
            .ok_or(GameError::InvalidAction)?;
        source.fact_source = self.reserve_finite_source(parent)?;
        let id = self.next_volley_id()?;
        self.admit_authoring_runtime_growth(1, "volley_flight")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<Flight>() + 128),
            "volley_flight_storage",
        )?;
        self.sustained.volleys.flights.insert(
            id,
            Flight {
                id,
                activation,
                copy: key,
                caster,
                target,
                source,
                program,
                program_bytes,
                applied,
                position,
                released: now,
                last: now,
                traveled: 0.0,
                hits: Vec::new(),
                counts,
            },
        );
        let a = self
            .sustained
            .volleys
            .owners
            .get_mut(&key)
            .unwrap()
            .active
            .as_mut()
            .unwrap();
        a.flight = Some(id);
        if continuous.is_none() {
            a.prepared -= 1;
        }
        if self.sustained.volleys.next_flight.is_none() {
            self.sustained.volleys.next_flight = Some(now);
            self.event_queue
                .push(BattleEvent::VolleyFlightTick { time_ms: now });
        }
        control::poll("volley_projectile_released")
    }
    pub(in crate::game::battle::core) fn volley_flight_tick(
        &mut self,
        now: u64,
    ) -> Result<(), GameError> {
        if self.sustained.volleys.next_flight != Some(now) {
            return Ok(());
        }
        self.sustained.volleys.next_flight = None;
        let ids: Vec<_> = self.sustained.volleys.flights.keys().copied().collect();
        for id in ids {
            let Some(mut f) = self.sustained.volleys.flights.remove(&id) else {
                continue;
            };
            self.sustained.volleys.processing = Some(controlled_clone(&f.source.fact_source)?);
            let end = self.advance_volley_projectile(&mut f, now)?;
            self.sustained.volleys.processing = None;
            if let Some(end) = end {
                self.finish_volley_projectile(f, now, end, true)?;
            } else {
                self.sustained.volleys.flights.insert(id, f);
            }
        }
        if !self.sustained.volleys.flights.is_empty() {
            let next = now.checked_add(1).ok_or(GameError::InvalidAction)?;
            self.sustained.volleys.next_flight = Some(next);
            self.event_queue
                .push(BattleEvent::VolleyFlightTick { time_ms: next });
        }
        Ok(())
    }
    fn advance_volley_projectile(
        &mut self,
        f: &mut Flight,
        now: u64,
    ) -> Result<Option<VolleyFlightEnd>, GameError> {
        let p = &f.program.projectile;
        if !self.skill_delivery_accepts_unit(
            f.source.source_side,
            f.caster,
            f.target,
            SkillHitTargetFilter::Enemies,
            false,
        ) || !self.single_target_can_target_unit(f.target, p.air)
        {
            return Ok(Some(VolleyFlightEnd::TargetLost));
        }
        let from = f.last;
        let until = now.min(
            f.released
                .checked_add(p.lifetime_ms.into())
                .ok_or(GameError::InvalidAction)?,
        );
        let a = self
            .sample_unit_body_at(f.target, from)
            .ok_or(GameError::InvalidAction)?;
        let b = self
            .sample_unit_body_at(f.target, until)
            .ok_or(GameError::InvalidAction)?;
        let delta = b.position - f.position;
        let distance = delta.length();
        let step = (ratio(p.speed) * (until - from) as f32 / 1000.0)
            .min((ratio(p.range) - f.traveled).max(0.0))
            .min(distance);
        let desired = if distance <= f32::EPSILON {
            f.position
        } else {
            f.position + delta * (step / distance)
        };
        let field = &self.scenario.battlefield;
        let mut fraction = 1.0f32;
        let mut terminal = None;
        for (s, d, lo, hi) in [
            (
                f.position.x,
                desired.x - f.position.x,
                0.0,
                f32::from(field.width),
            ),
            (
                f.position.y,
                desired.y - f.position.y,
                0.0,
                f32::from(field.height),
            ),
        ] {
            let boundary = if d > 0.0 {
                (hi - s) / d
            } else if d < 0.0 {
                (lo - s) / d
            } else {
                2.0
            };
            if boundary <= fraction {
                fraction = boundary.max(0.0);
                terminal = Some(VolleyFlightEnd::Boundary);
            }
        }
        if p.obstacle_stop {
            control::charge(
                u64::from(field.width) * u64::from(field.height),
                "volley_obstacle_trace",
            )?;
            let radius = ratio(p.width) * 0.5;
            for x in 0..i32::from(field.width) {
                for y in 0..i32::from(field.height) {
                    let tile = Position::new(x, y);
                    if !self.projectile_tile_valid(tile) {
                        if let Some(t) = rect_entry(
                            f.position,
                            desired,
                            WorldVec2::new(x as f32 - radius, y as f32 - radius),
                            WorldVec2::new(x as f32 + 1.0 + radius, y as f32 + 1.0 + radius),
                        ) {
                            if t <= fraction {
                                fraction = t;
                                terminal = Some(VolleyFlightEnd::Obstacle);
                            }
                        }
                    }
                }
            }
        }
        let window = ProjectileMotionWindow {
            start_time_ms: from,
            end_time_ms: until,
            start_position: f.position,
            end_position: desired,
        };
        let hit = window
            .resolve([ProjectileCollisionCandidate {
                id: f.target,
                target_start: a.position,
                target_end: b.position,
                reach: ratio(p.width) * 0.5 + a.radius.max(b.radius).max(0.0),
            }])
            .ordered_impacts
            .into_iter()
            .next();
        if let Some(hit) = hit {
            let impact_distance = (hit.position - f.position).length();
            if terminal.is_none() || impact_distance < step * fraction {
                let center = self
                    .sample_unit_body_at(f.target, hit.time_ms)
                    .ok_or(GameError::InvalidAction)?
                    .position
                    .project_to_tile();
                let tiles = p
                    .area
                    .affected_tiles(center, FacingDirection::Up)
                    .map_err(GameError::InvalidStaticData)?;
                let targets = self.collect_tile_area_targets_at(
                    hit.time_ms,
                    f.source.source_side,
                    f.caster,
                    &tiles,
                    SkillHitTargetFilter::Enemies,
                    false,
                );
                let air = p.air;
                for target in targets {
                    if !self.single_target_can_target_unit(target, air) {
                        continue;
                    }
                    self.volley_hit(f, target, hit.time_ms)?;
                }
                return Ok(Some(VolleyFlightEnd::Impact));
            }
        }
        f.position = f.position + (desired - f.position) * fraction;
        f.traveled += step * fraction;
        f.last = until;
        Ok(terminal.or_else(|| {
            if f.traveled >= ratio(p.range) {
                Some(VolleyFlightEnd::Range)
            } else if until >= f.released + u64::from(p.lifetime_ms) {
                Some(VolleyFlightEnd::Lifetime)
            } else {
                None
            }
        }))
    }
    fn volley_hit(
        &mut self,
        f: &mut Flight,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        let p = f.program.clone();
        let count = f
            .counts
            .get(&target)
            .copied()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(GameError::InvalidAction)?;
        let mut bonus = MarkRatio {
            numerator: 0,
            denominator: 1,
        };
        for m in &p.modifiers {
            if let VolleyChange::Multihit { step, cap } = m.change {
                let numerator = step
                    .numerator
                    .checked_mul(u64::from(count))
                    .ok_or(GameError::InvalidAction)?;
                let value = MarkRatio {
                    numerator,
                    denominator: step.denominator,
                };
                let value = if u128::from(value.numerator) * u128::from(cap.denominator)
                    > u128::from(cap.numerator) * u128::from(value.denominator)
                {
                    cap
                } else {
                    value
                };
                bonus = super::super::marks::ratio_sum(bonus, value)?;
            }
        }
        let power = (u128::from(p.projectile.power)
            * (100 * u128::from(bonus.denominator) + u128::from(bonus.numerator)))
        .div_ceil(100 * u128::from(bonus.denominator));
        let mut source = controlled_clone(&f.source)?;
        source.base_damage = u32::try_from(power).map_err(|_| GameError::InvalidAction)?;
        source.committed_at_ms = now;
        source.crit_roll_percent = None;
        let source = self.materialize_damage_source_snapshot_for_target_hit(source, target, count);
        let prior = std::mem::replace(
            &mut self.runtime_fact_source,
            controlled_clone(&f.source.fact_source)?,
        );
        let result = self.execute_committed_damage(CommittedDamageInput {
            target_id: target,
            source_snapshot: source,
            committed_at_ms: now,
            policy: CommittedDamagePolicy::GeneralCommand,
        });
        self.runtime_fact_source = prior;
        control::poll("volley_damage_settled")?;
        if result.is_some_and(|r| r.accepted_hit_count > 0) {
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(256),
                "volley_hit_storage",
            )?;
            f.counts.insert(target, count);
            f.hits.push((target, now));
            for status in &p.projectile.after {
                self.volley_hit_status(f, target, status, None, now)?;
            }
            for modifier in &p.modifiers {
                if let VolleyChange::Status { status } = &modifier.change {
                    self.volley_hit_status(f, target, status, Some(modifier), now)?;
                }
            }
        }
        Ok(())
    }
    fn volley_hit_status(
        &mut self,
        f: &Flight,
        target: UnitInstanceId,
        status: &PlanStatus,
        modifier: Option<&VolleyModifier>,
        now: u64,
    ) -> Result<(), GameError> {
        if !self.units.get(&target).is_some_and(|u| u.is_active()) {
            return Ok(());
        }
        let count = status
            .count
            .evaluate(0, 0)
            .ok_or(GameError::InvalidAction)?;
        let kind = match status.kind {
            PlanStatusKind::Stun => HitStatusKind::Stun,
            PlanStatusKind::Root => HitStatusKind::Root,
            PlanStatusKind::BasicLock => HitStatusKind::BasicLock,
            PlanStatusKind::Vulnerability => HitStatusKind::StackDamageTaken {
                per_stack: status.percent,
                stacks: count,
            },
        };
        let modifier_source = modifier.and_then(|m| {
            f.applied.get(&m.id).map(|copy| HitModifierSource {
                copy_uuid: *copy,
                definition: m.relic_definition.clone(),
                rule: m.id.clone(),
            })
        });
        let prior = std::mem::replace(
            &mut self.runtime_fact_source,
            f.source.fact_source.independent_origin()?,
        );
        let result = self.begin_committed_hit_status(
            f.caster,
            target,
            f.source.source_side,
            status.duration_ms,
            HitStatusProfile {
                definition: controlled_clone(&status.definition)?,
                kind,
                modifier_source,
                vulnerability: None,
            },
            now,
        );
        self.runtime_fact_source = prior;
        result?;
        Ok(())
    }
    pub(in crate::game::battle::core) fn finish_volley_projectile(
        &mut self,
        f: Flight,
        now: u64,
        end: VolleyFlightEnd,
        resume: bool,
    ) -> Result<(), GameError> {
        self.close_source_resource_delivery(
            f.source.fact_source.resource_delivery().map(Arc::as_ref),
        )?;
        if let Some(ticket) = &f.source.fact_source.delivery {
            self.runtime_facts.close_delivery(ticket, now)?;
        }
        let key = f.copy;
        let activation = f.activation;
        let id = f.id;
        let caster = f.caster;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<VolleyFlightResult>() + 128),
            "volley_result_storage",
        )?;
        self.sustained.volleys.settled.insert(
            id,
            VolleyFlightResult {
                id,
                activation,
                copy: key,
                target: f.target,
                released_at_ms: f.released,
                ended_at_ms: now,
                end,
                hits: f.hits,
            },
        );
        if resume {
            if let Some(o) = self.sustained.volleys.owners.get_mut(&key) {
                if let Some(a) = o
                    .active
                    .as_mut()
                    .filter(|a| a.id == activation && a.flight == Some(id))
                {
                    a.flight = None;
                    a.hits = f.counts;
                    a.next_at = Some(
                        now.checked_add(
                            continuous(&o.program).map_or(0, |(_, _, ms)| u64::from(ms)),
                        )
                        .ok_or(GameError::InvalidAction)?,
                    );
                    self.schedule_prepared_volley(key)?;
                }
            }
        }
        self.reconcile_resonance_use(caster);
        control::poll("volley_impact_barrier")
    }
}
