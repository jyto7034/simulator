use uuid::Uuid;

use crate::game::{
    battle::{
        core::{
            damage_runtime::{CommittedDamageInput, CommittedDamagePolicy},
            movement::types::WorldVec2,
            projectile_math::{
                projectile_flight_ms_between_world_points, ProjectileCollisionCandidate,
                ProjectileMotionWindow,
            },
            types::{ProjectileGuidance, ProjectileRecord},
            BattleCore,
        },
        damage::{DamageSourceSnapshot, DamageType},
        enums::BattleEvent,
        event_log::BattleLogEvent,
        ids::UnitInstanceId,
    },
    determinism,
    enums::Side,
};

#[derive(Debug, Clone)]
pub(super) struct ProjectileLaunch {
    pub(super) spatial: Option<super::types::BasicSpatialProjectile>,
    pub(super) fired_at_ms: u64,
    pub(super) attacker_instance_id: UnitInstanceId,
    pub(super) attacker_owner_at_launch: Side,
    pub(super) air_capable_at_launch: bool,
    pub(super) target_instance_id: UnitInstanceId,
    pub(super) attacker_origin: WorldVec2,
    pub(super) target_aim: WorldVec2,
    pub(super) speed_units_per_ms: u32,
    pub(super) guidance: ProjectileGuidance,
    pub(super) damage_type: DamageType,
    pub(super) source_snapshot: DamageSourceSnapshot,
}

impl BattleCore {
    pub(super) fn spawn_basic_attack_projectile(&mut self, mut launch: ProjectileLaunch) {
        // TODO: 추후 config 로 빼야함.
        const PROJECTILE_NS: u64 = 0x5052_4F4A_4543_544Cu64; // "PROJECTL"

        if let Some(spatial) = &launch.spatial {
            let delta = launch.target_aim - launch.attacker_origin;
            let delta = if delta.length() <= f32::EPSILON {
                use crate::game::battle::tile_range::FacingDirection;
                match self
                    .units
                    .get(&launch.attacker_instance_id)
                    .and_then(|u| u.facing_direction)
                    .unwrap_or(if launch.attacker_owner_at_launch == Side::Player {
                        FacingDirection::Right
                    } else {
                        FacingDirection::Left
                    }) {
                    FacingDirection::Up => WorldVec2 { x: 0.0, y: -1.0 },
                    FacingDirection::Right => WorldVec2 { x: 1.0, y: 0.0 },
                    FacingDirection::Down => WorldVec2 { x: 0.0, y: 1.0 },
                    FacingDirection::Left => WorldVec2 { x: -1.0, y: 0.0 },
                }
            } else {
                delta
            };
            launch.target_aim =
                launch.attacker_origin + delta * (spatial.range_tiles as f32 / delta.length());
        }
        let flight_ms = projectile_flight_ms_between_world_points(
            launch.attacker_origin,
            launch.target_aim,
            launch.speed_units_per_ms,
        );
        let flight_ms = launch
            .spatial
            .as_ref()
            .map_or(flight_ms, |s| flight_ms.min(s.lifetime_ms));
        let impact_ms = launch.fired_at_ms.saturating_add(flight_ms);

        let seed = determinism::seed_with_uuid(
            self.seed,
            PROJECTILE_NS,
            launch.source_snapshot.crit_roll_identity.source_instance_id,
        );
        let projectile_seq = self.projectile_seq;
        self.projectile_seq = self.projectile_seq.wrapping_add(1);
        let projectile_id = determinism::uuid_v4_from_seed(seed, PROJECTILE_NS, projectile_seq);

        if launch.source_snapshot.admit_fact_owner().is_err() {
            return;
        }
        self.projectiles.insert(
            projectile_id,
            ProjectileRecord {
                spatial: launch.spatial,
                fired_at_ms: launch.fired_at_ms,
                last_reevaluation_ms: launch.fired_at_ms,
                attacker_instance_id: launch.attacker_instance_id,
                attacker_owner_at_launch: launch.attacker_owner_at_launch,
                air_capable_at_launch: launch.air_capable_at_launch,
                target_instance_id: launch.target_instance_id,
                start: launch.attacker_origin,
                current_position: launch.attacker_origin,
                aim: launch.target_aim,
                speed_units_per_ms: launch.speed_units_per_ms,
                guidance: launch.guidance,
                damage_type: launch.damage_type,
                source_snapshot: launch.source_snapshot,
                max_travel_ms: flight_ms,
                cause: self.recording_cause().unwrap_or_default(),
            },
        );
        #[cfg(feature = "runtime-observation")]
        self.observe_projectile_peak();

        self.record_event_log(
            launch.fired_at_ms,
            BattleLogEvent::BasicAttackProjectileLaunched {
                projectile_id,
                attacker_instance_id: launch.attacker_instance_id,
                target_instance_id: launch.target_instance_id,
                start: launch.attacker_origin.quantized_milli(),
                aim: launch.target_aim.quantized_milli(),
                fired_at_ms: launch.fired_at_ms,
                expected_impact_time_ms: impact_ms,
            },
        );

        self.event_queue
            .push(BattleEvent::BasicAttackProjectileAdvance {
                time_ms: launch.fired_at_ms,
                projectile_id,
                cause: self.recording_cause().unwrap_or_default(),
            });
    }

