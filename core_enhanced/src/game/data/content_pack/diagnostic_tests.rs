use super::tests::ExternalFixture;
use super::*;
use crate::game::data::live_ron::{parse_live_ron, LiveRonPhase, RonPosition};
use serde::Deserialize;

fn rejected(fixture: &ExternalFixture, selection: &ContentSelection) -> LiveRonError {
    match load(fixture.root(), selection, &InputLimits::default()) {
        Ok(_) => panic!("invalid external content was published"),
        Err(error) => error,
    }
}

#[test]
fn diagnostics_preserve_typed_parser_position_and_leave_message_precision_absent() {
    let input = "\n\n[]";
    let ron::de::Error::Parser(_, position) = ron::de::from_str::<u32>(input).unwrap_err() else {
        panic!("typed parser error expected");
    };
    let error = parse_live_ron::<u32>("actual/file.ron", "test_input", input).unwrap_err();
    assert_eq!(error.phase, LiveRonPhase::Parse);
    assert_eq!(error.code, "ron_parse");
    assert_eq!(
        error.position,
        Some(RonPosition {
            line: position.line,
            column: position.col
        })
    );
    assert!(error.parser_reason.is_some());
    assert_eq!(error.pack_id, None);
    assert_eq!(error.expected, None);

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Strict {
        value: u32,
    }
    assert_eq!(
        parse_live_ron::<Strict>("f", "test_input", "(value:1)")
            .unwrap()
            .value,
        1
    );
    let input = "(value:1,not_a_field:2)";
    assert!(matches!(
        ron::de::from_str::<Strict>(input),
        Err(ron::de::Error::Message(_))
    ));
    let error = parse_live_ron::<Strict>("actual/file.ron", "test_input", input).unwrap_err();
    assert_eq!(error.position, None);
    assert_eq!(error.parser_reason, None);
    assert!(error.cause.contains("not_a_field"));
    let json = serde_json::to_value(&error).unwrap();
    assert_eq!(json["phase"], "parse");
    assert!(json["position"].is_null());
}

#[test]
fn diagnostics_use_decoded_manifest_identity_and_actual_comparison_operands() {
    let fixture = ExternalFixture::official();
    let mut manifest: PackManifest = ron::de::from_str(&fixture.read("official/pack.ron")).unwrap();
    fixture.write("official/pack.ron", "()");
    let error = rejected(&fixture, &fixture.selection());
    assert_eq!(error.phase, LiveRonPhase::Parse);
    assert_eq!(error.pack_id, None); // directory name is not the manifest identity

    manifest.pack_id = "different_manifest_identity".into();
    manifest.contract.execution += 1;
    fixture.manifest("official", manifest.clone());
    let error = rejected(&fixture, &fixture.selection());
    assert_eq!(error.phase, LiveRonPhase::Structure);
    assert_eq!(error.code, "content_contract_unsupported");
    assert_eq!(
        error.pack_id.as_deref(),
        Some("different_manifest_identity")
    );
    assert_eq!(
        error.expected,
        Some(serde_json::json!(ContentContract::current()))
    );
    assert_eq!(error.actual, Some(serde_json::json!(manifest.contract)));

    manifest.contract = ContentContract::current();
    manifest
        .required_capabilities
        .push("unimplemented_expression".into());
    fixture.manifest("official", manifest);
    let error = rejected(&fixture, &fixture.selection());
    assert_eq!(error.code, "content_capability_unsupported");
    assert_eq!(
        error.actual,
        Some(serde_json::json!("unimplemented_expression"))
    );
    assert_eq!(
        error.expected,
        Some(serde_json::json!(PackManifest::SUPPORTED_CAPABILITIES))
    );
}

