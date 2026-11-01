fn catalog_reference_fields(
    value: &serde_json::Value,
    path: &str,
    out: &mut Vec<(String, String)>,
) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, v) in map {
                let p = format!("{path}/{key}");
                if key == "relic_definition" {
                    out.push((p, v.as_str().unwrap().to_owned()));
                } else if key == "requires" {
                    for (i, id) in v.as_array().unwrap().iter().enumerate() {
                        out.push((format!("{p}/{i}"), id.as_str().unwrap().to_owned()));
                    }
                } else {
                    catalog_reference_fields(v, &p, out);
                }
            }
        }
        serde_json::Value::Array(values) => {
            for (i, v) in values.iter().enumerate() {
                catalog_reference_fields(v, &format!("{path}/{i}"), out);
            }
        }
        _ => (),
    }
}

// Full publication must resolve every explicit relic reference, including
// programs whose skill is not yet equipped by a fragment in this generation.
#[test]
fn authored_shared_program_publication_rejects_missing_relics() {
    use crate::authoring::composition::{
        self, canonical_hatred_tests as hatred, canonical_input_plan_tests as input,
        canonical_knight_tests as knight,
    };
    use crate::game::data::relic_data::RelicDatabase;
    use serde_json::json;

    let manual = composition::parse_yaml(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/manual_state.skill.yaml"
    )))
    .unwrap();
    let mut misses = vec![];
    for (label, document) in [
        ("relation", knight::resolved(false)),
        ("counter", hatred::resolved(false)),
        ("input", input::resolved()),
        ("state", manual),
        (
            "pair",
            crate::authoring::composition::canonical_pair_tests::resolved(),
        ),
        (
            "volley",
            composition::owned_action_tests::module(
                &crate::authoring::node_graph::compile(
                    &crate::authoring::node_graph::tests::prepared_volley_fixture().to_string(),
                    "integration-volley".into(),
                )
                .unwrap()
                .load_native()
                .unwrap(),
            ),
        ),
    ] {
        let fixture = SourceFixture::new();
        let mut refs = vec![];
        catalog_reference_fields(&document, "", &mut refs);
        assert!(!refs.is_empty(), "{label}");
        let path = "data/relics/base.ron";
        let mut relics: RelicDatabase =
            ron::de::from_str(&fs::read_to_string(fixture.source().join(path)).unwrap()).unwrap();
        for (i, (_, id)) in refs.iter().enumerate() {
            if relics.get_by_id(id).is_none() {
                relics.relics.push(
                    serde_json::from_value(json!({
                        "id":id,"uuid":uuid::Uuid::from_u128(0x12600+i as u128),
                        "name":"Explicit shared-program companion","description":"Publication test",
                        "rarity":"fixture","allow_duplicates":true,"provider_authored":true,
                        "effects":[],"rules":[],"dismantle_equipment_dust":1
                    }))
                    .unwrap(),
                );
            }
        }
        write(&fixture, path, &ron::ser::to_string(&relics).unwrap());
        write(&fixture, FILE, &document.to_string());
        let accepted = package(&fixture, "valid", &[FILE]).unwrap();
        let mut invalid = document.clone();
        // Counter menu children duplicate the program. Keep internal declarations
        // consistent so this reaches the full-catalog validation boundary.
        for (path, id) in &refs {
            if id == &refs[0].1 {
                *invalid.pointer_mut(path).unwrap() = json!("absent-explicit-relic");
            }
        }
        composition::compile_json(&invalid.to_string()).unwrap();
        write(&fixture, FILE, &invalid.to_string());
        match package(&fixture, "invalid", &[FILE]) {
            Ok(_) => misses.push(label),
            Err(error) => {
                assert!(
                    error
                        .field_path
                        .as_deref()
                        .is_some_and(|p| p.ends_with("relic_definition")),
                    "{label}: {error:?}"
                );
                assert!(!fixture.0.join("invalid").exists());
            }
        }
        assert_eq!(
            reload(&fixture, "valid").identity(),
            &accepted.content_identity
        );
    }
    assert!(
        misses.is_empty(),
        "Unknown explicit relic accepted by publication: {misses:?}"
    );
}

