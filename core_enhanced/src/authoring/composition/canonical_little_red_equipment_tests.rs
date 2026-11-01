use super::*;
use serde_json::{json, Value as Json};

pub(crate) const ORIGINAL: &str =
    include_str!("../../../../game_resources/authoring/little-red/equipment.skill.yaml");
pub(crate) const CATALOG: &str =
    include_str!("../../../../game_resources/authoring/little-red/equipment.catalog.json");
fn inputs() -> Json {
    serde_json::from_str::<Json>(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_little_red_equipment.values.json"
    ))
    .unwrap()["values"]
        .clone()
}
pub(crate) fn resolved() -> Json {
    let mut d = parse_yaml(ORIGINAL).unwrap();
    super::canonical_area_tests::replace_holes(&mut d, &inputs());
    d
}
pub(crate) fn catalog() -> Vec<crate::game::data::equipment_data::EquipmentMetadata> {
    let mut d: Json = serde_json::from_str(CATALOG).unwrap();
    super::canonical_area_tests::replace_holes(&mut d, &inputs());
    serde_json::from_value(d).unwrap()
}
pub(crate) fn artifact() -> Artifact {
    let a = compile_json(&resolved().to_string()).unwrap();
    Artifact::from_ron(&a.to_ron().unwrap()).unwrap()
}

#[test]
fn canonical_little_red_equipment_preserves_unknowns_registration_and_shared_profile() {
    let original = parse_yaml(ORIGINAL).unwrap();
    let document = resolved();
    let mut holes = vec![];
    super::canonical_area_tests::holes(&original, "", &mut holes);
    assert_eq!(holes.len(), 21);
    for (path, marker) in holes {
        let mut one = document.clone();
        *one.pointer_mut(&path).unwrap() = marker;
        assert_eq!(
            compile_json(&one.to_string()).unwrap_err().code,
            DiagnosticCode::Unresolved,
            "{path}"
        );
    }
    assert_eq!(
        compile_yaml(ORIGINAL).unwrap_err().code,
        DiagnosticCode::Unresolved
    );
    let artifact = artifact();
    assert_eq!(artifact.equipment_attacks().len(), 2);
    assert_eq!(artifact.equipment_defenses().len(), 1);
    assert_eq!(artifact.resources().len(), 2);
    assert!(artifact.load_native().unwrap().skills.is_empty());
    let mut fragment = super::canonical_input_plan_tests::resolved();
    assert_eq!(
        document["values"][2]["value"]["AttackProgram"]["cohort"]["spawn"],
        super::canonical_input_plan_tests::program(&mut fragment)["cohort"]
    );
    let equipment = catalog();
    let registered: crate::game::data::equipment_data::EquipmentDatabase = ron::de::from_str(
        include_str!("../../../../game_resources/data/equipments/base.ron"),
    )
    .unwrap();
    for item in &equipment {
        let old = registered.get_by_id(&item.id).unwrap();
        assert_eq!(item.uuid, old.uuid);
        assert_eq!(item.equipment_type, old.equipment_type);
        assert_eq!(item.content_family.as_deref(), Some("f-01-57_little_red"));
    }
    let gun = equipment[0].weapon_profile.as_ref().unwrap();
    assert_eq!(
        gun.weapon_archetype,
        crate::game::data::equipment_data::WeaponArchetype::Gun
    );
    assert_eq!(
        gun.damage_type,
        crate::game::battle::damage::DamageType::Physical
    );
    assert_eq!(gun.block_capacity, 0);
    assert!(
        serde_json::from_str::<Vec<crate::game::data::equipment_data::EquipmentMetadata>>(CATALOG)
            .is_err()
    );
    let sources: crate::game::data::pve_data::PveEncounterDatabase = ron::de::from_str(
        include_str!("../../../../game_resources/data/pve/encounters.ron"),
    )
    .unwrap();
    let rewards = sources
        .reward_sources
        .iter()
        .find(|s| s.encounter_id == "suppress_little_red")
        .unwrap();
    let revival = rewards.revival.as_ref().unwrap();
    for stage in [&revival.stage_2, &revival.stage_3] {
        assert_eq!(
            stage.equipment_choices,
            equipment.iter().map(|e| e.id.clone()).collect::<Vec<_>>()
        );
    }
}

#[test]
fn canonical_little_red_equipment_retains_verified_programs_with_explicit_bindings() {
    let actual = artifact();
    for (name, text) in [
        (
            "weapon",
            include_str!("../../../tests/fixtures/skill_composition/magazine.skill.yaml"),
        ),
        (
            "armor",
            include_str!("../../../tests/fixtures/skill_composition/deferred_damage.skill.yaml"),
        ),
        (
            "accessory",
            include_str!("../../../tests/fixtures/skill_composition/cohort_bounty.skill.yaml"),
        ),
    ] {
        let mut d = parse_yaml(text).unwrap();
        if name == "armor" {
            d["equipment_defenses"][0]["equipment"] = json!("placeholder_little_red_armor");
        } else {
            d["equipment_attacks"][0]["equipment"] =
                json!(format!("placeholder_little_red_{name}"));
            let p = &mut d["values"][0]["value"]["AttackProgram"];
            if name == "weapon" {
                p["magazine"]["reward_family"] = json!("f-01-57_little_red");
            } else {
                p["cohort"]["spawn"]["weakness_family"] = json!("f-01-57_little_red");
                p["cohort"]["bounty"]["pending"] = json!("little_red_bounty_pending");
                p["cohort"]["bounty"]["confirmed"] = json!("little_red_bounty_confirmed");
                for resource in d["resources"].as_array_mut().unwrap() {
                    resource["id"] =
                        json!(format!("little_red_{}", resource["id"].as_str().unwrap()));
                }
            }
        }
        let old = compile_json(&d.to_string()).unwrap();
        if name == "armor" {
            assert_eq!(
                serde_json::to_value(actual.equipment_defenses()).unwrap(),
                serde_json::to_value(old.equipment_defenses()).unwrap()
            );
        } else {
            let expected = &old.equipment_attacks()[0];
            let found = actual
                .equipment_attacks()
                .iter()
                .find(|b| b.equipment == expected.equipment)
                .unwrap();
            assert_eq!(
                serde_json::to_value(found).unwrap(),
                serde_json::to_value(expected).unwrap()
            );
            if name == "accessory" {
                assert_eq!(
                    serde_json::to_value(actual.resources()).unwrap(),
                    serde_json::to_value(old.resources()).unwrap()
                );
            }
        }
    }
}
