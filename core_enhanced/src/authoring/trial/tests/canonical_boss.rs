use super::*;
use crate::game::data::content_pack::ContentSelection;
use crate::game::data::{relic_data::RelicDatabase, skill_fragment_data::SkillFragmentMetadata};

// A real encounter reward and maintenance route shared by canonical boss tests.
// Only bounded combat stats/resonance and the authored test map are substituted.
pub(super) fn boss_fixture(
    bloom: bool,
    encounter: &str,
    abnormality: &str,
    mut metadata: SkillFragmentMetadata,
    additional_relics: RelicDatabase,
) -> (ExternalFixture, Value) {
    let (fixture, _, document) = state_world_fixture();
    let path = "official/data/skill_fragments/base.ron";
    let mut fragments: crate::game::data::skill_fragment_data::SkillFragmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    // A separate bounded acceptance completion: five resonance, not release balance.
    let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
        forms, ..
    } = &mut metadata.effect
    else {
        panic!()
    };
    for form in forms.values_mut() {
        form.resonance_max = 5;
    }
    let old = fragments
        .fragments
        .iter_mut()
        .find(|f| f.id == metadata.id)
        .unwrap();
    assert_eq!(old.uuid, metadata.uuid);
    *old = metadata;
    fixture.write(path, ron::ser::to_string(&fragments).unwrap());
    let path = "official/data/events/rewards/base.ron";
    let mut rewards: crate::game::data::reward_data::RewardDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    for reward in &mut rewards.rewards {
        reward.effects.retain(|e| !matches!(e, crate::game::reward::RewardEffect::GrantSkillFragment { fragment_id } if fragment_id.as_str() == "authoring_preview_fragment"));
    }
    fixture.write(path, ron::ser::to_string(&rewards).unwrap());
    let path = "official/data/map/facility_templates.ron";
    let mut maps: crate::game::map::FacilityTemplateDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let opening = maps.layouts[0]
        .activities
        .iter_mut()
        .find(|a| a.id == "trial_intake_corridor")
        .unwrap();
    opening.kind_id = crate::game::map::MapNodeKindId::new("boss_abnormality");
    opening.payload = crate::game::map::MapNodePayload::Encounter {
        encounter_id: Some(encounter.into()),
    };
    // This authored acceptance route reserves a real Boss activity; its slot
    // must explicitly permit that category as well as the old Combat fixture.
    maps.templates
        .iter_mut()
        .find(|t| t.id.as_str() == "standard_intro_a")
        .unwrap()
        .slots
        .iter_mut()
        .find(|s| s.id.as_str() == "intake_corridor")
        .unwrap()
        .allowed_categories
        .push(crate::game::map::MapNodeCategory::Boss);
    if bloom {
        // Replace this fixture's intermediate reward room with a maintenance
        // room, retaining its position/connections and the actual boss reward.
        let room = maps.layouts[0]
            .activities
            .iter_mut()
            .find(|a| a.id == "trial_supply_annex")
            .unwrap();
        room.kind_id = crate::game::map::MapNodeKindId::new("maintenance");
        room.payload = crate::game::map::MapNodePayload::Maintenance;
        maps.templates
            .iter_mut()
            .find(|t| t.id.as_str() == "standard_intro_a")
            .unwrap()
            .slots
            .iter_mut()
            .find(|s| s.id.as_str() == "supply_annex")
            .unwrap()
            .allowed_categories
            .push(crate::game::map::MapNodeCategory::Maintenance);
    }
    fixture.write(path, ron::ser::to_string(&maps).unwrap());
    let path = "official/data/abnormalities/base.ron";
    let mut abnormalities: crate::game::data::abnormality_data::AbnormalityDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let enemy = abnormalities
        .items
        .iter_mut()
        .find(|a| a.id == abnormality)
        .unwrap();
    enemy.max_health = 70;
    enemy.attack = 0;
    enemy.defense = 0;
    enemy.magic_resist = 0;
    fixture.write(path, ron::ser::to_string(&abnormalities).unwrap());
    let path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let enemy = enemies
        .profiles
        .iter_mut()
        .find(|e| e.id == "corroded_rusher")
        .unwrap();
    enemy.max_health = 3000;
    enemy.attack = 1;
    for profile in &mut enemies.profiles {
        if profile.basic_attack_range_preset.is_some() {
            profile.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(path, ron::ser::to_string(&enemies).unwrap());
    let path = "official/data/relics/base.ron";
    let mut relics: crate::game::data::relic_data::RelicDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    for relic in additional_relics.relics {
        assert!(relics.get_by_id(&relic.id).is_none());
        relics.relics.push(relic);
    }
    fixture.write(path, ron::ser::to_string(&relics).unwrap());
    fixture.write(
        "official/data/bundle.ron",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../game_resources/data/bundle.ron"
        )),
    );
    (fixture, document)
}

