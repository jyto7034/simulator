use uuid::Uuid;

use crate::ecs::resources::Position;
use crate::game::ability::DeliveryDef;
use crate::game::battle::core::BattleCore;
use crate::game::battle::damage::{
    apply_damage_to_unit, calculate_damage, BattleCommand, DamageContext, DamageRequest,
    DamageResult, DamageSource,
};
use crate::game::battle::enums::BattleEvent;
use crate::game::battle::enums::ProjectilePayload;
use crate::game::battle::ids::UnitInstanceId;
use crate::game::battle::timeline::{HpChangeReason, TimelineCause, TimelineEvent};
use crate::game::determinism;
use crate::game::enums::Side;
use crate::game::stats::TriggerType;

use super::movement::{ActionState, TILE_UNITS_PER_TILE};

fn projectile_flight_ms(distance_units: u64, speed_units_per_ms: u32) -> u64 {
    if distance_units == 0 {
        return 0;
    }
    let speed_units_per_ms = speed_units_per_ms as u64;
    if speed_units_per_ms == 0 {
        // Defensive: treat invalid projectile speed as "instant" rather than panicking/overflowing.
        return 0;
    }
    distance_units.saturating_add(speed_units_per_ms.saturating_sub(1)) / speed_units_per_ms
}

impl BattleCore {
    pub(super) fn schedule_projectile_hit_event(
        &mut self,
        fired_at_ms: u64,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        attacker_pos: Position,
        target_pos: Position,
        speed_units_per_ms: u32,
        payload: ProjectilePayload,
    ) {
        // TODO: 추후 config 로 빼야함.
        const PROJECTILE_NS: u64 = 0x5052_4F4A_4543_544Cu64; // "PROJECTL"

        let dist_tiles = attacker_pos.chebyshev(&target_pos).max(0) as u64;
        let distance_units = dist_tiles.saturating_mul(TILE_UNITS_PER_TILE);
        let flight_ms = projectile_flight_ms(distance_units, speed_units_per_ms);
        let impact_ms = fired_at_ms.saturating_add(flight_ms);

        let seed = self
            .recording_cause()
            .and_then(|cause| cause.parent_seq())
            .unwrap_or_else(|| {
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&attacker_instance_id.as_bytes()[..8]);
                fired_at_ms ^ u64::from_be_bytes(bytes)
            });
        let projectile_seq = self.projectile_seq;
        self.projectile_seq = self.projectile_seq.wrapping_add(1);
        let projectile_id = determinism::uuid_v4_from_seed(seed, PROJECTILE_NS, projectile_seq);

        self.projectiles
            .insert(projectile_id, super::ProjectileRecord { fired_at_ms });

