use super::*;
use crate::{
    authoring::node_graph::{compile, tests::prepared_basic_fixture, PortableGraph},
    game::{
        ability::{DeliveryDef, ProjectileHitPolicy},
        battle::{
            buffs::BuffId,
            core::{
                relic_causal_tests::joint_committed_damage, sim::BattleExecutionState,
                tests::native_action_fixture,
            },
            runtime_facts::ActionHandle,
            types::BattleUnitSourceIdentity,
        },
    },
};
fn fixture() -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let graph = compile(&prepared_basic_fixture().to_string(), "prepared".into()).unwrap();
    let db = PortableGraph::from_ron(&graph.to_ron().unwrap())
        .unwrap()
        .load_native()
        .unwrap();
    let (mut core, state, actor, enemy, ally) = native_action_fixture(
        db.get_by_id("prepared").unwrap().clone(),
        Default::default(),
    );
    let unit = core.units.get_mut(&actor).unwrap();
    unit.source_identity = BattleUnitSourceIdentity::Employee {
        employee_uuid: actor.as_uuid(),
        base_uuid: unit.base_uuid,
    };
    unit.stats.attack = 100;
    for id in [actor, enemy, ally] {
        let u = core.units.get_mut(&id).unwrap();
        u.stats.current_health = 1000;
        u.stats.max_health = 1000;
    }
    (core, state, actor, enemy, ally)
}
fn prepare(core: &mut BattleCore, actor: UnitInstanceId, time: u64) -> ActionHandle {
    core.units.get_mut(&actor).unwrap().resonance_current = 100;
    core.request_skill_action(time, actor, "prepared".into(), None)
        .unwrap()
}
fn attack(
    core: &mut BattleCore,
    actor: UnitInstanceId,
    target: UnitInstanceId,
    time: u64,
    regular: bool,
) {
    core.units.get_mut(&actor).unwrap().next_basic_attack_ms = time;
    core.process_event(
        BattleEvent::AttackStart {
            time_ms: time,
            attacker_instance_id: actor,
            target_instance_id: Some(target),
            schedule_next: regular,
            cause: Default::default(),
        },
        time,
    )
    .unwrap();
}
fn projectile(core: &mut BattleCore, actor: UnitInstanceId) {
    core.units.get_mut(&actor).unwrap().basic_attack.delivery = DeliveryDef::Projectile {
        speed_units_per_ms: 1000,
        hit_policy: ProjectileHitPolicy::TargetLocked,
        allow_targetless_cast: false,
        max_range_tiles: None,
        max_lifetime_ms: None,
        max_kills: None,
        max_pierces: None,
        collision: Default::default(),
    };
}
#[test]
fn prepared_basic_waits_without_timer_consumes_one_normal_attack_and_preserves_classification() {
    let (mut core, mut state, actor, enemy, _) = fixture();
    let grant = prepare(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 5000).unwrap();
    assert_eq!(
        core.sustained_effect_snapshot()
            .unwrap()
            .prepared_basics
            .len(),
        1
    );
    assert!(!core.resonance_use_has_obligations(actor));
    assert_eq!(core.query_action(&grant).unwrap().facts.resource_spent, 100);
    assert!(core
        .query_action(&grant)
        .unwrap()
        .effects_settled_at_ms
        .is_none());
    attack(&mut core, actor, enemy, 5000, true);
    core.step_battle_execution_until(&mut state, 5201).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 750);
    let damage = core.runtime_fact_summary().latest_damage.as_ref().unwrap();
    assert_ne!(damage.source.action.as_ref(), Some(&grant));
    assert!(damage.source.cast.is_none());
    assert_eq!(damage.modifier_sources.len(), 1);
    assert_eq!(damage.modifier_sources[0].action.as_ref(), Some(&grant));
    assert_eq!(
        damage.modifier_sources[0]
            .cast
            .as_ref()
            .unwrap()
            .skill_id
            .as_str(),
        "prepared"
    );
    assert!(core.sustained.prepared.is_empty());
    let result = core.query_action(&grant).unwrap();
    assert_eq!(result.facts.damage_hp_lost, 0);
    assert!(result.effects_settled_at_ms.is_some());
    assert_eq!(core.units[&actor].stats.attack, 100);
    attack(&mut core, actor, enemy, 5300, true);
    core.step_battle_execution_until(&mut state, 5501).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 650);
}
#[test]
fn prepared_basic_does_not_retrofit_existing_windup() {
    let (mut core, mut state, actor, enemy, _) = fixture();
    core.step_battle_execution_until(&mut state, 0).unwrap();
    attack(&mut core, actor, enemy, 0, true);
    prepare(&mut core, actor, 50);
    core.step_battle_execution_until(&mut state, 201).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 900);
    assert!(core.sustained_effect_snapshot().unwrap().prepared_basics[0]
        .bound_action
        .is_none());
    attack(&mut core, actor, enemy, 250, true);
    core.step_battle_execution_until(&mut state, 451).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 650);
}
#[test]
fn prepared_basic_triggered_attack_does_not_bind_or_consume() {
    let (mut core, mut state, actor, enemy, _) = fixture();
    prepare(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    attack(&mut core, actor, enemy, 10, false);
    core.step_battle_execution_until(&mut state, 211).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 900);
    assert!(core.sustained_effect_snapshot().unwrap().prepared_basics[0]
        .bound_action
        .is_none());
    attack(&mut core, actor, enemy, 300, true);
    core.step_battle_execution_until(&mut state, 501).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 650);
}
#[test]
fn prepared_basic_hard_cc_before_release_restores_preparation() {
    let (mut core, mut state, actor, enemy, _) = fixture();
    prepare(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    attack(&mut core, actor, enemy, 10, true);
    core.process_event(
        BattleEvent::ApplyBuff {
            time_ms: 50,
            caster_instance_id: enemy,
            target_instance_id: actor,
            buff_id: BuffId::from_name("stun"),
            duration_ms: 100,
            cause: Default::default(),
        },
        50,
    )
    .unwrap();
    assert!(core.sustained_effect_snapshot().unwrap().prepared_basics[0]
        .bound_action
        .is_none());
    core.step_battle_execution_until(&mut state, 211).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 1000);
    attack(&mut core, actor, enemy, 250, true);
    core.step_battle_execution_until(&mut state, 451).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 750);
}
#[test]
fn prepared_basic_ready_recast_replaces_but_bound_recast_preserves_old_attack() {
    for bound in [false, true] {
        let (mut core, mut state, actor, enemy, _) = fixture();
        let first = prepare(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        if bound {
            attack(&mut core, actor, enemy, 10, true);
        }
        prepare(&mut core, actor, 50);
        core.step_battle_execution_until(&mut state, 51).unwrap();
        assert_eq!(core.sustained.prepared.len(), if bound { 2 } else { 1 });
        if !bound {
            assert!(core
                .query_action(&first)
                .unwrap()
                .effects_settled_at_ms
                .is_some());
        }
        core.step_battle_execution_until(&mut state, 211).unwrap();
        assert_eq!(
            core.units[&enemy].stats.current_health,
            if bound { 750 } else { 1000 }
        );
        attack(&mut core, actor, enemy, 250, true);
        core.step_battle_execution_until(&mut state, 451).unwrap();
        assert_eq!(
            core.units[&enemy].stats.current_health,
            if bound { 500 } else { 750 }
        );
        assert!(core.sustained.prepared.is_empty());
    }
}
#[test]
fn prepared_basic_direct_kill_grants_timed_attack_state_without_retroactive_damage() {
    let (mut core, mut state, actor, enemy, _) = fixture();
    core.units.get_mut(&enemy).unwrap().stats.current_health = 200;
    let grant = prepare(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    attack(&mut core, actor, enemy, 10, true);
    core.step_battle_execution_until(&mut state, 211).unwrap();
    assert_eq!(core.units[&actor].stats.attack, 125);
    assert_eq!(core.sustained_effect_snapshot().unwrap().active.len(), 1);
    assert!(core.sustained.prepared.is_empty());
    assert!(!core.resonance_use_has_obligations(actor));
    assert!(core
        .query_action(&grant)
        .unwrap()
        .effects_settled_at_ms
        .is_none());
    core.step_battle_execution_until(&mut state, 2211).unwrap();
    assert_eq!(core.units[&actor].stats.attack, 100);
    assert!(core
        .query_action(&grant)
        .unwrap()
        .effects_settled_at_ms
        .is_some());
}
#[test]
fn prepared_basic_projectile_captures_native_release_power_and_survives_caster_death() {
    let (mut core, mut state, actor, enemy, _) = fixture();
    projectile(&mut core, actor);
    prepare(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    attack(&mut core, actor, enemy, 10, true);
    core.units.get_mut(&actor).unwrap().stats.attack = 200;
    core.step_battle_execution_until(&mut state, 211).unwrap();
    assert!(core.sustained_effect_snapshot().unwrap().prepared_basics[0].released);
    assert_eq!(core.units[&enemy].stats.current_health, 1000);
    joint_committed_damage(&mut core, enemy, actor, 1000, 212);
    assert!(core.sustained.prepared.is_empty());
    core.step_battle_execution_until(&mut state, 1300).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 500);
}
#[test]
fn prepared_basic_released_projectile_miss_does_not_refund_or_reward_other_damage() {
    let (mut core, mut state, actor, enemy, ally) = fixture();
    projectile(&mut core, actor);
    let grant = prepare(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    attack(&mut core, actor, enemy, 10, true);
    core.step_battle_execution_until(&mut state, 211).unwrap();
    joint_committed_damage(&mut core, ally, enemy, 1000, 212);
    core.step_battle_execution_until(&mut state, 1300).unwrap();
    assert!(core.sustained.prepared.is_empty());
    assert_eq!(core.units[&actor].stats.attack, 100);
    assert_eq!(core.query_action(&grant).unwrap().facts.resource_spent, 100);
    assert!(core
        .query_action(&grant)
        .unwrap()
        .effects_settled_at_ms
        .is_some());
}
#[test]
fn prepared_basic_source_death_discards_unreleased_preparation() {
    let (mut core, mut state, actor, enemy, _) = fixture();
    let grant = prepare(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    joint_committed_damage(&mut core, enemy, actor, 1000, 2);
    assert!(core.sustained.prepared.is_empty());
    assert!(core
        .query_action(&grant)
        .unwrap()
        .effects_settled_at_ms
        .is_some());
}
#[test]
fn prepared_basic_faults_restore_grant_release_reward_and_delivery_state() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in [
        "prepared_basic_granted",
        "prepared_basic_bound",
        "prepared_basic_released",
        "prepared_basic_rewarded",
        "prepared_basic_ended",
    ] {
        let (mut core, mut state, actor, enemy, _) = fixture();
        core.units.get_mut(&enemy).unwrap().stats.current_health = 200;
        prepare(&mut core, actor, 0);
        let time = if site == "prepared_basic_granted" {
            1
        } else {
            core.step_battle_execution_until(&mut state, 1).unwrap();
            core.units.get_mut(&actor).unwrap().next_basic_attack_ms = 10;
            core.event_queue.push(BattleEvent::AttackStart {
                time_ms: 10,
                attacker_instance_id: actor,
                target_instance_id: Some(enemy),
                schedule_next: true,
                cause: Default::default(),
            });
            if site == "prepared_basic_bound" {
                10
            } else {
                210
            }
        };
        core.step_battle_execution_until(&mut state, time - 1)
            .unwrap();
        let before = (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            serde_json::to_value(core.runtime_fact_view()).unwrap(),
            format!("{state:?}"),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || core.step_battle_execution_until(&mut state, time),
            )
        });
        assert!(
            token.is_cancelled() && result.is_err(),
            "{site}: {result:?}"
        );
        assert_eq!(
            (
                core.transaction_debug_snapshot(),
                core.event_queue_debug_snapshot(),
                serde_json::to_value(core.runtime_fact_view()).unwrap(),
                format!("{state:?}")
            ),
            before,
            "{site}"
        );
        core.step_battle_execution_until(&mut state, 211).unwrap();
        if site != "prepared_basic_granted" {
            assert_eq!(core.units[&actor].stats.attack, 125);
        }
    }
}

#[test]
fn prepared_basic_area_multiple_kills_grant_exactly_one_reward_without_resampling_power() {
    use crate::game::{
        battle::{core::movement::types::WorldVec2, tile_range::TileRangePattern},
        enums::Side,
        resources::Position,
    };
    let (mut core, mut state, actor, enemy, ally) = fixture();
    core.units.get_mut(&actor).unwrap().basic_attack.delivery = DeliveryDef::TileArea {
        footprint: TileRangePattern {
            rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
            include_anchor_tile: true,
        },
        area: Default::default(),
    };
    core.units.get_mut(&ally).unwrap().owner = Side::Opponent;
    core.units
        .get_mut(&ally)
        .unwrap()
        .set_world_position(WorldVec2::from_tile_center(Position::new(2, 2)));
    for id in [enemy, ally] {
        core.units.get_mut(&id).unwrap().stats.current_health = 240;
    }
    prepare(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    attack(&mut core, actor, enemy, 10, true);
    core.step_battle_execution_until(&mut state, 211).unwrap();
    let view = core.sustained_effect_snapshot().unwrap();
    assert_eq!(view.started, 2); // One preparation plus one reward, despite two kills.
    assert_eq!(view.active.len(), 1);
    assert_eq!(core.units[&actor].stats.attack, 125);
    assert!(!core.units.get(&enemy).is_some_and(|u| u.is_active()));
    assert!(!core.units.get(&ally).is_some_and(|u| u.is_active()));
}
#[test]
fn prepared_basic_withdrawal_and_battle_terminal_close_event_owned_preparation() {
    use crate::game::battle::{core::sim::BattleLiveCommand, scenario::BattleScenario};
    let (mut core, mut state, actor, _, _) = fixture();
    let grant = prepare(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 50).unwrap();
    core.apply_live_command(BattleLiveCommand::WithdrawUnit {
        unit_id: actor,
        time_ms: 50,
    })
    .unwrap();
    assert!(core.sustained.prepared.is_empty());
    assert!(core
        .query_action(&grant)
        .unwrap()
        .effects_settled_at_ms
        .is_some());
    let (mut core, mut state, actor, _, _) = fixture();
    Arc::make_mut(&mut core.scenario).terminal_outcomes =
        BattleScenario::fixture_survival_outcomes(50);
    prepare(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 51).unwrap();
    assert!(state.is_finished());
    let result = core.finalize_battle_execution(&mut state).unwrap();
    assert!(core.sustained.prepared.is_empty());
    assert_eq!(
        result.runtime_facts.view().usage().outstanding_deliveries,
        0
    );
}

#[test]
fn prepared_basic_equipment_bonus_finisher_is_excluded_after_shield_absorption() {
    use crate::game::{
        battle::damage::DamageType,
        data::{
            equipment_data::{EquipmentMetadata, EquipmentType},
            GameDataBuilder,
        },
        enums::{RiskLevel, Side},
        stats::{Effect, TriggerEffectTarget, TriggerType, TriggeredEffect},
    };
    for (hp, shield, expected_attack) in [(240, 0, 125), (300, 0, 100), (240, 50, 100)] {
        let (mut core, mut state, actor, enemy, _) = fixture();
        let id = uuid::Uuid::from_u128(0x9941);
        let item = EquipmentMetadata {
            content_family: None,
            id: "prepared-bonus".into(),
            uuid: id,
            name: "Bonus fixture".into(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            allow_duplicate_equip: true,
            bound: false,
            cannot_unequip_reason: "equipment_bound".into(),
            triggered_effects: std::collections::HashMap::from([(
                TriggerType::OnAttack,
                vec![TriggeredEffect::targeted(
                    TriggerEffectTarget::CounterpartUnit,
                    Effect::BonusDamage {
                        flat: 100,
                        percent: 0,
                        damage_type: DamageType::True,
                    },
                )],
            )]),
            ability_activations: vec![],
            weapon_profile: Some(Default::default()),
            event_check_modifiers: vec![],
            event_choice_keys: vec![],
        };
        core.game_data = GameDataBuilder::from_validated(&core.game_data.clone().into())
            .with_equipment(vec![item])
            .build_arc();
        core.items.insert(
            id,
            crate::game::battle::core::RuntimeItem {
            weapon_stage: Default::default(),
                instance_id: id,
                base_uuid: id,
                owner: Side::Player,
                owner_unit_instance: actor,
                persistent_copy_uuid: None,
            },
        );
        core.units.get_mut(&enemy).unwrap().stats.current_health = hp;
        if shield > 0 {
            core.begin_sustained(
                enemy,
                enemy,
                SustainedEffectDef::Shield {
                    duration_ms: 1000,
                    amount: shield,
                    priority: 0,
                    shared_radius_units: None,
                },
                0,
            )
            .unwrap();
        }
        prepare(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        attack(&mut core, actor, enemy, 10, true);
        core.step_battle_execution_until(&mut state, 211).unwrap();
        assert!(!core.units.get(&enemy).is_some_and(|u| u.is_active()));
        let damage = core.runtime_fact_summary().latest_damage.as_ref().unwrap();
        assert_eq!(
            damage
                .segments
                .iter()
                .map(|s| s.raw_damage)
                .collect::<Vec<_>>(),
            vec![250, 100]
        );
        assert_eq!(
            core.units[&actor].stats.attack, expected_attack,
            "hp={hp}, shield={shield}"
        );
    }
}
#[test]
fn prepared_basic_superseded_cancelled_windup_does_not_resurrect_old_charge() {
    let (mut core, mut state, actor, enemy, _) = fixture();
    let old = prepare(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    attack(&mut core, actor, enemy, 10, true);
    prepare(&mut core, actor, 50);
    core.step_battle_execution_until(&mut state, 51).unwrap();
    core.process_event(
        BattleEvent::ApplyBuff {
            time_ms: 60,
            caster_instance_id: enemy,
            target_instance_id: actor,
            buff_id: BuffId::from_name("stun"),
            duration_ms: 100,
            cause: Default::default(),
        },
        60,
    )
    .unwrap();
    assert_eq!(core.sustained.prepared.len(), 1);
    assert!(core
        .query_action(&old)
        .unwrap()
        .effects_settled_at_ms
        .is_some());
    core.step_battle_execution_until(&mut state, 211).unwrap();
    attack(&mut core, actor, enemy, 250, true);
    core.step_battle_execution_until(&mut state, 451).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 750);
    assert!(core.sustained.prepared.is_empty());
}