pub(super) fn prepare_deployments(
    fixture: &ExternalFixture,
    selection: &ContentSelection,
    document: &mut Value,
) -> (usize, usize, usize) {
    let cancel = CancellationToken::default();
    let prep = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(25 - prep);
    let probe = TrialSession::prepare(
        crate::authoring::validate(fixture.root(), &selection, &Default::default(), &cancel)
            .unwrap(),
        checked(document),
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    let candidates = probe.step(2).unwrap().public_result().unwrap()["read_model"]
        ["game_state_context"]["candidates"]
        .as_array()
        .unwrap();
    let candidate = candidates
        .iter()
        .position(|v| v["employee_definition_id"] == "scout_runner")
        .unwrap();
    let binding = document["preparation_steps"][3]["bindings"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|b| b["to"] == "/command/employee_definition_ids/2")
        .unwrap();
    binding["from"]["pointer"] = json!(format!(
        "/read_model/game_state_context/candidates/{candidate}/employee_definition_id"
    ));
    drop(probe);
    let probe = TrialSession::prepare(
        crate::authoring::validate(fixture.root(), &selection, &Default::default(), &cancel)
            .unwrap(),
        checked(document),
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    let entry = probe.step(5).unwrap().public_result().unwrap();
    let setup = &entry["payload"]["battle_setup_snapshot"];
    let zones = setup["deployment_zones"].as_array().unwrap();
    let ground = zones.iter().position(|z| z["kind"] == "Ground").unwrap();
    let cells = zones[ground]["cells"].as_array().unwrap();
    let cell_at = |x, y| {
        cells
            .iter()
            .position(|cell| *cell == json!({"x":x,"y":y}))
            .expect("authored boss arena firing cell")
    };
    // The fixed seed selects the twin-wing arena. The first attempt left the
    // boss at (2,5), beyond either melee weapon; the rifle covers that lane.
    let positions = [(cell_at(6, 7), "left"), (cell_at(5, 6), "down")];
    let scout_cell = cell_at(2, 7);
    let scout = trial_deploy_employee_index(&probe.state.application, entry, "scout_runner");
    for (index, (cell, facing)) in [9, 10].into_iter().zip(positions) {
        document["steps"][index - prep]["command"]["facing"] = json!(facing);
        let binding = document["steps"][index - prep]["bindings"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|b| b["to"] == "/command/position")
            .unwrap();
        binding["from"]["pointer"] = json!(format!(
            "/payload/battle_setup_snapshot/deployment_zones/{ground}/cells/{cell}"
        ));
    }
    drop(probe);
    (ground, scout_cell, scout)
}

pub(super) fn insert(document: &mut Value, index: usize, step: Value) {
    fn shift(v: &mut Value, index: usize) {
        match v {
            Value::Object(m) => {
                for (k, v) in m {
                    if k == "step" && v.as_u64().is_some_and(|n| n >= index as u64) {
                        *v = json!(v.as_u64().unwrap() + 1);
                    } else {
                        shift(v, index);
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    shift(v, index);
                }
            }
            _ => (),
        }
    }
    shift(document, index);
    let prep = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .insert(index - prep, step);
}
pub(super) fn push(document: &mut Value, step: Value) -> usize {
    let i = document["preparation_steps"].as_array().unwrap().len()
        + document["steps"].as_array().unwrap().len();
    document["steps"].as_array_mut().unwrap().push(step);
    i
}
pub(super) fn clock_step(document: &mut Value, delta: u64) -> usize {
    push(
        document,
        json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":delta,"after_seq":0}}),
    )
}
