use super::canonical_area_tests::{holes, replace_holes};
use super::*;
use serde_json::{json, Value as Json};

pub(crate) const FRAGMENT: &str = "fragment_funeral_butterfly_eulogy";
pub(crate) const WEAPON: &str = "placeholder_funeral_butterfly_weapon";
pub(crate) const FAMILY: &str = "t-02-99_funeral_dying_butterflies";
pub(crate) fn original(form: &str) -> &'static str {
    match form {
        "base" => include_str!("../../../../game_resources/authoring/funeral-of-dying-butterflies/base.skill.yaml"),
        "bloom" => include_str!("../../../../game_resources/authoring/funeral-of-dying-butterflies/bloom.skill.yaml"),
        "equipment" => include_str!("../../../../game_resources/authoring/funeral-of-dying-butterflies/equipment.skill.yaml"),
        _ => panic!("unknown fixture source"),
    }
}
pub(crate) fn inputs() -> Json {
    serde_json::from_str::<Json>(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_funeral.values.json"
    ))
    .unwrap()["values"]
        .clone()
}
pub(crate) fn resolved(form: &str) -> Json {
    let mut d = parse_yaml(original(form)).unwrap();
    replace_holes(&mut d, &inputs());
    d
}
pub(crate) const FRAGMENT_CATALOG: &str = include_str!(
    "../../../../game_resources/authoring/funeral-of-dying-butterflies/fragment.catalog.json"
);
pub(crate) const EQUIPMENT_CATALOG: &str = include_str!(
    "../../../../game_resources/authoring/funeral-of-dying-butterflies/equipment.catalog.json"
);
pub(crate) fn fragment() -> crate::game::data::skill_fragment_data::SkillFragmentMetadata {
    let mut d: Json = serde_json::from_str(FRAGMENT_CATALOG).unwrap();
    replace_holes(&mut d, &inputs());
    serde_json::from_value(d).unwrap()
}
pub(crate) fn equipment() -> Vec<crate::game::data::equipment_data::EquipmentMetadata> {
    let mut d: Json = serde_json::from_str(EQUIPMENT_CATALOG).unwrap();
    replace_holes(&mut d, &inputs());
    serde_json::from_value(d).unwrap()
}
pub(crate) fn relics() -> crate::game::data::relic_data::RelicDatabase {
    let mut d: Json = serde_json::from_str(include_str!(
        "../../../../game_resources/authoring/relics/tile-burst-modifiers.catalog.json"
    ))
    .unwrap();
    replace_holes(&mut d, &inputs());
    let mut result: crate::game::data::relic_data::RelicDatabase =
        serde_json::from_value(d).unwrap();
    result.relics.extend(
        super::canonical_knight_tests::relics()
            .relics
            .into_iter()
            .filter(|r| ["widened-horizon", "deep-imprint"].contains(&r.id.as_str())),
    );
    result
}
pub(crate) fn program(d: &mut Json) -> &mut Json {
    &mut d["values"][0]["value"]["Effects"][0]["op"]["TileBurst"]["program"]
}

/// Existing BattleCore harness identities only; every behavior and companion
/// comes from the canonical source/catalog with explicit technical completion.
pub(crate) fn battle_fixture_content(
    form: &str,
) -> crate::authoring::node_graph::NativeGraphContent {
    let mut d = resolved(if form.is_empty() { "base" } else { "bloom" });
    d["skills"][0]["id"] = json!("r11_aura");
    if !form.is_empty() {
        program(&mut d)["source"]["WeaponStacks"]["weapon"] = json!("r11_weapon");
    }
    let mut marks = equipment()[0]
        .weapon_profile
        .as_ref()
        .unwrap()
        .mark_weapon
        .clone()
        .unwrap();
    marks.evolution.fragment = "r09_fragment".into();
    crate::authoring::node_graph::NativeGraphContent {
        skills: compile_json(&d.to_string()).unwrap().load_native().unwrap(),
        equipment: vec![],
        mark_weapons: serde_json::from_value(json!([{"weapon":"r11_weapon", "definition":marks}]))
            .unwrap(),
    }
}

