use uuid::Uuid;

use crate::game::{
    battle::{
        core::{
            movement::types::{WorldVec2, DATA_UNITS_PER_WORLD},
            projectile_math::projectile_flight_ms_between_world_points,
            spatial::moving_circle_sweep_hit_fraction,
            types::{ProjectileGuidance, ProjectileRecord},
            BattleCore,
        },
        damage::{calculate_damage, DamageContext, DamageSourceSnapshot, DamageType},
        enums::BattleEvent,
        event_log::{BattleLogEvent, HpChangeReason},
        ids::UnitInstanceId,
    },
    determinism,
    enums::Side,
    stats::TriggerType,
};

#[derive(Debug, Clone)]
pub(super) struct ProjectileLaunch {
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

const BASIC_ATTACK_PROJECTILE_REEVALUATION_TICK_MS: u64 = 1;

fn projectile_impact_position_at_hit_fraction(
    window_start: WorldVec2,
    window_end: WorldVec2,
    hit_fraction: f32,
) -> WorldVec2 {
    window_start + (window_end - window_start) * hit_fraction.clamp(0.0, 1.0)
}

impl BattleCore {
    pub(super) fn spawn_basic_attack_projectile(&mut self, launch: ProjectileLaunch) {
        // TODO: 추후 config 로 빼야함.
        const PROJECTILE_NS: u64 = 0x5052_4F4A_4543_544Cu64; // "PROJECTL"

        let flight_ms = projectile_flight_ms_between_world_points(
            launch.attacker_origin,
            launch.target_aim,
            launch.speed_units_per_ms,
        );
        let impact_ms = launch.fired_at_ms.saturating_add(flight_ms);

        let seed = self
            .recording_cause()
            .and_then(|cause| cause.parent_seq())
            .unwrap_or_else(|| {
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&launch.attacker_instance_id.as_bytes()[..8]);
                launch.fired_at_ms ^ u64::from_be_bytes(bytes)
            });
        let projectile_seq = self.projectile_seq;
        self.projectile_seq = self.projectile_seq.wrapping_add(1);
        let projectile_id = determinism::uuid_v4_from_seed(seed, PROJECTILE_NS, projectile_seq);

