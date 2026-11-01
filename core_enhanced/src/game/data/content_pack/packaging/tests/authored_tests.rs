use super::*;
use crate::game::data::live_ron::LiveRonPhase;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/skill_composition/woodsman_monk.skill.yaml"
));
const FILE: &str = "data/skills/authored.skill.yaml";

#[test]
fn authored_tile_burst_publication_resolves_explicit_weapon_and_relic_companions() {
    use crate::{authoring::{composition, node_graph}, game::data::{equipment_data::EquipmentDatabase, relic_data::RelicDatabase}};
    let fixture = SourceFixture::new();
    let mut original = node_graph::compile(
        &node_graph::tests::tile_burst_fixture("-orb").to_string(), "r11_aura".into(),
    ).unwrap().load_content().unwrap();
    let document = composition::owned_action_tests::module(&original.skills);
    original.mark_weapons[0].definition.evolution.fragment = "fragment_funeral_butterfly_eulogy".into();
    original.mark_weapons[0].definition.evolution.form = "awakened".into();
    let equipment_path = "data/equipments/base.ron";
    let mut equipment: EquipmentDatabase = ron::de::from_str(&fs::read_to_string(fixture.source().join(equipment_path)).unwrap()).unwrap();
    let mut weapon = equipment.get_by_id("solemn_lament").unwrap().clone();
    weapon.id = "r11_weapon".into();
    weapon.uuid = uuid::Uuid::from_u128(0x11901);
    weapon.weapon_profile.as_mut().unwrap().mark_weapon = Some(original.mark_weapons[0].definition.clone());
    equipment.items.push(weapon);
    write(&fixture, equipment_path, &ron::ser::to_string(&equipment).unwrap());
    let relic_path = "data/relics/base.ron";
    let mut relics: RelicDatabase = ron::de::from_str(&fs::read_to_string(fixture.source().join(relic_path)).unwrap()).unwrap();
    for (i, id) in ["widened-horizon", "overlapping-explosions", "explosion-vulnerability", "deep-imprint", "funeral-butterfly-orb"].into_iter().enumerate() {
        if relics.get_by_id(id).is_none() {
            relics.relics.push(serde_json::from_value(serde_json::json!({
                "id":id,"uuid":uuid::Uuid::from_u128(0x11910+i as u128),"name":"Publication fixture",
                "description":"Explicit companion for authored effect","rarity":"fixture","allow_duplicates":true,
                "provider_authored":true,"effects":[],"rules":[],"dismantle_equipment_dust":1
            })).unwrap());
        }
    }
    write(&fixture, relic_path, &ron::ser::to_string(&relics).unwrap());
    write(&fixture, FILE, &document.to_string());
    let receipt = package(&fixture, "accepted-burst", &[FILE]).unwrap();
    let accepted = reload(&fixture, "accepted-burst");
    assert_eq!(accepted.identity(), &receipt.content_identity);
    assert_eq!(accepted.equipment_data.get_by_id("r11_weapon").unwrap().weapon_profile.as_ref().unwrap().mark_weapon.as_ref(), Some(&original.mark_weapons[0].definition));
    for (fault, field) in [
        ("missing-weapon", "source.WeaponStacks.weapon"),
        ("non-mark-weapon", "source.WeaponStacks.weapon"),
        ("store-key", "source.WeaponStacks.key"),
        ("modifier-relic", "modifiers[0].relic_definition"),
        ("orb-relic", "source.WeaponStacks.conversion.relic_definition"),
    ] {
        let mut invalid = document.clone();
        let program = &mut invalid["skills"][0]["body"]["Action"]["effects"][0]["Append"]["effects"]["Literal"]["Effects"][0]["op"]["TileBurst"]["program"];
        match fault {
            "missing-weapon" => program["source"]["WeaponStacks"]["weapon"] = serde_json::json!("absent-weapon"),
            "non-mark-weapon" => program["source"]["WeaponStacks"]["weapon"] = serde_json::json!("justitia"),
            "store-key" => program["source"]["WeaponStacks"]["key"] = serde_json::json!("another-store"),
            "modifier-relic" => program["modifiers"][0]["relic_definition"] = serde_json::json!("absent-relic"),
            _ => program["source"]["WeaponStacks"]["conversion"]["relic_definition"] = serde_json::json!("absent-relic"),
        }
        // Standalone compilation cannot resolve other catalogs. Publication must.
        composition::compile_json(&invalid.to_string()).unwrap();
        write(&fixture, FILE, &invalid.to_string());
        let error = package(&fixture, fault, &[FILE]).unwrap_err();
        assert_eq!(error.object_id.as_deref(), Some("r11_aura"));
        assert!(error.field_path.as_deref().unwrap().ends_with(field), "{fault}: {error:?}");
        assert!(!fixture.0.join(fault).exists());
        assert_eq!(reload(&fixture, "accepted-burst").identity(), accepted.identity());
    }
}

