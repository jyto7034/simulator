use super::*;
use crate::{
    authoring::node_graph::{compile, tests::reactive_heal_fixture, PortableGraph},
    game::{
        battle::{
            core::{sim::BattleExecutionState, tests::native_action_fixture},
            event_log::{BattleEventCause, SkillCastTarget},
            types::BattleUnitSourceIdentity,
        },
        data::{skill_data::SkillDatabase, GameDataBuilder},
    },
};
const ATTACK: &str = r#"(id:"enemy_combo", name:"", kind:Targeted, use_lifetime:Action,
 cast_targeting:Explicit(selection:automatic_unit,target:EnemySingle(rule:Nearest),reach:global,air_capable:true),
 steps:[(id:"hit",delay_ms:0,target:CastTarget,delivery:Instant,effects:[
 (id:"first",op:Damage(amount:100,damage_type:True)),(id:"second",op:Damage(amount:100,damage_type:True))])])"#;
fn fixture() -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let graph = compile(&reactive_heal_fixture().to_string(), "reactive".into()).unwrap();
    let db = PortableGraph::from_ron(&graph.to_ron().unwrap())
        .unwrap()
        .load_native()
        .unwrap();
    let skill = db.get_by_id("reactive").unwrap().clone();
    let (mut core, state, actor, enemy, ally) =
        native_action_fixture(skill.clone(), Default::default());
    let attack: crate::game::ability::SkillDef = ron::de::from_str(ATTACK).unwrap();
    core.game_data = GameDataBuilder::live_defaults()
        .with_abnormalities(vec![])
        .with_skills(SkillDatabase::new(vec![skill, attack]))
        .build_arc();
    for id in [actor, ally] {
        let unit = core.units.get_mut(&id).unwrap();
        unit.source_identity = BattleUnitSourceIdentity::Employee {
            employee_uuid: id.as_uuid(),
            base_uuid: unit.base_uuid,
        };
        unit.employee_trauma = Some(0);
        unit.stats.max_health = 1000;
        unit.stats.current_health = 600;
    }
    (core, state, actor, enemy, ally)
}
fn install(
    core: &mut BattleCore,
    state: &mut BattleExecutionState,
    actor: UnitInstanceId,
    target: UnitInstanceId,
) -> crate::game::battle::runtime_facts::ActionHandle {
    let action = core
        .request_skill_action(
            0,
            actor,
            "reactive".into(),
            Some(SkillCastTarget::Unit {
                unit_instance_id: target,
            }),
        )
        .unwrap();
    core.step_battle_execution_until(state, 1).unwrap();
    assert_eq!(core.sustained_effect_snapshot().unwrap().active.len(), 1);
    action
}
fn attack(
    core: &mut BattleCore,
    state: &mut BattleExecutionState,
    enemy: UnitInstanceId,
    target: UnitInstanceId,
    now: u64,
) {
    enqueue_attack(core, enemy, target, now);
    core.step_battle_execution_until(state, now).unwrap();
}
fn enqueue_attack(core: &mut BattleCore, enemy: UnitInstanceId, target: UnitInstanceId, now: u64) {
    core.runtime_transaction(|core| {
        core.invoke_ability(
            now,
            enemy,
            "enemy_combo",
            Some(SkillCastTarget::Unit {
                unit_instance_id: target,
            }),
            BattleEventCause::default(),
            false,
        );
        runtime_control::poll()
    })
    .unwrap();
}
#[test]
fn reactive_heal_live_graph_consumes_once_per_hostile_action_and_depletes() {
    let (mut core, mut state, actor, enemy, ally) = fixture();
    install(&mut core, &mut state, actor, ally);
    attack(&mut core, &mut state, enemy, ally, 10);
    assert_eq!(core.units[&ally].stats.current_health, 600);
    let view = core.sustained_effect_snapshot().unwrap().active[0]
        .reactive_heal
        .clone()
        .unwrap();
    assert_eq!((view.remaining_charges, view.consumed_actions), (1, 1));
    attack(&mut core, &mut state, enemy, ally, 20);
    assert_eq!(core.units[&ally].stats.current_health, 600);
    assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
    assert!(!core.resonance_use_has_obligations(actor));
    attack(&mut core, &mut state, enemy, ally, 30);
    assert_eq!(core.units[&ally].stats.current_health, 400);
}
#[test]
fn reactive_heal_rejects_enemy_selection_without_cost() {
    let (mut core, _, actor, enemy, _) = fixture();
    let before = core.transaction_debug_snapshot();
    assert!(core
        .request_skill_action(
            0,
            actor,
            "reactive".into(),
            Some(SkillCastTarget::Unit {
                unit_instance_id: enemy
            })
        )
        .is_err());
    assert_eq!(core.transaction_debug_snapshot(), before);
}
#[test]
fn reactive_heal_survives_caster_death_and_does_not_resurrect_target() {
    let (mut core, mut state, actor, enemy, ally) = fixture();
    let action = install(&mut core, &mut state, actor, ally);
    core.units.get_mut(&actor).unwrap().stats.current_health = 50;
    attack(&mut core, &mut state, enemy, actor, 10);
    assert!(!core.units[&actor].is_active());
    assert_eq!(core.sustained_effect_snapshot().unwrap().active.len(), 1);
    attack(&mut core, &mut state, enemy, ally, 20);
    assert_eq!(core.units[&ally].stats.current_health, 600);
    let action_view = core.query_action(&action).unwrap();
    let heal = action_view.facts.latest_nonzero_heal.as_ref().unwrap();
    assert_eq!(heal.source.action.as_ref(), Some(&action));
    assert_eq!((heal.target, heal.actual_delta()), (ally, 200));
    core.units.get_mut(&ally).unwrap().stats.current_health = 50;
    attack(&mut core, &mut state, enemy, ally, 30);
    assert!(!core.units[&ally].is_active());
    assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
}

