use super::super::{
    damage_runtime::{CommittedDamageInput, CommittedDamagePolicy},
    projectile_math::{rect_entry, ProjectileCollisionCandidate, ProjectileMotionWindow},
};
use super::*;
use crate::game::{
    ability::{
        relation_action::{PlanStatus, PlanStatusKind},
        sustained::{
            HitModifierSource, HitStatusKind, HitStatusProfile, HitVulnerability,
            SustainedEffectDef,
        },
        SkillHitTargetFilter,
    },
    battle::{
        damage::{DamageModifiers, DamageSource, DamageType},
        tile_range::FacingDirection,
    },
};
impl BattleCore {
    fn shot_flight_end(
        &self,
        p: &ShotProjectileDef,
        start: WorldVec2,
        direction: WorldVec2,
    ) -> Result<(WorldVec2, u64, ShotEnd), GameError> {
        let speed = ratio_value(p.speed);
        let range = ratio_value(p.range);
        let lifetime = p.lifetime_ms as f32 / 1000.0;
        if !speed.is_finite() || !range.is_finite() || speed <= 0.0 || range <= 0.0 {
            return Err(GameError::InvalidAction);
        }
        let distance = range.min(speed * lifetime);
        let desired = start + direction * distance;
        let radius = ratio_value(p.width) * 0.5;
        let mut fraction = 1.0f32;
        let mut end = if range <= speed * lifetime {
            ShotEnd::Range
        } else {
            ShotEnd::Lifetime
        };
        let field = &self.scenario.battlefield;
        // Boundary exits are geometric even for a projectile that passes walls.
        for (s, d, lo, hi) in [
            (start.x, desired.x - start.x, 0.0, f32::from(field.width)),
            (start.y, desired.y - start.y, 0.0, f32::from(field.height)),
        ] {
            if d > 0.0 {
                let f = (hi - s) / d;
                if f < fraction {
                    fraction = f.max(0.0);
                    end = ShotEnd::Boundary;
                }
            } else if d < 0.0 {
                let f = (lo - s) / d;
                if f < fraction {
                    fraction = f.max(0.0);
                    end = ShotEnd::Boundary;
                }
            }
        }
        if p.obstacle_stop {
            control::charge(
                u64::from(field.width) * u64::from(field.height),
                "shot_obstacle_trace",
            )?;
            for x in 0..i32::from(field.width) {
                for y in 0..i32::from(field.height) {
                    let tile = Position::new(x, y);
                    if !self.projectile_tile_valid(tile) {
                        if let Some(f) = rect_entry(
                            start,
                            desired,
                            WorldVec2::new(x as f32 - radius, y as f32 - radius),
                            WorldVec2::new(x as f32 + 1.0 + radius, y as f32 + 1.0 + radius),
                        ) {
                            if f <= fraction {
                                fraction = f;
                                end = ShotEnd::Obstacle;
                            }
                        }
                    }
                }
            }
        }
        let duration = ((distance * fraction / speed) * 1000.0).ceil() as u64;
        Ok((
            start + (desired - start) * fraction,
            duration.min(u64::from(p.lifetime_ms)),
            end,
        ))
    }
    pub(in crate::game::battle::core) fn release_input_plan(
        &mut self,
        caster: UnitInstanceId,
        now: u64,
    ) -> Result<super::super::types::SkillStepResult, GameError> {
        let pending = controlled_clone(
            self.sustained
                .input_plans
                .pending
                .as_ref()
                .ok_or(GameError::InvalidAction)?,
        )?;
        if pending.preparation.caster != caster
            || pending.action.as_ref() != self.runtime_fact_source.action.as_ref()
        {
            return Err(GameError::InvalidAction);
        }
        let copy = pending.preparation.copy;
        let o = &self.sustained.input_plans.owners[&copy];
        let program = o.program.clone();
        let applied = Arc::new(controlled_clone(&o.applied)?);
        let mut template = self
            .damage_source_snapshot_template_for_unit(
                caster,
                DamageSource::Ability,
                DamageType::Physical,
                0,
                DamageModifiers::default(),
                0,
                now,
                false,
            )
            .ok_or(GameError::InvalidAction)?;
        template.fact_source = controlled_clone(&self.runtime_fact_source)?;
        let batch = self.next_shot_id()?;
        for (item, direction) in pending.layout.iter().zip(&pending.directions) {
            control::charge(1, "shot_volley_release")?;
            let p = match item.kind {
                ShotKind::Normal => &program.normal_projectile,
                ShotKind::Enhanced => &program.enhanced_projectile,
            };
            let start = WorldVec2::from_tile_center(item.tile);
            let (aim, duration, end) = self.shot_flight_end(p, start, *direction)?;
            let id = self.next_shot_id()?;
            let mut source = controlled_clone(&template)?;
            source.fact_source = self.reserve_finite_source(source.fact_source)?;
            source.base_damage = p.power;
            source.damage_type = p.damage_type;
            self.admit_authoring_runtime_growth(1, "shot_flight_entity")?;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(std::mem::size_of::<Flight>() + 128),
                "shot_flight_storage",
            )?;
            self.sustained.input_plans.flights.insert(
                id,
                Flight {
                    id,
                    batch,
                    copy,
                    caster,
                    kind: item.kind,
                    program: program.clone(),
                    applied: applied.clone(),
                    source,
                    start,
                    aim,
                    direction: *direction,
                    released: now,
                    duration,
                    last: None,
                    hits: Vec::new(),
                    actual_hp_loss: 0,
                    end,
                    hit_results: Vec::new(),
                },
            );
        }
        let o = self.sustained.input_plans.owners.get_mut(&copy).unwrap();
        o.normal = 0;
        o.enhanced = 0;
        o.memory = pending.layout;
        control::poll("shot_volley_cost_and_memory")?;
        self.admit_authoring_runtime_growth(1, "shot_receipt_entity")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<ShotCommitReceipt>() + 128),
            "shot_receipt_storage",
        )?;
        self.sustained.input_plans.receipts.insert(
            pending.preparation.id,
            ShotCommitReceipt {
                copy,
                preparation: pending.preparation.id,
                request: pending.request,
                batch,
                normal: pending.preparation.normal,
                enhanced: pending.preparation.enhanced,
                released_at_ms: now,
            },
        );
        if self.sustained.input_plans.next_flight_tick.is_none() {
            self.sustained.input_plans.next_flight_tick = Some(now);
            self.event_queue
                .push(BattleEvent::ShotFlightTick { time_ms: now });
        }
        Ok(super::super::types::SkillStepResult {
            applied_effect_count: 1,
            ..Default::default()
        })
    }
    pub(in crate::game::battle::core) fn shot_flight_tick(
        &mut self,
        now: u64,
    ) -> Result<(), GameError> {
        if self.sustained.input_plans.next_flight_tick != Some(now) {
            return Ok(());
        }
        self.sustained.input_plans.next_flight_tick = None;
        let ids: Vec<_> = self.sustained.input_plans.flights.keys().copied().collect();
        for id in ids {
            let Some(mut f) = self.sustained.input_plans.flights.remove(&id) else {
                continue;
            };
            self.sustained.input_plans.processing = Some(controlled_clone(&f.source.fact_source)?);
            let profile = match f.kind {
                ShotKind::Normal => &f.program.normal_projectile,
                ShotKind::Enhanced => &f.program.enhanced_projectile,
            };
            let from = f.last.unwrap_or(f.released);
            let until = now.min(f.released + f.duration);
            let window = ProjectileMotionWindow::fixed_trajectory(
                from,
                until,
                f.start,
                f.aim,
                from - f.released,
                until - f.released,
                f.duration,
            );
            control::charge(
                (self.units.len() + f.hits.len() + 1) as u64,
                "shot_collision_candidates",
            )?;
            let candidates = self
                .units
                .values()
                .filter(|u| {
                    !f.hits.contains(&u.instance_id)
                        && self.skill_delivery_accepts_unit(
                            f.source.source_side,
                            f.caster,
                            u.instance_id,
                            SkillHitTargetFilter::Enemies,
                            false,
                        )
                        && self.single_target_can_target_unit(u.instance_id, profile.air)
                })
                .filter_map(|u| {
                    let a = self.sample_unit_body_at(u.instance_id, from)?;
                    let b = self.sample_unit_body_at(u.instance_id, until)?;
                    Some(ProjectileCollisionCandidate {
                        id: u.instance_id,
                        target_start: a.position,
                        target_end: b.position,
                        reach: ratio_value(profile.width) * 0.5 + a.radius.max(b.radius).max(0.0),
                    })
                })
                .collect::<Vec<_>>();
            let impacts = window.resolve(candidates).ordered_impacts;
            let limit = profile.max_hits;
            let mut end = None;
            for hit in impacts {
                if matches!(f.end, ShotEnd::Obstacle | ShotEnd::Boundary)
                    && hit.time_ms >= f.released + f.duration
                {
                    break;
                }
                crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    (f.hits.len() + 1).checked_mul(
                        std::mem::size_of::<UnitInstanceId>() * 2
                            + std::mem::size_of::<ShotHitResult>() * 2,
                    ),
                    "shot_hit_storage",
                )?;
                f.hits.push(hit.id);
                let continues = limit.is_none_or(|n| f.hits.len() < n as usize)
                    && hit.time_ms < f.released + f.duration;
                let center = self
                    .sample_unit_body_at(hit.id, hit.time_ms)
                    .ok_or(GameError::InvalidAction)?
                    .position;
                self.shot_hit(&mut f, hit.id, center, hit.time_ms, continues)?;
                if !continues {
                    end = Some(if limit.is_some_and(|n| f.hits.len() >= n as usize) {
                        ShotEnd::HitLimit
                    } else {
                        f.end
                    });
                    break;
                }
            }
            f.last = Some(until);
            self.sustained.input_plans.processing = None;
            if let Some(end) = end.or((until >= f.released + f.duration).then_some(f.end)) {
                self.finish_shot_flight(f, now, end)?;
            } else {
                self.sustained.input_plans.flights.insert(id, f);
            }
        }
        if !self.sustained.input_plans.flights.is_empty() {
            let next = now.checked_add(1).ok_or(GameError::InvalidAction)?;
            self.sustained.input_plans.next_flight_tick = Some(next);
            self.event_queue
                .push(BattleEvent::ShotFlightTick { time_ms: next });
        }
        Ok(())
    }
    fn finish_shot_flight(&mut self, f: Flight, now: u64, end: ShotEnd) -> Result<(), GameError> {
        let source = &f.source.fact_source;
        self.close_source_resource_delivery(source.resource_delivery().map(Arc::as_ref))?;
        if let Some(ticket) = &source.delivery {
            self.runtime_facts.close_delivery(ticket, now)?;
        }
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<ShotFlightResult>() + 128),
            "shot_settlement_storage",
        )?;
        self.sustained.input_plans.settled.insert(
            f.id,
            ShotFlightResult {
                id: f.id,
                batch: f.batch,
                copy: f.copy,
                kind: f.kind,
                ended_at_ms: now,
                end,
                hits: f.hit_results,
            },
        );
        self.reconcile_resonance_use(f.caster);
        control::poll("shot_flight_settled")
    }
    fn shot_hit(
        &mut self,
        f: &mut Flight,
        target: UnitInstanceId,
        position: WorldVec2,
        now: u64,
        continues: bool,
    ) -> Result<(), GameError> {
        let program = f.program.clone();
        let p = match f.kind {
            ShotKind::Normal => &program.normal_projectile,
            ShotKind::Enhanced => &program.enhanced_projectile,
        };
        let mut bonus = 0u128;
        let mut denominator = 1u128;
        for m in &program.modifiers {
            if let ShotChange::Wound {
                profiles, percent, ..
            } = &m.change
            {
                if profiles.contains(&p.definition) {
                    let n = self
                        .sustained
                        .input_plans
                        .wounds
                        .get(&(f.copy, m.id.clone(), target))
                        .filter(|w| now < w.expires)
                        .map_or(0, |w| w.count);
                    bonus = bonus
                        .checked_mul(u128::from(percent.denominator))
                        .and_then(|v| {
                            v.checked_add(
                                u128::from(n) * u128::from(percent.numerator) * denominator,
                            )
                        })
                        .ok_or(GameError::InvalidAction)?;
                    denominator = denominator
                        .checked_mul(u128::from(percent.denominator))
                        .ok_or(GameError::InvalidAction)?;
                }
            }
        }
        let base = (u128::from(p.power) * (100 * denominator + bonus)) / (100 * denominator);
        let mut snapshot = controlled_clone(&f.source)?;
        snapshot.base_damage = u32::try_from(base).map_err(|_| GameError::InvalidAction)?;
        snapshot.committed_at_ms = now;
        snapshot.crit_roll_percent = None;
        let snapshot = self.materialize_damage_source_snapshot_for_target_hit(
            snapshot,
            target,
            u32::try_from(f.hits.len()).map_err(|_| GameError::InvalidAction)?,
        );
        let prior = std::mem::replace(
            &mut self.runtime_fact_source,
            controlled_clone(&f.source.fact_source)?,
        );
        let summary = self.execute_damage_with_observation_origin(
            CommittedDamageInput {
                target_id: target,
                source_snapshot: snapshot,
                committed_at_ms: now,
                policy: CommittedDamagePolicy::GeneralCommand,
            },
            false,
            Some(&f.source.fact_source),
        );
        self.runtime_fact_source = prior;
        control::poll("shot_direct_damage")?;
        let effective = summary.as_ref().is_some_and(|s| s.accepted_hit_count > 0);
        let loss = summary.map_or(0, |s| s.actual_hp_loss);
        f.actual_hp_loss = f
            .actual_hp_loss
            .checked_add(u64::from(loss))
            .ok_or(GameError::InvalidAction)?;
        f.hit_results.push(ShotHitResult {
            statuses: Vec::new(),
            target,
            time_ms: now,
            actual_hp_loss: loss,
            effective_hit: effective,
            killed: self
                .units
                .get(&target)
                .is_some_and(|u| u.lifecycle.is_dead()),
        });
        if effective {
            for s in &p.after {
                let applied = self.shot_apply_status(f, target, s, None, now)?;
                self.record_shot_status(f, s, applied)?;
            }
            for m in &program.modifiers {
                if let ShotChange::Push { profile, tiles } = &m.change {
                    if profile == &p.definition {
                        self.push_shot_target(
                            f.caster,
                            target,
                            f.direction,
                            ratio_value(*tiles),
                            now,
                        )?;
                    }
                }
            }
            for m in &program.modifiers {
                if let ShotChange::Status { profile, status } = &m.change {
                    if profile == &p.definition {
                        let applied = self.shot_apply_status(f, target, status, Some(m), now)?;
                        self.record_shot_status(f, status, applied)?;
                    }
                }
            }
            if self.units.get(&target).is_some_and(|u| u.is_active()) {
                for m in &program.modifiers {
                    if let ShotChange::Wound {
                        profiles,
                        amount,
                        cap,
                        duration_ms,
                        ..
                    } = &m.change
                    {
                        if profiles.contains(&p.definition) {
                            let mut gain = *amount;
                            for g in &program.modifiers {
                                if let ShotChange::WoundGrant { definition, factor } = &g.change {
                                    if definition == &m.id {
                                        gain = scale_floor(gain, *factor)?;
                                    }
                                }
                            }
                            let key = (f.copy, m.id.clone(), target);
                            let prior = self
                                .sustained
                                .input_plans
                                .wounds
                                .get(&key)
                                .filter(|w| now < w.expires)
                                .map_or(0, |w| w.count);
                            if !self.sustained.input_plans.wounds.contains_key(&key) {
                                self.admit_authoring_runtime_growth(1, "shot_wound_entity")?;
                                crate::game::battle::runtime_control::admit_retained_growth(
                                    0,
                                    Some(m.id.len() + std::mem::size_of::<Wound>() + 192),
                                    "shot_wound_storage",
                                )?;
                            }
                            self.sustained.input_plans.wounds.insert(
                                key,
                                Wound {
                                    count: prior.saturating_add(gain).min(*cap),
                                    expires: now
                                        .checked_add(u64::from(*duration_ms))
                                        .ok_or(GameError::InvalidAction)?,
                                },
                            );
                        }
                    }
                }
            }
            if continues {
                for m in &program.modifiers {
                    if let ShotChange::Explosion {
                        profile,
                        power,
                        damage_type,
                        range,
                        air,
                        bonus,
                    } = &m.change
                    {
                        if profile != &p.definition {
                            continue;
                        }
                        let tiles = range
                            .affected_tiles(position.project_to_tile(), FacingDirection::Up)
                            .map_err(GameError::InvalidStaticData)?;
                        let mut targets = self.collect_tile_area_targets_at(
                            now,
                            f.source.source_side,
                            f.caster,
                            &tiles,
                            SkillHitTargetFilter::Enemies,
                            false,
                        );
                        targets.retain(|target| self.single_target_can_target_unit(*target, *air));
                        let area_percent = self.area_damage_percent(
                            f.caster,
                            f.source.source_side,
                            &f.source.fact_source,
                            &targets,
                            now,
                        )?;
                        for target in targets {
                            if !self.single_target_can_target_unit(target, *air) {
                                continue;
                            }
                            let mut snapshot = controlled_clone(&f.source)?;
                            snapshot.modifiers.damage_amp_percent = snapshot
                                .modifiers
                                .damage_amp_percent
                                .saturating_add(area_percent);
                            snapshot.base_damage = u32::try_from(
                                (u128::from(*power)
                                    * (100 * u128::from(bonus.denominator)
                                        + u128::from(bonus.numerator)))
                                    / (100 * u128::from(bonus.denominator)),
                            )
                            .map_err(|_| GameError::InvalidAction)?;
                            snapshot.damage_type = *damage_type;
                            snapshot.source = DamageSource::Ability;
                            snapshot.fact_source = self.derived_shot_source(f, m, target)?;
                            snapshot.crit_roll_percent = None;
                            snapshot.committed_at_ms = now;
                            let snapshot = self.materialize_damage_source_snapshot_for_target_hit(
                                snapshot, target, 0,
                            );
                            let prior = std::mem::replace(
                                &mut self.runtime_fact_source,
                                controlled_clone(&snapshot.fact_source)?,
                            );
                            self.execute_committed_damage(CommittedDamageInput {
                                target_id: target,
                                source_snapshot: snapshot,
                                committed_at_ms: now,
                                policy: CommittedDamagePolicy::GeneralCommand,
                            });
                            self.runtime_fact_source = prior;
                            control::poll("shot_piercing_explosion")?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn derived_shot_source(
        &mut self,
        f: &Flight,
        m: &ShotModifier,
        target: UnitInstanceId,
    ) -> Result<RuntimeFactSource, GameError> {
        use super::super::relic_causal::{
            CausalCauseObservation, CausalFactObservation, CausalSourceData,
        };
        let parent = &f.source.fact_source;
        let observation = Arc::new(CausalFactObservation {
            actor: parent.actor,
            equipment: parent.equipment,
            fragment: parent.fragment.clone(),
            cast: parent.cast.clone(),
            authored: parent.authored.clone(),
        });
        let mut source = parent.independent_origin()?;
        source.fragment = None;
        source.equipment = None;
        source.cast = None;
        source.authored = None;
        source = self.causal_root_source(&source, true)?;
        let context = source
            .causal()
            .and_then(|c| c.context.clone())
            .ok_or(GameError::InvalidAction)?;
        let cause = Arc::new(CausalCauseObservation {
            root: context.root,
            owner: f.caster,
            root_source: observation.clone(),
            triggering_source: observation,
            actual_killer: None,
            event_target: Some(target),
            copy_uuid: *f.applied.get(&m.id).ok_or(GameError::InvalidAction)?,
            definition_id: m.relic_definition.clone(),
            variant: crate::game::relic::RelicVariant::Normal,
            rule_id: m.id.clone(),
            event: None,
        });
        source.with_causal(Some(Arc::new(CausalSourceData {
            context: Some(context),
            program: None,
            amount: None,
            cause: Some(cause),
        })))
    }
    pub(in crate::game::battle::core) fn push_shot_target(
        &mut self,
        caster: UnitInstanceId,
        target: UnitInstanceId,
        direction: WorldVec2,
        distance: f32,
        now: u64,
    ) -> Result<(), GameError> {
        use crate::game::battle::{
            core::movement::ActionState,
            damage::BattleCommand,
            event_log::{BattleLogEvent, MovementStopReason},
        };
        let Some(u) = self.units.get(&target).filter(|u| u.is_active()) else {
            return Ok(());
        };
        if u.body.radius > 0.5 || self.sustained_invulnerable(target, now)? {
            return Ok(());
        }
        let from = self
            .sample_unit_body_at(target, now)
            .ok_or(GameError::InvalidAction)?
            .position;
        let radius = u.body.radius;
        let mut to = from;
        let steps = (distance / 0.25).ceil() as u32;
        let can_move = |start: WorldVec2, candidate: WorldVec2| -> Result<bool, GameError> {
            control::charge(self.units.len() as u64 + 1, "shot_push_trace")?;
            let min = WorldVec2::new(candidate.x - radius, candidate.y - radius).project_to_tile();
            let max = WorldVec2::new(candidate.x + radius, candidate.y + radius).project_to_tile();
            if (min.x..=max.x).any(|x| {
                (min.y..=max.y).any(|y| !self.battlefield.is_walkable_tile(Position::new(x, y)))
            }) {
                return Ok(false);
            }
            if self
                .units
                .values()
                .filter(|other| other.instance_id != target && other.is_active())
                .any(|other| {
                    let position = self
                        .sample_unit_body_at(other.instance_id, now)
                        .map_or(other.body.position, |body| body.position);
                    !ProjectileMotionWindow::fixed_trajectory(0, 1, start, candidate, 0, 1, 1)
                        .resolve([ProjectileCollisionCandidate {
                            id: other.instance_id,
                            target_start: position,
                            target_end: position,
                            reach: radius + other.body.radius,
                        }])
                        .ordered_impacts
                        .is_empty()
                })
            {
                return Ok(false);
            }
            Ok(true)
        };
        for step in 1..=steps {
            let candidate = from + direction * (step as f32 * 0.25).min(distance);
            if !can_move(to, candidate)? {
                // Preserve the reachable part of a blocked segment, including
                // pushes shorter than the coarse sweep step. Reuse the same
                // terrain/body predicate and bound refinement to data precision.
                let mut low = to;
                let mut high = candidate;
                let precision = 1.0 / super::super::movement::types::DATA_UNITS_PER_WORLD;
                for _ in 0..20 {
                    if low.distance_squared(high) <= precision * precision {
                        break;
                    }
                    let middle = low + (high - low) * 0.5;
                    if middle == low || middle == high {
                        break;
                    }
                    if can_move(to, middle)? {
                        low = middle;
                    } else {
                        high = middle;
                    }
                }
                to = low;
                break;
            }
            to = candidate;
        }
        if to == from {
            return Ok(());
        }
        self.process_commands_with_modifier_source(
            vec![BattleCommand::InterruptCast {
                source_id: caster,
                target_id: target,
            }],
            now,
            Some(caster),
        );
        control::poll("shot_push_interrupt")?;
        self.release_tactical_engagements(&[target]);
        self.active_movement_segments.remove(&target);
        self.retarget_controlled_basic(target, now);
        let unit = self.units.get_mut(&target).unwrap();
        unit.body.position = to;
        unit.move_epoch = unit.move_epoch.wrapping_add(1);
        unit.action_state = ActionState::Idle;
        self.reconcile_sustained(now)?;
        self.refresh_block_state();
        self.record_event_log(
            now,
            BattleLogEvent::UnitDisplaced {
                source_instance_id: caster,
                unit_instance_id: target,
                from_position: from.project_to_tile(),
                to_position: to.project_to_tile(),
                from_world_position: from.quantized_milli(),
                to_world_position: to.quantized_milli(),
            },
        );
        self.record_movement_stopped(now, target, MovementStopReason::Displaced, None);
        Ok(())
    }
    fn record_shot_status(
        &self,
        f: &mut Flight,
        status: &PlanStatus,
        applied: bool,
    ) -> Result<(), GameError> {
        let result = f.hit_results.last_mut().ok_or(GameError::InvalidAction)?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(status.definition.len() + 2 * std::mem::size_of::<ShotStatusApplication>()),
            "shot_status_result",
        )?;
        result.statuses.push(ShotStatusApplication {
            definition: controlled_clone(&status.definition)?,
            applied,
        });
        Ok(())
    }
    fn shot_apply_status(
        &mut self,
        f: &Flight,
        target: UnitInstanceId,
        status: &PlanStatus,
        modifier: Option<&ShotModifier>,
        now: u64,
    ) -> Result<bool, GameError> {
        if !self.units.get(&target).is_some_and(|u| u.is_active()) {
            return Ok(false);
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
            f.applied.get(&m.id).map(|copy_uuid| HitModifierSource {
                copy_uuid: *copy_uuid,
                definition: m.relic_definition.clone(),
                rule: m.id.clone(),
            })
        });
        let vulnerability = f.program.modifiers.iter().find_map(|m| match &m.change {
            ShotChange::ControlVulnerability {
                definition,
                percent,
                duration_ms,
            } if definition == &status.definition => Some(HitVulnerability {
                definition: m.id.clone(),
                percent: *percent,
                duration_ms: *duration_ms,
                modifier_source: f.applied.get(&m.id).map(|copy_uuid| HitModifierSource {
                    copy_uuid: *copy_uuid,
                    definition: m.relic_definition.clone(),
                    rule: m.id.clone(),
                }),
            }),
            _ => None,
        });
        let source = f.source.fact_source.independent_origin()?;
        let prior = std::mem::replace(&mut self.runtime_fact_source, source);
        let result = self.begin_committed_hit_status(
            f.caster,
            target,
            f.source.source_side,
            status.duration_ms,
            HitStatusProfile {
                definition: status.definition.clone(),
                kind,
                modifier_source,
                vulnerability,
            },
            now,
        );
        self.runtime_fact_source = prior;
        result
    }
    pub(in crate::game::battle::core) fn clear_input_plans(
        &mut self,
        now: u64,
    ) -> Result<(), GameError> {
        let ids: Vec<_> = self.sustained.input_plans.flights.keys().copied().collect();
        for id in ids {
            let f = self.sustained.input_plans.flights.remove(&id).unwrap();
            self.finish_shot_flight(f, now, ShotEnd::BattleEnded)?;
        }
        self.sustained.input_plans.owners.clear();
        self.sustained.input_plans.preparation = None;
        self.sustained.input_plans.pending = None;
        self.sustained.input_plans.disconnected = false;
        self.sustained.input_plans.wounds.clear();
        self.sustained.input_plans.next_flight_tick = None;
        self.event_queue.discard_input_plan_events()?;
        Ok(())
    }
}