    pub(super) fn advance_basic_attack_projectile(&mut self, time_ms: u64, projectile_id: Uuid) {
        let Some(source) = self
            .projectiles
            .get(&projectile_id)
            .map(|projectile| projectile.source_snapshot.fact_source.clone())
        else {
            return;
        };
        let ticket = source.delivery.clone();
        let prior = std::mem::replace(&mut self.runtime_fact_source, source);
        self.advance_basic_attack_projectile_inner(time_ms, projectile_id);
        self.runtime_fact_source = prior;
        if !self.projectiles.contains_key(&projectile_id) {
            if let Some(ticket) = ticket {
                let _ = self.runtime_facts.close_delivery(&ticket, time_ms);
            }
        }
    }

    fn advance_basic_attack_projectile_inner(&mut self, time_ms: u64, projectile_id: Uuid) {
        let Some(mut projectile) = self.projectiles.remove(&projectile_id) else {
            return;
        };

        let reevaluation_time_ms = time_ms
            .min(
                projectile
                    .fired_at_ms
                    .saturating_add(projectile.max_travel_ms),
            )
            .max(projectile.fired_at_ms);

        if reevaluation_time_ms <= projectile.last_reevaluation_ms
            && projectile.max_travel_ms > 0
            && time_ms > projectile.fired_at_ms
        {
            self.projectiles.insert(projectile_id, projectile);
            #[cfg(feature = "runtime-observation")]
            self.observe_projectile_peak();
            return;
        }

        if projectile.spatial.is_some() {
            self.advance_spatial_basic_projectile(reevaluation_time_ms, projectile_id, projectile);
            return;
        }
        let Some(target) = self.units.get(&projectile.target_instance_id) else {
            self.record_basic_attack_projectile_miss(
                reevaluation_time_ms,
                projectile_id,
                projectile.attacker_instance_id,
                projectile.target_instance_id,
                projectile.damage_type,
                projectile.current_position,
            );
            return;
        };

        if !target.is_active()
            || target.owner == projectile.attacker_owner_at_launch
            || !self.single_target_can_target_unit(
                projectile.target_instance_id,
                projectile.air_capable_at_launch,
            )
        {
            self.record_basic_attack_projectile_miss(
                reevaluation_time_ms,
                projectile_id,
                projectile.attacker_instance_id,
                projectile.target_instance_id,
                projectile.damage_type,
                projectile.current_position,
            );
            return;
        }

        let from_time_ms = projectile.last_reevaluation_ms;
        let Some(target_body_start) =
            self.sample_unit_body_at(projectile.target_instance_id, from_time_ms)
        else {
            self.record_basic_attack_projectile_miss(
                reevaluation_time_ms,
                projectile_id,
                projectile.attacker_instance_id,
                projectile.target_instance_id,
                projectile.damage_type,
                projectile.current_position,
            );
            return;
        };
        let Some(target_body_end) =
            self.sample_unit_body_at(projectile.target_instance_id, reevaluation_time_ms)
        else {
            self.record_basic_attack_projectile_miss(
                reevaluation_time_ms,
                projectile_id,
                projectile.attacker_instance_id,
                projectile.target_instance_id,
                projectile.damage_type,
                projectile.current_position,
            );
            return;
        };

        let aim = match projectile.guidance {
            ProjectileGuidance::Homing => target_body_end.position,
            ProjectileGuidance::Fixed => projectile.aim,
        };
        let reach = target_body_start
            .radius
            .max(target_body_end.radius)
            .max(0.0);

        let motion = ProjectileMotionWindow::speed_limited(
            from_time_ms,
            reevaluation_time_ms,
            projectile.current_position,
            aim,
            projectile.speed_units_per_ms,
        )
        .resolve([ProjectileCollisionCandidate {
            id: projectile.target_instance_id,
            target_start: target_body_start.position,
            target_end: target_body_end.position,
            reach,
        }]);
        let next_position = motion.end_position;
        let next_reevaluation_ms = motion.next_reevaluation_ms;

        if let Some(impact) = motion.ordered_impacts.into_iter().next() {
            let Ok(snapshot) =
                crate::authoring::clone_shape::controlled_clone(&projectile.source_snapshot)
            else {
                return;
            };
            let actual = self.apply_basic_attack_projectile_hit_at(
                impact.time_ms,
                projectile_id,
                projectile.attacker_instance_id,
                impact.id,
                snapshot,
                impact.position,
            );
            let _ =
                self.finish_basic_lifesteal(&projectile.source_snapshot, actual, impact.time_ms);
            return;
        }

        projectile.current_position = next_position;
        projectile.aim = aim;
        projectile.last_reevaluation_ms = reevaluation_time_ms;

        let expired = reevaluation_time_ms
            >= projectile
                .fired_at_ms
                .saturating_add(projectile.max_travel_ms);
        if expired {
            self.record_basic_attack_projectile_miss(
                reevaluation_time_ms,
                projectile_id,
                projectile.attacker_instance_id,
                projectile.target_instance_id,
                projectile.damage_type,
                projectile.current_position,
            );
            return;
        }

        self.event_queue
            .push(BattleEvent::BasicAttackProjectileAdvance {
                time_ms: next_reevaluation_ms,
                projectile_id,
                cause: projectile.cause.clone(),
            });
        self.projectiles.insert(projectile_id, projectile);
        #[cfg(feature = "runtime-observation")]
        self.observe_projectile_peak();
    }

