use super::*;
use crate::game::data::live_ron::GameplayRonSourceSet;

fn official() -> Vec<LocatedDefinition> {
    let sources = GameplayRonSourceSet::embedded();
    CatalogKind::ALL
        .iter()
        .zip(sources.source_inventory())
        .flat_map(|(kind, (path, input))| {
            assert_eq!(kind.official_path(), path);
            kind.parse(
                "official",
                &format!("packs/official/{path}"),
                input,
                &super::super::InputLimits::default(),
            )
            .unwrap()
        })
        .collect()
}

#[test]
fn all_26_typed_catalogs_reassemble_and_validate_with_original_sources() {
    assert_eq!(CatalogKind::ALL.len(), 26);
    assert!(CatalogKind::ALL.contains(&CatalogKind::Relics));
    assert_eq!(CatalogKind::Relics.official_path(), "data/relics/base.ron");
    let relics = CatalogKind::Relics
        .parse(
            "official",
            "packs/official/data/relics/base.ron",
            GameplayRonSourceSet::embedded().relics,
            &super::super::InputLimits::default(),
        )
        .unwrap();
    assert!(
        relics.is_empty(),
        "official relic balance remains unauthored"
    );
    let definitions = official();
    assert!(definitions.len() > 100);
    let payloads: Vec<_> = definitions.iter().map(|d| d.definition.clone()).collect();
    let first = super::super::canonical::bytes(&payloads).unwrap();
    let second = super::super::canonical::bytes(&payloads.clone()).unwrap();
    assert_eq!(first, second);
    let parsed = assemble(definitions).unwrap();
    let effect = &parsed.skills.skills[0].steps[0].effects[0];
    assert_eq!(
        effect.source.as_ref().unwrap().source_file,
        "packs/official/data/skills/base.ron"
    );
    parsed.into_validated().unwrap();
}

#[test]
fn skill_and_corroded_presets_are_lowered_only_after_cross_file_assembly() {
    let embedded = GameplayRonSourceSet::embedded();
    let skills = RawSkillDatabase::from_ron("official", embedded.skills).unwrap();
    let skill_presets = RawSkillDatabase {
        range_presets: skills.range_presets,
        skills: vec![],
    };
    let skill_definitions = RawSkillDatabase {
        range_presets: vec![],
        skills: skills.skills,
    };
    let profiles: RawCorrodedEmployeeProfileDatabase = parse_live_ron(
        "official",
        "corroded_employees",
        embedded.corroded_employees,
    )
    .unwrap();
    let profile_presets = RawCorrodedEmployeeProfileDatabase {
        range_presets: profiles.range_presets,
        profiles: vec![],
    };
    let profile_definitions = RawCorrodedEmployeeProfileDatabase {
        range_presets: vec![],
        profiles: profiles.profiles,
    };
    let skill_input = ron::ser::to_string(&skill_definitions).unwrap();
    let profile_input = ron::ser::to_string(&profile_definitions).unwrap();
    assert!(crate::game::data::skill_data::SkillDatabase::from_ron("early", &skill_input).is_err());
    assert!(ron::de::from_str::<
        crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase,
    >(&profile_input)
    .is_err());
    let mut definitions = official();
    definitions.retain(|d| {
        !matches!(
            d.definition.key().kind.catalog(),
            CatalogKind::Skills | CatalogKind::CorrodedEmployees
        )
    });
    for (kind, pack, path, input) in [
        (
            CatalogKind::Skills,
            "definitions",
            "defs/skills.ron",
            skill_input,
        ),
        (
            CatalogKind::CorrodedEmployees,
            "definitions",
            "defs/profiles.ron",
            profile_input,
        ),
        (
            CatalogKind::Skills,
            "presets",
            "presets/skills.ron",
            ron::ser::to_string(&skill_presets).unwrap(),
        ),
        (
            CatalogKind::CorrodedEmployees,
            "presets",
            "presets/profiles.ron",
            ron::ser::to_string(&profile_presets).unwrap(),
        ),
    ] {
        definitions.extend(
            kind.parse(pack, path, &input, &super::super::InputLimits::default())
                .unwrap(),
        );
    }
    let parsed = assemble(definitions).unwrap();
    assert!(parsed.corroded_employees.profiles[0]
        .basic_attack
        .defense_tile_range
        .is_some());
    assert_eq!(
        parsed.skills.skills[0].steps[0].effects[0]
            .source
            .as_ref()
            .unwrap()
            .source_file,
        "defs/skills.ron"
    );
    parsed.into_validated().unwrap();
}

#[test]
fn duplicate_uuid_reaches_fallible_final_validation_without_constructor_panic() {
    let mut definitions = official();
    let mut duplicate = definitions
        .iter()
        .find(|d| matches!(d.definition, RawDefinition::Equipment(_)))
        .unwrap()
        .clone();
    if let RawDefinition::Equipment(item) = &mut duplicate.definition {
        item.id = "custom_duplicate_uuid".into();
    }
    duplicate.source = DefinitionSource {
        pack_id: "custom".into(),
        source_file: "custom/equipment.ron".into(),
        field_path: "items[0]".into(),
    };
    definitions.push(duplicate);
    let sources = SourceTable::from_definitions(&definitions);
    let parsed = assemble(definitions).unwrap();
    let error = sources.relocate(
        parsed
            .into_validated()
            .err()
            .expect("UUID collision must fail"),
    );
    assert!(error.cause.contains("uuid"), "{error}");
    assert_eq!(error.source_file, "custom/equipment.ron");
    assert_eq!(error.field_path.as_deref(), Some("items[0].uuid"));
}

#[test]
fn winning_effect_keeps_original_definition_index_after_filtering_and_append() {
    let mut definitions = official();
    let input = include_str!("../../../../../tests/fixtures/content_authoring/literal_effects.ron");
    let custom = CatalogKind::Skills
        .parse(
            "custom",
            "custom/skills.ron",
            input,
            &super::super::InputLimits::default(),
        )
        .unwrap();
    definitions.extend(
        custom
            .into_iter()
            .filter(|d| d.definition.key().id == "authoring_beta"),
    );
    let parsed = assemble(definitions).unwrap();
    let beta = parsed.skills.get_by_id("authoring_beta").unwrap();
    let source = beta.steps[0].effects[0].source.as_ref().unwrap();
    assert_eq!(source.source_file, "custom/skills.ron");
    assert_eq!(source.field_path, "skills[1].steps[0].effects[0]");
    assert_eq!(source.site.action.definition_id.as_ref(), "authoring_beta");
}

#[test]
fn raw_missing_preset_is_accepted_until_final_winner_lowering_and_relocated() {
    let mut definitions = official();
    definitions.retain(|d| d.definition.key().kind.catalog() != CatalogKind::Skills);
    let input = GameplayRonSourceSet::embedded().skills.replacen(
        "Some(\"front_3x2\")",
        "Some(\"missing_preset\")",
        1,
    );
    let raw = CatalogKind::Skills
        .parse(
            "custom",
            "custom/skills.ron",
            &input,
            &super::super::InputLimits::default(),
        )
        .unwrap();
    definitions.extend(raw);
    let error = assemble(definitions)
        .err()
        .expect("final missing preset must fail");
    assert_eq!(error.source_file, "custom/skills.ron");
    assert_eq!(
        error.field_path.as_deref(),
        Some("skills[0].cast_targeting")
    );
    assert!(error.cause.contains("missing_preset"));
}
