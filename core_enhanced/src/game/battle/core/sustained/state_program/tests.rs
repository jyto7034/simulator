use super::*;
use crate::{
    authoring::composition::{compile_json, compile_yaml, parse_yaml, Artifact},
    game::{
        battle::{
            core::{sim::BattleExecutionState, tests::native_action_fixture},
            event_log::{BattleEventCause, SkillCastTarget},
            types::BattleUnitSourceIdentity,
        },
        data::{skill_data::SkillDatabase, GameDataBuilder},
    },
};
use serde_json::{json, Value};
const YAML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/skill_composition/state_events.skill.yaml"
));
const ATTACK: &str = r#"(id:"enemy_combo",name:"",kind:Targeted,use_lifetime:Action,
cast_targeting:Explicit(selection:automatic_unit,target:EnemySingle(rule:Nearest),reach:global,air_capable:true),
steps:[(id:"hit",delay_ms:0,target:CastTarget,delivery:Instant,effects:[
(id:"first",op:Damage(amount:100,damage_type:True)),(id:"second",op:Damage(amount:100,damage_type:True))])])"#;
type Fixture = (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
);
fn fixture(artifact: Artifact) -> Fixture {
    let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    let db = artifact.load_native().unwrap();
    let skill = db.get_by_id("authored-state").unwrap().clone();
    let (mut core, state, actor, enemy, ally) =
        native_action_fixture(skill.clone(), Default::default());
    let attack = ron::de::from_str(ATTACK).unwrap();
    // Merging the fixture's enemy skill must preserve the compiled author's
    // provenance, as the pack publication adapter does for catalog merges.
    let authored_file = skill.steps[0].effects[0]
        .source()
        .unwrap()
        .source_file
        .clone();
    let mut merged = SkillDatabase::new(vec![skill, attack]);
    merged.bind_definition_sources(|index, _| {
        if index == 0 {
            (authored_file.clone(), "skills[0]".into())
        } else {
            ("fixture:enemy".into(), "skills[0]".into())
        }
    });
    core.game_data = GameDataBuilder::live_defaults()
        .with_abnormalities(vec![])
        .with_skills(merged)
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
            "authored-state".into(),
            Some(SkillCastTarget::Unit {
                unit_instance_id: target,
            }),
        )
        .unwrap();
    core.step_battle_execution_until(state, 1).unwrap();
    assert_eq!(states(core).len(), 1);
    action
}
fn states(core: &BattleCore) -> Vec<StateView> {
    core.sustained_effect_snapshot()
        .unwrap()
        .active
        .into_iter()
        .filter_map(|s| s.state)
        .collect()
}
fn attack(
    core: &mut BattleCore,
    state: &mut BattleExecutionState,
    enemy: UnitInstanceId,
    target: UnitInstanceId,
    now: u64,
) -> Result<(), GameError> {
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
    })?;
    core.step_battle_execution_until(state, now).map(|_| ())
}
fn program(doc: &mut Value) -> &mut Value {
    &mut doc["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]["State"]
        ["program"]
}
fn edited(edit: impl FnOnce(&mut Value)) -> Artifact {
    let mut doc = parse_yaml(YAML).unwrap();
    edit(program(&mut doc));
    compile_json(&doc.to_string()).unwrap()
}
#[test]
fn authored_state_yaml_two_actions_consume_fields_and_heal_once_each() {
    let artifact = compile_yaml(YAML).unwrap();
    let (mut core, mut state, actor, enemy, ally) = fixture(artifact.clone());
    let action = install(&mut core, &mut state, actor, ally);
    attack(&mut core, &mut state, enemy, ally, 10).unwrap();
    assert_eq!(core.units[&ally].stats.current_health, 600); // 600-100+200-100
    assert_eq!(states(&core)[0].fields[0].value, StateScalar::Integer(1));
    assert_eq!(states(&core)[0].events, 1);
    let view = core.query_action(&action).unwrap();
    let heal = view.facts.latest_nonzero_heal.as_ref().unwrap();
    assert_eq!(heal.actual_delta(), 200);
    let authored = heal.source.authored.as_ref().unwrap();
    assert!(authored.field_path.contains(".op.Sustained.definition.State.program.handlers[0].body"), "{authored:?}");
    assert!(artifact.origin_for(authored).is_some(), "{authored:?}");
    attack(&mut core, &mut state, enemy, ally, 20).unwrap();
    assert_eq!(core.units[&ally].stats.current_health, 600);
    assert!(states(&core).is_empty());
}
#[test]
fn authored_state_recipient_lifetime_survives_source_death_but_not_target_death() {
    let (mut core, mut state, actor, enemy, ally) = fixture(compile_yaml(YAML).unwrap());
    install(&mut core, &mut state, actor, ally);
    core.units.get_mut(&actor).unwrap().stats.current_health = 50;
    attack(&mut core, &mut state, enemy, actor, 10).unwrap();
    assert!(!core.units[&actor].is_active());
    attack(&mut core, &mut state, enemy, ally, 20).unwrap();
    assert_eq!(core.units[&ally].stats.current_health, 600);
    core.units.get_mut(&ally).unwrap().stats.current_health = 50;
    attack(&mut core, &mut state, enemy, ally, 30).unwrap();
    assert!(!core.units[&ally].is_active());
    assert!(states(&core).is_empty());
}
#[test]
fn authored_state_absorption_and_health_cost_do_not_emit_hp_loss_event() {
    let (mut core, mut state, actor, enemy, ally) = fixture(compile_yaml(YAML).unwrap());
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
    attack(&mut core, &mut state, enemy, ally, 10).unwrap();
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
    assert_eq!(states(&core)[0].fields[0].value, StateScalar::Integer(2));
    assert_eq!(states(&core)[0].events, 0);
}
#[test]
fn authored_state_same_engine_runs_damage_accumulation_and_boolean_gate() {
    let artifact = edited(|p| {
        p["id"] = json!("threshold-aid");
        p["fields"] = json!([
            {"name":"absorbed","value":{"Integer":{"unit":"Magnitude","initial":0,"min":0,"max":1000}}},
            {"name":"armed","value":{"Bool":true}}
        ]);
        p["handlers"][0]["event"]["HpLost"]["once_per_action"] = json!(false);
        p["handlers"][0]["body"] = json!([
            {"Set":{"field":"absorbed","value":{"moment":"Live","root":{"Add":[{"Read":{"StateField":{"name":"absorbed","unit":"Magnitude"}}},{"Read":"EventHpLost"}]}}}},
            {"If":{"condition":{"Compare":{"left":{"moment":"Live","root":{"Read":{"StateField":{"name":"absorbed","unit":"Magnitude"}}}},"comparison":"AtLeast","right":{"moment":"Live","root":{"Const":{"value":200,"unit":"Magnitude"}}}}},"then_ops":[
                {"If":{"condition":{"Bool":{"field":"armed","equals":true}},"then_ops":[{"SetBool":{"field":"armed","value":false}},{"HealOwner":{"amount":{"moment":"Live","root":{"Const":{"value":150,"unit":"Magnitude"}}}}}],"else_ops":[]}}
            ],"else_ops":[]}}
        ]);
    });
    let (mut core, mut state, actor, enemy, ally) = fixture(artifact);
    install(&mut core, &mut state, actor, ally);
    attack(&mut core, &mut state, enemy, ally, 10).unwrap();
    assert_eq!(core.units[&ally].stats.current_health, 550); // 600-200+150
    assert_eq!(states(&core)[0].fields[0].value, StateScalar::Integer(200));
    assert_eq!(states(&core)[0].fields[1].value, StateScalar::Bool(false));
    attack(&mut core, &mut state, enemy, ally, 20).unwrap();
    assert_eq!(core.units[&ally].stats.current_health, 350); // disarmed: no second heal
    assert_eq!(states(&core)[0].fields[0].value, StateScalar::Integer(400));
}
#[test]
fn authored_state_expiry_handler_is_not_external_termination() {
    let artifact = edited(|p| {
        p["handlers"] = json!([{ "id":"natural", "event":"Expired", "body":[{"HealOwner":{"amount":{"moment":"Live","root":{"Const":{"value":75,"unit":"Magnitude"}}}}},"End"] }]);
    });
    let (mut core, mut state, actor, _, ally) = fixture(artifact.clone());
    install(&mut core, &mut state, actor, ally);
    core.step_battle_execution_until(&mut state, 2001).unwrap();
    assert_eq!(core.units[&ally].stats.current_health, 675);
    assert!(states(&core).is_empty());
    let (mut core, mut state, actor, _, ally) = fixture(artifact);
    install(&mut core, &mut state, actor, ally);
    let id = core.sustained_effect_snapshot().unwrap().active[0].activation;
    core.end_sustained(id, 20, SustainedEndReason::Stopped)
        .unwrap();
    core.step_battle_execution_until(&mut state, 2001).unwrap();
    assert_eq!(core.units[&ally].stats.current_health, 600);
    assert!(states(&core).is_empty());
}
#[test]
fn authored_state_bad_types_fields_and_event_reads_fail_compilation() {
    for (needle, replacement) in [
        ("name: remaining\n", "name: renamed\n"),
        ("unit: Count", "unit: Magnitude"),
        ("moment: Live", "moment: Snapshot"),
    ] {
        let modified = YAML.replacen(needle, replacement, 1);
        assert_ne!(modified, YAML);
        assert!(compile_yaml(&modified).is_err(), "{needle}");
    }
    let mut doc = parse_yaml(YAML).unwrap();
    program(&mut doc)["handlers"][0]["event"] = json!("Expired");
    program(&mut doc)["handlers"][0]["body"] =
        json!([{ "HealOwner":{"amount":{"moment":"Live","root":{"Read":"EventHpLost"}}} }]);
    assert!(compile_json(&doc.to_string()).is_err());
}
#[test]
fn authored_state_runtime_field_overflow_is_a_latched_fault() {
    let artifact = edited(|p| {
        p["handlers"][0]["body"][0]["Set"]["value"]["root"] =
            json!({"Const":{"value":3,"unit":"Count"}});
    });
    let (mut core, mut state, actor, enemy, ally) = fixture(artifact);
    install(&mut core, &mut state, actor, ally);
    let error = attack(&mut core, &mut state, enemy, ally, 10).unwrap_err();
    assert!(
        matches!(error, GameError::ValueExpressionFault(_)),
        "{error:?}"
    );
    let GameError::ValueExpressionFault(fault) = &error else {
        panic!()
    };
    assert_eq!(fault.site.action.definition_id.as_str(), "authored-state");
    assert_ne!(fault.site.action.action_id.as_str(), "reaction");
    assert!(core.step_battle_execution_until(&mut state, 20).is_err());
}

#[test]
fn authored_state_replacement_discards_old_expiry_and_other_provider_is_independent() {
    let artifact = compile_yaml(YAML).unwrap();
    let skill = artifact
        .load_native()
        .unwrap()
        .get_by_id("authored-state")
        .unwrap()
        .clone();
    let crate::game::ability::SkillEffectDef::Sustained { definition } =
        &skill.steps[0].effects[0].op
    else {
        panic!()
    };
    let (mut core, mut state, actor, enemy, ally) = fixture(artifact);
    install(&mut core, &mut state, actor, ally);
    attack(&mut core, &mut state, enemy, ally, 10).unwrap();
    let saved_source = core.runtime_fact_source.clone();
    core.runtime_fact_source = core
        .sustained
        .records
        .values()
        .find(|r| r.state.is_some())
        .unwrap()
        .source
        .observation();
    core.begin_sustained(actor, ally, definition.clone(), 1000)
        .unwrap();
    assert_eq!(states(&core).len(), 1);
    assert_eq!(states(&core)[0].fields[0].value, StateScalar::Integer(2));
    core.begin_sustained(ally, ally, definition.clone(), 1000)
        .unwrap();
    core.runtime_fact_source = saved_source;
    assert_eq!(states(&core).len(), 2);
    core.step_battle_execution_until(&mut state, 2001).unwrap();
    assert_eq!(states(&core).len(), 2); // old generation's deadline did not expire either
    attack(&mut core, &mut state, enemy, ally, 2010).unwrap();
    assert_eq!(core.units[&ally].stats.current_health, 800); // two independent 200 heals, 200 damage
    assert!(states(&core)
        .iter()
        .all(|s| s.fields[0].value == StateScalar::Integer(1)));
    core.step_battle_execution_until(&mut state, 3000).unwrap();
    assert!(states(&core).is_empty());
}

#[test]
fn authored_state_independent_lease_does_not_keep_the_use_locked() {
    let artifact = edited(|p| p["owns_use"] = json!(false));
    let (mut core, mut state, actor, enemy, ally) = fixture(artifact);
    install(&mut core, &mut state, actor, ally);
    assert!(!core.resonance_use_has_obligations(actor));
    assert_eq!(states(&core).len(), 1);
    attack(&mut core, &mut state, enemy, ally, 10).unwrap();
    assert_eq!(core.units[&ally].stats.current_health, 600);
}

#[test]
fn authored_state_source_exit_policy_and_friendly_filter_are_explicit() {
    let artifact = edited(|p| p["end_on_source_exit"] = json!(true));
    let (mut core, mut state, actor, enemy, ally) = fixture(artifact);
    install(&mut core, &mut state, actor, ally);
    core.units.get_mut(&actor).unwrap().stats.current_health = 50;
    attack(&mut core, &mut state, enemy, actor, 10).unwrap();
    assert!(states(&core).is_empty());
    let (mut core, mut state, actor, enemy, ally) = fixture(compile_yaml(YAML).unwrap());
    install(&mut core, &mut state, actor, ally);
    core.runtime_transaction(|core| {
        core.invoke_ability(
            10,
            enemy,
            "enemy_combo",
            Some(SkillCastTarget::Unit {
                unit_instance_id: ally,
            }),
            BattleEventCause::default(),
            false,
        );
        runtime_control::poll()
    })
    .unwrap();
    core.units.get_mut(&enemy).unwrap().owner = crate::game::enums::Side::Player;
    core.step_battle_execution_until(&mut state, 10).unwrap();
    assert_eq!(states(&core)[0].events, 0);
    assert_eq!(core.units[&ally].stats.current_health, 400);
}

#[test]
fn authored_state_expression_fault_preserves_first_failure_and_releases_operation_scope() {
    let program: StateProgram =
        serde_json::from_value(program(&mut parse_yaml(YAML).unwrap()).clone()).unwrap();
    let compiled = program.compile().unwrap();
    let CompiledStateOp::Set { value, .. } = &compiled.handlers[0].body[0] else {
        panic!()
    };
    let error = runtime_control::run(Default::default(), || {
        let _ = field_fault(value, "original state fault");
        let _ = runtime_control::invariant("later_native_fault");
        runtime_control::charge(1, "must_remain_failed")
    })
    .unwrap_err();
    assert!(
        matches!(error, GameError::ValueExpressionFault(ref e) if e.cause == "original state fault")
    );
    runtime_control::run(Default::default(), || {
        runtime_control::charge(1, "independent_operation")
    })
    .unwrap();
    let error = runtime_control::run(Default::default(), || {
        let _ = runtime_control::invariant("first_native_fault");
        let _ = field_fault(value, "later state fault");
        runtime_control::poll()
    })
    .unwrap_err();
    assert!(
        matches!(error, GameError::BattleRuntimeFault(ref e) if e.site == "first_native_fault")
    );
}

#[test]
fn authored_state_catalog_uses_selected_limits_and_activation_shares_published_plan() {
    let artifact = compile_yaml(YAML).unwrap();
    let db = artifact.load_native().unwrap();
    let resources = crate::game::resource::ResourceCatalog { resources: vec![] };
    let mut limits = expr::ExprLimits::default();
    limits.max_nodes = 1;
    assert!(expr::ValueExprCatalog::compile_skills(&db, &resources, &limits).is_err());
    limits = expr::ExprLimits::default();
    limits.max_compile_work = 1;
    assert!(expr::ValueExprCatalog::compile_skills(&db, &resources, &limits).is_err());
    let (mut core, mut state, actor, _, ally) = fixture(artifact);
    let compiled = core
        .game_data
        .value_expr_catalog()
        .states()
        .values()
        .next()
        .unwrap()
        .clone();
    assert!(core
        .game_data
        .value_expr_catalog()
        .contains_authored_expressions());
    install(&mut core, &mut state, actor, ally);
    let active = core
        .sustained
        .records
        .values()
        .find_map(|r| r.state.as_ref())
        .unwrap();
    assert!(Arc::ptr_eq(&compiled, &active.program));
}

#[test]
fn composition_prepared_state_shared_event_history_counts_toward_live_entity_budget() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    let artifact = edited(|p| {
        p["handlers"].as_array_mut().unwrap().truncate(1);
        p["handlers"][0]["body"].as_array_mut().unwrap().truncate(1);
    });
    let (mut core, mut state, actor, enemy, ally) = fixture(artifact);
    install(&mut core, &mut state, actor, ally);
    let query = |core: &BattleCore, limit| {
        control::run(
            &ExecutionLimits {
                max_retained_entities: limit,
                ..Default::default()
            },
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || core.admit_authoring_runtime_growth(0, "authored_state_history_bound"),
        )
    };
    let (result, usage) = query(&core, 65536).unwrap();
    result.unwrap();
    let before = usage.retained_entities_peak;
    attack(&mut core, &mut state, enemy, ally, 10).unwrap();
    assert_eq!(states(&core)[0].events, 1);
    let (result, usage) = query(&core, before + 1).unwrap();
    result.unwrap();
    assert_eq!(usage.retained_entities_peak, before + 1);
    attack(&mut core, &mut state, enemy, ally, 20).unwrap();
    assert_eq!(states(&core)[0].events, 2);
    let failure = query(&core, before + 1).unwrap_err();
    assert_eq!(failure.site, "authored_state_history_bound");
    assert_eq!(failure.requested, Some((before + 2) as u64));
}