    fn record_basic_attack_projectile_miss(
        &mut self,
        time_ms: u64,
        projectile_id: Uuid,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        _damage_type: DamageType,
        impact_position: WorldVec2,
    ) {
        self.record_event_log(
            time_ms,
            BattleLogEvent::BasicAttackProjectileImpacted {
                projectile_id,
                attacker_instance_id,
                target_instance_id,
                impact_position: impact_position.quantized_milli(),
                hit: false,
            },
        );
    }

    fn apply_basic_attack_projectile_hit_at(
        &mut self,
        time_ms: u64,
        projectile_id: Uuid,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        source_snapshot: DamageSourceSnapshot,
        impact_position: WorldVec2,
    ) -> u64 {
        {
            let Some(target) = self.units.get(&target_instance_id) else {
                self.record_basic_attack_projectile_miss(
                    time_ms,
                    projectile_id,
                    attacker_instance_id,
                    target_instance_id,
                    source_snapshot.damage_type,
                    impact_position,
                );
                return 0;
            };
            if !target.is_active() {
                self.record_basic_attack_projectile_miss(
                    time_ms,
                    projectile_id,
                    attacker_instance_id,
                    target_instance_id,
                    source_snapshot.damage_type,
                    impact_position,
                );
                return 0;
            }
        }

        self.record_event_log(
            time_ms,
            BattleLogEvent::BasicAttackProjectileImpacted {
                projectile_id,
                attacker_instance_id,
                target_instance_id,
                impact_position: impact_position.quantized_milli(),
                hit: true,
            },
        );

        let summary = self.execute_committed_damage(CommittedDamageInput {
            target_id: target_instance_id,
            source_snapshot,
            committed_at_ms: time_ms,
            policy: CommittedDamagePolicy::BasicAttack {
                occurrence_id: projectile_id,
                occurrence_time_ms: time_ms,
            },
        });

        self.schedule_pending_autocasts(time_ms);
        summary.map_or(0, |s| s.actual_hp_loss)
    }
}

