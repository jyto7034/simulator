#[path = "canonical_deferred.rs"]
mod canonical_deferred;
#[path = "canonical_scarecrow.rs"]
mod canonical_scarecrow;
#[path = "canonical_area_strikes.rs"]
mod canonical_area_strikes;
#[path = "composition_packaging.rs"]
mod packaging;
#[path = "composition_equipment_state.rs"]
mod equipment_state;
use super::*;
use crate::authoring::composition::CompositionTrial;
#[path = "composition_basic_cone.rs"]
mod basic_cone;
#[path = "composition_duration_modifiers.rs"]
mod duration_modifiers;
#[path = "composition_support_radius.rs"]
mod support_radius;
#[path = "composition_control.rs"]
mod control_effects;
#[path = "composition_taunt_damage.rs"]
mod taunt_damage;
#[path = "composition_range.rs"]
mod tile_range;
#[path = "composition_mountain.rs"]
mod mountain;
#[path = "composition_channel_ground.rs"]
mod channel_ground;
#[path = "composition_equipment_speed.rs"]
mod equipment_speed;
#[path = "composition_equipment_defense.rs"]
mod equipment_defense;
#[path = "composition_channel_upkeep.rs"]
mod channel_upkeep;
#[path = "composition_source_direct.rs"]
mod source_direct;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/skill_composition/woodsman_monk.skill.yaml"
));
const STATE_SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/skill_composition/state_events.skill.yaml"
));

#[test]
fn composition_equipment_attack_world_automatic_weapon_rewards_and_run_commit() {
    let (fixture, _, mut document) = state_world_fixture();
    let preparation=document["preparation_steps"].as_array().unwrap().len();
    document["steps"].as_array_mut().unwrap().truncate(25-preparation);
    // One clock advance ends at the encounter result; commands after terminal
    // combat are intentionally rejected by the real application.
    document["steps"].as_array_mut().unwrap().push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":20000,"after_seq":0}}));
    let source=include_str!("../../../../tests/fixtures/skill_composition/equipment_attack.skill.yaml").replace("equipment: r11_weapon","equipment: standard_training_sword");
    let cancel=CancellationToken::default();
    let mut expected=Vec::new();
    for (grant,heal) in [(2,7),(8,5)] {
        let current=source.replace("Grant: 2",&format!("Grant: {grant}"))
            .replace("value: 7, unit: Magnitude",&format!("value: {heal}, unit: Magnitude"));
        let mut trial=CompositionTrial::prepare(fixture.root(),&fixture.selection(),&current,checked(&document),None,TrialLimits::default(),&cancel).unwrap();
        let mut before=None;
        let mut rewards=Vec::new();
        let mut ordinal=None;
        while !trial.session().is_complete() {
            let pending=trial.session().next_step_index();
            let index=trial.advance(&cancel).unwrap_or_else(|e|panic!("equipment World step {pending}: {e:?}")).index();
            let observation=trial.session().observation().unwrap();
            if index==24 { before=Some(observation.run_resources().unwrap().clone()); }
            if index<25 {continue;}
            let Some(observed)=observation.active().or_else(||observation.last_terminal().map(|t|t.mandatory())).and_then(|o|o.latest_nonzero_sourced_hp()) else {continue;};
            if ordinal==Some(observed.ordinal()) {continue;}
            ordinal=Some(observed.ordinal());
            let Some(origin)=observed.authored_source() else {continue;};
            if !origin.definition_id().starts_with("equipment:") {continue;}
            assert!(observed.source().is_none(),"automatic callback must not invent a skill cast");
            assert_eq!(origin.source_file(),trial.identity().artifact_path);
            assert!(trial.origin_for_observed(origin).is_some());
            let equipment=observed.equipment_source().unwrap();
            assert!(equipment.persistent_copy_uuid.is_some());
            rewards.push(json!({"step":index,"heal":observed.change().hp_delta(),"equipment":equipment,"latest":observed}));
        }
        let observation=trial.session().observation().unwrap();
        assert!(observation.active().is_none()&&observation.last_terminal().is_some(),"ordinary equipped weapon must finish this encounter");
        assert_eq!(rewards.len(),1,"second encounter rewards: {rewards:?}, run={:?}",observation.run_resources());
        assert_eq!(rewards[0]["heal"],json!(heal));
        let run=observation.run_resources().unwrap();
        let copy=uuid::Uuid::parse_str(rewards[0]["equipment"]["persistent_copy_uuid"].as_str().unwrap()).unwrap();
        let owner=crate::game::resource::ResourceOwner::EquipmentCopy(copy);
        let matches=|e:&&crate::game::resource::RunResourceEntry|e.resource.as_str()=="weapon_mass"&&e.owner==owner;
        let prior=before.as_ref().unwrap().entries.iter().find(matches).map_or(0,|e|e.balance);
        let balance=run.entries.iter().find(matches).unwrap().balance;
        assert_eq!(balance,prior+grant);
        let reopened:crate::game::resource::RunResourceState=ron::de::from_str(&ron::ser::to_string(run).unwrap()).unwrap();
        assert_eq!(&reopened,run);
        eprintln!("equipment World grant {grant}: prior={prior} balance={balance} rewards={rewards:?}");
        expected.push(json!({"grant":grant,"heal":heal,"prior":prior,"balance":balance,"run":run,"rewards":rewards,"terminal":true}));
    }
    if let Some(directory)=std::env::var_os("EQUIPMENT_ATTACK_COMPOSITION_FIXTURE_DIR") {
        let directory=std::path::PathBuf::from(directory);
        assert!(!directory.exists(),"preserve earlier fixture evidence");
        copy_fixture(fixture.root(),&directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"),&source).unwrap();
        std::fs::write(directory.join("trial.json"),serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(directory.join("expected.json"),serde_json::to_vec_pretty(&expected).unwrap()).unwrap();
    }
}

fn copy_fixture(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let out = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_fixture(&entry.path(), &out);
        } else {
            std::fs::copy(entry.path(), out).unwrap();
        }
    }
}