fn write(fixture: &SourceFixture, path: &str, input: &str) {
    let path = fixture.source().join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, input).unwrap();
}
fn package(
    fixture: &SourceFixture,
    name: &str,
    files: &[&str],
) -> Result<PackageReceipt, LiveRonError> {
    package_official_with_skill_files(
        &fixture.source(),
        &fixture.0.join(name),
        &files.iter().map(PathBuf::from).collect::<Vec<_>>(),
        &InputLimits::default(),
    )
}
fn reload(fixture: &SourceFixture, name: &str) -> ValidatedGameData {
    load(
        &fixture.0,
        &ContentSelection {
            packs: vec![name.into()],
            winners: vec![],
        },
        &InputLimits::default(),
    )
    .unwrap()
}

#[test]
fn authored_pack_yaml_preserves_bytes_origins_and_matches_compiled_artifact() {
    let fixture = SourceFixture::new();
    let source =
        format!("# Captured authored source with unmatched [ {{ ( in a YAML comment\r\n{SOURCE}\n");
    write(&fixture, FILE, &source);
    let source = format!("# {}\n{source}", "x".repeat(70_000));
    write(&fixture, FILE, &source);
    let before = fixture.bytes();
    let receipt = package(&fixture, "yaml", &[FILE]).unwrap();
    assert_eq!(receipt.source_receipt.files.len(), 28);
    assert_eq!(receipt.pack_receipt.files.len(), 28);
    assert_eq!(
        fs::read(fixture.0.join("yaml").join(FILE)).unwrap(),
        source.as_bytes()
    );
    let digest = format!("{:x}", Sha256::digest(source.as_bytes()));
    assert!(receipt
        .source_receipt
        .files
        .iter()
        .any(|r| r.path == FILE && r.sha256 == digest));
    let accepted = reload(&fixture, "yaml");
    for id in ["woodsman-authored", "monk-authored"] {
        let skill = accepted.skill_data.get_by_id(id).unwrap();
        let origin = &accepted
            .source_metadata()
            .unwrap()
            .definitions()
            .iter()
            .find(|(key, _)| key.id == id)
            .unwrap()
            .1;
        assert_eq!(origin.source_file, format!("yaml/{FILE}"));
        for effect in skill.steps.iter().flat_map(|s| &s.effects) {
            assert_eq!(
                effect.source.as_ref().unwrap().source_file,
                origin.source_file
            );
        }
    }
    let artifact = crate::authoring::composition::compile_yaml(&source)
        .unwrap()
        .to_ron()
        .unwrap();
    let artifact_file = "data/skills/authored.skill-composition.ron";
    write(&fixture, artifact_file, &artifact);
    let compiled = package(&fixture, "compiled", &[artifact_file]).unwrap();
    assert_eq!(receipt.content_identity, compiled.content_identity);
    // The legacy entry point continues to select only its fixed catalogs.
    let legacy = fixture.package(&fixture.0.join("legacy")).unwrap();
    assert_eq!(legacy.source_receipt.files.len(), 27);
    assert!(reload(&fixture, "legacy")
        .skill_data
        .get_by_id("monk-authored")
        .is_none());
    assert_eq!(fixture.bytes(), before);
    assert_eq!(
        fs::read(fixture.source().join(FILE)).unwrap(),
        source.as_bytes()
    );
}

