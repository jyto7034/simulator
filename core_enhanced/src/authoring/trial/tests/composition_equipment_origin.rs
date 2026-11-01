use super::*;
use crate::authoring::provenance::{EquipmentOriginClaim, EquipmentOriginRole};

#[test]
fn integration_equipment_origin_rejects_mixed_role_definition_uuid_and_field() {
    let (fixture, _, document) = state_world_fixture();
    let source = include_str!("../../../../tests/fixtures/skill_composition/adaptive_shield.skill.yaml")
        .replace("equipment: r11_weapon", "equipment: standard_training_sword");
    let trial = CompositionTrial::prepare(fixture.root(), &fixture.selection(), &source,
        checked(&document), None, TrialLimits::default(), &CancellationToken::default()).unwrap();
    let equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    let uuid = equipment.get_by_id("standard_training_sword").unwrap().uuid;
    let claim = EquipmentOriginClaim {
        generation: Some(&trial.identity().content), role: EquipmentOriginRole::AdaptiveShield,
        equipment_uuid: Some(uuid), definition_id: Some("standard_training_sword"),
        source_file: &trial.identity().artifact_path,
        field_path: "equipment_defenses[0].program.AdaptiveBasicShield",
    };
    let origin = trial.observed_equipment_origin(&claim).expect("normal original producer");
    assert_eq!(origin.definition, "values[0].value.DefenseProgram");
    assert!(origin.inputs.contains(&"values[0].value".to_owned()), "{origin:?}");
    let mut accepted = Vec::new();
    for (name, bad) in [
        ("wrong_role", EquipmentOriginClaim { role: EquipmentOriginRole::SkillBarrier, ..claim }),
        ("unknown_equipment_uuid", EquipmentOriginClaim { equipment_uuid: Some(uuid::Uuid::nil()), ..claim }),
        ("missing_equipment_uuid", EquipmentOriginClaim { equipment_uuid: None, ..claim }),
        ("wrong_definition", EquipmentOriginClaim { definition_id: Some("another_armor"), ..claim }),
        ("missing_definition", EquipmentOriginClaim { definition_id: None, ..claim }),
        ("unrelated_existing_field", EquipmentOriginClaim { field_path: "equipment_defenses[0].equipment", ..claim }),
        ("nonproducer_descendant", EquipmentOriginClaim { field_path: "equipment_defenses[0].program.AdaptiveBasicShield.duration_ms", ..claim }),
        ("unknown_descendant", EquipmentOriginClaim { field_path: "equipment_defenses[0].program.AdaptiveBasicShield.absent", ..claim }),
    ] {
        if trial.observed_equipment_origin(&bad).is_some() { accepted.push(name); }
    }
    eprintln!("equipment origin incorrectly accepted: {accepted:?}");
    assert!(accepted.is_empty(), "mixed equipment producer claims accepted: {accepted:?}");
    assert!(trial.observed_equipment_origin(&EquipmentOriginClaim { generation: None, ..claim }).is_none());
    assert!(trial.observed_equipment_origin(&EquipmentOriginClaim { source_file: "other.ron", ..claim }).is_none());
    assert_eq!(trial.observed_equipment_origin(&claim).unwrap().definition, origin.definition);
}