fn state_world_fixture() -> (ExternalFixture, String, Value) {
    let (fixture, base, mut document) = public_amount_trial_fixture(|fixture| {
        let path = "official/data/skill_fragments/base.ron";
        fixture.write(
            path,
            fixture.read(path).replace(
                "skill_id: \"fragment_freischutz_black_round\", resonance_max: 100",
                "skill_id: \"fragment_freischutz_black_round\", resonance_max: 25",
            ),
        );
        let enemies = "official/data/enemies/corroded_employees.ron";
        // Keep this isolated encounter alive for both reaction charges and the
        // six-second observation window. Production balance is untouched.
        fixture.write(
            enemies,
            fixture
                .read(enemies)
                .replacen("max_health: 170,", "max_health: 300,", 1),
        );
    });
    let keep = document["steps"].as_array().unwrap().len() - 3;
    document["steps"].as_array_mut().unwrap().truncate(keep);
    let cancel = CancellationToken::default();
    let probe = simulate(
        fixture_content(&base),
        checked(&document),
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    // Place the employee on the incoming route. An adjacent firing cell can
    // leave the defended objective, rather than the employee, taking all hits.
    let entry = probe.step(20).unwrap().public_result().unwrap();
    let setup = &entry["payload"]["battle_setup_snapshot"];
    let zones = setup["deployment_zones"].as_array().unwrap();
    let ground = zones.iter().position(|z| z["kind"] == "Ground").unwrap();
    let cells = zones[ground]["cells"].as_array().unwrap();
    let route = setup["routes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "defense_main")
        .unwrap();
    let occupied = entry["payload"]["battle_update"]["checkpoint"]["units"]
        .as_array()
        .unwrap();
    let (cell_index, cell) = route["cells"]
        .as_array()
        .unwrap()
        .iter()
        .rev()
        .filter(|cell| !occupied.iter().any(|u| u["position"] == **cell))
        .find_map(|cell| {
            cells
                .iter()
                .position(|candidate| candidate == cell)
                .map(|i| (i, cell))
        })
        .expect("fixture needs a deployable interception cell on the actual route");
    let dx = cell["x"].as_i64().unwrap() - route["end"]["x"].as_i64().unwrap();
    let dy = cell["y"].as_i64().unwrap() - route["end"]["y"].as_i64().unwrap();
    let facing = if dx.abs() >= dy.abs() {
        if dx > 0 {
            "right"
        } else {
            "left"
        }
    } else if dy > 0 {
        "down"
    } else {
        "up"
    };
    let deploy = document["steps"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .rev()
        .find(|s| s["command"]["type"] == "deploy_unit")
        .unwrap();
    deploy["command"]["facing"] = json!(facing);
    let binding = deploy["bindings"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|b| b["to"] == "/command/position")
        .unwrap();
    binding["from"]["pointer"] = json!(format!(
        "/payload/battle_setup_snapshot/deployment_zones/{ground}/cells/{cell_index}"
    ));
    drop(probe);
    let probe = simulate(
        fixture_content(&base),
        checked(&document),
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    let units = probe.step(24).unwrap().public_result().unwrap()["payload"]["battle_update"]
        ["checkpoint"]["units"]
        .as_array()
        .unwrap();
    let player = units
        .iter()
        .position(|u| u["owner"] == "Player" && u["unit_source"]["kind"] == "employee")
        .unwrap();
    let mut begin = authority_step("begin_skill_targeting", 24);
    begin["command"]["employee_uuid"] = Value::Null;
    begin["command"]["skill_id"] = json!("fragment_freischutz_black_round");
    begin["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}));
    let mut activate = authority_step("activate_skill", 25);
    activate["command"]["employee_uuid"] = Value::Null;
    activate["command"]["skill_id"] = json!("fragment_freischutz_black_round");
    activate["command"]["target"] = json!({"type":"Unit","unit_instance_id":null});
    activate["bindings"].as_array_mut().unwrap().extend([
        json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}),
        json!({"from":{"step":24,"pointer":format!("/payload/battle_update/checkpoint/units/{player}/unit_instance_id")},"to":"/command/target/unit_instance_id","value_type":"string"}),
    ]);
    let steps = document["steps"].as_array_mut().unwrap();
    steps.push(begin);
    steps.push(activate);
    for _ in 0..20 {
        steps.push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    // A six-second test duration lets two ordinary enemy actions occur. Release
    // balance remains unspecified; no battle state is injected into World.
    let source = STATE_SOURCE
        .replace("authored-state", "fragment_freischutz_black_round")
        .replace("capacity: 100", "capacity: 25")
        .replace("duration_ms: 2000", "duration_ms: 6000");
    (fixture, source, document)
}

fn healing_deltas(value: &Value, result: &mut Vec<i64>) {
    match value {
        Value::Object(v) => {
            if v.get("type").is_some_and(|v| v == "HpChanged") {
                if let Some(delta) = v.get("delta").and_then(Value::as_i64).filter(|v| *v > 0) {
                    result.push(delta);
                }
            }
            for value in v.values() {
                healing_deltas(value, result);
            }
        }
        Value::Array(v) => {
            for value in v {
                healing_deltas(value, result);
            }
        }
        _ => {}
    }
}

#[test]
fn authored_state_world_reward_equip_and_damage_reactions_keep_source_identity() {
    let (fixture, source, document) = state_world_fixture();
    let cancel = CancellationToken::default();
    let mut trial = CompositionTrial::prepare(
        fixture.root(),
        &fixture.selection(),
        &source,
        checked(&document),
        None,
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    let mut heals = Vec::new();
    let mut traced = false;
    while !trial.session().is_complete() {
        let step = trial.advance(&cancel).unwrap();
        if step.index() < 27 {
            continue;
        }
        let observation = trial.session().observation().unwrap();
        if let Some(hp) = observation
            .active()
            .and_then(|a| a.latest_nonzero_sourced_hp())
        {
            if let Some(source) = hp.source() {
                let authored = source.authored();
                if authored.source_file() == trial.identity().artifact_path
                    && authored.field_path().contains(".op.Sustained.definition.State.program.")
                {
                    assert!(trial.origin_for_observed(authored).is_some());
                    traced = true;
                }
            }
        }
    }
    healing_deltas(
        trial
            .session()
            .step(trial.session().next_step_index() - 1)
            .unwrap()
            .public_result()
            .unwrap(),
        &mut heals,
    );
    eprintln!("World authored state healing deltas: {heals:?}; source mapped: {traced}");
    assert_eq!(
        heals.len(),
        2,
        "two charges must react to two real enemy actions"
    );
    assert!(
        traced,
        "state healing must be observed with the authored source"
    );
    if let Some(destination) = std::env::var_os("STATE_COMPOSITION_FIXTURE_DIR") {
        let destination = std::path::PathBuf::from(destination);
        assert!(
            !destination.exists(),
            "fixture export never overwrites prior input"
        );
        copy_fixture(fixture.root(), &destination.join("content"));
        std::fs::write(destination.join("source.skill.yaml"), &source).unwrap();
        std::fs::write(destination.join("trial.json"), serde_json::to_vec_pretty(&json!({
            "root":destination.join("content"), "selection":fixture.selection(), "scenario":document
        })).unwrap()).unwrap();
        std::fs::write(
            destination.join("expected.json"),
            serde_json::to_vec_pretty(&json!({"heals":heals, "identity":trial.identity()}))
                .unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn authored_state_world_runtime_fault_latches_host_and_blocks_progress_and_save() {
    use crate::application::{BattleFaultCause, BattleFaultOperation};
    let (fixture, source, document) = state_world_fixture();
    let broken = source.replace("value: -1", "value: 1");
    assert_ne!(broken, source);
    let cancel = CancellationToken::default();
    let mut trial = CompositionTrial::prepare(
        fixture.root(),
        &fixture.selection(),
        &broken,
        checked(&document),
        None,
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    while trial.session().next_step_index() <= 26 {
        trial.advance(&cancel).unwrap();
    }
    // Use the actual application host after real public preparation. The trial's
    // speculative step rollback is not evidence for the host's terminal gate.
    let mut app = trial
        .session()
        .state
        .application
        .fork_for_authoring()
        .unwrap();
    let mut failed = false;
    for _ in 0..20 {
        let before = serde_json::to_value(app.authoring_observation().unwrap()).unwrap();
        let resources = serde_json::to_value(app.run_resources()).unwrap();
        if let Err(error) = app.advance_active_battle_by(300, 0) {
            assert_eq!(error.code(), "value_expression_fault", "{error:?}");
            let first = *app.battle_fault().unwrap();
            assert_eq!(first.operation, BattleFaultOperation::Elapsed);
            assert!(matches!(first.cause, BattleFaultCause::Expression { .. }));
            assert_eq!(
                serde_json::to_value(app.authoring_observation().unwrap()).unwrap(),
                before
            );
            assert_eq!(
                serde_json::to_value(app.run_resources()).unwrap(),
                resources
            );
            assert_eq!(
                app.advance_active_battle_by(1, 0).unwrap_err().code(),
                "battle_faulted"
            );
            assert_eq!(
                app.active_battle_reconnect(0).unwrap_err().code(),
                "battle_faulted"
            );
            assert_eq!(
                app.export_active_run_checkpoint().err().unwrap().code(),
                "battle_faulted"
            );
            assert_eq!(
                app.capture_persistence_rollback().err().unwrap().code(),
                "battle_faulted"
            );
            assert_eq!(app.battle_fault(), Some(&first));
            let snapshot = app.snapshot(ApplicationSnapshotKind::Run).unwrap();
            assert_eq!(
                snapshot.read_model().as_value()["battle_fault"],
                json!(first)
            );
            assert_eq!(
                snapshot.read_model().as_value()["action_capabilities"],
                json!([])
            );
            failed = true;
            break;
        }
    }
    assert!(
        failed,
        "a real enemy HP loss must trigger the authored bounds error"
    );
}

fn ability_damage(value: &Value, out: &mut Vec<u64>) {
    match value {
        Value::Object(object) => {
            if object.get("damage_source").is_some_and(|v| v == "Ability") {
                if let Some(amount) = object.get("raw_damage").and_then(Value::as_u64) {
                    out.push(amount);
                }
            }
            for value in object.values() {
                ability_damage(value, out);
            }
        }
        Value::Array(values) => {
            for value in values {
                ability_damage(value, out);
            }
        }
        _ => {}
    }
}

#[test]
fn composition_world_trial_uses_real_equipped_fragment_and_pins_source_generation() {
    let (fixture, base, mut document) = public_amount_trial_fixture(|fixture| {
        // Every fragment form referencing the replaced definition must agree
        // with its capacity. This is authored fixture content, not a loader
        // exception or automatic runtime remapping.
        let path = "official/data/skill_fragments/base.ron";
        let before = fixture.read(path);
        let after = before.replace(
            "skill_id: \"fragment_freischutz_black_round\", resonance_max: 100",
            "skill_id: \"fragment_freischutz_black_round\", resonance_max: 25",
        );
        assert_ne!(before, after);
        fixture.write(path, after);
    });
    let original_catalog = fixture.read("official/data/skills/base.ron");
    // The external fixture's actual reward/equip commands already connect this
    // ID to an employee. The preview only replaces this skill definition.
    let source = SOURCE
        .replace("monk-authored", "fragment_freischutz_black_round")
        .replace("capacity: 100", "capacity: 25");
    let changed = source.replace("value: 3\n", "value: 5\n");
    assert_ne!(source, changed);
    let steps = document["steps"].as_array_mut().unwrap();
    steps.truncate(steps.len() - 3);
    let mut activate = authority_step("activate_skill", 24);
    activate["command"]["employee_uuid"] = Value::Null;
    activate["command"]["skill_id"] = json!("fragment_freischutz_black_round");
    activate["bindings"].as_array_mut().unwrap().push(json!({
        "from":{"step":9,"pointer":"/payload/employee_uuid"},
        "to":"/command/employee_uuid","value_type":"string"}));
    steps.push(activate);
    steps.push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":400,"after_seq":0}}));
    let cancel = CancellationToken::default();
    let prepare = |source: &str| {
        CompositionTrial::prepare(
            fixture.root(),
            &fixture.selection(),
            source,
            checked(&document),
            None,
            TrialLimits::default(),
            &cancel,
        )
        .unwrap()
    };
    let mut old = prepare(&source);
    while old.session().next_step_index() <= 25 {
        old.advance(&cancel).unwrap();
    }
    let identity = serde_json::to_value(old.identity()).unwrap();
    let mut new = prepare(&changed);
    assert_ne!(old.identity().run_id, new.identity().run_id);
    assert_ne!(old.identity().source_digest, new.identity().source_digest);
    assert_ne!(old.identity().content, new.identity().content);
    old.run_to_end(&cancel).unwrap();
    new.run_to_end(&cancel).unwrap();
    assert_eq!(serde_json::to_value(old.identity()).unwrap(), identity);
    let mut old_damage = Vec::new();
    let mut new_damage = Vec::new();
    ability_damage(
        old.session().step(26).unwrap().public_result().unwrap(),
        &mut old_damage,
    );
    ability_damage(
        new.session().step(26).unwrap().public_result().unwrap(),
        &mut new_damage,
    );
    assert_eq!(old_damage.len(), 1, "old ability damage: {old_damage:?}");
    assert_eq!(new_damage.len(), 1, "new ability damage: {new_damage:?}");
    // The fixture's equipped employee has 12 attack. The authored ratios are
    // 3/2 and 5/2; these literal expectations are independent of the compiler.
    assert_eq!(old_damage, vec![18]);
    assert_eq!(new_damage, vec![30]);
    assert!(
        new_damage[0] > old_damage[0],
        "{old_damage:?} -> {new_damage:?}"
    );
    let observed = old.session().observation().unwrap();
    eprintln!("World composition ability raw damage: {old_damage:?} -> {new_damage:?}");
    let latest = observed
        .active()
        .unwrap()
        .latest_nonzero_sourced_hp()
        .unwrap();
    let origin = latest.source().unwrap().authored();
    assert_eq!(origin.source_file(), old.identity().artifact_path);
    assert!(old.origin_for_observed(origin).is_some());
    assert_observed_origin_integrity(&old, origin);
    assert!(new.origin_for_observed(origin).is_none());
    assert_eq!(
        fixture.read("official/data/skills/base.ron"),
        original_catalog
    );
    assert_eq!(fixture.load().identity(), base.identity());
    if let Some(destination) = std::env::var_os("COMPOSITION_PREVIEW_FIXTURE_DIR") {
        let destination = std::path::PathBuf::from(destination);
        assert!(
            !destination.exists(),
            "fixture export never overwrites a prior input"
        );
        copy_fixture(fixture.root(), &destination.join("content"));
        std::fs::write(destination.join("source.skill.yaml"), source).unwrap();
        std::fs::write(destination.join("changed.skill.yaml"), changed).unwrap();
        std::fs::write(destination.join("trial.json"), serde_json::to_vec_pretty(&json!({
            "root": destination.join("content"), "selection": fixture.selection(), "scenario": document,
        })).unwrap()).unwrap();
        std::fs::write(destination.join("expected.json"), serde_json::to_vec_pretty(&json!({
            "old_raw_damage":old_damage, "new_raw_damage":new_damage, "old_identity":old.identity(), "new_identity":new.identity(),
        })).unwrap()).unwrap();
    }
}

#[test]
fn personal_shield_composition_world_absorbs_damage_then_expires() {
    let (fixture, _, mut document) = state_world_fixture();
    // Receipt indices include preparation; the editable scenario array does not.
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    let ready_steps = 25 - preparation;
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(ready_steps);
    let mut activate = authority_step("activate_skill", 24);
    activate["command"]["employee_uuid"] = Value::Null;
    activate["command"]["skill_id"] = json!("fragment_freischutz_black_round");
    activate["bindings"].as_array_mut().unwrap().push(json!({
        "from":{"step":9,"pointer":"/payload/employee_uuid"},
        "to":"/command/employee_uuid","value_type":"string"
    }));
    let steps = document["steps"].as_array_mut().unwrap();
    steps.push(activate);
    steps.push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":1,"after_seq":0}}));
    for _ in 0..20 {
        steps.push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/personal_shield.skill.yaml"
    ))
    .replace(
        "authored-personal-shield",
        "fragment_freischutz_black_round",
    )
    .replace("capacity: 100", "capacity: 25");
    assert_eq!(
        document["steps"][ready_steps]["command"]["type"],
        "activate_skill"
    );
    let cancel = CancellationToken::default();
    let mut trial = CompositionTrial::prepare(
        fixture.root(),
        &fixture.selection(),
        &source,
        checked(&document),
        None,
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    let mut created = None;
    let mut protected_hp = None;
    let mut consumed = false;
    let mut expired = false;
    while !trial.session().is_complete() {
        let pending = trial.session().next_step_index();
        let index = trial
            .advance(&cancel)
            .unwrap_or_else(|error| {
                panic!(
                    "World shield step {pending}: {error:?}; requested {}",
                    document["steps"][pending - preparation]
                )
            })
            .index();
        if index < 26 {
            continue;
        }
        let checkpoint = &trial
            .session()
            .step(index)
            .unwrap()
            .public_result()
            .unwrap()["payload"]["battle_update"]["checkpoint"];
        let pools: Vec<_> = checkpoint["sustained_effects"]["active"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v.get("personal_grant").is_some())
            .collect();
        if index == 26 {
            assert_eq!(pools.len(), 1, "{checkpoint}");
            let pool = pools[0];
            let target = checkpoint["units"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["unit_instance_id"] == pool["target"])
                .unwrap();
            eprintln!("World personal shield initial pool: {pool}; target: {target}");
            let capacity = pool["shield"]["capacity"].as_u64().unwrap();
            assert_eq!(
                capacity,
                target["stats"]["max_health"].as_u64().unwrap() * 20 / 100
            );
            protected_hp = target["stats"]["current_health"].as_u64();
            assert_eq!(pool["shield"]["remaining"], capacity);
            assert_eq!(pool["is_channel"], false);
            assert!(pool["personal_grant"]["fragment_copy_uuid"].is_string());
            created = Some(capacity);
        } else if let Some(pool) = pools.first() {
            consumed |= pool["shield"]["remaining"].as_u64().unwrap() < created.unwrap();
            let target = checkpoint["units"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["unit_instance_id"] == pool["target"])
                .unwrap();
            assert_eq!(
                target["stats"]["current_health"].as_u64(),
                protected_hp,
                "absorbed incoming damage must not remove HP"
            );
        } else {
            assert!(
                index >= 46,
                "pool must last until its authored deadline, not deplete early"
            );
            expired = true;
        }
    }
    if !consumed {
        eprintln!(
            "World shield final checkpoint: {}",
            trial
                .session()
                .step(trial.session().next_step_index() - 1)
                .unwrap()
                .public_result()
                .unwrap()["payload"]["battle_update"]["checkpoint"]
        );
    }
    assert!(
        consumed,
        "actual enemy pressure must consume the authored pool"
    );
    assert!(expired, "finite authored shield must end");
    if let Some(destination) = std::env::var_os("SHIELD_COMPOSITION_FIXTURE_DIR") {
        let destination = std::path::PathBuf::from(destination);
        assert!(
            !destination.exists(),
            "fixture export never overwrites prior input"
        );
        copy_fixture(fixture.root(), &destination.join("content"));
        std::fs::write(destination.join("source.skill.yaml"), &source).unwrap();
        std::fs::write(destination.join("trial.json"), serde_json::to_vec_pretty(&json!({
            "root":destination.join("content"), "selection":fixture.selection(), "scenario":document
        })).unwrap()).unwrap();
        std::fs::write(destination.join("expected.json"), serde_json::to_vec_pretty(&json!({
            "initial_capacity":created,"consumed":consumed,"expired":expired,"identity":trial.identity()
        })).unwrap()).unwrap();
    }
}

#[test]
fn composition_manual_state_world_same_input_consumes_parent_and_traces_child() {
    let (fixture, _, mut document) = state_world_fixture();
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(25 - preparation);
    let activate = |previous| {
        let mut step = authority_step("activate_skill", previous);
        step["command"]["employee_uuid"] = Value::Null;
        step["command"]["skill_id"] = json!("fragment_freischutz_black_round");
        step["bindings"].as_array_mut().unwrap().push(json!({
            "from":{"step":9,"pointer":"/payload/employee_uuid"},
            "to":"/command/employee_uuid","value_type":"string"
        }));
        step
    };
    document["steps"].as_array_mut().unwrap().extend([
        activate(24),
        json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":1,"after_seq":0}}),
        json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":500,"after_seq":0}}),
        activate(27),
    ]);
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/manual_state.skill.yaml"
    ))
    .replace("authored-manual-state", "fragment_freischutz_black_round")
    .replace("capacity: 100", "capacity: 25");
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for (cap, damage) in [(45, 40), (35, 35)] {
        let source = source.replace(
            "value: 45, unit: Magnitude",
            &format!("value: {cap}, unit: Magnitude"),
        );
        let mut trial = CompositionTrial::prepare(
            fixture.root(),
            &fixture.selection(),
            &source,
            checked(&document),
            None,
            TrialLimits::default(),
            &cancel,
        )
        .unwrap();
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("manual state World step {pending}: {e:?}"))
                .index();
            if index == 26 {
                let checkpoint = &trial
                    .session()
                    .step(index)
                    .unwrap()
                    .public_result()
                    .unwrap()["payload"]["battle_update"]["checkpoint"];
                let active = checkpoint["sustained_effects"]["active"]
                    .as_array()
                    .unwrap();
                assert!(
                    active
                        .iter()
                        .any(|s| s["state"]["definition"] == "temporary-weapon"
                            && !s["overlay"].is_null()),
                    "{checkpoint}"
                );
            }
        }
        let result = trial.session().step(28).unwrap().public_result().unwrap();
        assert!(
            !result["payload"]["battle_update"]["checkpoint"]["sustained_effects"]["active"]
                .as_array()
                .unwrap()
                .iter()
                .any(|s| s["state"]["definition"] == "temporary-weapon")
        );
        let mut amounts = Vec::new();
        ability_damage(result, &mut amounts);
        assert_eq!(amounts, vec![damage], "actual child raw damage, cap {cap}");
        let latest = trial
            .session()
            .observation()
            .unwrap()
            .active()
            .unwrap()
            .latest_nonzero_sourced_hp()
            .unwrap();
        let origin = latest.source().unwrap().authored();
        assert_eq!(origin.source_file(), trial.identity().artifact_path);
        assert!(trial.origin_for_observed(origin).is_some());
        eprintln!(
            "manual state World cap {cap}, child raw damage {damage}, sourced HP: {}",
            serde_json::to_value(latest).unwrap()
        );
        expected.push(json!({"cap":cap,"raw_damage":damage,"latest":latest}));
    }
    if let Some(destination) = std::env::var_os("MANUAL_STATE_COMPOSITION_FIXTURE_DIR") {
        let destination = std::path::PathBuf::from(destination);
        assert!(
            !destination.exists(),
            "fixture export never overwrites prior input"
        );
        copy_fixture(fixture.root(), &destination.join("content"));
        std::fs::write(destination.join("source.skill.yaml"), &source).unwrap();
        std::fs::write(destination.join("trial.json"), serde_json::to_vec_pretty(&json!({"root":destination.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(
            destination.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn composition_resource_world_fragment_attempt_balance_drives_live_heal() {
    use crate::game::resource::{ResourceLifetime, ResourceOwner};
    let (fixture, _, mut document) = state_world_fixture();
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"].as_array_mut().unwrap().truncate(25 - preparation);
    document["steps"].as_array_mut().unwrap().push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":5000,"after_seq":0}}));
    let mut activate = authority_step("activate_skill", 25);
    activate["command"]["employee_uuid"] = Value::Null;
    activate["command"]["skill_id"] = json!("fragment_freischutz_black_round");
    activate["bindings"].as_array_mut().unwrap().push(json!({
        "from":{"step":9,"pointer":"/payload/employee_uuid"},
        "to":"/command/employee_uuid","value_type":"string"
    }));
    document["steps"].as_array_mut().unwrap().extend([
        activate,
        json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":1,"after_seq":0}}),
    ]);
    let source = include_str!("../../../../tests/fixtures/skill_composition/resources.skill.yaml")
        .replace("authored-attempt-heal", "fragment_freischutz_black_round")
        .replace("capacity: 100", "capacity: 25");
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for (cap, balance) in [(15, 10), (6, 1)] {
        let current = source.replace("initial: 0, cap: 15", &format!("initial: 0, cap: {cap}"));
        let mut trial = CompositionTrial::prepare(fixture.root(), &fixture.selection(), &current,
            checked(&document), None, TrialLimits::default(), &cancel).unwrap();
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            trial.advance(&cancel).unwrap_or_else(|e| panic!("resource World step {pending}: {e:?}"));
        }
        let observation = trial.session().observation().unwrap();
        let resources = observation.active_typed_resources().unwrap();
        let (key, value) = resources.entries().find(|(k, _)| k.resource.as_str() == "fragment_meter").unwrap();
        assert!(matches!(key.owner, ResourceOwner::FragmentCopy(_)));
        assert_eq!(value.balance, balance);
        assert_eq!(value.reserved, 0);
        assert!(resources.scopes().any(|(id, kind, _, _)| id == key.scope && kind == ResourceLifetime::Attempt));
        assert!(observation.run_resources().unwrap().entries.is_empty());
        let before = &trial.session().step(25).unwrap().public_result().unwrap()["payload"]["battle_update"]["checkpoint"]["units"];
        let unit = before.as_array().unwrap().iter().find(|u| u["owner"] == "Player" && u["unit_source"]["kind"] == "employee").unwrap();
        let missing = unit["stats"]["max_health"].as_i64().unwrap() - unit["stats"]["current_health"].as_i64().unwrap();
        assert!(missing >= 2, "fixture must expose two distinct actual heal outcomes: {unit}");
        let latest = observation.active().unwrap().latest_nonzero_sourced_hp().unwrap();
        let observed = serde_json::to_value(latest).unwrap();
        let origin = latest.source().unwrap().authored();
        assert_eq!(origin.source_file(), trial.identity().artifact_path);
        assert!(trial.origin_for_observed(origin).is_some());
        let mut heals = Vec::new();
        healing_deltas(trial.session().step(27).unwrap().public_result().unwrap(), &mut heals);
        assert_eq!(heals, vec![missing.min(i64::from(balance))], "live resource read must drive actual healing");
        eprintln!("resource World cap {cap}: balance {balance}, missing {missing}, heal {heals:?}; {observed}");
        expected.push(json!({"cap":cap,"balance":balance,"heals":heals,"latest":observed}));
    }
    if let Some(directory) = std::env::var_os("RESOURCE_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "fixture export must preserve earlier evidence");
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), &source).unwrap();
        std::fs::write(directory.join("trial.json"), serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(directory.join("expected.json"), serde_json::to_vec_pretty(&expected).unwrap()).unwrap();
    }
}

#[test]
fn composition_basic_kill_world_normal_attack_drives_authored_reward() {
    let (fixture, _, mut document) = state_world_fixture();
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"].as_array_mut().unwrap().truncate(25 - preparation);
    document["steps"].as_array_mut().unwrap().push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":5000,"after_seq":0}}));
    let mut activate = authority_step("activate_skill", 25);
    activate["command"]["employee_uuid"] = Value::Null;
    activate["command"]["skill_id"] = json!("fragment_freischutz_black_round");
    activate["bindings"].as_array_mut().unwrap().push(json!({
        "from":{"step":9,"pointer":"/payload/employee_uuid"},
        "to":"/command/employee_uuid","value_type":"string"
    }));
    document["steps"].as_array_mut().unwrap().push(activate);
    // Four ticks reach the actual terminal kill; another clock is invalid after battle exit.
    for _ in 0..4 {
        document["steps"].as_array_mut().unwrap().push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    let source = include_str!("../../../../tests/fixtures/skill_composition/basic_kill.skill.yaml")
        .replace("authored-basic-kill", "fragment_freischutz_black_round")
        .replace("capacity: 100", "capacity: 25")
        .replace("duration_ms: 2000", "duration_ms: 6000")
        .replace("power: 40", "power: 500");
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for heal in [5, 100] {
        let current = source.replace("value: 100, unit: Magnitude", &format!("value: {heal}, unit: Magnitude"));
        let mut trial = CompositionTrial::prepare(fixture.root(), &fixture.selection(), &current,
            checked(&document), None, TrialLimits::default(), &cancel).unwrap();
        let mut rewards = Vec::new();
        let mut last_reward = None;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial.advance(&cancel).unwrap_or_else(|e| panic!("basic kill World step {pending}: {e:?}")).index();
            let result = trial.session().step(index).unwrap().public_result().unwrap();
            if index < 27 {continue;}
            let mut heals = Vec::new();
            healing_deltas(result, &mut heals);
            if !heals.is_empty() {
                let observation = trial.session().observation().unwrap();
                let observed = observation.active().or_else(|| observation.last_terminal().map(|t|t.mandatory())).unwrap().latest_nonzero_sourced_hp().unwrap();
                if last_reward == Some(observed.ordinal()) {continue;}
                last_reward = Some(observed.ordinal());
                let origin = observed.source().unwrap().authored();
                assert_eq!(origin.source_file(), trial.identity().artifact_path);
                assert!(trial.origin_for_observed(origin).is_some());
                rewards.push(json!({"step":index,"heals":heals,"latest":observed}));
            }
        }
        assert!(trial.session().observation().unwrap().active().is_none());
        assert!(trial.session().observation().unwrap().last_terminal().is_some(), "the direct kill must finalize this real encounter");
        assert_eq!(rewards.len(), 1, "one authored reward: {rewards:?}");
        // This employee's max HP is 180. The previous resource fixture's 10
        // was a resource-limited heal, not this employee's missing health.
        let before = trial.session().step(25).unwrap().public_result().unwrap()["payload"]["battle_update"]["checkpoint"]["units"].as_array().unwrap();
        let employee = before.iter().find(|u|u["owner"]=="Player" && u["unit_source"]["kind"]=="employee").unwrap();
        assert_eq!(employee["stats"]["max_health"],json!(180));
        assert_eq!(employee["stats"]["current_health"],json!(163));
        assert_eq!(rewards[0]["latest"]["change"]["before"],json!(162));
        assert_eq!(rewards[0]["heals"], json!([if heal == 5 {5} else {18}]), "{rewards:?}");
        eprintln!("basic kill World heal {heal}: {rewards:?}");
        expected.push(json!({"heal":heal,"rewards":rewards,"terminal":true}));
    }
    if let Some(directory) = std::env::var_os("BASIC_KILL_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "fixture export must preserve earlier evidence");
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), &source).unwrap();
        std::fs::write(directory.join("trial.json"), serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(directory.join("expected.json"), serde_json::to_vec_pretty(&expected).unwrap()).unwrap();
    }
}

#[test]
fn composition_state_resource_world_kill_commits_run_balance() {
    let (fixture, _, mut document) = state_world_fixture();
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"].as_array_mut().unwrap().truncate(25 - preparation);
    document["steps"].as_array_mut().unwrap().push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":5000,"after_seq":0}}));
    let mut activate = authority_step("activate_skill", 25);
    activate["command"]["employee_uuid"] = Value::Null;
    activate["command"]["skill_id"] = json!("fragment_freischutz_black_round");
    activate["bindings"].as_array_mut().unwrap().push(json!({
        "from":{"step":9,"pointer":"/payload/employee_uuid"},
        "to":"/command/employee_uuid","value_type":"string"
    }));
    document["steps"].as_array_mut().unwrap().push(activate);
    // Four ticks reach the actual terminal kill; another clock is invalid after battle exit.
    for _ in 0..4 {
        document["steps"].as_array_mut().unwrap().push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    let source = include_str!("../../../../tests/fixtures/skill_composition/state_resources.skill.yaml")
        .replace("authored-basic-kill", "fragment_freischutz_black_round")
        .replace("capacity: 100", "capacity: 25")
        .replace("duration_ms: 2000", "duration_ms: 6000")
        .replace("power: 40", "power: 500")
        .replace("lifetime: Attempt", "lifetime: Run");
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for (grant, balance) in [(7, 4), (20, 12)] {
        let heal = 5;
        let current = source.replace("value: 100, unit: Magnitude", &format!("value: {heal}, unit: Magnitude"))
            .replace("Grant: 7", &format!("Grant: {grant}"));
        let mut trial = CompositionTrial::prepare(fixture.root(), &fixture.selection(), &current,
            checked(&document), None, TrialLimits::default(), &cancel).unwrap();
        let mut rewards = Vec::new();
        let mut last_reward = None;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial.advance(&cancel).unwrap_or_else(|e| panic!("state resource World step {pending}: {e:?}")).index();
            let result = trial.session().step(index).unwrap().public_result().unwrap();
            if index < 27 {continue;}
            let mut heals = Vec::new();
            healing_deltas(result, &mut heals);
            if !heals.is_empty() {
                let observation = trial.session().observation().unwrap();
                let observed = observation.active().or_else(|| observation.last_terminal().map(|t|t.mandatory())).unwrap().latest_nonzero_sourced_hp().unwrap();
                if last_reward == Some(observed.ordinal()) {continue;}
                last_reward = Some(observed.ordinal());
                let origin = observed.source().unwrap().authored();
                assert_eq!(origin.source_file(), trial.identity().artifact_path);
                assert!(trial.origin_for_observed(origin).is_some());
                rewards.push(json!({"step":index,"heals":heals,"latest":observed}));
            }
        }
        assert!(trial.session().observation().unwrap().active().is_none());
        assert!(trial.session().observation().unwrap().last_terminal().is_some(), "the direct kill must finalize this real encounter");
        assert_eq!(rewards.len(), 1, "one authored reward: {rewards:?}");
        // This employee's max HP is 180. The previous resource fixture's 10
        // was a resource-limited heal, not this employee's missing health.
        let before = trial.session().step(25).unwrap().public_result().unwrap()["payload"]["battle_update"]["checkpoint"]["units"].as_array().unwrap();
        let employee = before.iter().find(|u|u["owner"]=="Player" && u["unit_source"]["kind"]=="employee").unwrap();
        assert_eq!(employee["stats"]["max_health"],json!(180));
        assert_eq!(employee["stats"]["current_health"],json!(163));
        assert_eq!(rewards[0]["latest"]["change"]["before"],json!(162));
        assert_eq!(rewards[0]["heals"], json!([if heal == 5 {5} else {18}]), "{rewards:?}");
        let observation = trial.session().observation().unwrap();
        let run = observation.run_resources().unwrap();
        let entry = run.entries.iter().find(|e| e.resource.as_str() == "kill_meter").unwrap();
        assert_eq!(entry.balance, balance);
        assert!(matches!(entry.owner, crate::game::resource::ResourceOwner::FragmentCopy(_)));
        let reopened: crate::game::resource::RunResourceState = ron::de::from_str(&ron::ser::to_string(run).unwrap()).unwrap();
        assert_eq!(&reopened, run);
        eprintln!("state resource World grant {grant}: {run:?}, {rewards:?}");
        expected.push(json!({"grant":grant,"balance":balance,"run":run,"rewards":rewards,"terminal":true}));
    }
    if let Some(directory) = std::env::var_os("STATE_RESOURCE_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "fixture export must preserve earlier evidence");
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), &source).unwrap();
        std::fs::write(directory.join("trial.json"), serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(directory.join("expected.json"), serde_json::to_vec_pretty(&expected).unwrap()).unwrap();
    }
}

#[test]
fn composition_kill_eligibility_world_qualified_reward_commits_run_balance() {
    let (fixture, _, mut document) = state_world_fixture();
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"].as_array_mut().unwrap().truncate(25 - preparation);
    document["steps"].as_array_mut().unwrap().push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":5000,"after_seq":0}}));
    let mut activate = authority_step("activate_skill", 25);
    activate["command"]["employee_uuid"] = Value::Null;
    activate["command"]["skill_id"] = json!("fragment_freischutz_black_round");
    activate["bindings"].as_array_mut().unwrap().push(json!({
        "from":{"step":9,"pointer":"/payload/employee_uuid"},
        "to":"/command/employee_uuid","value_type":"string"
    }));
    document["steps"].as_array_mut().unwrap().push(activate);
    for _ in 0..4 {
        document["steps"].as_array_mut().unwrap().push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    let source = include_str!("../../../../tests/fixtures/skill_composition/kill_eligibility.skill.yaml")
        .replace("authored-basic-kill", "fragment_freischutz_black_round")
        .replace("capacity: 100", "capacity: 25")
        .replace("duration_ms: 2000", "duration_ms: 6000")
        .replace("power: 40", "power: 500");
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for (grant, heal, threat) in [(2, 7, "normal"), (8, 5, "normal"), (2, 7, "elite")] {
        let current = source.replace("Grant: 2", &format!("Grant: {grant}"))
            .replace("value: 7, unit: Magnitude", &format!("value: {heal}, unit: Magnitude"))
            .replacen("threat: normal", &format!("threat: {threat}"), 1);
        let mut trial = CompositionTrial::prepare(fixture.root(), &fixture.selection(), &current,
            checked(&document), None, TrialLimits::default(), &cancel).unwrap();
        let mut rewards = Vec::new();
        let mut last_reward = None;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial.advance(&cancel).unwrap_or_else(|e| panic!("kill eligibility World step {pending}: {e:?}")).index();
            if index < 27 { continue; }
            let result = trial.session().step(index).unwrap().public_result().unwrap();
            let mut heals = Vec::new();
            healing_deltas(result, &mut heals);
            if !heals.is_empty() {
                let observation = trial.session().observation().unwrap();
                let observed = observation.active().or_else(|| observation.last_terminal().map(|t|t.mandatory())).unwrap().latest_nonzero_sourced_hp().unwrap();
                if last_reward == Some(observed.ordinal()) { continue; }
                last_reward = Some(observed.ordinal());
                let origin = observed.source().unwrap().authored();
                assert_eq!(origin.source_file(), trial.identity().artifact_path);
                assert!(trial.origin_for_observed(origin).is_some());
                rewards.push(json!({"step":index,"heals":heals,"latest":observed}));
            }
        }
        let observation = trial.session().observation().unwrap();
        assert!(observation.active().is_none() && observation.last_terminal().is_some());
        let run = observation.run_resources().unwrap();
        let entry = run.entries.iter().find(|e| e.resource.as_str() == "kill_meter");
        let balance = entry.map_or(0, |e| e.balance);
        let qualified = threat == "normal";
        assert_eq!(balance, if qualified { grant } else { 0 });
        assert_eq!(rewards.len(), usize::from(qualified));
        if qualified {
            assert_eq!(rewards[0]["heals"], json!([heal]));
            assert!(matches!(entry.unwrap().owner, crate::game::resource::ResourceOwner::FragmentCopy(_)));
        }
        let reopened: crate::game::resource::RunResourceState = ron::de::from_str(&ron::ser::to_string(run).unwrap()).unwrap();
        assert_eq!(&reopened, run);
        eprintln!("kill eligibility World {threat} grant {grant}: {run:?}, {rewards:?}");
        expected.push(json!({"grant":grant,"heal":heal,"threat":threat,"balance":balance,"run":run,"rewards":rewards,"terminal":true}));
    }
    if let Some(directory) = std::env::var_os("KILL_ELIGIBILITY_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "fixture export must preserve earlier evidence");
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), &source).unwrap();
        std::fs::write(directory.join("trial.json"), serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(directory.join("expected.json"), serde_json::to_vec_pretty(&expected).unwrap()).unwrap();
    }
}

#[test]
fn composition_state_reward_world_expiry_changes_real_attack_then_restores() {
    let (fixture, _, mut document) = state_world_fixture();
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"].as_array_mut().unwrap().truncate(25 - preparation);
    document["steps"].as_array_mut().unwrap().push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":5000,"after_seq":0}}));
    let mut activate = authority_step("activate_skill", 25);
    activate["command"]["employee_uuid"] = Value::Null;
    activate["command"]["skill_id"] = json!("fragment_freischutz_black_round");
    activate["bindings"].as_array_mut().unwrap().push(json!({
        "from":{"step":9,"pointer":"/payload/employee_uuid"},
        "to":"/command/employee_uuid","value_type":"string"
    }));
    document["steps"].as_array_mut().unwrap().push(activate);
    for _ in 0..10 {
        document["steps"].as_array_mut().unwrap().push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));
    }
    let source = include_str!("../../../../tests/fixtures/skill_composition/state_reward.skill.yaml")
        .replace("authored-basic-kill", "fragment_freischutz_black_round")
        .replace("capacity: 100", "capacity: 25")
        .replace("duration_ms: 2000", "duration_ms: 600")
        .replace("BasicAttackKill: { primary_only: false, once_per_action: true }", "Expired:")
        .replace("event:\n                          Expired:", "event: Expired");
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for (percent, attack) in [(25,15),(50,18)] {
        let current = source.replace("attack_percent: 25", &format!("attack_percent: {percent}"))
            .replace("value: 100, unit: Magnitude", "value: 5, unit: Magnitude");
        let mut trial = CompositionTrial::prepare(fixture.root(), &fixture.selection(), &current,
            checked(&document), None, TrialLimits::default(), &cancel).unwrap();
        let mut observed = Vec::new();
        let mut restored = false;
        let mut base_attack = None;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial.advance(&cancel).unwrap_or_else(|e| panic!("state reward World step {pending}: {e:?}")).index();
            if index < 25 { continue; }
            let checkpoint = &trial.session().step(index).unwrap().public_result().unwrap()["payload"]["battle_update"]["checkpoint"];
            let units = checkpoint["units"].as_array().unwrap();
            let employee = units.iter().find(|u|u["owner"]=="Player" && u["unit_source"]["kind"]=="employee").unwrap();
            if index == 25 {
                assert_eq!(employee["stats"]["attack"], json!(12));
                base_attack = Some(12);
            }
            let reward = checkpoint["sustained_effects"]["active"].as_array().unwrap().iter()
                .find(|r|r["target_state"]["definition"]=="direct-kill-reward");
            if let Some(reward) = reward {
                assert_eq!(employee["stats"]["attack"], json!(attack));
                assert_eq!(reward["target_state"]["attack_percent"], json!(percent));
                assert!(reward["fragment_copy_uuid"].is_string());
                assert_eq!(reward["target"], employee["unit_instance_id"]);
                assert!(reward["authored_site"].is_object());
                assert!(reward["flow_occurrence"].is_null());
                observed.push(json!({"step":index,"attack":attack,"reward":reward}));
            } else if !observed.is_empty() {
                assert_eq!(employee["stats"]["attack"], json!(12));
                restored = true;
            }
        }
        assert_eq!(base_attack, Some(12));
        assert!(!observed.is_empty() && restored, "reward and natural restoration: {observed:?}");
        let observation = trial.session().observation().unwrap().active().unwrap();
        let latest = observation.latest_nonzero_sourced_hp().unwrap();
        assert!(trial.origin_for_observed(latest.source().unwrap().authored()).is_some());
        let change = serde_json::to_value(latest).unwrap();
        assert_eq!(change["change"]["after"].as_i64().unwrap() - change["change"]["before"].as_i64().unwrap(), 5);
        eprintln!("state reward World percent {percent}: {observed:?}");
        expected.push(json!({"percent":percent,"attack":attack,"observed":observed,"restored":restored,"heal":5}));
    }
    if let Some(directory) = std::env::var_os("STATE_REWARD_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "fixture export must preserve earlier evidence");
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), &source).unwrap();
        std::fs::write(directory.join("trial.json"), serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(directory.join("expected.json"), serde_json::to_vec_pretty(&expected).unwrap()).unwrap();
    }
}

#[test]
fn composition_prepared_state_world_weapon_release_drives_authored_reward() {
    let (fixture, _, mut document) = state_world_fixture();
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"].as_array_mut().unwrap().truncate(25 - preparation);
    document["steps"].as_array_mut().unwrap().push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":5000,"after_seq":0}}));
    let mut activate = authority_step("activate_skill", 25);
    activate["command"]["employee_uuid"] = Value::Null;
    activate["command"]["skill_id"] = json!("fragment_freischutz_black_round");
    activate["bindings"].as_array_mut().unwrap().push(json!({
        "from":{"step":9,"pointer":"/payload/employee_uuid"},
        "to":"/command/employee_uuid","value_type":"string"
    }));
    document["steps"].as_array_mut().unwrap().extend([
        activate,
        json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":1,"after_seq":0}}),
        json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":3000,"after_seq":0}}),
    ]);
    // A test-only large multiplier guarantees a kill with the actual equipped
    // weapon. A second authored heal makes the result observable after victory.
    let source = include_str!("../../../../tests/fixtures/skill_composition/prepared_state.skill.yaml")
        .replace("authored-prepared-state", "fragment_freischutz_black_round")
        .replace("capacity: 100", "capacity: 25")
        .replace("numerator: 5", "numerator: 100")
        .replace("definitions:", "                          - HealOwner:\n                              amount:\n                                moment: Live\n                                root:\n                                  Const: { value: 100, unit: Magnitude }\ndefinitions:");
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for (numerator, heal) in [(100,100),(200,5)] {
        let current = source.replace("numerator: 100", &format!("numerator: {numerator}"))
            .replace("value: 100, unit: Magnitude", &format!("value: {heal}, unit: Magnitude"));
        let mut trial = CompositionTrial::prepare(fixture.root(), &fixture.selection(), &current,
            checked(&document), None, TrialLimits::default(), &cancel).unwrap();
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            trial.advance(&cancel).unwrap_or_else(|e| panic!("prepared state World step {pending}: {e:?}"));
        }
        let installed = &trial.session().step(27).unwrap().public_result().unwrap()["payload"]["battle_update"]["checkpoint"]["sustained_effects"]["prepared_basics"];
        assert_eq!(installed.as_array().unwrap().len(), 1, "{installed}");
        assert_eq!(installed[0]["profile"]["numerator"], json!(numerator));
        assert_eq!(installed[0]["state"]["definition"], json!("prepared-strike"));
        assert!(installed[0]["fragment_copy_uuid"].is_string());
        let observation = trial.session().observation().unwrap();
        let terminal = observation.last_terminal().expect("the prepared weapon attack must win this encounter");
        let latest = terminal.mandatory().latest_nonzero_sourced_hp().unwrap();
        let origin = latest.source().unwrap().authored();
        assert_eq!(origin.source_file(), trial.identity().artifact_path);
        assert!(trial.origin_for_observed(origin).is_some());
        let value = serde_json::to_value(latest).unwrap();
        let delta = value["change"]["after"].as_i64().unwrap() - value["change"]["before"].as_i64().unwrap();
        assert_eq!(delta, if heal == 5 {5} else {18});
        let mut heals = Vec::new();
        healing_deltas(trial.session().step(28).unwrap().public_result().unwrap(), &mut heals);
        assert_eq!(heals, vec![delta]);
        eprintln!("prepared state World ratio {numerator}/2, heal {heal}: {value}");
        expected.push(json!({"numerator":numerator,"heal":heal,"hp_delta":delta,"installed":installed,"latest":value,"terminal":true}));
    }
    if let Some(directory) = std::env::var_os("PREPARED_STATE_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "fixture export must preserve earlier evidence");
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), &source).unwrap();
        std::fs::write(directory.join("trial.json"), serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(directory.join("expected.json"), serde_json::to_vec_pretty(&expected).unwrap()).unwrap();
    }
}

#[test]
fn composition_target_state_world_real_stats_and_use_restore_after_expiry() {
    let (fixture, _, mut document) = state_world_fixture();
    let preparation=document["preparation_steps"].as_array().unwrap().len();
    document["steps"].as_array_mut().unwrap().truncate(25-preparation);
    document["steps"].as_array_mut().unwrap().push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":5000,"after_seq":0}}));
    let mut activate=authority_step("activate_skill",25);
    activate["command"]["employee_uuid"]=Value::Null;
    activate["command"]["skill_id"]=json!("fragment_freischutz_black_round");
    activate["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}));
    document["steps"].as_array_mut().unwrap().push(activate);
    for _ in 0..10 {document["steps"].as_array_mut().unwrap().push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}}));}
    let source=include_str!("../../../../tests/fixtures/skill_composition/target_state.skill.yaml")
        .replace("authored-target-state","fragment_freischutz_black_round").replace("capacity: 100","capacity: 25");
    let cancel=CancellationToken::default();
    let mut expected=Vec::new();
    for (percent,attack) in [(25,15),(50,18)] {
        let current=source.replace("attack_percent: 25",&format!("attack_percent: {percent}"));
        let mut trial=CompositionTrial::prepare(fixture.root(),&fixture.selection(),&current,checked(&document),None,TrialLimits::default(),&cancel).unwrap();
        let mut observed=Vec::new();let mut restored=false;
        while !trial.session().is_complete() {
            let pending=trial.session().next_step_index();
            let index=trial.advance(&cancel).unwrap_or_else(|e|panic!("target state World step {pending}: {e:?}")).index();
            if index<25 {continue;}
            let result=trial.session().step(index).unwrap().public_result().unwrap();
            let cp=&result["payload"]["battle_update"]["checkpoint"];
            let employee=cp["units"].as_array().unwrap().iter().find(|u|u["owner"]=="Player"&&u["unit_source"]["kind"]=="employee").unwrap();
            let effect=cp["sustained_effects"]["active"].as_array().unwrap().iter().find(|r|r["target_state"]["definition"]=="resolve");
            if let Some(effect)=effect {
                assert_eq!(employee["stats"]["attack"],json!(attack));
                assert_eq!(effect["target"],employee["unit_instance_id"]);
                assert_eq!(effect["target_state"]["hp_floor_one"],json!(true));
                assert!(effect["fragment_copy_uuid"].is_string());
                let latest=trial.session().observation().unwrap().active().unwrap().latest_sourced_sustained().unwrap();
                let authored=latest.authored_source().unwrap();
                let trace=trial.origin_for_observed(authored).expect("actual activation must map to this source generation");
                assert_eq!(authored.source_file(),trial.identity().artifact_path);
                assert_eq!(latest.target_unit().to_string(),employee["unit_instance_id"].as_str().unwrap());
                match latest.change() {
                    crate::authoring::ApplicationChange::Sustained{ended, before, after,..} => {
                        assert!(ended.is_none());assert_eq!(before.unwrap().attack,12);assert_eq!(after.unwrap().attack,attack);
                    }
                    other=>panic!("expected actual sustained application: {other:?}"),
                }
                assert!(trace.definition.starts_with("definitions[0].body.effects[0]"),"{trace:?}");
                assert!(trace.inputs.iter().any(|p|p=="values[0].value"));
                assert!(!trace.calls.is_empty());
                observed.push(json!({"step":index,"attack":attack,"effect":effect,"origin":trace}));
            } else if !observed.is_empty() {assert_eq!(employee["stats"]["attack"],json!(12));restored=true;}
        }
        assert!(!observed.is_empty()&&restored,"actual grant/expiry: {observed:?}");
        let latest=trial.session().observation().unwrap().active().unwrap().latest_sourced_sustained().unwrap();
        assert!(matches!(latest.change(),crate::authoring::ApplicationChange::Sustained{ended:Some(crate::game::ability::sustained::SustainedEndReason::Expired),..}));
        assert!(trial.origin_for_observed(latest.authored_source().unwrap()).is_some());
        if percent == 25 {
            for terminal in [false,true] {
                let mut scenario=document.clone();
                if terminal {scenario["steps"].as_array_mut().unwrap().push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":20000,"after_seq":0}}));}
                let response=crate::authoring::composition::run_composition(crate::authoring::composition::CompositionRunRequest {
                    source:current.clone(),epoch:21,revision:4,request_id:8,
                    artifact_digest:trial.identity().artifact_digest.clone(),root:fixture.root().into(),selection:fixture.selection(),
                    scenario:serde_json::to_string(&scenario).unwrap(),lab:None,
                },&cancel).unwrap();
                let value=serde_json::to_value(response).unwrap();
                assert_eq!(value["status"],"completed","{value}");
                let state=&value["latest_sustained"];
                assert_eq!(state["battle_terminal"],terminal,"{value}");
                assert_eq!(state["ended"],"Expired","{value}");assert_eq!(state["before"]["attack"],15);assert_eq!(state["after"]["attack"],12);
                assert!(state["origin"]["definition"].as_str().unwrap().starts_with("definitions[0].body.effects[0]"));
                assert!(state["origin"]["inputs"].as_array().unwrap().contains(&json!("values[0].value")));
                assert!(!state["origin"]["calls"].as_array().unwrap().is_empty());
                assert_eq!(state["source_file"],value["identity"]["artifact_path"]);
                assert_eq!(value["identity"]["source_digest"],trial.identity().source_digest);
                eprintln!("sustained observation run service terminal={terminal}: {value}");
            }
        }
        eprintln!("target state World percent {percent}: {observed:?}");
        expected.push(json!({"percent":percent,"attack":attack,"observed":observed,"restored":restored}));
    }
    if let Some(directory)=std::env::var_os("TARGET_STATE_COMPOSITION_FIXTURE_DIR") {
        let directory=std::path::PathBuf::from(directory);assert!(!directory.exists(),"preserve earlier fixture evidence");
        copy_fixture(fixture.root(),&directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"),&source).unwrap();
        std::fs::write(directory.join("trial.json"),serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(directory.join("expected.json"),serde_json::to_vec_pretty(&expected).unwrap()).unwrap();
    }
}

fn channel_support_world_fixture() -> (ExternalFixture, String, Value, Value, Value) {
    channel_support_world_fixture_with_cell(false)
}

fn channel_support_world_fixture_with_cell(orthogonal: bool) -> (ExternalFixture, String, Value, Value, Value) {
    let (fixture, _, mut document) = state_world_fixture();
    // The official catalog deliberately contains no invented relic definitions.
    // Declare the authored selector only in this isolated test content pack.
    fixture.write("official/data/relics/base.ron", r#"(relics:[(id:"single-target-conversion",uuid:"00000000-0000-0000-0000-00000000c501",name:"Support conversion fixture",description:"Test selector; inventory remains empty",rarity:"fixture",allow_duplicates:true,effects:[(id:"unrelated_damage",selector:(all_tags:["unrelated_fixture_damage"]),percent:1)],dismantle_equipment_dust:1)])"#);
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"].as_array_mut().unwrap().truncate(25 - preparation);
    let source = include_str!("../../../../tests/fixtures/skill_composition/channel_support.skill.yaml")
        .replace("authored-channel-support", "fragment_freischutz_black_round")
        .replace("capacity: 100", "capacity: 25")
        .replace("duration_ms: 200", "duration_ms: 1500");
    let cancel = CancellationToken::default();
    let mut probe = CompositionTrial::prepare(fixture.root(), &fixture.selection(), &source,
        checked(&document), None, TrialLimits::default(), &cancel).unwrap();
    probe.run_to_end(&cancel).unwrap();
    let cp = &probe.session().step(24).unwrap().public_result().unwrap()["payload"]["battle_update"]["checkpoint"];
    let units = cp["units"].as_array().unwrap();
    let owner = units.iter().find(|u| u["owner"] == "Player" && u["unit_source"]["kind"] == "employee").unwrap();
    let owner_id = owner["unit_instance_id"].clone();
    let owner_attack = owner["stats"]["attack"].clone();
    let entry = probe.session().step(20).unwrap().public_result().unwrap();
    let zones = entry["payload"]["battle_setup_snapshot"]["deployment_zones"].as_array().unwrap();
    let ground = zones.iter().position(|z| z["kind"] == "Ground").unwrap();
    let cells = zones[ground]["cells"].as_array().unwrap();
    let cell = cells.iter().position(|cell| {
        let dx = (cell["x"].as_i64().unwrap() - owner["position"]["x"].as_i64().unwrap()).abs();
        let dy = (cell["y"].as_i64().unwrap() - owner["position"]["y"].as_i64().unwrap()).abs();
        dx <= 1 && dy <= 1 && dx + dy > 0 && (!orthogonal || dx == 0 || dy == 0) && !units.iter().any(|u| u["position"] == *cell)
    }).expect("a real free adjacent deployment cell");
    let mut deploy = authority_step("deploy_unit", 24);
    deploy["command"]["employee_uuid"] = Value::Null;
    deploy["command"]["position"] = Value::Null;
    deploy["command"]["facing"] = json!("right");
    deploy["bindings"].as_array_mut().unwrap().extend([
        json!({"from":{"step":10,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}),
        json!({"from":{"step":20,"pointer":format!("/payload/battle_setup_snapshot/deployment_zones/{ground}/cells/{cell}")},"to":"/command/position","value_type":"object"}),
    ]);
    let mut activate = authority_step("activate_skill", 25);
    activate["command"]["employee_uuid"] = Value::Null;
    activate["command"]["skill_id"] = json!("fragment_freischutz_black_round");
    activate["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}));
    document["steps"].as_array_mut().unwrap().extend([deploy, activate]);
    for _ in 0..7 { document["steps"].as_array_mut().unwrap().push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":300,"after_seq":0}})); }
    drop(probe);
    (fixture, source, document, owner_id, owner_attack)
}

#[test]
fn composition_channel_support_world_membership_lifetime_and_source() {
    let (fixture, source, document, owner_id, owner_attack) = channel_support_world_fixture();
    let cancel = CancellationToken::default();
    let mut expected = Vec::new();
    for percent in [20, 35] {
        let current = source.replace("basic_damage_percent: 20", &format!("basic_damage_percent: {percent}"));
        let mut trial = CompositionTrial::prepare(fixture.root(), &fixture.selection(), &current,
            checked(&document), None, TrialLimits::default(), &cancel).unwrap();
        let mut active = Vec::new();
        let mut ended = false;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial.advance(&cancel).unwrap_or_else(|e| panic!("channel World step {pending}: {e:?}")).index();
            if index < 27 { continue; }
            let result = trial.session().step(index).unwrap().public_result().unwrap();
            let cp = &result["payload"]["battle_update"]["checkpoint"];
            let effect = cp["sustained_effects"]["active"].as_array().unwrap().iter().find(|r| r["is_channel"] == true);
            if let Some(effect) = effect {
                assert_eq!(effect["caster"], owner_id);
                assert_eq!(effect["members"].as_array().unwrap().len(), 1);
                assert_ne!(effect["members"][0], owner_id);
                assert!(effect["fragment_copy_uuid"].is_string());
                let owner = cp["units"].as_array().unwrap().iter().find(|u| u["unit_instance_id"] == owner_id).unwrap();
                assert_eq!(owner["stats"]["attack"], owner_attack);
                let latest = trial.session().observation().unwrap().active().unwrap().latest_sourced_sustained().unwrap();
                let authored = latest.authored_source().unwrap();
                assert_eq!(authored.source_file(), trial.identity().artifact_path);
                let origin = trial.origin_for_observed(authored).unwrap();
                assert!(origin.inputs.iter().any(|p| p == "values[0].value"));
                active.push(json!({"step":index,"effect":effect,"origin":origin}));
            } else if !active.is_empty() { ended = true; }
        }
        assert!(!active.is_empty() && ended, "channel World observations: {active:?}");
        let latest = trial.session().observation().unwrap().active().unwrap().latest_sourced_sustained().unwrap();
        assert!(matches!(latest.change(), crate::authoring::ApplicationChange::Sustained { ended:Some(crate::game::ability::sustained::SustainedEndReason::Expired), .. }));
        expected.push(json!({"percent":percent,"active":active,"ended":ended}));
    }
    if let Some(directory) = std::env::var_os("CHANNEL_SUPPORT_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "preserve earlier fixture evidence");
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), &source).unwrap();
        std::fs::write(directory.join("trial.json"), serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(directory.join("expected.json"), serde_json::to_vec_pretty(&expected).unwrap()).unwrap();
    }
}

#[path = "composition_equipment_evasion.rs"]
mod composition_equipment_evasion;

#[path = "composition_equipment_support.rs"]
mod composition_equipment_support;
#[path = "composition_heal_barrier.rs"]
mod heal_barrier;

#[path = "composition_adaptive_shield.rs"]
mod adaptive_shield;

#[path = "composition_basic_heal_echo.rs"]
mod basic_heal_echo;

include!("composition_overheal_reserve.rs");

include!("composition_deferred_damage.rs");

include!("composition_magazine.rs");
include!("composition_shared_cohort.rs");

include!("composition_cohort_bounty.rs");
include!("composition_equipment_forms.rs");

include!("composition_equipment_death_burst.rs");
include!("composition_last_will.rs");
include!("composition_weapon_stages.rs");

include!("composition_channel_opening.rs");

include!("composition_stun_end.rs");

include!("composition_lifesteal.rs");
include!("composition_primary_hp.rs");
include!("composition_ready_wait.rs");
include!("composition_nearby_defense.rs");
include!("composition_nearby_ally_damage.rs");
include!("composition_revival.rs");
include!("composition_metamorphosis.rs");

include!("composition_area_batch.rs");
include!("composition_multi_target.rs");
include!("composition_kill_cast.rs");

include!("composition_resonance_gain.rs");

#[path = "composition_actual_heal_reaction.rs"]
mod actual_heal_reaction;

#[path = "composition_threshold_control.rs"]
mod threshold_control;

#[path = "composition_basic_slow.rs"]
mod basic_slow;
#[path = "composition_recipient_support.rs"]
mod recipient_support;

#[path = "composition_incoming_flat.rs"]
mod incoming_flat;
#[path = "composition_conditional_damage.rs"]
mod conditional_damage;
#[path = "composition_direct_penetration.rs"]
mod direct_penetration;

#[path = "composition_hp_guard.rs"]
mod hp_guard;

#[path = "composition_pleasure.rs"]
mod pleasure;

#[path = "composition_distant_defense.rs"]
mod distant_defense;
#[path = "composition_shield_beneficiary.rs"]
mod shield_beneficiary;

#[path = "composition_wearer_reduction.rs"]
mod wearer_reduction;

#[path = "composition_attack_continuity.rs"]
mod attack_continuity;

#[path = "composition_palm_slow.rs"]
mod palm_slow;

#[path = "composition_high_hp_damage.rs"]
mod high_hp_damage;

include!("composition_kill_speed.rs");

#[path = "composition_chance_stun.rs"]
mod chance_stun;

#[path = "composition_headshot_resonance.rs"]
mod headshot_resonance;

#[path = "composition_idle_hp_guard.rs"]
mod idle_hp_guard;

#[path = "composition_release_gap.rs"]
mod release_gap;

#[path = "canonical_knight.rs"]
mod canonical_knight;

#[path = "canonical_boss.rs"]
mod canonical_boss;
#[path = "canonical_hatred.rs"]
mod canonical_hatred;

#[path = "canonical_pair.rs"]
mod canonical_pair;

#[path = "canonical_input_plan.rs"]
mod canonical_input_plan;

// A normal source must resolve before testing isolated corruptions. The actual
// trial tests independently assert the reward/equip copy and HP/state outcome.
fn assert_observed_origin_integrity(
    trial: &CompositionTrial,
    observed: crate::authoring::AuthoredEffectView<'_>,
) {
    assert!(trial.origin_for_observed(observed).is_some());
    let original = observed.source();
    let mut invalid = Vec::new();
    let mut changed = original.clone();
    changed.site.action.definition_id = "other-definition".into();
    invalid.push(("definition", changed));
    let mut changed = original.clone();
    changed.site.action.action_id = "other-action".into();
    invalid.push(("action", changed));
    let mut changed = original.clone();
    changed.site.effect_id = "other-effect".into();
    invalid.push(("effect", changed));
    let mut changed = original.clone();
    changed.origin_definition_id = "other-origin".into();
    invalid.push(("origin_definition", changed));
    let mut changed = original.clone();
    changed.origin = crate::game::ability::authoring::EffectOrigin::Added {
        source_definition_id: "other-origin".into(),
    };
    invalid.push(("origin_kind", changed));
    let mut changed = original.clone();
    changed.field_path.push_str(".absent_field");
    invalid.push(("field_path", changed));
    let mut changed = original.clone();
    changed.source_file.push_str("-other-generation");
    invalid.push(("generation", changed));
    for (label, changed) in invalid {
        assert!(trial.origin_for(&changed).is_none(), "accepted mixed {label}: {changed:?}");
    }
}

#[path = "composition_equipment_origin.rs"]
mod equipment_origin;