#[test]
fn diagnostics_retain_catalog_pack_at_parse_and_acquisition_boundaries() {
    let fixture = ExternalFixture::official();
    let mut manifest: PackManifest = ron::de::from_str(&fixture.read("official/pack.ron")).unwrap();
    manifest.pack_id = "actual_owner".into();
    fixture.manifest("official", manifest.clone());
    fixture.write("official/data/skills/base.ron", "(skills: [false])");
    let error = rejected(&fixture, &fixture.selection());
    assert_eq!(error.phase, LiveRonPhase::Parse);
    assert_eq!(error.pack_id.as_deref(), Some("actual_owner"));
    assert_eq!(error.catalog_kind, Some(CatalogKind::Skills));
    assert_eq!(error.definition_kind, None);
    assert_eq!(error.source_file, "official/data/skills/base.ron");

    manifest
        .files
        .iter_mut()
        .find(|f| f.kind == CatalogKind::Skills)
        .unwrap()
        .path = "missing.ron".into();
    fixture.manifest("official", manifest);
    let error = rejected(&fixture, &fixture.selection());
    assert_eq!(error.phase, LiveRonPhase::Acquisition);
    assert_eq!(error.code, "content_acquisition");
    assert_eq!(error.pack_id.as_deref(), Some("actual_owner"));
    assert_eq!(error.catalog_kind, Some(CatalogKind::Skills));
    assert_eq!(error.source_file, "official/missing.ron");
}

#[test]
fn diagnostics_separate_resolution_and_compilation_from_final_validation() {
    let fixture = ExternalFixture::official();
    let key = fixture.replace_first_skill("replacement", false);
    let mut selection = fixture.selection();
    selection.packs.push("replacement".into());
    let mut manifest: PackManifest =
        ron::de::from_str(&fixture.read("replacement/pack.ron")).unwrap();
    manifest.operations[0].after = vec![];
    fixture.manifest("replacement", manifest);
    let error = rejected(&fixture, &selection);
    assert_eq!(error.phase, LiveRonPhase::Resolution);
    assert_eq!(error.pack_id.as_deref(), Some("replacement"));
    assert_eq!(error.definition_kind, Some(key.kind));
    assert_eq!(error.catalog_kind, Some(CatalogKind::Skills));

    let path = "official/data/summoned_units/base.ron";
    let mut catalog: crate::game::data::summoned_unit_data::SummonedUnitCatalogData =
        ron::de::from_str(&fixture.read(path)).unwrap();
    catalog.definitions[0].max_alive = 0;
    fixture.write(path, ron::ser::to_string(&catalog).unwrap());
    let error = rejected(&fixture, &fixture.selection());
    assert_eq!(error.phase, LiveRonPhase::Compile, "{error}");
    assert_eq!(error.code, "content_compile");
    assert_eq!(error.source_file, path);
    assert_eq!(error.pack_id.as_deref(), Some("official"));
    assert_eq!(error.definition_kind, Some(DefinitionKind::SummonedUnit));
}

#[test]
fn aggregate_writer_failures_keep_actual_contributors_through_common_validation() {
    let fixture = ExternalFixture::official();
    let target = fixture.replace_first_skill("left", false);
    fixture.replace_first_skill("right", false);
    let selection = ContentSelection {
        packs: vec!["right".into(), "official".into(), "left".into()],
        winners: vec![],
    };
    let check = |expected: &[(&str, &str, &str)]| {
        let plain = rejected(&fixture, &selection);
        let common = crate::authoring::validate(
            fixture.root(),
            &selection,
            &crate::authoring::AuthoringLimits::default(),
            &crate::authoring::CancellationToken::default(),
        )
        .err()
        .unwrap();
        let crate::authoring::AuthoringError::Content { diagnostic } = common else {
            panic!("expected content rejection");
        };
        assert_eq!(diagnostic, plain);
        assert_eq!(plain.phase, LiveRonPhase::Resolution);
        assert_eq!(plain.pack_id, None);
        assert_eq!(plain.object_id.as_deref(), Some(target.id.as_str()));
        assert_eq!(plain.definition_kind, Some(target.kind));
        let actual: Vec<_> = plain
            .related_sources
            .iter()
            .map(|source| {
                assert_eq!(source.catalog_kind, Some(CatalogKind::Skills));
                assert_eq!(source.definition_kind, Some(target.kind));
                assert_eq!(source.object_id.as_deref(), Some(target.id.as_str()));
                (
                    source.pack_id.as_deref().unwrap(),
                    source.source_file.as_str(),
                    source.field_path.as_deref().unwrap(),
                )
            })
            .collect();
        assert_eq!(actual, expected);
    };
    check(&[
        ("left", "left/pack.ron", "operations[0]"),
        ("right", "right/pack.ron", "operations[0]"),
    ]);

    for directory in ["left", "right"] {
        let mut manifest: PackManifest =
            ron::de::from_str(&fixture.read(&format!("{directory}/pack.ron"))).unwrap();
        manifest.operations.clear();
        fixture.manifest(directory, manifest);
    }
    check(&[
        ("left", "left/skills.ron", "skills[0]"),
        ("official", "official/data/skills/base.ron", "skills[0]"),
        ("right", "right/skills.ron", "skills[0]"),
    ]);
}