impl BattleCore {
    fn advance_spatial_basic_projectile(
        &mut self,
        now: u64,
        id: Uuid,
        mut projectile: ProjectileRecord,
    ) {
        use crate::authoring::{clone_shape::controlled_clone, control};
        let spatial = projectile.spatial.as_ref().unwrap();
        if control::charge(
            (self.units.len() as u64).saturating_mul(spatial.hit_unit_ids.len() as u64 + 8),
            "basic_spatial_scan",
        )
        .is_err()
            || crate::game::battle::runtime_control::admit_retained_growth(
                0,
                self.units.len().checked_mul(
                    std::mem::size_of::<ProjectileCollisionCandidate<UnitInstanceId>>() + 96,
                ),
                "basic_spatial_candidates",
            )
            .is_err()
        {
            return;
        }
        let from = projectile.last_reevaluation_ms;
        let candidates: Vec<_> = self
            .units
            .values()
            .filter_map(|unit| {
                if !unit.is_active()
                    || unit.owner == projectile.attacker_owner_at_launch
                    || spatial.hit_unit_ids.contains(&unit.instance_id)
                    || !self.single_target_can_target_unit(
                        unit.instance_id,
                        projectile.air_capable_at_launch,
                    )
                {
                    return None;
                }
                let start = self.sample_unit_body_at(unit.instance_id, from)?;
                let end = self.sample_unit_body_at(unit.instance_id, now)?;
                Some(ProjectileCollisionCandidate {
                    id: unit.instance_id,
                    target_start: start.position,
                    target_end: end.position,
                    reach: start.radius.max(end.radius).max(0.0)
                        + spatial.collision.radius_units as f32
                            / super::movement::types::DATA_UNITS_PER_WORLD,
                })
            })
            .collect();
        let motion = ProjectileMotionWindow::speed_limited(
            from,
            now,
            projectile.current_position,
            projectile.aim,
            projectile.speed_units_per_ms,
        )
        .resolve(candidates);
        for impact in motion.ordered_impacts {
            if !self.units.get(&impact.id).is_some_and(|u| u.is_active()) {
                continue;
            }
            let spatial = projectile.spatial.as_mut().unwrap();
            if self
                .admit_authoring_runtime_growth(1, "basic_spatial_hit")
                .is_err()
                || crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    spatial
                        .hit_unit_ids
                        .len()
                        .checked_add(1)
                        .and_then(|n| n.checked_mul(std::mem::size_of::<UnitInstanceId>() * 2)),
                    "basic_spatial_hit_storage",
                )
                .is_err()
            {
                return;
            }
            spatial.hit_unit_ids.push(impact.id);
            let Ok(snapshot) = controlled_clone(&projectile.source_snapshot) else {
                return;
            };
            let actual = self.apply_basic_attack_projectile_hit_at(
                impact.time_ms,
                id,
                projectile.attacker_instance_id,
                impact.id,
                snapshot,
                impact.position,
            );
            if Self::add_basic_hp_loss(&mut spatial.actual_hp_loss, actual).is_err() {
                return;
            }
            if !self.units.get(&impact.id).is_some_and(|u| u.is_active()) {
                spatial.kills = spatial.kills.saturating_add(1);
            }
            let piercing =
                spatial.collision.piercing || spatial.collision.despawn_on_hit == Some(false);
            let hit_count = spatial.hit_unit_ids.len() as u64;
            if !piercing
                || spatial
                    .collision
                    .max_hits
                    .is_some_and(|limit| hit_count >= u64::from(limit))
                || spatial
                    .max_pierces
                    .is_some_and(|limit| hit_count > u64::from(limit))
                || spatial
                    .max_kills
                    .is_some_and(|limit| spatial.kills >= limit)
            {
                let _ = self.finish_basic_lifesteal(
                    &projectile.source_snapshot,
                    spatial.actual_hp_loss,
                    impact.time_ms,
                );
                return;
            }
        }
        if now
            >= projectile
                .fired_at_ms
                .saturating_add(projectile.max_travel_ms)
        {
            if projectile.spatial.as_ref().unwrap().hit_unit_ids.is_empty() {
                self.record_basic_attack_projectile_miss(
                    now,
                    id,
                    projectile.attacker_instance_id,
                    projectile.target_instance_id,
                    projectile.damage_type,
                    motion.end_position,
                );
            }
            let _ = self.finish_basic_lifesteal(
                &projectile.source_snapshot,
                projectile.spatial.as_ref().unwrap().actual_hp_loss,
                now,
            );
            return;
        }
        projectile.current_position = motion.end_position;
        projectile.last_reevaluation_ms = now;
        self.event_queue
            .push(BattleEvent::BasicAttackProjectileAdvance {
                time_ms: motion.next_reevaluation_ms,
                projectile_id: id,
                cause: projectile.cause.clone(),
            });
        self.projectiles.insert(id, projectile);
        #[cfg(feature = "runtime-observation")]
        self.observe_projectile_peak();
    }
}