#[test]
fn authored_state_predicates_short_circuit_live_faults_in_declared_order() {
    // The second operand reads a live zero divisor. Static validation checks it,
    // but boolean evaluation must skip it when the first operand decides.
    for (operator, armed, expected_hp) in [("All", false, 400), ("Any", true, 407)] {
        let mut doc = parse_yaml(YAML).unwrap();
        doc["version"] = json!(2);
        doc["predicates"] = json!([{"name":"guarded-ratio","context":"State","parameters":[
        {"name":"enabled","value_type":"BoolField"},{"name":"ratio","value_type":{"Amount":"Scalar"}}],
        "body":{"Logic":{operator:[
            {"Bool":{"field":{"Parameter":"enabled"},"equals":true}},
            {"Compare":{"left":{"Parameter":"ratio"},"comparison":"Greater","right":{"Literal":{"Amount":{"moment":"Live","root":{"Const":{"value":0,"unit":"Scalar"}}}}}}}
        ]}}}]);
        let p = program(&mut doc);
        p["fields"] = json!([
            {"name":"armed","value":{"Bool":armed}},
            {"name":"divisor","value":{"Integer":{"unit":"Scalar","initial":0,"min":0,"max":1}}}
        ]);
        p["handlers"][0]["body"] = json!([{"If":{"condition":{"Predicate":{"name":"guarded-ratio","arguments":[
            {"name":"enabled","value":{"Literal":{"BoolField":"armed"}}},
            {"name":"ratio","value":{"Literal":{"Amount":{"moment":"Live","root":{"Ratio":{"numerator":{"Const":{"value":1,"unit":"Scalar"}},"denominator":{"Read":{"StateField":{"name":"divisor","unit":"Scalar"}}},"rounding":"Floor"}}}}}}
        ]}},"then_ops":[{"HealOwner":{"amount":{"moment":"Live","root":{"Const":{"value":7,"unit":"Magnitude"}}}}}],"else_ops":[]}}]);
        let (mut core, mut state, actor, enemy, ally) =
            fixture(compile_json(&doc.to_string()).unwrap());
        install(&mut core, &mut state, actor, ally);
        attack(&mut core, &mut state, enemy, ally, 10).unwrap();
        assert_eq!(
            core.units[&ally].stats.current_health, expected_hp,
            "{operator}"
        );
        assert_eq!(states(&core)[0].fields[1].value, StateScalar::Integer(0));
    }
}

include!("canonical_queen_tests.inc.rs");