        self.event_queue.push(BattleEvent::ProjectileHit {
            time_ms: impact_ms,
            projectile_id,
            attacker_instance_id,
            target_instance_id,
            payload,
            cause: self.recording_cause().unwrap_or_default(),
        });
    }

    fn calculate_basic_attack_damage_snapshot(
        &mut self,
        attacker_instance_id: UnitInstanceId,
        attacker_owner: Side,
        attacker_attack: u32,
        target_instance_id: UnitInstanceId,
        target_owner: Side,
        target_defense: u32,
        target_current_hp: u32,
        target_max_hp: u32,
        time_ms: u64,
    ) -> DamageResult {
        let on_attack_effects =
            self.collect_all_triggers(attacker_instance_id, TriggerType::OnAttack);
        let on_hit_effects = self.collect_all_triggers(target_instance_id, TriggerType::OnHit);

        let ctx = DamageContext {
            attacker_side: attacker_owner,
            target_side: target_owner,
            attacker_attack,
            target_defense,
            target_current_hp,
            target_max_hp,
            on_attack_effects: &on_attack_effects,
            on_hit_effects: &on_hit_effects,
        };

        let request = DamageRequest {
            source: DamageSource::BasicAttack,
            attacker_id: attacker_instance_id,
            target_id: target_instance_id,
            base_damage: attacker_attack,
            time_ms,
        };

        calculate_damage(&request, &ctx)
    }

    fn apply_damage_and_record(
        &mut self,
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        damage: u32,
        time_ms: u64,
        reason: HpChangeReason,
    ) {
        let (target_owner, hp_before) = {
            let Some(target) = self.units.get(&target_instance_id) else {
                return;
            };
            if target.is_dead() {
                return;
            }
            (target.owner, target.stats.current_health)
        };

        let Some(target) = self.units.get_mut(&target_instance_id) else {
            return;
        };

        apply_damage_to_unit(&mut target.stats, damage);
        let hp_after = target.stats.current_health;
        let delta = hp_after as i32 - hp_before as i32;

        self.record_timeline(
            time_ms,
            TimelineEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                delta,
                hp_before,
                hp_after,
                reason,
            },
        );

        if hp_after < hp_before && matches!(reason, HpChangeReason::BasicAttack) {
            let gained = (hp_before - hp_after) / 10;
            if gained > 0 {
                self.add_resonance(target_instance_id, gained, time_ms, hp_after > 0);
            }
        }

        if hp_after == 0 {
            if let Some(target) = self.units.get_mut(&target_instance_id) {
                target.move_epoch = target.move_epoch.wrapping_add(1);
                target.action_state = ActionState::Idle;
            }
            if let Some(position) = self.battlefield.remove(target_instance_id) {
                if let Some(target) = self.units.get(&target_instance_id) {
                    self.graveyard
                        .insert(target_instance_id, target.to_snapshot(position));
                }
            }
            self.record_timeline(
                time_ms,
                TimelineEvent::UnitDied {
                    unit_instance_id: target_instance_id,
                    owner: target_owner,
                    killer_instance_id: source_instance_id,
                },
            );
        }
    }

    fn apply_hp_delta_and_record(
        &mut self,
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        delta: i32,
        time_ms: u64,
        reason: HpChangeReason,
    ) {
        if delta == 0 {
            return;
        }

        let (target_owner, hp_before, max_health) = {
            let Some(target) = self.units.get(&target_instance_id) else {
                return;
            };
            if target.is_dead() {
                return;
            }
            (
                target.owner,
                target.stats.current_health,
                target.stats.max_health,
            )
        };

        let Some(target) = self.units.get_mut(&target_instance_id) else {
            return;
        };

        let hp_after = if delta > 0 {
            hp_before
                .saturating_add(delta as u32)
                .min(max_health.max(1))
        } else {
            hp_before.saturating_sub(delta.unsigned_abs())
        };
        target.stats.current_health = hp_after;

        let applied_delta = hp_after as i32 - hp_before as i32;
        self.record_timeline(
            time_ms,
            TimelineEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                delta: applied_delta,
                hp_before,
                hp_after,
                reason,
            },
        );

        if hp_after < hp_before && matches!(reason, HpChangeReason::BasicAttack) {
            let gained = (hp_before - hp_after) / 10;
            if gained > 0 {
                self.add_resonance(target_instance_id, gained, time_ms, hp_after > 0);
            }
        }

        if hp_after == 0 {
            if let Some(position) = self.battlefield.remove(target_instance_id) {
                if let Some(target) = self.units.get(&target_instance_id) {
                    self.graveyard
                        .insert(target_instance_id, target.to_snapshot(position));
                }
            }
            self.record_timeline(
                time_ms,
                TimelineEvent::UnitDied {
                    unit_instance_id: target_instance_id,
                    owner: target_owner,
                    killer_instance_id: source_instance_id,
                },
            );
        }
    }

    pub(super) fn apply_projectile_hit(
        &mut self,
        time_ms: u64,
        projectile_id: Uuid,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        payload: ProjectilePayload,
    ) {
        if self.projectiles.remove(&projectile_id).is_none() {
            return;
        };

        match payload {
            ProjectilePayload::BasicAttack => {
                let target_alive = self
                    .units
                    .get(&target_instance_id)
                    .is_some_and(|t| !t.is_dead());
                if !target_alive {
                    self.record_timeline(
                        time_ms,
                        TimelineEvent::ProjectileMiss {
                            projectile_id,
                            attacker_instance_id,
                            target_instance_id,
                        },
                    );
                    return;
                }
            }
            ProjectilePayload::Skill {
                skill_id,
                cast_target,
                ..
            } => {
                let Some(skill) = self.game_data.skill_data.get_by_id(&skill_id).cloned() else {
                    return;
                };

                let targets = self.resolve_skill_targets_at_execute(
                    attacker_instance_id,
                    &skill,
                    cast_target,
                );
                let commands =
                    Self::build_skill_effect_commands(attacker_instance_id, &skill, &targets);

                if !commands.is_empty() {
                    self.process_commands(commands, time_ms);
                }
                self.schedule_pending_autocasts(time_ms);
                return;
            }
        }

        let (target_owner, target_defense, target_current_hp, target_max_hp) = {
            let Some(target) = self.units.get(&target_instance_id) else {
                return;
            };
            if target.is_dead() {
                return;
            }
            (
                target.owner,
                target.stats.defense,
                target.stats.current_health,
                target.stats.max_health,
            )
        };

        let attacker_live = matches!(
            self.units.get(&attacker_instance_id),
            Some(unit) if !unit.is_dead()
        );

        let attacker_attack = if let Some(unit) = self.units.get(&attacker_instance_id) {
            unit.stats.attack
        } else {
            self.graveyard
                .get(&attacker_instance_id)
                .map(|s| s.stats.attack)
                .unwrap_or(0)
        };

        if !attacker_live {
            // 피격 시점에 공격자가 죽었다면, 기본 공격 데미지만 입힘.
            let base_damage = attacker_attack.saturating_sub(target_defense).max(1);
            self.apply_damage_and_record(
                Some(attacker_instance_id),
                target_instance_id,
                base_damage,
                time_ms,
                HpChangeReason::BasicAttack,
            );
            self.schedule_pending_autocasts(time_ms);
            return;
        }

        let attacker_owner = match self.units.get(&attacker_instance_id) {
            Some(unit) => unit.owner,
            None => target_owner,
        };

        let on_attack_effects =
            self.collect_all_triggers(attacker_instance_id, TriggerType::OnAttack);
        let on_hit_effects = self.collect_all_triggers(target_instance_id, TriggerType::OnHit);

        let ctx = DamageContext {
            attacker_side: attacker_owner,
            target_side: target_owner,
            attacker_attack,
            target_defense,
            target_current_hp,
            target_max_hp,
            on_attack_effects: &on_attack_effects,
            on_hit_effects: &on_hit_effects,
        };

        let request = DamageRequest {
            source: DamageSource::BasicAttack,
            attacker_id: attacker_instance_id,
            target_id: target_instance_id,
            base_damage: attacker_attack,
            time_ms,
        };

        let result = calculate_damage(&request, &ctx);

        let dealt = target_current_hp.saturating_sub(result.target_remaining_hp);
        let gained = dealt / 10;
        if gained > 0 {
            self.add_resonance(attacker_instance_id, gained, time_ms, true);
        }

        self.apply_damage_and_record(
            Some(attacker_instance_id),
            target_instance_id,
            result.final_damage,
            time_ms,
            HpChangeReason::BasicAttack,
        );

        if !result.triggered_commands.is_empty() {
            self.process_commands(result.triggered_commands, time_ms);
        }

        self.schedule_pending_autocasts(time_ms);
    }

    pub(super) fn resolve_basic_attack(
        &mut self,
        attacker_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
        current_time_ms: u64,
    ) -> bool {
        let (attacker_owner, attacker_base_uuid, attacker_attack) = {
            let Some(attacker) = self.units.get(&attacker_instance_id) else {
                return false;
            };
            if attacker.is_dead() {
                return false;
            }
            (attacker.owner, attacker.base_uuid, attacker.stats.attack)
        };

        let (target_owner, target_defense, target_current_hp, target_max_hp) = {
            let Some(target) = self.units.get(&target_id) else {
                return false;
            };
            if target.is_dead() {
                return false;
            }
            (
                target.owner,
                target.stats.defense,
                target.stats.current_health,
                target.stats.max_health,
            )
        };

        if target_owner == attacker_owner {
            return false;
        }

        let Some(attacker_pos) = self.battlefield.position_of(attacker_instance_id) else {
            return false;
        };
        let Some(target_pos) = self.battlefield.position_of(target_id) else {
            return false;
        };

        let basic = self
            .game_data
            .abnormality_data
            .get_by_uuid(&attacker_base_uuid)
            .map(|m| m.basic_attack.clone())
            .unwrap_or_default();

        // 사거리 체크(스펙: chebyshev)
        if attacker_pos.chebyshev(&target_pos) > basic.range_tiles as i32 {
            return false;
        }

        match basic.delivery {
            DeliveryDef::Instant => {
                // Immediate resonance gain on attack release.
                self.add_resonance(attacker_instance_id, 10, current_time_ms, true);

                let result = self.calculate_basic_attack_damage_snapshot(
                    attacker_instance_id,
                    attacker_owner,
                    attacker_attack,
                    target_id,
                    target_owner,
                    target_defense,
                    target_current_hp,
                    target_max_hp,
                    current_time_ms,
                );

                // Resonance gain: 10% of actual HP decrease dealt.
                let dealt = target_current_hp.saturating_sub(result.target_remaining_hp);
                let gained = dealt / 10;
                if gained > 0 {
                    self.add_resonance(attacker_instance_id, gained, current_time_ms, true);
                }

                self.apply_damage_and_record(
                    Some(attacker_instance_id),
                    target_id,
                    result.final_damage,
                    current_time_ms,
                    HpChangeReason::BasicAttack,
                );

                if !result.triggered_commands.is_empty() {
                    self.process_commands(result.triggered_commands, current_time_ms);
                }

                self.schedule_pending_autocasts(current_time_ms);
                true
            }

            DeliveryDef::Projectile { speed_units_per_ms } => {
                self.add_resonance(attacker_instance_id, 10, current_time_ms, true);
                self.schedule_projectile_hit_event(
                    current_time_ms,
                    attacker_instance_id,
                    target_id,
                    attacker_pos,
                    target_pos,
                    speed_units_per_ms,
                    ProjectilePayload::BasicAttack,
                );
                true
            }
        }
    }

    pub(super) fn process_commands(&mut self, commands: Vec<BattleCommand>, current_time_ms: u64) {
        for command in commands {
            match command {
                BattleCommand::UnitDied { .. } => {
                    // Death is derived from HP reaching 0 and recorded in HpChanged handling.
                }
                BattleCommand::CastSkill { .. } => {
                    // Skill execution is not implemented yet in this core; ignore for now.
                }
                BattleCommand::ApplyModifier {
                    target_id,
                    modifier,
                } => {
                    let Some(target) = self.units.get_mut(&target_id) else {
                        continue;
                    };
                    if target.is_dead() {
                        continue;
                    }

                    let before = target.stats;
                    target.stats.apply_modifier(modifier);
                    let after = target.stats;

                    self.record_timeline(
                        current_time_ms,
                        TimelineEvent::StatChanged {
                            source_instance_id: None,
                            target_instance_id: target_id,
                            modifier,
                            stats_before: before,
                            stats_after: after,
                        },
                    );
                }
                BattleCommand::ApplyHeal {
                    target_id,
                    flat,
                    percent,
                    source_id,
                } => {
                    let max_health = match self.units.get(&target_id) {
                        Some(unit) if !unit.is_dead() => unit.stats.max_health.max(1),
                        _ => continue,
                    };

                    let percent_delta = (i64::from(max_health) * i64::from(percent)) / 100;
                    let delta_i64 = i64::from(flat).saturating_add(percent_delta);
                    let delta = delta_i64.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;

                    self.apply_hp_delta_and_record(
                        source_id,
                        target_id,
                        delta,
                        current_time_ms,
                        HpChangeReason::Command,
                    );
                }
                BattleCommand::ScheduleAttack {
                    attacker_id,
                    target_id,
                    time_ms,
                } => {
                    self.event_queue.push(BattleEvent::AttackStart {
                        time_ms: current_time_ms.saturating_add(time_ms),
                        attacker_instance_id: attacker_id,
                        target_instance_id: target_id,
                        schedule_next: false,
                        cause: self.recording_cause().unwrap_or(TimelineCause::default()),
                    });
                }
                BattleCommand::ApplyBuff {
                    caster_id,
                    target_id,
                    buff_id,
                    duration_ms,
                } => {
                    self.event_queue.push(BattleEvent::ApplyBuff {
                        time_ms: current_time_ms,
                        caster_instance_id: caster_id,
                        target_instance_id: target_id,
                        buff_id,
                        duration_ms,
                        cause: self.recording_cause().unwrap_or_default(),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::projectile_flight_ms;
    use crate::ecs::resources::Position;
    use crate::game::battle::core::movement::ActionState;
    use crate::game::battle::core::ProjectileRecord;
    use crate::game::battle::core::types::RuntimeUnit;
    use crate::game::battle::enums::ProjectilePayload;
    use crate::game::battle::timeline::{HpChangeReason, Timeline};
    use crate::game::battle::types::PlayerDeckInfo;
    use crate::game::data::{
        abnormality_data::AbnormalityDatabase, artifact_data::ArtifactDatabase,
        bonus_data::BonusDatabase, equipment_data::EquipmentDatabase,
        pve_data::PveEncounterDatabase, random_event_data::RandomEventDatabase,
        shop_data::ShopDatabase, skill_data::SkillDatabase, GameDataBase,
    };
    use crate::game::enums::Side;
    use crate::game::stats::UnitStats;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;
    use uuid::Uuid;

    #[test]
    fn projectile_flight_ms_is_zero_when_speed_is_zero() {
        assert_eq!(projectile_flight_ms(1, 0), 0);
        assert_eq!(projectile_flight_ms(1_000_000, 0), 0);
    }

    #[test]
    fn projectile_flight_ms_is_zero_when_distance_is_zero() {
        assert_eq!(projectile_flight_ms(0, 0), 0);
        assert_eq!(projectile_flight_ms(0, 1), 0);
        assert_eq!(projectile_flight_ms(0, u32::MAX), 0);
    }

    #[test]
    fn projectile_flight_ms_uses_ceil_division() {
        assert_eq!(projectile_flight_ms(1_000_000, 3_000), 334);
        assert_eq!(projectile_flight_ms(1_000_000, 2_000_000), 1);
        assert_eq!(projectile_flight_ms(2_000_000, 2_000_000), 1);
    }

    fn empty_game_data() -> Arc<GameDataBase> {
        Arc::new(GameDataBase::new(
            Arc::new(AbnormalityDatabase::new(vec![])),
            Arc::new(ArtifactDatabase::new(vec![])),
            Arc::new(EquipmentDatabase::new(vec![])),
            Arc::new(ShopDatabase::new(vec![])),
            Arc::new(BonusDatabase::new(vec![])),
            Arc::new(RandomEventDatabase::new(vec![])),
            Arc::new(PveEncounterDatabase::new(vec![])),
            Arc::new(SkillDatabase::new(vec![])),
            crate::game::data::event_pools::EventPoolConfig {
                dawn: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                noon: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                dusk: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                midnight: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                white: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
            },
        ))
    }

    fn write_timeline_export(name: &str, timeline: &Timeline) -> PathBuf {
        let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("timeline_exports");
        std::fs::create_dir_all(&out_dir).expect("create timeline_exports directory");
        let out_path = out_dir.join(format!("{name}.json"));
        timeline
            .write_pretty_json(&out_path)
            .expect("write timeline json");
        out_path
    }

    #[test]
    fn apply_projectile_hit_is_idempotent_for_same_projectile_id() {
        let empty_deck = PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: HashMap::new(),
        };
        let mut core =
            super::BattleCore::new(&empty_deck, &empty_deck, empty_game_data(), (4, 4), 1);

        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(1));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(2));

        let mut attacker_stats = UnitStats::with_values(100, 100, 10, 0, 1000);
        attacker_stats.move_speed_units_per_ms = 1;
        let mut target_stats = UnitStats::with_values(100, 100, 0, 0, 1000);
        target_stats.move_speed_units_per_ms = 1;

        core.units.insert(
            attacker_id,
            RuntimeUnit {
                instance_id: attacker_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats: attacker_stats,
                pos_x_units: 0,
                pos_y_units: 0,
                move_epoch: 0,
                action_state: ActionState::Idle,
                action_locks: Default::default(),
                current_target: None,
                next_basic_attack_ms: 0,
                pending_basic_attack: false,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                pending_autocast: None,
            },
        );

        core.units.insert(
            target_id,
            RuntimeUnit {
                instance_id: target_id,
                owner: Side::Opponent,
                base_uuid: Uuid::nil(),
                stats: target_stats,
                pos_x_units: 0,
                pos_y_units: 0,
                move_epoch: 0,
                action_state: ActionState::Idle,
                action_locks: Default::default(),
                current_target: None,
                next_basic_attack_ms: 0,
                pending_basic_attack: false,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                pending_autocast: None,
            },
        );

        core.battlefield
            .place(attacker_id, Position::new(0, 0))
            .unwrap();
        core.battlefield
            .place(target_id, Position::new(1, 0))
            .unwrap();

        let projectile_id = Uuid::from_u128(0xAAAA);
        core.projectiles
            .insert(projectile_id, ProjectileRecord { fired_at_ms: 0 });

        core.apply_projectile_hit(
            10,
            projectile_id,
            attacker_id,
            target_id,
            ProjectilePayload::BasicAttack,
        );

        let hp_after_first = core
            .units
            .get(&target_id)
            .map(|u| u.stats.current_health)
            .unwrap();

        core.apply_projectile_hit(
            10,
            projectile_id,
            attacker_id,
            target_id,
            ProjectilePayload::BasicAttack,
        );

        let hp_after_second = core
            .units
            .get(&target_id)
            .map(|u| u.stats.current_health)
            .unwrap();

        assert_eq!(hp_after_first, hp_after_second);

        let hits = core
            .timeline
            .entries
            .iter()
            .filter(|entry| match &entry.event {
                crate::game::battle::timeline::TimelineEvent::HpChanged {
                    source_instance_id,
                    target_instance_id,
                    reason,
                    ..
                } if *source_instance_id == Some(attacker_id)
                    && *target_instance_id == target_id
                    && *reason == HpChangeReason::BasicAttack =>
                {
                    true
                }
                _ => false,
            })
            .count();
        assert_eq!(hits, 1);

        write_timeline_export(
            "apply_projectile_hit_is_idempotent_for_same_projectile_id",
            &core.timeline,
        );
    }
}