#[test]
fn every_native_effect_relic_field_is_checked_at_the_catalog_boundary() {
    use crate::authoring::node_graph::{self, tests::*};
    use crate::game::data::{
        equipment_data::EquipmentDatabase, relic_data::RelicDatabase,
    };
    use serde_json::json;
    let equipment = EquipmentDatabase::new(vec![]);
    let mut checked = 0;
    for (label, graph) in [
        ("relation", relation_action_fixture(false)),
        ("counter", counter_channel_fixture(false)),
        ("input", input_plan_fixture()),
        ("volley", prepared_volley_fixture()),
        ("pair", paired_areas_fixture()),
        ("fusion", fusion_placement_fixture(false)),
        ("status", status_strike_fixture()),
        ("guard", parry_fixture()),
        ("overlay", attack_overlay_fixture()),
    ] {
        let native = node_graph::compile(&graph.to_string(), "catalog-integration".into())
            .unwrap()
            .load_native()
            .unwrap();
        let document = serde_json::to_value(&native).unwrap();
        let mut refs = vec![];
        catalog_reference_fields(&document, "", &mut refs);
        assert!(!refs.is_empty(), "{label}");
        let mut relics = RelicDatabase::default();
        for (_, id) in &refs {
            if relics.get_by_id(id).is_none() {
                relics.relics.push(
                    serde_json::from_value(json!({
                        "id":id,"uuid":uuid::Uuid::from_u128(0x12680+relics.relics.len() as u128),
                        "name":"Native explicit companion","description":"Catalog boundary test",
                        "rarity":"fixture","allow_duplicates":true,"provider_authored":true,
                        "effects":[],"rules":[],"dismantle_equipment_dust":1
                    }))
                    .unwrap(),
                );
            }
        }
        native
            .validate_effect_catalog_references(&equipment, &relics)
            .unwrap();
        // Target each serialized field separately. This intentionally exercises
        // only catalog resolution; complete-publication cases below/above also
        // verify structural consistency and preservation of the prior pack.
        for (pointer, _) in &refs {
            let mut invalid = document.clone();
            *invalid.pointer_mut(pointer).unwrap() = json!("absent-explicit-relic");
            let mut invalid_native = native.clone();
            invalid_native.skills = serde_json::from_value(invalid["skills"].clone()).unwrap();
            let invalid = invalid_native;
            let error = invalid
                .validate_effect_catalog_references(&equipment, &relics)
                .unwrap_err();
            let expected =
                pointer
                    .trim_start_matches('/')
                    .split('/')
                    .fold(String::new(), |mut path, part| {
                        if part.parse::<usize>().is_ok() {
                            path.push_str(&format!("[{part}]"));
                        } else {
                            if !path.is_empty() {
                                path.push('.');
                            }
                            path.push_str(part);
                        }
                        path
                    });
            assert_eq!(error.field_path, Some(expected), "{label}/{pointer}: {error:?}");
            assert_eq!(error.cause, "Skill effect references unknown relic");
            checked += 1;
        }
    }
    eprintln!(
        "catalog boundary: {checked} individual explicit fields across nine native consumers"
    );
}

#[test]
fn native_mobile_and_guard_publication_resolves_companions_before_commit() {
    use crate::authoring::node_graph::{self, tests::*};
    use crate::game::data::{relic_data::RelicDatabase, skill_data::SkillDatabase};
    use serde_json::json;
    const NATIVE: &str = "data/skills/catalog-integration.ron";
    for (label, graph) in [
        ("fusion", fusion_placement_fixture(false)),
        ("status", status_strike_fixture()),
        ("guard", parry_fixture()),
    ] {
        let native = node_graph::compile(&graph.to_string(), "catalog-integration".into())
            .unwrap()
            .load_native()
            .unwrap();
        let mut document = serde_json::to_value(&native).unwrap();
        let mut refs = vec![];
        catalog_reference_fields(&document, "", &mut refs);
        let fixture = SourceFixture::new();
        let path = "data/relics/base.ron";
        let mut relics: RelicDatabase =
            ron::de::from_str(&fs::read_to_string(fixture.source().join(path)).unwrap()).unwrap();
        for (_, id) in &refs {
            if relics.get_by_id(id).is_none() {
                relics.relics.push(
                    serde_json::from_value(json!({
                        "id":id,"uuid":uuid::Uuid::from_u128(0x126c0+relics.relics.len() as u128),
                        "name":"Native publication companion","description":"Publication test",
                        "rarity":"fixture","allow_duplicates":true,"provider_authored":true,
                        "effects":[],"rules":[],"dismantle_equipment_dust":1
                    }))
                    .unwrap(),
                );
            }
        }
        write(&fixture, path, &ron::ser::to_string(&relics).unwrap());
        write(&fixture, NATIVE, &ron::ser::to_string(&native).unwrap());
        let valid = package(&fixture, "valid", &[NATIVE]).unwrap();
        for (pointer, id) in &refs {
            if id == &refs[0].1 {
                *document.pointer_mut(pointer).unwrap() = json!("absent-explicit-relic");
            }
        }
        let invalid =
            SkillDatabase::new(serde_json::from_value(document["skills"].clone()).unwrap());
        let ron = ron::ser::to_string(&invalid).unwrap();
        SkillDatabase::from_ron(NATIVE, &ron).unwrap();
        write(&fixture, NATIVE, &ron);
        let error = package(&fixture, "invalid", &[NATIVE]).unwrap_err();
        assert!(
            error
                .field_path
                .as_deref()
                .is_some_and(|p| p.ends_with("relic_definition")),
            "{label}: {error:?}"
        );
        assert!(!fixture.0.join("invalid").exists());
        assert_eq!(
            reload(&fixture, "valid").identity(),
            &valid.content_identity
        );
    }
}