#[test]
fn source_metadata_retains_actual_remove_writer_without_a_final_definition() {
    use crate::game::{ability::SkillId, data::skill_data::RawSkillDatabase};
    let fixture = ExternalFixture::official();
    let mut raw =
        RawSkillDatabase::from_ron("source", &fixture.read("official/data/skills/base.ron"))
            .unwrap();
    let mut skill = raw.skills.remove(0);
    skill.id = SkillId::from("unreferenced_authoring_example");
    let target = DefinitionKey {
        kind: DefinitionKind::Skill,
        id: skill.id.to_string(),
    };
    fixture.write(
        "extra/skills.ron",
        ron::ser::to_string(&RawSkillDatabase {
            skills: vec![skill],
            range_presets: vec![],
        })
        .unwrap(),
    );
    fixture.manifest(
        "extra",
        PackManifest {
            pack_id: "extra".into(),
            pack_version: "1".into(),
            namespace: PackNamespace::Global,
            contract: ContentContract::current(),
            required_capabilities: vec![],
            dependencies: vec!["official".into()],
            files: vec![CatalogFile {
                kind: CatalogKind::Skills,
                path: "skills.ron".into(),
            }],
            operations: vec![],
        },
    );
    fixture.manifest(
        "removal",
        PackManifest {
            pack_id: "removal".into(),
            pack_version: "1".into(),
            namespace: PackNamespace::Global,
            contract: ContentContract::current(),
            required_capabilities: vec![],
            dependencies: vec!["extra".into()],
            files: vec![],
            operations: vec![DefinitionOperation {
                target: target.clone(),
                operation: OperationKind::Remove,
                after: vec!["extra".into()],
            }],
        },
    );
    let selection = ContentSelection {
        packs: vec!["official".into(), "extra".into(), "removal".into()],
        winners: vec![],
    };
    let data = load(fixture.root(), &selection, &InputLimits::default()).unwrap();
    assert!(data.skill_data.get_by_id(&target.id).is_none());
    let metadata = data.source_metadata().unwrap();
    assert!(metadata.source_for(&target).is_none());
    let writers: Vec<_> = metadata
        .writers()
        .iter()
        .filter(|w| w.target == target)
        .collect();
    assert_eq!(writers.len(), 2);
    let selected = writers.iter().find(|w| w.selected).unwrap();
    assert_eq!(selected.pack_id, "removal");
    assert_eq!(selected.operation, DefinitionWriteKind::Remove);
    assert_eq!(
        selected.after,
        std::collections::BTreeSet::from(["extra".into()])
    );
    let original_identity = data.identity().clone();
    fixture.write("removal/pack.ron", "malformed after publication");
    assert_eq!(data.identity(), &original_identity);
    assert!(metadata
        .writers()
        .iter()
        .any(|w| w.selected && w.operation == DefinitionWriteKind::Remove));
}