#[test]
fn authored_pack_source_errors_preserve_diagnostics_and_previous_generation() {
    let fixture = SourceFixture::new();
    write(&fixture, FILE, SOURCE);
    package(&fixture, "accepted", &[FILE]).unwrap();
    let accepted = reload(&fixture, "accepted");
    let prior = accepted.identity().clone();
    let mut unresolved = crate::authoring::composition::parse_yaml(SOURCE).unwrap();
    unresolved["skills"][0]["admission"]["capacity"] =
        serde_json::json!({"Unresolved":"capacity not decided"});
    let sources = [
        (serde_json::to_string(&unresolved).unwrap(), "Unresolved"),
        (SOURCE.replacen("version: 1", "version: 999", 1), "Version"),
        ("version: [".into(), "Parse"),
        (
            SOURCE.replacen("Shared: weapon-power", "Shared: missing-power", 1),
            "Reference",
        ),
    ];
    for (index, (source, code)) in sources.iter().enumerate() {
        write(&fixture, FILE, source);
        let name = format!("rejected-{index}");
        let error = package(&fixture, &name, &[FILE]).unwrap_err();
        assert_eq!(error.source_file, FILE);
        assert_eq!(
            error.code, "composition_source_invalid",
            "case {index}: {error:?}"
        );
        assert_eq!(error.phase, LiveRonPhase::Compile);
        let diagnostic = error.actual.as_ref().unwrap();
        assert_eq!(diagnostic["code"], *code);
        assert_eq!(
            diagnostic["source_digest"],
            format!("{:x}", Sha256::digest(source.as_bytes()))
        );
        if index == 0 {
            assert_eq!(
                error.object_id.as_deref(),
                Some("woodsman-monk-composition-fixture")
            );
            assert_eq!(
                error.field_path.as_deref(),
                Some("skills[0].admission.capacity")
            );
        }
        assert!(!fixture.0.join(name).exists());
        assert_eq!(
            fs::read_to_string(fixture.source().join(FILE)).unwrap(),
            *source
        );
        assert_eq!(accepted.identity(), &prior);
        assert!(accepted.skill_data.get_by_id("monk-authored").is_some());
    }
    write(&fixture, FILE, SOURCE);
    assert_eq!(
        package(&fixture, "retry", &[FILE])
            .unwrap()
            .content_identity,
        prior
    );
}

#[test]
fn authored_pack_inventory_and_conflicts_reject_before_output() {
    let fixture = SourceFixture::new();
    write(&fixture, FILE, SOURCE);
    write(&fixture, "data/skills/duplicate.skill.yaml", SOURCE);
    for (index, files) in [
        vec![FILE, FILE],
        vec!["data/skills/missing.skill.yaml"],
        vec!["../outside.skill.yaml"],
        vec![FILE, "data/skills/duplicate.skill.yaml"],
    ]
    .iter()
    .enumerate()
    {
        let name = format!("invalid-{index}");
        assert!(package(&fixture, &name, files).is_err());
        assert!(!fixture.0.join(name).exists());
    }
    symlink(
        fixture.source().join(FILE),
        fixture.source().join("data/skills/alias.skill.yaml"),
    )
    .unwrap();
    assert!(package(&fixture, "alias", &["data/skills/alias.skill.yaml"]).is_err());
    assert!(!fixture.0.join("alias").exists());
    let low = InputLimits {
        max_files: 27,
        ..InputLimits::default()
    };
    assert!(package_official_with_skill_files(
        &fixture.source(),
        &fixture.0.join("limited"),
        &[FILE.into()],
        &low
    )
    .is_err());
    assert!(!fixture.0.join("limited").exists());
    package(&fixture, "accepted", &[FILE]).unwrap();
    let before = fs::read(fixture.0.join("accepted/pack.ron")).unwrap();
    assert!(package(&fixture, "accepted", &[FILE]).is_err());
    assert_eq!(
        fs::read(fixture.0.join("accepted/pack.ron")).unwrap(),
        before
    );
    let mut manifest: PackManifest = ron::de::from_bytes(&before).unwrap();
    manifest
        .required_capabilities
        .retain(|c| c != "skill_composition_yaml_v1");
    fs::write(
        fixture.0.join("accepted/pack.ron"),
        ron::ser::to_string(&manifest).unwrap(),
    )
    .unwrap();
    let error = load(
        &fixture.0,
        &ContentSelection {
            packs: vec!["accepted".into()],
            winners: vec![],
        },
        &InputLimits::default(),
    )
    .err()
    .unwrap();
    assert_eq!(error.field_path.as_deref(), Some("required_capabilities"));
    assert!(error.cause.contains("skill_composition_yaml_v1"));

    let oversized = format!("# {}\n{SOURCE}", "x".repeat(200_000));
    write(&fixture, FILE, &oversized);
    let limits = InputLimits {
        max_file_bytes: 200_000,
        ..InputLimits::default()
    };
    let error = package_official_with_skill_files(
        &fixture.source(),
        &fixture.0.join("source-byte-limit"),
        &[FILE.into()],
        &limits,
    )
    .unwrap_err();
    assert_eq!(error.source_file, FILE);
    assert_eq!(error.phase, LiveRonPhase::Acquisition);
    assert!(error.cause.contains("limit"));
    assert!(!fixture.0.join("source-byte-limit").exists());
}

