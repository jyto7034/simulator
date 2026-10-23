use super::*;
use crate::application::{ApplicationBattleAuthority, ApplicationExecutionContext};
use crate::authoring::TrialPreview;
use crate::game::{ability::SkillId, battle::event_log::SkillCastTarget, resources::Position};

#[test]
fn u07_public_external_ron_trial_creates_actor_traps_walls_and_detaches_pool() {
    for (case, object, extra, seed_pool) in [
        ("actor", "u07_trial_actor", "", false),
        ("small", "laetitia_small_friend", ",allow_same_source_evolution:true", false),
        ("big", "laetitia_big_friend", "", false),
        ("ice", "common_ice_wall", "", false),
        ("fallback", "melting_love_meat_wall", ",wall_cost:Some((pool_id:\"trial_meat\",health_cost_percent:20,wall_health_percent:200))", false),
        ("segments", "melting_love_meat_wall", ",wall_cost:Some((pool_id:\"trial_meat\",health_cost_percent:20,wall_health_percent:200))", true),
    ] {
        let (fixture, token, mut document) = public_amount_trial_fixture(|f| {
            let path = "official/data/skills/base.ron";
            let mut skills = crate::game::data::skill_data::RawSkillDatabase::from_ron("fixture", &f.read(path)).unwrap();
            let pool = if seed_pool { r#"(id:"pool",delay_ms:0,target:SelfUnit,delivery:Instant,effects:[(id:"seed",op:Sustained(definition:SegmentedShield(duration_ms:5000,pool:(pool_id:"trial_meat",segment_health:100,max_segments:3,initial_health:150,block_per_segment:1,priority:0))))]),"# } else { "" };
            let authored = format!(r#"(skills:[(id:"fragment_freischutz_black_round",name:"U07 {case}",kind:Targeted,use_lifetime:Action,focus_time_ms:100,
                cast_targeting:Explicit(selection:player_tile,target:CastTarget,reach:global,air_capable:false),steps:[{pool}
                (id:"place",delay_ms:1,target:CastTarget,delivery:Instant,effects:[(id:"create",op:CreateObject(definition:(definition_id:"{object}",anchor:CastTile,offsets:[(x:0,y:0)]{extra})))])])])"#);
            let replacement = crate::game::data::skill_data::RawSkillDatabase::from_ron("u07/trial", &authored).unwrap();
            let slot = skills.skills.iter_mut().find(|s| s.id.as_str() == "fragment_freischutz_black_round").unwrap();
            *slot = replacement.skills[0].clone();
            f.write(path, ron::ser::to_string(&skills).unwrap());
            let fragments = "official/data/skill_fragments/base.ron";
            f.write(fragments, f.read(fragments).replace("resonance_max: 25", "resonance_max: 1"));
            let path = "official/data/summoned_units/base.ron";
            let mut objects: crate::game::data::summoned_unit_data::SummonedUnitCatalogData = ron::de::from_str(&f.read(path)).unwrap();
            objects.objects.push(ron::de::from_str(r#"(id:"u07_trial_actor",base_uuid:"c0de0700-0000-0000-0000-000000000901",family:"trial_actor",display_name:"Trial Actor",ttl_ms:30000,max_alive:2,max_total:4,on_source_removed:Persist,
                kind:Actor((max_health:100,attack:10,defense:0,magic_resistance:0,movement:(),motion:Stationary,mobility:ground,basic_attack:(),resonance:(),skill_id:None)))"#).unwrap());
            f.write(path, ron::ser::to_string(&objects).unwrap());
        });
        let cancel = CancellationToken::default();
        let mut discovery = TrialSession::prepare(fixture_content(&token), checked(&document), TrialLimits::default(), &cancel).unwrap();
        while discovery.next_step_index() <= 25 { discovery.advance(&cancel).unwrap(); }
        let before = discovery.receipt_documents().map(str::to_owned).collect::<Vec<_>>();
        let work = discovery.accepted_work();
        let current = discovery.step(25).unwrap().public_result().unwrap();
        let update = &current["payload"]["battle_update"];
        let context = ApplicationExecutionContext { expected_battle_authority: Some(ApplicationBattleAuthority {
            battle_uuid: serde_json::from_value(update["battle_uuid"].clone()).unwrap(), battle_state_revision: update["battle_state_revision"].as_u64().unwrap(),
        }), ..Default::default() };
        let employee = serde_json::from_value(discovery.step(9).unwrap().public_result().unwrap()["payload"]["employee_uuid"].clone()).unwrap();
        let mut positions: Vec<Position> = serde_json::from_value(discovery.step(19).unwrap().public_result().unwrap()["payload"]["combat_preview"]["valid_tiles"].clone()).unwrap();
        let enemies: Vec<Position> = update["checkpoint"]["units"].as_array().unwrap().iter().filter(|u| u["owner"] == "Opponent").map(|u| serde_json::from_value(u["position"].clone()).unwrap()).collect();
        positions.sort_by_key(|p| std::cmp::Reverse(enemies.iter().map(|enemy| (p.x-enemy.x).abs().max((p.y-enemy.y).abs())).min().unwrap_or(0)));
        let target = positions.into_iter().find(|position| {
            let preview = discovery.preview(TrialPreview::PreviewSkillTarget { employee_uuid: employee, skill_id: SkillId::from("fragment_freischutz_black_round"), target: SkillCastTarget::Tile { position: *position } }, context.clone(), &ExecutionLimits::default(), &cancel).unwrap();
            let result: Value = serde_json::from_str(preview.document()).unwrap();
            result["payload"]["eligibility"]["enabled"] == true
        }).expect("public eligible object tile");
        assert_eq!(discovery.accepted_work(), work);
        assert_eq!(discovery.receipt_documents().map(str::to_owned).collect::<Vec<_>>(), before);
        let steps = document["steps"].as_array_mut().unwrap();
        let activate = steps.iter_mut().find(|step| step["command"]["type"] == "activate_skill").unwrap();
        activate["command"]["target"] = json!({"type":"Tile","position":target});
        activate["bindings"].as_array_mut().unwrap().retain(|b| b["to"] != "/command/target/unit_instance_id");
        if case == "small" {
            let mut begin = authority_step("begin_skill_targeting", 28);
            begin["command"]["employee_uuid"] = json!(employee); begin["command"]["skill_id"] = json!("fragment_freischutz_black_round");
            let mut again = authority_step("activate_skill", 29);
            again["command"]["employee_uuid"] = json!(employee); again["command"]["skill_id"] = json!("fragment_freischutz_black_round"); again["command"]["target"] = json!({"type":"Tile","position":target});
            steps.extend([json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":1000,"after_seq":0}}), begin, again, json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":700,"after_seq":0}})]);
        }
        let session = simulate(fixture_content(&token), checked(&document), TrialLimits::default(), &cancel).unwrap_or_else(|failure| panic!("{case}: {failure:?}"));
        let final_index = if case == "small" { 31 } else { 27 };
        let checkpoint = &session.step(final_index).unwrap().public_result().unwrap()["payload"]["battle_update"]["checkpoint"];
        let active = checkpoint["independent_objects"]["active"].as_array().unwrap();
        assert_eq!(active.len(), 1, "{case}: {checkpoint}");
        assert_eq!(active[0]["definition_id"], if case == "small" { "laetitia_big_friend" } else { object });
        assert!(active[0]["source"]["fragment_copy_uuid"].is_string());
        if case == "small" { assert_eq!(active[0]["epoch"], 2); }
        if case == "segments" {
            assert_eq!(active[0]["max_health"], 50);
            let pools = checkpoint["sustained_effects"]["active"].as_array().unwrap();
            assert!(pools.iter().any(|pool| pool["segmented"]["remaining_health"] == 100));
        }
        if case == "fallback" { assert!(active[0]["max_health"].as_u64().unwrap() > 0); }
        if let Some(directory) = std::env::var_os("U07_TRIAL_EXPORT") {
            fn copy_tree(source: &std::path::Path, target: &std::path::Path) {
                std::fs::create_dir(target).unwrap();
                for entry in std::fs::read_dir(source).unwrap() {
                    let entry = entry.unwrap();
                    let to = target.join(entry.file_name());
                    if entry.file_type().unwrap().is_dir() { copy_tree(&entry.path(), &to); }
                    else { std::fs::copy(entry.path(), to).unwrap(); }
                }
            }
            let destination = std::path::PathBuf::from(directory).join(case);
            std::fs::create_dir(&destination).unwrap();
            copy_tree(fixture.root(), &destination.join("content"));
            std::fs::write(destination.join("selection.json"), serde_json::to_vec_pretty(&fixture.selection()).unwrap()).unwrap();
            std::fs::write(destination.join("scenario.json"), serde_json::to_vec_pretty(&document).unwrap()).unwrap();
            std::fs::write(destination.join("expected_objects.json"), serde_json::to_vec_pretty(&checkpoint["independent_objects"]).unwrap()).unwrap();
        }
        let reconnect = session.reconnect(0, &ExecutionLimits::default(), &cancel).unwrap();
        let reconnect: Value = serde_json::from_str(reconnect.document()).unwrap();
        assert_eq!(reconnect["payload"]["battle_update"]["checkpoint"]["independent_objects"], checkpoint["independent_objects"]);
    }
}
