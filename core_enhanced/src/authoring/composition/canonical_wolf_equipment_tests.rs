use super::*;
use serde_json::{json, Value as Json};

pub(crate) const ORIGINAL: &str =
    include_str!("../../../../game_resources/authoring/big-bad-wolf/equipment.skill.yaml");
pub(crate) const CATALOG: &str =
    include_str!("../../../../game_resources/authoring/big-bad-wolf/equipment.catalog.json");
fn values() -> Json {
    serde_json::from_str::<Json>(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_wolf_equipment.values.json"
    ))
    .unwrap()["values"]
        .clone()
}
pub(crate) fn resolved() -> Json {
    let mut d = parse_yaml(ORIGINAL).unwrap();
    super::canonical_area_tests::replace_holes(&mut d, &values());
    d
}
pub(crate) fn catalog() -> crate::game::data::equipment_data::EquipmentMetadata {
    let mut d: Json = serde_json::from_str(CATALOG).unwrap();
    super::canonical_area_tests::replace_holes(&mut d, &values());
    serde_json::from_value(d).unwrap()
}
#[test]
fn canonical_wolf_equipment_preserves_unknowns_and_existing_reward_identity() {
    let original = parse_yaml(ORIGINAL).unwrap();
    let completed = resolved();
    let mut holes = vec![];
    super::canonical_area_tests::holes(&original, "", &mut holes);
    assert_eq!(holes.len(), 16);
    for (path, marker) in holes {
        let mut one = completed.clone();
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
    let metadata = catalog();
    let registered: crate::game::data::equipment_data::EquipmentDatabase = ron::de::from_str(
        include_str!("../../../../game_resources/data/equipments/base.ron"),
    )
    .unwrap();
    let old = registered.get_by_id(&metadata.id).unwrap();
    assert_eq!(
        (metadata.uuid, metadata.equipment_type),
        (old.uuid, old.equipment_type)
    );
    assert_eq!(
        metadata.content_family.as_deref(),
        Some("f-02-58_big_bad_wolf")
    );
    assert!(metadata.weapon_profile.is_none());
    assert!(metadata.triggered_effects.is_empty());
    assert!(
        serde_json::from_str::<crate::game::data::equipment_data::EquipmentMetadata>(CATALOG)
            .is_err()
    );
    let rewards: crate::game::data::pve_data::PveEncounterDatabase = ron::de::from_str(
        include_str!("../../../../game_resources/data/pve/encounters.ron"),
    )
    .unwrap();
    let source = rewards
        .reward_sources
        .iter()
        .find(|s| s.reward_source_id == "normal_boss_big_bad_wolf")
        .unwrap();
    assert!(source.fragment_id.is_none());
    assert!(source.first_equipment_ids.is_empty());
    let revival = source.revival.as_ref().unwrap();
    for stage in [&revival.stage_2, &revival.stage_3] {
        assert_eq!(stage.equipment_choices, vec![metadata.id.clone()]);
    }
}
#[test]
fn canonical_wolf_equipment_preserves_verified_four_form_program() {
    let artifact = compile_json(&resolved().to_string()).unwrap();
    let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    assert!(artifact.load_native().unwrap().skills.is_empty());
    assert!(artifact.equipment_defenses().is_empty());
    assert_eq!(artifact.equipment_attacks().len(), 1);
    assert_eq!(artifact.resources().len(), 2);
    let mut old = parse_yaml(include_str!(
        "../../../tests/fixtures/skill_composition/equipment_forms.skill.yaml"
    ))
    .unwrap();
    old["equipment_attacks"][0]["equipment"] = json!("placeholder_big_bad_wolf_accessory");
    old["values"][0]["value"]["AttackProgram"]["victory_forms"]["pending"] =
        json!("wolf_mask_pending");
    old["values"][0]["value"]["AttackProgram"]["victory_forms"]["confirmed"] =
        json!("wolf_mask_wins");
    for resource in old["resources"].as_array_mut().unwrap() {
        resource["id"] = json!(format!("wolf_{}", resource["id"].as_str().unwrap()));
    }
    let expected = compile_json(&old.to_string()).unwrap();
    assert_eq!(
        serde_json::to_value(artifact.equipment_attacks()).unwrap(),
        serde_json::to_value(expected.equipment_attacks()).unwrap()
    );
    assert_eq!(
        serde_json::to_value(artifact.resources()).unwrap(),
        serde_json::to_value(expected.resources()).unwrap()
    );
}
