use super::*;

#[test]
fn authored_pack_world_equipped_copy_runs_published_yaml_and_commits_rewards() {
    use crate::game::data::content_pack::{
        package_official_with_skill_files, ContentSelection, InputLimits,
    };
    let (fixture, _, mut document) = state_world_fixture();
    fixture.write(
        "official/data/bundle.ron",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../game_resources/data/bundle.ron"
        )),
    );
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(25 - preparation);
    document["steps"].as_array_mut().unwrap().push(json!({
        "kind":"clock", "clock":{"mode":"elapsed","delta_ms":20000,"after_seq":0}
    }));
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/equipment_attack.skill.yaml")
            .replace(
                "equipment: r11_weapon",
                "equipment: standard_training_sword",
            );
    let source_path = "data/skills/authored-weapon.skill.yaml";
    let cancel = CancellationToken::default();
    let mut identities = Vec::new();
    let mut results = Vec::new();
    for (grant, heal) in [(2, 7), (8, 5)] {
        let current = source
            .replace("Grant: 2", &format!("Grant: {grant}"))
            .replace(
                "value: 7, unit: Magnitude",
                &format!("value: {heal}, unit: Magnitude"),
            );
        fixture.write(&format!("official/{source_path}"), &current);
        let name = format!("published-{grant}");
        let receipt = package_official_with_skill_files(
            &fixture.root().join("official"),
            &fixture.root().join(&name),
            &[source_path.into()],
            &InputLimits::default(),
        )
        .unwrap();
        let selection = ContentSelection {
            packs: vec![name.clone().into()],
            winners: vec![],
        };
        let content = crate::authoring::validate(
            fixture.root(),
            &selection,
            &crate::authoring::AuthoringLimits::default(),
            &cancel,
        )
        .unwrap();
        assert_eq!(content.token().identity(), &receipt.content_identity);
        identities.push(receipt.content_identity);
        let mut trial =
            TrialSession::prepare(content, checked(&document), TrialLimits::default(), &cancel)
                .unwrap();
        // An edit to the original cannot replace the already published bytes or
        // the generation held by a running World.
        fixture.write(&format!("official/{source_path}"), "version: [");
        let rejected = fixture.root().join(format!("rejected-{grant}"));
        assert!(package_official_with_skill_files(
            &fixture.root().join("official"),
            &rejected,
            &[source_path.into()],
            &InputLimits::default()
        )
        .is_err());
        assert!(!rejected.exists());
        let mut before = None;
        let mut rewards = Vec::new();
        let mut ordinal = None;
        while !trial.is_complete() {
            let pending = trial.next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("published World step {pending}: {e:?}"))
                .index();
            let observation = trial.observation().unwrap();
            if index == 24 {
                before = Some(observation.run_resources().unwrap().clone());
            }
            if index < 25 {
                continue;
            }
            let Some(observed) = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
                .and_then(|o| o.latest_nonzero_sourced_hp())
            else {
                continue;
            };
            if ordinal == Some(observed.ordinal()) {
                continue;
            }
            ordinal = Some(observed.ordinal());
            let Some(origin) = observed.authored_source() else {
                continue;
            };
            if !origin.definition_id().starts_with("equipment:") {
                continue;
            }
            assert!(observed.source().is_none());
            assert_eq!(origin.source_file(), format!("{name}/{source_path}"));
            let equipment = observed.equipment_source().unwrap();
            assert!(equipment.persistent_copy_uuid.is_some());
            rewards.push(json!({"heal":observed.change().hp_delta(),"equipment":equipment}));
        }
        let observation = trial.observation().unwrap();
        assert!(observation.active().is_none() && observation.last_terminal().is_some());
        assert_eq!(rewards.len(), 1, "{rewards:?}");
        assert_eq!(rewards[0]["heal"], json!(heal));
        let run = observation.run_resources().unwrap();
        let copy = uuid::Uuid::parse_str(
            rewards[0]["equipment"]["persistent_copy_uuid"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        let owner = crate::game::resource::ResourceOwner::EquipmentCopy(copy);
        let matches = |e: &&crate::game::resource::RunResourceEntry| {
            e.resource.as_str() == "weapon_mass" && e.owner == owner
        };
        let prior = before
            .as_ref()
            .unwrap()
            .entries
            .iter()
            .find(matches)
            .map_or(0, |e| e.balance);
        let balance = run.entries.iter().find(matches).unwrap().balance;
        assert_eq!(balance, prior + grant);
        results.push(
            json!({"grant":grant,"heal":heal,"prior":prior,"balance":balance,"rewards":rewards}),
        );
    }
    assert_ne!(identities[0], identities[1]);
    eprintln!("published YAML World results: {results:?}");
}