#[test]
fn canonical_funeral_sources_preserve_unknowns_and_relic_gated_tile_programs() {
    use crate::game::ability::{SkillEffectDef, SkillUseLifetime};
    for (form, count) in [("base", 6), ("bloom", 14), ("equipment", 20)] {
        let raw = parse_yaml(original(form)).unwrap();
        let d = resolved(form);
        let mut pending = vec![];
        holes(&raw, "", &mut pending);
        assert_eq!(pending.len(), count, "{form}");
        assert_eq!(
            compile_yaml(original(form)).unwrap_err().code,
            DiagnosticCode::Unresolved
        );
        for (path, marker) in pending {
            let mut one = d.clone();
            *one.pointer_mut(&path).unwrap() = marker;
            let error = compile_json(&one.to_string()).unwrap_err();
            assert_eq!(
                error.code,
                DiagnosticCode::Unresolved,
                "{form}/{path}: {error:?}"
            );
            assert!(!error.source.definition.is_empty());
        }
        let artifact = compile_json(&d.to_string()).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        if form == "equipment" {
            assert_eq!(artifact.equipment_attacks().len(), 2);
            assert_eq!(artifact.equipment_defenses().len(), 1);
            assert!(artifact.load_native().unwrap().skills.is_empty());
        } else {
            let native = artifact.load_native().unwrap();
            assert_eq!(native.skills[0].use_lifetime, SkillUseLifetime::Action);
            let SkillEffectDef::TileBurst { program: actual } =
                &native.skills[0].steps[0].effects[0].op
            else {
                panic!()
            };
            let expected = crate::authoring::node_graph::compile(
                &crate::authoring::node_graph::tests::tile_burst_fixture(if form == "base" {
                    ""
                } else {
                    "-orb"
                })
                .to_string(),
                "legacy".into(),
            )
            .unwrap()
            .load_content()
            .unwrap()
            .skills;
            let SkillEffectDef::TileBurst { program: expected } =
                &expected.skills[0].steps[0].effects[0].op
            else {
                panic!()
            };
            let mut expected = expected.as_ref().clone();
            if let crate::game::ability::tile_burst::TileBurstSource::WeaponStacks {
                weapon, ..
            } = &mut expected.source
            {
                *weapon = WEAPON.into();
            }
            for modifier in &mut expected.modifiers {
                modifier.id = match modifier.relic_definition.as_str() {
                    "widened-horizon" => "funeral-range",
                    "overlapping-explosions" => "funeral-overlap",
                    "explosion-vulnerability" => "funeral-power",
                    _ => panic!(),
                }
                .into();
            }
            assert_eq!(actual.as_ref(), &expected);
            assert_eq!(native.skills[0].resonance_admission, expected_admission());
        }
    }
}
fn expected_admission() -> Option<crate::game::ability::resonance::ResonanceAdmission> {
    crate::authoring::node_graph::compile(
        &crate::authoring::node_graph::tests::tile_burst_fixture("").to_string(),
        "legacy".into(),
    )
    .unwrap()
    .load_content()
    .unwrap()
    .skills
    .skills[0]
        .resonance_admission
}

#[test]
fn canonical_funeral_catalog_keeps_copy_identity_and_declares_variant_and_evolution() {
    let gear = equipment();
    let old: crate::game::data::equipment_data::EquipmentDatabase = ron::de::from_str(
        include_str!("../../../../game_resources/data/equipments/base.ron"),
    )
    .unwrap();
    for g in &gear {
        let prior = old.get_by_id(&g.id).unwrap();
        assert_eq!(
            (g.uuid, g.equipment_type),
            (prior.uuid, prior.equipment_type)
        );
        assert_eq!(g.content_family.as_deref(), Some(FAMILY));
    }
    let f = fragment();
    assert_eq!(f.id.as_str(), FRAGMENT);
    assert_eq!(f.uuid.to_string(), "710e8400-e29b-41d4-a716-446655441006");
    assert!(
        matches!(&f.origin, Some(crate::game::data::skill_fragment_data::SkillFragmentOrigin::Abnormality { abnormality_id }) if abnormality_id == FAMILY)
    );
    let gun = gear[0].weapon_profile.as_ref().unwrap();
    assert_eq!(
        gun.weapon_archetype,
        crate::game::data::equipment_data::WeaponArchetype::Gun
    );
    assert_eq!(gun.block_capacity, 0);
    let marks = gun.mark_weapon.as_ref().unwrap();
    marks.validate().unwrap();
    assert_eq!(marks.evolution.fragment, FRAGMENT);
    assert_eq!(marks.evolution.form, "bloom");
    assert_eq!(marks.upgraded.amount, 1);
    assert_eq!(marks.modifiers[0].relic_definition, "deep-imprint");
    assert_eq!(marks.modifiers[0].multiplier.numerator, 2);
    let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
        forms, ..
    } = &f.effect
    else {
        panic!()
    };
    for form in ["base", "bloom"] {
        let skill = compile_json(&resolved(form).to_string())
            .unwrap()
            .load_native()
            .unwrap()
            .skills
            .remove(0);
        assert_eq!(forms[form].skill_id, skill.id);
    }
    let relics = relics();
    assert_eq!(relics.relics.len(), 5);
    assert_eq!(
        relics
            .relics
            .iter()
            .map(|r| &r.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        5
    );
    for raw in [FRAGMENT_CATALOG, EQUIPMENT_CATALOG] {
        let mut unresolved = vec![];
        holes(&serde_json::from_str(raw).unwrap(), "", &mut unresolved);
        assert_eq!(
            unresolved.len(),
            if raw == FRAGMENT_CATALOG { 6 } else { 25 }
        );
    }
    assert!(
        serde_json::from_str::<crate::game::data::skill_fragment_data::SkillFragmentMetadata>(
            FRAGMENT_CATALOG
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<Vec<crate::game::data::equipment_data::EquipmentMetadata>>(
            EQUIPMENT_CATALOG
        )
        .is_err()
    );
}

/// Isolate one independent equipment provider for existing native oracles.
/// Only the harness equipment ID changes; the authored program is untouched.
pub(crate) fn equipment_fixture(kind: &str) -> String {
    let mut d = resolved("equipment");
    let (index, group, binding, id) = match kind {
        "weapon" => (0, "equipment_attacks", 0, "r11_weapon"),
        "armor" => (1, "equipment_defenses", 0, "last_will_fixture_armor"),
        "accessory" => (2, "equipment_attacks", 1, "r11_accessory"),
        _ => panic!(),
    };
    let value = d["values"][index].clone();
    let mut provider = d[group][binding].clone();
    provider["equipment"] = json!(id);
    d["values"] = json!([value]);
    d["equipment_attacks"] = json!([]);
    d["equipment_defenses"] = json!([]);
    d[group] = json!([provider]);
    d.to_string()
}
