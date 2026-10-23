mod approach_taunt_tests {
    use super::*;
    use crate::game::battle::core::{
        movement::types::{MovementGoal, WorldVec2},
        sim::{BattleExecutionState, BattleLiveCommand},
    };
    fn fixture() -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let authored = r12_control_ron("Sustained(definition: Control(duration_ms: 1500, kind: ApproachTaunt(profile: (speed_percent: 100, attack_range_units: 1000000, damage_reduction_percent: 20, basic_damage_bonus_percent: 30))))");
        let (mut core, state, a, b) = r11_configured_effect_core(&authored, |s| {
            r12_ranged_enemy(s);
            s.terminal_outcomes = BattleScenario::fixture_survival_outcomes(5000);
        });
        let enemy = core
            .units
            .values()
            .find(|u| u.owner == Side::Opponent)
            .unwrap()
            .instance_id;
        for (id, x, y) in [(a, 2, 1), (b, 5, 2), (enemy, 6, 1)] {
            crate::game::battle::core::relic_structural_tests::flow_cases::place_stationary(
                &mut core, id, x, y,
            );
            let u = core.units.get_mut(&id).unwrap();
            u.stats.current_health = 1000;
            u.stats.max_health = 1000;
            u.next_basic_attack_ms = 1000;
            u.basic_attack_windup_until_ms = None;
            u.action_locks = Default::default();
        }
        let u = core.units.get_mut(&b).unwrap();
        u.block_capacity = 1;
        u.block_radius_units = 1.5;
        let u = core.units.get_mut(&a).unwrap();
        u.block_capacity = 1;
        u.block_radius_units = 1.0;
        let u = core.units.get_mut(&enemy).unwrap();
        u.stats.move_speed_units_per_ms = 1000;
        u.body.move_speed = 1.0;
        core.refresh_block_state();
        (core, state, a, b, enemy)
    }
    fn cast(core: &mut BattleCore, actor: UnitInstanceId, now: u64) {
        core.apply_live_command(BattleLiveCommand::ActivateSkill {
            unit_id: actor,
            skill_id: "r11_aura".into(),
            target: None,
            time_ms: now,
        })
        .unwrap();
    }
    #[test]
    fn approach_taunt_releases_other_blocker_and_approaches_before_ranged_attack() {
        let (mut core, mut state, caster, other, enemy) = fixture();
        assert_eq!(core.blocked_by(enemy), Some(other));
        cast(&mut core, caster, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert_eq!(core.blocked_by(enemy), None);
        assert_eq!(core.taunt_source(enemy), Some(caster));
        assert_eq!(core.units[&enemy].stats.move_speed_units_per_ms, 2000);
        assert!(!core.is_basic_attack_target_in_range(enemy, caster));
        assert!(!core.is_basic_attack_target_in_range(enemy, other));
        let goals = core.build_continuous_attack_goals();
        assert!(
            matches!(goals.get(&enemy),Some(MovementGoal::MoveToPoint{point,..}) if *point==core.units[&caster].body.position)
        );
        let initial = core.units[&enemy].body.position.x;
        core.step_battle_execution_until(&mut state, 500).unwrap();
        assert!(
            core.units[&enemy].body.position.x < initial - 0.5,
            "enemy must actually chase: initial={initial}, body={:?}",
            core.units[&enemy].body
        );
        assert_eq!(
            core.blocked_by(enemy),
            None,
            "the previous blocker cannot reacquire while taunted"
        );
        assert_eq!(core.units[&caster].stats.current_health, 1000);
        core.step_battle_execution_until(&mut state, 1501).unwrap();
        assert_eq!(core.taunt_source(enemy), None);
        assert_eq!(core.units[&enemy].stats.move_speed_units_per_ms, 1000);
    }
    #[test]
    fn approach_taunt_replacement_never_revives_old_control_and_source_exit_cleans_bundle() {
        let (mut core, mut state, caster, other, enemy) = fixture();
        cast(&mut core, caster, 0);
        core.step_battle_execution_until(&mut state, 100).unwrap();
        cast(&mut core, other, 100);
        core.step_battle_execution_until(&mut state, 101).unwrap();
        assert_eq!(core.taunt_source(enemy), Some(other));
        assert_eq!(
            core.units[&enemy].stats.move_speed_units_per_ms, 2000,
            "replacement does not stack speed"
        );
        assert_eq!(core.sustained_effect_snapshot().unwrap().active.len(), 1);
        core.apply_live_command(BattleLiveCommand::WithdrawUnit {
            unit_id: other,
            time_ms: 102,
        })
        .unwrap();
        core.step_battle_execution_until(&mut state, 102).unwrap();
        assert_eq!(core.taunt_source(enemy), None);
        assert_eq!(core.units[&enemy].stats.move_speed_units_per_ms, 1000);
        assert!(!core.resonance_use_has_obligations(caster));
    }
    #[test]
    fn approach_taunt_attack_limit_uses_continuous_distance_and_preserves_native_reach() {
        let (mut core, mut state, caster, _, enemy) = fixture();
        cast(&mut core, caster, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        let center = core.units[&caster].body.position;
        core.active_movement_segments.remove(&enemy);
        core.units
            .get_mut(&enemy)
            .unwrap()
            .set_world_position(WorldVec2::new(center.x + 1.001, center.y));
        assert!(!core.is_basic_attack_target_in_range(enemy, caster));
        core.units
            .get_mut(&enemy)
            .unwrap()
            .set_world_position(WorldVec2::new(center.x + 1.0, center.y));
        assert!(core.is_basic_attack_target_in_range(enemy, caster));
        core.units.get_mut(&caster).unwrap().block_capacity = 0;
        core.refresh_block_state();
        let u = core.units.get_mut(&enemy).unwrap();
        u.basic_attack.range_policy = crate::game::battle::tile_range::TileRangePolicy::Pattern;
        u.basic_attack.defense_tile_range =
            Some(crate::game::battle::tile_range::TileRangePattern {
                include_anchor_tile: true,
                rows: vec!["@".into()],
            });
        assert!(
            !core.is_basic_attack_target_in_range(enemy, caster),
            "taunt does not extend a shorter original range"
        );
    }
    #[test]
    fn approach_taunt_cancellation_failure_restores_bundle_block_and_pending_events() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let (mut core, mut state, caster, other, enemy) = fixture();
        cast(&mut core, caster, 0);
        core.step_battle_execution_until(&mut state, 0).unwrap();
        let before = (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            format!("{state:?}"),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("approach_taunt_cancelled_attacks", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || core.step_battle_execution_until(&mut state, 1),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(
            (
                core.transaction_debug_snapshot(),
                core.event_queue_debug_snapshot(),
                format!("{state:?}")
            ),
            before
        );
        assert_eq!(core.blocked_by(enemy), Some(other));
        assert_eq!(core.taunt_source(enemy), None);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert_eq!(core.taunt_source(enemy), Some(caster));
    }
    #[test]
    fn approach_taunt_explicit_immunity_preserves_previous_bundle_and_blocking() {
        let (mut core, mut state, caster, other, enemy) = fixture();
        core.units
            .get_mut(&enemy)
            .unwrap()
            .target_traits
            .push(crate::game::battle::types::UnitTargetTrait::TauntImmune);
        cast(&mut core, caster, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert_eq!(core.taunt_source(enemy), None);
        assert_eq!(core.units[&enemy].stats.move_speed_units_per_ms, 1000);
        assert_eq!(core.blocked_by(enemy), Some(other));
        assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
    }
    #[test]
    fn approach_taunt_damage_modifiers_apply_only_to_the_owned_pair_and_basic_source() {
        use crate::game::battle::damage::{BattleCommand, DamageSource, DamageType};
        let (mut core, mut state, caster, other, enemy) = fixture();
        cast(&mut core, caster, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        for unit in core.units.values_mut() {
            unit.stats.defense = 0;
        }
        for (source, target, kind, loss) in [
            (enemy, caster, DamageSource::Ability, 80),
            (enemy, other, DamageSource::Ability, 100),
            (caster, enemy, DamageSource::BasicAttack, 130),
            (other, enemy, DamageSource::BasicAttack, 100),
            (caster, enemy, DamageSource::Ability, 100),
        ] {
            let before = core.units[&target].stats.current_health;
            core.runtime_transaction(|c| {
                let snapshot = c
                    .damage_source_snapshot_for_unit(
                        source,
                        target,
                        kind,
                        DamageType::Physical,
                        100,
                        Default::default(),
                        0,
                        1,
                        false,
                    )
                    .ok_or(GameError::InvalidAction)?;
                c.process_commands(
                    vec![BattleCommand::ApplyDamage {
                        target_id: target,
                        source_snapshot: snapshot,
                    }],
                    1,
                );
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
            assert_eq!(
                before - core.units[&target].stats.current_health,
                loss,
                "{kind:?} {source:?} -> {target:?}"
            );
        }
    }
    fn offense(
        delivery: &str,
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let authored = R08_SKILLS
            .replace("r08_strike", "r11_aura")
            .replace("delay_ms: 200", "delay_ms: 0")
            .replace("delivery: Instant", &format!("delivery: {delivery}"));
        let (mut core, state, actor, _) = r11_configured_effect_core(&authored, |s| {
            r12_ranged_enemy(s);
            s.terminal_outcomes = BattleScenario::fixture_survival_outcomes(4000);
        });
        let enemy = core
            .units
            .values()
            .find(|u| u.owner == Side::Opponent)
            .unwrap()
            .instance_id;
        for u in core.units.values_mut() {
            u.next_basic_attack_ms = 100000;
            u.basic_attack_epoch += 1;
            u.stats.move_speed_units_per_ms = 0;
            u.body.move_speed = 0.0;
        }
        (core, state, actor, enemy)
    }
    fn force_taunt(
        core: &mut BattleCore,
        source: UnitInstanceId,
        target: UnitInstanceId,
        now: u64,
    ) {
        use crate::game::ability::sustained::*;
        core.runtime_transaction(|c| {
            c.begin_sustained(
                source,
                target,
                SustainedEffectDef::Control {
                    duration_ms: 1500,
                    kind: ControlKind::ApproachTaunt {
                        profile: ApproachTauntProfile {
                            speed_percent: 100,
                            attack_range_units: 1000000,
                            damage_reduction_percent: 0,
                            basic_damage_bonus_percent: 0,
                        },
                    },
                },
                now,
            )
        })
        .unwrap();
    }
    #[test]
    fn approach_taunt_checks_skill_start_and_release_and_cancels_unreleased_cast() {
        use crate::game::battle::core::relic_structural_tests::flow_cases::place_stationary;
        let (mut core, mut state, actor, enemy) = offense("Instant");
        force_taunt(&mut core, enemy, actor, 0);
        let before = core.units[&actor].resonance_current;
        assert!(core
            .apply_live_command(BattleLiveCommand::ActivateSkill {
                unit_id: actor,
                skill_id: "r11_aura".into(),
                target: None,
                time_ms: 0
            })
            .is_err());
        assert_eq!(core.units[&actor].resonance_current, before);
        place_stationary(&mut core, actor, 5, 1);
        let hp = core.units[&enemy].stats.current_health;
        cast(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 10).unwrap();
        assert!(core.units[&actor].pending_skill_cast.is_some());
        place_stationary(&mut core, enemy, 7, 1);
        core.step_battle_execution_until(&mut state, 200).unwrap();
        assert_eq!(
            core.units[&enemy].stats.current_health, hp,
            "out of one-tile reach at release"
        );
        let (mut core, mut state, actor, enemy) = offense("Instant");
        cast(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 10).unwrap();
        assert!(core.units[&actor].pending_skill_cast.is_some());
        force_taunt(&mut core, enemy, actor, 10);
        assert!(core.units[&actor].pending_skill_cast.is_none());
        let hp = core.units[&enemy].stats.current_health;
        core.step_battle_execution_until(&mut state, 200).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, hp);
    }
    #[test]
    fn approach_taunt_preserves_already_released_projectile() {
        let (mut core, mut state, actor, enemy) = offense("Projectile(speed_units_per_ms: 25000)");
        cast(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 101).unwrap();
        assert!(!core.active_projectiles.is_empty());
        let hp = core.units[&enemy].stats.current_health;
        force_taunt(&mut core, enemy, actor, 101);
        core.step_battle_execution_until(&mut state, 1000).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, hp - 100);
        assert!(core.active_projectiles.is_empty());
    }
}