#[test]
fn authored_pack_publishes_all_companion_definitions_and_rejects_missing_equipment() {
    let fixture = SourceFixture::new();
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/skill_composition");
    let files: Vec<_> = [
        "equipment_attack",
        "equipment_defense",
        "equipment_support",
        "equipment_state",
    ]
    .iter()
    .map(|name| format!("data/skills/{name}.skill.yaml"))
    .collect();
    for file in &files {
        let source =
            fs::read_to_string(fixtures.join(Path::new(file).file_name().unwrap())).unwrap();
        write(
            &fixture,
            file,
            &source.replace("equipment: r11_weapon", "equipment: justitia"),
        );
    }
    let refs: Vec<_> = files.iter().map(String::as_str).collect();
    package(&fixture, "companions", &refs).unwrap();
    let accepted = reload(&fixture, "companions");
    assert!(accepted
        .resource_data
        .resources
        .iter()
        .any(|r| r.id.as_str() == "weapon_mass"));
    let origins = accepted.source_metadata().unwrap().definitions();
    for kind in [
        DefinitionKind::EquipmentAttack,
        DefinitionKind::EquipmentDefense,
        DefinitionKind::EquipmentSupport,
        DefinitionKind::EquipmentState,
    ] {
        assert!(
            origins.iter().any(|(key, source)| key.kind == kind
                && key.id == "justitia"
                && source.source_file.ends_with(".skill.yaml")),
            "{kind:?}"
        );
    }
    let source = fs::read_to_string(fixture.source().join(&files[0])).unwrap();
    write(
        &fixture,
        &files[0],
        &source.replace("equipment: justitia", "equipment: missing_equipment"),
    );
    let error = package(&fixture, "missing-equipment", &refs).unwrap_err();
    assert!(error.to_string().contains("missing_equipment"), "{error:?}");
    assert!(!fixture.0.join("missing-equipment").exists());
    assert!(accepted
        .resource_data
        .resources
        .iter()
        .any(|r| r.id.as_str() == "weapon_mass"));
}

#[test]
fn authored_pack_yaml_enforces_host_nesting_before_lowering_unused_source() {
    let mut root = serde_json::json!({"Const":{"value":1,"unit":"Magnitude"}});
    for _ in 0..8 {
        root = serde_json::json!({"Add":[root, {"Const":{"value":1,"unit":"Magnitude"}}]});
    }
    let input = serde_json::json!({
        "version":1, "id":"deep-unused-authored-value",
        "values":[{"name":"unused","value":{"Amount":{"moment":"Live","root":root}}}],
        "definitions":[], "skills":[],
        "equipment_supports":[{"equipment":"r11_weapon","program":{"Literal":{"SupportProgram":{"heal_percent":1,"shield_percent":1}}}}]
    })
    .to_string();
    let artifact = crate::authoring::composition::compile_yaml(&input).unwrap();
    assert!(artifact.load_native().unwrap().skills.is_empty());
    let limits = crate::game::value_expr::ExprParseLimits::default();

    let oversized = format!(
        "# {}\n{input}",
        "x".repeat(crate::authoring::composition::MAX_INPUT_BYTES)
    );
    let byte_error = crate::game::data::skill_data::RawSkillDatabase::from_ron_with_companions(
        FILE, &oversized, &limits, 64,
    )
    .err()
    .unwrap();
    assert_eq!(byte_error.code, "composition_source_invalid");
    assert_eq!(byte_error.actual.as_ref().unwrap()["code"], "Limit");
    assert!(byte_error.cause.contains("byte limit"));
    let low_expression = crate::game::value_expr::ExprParseLimits {
        max_input_bytes: 64,
        ..limits
    };
    let expression_error =
        crate::game::data::skill_data::RawSkillDatabase::from_ron_with_companions(
            FILE,
            SOURCE,
            &low_expression,
            64,
        )
        .err()
        .unwrap();
    assert_eq!(expression_error.code, "value_expr_input_limit");
    let error = crate::game::data::skill_data::RawSkillDatabase::from_ron_with_companions(
        FILE, &input, &limits, 4,
    )
    .err()
    .unwrap();
    assert_eq!(error.code, "composition_source_invalid");
    assert_eq!(error.actual.as_ref().unwrap()["code"], "Limit");
    assert!(error.cause.contains("YAML nesting limit"));
    assert_eq!(
        error.actual.as_ref().unwrap()["source_digest"],
        format!("{:x}", Sha256::digest(input.as_bytes()))
    );
    assert!(
        crate::game::data::skill_data::RawSkillDatabase::from_ron_with_companions(
            FILE, &input, &limits, 64
        )
        .is_ok()
    );
}


