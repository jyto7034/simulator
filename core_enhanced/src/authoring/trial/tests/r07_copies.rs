use super::*;
use crate::application::{ApplicationCheckpoint, ApplicationSnapshotKind};
use crate::authoring::{AmountBattle, AmountQuery, TraceLimits, TraceOptions};

fn fixture(moment: &str) -> (ExternalFixture, ValidatedGameData, Value) {
    public_amount_trial_fixture(|f| {
        let fragments = "official/data/skill_fragments/base.ron";
        let text=f.read(fragments).replacen("forms:{\"base\":(skill_id:\"fragment_freischutz_black_round\", resonance_max: 25)},dismantle_recipe:None",
            "forms:{\"base\":(skill_id:\"fragment_freischutz_black_round\", resonance_max: 25),\"awakened\":(skill_id:\"r07_awakened\",resonance_max:25,bloom_recipe:Some((enkephalin:3,materials:[(material_id:\"r07_abnormality_part\",amount:1),(material_id:\"r07_abnormality_part\",amount:2)])))},dismantle_recipe:Some((enkephalin:2,materials:[(material_id:\"r07_abnormality_part\",amount:1)]))",1);
        f.write(fragments, text);
        let equipment = "official/data/equipments/base.ron";
        f.write(equipment,f.read(equipment).replacen("materials: [",r#"materials: [EquipmentMaterialMetadata(id:"r07_abnormality_part",uuid:"e1000000-0000-0000-0000-000000000707",name:"Freischutz part",description:"Synthetic abnormality part for R07 acceptance",material_type:AbnormalityPart,rarity:WAW,equipment_type:None),"#,1));
        let rewards = "official/data/events/rewards/base.ron";
        f.write(rewards,f.read(rewards).replacen("GrantSkillFragment(fragment_id: \"authoring_preview_fragment\"),","GrantSkillFragment(fragment_id: \"authoring_preview_fragment\"), GrantEquipmentMaterial(material_id:\"r07_abnormality_part\",amount:3),",1));
        let skill_path = "official/data/skills/base.ron";
        let skills = crate::game::data::skill_data::RawSkillDatabase::from_ron(
            "fixture",
            &f.read(skill_path),
        )
        .unwrap();
        let base = skills
            .skills
            .iter()
            .find(|s| s.id.as_str() == "fragment_freischutz_black_round")
            .unwrap();
        let mut raw = ron::ser::to_string(base)
            .unwrap()
            .replace("fragment_freischutz_black_round", "r07_awakened");
        let amount=format!("DamageExpr(amount:(moment:{moment},root:Const((value:84,unit:Magnitude))),damage_type:Magic)");
        assert!(
            raw.contains("Damage(amount:42,damage_type:Magic,)"),
            "{raw}"
        );
        raw = raw.replace("Damage(amount:42,damage_type:Magic,)", &amount);
        let lifetimes = [
            "Run",
            "Attempt",
            "Deployment",
            "Activation",
            "Delivery",
            "Chain",
        ];
        let catalog = lifetimes
            .iter()
            .map(|l| format!("(id:\"r07_{l}\",owner:FragmentCopy,lifetime:{l},initial:7,cap:15)"))
            .collect::<Vec<_>>()
            .join(",");
        f.write(
            "official/data/resources/base.ron",
            format!("(resources:[{catalog}])"),
        );
        let operations = lifetimes
            .iter()
            .map(|l| format!("(resource:\"r07_{l}\",owner:SourceFragment,operation:Consume(2))"))
            .collect::<Vec<_>>()
            .join(",");
        raw = raw.replacen(
            "effects:[",
            &format!(
                "effects:[(id:\"r07_resource_cost\",op:Resources(operations:[{operations}])) ,"
            ),
            1,
        );

        f.write(
            skill_path,
            f.read(skill_path)
                .replacen("skills: [", &format!("skills: [{raw},"), 1),
        );
        let map = "official/data/map/facility_templates.ron";
        let mut catalog: crate::game::map::FacilityTemplateDatabase = ron::de::from_str(&f.read(map)).unwrap();
        let intro = catalog.templates.iter_mut().find(|t|t.id.as_str()=="standard_intro_a").unwrap();
        intro.slots.push(ron::de::from_str(r#"(id:"r07_maintenance",role:Ordinary,allowed_categories:[Maintenance])"#).unwrap());
        intro.slots.push(ron::de::from_str(r#"(id:"r07_savepoint",role:Ordinary,allowed_categories:[SavePoint],tags:["save_point_candidate"])"#).unwrap());
        for (from,to) in [("intake_corridor","r07_maintenance"),("r07_maintenance","r07_savepoint"),("r07_savepoint","supply_annex")] {
            intro.connections.push(ron::de::from_str(&format!(r#"(from_slot_id:"{from}",to_slot_id:"{to}",direction:Bidirectional)"#)).unwrap());
        }
        f.write(map,ron::ser::to_string(&catalog).unwrap());
        let assignments = "official/data/map/node_assignment_profiles.ron";
        let mut profiles: crate::game::map::NodeAssignmentProfileDatabase = ron::de::from_str(&f.read(assignments)).unwrap();
        let intro = profiles.profiles.iter_mut().find(|p|p.id=="standard_floor_1").unwrap();
        intro.fixed_slot_assignments.push(ron::de::from_str(r#"(slot_id:"r07_maintenance",kind_id:"maintenance",payload:Maintenance)"#).unwrap());
        intro.fixed_slot_assignments.push(ron::de::from_str(r#"(slot_id:"r07_savepoint",kind_id:"save_point",payload:SavePoint)"#).unwrap());
        f.write(assignments,ron::ser::to_string(&profiles).unwrap());
    })
}
fn rewrite_indices(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if key == "step" && value.as_u64().is_some_and(|n| n >= 15) {
                    *value = json!(value.as_u64().unwrap() + 8);
                } else {
                    rewrite_indices(value);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                rewrite_indices(value);
            }
        }
        _ => {}
    }
}
fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let e = entry.unwrap();
        let target = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_tree(&e.path(), &target)
        } else {
            std::fs::copy(e.path(), target).unwrap();
        }
    }
}
#[test]
fn r07_f07_f08_f09_public_ron_recipe_forms_sources_resources_and_fresh_process() {
    const CHILD: &str = "R07_PUBLIC_RESTORE_CHILD";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = std::path::PathBuf::from(root);
        let value: Value =
            serde_json::from_slice(&std::fs::read(root.join("save.json")).unwrap()).unwrap();
        let selection = serde_json::from_value(value["selection"].clone()).unwrap();
        let token = crate::game::data::content_pack::load(
            &root.join("content"),
            &selection,
            &Default::default(),
        )
        .unwrap();
        let mut app = CoreApplication::new(token, 900);
        app.install_authoritative_player_profile(value["profile"].clone().into()).unwrap();
        let checkpoint = ApplicationCheckpoint::new(
            serde_json::to_vec(&value["checkpoint"]).unwrap(),
            0,
            value["peak"].as_u64().unwrap(),
        );
        app.continue_from_active_run_checkpoint(uuid::Uuid::from_u128(25), &checkpoint)
            .unwrap();
        assert_eq!(
            serde_json::to_value(app.snapshot(ApplicationSnapshotKind::Inventory).unwrap())
                .unwrap(),
            value["inventory"]
        );
        assert_eq!(
            serde_json::to_value(app.certified_input_fingerprint().unwrap()).unwrap(),
            value["fingerprint"]
        );
        return;
    }
    for moment in ["Snapshot", "Live"] {
        let (fixture, token, mut document) = fixture(moment);
        let cancel = CancellationToken::default();
        let baseline = simulate(
            fixture_content(&token),
            checked(&document),
            TrialLimits {
                amount_observation: true,
                ..Default::default()
            },
            &cancel,
        )
        .unwrap();
        let baseline_amount = baseline
            .preview_amount(
                AmountQuery::Latest {
                    battle: AmountBattle::LastTerminal,
                },
                &ExecutionLimits::default(),
                &cancel,
            )
            .unwrap();
        assert!(baseline_amount.document().contains("\"base\""));
        let map = &baseline.step(13).unwrap().public_result().unwrap()["payload"]["completion"]
            ["payload"]["map"]["nodes"];
        let nodes = map.as_array().unwrap();
        let maintenance = nodes
            .iter()
            .find(|n| n["slot_id"] == "r07_maintenance")
            .unwrap()["id"]
            .clone();
        let savepoint = nodes
            .iter()
            .find(|n| n["slot_id"] == "r07_savepoint")
            .unwrap()["id"]
            .clone();
        let copy = baseline.step(13).unwrap().public_result().unwrap()["payload"]
            ["skill_fragment_diffs"][0]["copy_uuid"]
            .clone();
        assert!(copy.is_string());
        rewrite_indices(&mut document);
        for step in document["steps"].as_array_mut().unwrap().iter_mut() {
            if step["command"]["skill_id"] == "fragment_freischutz_black_round" {
                step["command"]["skill_id"] = json!("r07_awakened");
            }
        }
        let inserted = [
            json!({"kind":"command","command":{"type":"select_map_node","node_id":maintenance}}),
            json!({"kind":"command","command":{"type":"confirm_enter_node"}}),
            json!({"kind":"command","command":{"type":"awaken_skill_fragment","copy_uuid":copy,"form_id":"awakened"}}),
            json!({"kind":"command","command":{"type":"select_skill_fragment_form","copy_uuid":copy,"form_id":"awakened"}}),
            json!({"kind":"command","command":{"type":"complete_node"}}),
            json!({"kind":"command","command":{"type":"select_map_node","node_id":savepoint}}),
            json!({"kind":"command","command":{"type":"confirm_enter_node"}}),
            json!({"kind":"command","command":{"type":"complete_node"}}),
        ];
        document["steps"]
            .as_array_mut()
            .unwrap()
            .splice(6..6, inserted);
        let mut query_documents = Vec::new();
        for traced in [false, true] {
            let mut session = TrialSession::prepare(
                fixture_content(&token),
                checked(&document),
                TrialLimits {
                    amount_observation: true,
                    trace: if traced {
                        TraceOptions {
                            limits: Some(TraceLimits::default()),
                        }
                    } else {
                        TraceOptions::default()
                    },
                    ..Default::default()
                },
                &cancel,
            )
            .unwrap();
            let mut saved = false;
            let mut fingerprints = Vec::new();
            while !session.is_complete() {
                let index = session.next_step_index();
                if index == 17 {
                    let before = session
                        .receipt_documents()
                        .map(str::to_owned)
                        .collect::<Vec<_>>();
                    let work = session.accepted_work();
                    let stopped = CancellationToken::default();
                    stopped.cancel();
                    assert!(session.advance(&stopped).is_err());
                    let mid = CancellationToken::default();
                    let result = control::with_test_cancellation("trial_step_record", 1, || {
                        session.advance(&mid)
                    });
                    assert!(mid.is_cancelled());
                    assert!(result.is_err());
                    assert_eq!(session.next_step_index(), index);
                    assert_eq!(session.accepted_work(), work);
                    assert_eq!(
                        session
                            .receipt_documents()
                            .map(str::to_owned)
                            .collect::<Vec<_>>(),
                        before
                    );
                }
                session.advance(&cancel).unwrap();
                if [14, 17, 18].contains(&index) {
                    fingerprints.push(
                        session
                            .state
                            .application
                            .certified_input_fingerprint()
                            .unwrap(),
                    );
                }
                if index == 17 {
                    let result = session.step(index).unwrap().public_result().unwrap();
                    assert_eq!(result["payload"]["copy"]["selected_form"], "base");
                    assert_eq!(result["payload"]["enkephalin_spent"], 3);
                    assert_eq!(
                        result["payload"]["inventory_diff"]["material_stacks"][0]["amount"],
                        0
                    );
                }
                if index == 22 && !traced {
                    let export = session
                        .state
                        .application
                        .export_active_run_checkpoint()
                        .unwrap()
                        .unwrap();
                    let dir =
                        std::env::temp_dir().join(format!("r07-save-{}", uuid::Uuid::new_v4()));
                    copy_tree(fixture.root(), &dir.join("content"));
                    let save = json!({"profile":session.state.application.snapshot(ApplicationSnapshotKind::PlayerProfile).unwrap().read_model().clone(),"selection":fixture.selection(),"checkpoint":serde_json::from_slice::<Value>(export.core_payload_json()).unwrap(),"peak":export.run_peak_certified_power(),"inventory":session.state.application.snapshot(ApplicationSnapshotKind::Inventory).unwrap(),"fingerprint":session.state.application.certified_input_fingerprint().unwrap()});
                    std::fs::write(dir.join("save.json"), serde_json::to_vec(&save).unwrap())
                        .unwrap();
                    let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","authoring::trial::tests::r07_copies::r07_f07_f08_f09_public_ron_recipe_forms_sources_resources_and_fresh_process","--nocapture"]).env(CHILD,&dir).output().unwrap();
                    assert!(
                        output.status.success(),
                        "{}\n{}",
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                    std::fs::remove_dir_all(dir).unwrap();
                    saved = true;
                }
            }
            if !traced {
                assert!(saved);
            }
            assert_ne!(fingerprints[0], fingerprints[1]);
            assert_ne!(fingerprints[1], fingerprints[2]);
            let query = AmountQuery::Latest {
                battle: AmountBattle::LastTerminal,
            };
            let amount = session
                .preview_amount(query, &ExecutionLimits::default(), &cancel)
                .unwrap();
            let explained = session
                .explain_amount(query, &ExecutionLimits::default(), &cancel)
                .unwrap();
            let value: Value = serde_json::from_str(amount.document()).unwrap();
            assert_eq!(value["source"]["fragment"]["copy_uuid"], copy, "{value}");
            assert_eq!(value["source"]["fragment"]["form_id"], "awakened");
            assert_eq!(value["source"]["fragment"]["skill_id"], "r07_awakened");
            let facts = serde_json::to_value(
                session
                    .state
                    .application
                    .battle_runtime_facts()
                    .unwrap()
                    .unwrap(),
            )
            .unwrap();
            assert!(facts.to_string().contains("fragment_copy"), "{facts}");
            assert!(facts.to_string().contains("r07_Chain"));
            assert!(facts["active_actions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|action| action["facts"]["cost_mutations"] == 7));

            if traced {
                let trace = session.trace().unwrap().last_terminal().unwrap();
                let text = std::str::from_utf8(trace.ndjson()).unwrap();
                assert!(text.contains("awakened"));
                assert!(text.contains(copy.as_str().unwrap()));
            }
            for mode in 0..3 {
                let mut limits = ExecutionLimits::default();
                match mode {
                    0 => limits.max_work = amount.usage().work,
                    1 => limits.max_metadata_bytes = amount.usage().metadata_bytes_peak,
                    _ => limits.max_result_bytes = amount.usage().result_bytes_peak,
                };
                assert_eq!(
                    session
                        .preview_amount(query, &limits, &cancel)
                        .unwrap()
                        .document(),
                    amount.document()
                );
                match mode {
                    0 => limits.max_work -= 1,
                    1 => limits.max_metadata_bytes -= 1,
                    _ => limits.max_result_bytes -= 1,
                };
                assert!(session.preview_amount(query, &limits, &cancel).is_err());
                assert_eq!(
                    session
                        .preview_amount(query, &ExecutionLimits::default(), &cancel)
                        .unwrap()
                        .document(),
                    amount.document()
                );
            }
            query_documents.push((
                amount.document().to_owned(),
                explained.document().to_owned(),
            ));
        }
        assert_eq!(query_documents[0], query_documents[1]);
        if let Some(root) = std::env::var_os("R07_PUBLIC_CLI_EXPORT") {
            let root = std::path::PathBuf::from(root).join(moment);
            assert!(!root.exists());
            copy_tree(fixture.root(), &root.join("content"));
            std::fs::write(
                root.join("selection.json"),
                serde_json::to_vec(&fixture.selection()).unwrap(),
            )
            .unwrap();
            std::fs::write(
                root.join("scenario.json"),
                serde_json::to_vec(&document).unwrap(),
            )
            .unwrap();
        }
    }
}
#[test]
fn r07_f04_f12_public_ron_invalid_forms_recipes_and_removed_commands_reject() {
    let original = r#"forms: { "base": (skill_id: "fragment_freischutz_black_round", resonance_max: 100), "awakened": (skill_id: "freischutz_magic_bullet", resonance_max: 100) }"#;
    for replacement in [
        r#"forms: {"awakened":(skill_id:"freischutz_magic_bullet", resonance_max: 100)}"#,
        r#"forms: {"base":(skill_id:"missing", resonance_max: 100)}"#,
        r#"forms: {"base":(skill_id:"fragment_freischutz_black_round", resonance_max: 100,bloom_recipe:Some((enkephalin:1,materials:[])))}"#,
        r#"forms: {"base":(skill_id:"fragment_freischutz_black_round", resonance_max: 100),"x":(skill_id:"freischutz_magic_bullet", resonance_max: 100,bloom_recipe:Some((enkephalin:0,materials:[(material_id:"missing",amount:1)])))}"#,
        r#"forms: {"base":(skill_id:"fragment_freischutz_black_round", resonance_max: 100),"x":(skill_id:"freischutz_magic_bullet", resonance_max: 100,bloom_recipe:Some((enkephalin:0,materials:[(material_id:"equipment_dust",amount:4294967295),(material_id:"equipment_dust",amount:1)])))}"#,
        r#"forms: {"base":(skill_id:"fragment_freischutz_black_round", resonance_max: 100,compatibility:Some((allowed_weapon_archetypes:[Gun,Gun])))}"#,
        r#"forms: {"base":(skill_id:"fragment_freischutz_black_round", resonance_max: 100,unknown:true)}"#,
        r#"forms: {"base":(skill_id:"fragment_freischutz_black_round", resonance_max: 100),"base":(skill_id:"freischutz_magic_bullet", resonance_max: 100)}"#,
    ] {
        let fixture = ExternalFixture::official_with_trial_route();
        let path = "official/data/skill_fragments/base.ron";
        let text = fixture.read(path);
        assert!(text.contains(original));
        fixture.write(path, text.replacen(original, replacement, 1));
        assert!(
            crate::game::data::content_pack::load(
                fixture.root(),
                &fixture.selection(),
                &Default::default()
            )
            .is_err(),
            "{replacement}"
        );
    }
    for command in [
        json!({"type":"upgrade_skill_fragment","fragment_id":"fragment_freischutz"}),
        json!({"type":"equip_skill_fragment","employee_uuid":uuid::Uuid::nil(),"fragment_id":"fragment_freischutz"}),
    ] {
        assert!(crate::application::ApplicationCommand::from_document(
            crate::application::ApplicationDocument::new(command)
        )
        .is_err());
    }
}