        self.projectiles.insert(
            projectile_id,
            ProjectileRecord {
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

        let window_duration_ms = reevaluation_time_ms.saturating_sub(from_time_ms);
        let aim = match projectile.guidance {
            ProjectileGuidance::Homing => target_body_end.position,
            ProjectileGuidance::Fixed => projectile.aim,
        };
        let direction = aim - projectile.current_position;
        let distance = direction.length();
        let speed_world_per_ms = projectile.speed_units_per_ms as f32 / DATA_UNITS_PER_WORLD;
        let max_step = speed_world_per_ms * window_duration_ms as f32;
        let next_position = if distance <= f32::EPSILON
            || projectile.speed_units_per_ms == 0
            || max_step >= distance
        {
            aim
        } else {
            projectile.current_position + direction * (max_step / distance)
        };
        let reach = target_body_start
            .radius
            .max(target_body_end.radius)
            .max(0.0);

        if let Some(hit_fraction) = moving_circle_sweep_hit_fraction(
            projectile.current_position,
            next_position,
            reach,
            target_body_start.position,
            target_body_end.position,
        ) {
            let elapsed_delta = ((window_duration_ms as f32) * hit_fraction).ceil() as u64;
            let impact_time_ms = from_time_ms.saturating_add(elapsed_delta.min(window_duration_ms));
            let impact_position = projectile_impact_position_at_hit_fraction(
                projectile.current_position,
                next_position,
                hit_fraction,
            );
            self.apply_basic_attack_projectile_hit_at(
                impact_time_ms,
                projectile_id,
                projectile.attacker_instance_id,
                projectile.target_instance_id,
                projectile.source_snapshot,
                impact_position,
            );
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

        let next_reevaluation_ms =
            reevaluation_time_ms.saturating_add(BASIC_ATTACK_PROJECTILE_REEVALUATION_TICK_MS);
        self.event_queue
            .push(BattleEvent::BasicAttackProjectileAdvance {
                time_ms: next_reevaluation_ms,
                projectile_id,
                cause: projectile.cause.clone(),
            });
        self.projectiles.insert(projectile_id, projectile);
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
    ) {
        let (
            target_owner,
            target_defense,
            target_magic_resist,
            target_incoming_modifiers,
            target_current_hp,
            target_max_hp,
        ) = {
            let Some(target) = self.units.get(&target_instance_id) else {
                self.record_basic_attack_projectile_miss(
                    time_ms,
                    projectile_id,
                    attacker_instance_id,
                    target_instance_id,
                    source_snapshot.damage_type,
                    impact_position,
                );
                return;
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
                return;
            }
            (
                target.owner,
                target.stats.defense,
                target.stats.magic_resist,
                target.incoming_damage_modifiers,
                target.stats.current_health,
                target.stats.max_health,
            )
        };

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

        let attacker_live = self
            .units
            .get(&attacker_instance_id)
            .is_some_and(|unit| unit.is_active());
        let snapshot_on_attack_effects =
            Self::damage_effects_from_source_snapshot(&source_snapshot);
        let on_hit_effects = self.collect_all_triggers(target_instance_id, TriggerType::OnHit);
        let mut trigger_effect_commands = Vec::new();
        if attacker_live {
            trigger_effect_commands.extend(self.trigger_commands_from_effects(
                self.collect_all_triggers(attacker_instance_id, TriggerType::OnAttack),
                super::commands::TriggerEffectContext {
                    trigger_unit_id: attacker_instance_id,
                    counterpart_unit_id: Some(target_instance_id),
                },
                Some(target_instance_id),
                false,
            ));
        }
        trigger_effect_commands.extend(self.trigger_commands_from_effects(
            on_hit_effects.clone(),
            super::commands::TriggerEffectContext {
                trigger_unit_id: target_instance_id,
                counterpart_unit_id: Some(attacker_instance_id),
            },
            Some(attacker_instance_id),
            false,
        ));
        let mut trigger_ability_commands = if attacker_live {
            Self::activation_commands_from_bindings(
                self.collect_all_trigger_activations(attacker_instance_id, TriggerType::OnAttack),
                attacker_instance_id,
                Some(target_instance_id),
            )
        } else {
            Vec::new()
        };
        trigger_ability_commands.extend(Self::activation_commands_from_bindings(
            self.collect_all_trigger_activations(target_instance_id, TriggerType::OnHit),
            target_instance_id,
            Some(attacker_instance_id),
        ));

        let ctx = DamageContext {
            attacker_side: source_snapshot.source_side,
            target_side: target_owner,
            attacker_attack: source_snapshot.source_attack,
            target_armor: target_defense,
            target_magic_resist,
            target_incoming_modifiers,
            target_current_hp,
            target_max_hp,
            on_attack_effects: &snapshot_on_attack_effects,
            on_hit_effects: &on_hit_effects,
        };

        let request = source_snapshot.request(target_instance_id);
        let result = calculate_damage(&request, &ctx);

        let dealt = target_current_hp.saturating_sub(result.target_remaining_hp);
        self.grant_basic_attack_damage_dealt_resonance(attacker_instance_id, dealt, time_ms);

        self.apply_damage_result_and_record(
            Some(attacker_instance_id),
            &result,
            time_ms,
            HpChangeReason::BasicAttack,
        );

        let mut triggered_commands = result.triggered_commands;
        triggered_commands.splice(0..0, trigger_effect_commands);
        if !triggered_commands.is_empty() {
            self.process_commands(triggered_commands, time_ms);
        }
        if !trigger_ability_commands.is_empty() {
            self.process_commands(trigger_ability_commands, time_ms);
        }

        self.schedule_pending_autocasts(time_ms);
    }
}