fn remaining(core: &BattleCore) -> u32 {
    core.sustained_effect_snapshot()
        .unwrap()
        .active
        .iter()
        .find_map(|r| r.reactive_heal.as_ref().map(|h| h.remaining_charges))
        .unwrap()
}
#[test]
fn reactive_heal_ignores_absorbed_damage_health_cost_and_friendly_actions() {
    use crate::game::ability::sustained::SustainedEffectDef;
    let (mut core, mut state, actor, enemy, ally) = fixture();
    install(&mut core, &mut state, actor, ally);
    core.begin_sustained(
        actor,
        ally,
        SustainedEffectDef::Shield {
            duration_ms: 1000,
            amount: 200,
            priority: 0,
            shared_radius_units: None,
        },
        1,
    )
    .unwrap();
    attack(&mut core, &mut state, enemy, ally, 10);
    assert_eq!(core.units[&ally].stats.current_health, 600);
    assert_eq!(remaining(&core), 2);
    core.runtime_transaction(|core| {
        core.process_commands(
            vec![BattleCommand::PayHealthCost {
                caster_id: ally,
                amount: 100,
                allow_lethal: false,
            }],
            11,
        );
        runtime_control::poll()
    })
    .unwrap();
    assert_eq!(core.units[&ally].stats.current_health, 500);
    assert_eq!(remaining(&core), 2);
    // Preserve an actual attack action, then make its source allied before impact.
    enqueue_attack(&mut core, enemy, ally, 20);
    core.units.get_mut(&enemy).unwrap().owner = crate::game::enums::Side::Player;
    core.step_battle_execution_until(&mut state, 20).unwrap();
    assert_eq!(remaining(&core), 2);
    assert_eq!(core.units[&ally].stats.current_health, 300);
}
#[test]
fn reactive_heal_expires_without_hit_and_unlocks_use() {
    let (mut core, mut state, actor, enemy, ally) = fixture();
    install(&mut core, &mut state, actor, ally);
    core.step_battle_execution_until(&mut state, 2000).unwrap();
    assert_eq!(remaining(&core), 2);
    assert!(core.resonance_use_has_obligations(actor));
    core.step_battle_execution_until(&mut state, 2001).unwrap();
    assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
    assert!(!core.resonance_use_has_obligations(actor));
    attack(&mut core, &mut state, enemy, ally, 2010);
    assert_eq!(core.units[&ally].stats.current_health, 400);
}
#[test]
fn reactive_heal_same_copy_replaces_timer_other_caster_stacks() {
    use crate::game::ability::sustained::SustainedEffectDef;
    let (mut core, mut state, actor, enemy, ally) = fixture();
    let definition = |duration_ms, charges| SustainedEffectDef::ReactiveHeal {
        duration_ms,
        profile: ReactiveHealProfile {
            definition: "same".into(),
            charges,
            max_health_percent: 10,
        },
    };
    core.begin_sustained(actor, ally, definition(1000, 1), 0)
        .unwrap();
    core.begin_sustained(actor, ally, definition(2000, 3), 500)
        .unwrap();
    assert_eq!(core.sustained_effect_snapshot().unwrap().active.len(), 1);
    assert_eq!(remaining(&core), 3);
    core.begin_sustained(ally, ally, definition(2000, 3), 500)
        .unwrap();
    assert_eq!(core.sustained_effect_snapshot().unwrap().active.len(), 2);
    core.step_battle_execution_until(&mut state, 1000).unwrap();
    assert_eq!(core.sustained_effect_snapshot().unwrap().active.len(), 2);
    attack(&mut core, &mut state, enemy, ally, 1010);
    assert_eq!(core.units[&ally].stats.current_health, 600);
    for effect in core.sustained_effect_snapshot().unwrap().active {
        assert_eq!(effect.reactive_heal.unwrap().remaining_charges, 2);
    }
    core.step_battle_execution_until(&mut state, 2500).unwrap();
    assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
}
#[test]
fn reactive_heal_rejects_non_employee_and_out_of_range_without_cost() {
    for invalid in ["non_employee", "outside_range"] {
        let (mut core, _, actor, _, ally) = fixture();
        if invalid == "non_employee" {
            core.units.get_mut(&ally).unwrap().employee_trauma = None;
        } else {
            // The opposite board corner lies outside the authored 7x7 pattern.
            core.battlefield
                .ensure_walkable_tile(crate::game::resources::Position::new(0, 0))
                .unwrap();
            core.units.get_mut(&actor).unwrap().set_world_position(
                crate::game::battle::core::movement::types::WorldVec2::new(0.0, 0.0),
            );
        }
        let before = core.transaction_debug_snapshot();
        assert!(
            core.request_skill_action(
                0,
                actor,
                "reactive".into(),
                Some(SkillCastTarget::Unit {
                    unit_instance_id: ally,
                })
            )
            .is_err(),
            "{invalid}"
        );
        assert_eq!(core.transaction_debug_snapshot(), before, "{invalid}");
    }
}
#[test]
fn reactive_heal_consume_and_heal_faults_rollback_damage_charge_and_retry() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in ["reactive_heal_consumed", "reactive_heal_applied"] {
        let (mut core, mut state, actor, enemy, ally) = fixture();
        install(&mut core, &mut state, actor, ally);
        enqueue_attack(&mut core, enemy, ally, 10);
        let before = (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            format!("{state:?}"),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || core.step_battle_execution_until(&mut state, 10),
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
                format!("{state:?}")
            ),
            before,
            "{site}"
        );
        core.step_battle_execution_until(&mut state, 10).unwrap();
        assert_eq!(core.units[&ally].stats.current_health, 600);
        assert_eq!(remaining(&core), 1);
    }
}