#[test]
fn integration_equipment_origin_separates_registered_roles_and_original_versions() {
    use crate::game::{
        ability::equipment_support::ControlDurationKind,
        battle::{core::equipment_support::SupportChannel, damage::DamageType},
    };
    use EquipmentOriginRole as Role;
    let (fixture, _, document) = state_world_fixture();
    let equipment_path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(equipment_path)).unwrap();
    let uuid = equipment.get_by_id("standard_training_sword").unwrap().uuid;
    let other_uuid = uuid::Uuid::from_u128(0xe991);
    let mut other = equipment.get_by_id("standard_training_sword").unwrap().clone();
    other.id = "origin-other-weapon".into();
    other.uuid = other_uuid;
    equipment.items.push(other);
    fixture.write(equipment_path, ron::ser::to_string(&equipment).unwrap());
    let mut authored = crate::authoring::composition::parse_yaml(
        include_str!("../../../../tests/fixtures/skill_composition/adaptive_shield.skill.yaml")).unwrap();
    authored["equipment_defenses"][0]["equipment"] = json!("standard_training_sword");
    authored["equipment_defenses"].as_array_mut().unwrap().push(json!({
        "equipment":"origin-other-weapon", "program":{"Shared":"adaptive-defense"}
    }));
    authored["values"].as_array_mut().unwrap().push(json!({
        "name":"support", "value":{"SupportProgram":{
            "heal_percent":20, "shield_percent":30, "received_heal_percent":{"moment":"Live","root":{"Const":{"value":10,"unit":"Scalar"}}},
            "control_duration":{"reduction_percent":15,"kinds":["Stun"]},
            "shield_beneficiary_damage_percent":25
        }}
    }));
    authored["equipment_supports"] = json!([
        {"equipment":"standard_training_sword","program":{"Shared":"support"}}
    ]);
    authored["values"].as_array_mut().unwrap().push(json!({
        "name":"attack", "value":{"AttackProgram":{
            "reactions":[],
            "basic_speed_points":{"moment":"Live","root":{"Const":{"value":12,"unit":"AttackSpeedPoints"}}},
            "primary_damage_percent":{"moment":"Live","root":{"Const":{"value":17,"unit":"Scalar"}}},
            "area_damage_percent":{"moment":"Live","root":{"Const":{"value":19,"unit":"Scalar"}}},
            "direct_physical_penetration_percent":21, "direct_magic_penetration_percent":23
        }}
    }));
    authored["equipment_attacks"] = json!([
        {"equipment":"standard_training_sword","program":{"Shared":"attack"}}
    ]);
    let source = authored.to_string();
    let artifact = crate::authoring::composition::compile_yaml(&source).unwrap();
    let trial = CompositionTrial::prepare(fixture.root(), &fixture.selection(), &source,
        checked(&document), None, TrialLimits::default(), &CancellationToken::default()).unwrap();
    // These are literal source addresses from the authored declarations, not
    // output from the compiler's producer lookup or the query under test.
    let expected = [
        (Role::AdaptiveShield, "standard_training_sword", "equipment_defenses[0].program.AdaptiveBasicShield", "values[0].value.DefenseProgram"),
        (Role::EquipmentSupport(SupportChannel::Heal), "standard_training_sword", "equipment_supports[0].program.heal_percent", "values[1].value.SupportProgram"),
        (Role::EquipmentSupport(SupportChannel::Shield), "standard_training_sword", "equipment_supports[0].program.shield_percent", "values[1].value.SupportProgram"),
        (Role::ReceivedHeal, "standard_training_sword", "equipment_supports[0].program.received_heal_percent", "values[1].value.SupportProgram"),
        (Role::ControlDuration(ControlDurationKind::Stun), "standard_training_sword", "equipment_supports[0].program.control_duration", "values[1].value.SupportProgram"),
        (Role::ShieldBeneficiaryDamage, "standard_training_sword", "equipment_supports[0].program.shield_beneficiary_damage_percent", "values[1].value.SupportProgram"),
        (Role::EquipmentSpeed, "equipment:standard_training_sword", "equipment_attacks[0].program.basic_speed_points", "values[2].value.AttackProgram"),
        (Role::PrimaryDamage, "equipment:standard_training_sword", "equipment_attacks[0].program.primary_damage_percent", "values[2].value.AttackProgram"),
        (Role::AreaDamage, "equipment:standard_training_sword", "equipment_attacks[0].program.area_damage_percent", "values[2].value.AttackProgram"),
        (Role::DirectPenetration(DamageType::Physical), "equipment:standard_training_sword", "equipment_attacks[0].program.direct_physical_penetration_percent", "values[2].value.AttackProgram"),
        (Role::DirectPenetration(DamageType::Magic), "equipment:standard_training_sword", "equipment_attacks[0].program.direct_magic_penetration_percent", "values[2].value.AttackProgram"),
    ];
    let mut imported = crate::authoring::composition::Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    let other_original = crate::authoring::composition::compile_yaml(&format!("{source}\n ")).unwrap();
    for (i, &(role, definition, path, original)) in expected.iter().enumerate() {
        let claim = EquipmentOriginClaim {
            generation: Some(&trial.identity().content), role,
            equipment_uuid: Some(uuid), definition_id: Some(definition),
            source_file: &trial.identity().artifact_path, field_path: path,
        };
        assert_eq!(trial.observed_equipment_origin(&claim).unwrap().definition, original);
        for (j, &(other_role, _, other_path, _)) in expected.iter().enumerate() {
            if i == j { continue; }
            assert!(trial.observed_equipment_origin(&EquipmentOriginClaim { role: other_role, ..claim }).is_none(),
                "{role:?} accepted another role {other_role:?}");
            assert!(trial.observed_equipment_origin(&EquipmentOriginClaim { field_path: other_path, ..claim }).is_none(),
                "{role:?} accepted another producer field {other_path}");
        }
        assert!(trial.observed_equipment_origin(&EquipmentOriginClaim { equipment_uuid: Some(other_uuid), ..claim }).is_none());
        assert!(trial.session().composition_equipment_origin(&imported, &claim).is_none());
        assert!(trial.session().composition_equipment_origin(&other_original, &claim).is_none());
        assert_eq!(trial.session().composition_equipment_origin(&artifact, &claim).unwrap().definition, original);
    }
    imported.verify_yaml_source(&source).unwrap();
    let claim = EquipmentOriginClaim {
        generation: Some(&trial.identity().content), role: Role::AdaptiveShield,
        equipment_uuid: Some(other_uuid), definition_id: Some("origin-other-weapon"),
        source_file: &trial.identity().artifact_path,
        field_path: "equipment_defenses[1].program.AdaptiveBasicShield",
    };
    assert_eq!(trial.session().composition_equipment_origin(&imported, &claim).unwrap().definition, "values[0].value.DefenseProgram");
    assert!(trial.observed_equipment_origin(&EquipmentOriginClaim { equipment_uuid: Some(uuid), ..claim }).is_none());
    let claim = EquipmentOriginClaim {
        role: Role::ControlDuration(ControlDurationKind::Freeze),
        equipment_uuid: Some(uuid), definition_id: Some("standard_training_sword"),
        field_path: "equipment_supports[0].program.control_duration", ..claim
    };
    assert!(trial.observed_equipment_origin(&claim).is_none(), "unregistered control kind");
}
