mod weapon_growth_tests {
    use super::*;
    use crate::game::{
        battle::{
            core::sim::BattleExecutionState,
            damage::{DamageSource, DamageType},
            types::BattleUnitThreatClass,
        },
        resource::{
            ResourceCatalog, ResourceOwner, RunResourceEntry, RunResourceState, WeaponKillGrowthDef,
        },
    };
    fn definition() -> WeaponKillGrowthDef {
        ron::de::from_str(r#"(resource:"fixture.weapon.mass",normal:2,elite:5,abnormality:9,damage_per_stack:10,secondary_percent:40)"#).unwrap()
    }
    fn fixture(
        initial: u32,
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
        RunResourceState,
    ) {
        fixture_swap(initial, false)
    }
    fn fixture_swap(
        initial: u32,
        swap: bool,
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
        RunResourceState,
    ) {
        fixture_options(initial, swap, false)
    }
    fn fixture_options(
        initial: u32,
        swap: bool,
        area: bool,
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
        RunResourceState,
    ) {
        let def = definition();
        let authored = r12_control_ron("Heal(amount:1)");
        let (mut core, state, a, b) = r12_fragment_equipped_effect_core_with_data(
            &authored,
            |s| {
                s.terminal_outcomes = BattleScenario::fixture_survival_outcomes(8000);
                s.groups[0].enemy_movement_plan = None;
                if area {
                    let mut secondary = s.groups[0].spawns[0].clone();
                    secondary.unit_ref =
                        crate::game::battle::scenario::ScenarioUnitRef::new("growth_secondary");
                    secondary.instance_salt += 100;
                    secondary.position = crate::game::resources::Position::new(6, 3);
                    s.groups[0].spawns.push(secondary);
                }
                for (i, spawn) in s.groups[1].spawns.iter_mut().enumerate() {
                    spawn.draft.equipped_items[0].persistent_copy_uuid = Some(Uuid::from_u128(
                        0xe100 + (if swap { 1 - i } else { i }) as u128,
                    ));
                }
            },
            |w| w.weapon_profile.as_mut().unwrap().kill_growth = Some(def.clone()),
            None,
            |b| {
                b.with_resources(ResourceCatalog {
                    resources: vec![def.resource_definition()],
                })
            },
        );
        let enemy = core
            .units
            .values()
            .filter(|u| u.owner == Side::Opponent)
            .min_by_key(|u| u.spawn_order)
            .unwrap()
            .instance_id;
        core.units.get_mut(&enemy).unwrap().threat_class = BattleUnitThreatClass::Normal;
        for unit in core.units.values_mut() {
            unit.stats.attack_interval_ms = 100_000;
            unit.next_basic_attack_ms = 100_000;
            unit.stats.move_speed_units_per_ms = 0;
            unit.stats.defense = 0;
            unit.stats.current_health = 1000;
            unit.stats.max_health = 1000;
        }
        for (id, x, y) in [(a, 2, 1), (b, 7, 3), (enemy, 3, 1)] {
            crate::game::battle::core::relic_structural_tests::flow_cases::place_stationary(
                &mut core, id, x, y,
            );
        }
        let owners: std::collections::BTreeSet<_> = core
            .items
            .values()
            .filter_map(|i| i.persistent_copy_uuid)
            .map(ResourceOwner::EquipmentCopy)
            .collect();
        let owner = owner(&core, a);
        let run = RunResourceState {
            revision: 0,
            closed: false,
            entries: if initial == 0 {
                vec![]
            } else {
                vec![RunResourceEntry {
                    resource: def.resource.clone(),
                    owner,
                    balance: initial,
                    revision: 0,
                }]
            },
        };
        core.mount_run_resources(&run, &owners).unwrap();
        (core, state, a, b, enemy, run)
    }
    fn owner(core: &BattleCore, actor: UnitInstanceId) -> ResourceOwner {
        ResourceOwner::EquipmentCopy(
            core.items
                .values()
                .find(|i| i.owner_unit_instance == actor)
                .unwrap()
                .persistent_copy_uuid
                .unwrap(),
        )
    }
    fn balance(core: &BattleCore, actor: UnitInstanceId) -> u32 {
        balance_owner(core, owner(core, actor))
    }
    fn balance_owner(core: &BattleCore, owner: ResourceOwner) -> u32 {
        let view = core.typed_resources().unwrap();
        let ordinal = view
            .scopes()
            .find(|(_, l, _, _)| *l == crate::game::resource::ResourceLifetime::Run)
            .unwrap()
            .0;
        view.value(&definition().resource, owner, ordinal)
            .unwrap()
            .balance
    }
    fn attack(
        core: &mut BattleCore,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        now: u64,
        regular: bool,
    ) {
        core.units.get_mut(&actor).unwrap().next_basic_attack_ms = now;
        core.process_event(
            BattleEvent::AttackStart {
                time_ms: now,
                attacker_instance_id: actor,
                target_instance_id: Some(target),
                schedule_next: regular,
                cause: Default::default(),
            },
            now,
        )
        .unwrap();
    }
    #[test]
    fn weapon_growth_actual_basic_uses_saved_mass_and_awards_primary_once() {
        let (mut core, mut state, a, b, enemy, run) = fixture(3);
        attack(&mut core, a, enemy, 10, true);
        core.step_battle_execution_until(&mut state, 12).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 870);
        assert_eq!(balance(&core, a), 3);
        core.units.get_mut(&enemy).unwrap().stats.current_health = 100;
        attack(&mut core, a, enemy, 20, true);
        core.step_battle_execution_until(&mut state, 22).unwrap();
        assert!(core.units[&enemy].lifecycle.is_dead());
        assert_eq!(balance(&core, a), 5);
        assert_eq!(balance(&core, b), 0);
        let mut saved = run.clone();
        core.capture_run_resources()
            .unwrap()
            .unwrap()
            .apply(&mut saved)
            .unwrap();
        let roundtrip: RunResourceState =
            ron::de::from_str(&ron::ser::to_string(&saved).unwrap()).unwrap();
        assert_eq!(saved, roundtrip);
        assert_eq!(
            saved
                .entries
                .iter()
                .find(|e| e.owner == owner(&core, a))
                .unwrap()
                .balance,
            5
        );
        assert_eq!(
            run.entries[0].balance, 3,
            "battle overlay cannot mutate retreat savepoint"
        );
    }
    #[test]
    fn weapon_growth_triggered_and_independent_skill_damage_do_not_gain_or_scale() {
        let (mut core, mut state, a, _, enemy, _) = fixture(3);
        attack(&mut core, a, enemy, 10, false);
        core.step_battle_execution_until(&mut state, 12).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 900);
        let snap = core
            .damage_source_snapshot_for_unit(
                a,
                enemy,
                DamageSource::Ability,
                DamageType::True,
                100,
                Default::default(),
                0,
                20,
                false,
            )
            .unwrap();
        assert!(snap.weapon_growth.is_none());
        core.units.get_mut(&enemy).unwrap().stats.current_health = 100;
        crate::game::battle::core::relic_causal_tests::joint_committed_damage(
            &mut core, a, enemy, 100, 20,
        );
        assert_eq!(balance(&core, a), 3);
    }
    #[test]
    fn weapon_growth_secondary_damage_uses_captured_balance_without_award() {
        let (mut core, _, a, b, enemy, _) = fixture(3);
        let mut snap = core
            .damage_source_snapshot_for_unit(
                a,
                enemy,
                DamageSource::BasicAttack,
                DamageType::True,
                100,
                Default::default(),
                0,
                10,
                false,
            )
            .unwrap();
        core.bind_weapon_growth(&mut snap, enemy).unwrap();
        let mut primary = snap.clone();
        core.apply_weapon_growth_damage(&mut primary, enemy)
            .unwrap();
        assert_eq!(primary.base_damage, 130);
        core.apply_weapon_growth_damage(&mut snap, b).unwrap();
        assert_eq!(snap.base_damage, 112);
        core.weapon_growth_kill(&snap, b, 10).unwrap();
        assert_eq!(balance(&core, a), 3);
    }
    #[test]
    fn weapon_growth_classification_excludes_summons_and_saturates() {
        for (class, expected) in [
            (BattleUnitThreatClass::Normal, 2),
            (BattleUnitThreatClass::Elite, 5),
            (BattleUnitThreatClass::Boss, 9),
        ] {
            let (mut core, mut state, a, _, enemy, _) = fixture(0);
            core.units.get_mut(&enemy).unwrap().threat_class = class;
            core.units.get_mut(&enemy).unwrap().stats.current_health = 10;
            attack(&mut core, a, enemy, 10, true);
            core.step_battle_execution_until(&mut state, 12).unwrap();
            assert_eq!(balance(&core, a), expected);
        }
        let (mut core, mut state, a, _, enemy, _) = fixture(u32::MAX - 1);
        attack(&mut core, a, enemy, 10, true);
        core.step_battle_execution_until(&mut state, 12).unwrap();
        assert_eq!(balance(&core, a), u32::MAX);
        let (mut core, mut state, a, _, enemy, _) = fixture(0);
        let base = core.units[&enemy].base_uuid;
        core.units.get_mut(&enemy).unwrap().source_identity =
            crate::game::battle::types::BattleUnitSourceIdentity::SummonedUnit {
                definition_id: "temporary".into(),
                base_uuid: base,
                summoner_instance_id: a,
                summon_sequence: 1,
                batch_uuid: Uuid::from_u128(12),
            };
        core.units.get_mut(&enemy).unwrap().stats.current_health = 10;
        attack(&mut core, a, enemy, 10, true);
        core.step_battle_execution_until(&mut state, 12).unwrap();
        assert_eq!(balance(&core, a), 0);
    }
    #[test]
    fn weapon_growth_saved_copy_survives_new_battle_and_swap_but_new_copy_starts_zero() {
        let (mut core, mut state, a, _, enemy, mut run) = fixture(3);
        core.units.get_mut(&enemy).unwrap().stats.current_health = 1;
        attack(&mut core, a, enemy, 10, true);
        core.step_battle_execution_until(&mut state, 12).unwrap();
        core.capture_run_resources()
            .unwrap()
            .unwrap()
            .apply(&mut run)
            .unwrap();
        let (mut next, _, na, nb, _, _) = fixture(0);
        // Use a fresh native resource runtime, exactly as battle entry does.
        next.runtime_facts.resources = None;
        let owners = next
            .items
            .values()
            .filter_map(|i| i.persistent_copy_uuid)
            .map(ResourceOwner::EquipmentCopy)
            .collect();
        next.mount_run_resources(&run, &owners).unwrap();
        assert_eq!(balance(&next, na), 5);
        assert_eq!(balance(&next, nb), 0);
        let (mut swapped, _, sa, sb, _, _) = fixture_swap(0, true);
        swapped.runtime_facts.resources = None;
        let owners = swapped
            .items
            .values()
            .filter_map(|i| i.persistent_copy_uuid)
            .map(ResourceOwner::EquipmentCopy)
            .collect();
        swapped.mount_run_resources(&run, &owners).unwrap();
        assert_eq!(
            balance(&swapped, sb),
            5,
            "new battle projects transferred actual copies"
        );
        assert_eq!(balance(&swapped, sa), 0);
    }
    #[test]
    fn weapon_growth_actual_area_only_primary_awards_and_all_hits_use_pre_kill_balance() {
        use crate::game::{ability::DeliveryDef, battle::tile_range::TileRangePattern};
        for (primary_hp, secondary_hp, expected_mass, expected_secondary) in
            [(110, 1000, 5, 888), (1000, 110, 3, 0), (110, 110, 5, 0)]
        {
            let (mut core, mut state, a, _, enemy, _) = fixture_options(3, false, true);
            let secondary = core
                .units
                .values()
                .find(|u| u.owner == Side::Opponent && u.instance_id != enemy)
                .unwrap()
                .instance_id;
            crate::game::battle::core::relic_structural_tests::flow_cases::place_stationary(
                &mut core, secondary, 3, 2,
            );
            core.units.get_mut(&enemy).unwrap().stats.current_health = primary_hp;
            core.units.get_mut(&secondary).unwrap().stats.current_health = secondary_hp;
            core.units.get_mut(&a).unwrap().basic_attack.delivery = DeliveryDef::TileArea {
                footprint: TileRangePattern {
                    rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
                    include_anchor_tile: true,
                },
                area: Default::default(),
            };
            attack(&mut core, a, enemy, 10, true);
            core.step_battle_execution_until(&mut state, 12).unwrap();
            assert_eq!(balance(&core, a), expected_mass);
            assert_eq!(
                core.units[&secondary].stats.current_health,
                expected_secondary
            );
        }
    }
    #[test]
    fn weapon_growth_award_fault_restores_hp_death_queue_resource_and_retry_awards_once() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let (mut core, mut state, a, _, enemy, _) = fixture(3);
        core.units.get_mut(&enemy).unwrap().stats.current_health = 10;
        attack(&mut core, a, enemy, 10, true);
        core.step_battle_execution_until(&mut state, 10).unwrap();
        let before = (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            serde_json::to_value(core.runtime_fact_view()).unwrap(),
            format!("{state:?}"),
            balance(&core, a),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("weapon_growth_awarded", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || core.step_battle_execution_until(&mut state, 12),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(
            before,
            (
                core.transaction_debug_snapshot(),
                core.event_queue_debug_snapshot(),
                serde_json::to_value(core.runtime_fact_view()).unwrap(),
                format!("{state:?}"),
                balance(&core, a)
            )
        );
        assert_eq!(core.units[&enemy].stats.current_health, 10);
        core.step_battle_execution_until(&mut state, 12).unwrap();
        assert_eq!(balance(&core, a), 5);
    }
    #[test]
    fn weapon_growth_released_projectile_keeps_copy_and_pre_kill_mass_after_withdrawal() {
        use crate::game::{
            ability::{DeliveryDef, ProjectileHitPolicy},
            battle::core::sim::BattleLiveCommand,
        };
        let (mut core, mut state, a, _, enemy, _) = fixture(3);
        core.units.get_mut(&enemy).unwrap().stats.current_health = 110;
        core.units.get_mut(&a).unwrap().basic_attack.delivery = DeliveryDef::Projectile {
            speed_units_per_ms: 1000,
            hit_policy: ProjectileHitPolicy::TargetLocked,
            allow_targetless_cast: false,
            max_range_tiles: None,
            max_lifetime_ms: None,
            max_kills: None,
            max_pierces: None,
            collision: Default::default(),
        };
        attack(&mut core, a, enemy, 10, true);
        core.step_battle_execution_until(&mut state, 12).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 110);
        assert!(
            !core.projectiles.is_empty(),
            "expected released carrier; events={:?}",
            core.event_log.entries
        );
        let captured_owner = owner(&core, a);
        core.apply_live_command(BattleLiveCommand::WithdrawUnit {
            unit_id: a,
            time_ms: 12,
        })
        .unwrap();
        core.step_battle_execution_until(&mut state, 1200).unwrap();
        assert!(
            core.units[&enemy].lifecycle.is_dead(),
            "remaining HP: {}",
            core.units[&enemy].stats.current_health
        );
        assert_eq!(balance_owner(&core, captured_owner), 5);
    }
}