#[test]
fn authored_presence_publication_rejects_missing_modifier_relic_and_preserves_generation() {
    use crate::authoring::composition::{self, canonical_presence_tests as canonical};
    use crate::game::data::relic_data::RelicDatabase;
    let fixture = SourceFixture::new();
    let path = "data/relics/base.ron";
    let mut relics: RelicDatabase =
        ron::de::from_str(&fs::read_to_string(fixture.source().join(path)).unwrap()).unwrap();
    for relic in canonical::relics().relics {
        assert!(relics.get_by_id(&relic.id).is_none());
        relics.relics.push(relic);
    }
    write(&fixture, path, &ron::ser::to_string(&relics).unwrap());
    let document = canonical::resolved("fragment");
    write(&fixture, FILE, &document.to_string());
    let receipt = package(&fixture, "accepted-presence", &[FILE]).unwrap();
    assert_eq!(
        reload(&fixture, "accepted-presence").identity(),
        &receipt.content_identity
    );
    for i in 0..6 {
        let mut invalid = document.clone();
        canonical::program(&mut invalid)["modifiers"][i]["relic_definition"] =
            serde_json::json!("missing-presence-relic");
        composition::compile_json(&invalid.to_string()).unwrap();
        write(&fixture, FILE, &invalid.to_string());
        let output = format!("missing-relic-{i}");
        let error = package(&fixture, &output, &[FILE]).unwrap_err();
        assert_eq!(error.object_id.as_deref(), Some(canonical::SKILL));
        assert!(
            error.field_path.as_deref().unwrap().ends_with(&format!(
                "op.PresenceBurst.program.modifiers[{i}].relic_definition"
            )),
            "{error:?}"
        );
        assert!(!fixture.0.join(output).exists());
        assert_eq!(
            reload(&fixture, "accepted-presence").identity(),
            &receipt.content_identity
        );
    }
}

#[test]
fn authored_overlay_publication_rejects_missing_piercing_relic_and_preserves_generation() {
    use crate::authoring::composition::{self, canonical_overlay_tests as canonical};
    use crate::game::data::relic_data::RelicDatabase;
    let fixture = SourceFixture::new();
    let path = "data/relics/base.ron";
    let mut relics: RelicDatabase = ron::de::from_str(&fs::read_to_string(fixture.source().join(path)).unwrap()).unwrap();
    for relic in canonical::relics().relics {
        assert!(relics.get_by_id(&relic.id).is_none());
        relics.relics.push(relic);
    }
    write(&fixture, path, &ron::ser::to_string(&relics).unwrap());
    let document = canonical::resolved("fragment");
    write(&fixture, FILE, &document.to_string());
    let receipt = package(&fixture, "accepted-overlay", &[FILE]).unwrap();
    assert_eq!(reload(&fixture, "accepted-overlay").identity(), &receipt.content_identity);
    let mut invalid = document.clone();
    canonical::overlay(&mut invalid)["profile"]["line"]["piercing"]["relic_definition"] = serde_json::json!("missing-overlay-relic");
    composition::compile_json(&invalid.to_string()).unwrap();
    write(&fixture, FILE, &invalid.to_string());
    let error = package(&fixture, "missing-overlay-relic", &[FILE]).unwrap_err();
    assert_eq!(error.object_id.as_deref(), Some(canonical::SKILL));
    assert!(error.field_path.as_deref().unwrap().ends_with("op.Sustained.definition.BasicAttackOverlay.profile.line.piercing.relic_definition"), "{error:?}");
    assert!(!fixture.0.join("missing-overlay-relic").exists());
    assert_eq!(reload(&fixture, "accepted-overlay").identity(), &receipt.content_identity);
}

include!("catalog_references.inc.rs");
