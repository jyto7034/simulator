use super::*;
use crate::game::data::relic_data::*;

fn definition(id: &str, number: u128, duplicates: bool) -> RelicDefinition {
    RelicDefinition {
        id: id.into(),
        uuid: Uuid::from_u128(number),
        name: id.into(),
        description: "U08 authored test value".into(),
        rarity: "probe".into(),
        allow_duplicates: duplicates,
        incompatible_with: vec![],
        rules: vec![],
        effects: vec![RelicDamageEffect {
            id: "damage".into(),
            selector: Default::default(),
            flat: 20,
            percent: 20,
        }],
        corruption: Some(RelicCorruptionVariant {
            rules: vec![],
            effects: vec![RelicDamageEffect {
                id: "damage".into(),
                selector: Default::default(),
                flat: 40,
                percent: 50,
            }],
            penalties: vec![
                RelicPenalty::DefenseFlat { amount: -15 },
                RelicPenalty::MagicResistFlat { amount: -10 },
            ],
        }),
        dismantle_equipment_dust: 7,
    }
}
fn bytes(value: &impl Serialize) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}

#[test]
fn normal_and_corrupt_copies_share_duplicate_policy_and_failed_transfer_preserves_both_owners() {
    for dormant in [false, true] {
        let mut definition = definition("one", 1, false);
        if dormant {
            definition.effects[0].selector.equipment_id = Some("absent_weapon_probe".into());
            definition.corruption.as_mut().unwrap().effects[0]
                .selector
                .equipment_id = Some("absent_weapon_probe".into());
        }
        let db = RelicDatabase {
            relics: vec![definition],
        };
        db.validate_contract().unwrap();
        let definitions = bytes(&db);
        for reverse in [false, true] {
            let mut inventory = RelicInventory::default();
            let mut uuids = UuidManager::new(100);
            let first = inventory.grant(&db.relics[0], &mut uuids).unwrap();
            let second = inventory.grant(&db.relics[0], &mut uuids).unwrap();
            assert_ne!(first, second);
            inventory.corrupt(second, &db).unwrap();
            let employee_a = Uuid::from_u128(21);
            let employee_b = Uuid::from_u128(22);
            let (accepted, rejected) = if reverse {
                (second, first)
            } else {
                (first, second)
            };
            // The same definition is legal on different employees, including
            // when both variants' authored provider conditions are dormant.
            inventory.equip(accepted, employee_a, 0, &db).unwrap();
            inventory.equip(rejected, employee_b, 2, &db).unwrap();
            let before = bytes(&inventory);
            let ids = bytes(&uuids);
            assert!(
                matches!(
                    inventory.equip(rejected, employee_a, 1, &db),
                    Err(GameError::InvalidAction)
                ),
                "reverse={reverse} dormant={dormant}"
            );
            assert_eq!(bytes(&inventory), before);
            assert_eq!(bytes(&uuids), ids);
            assert_eq!(bytes(&db), definitions);
            assert_eq!(
                inventory.employee_slots(employee_a),
                [Some(accepted), None, None]
            );
            assert_eq!(
                inventory.employee_slots(employee_b),
                [None, None, Some(rejected)]
            );
            assert_eq!(
                inventory.equipped_to(rejected).unwrap(),
                Some((employee_b, 2))
            );
            assert_eq!(inventory.get(first).unwrap().variant, RelicVariant::Normal);
            assert_eq!(
                inventory.get(second).unwrap().variant,
                RelicVariant::Corrupt
            );
            inventory
                .validate(&db, |id| id == employee_a || id == employee_b)
                .unwrap();
        }
    }
}

#[test]
fn incompatibility_is_symmetric_and_independent_of_variant_or_dormant_source_conditions() {
    let mut a = definition("a", 1, true);
    let mut b = definition("b", 2, true);
    a.incompatible_with = vec!["b".into()];
    b.effects[0].selector.equipment_id = Some("absent_weapon_probe".into());
    let db = RelicDatabase { relics: vec![a, b] };
    db.validate_contract().unwrap();
    for reverse in [false, true] {
        let mut inventory = RelicInventory::default();
        let mut uuids = UuidManager::new(102);
        let first = inventory.grant(&db.relics[0], &mut uuids).unwrap();
        let second = inventory.grant(&db.relics[1], &mut uuids).unwrap();
        inventory.corrupt(second, &db).unwrap();
        let (accepted, rejected) = if reverse {
            (second, first)
        } else {
            (first, second)
        };
        let employee = Uuid::from_u128(31);
        inventory
            .equip(accepted, employee, if reverse { 2 } else { 0 }, &db)
            .unwrap();
        let before = bytes(&inventory);
        assert!(inventory.equip(rejected, employee, 1, &db).is_err());
        assert_eq!(bytes(&inventory), before);
    }
}

#[test]
fn three_slots_move_and_recover_preserve_corrupt_copy_without_a_weapon_dependency() {
    let db = RelicDatabase {
        relics: vec![definition("same", 1, true)],
    };
    let mut inventory = RelicInventory::default();
    let mut uuids = UuidManager::new(103);
    let employee = Uuid::from_u128(41);
    let other = Uuid::from_u128(42);
    let copies: Vec<_> = (0..4)
        .map(|_| inventory.grant(&db.relics[0], &mut uuids).unwrap())
        .collect();
    for (slot, copy) in copies[..3].iter().enumerate() {
        inventory.equip(*copy, employee, slot, &db).unwrap();
    }
    let before = bytes(&inventory);
    assert!(inventory.equip(copies[3], employee, 3, &db).is_err());
    assert_eq!(bytes(&inventory), before);
    inventory.corrupt(copies[0], &db).unwrap();
    inventory.equip(copies[0], other, 1, &db).unwrap();
    assert_eq!(inventory.employee_slots(employee)[0], None);
    assert_eq!(inventory.equipped_to(copies[0]).unwrap(), Some((other, 1)));
    inventory.recover_employee(other);
    let recovered = bytes(&inventory);
    inventory.recover_employee(other);
    assert_eq!(bytes(&inventory), recovered);
    assert_eq!(
        inventory.get(copies[0]).unwrap().variant,
        RelicVariant::Corrupt
    );
    assert!(inventory.equipped_to(copies[0]).unwrap().is_none());
    assert_eq!(inventory.copies().count(), 4);
}

#[test]
fn forged_double_placement_missing_owner_or_variant_is_rejected_at_restore_validation() {
    let db = RelicDatabase {
        relics: vec![definition("one", 1, true)],
    };
    let mut inventory = RelicInventory::default();
    let mut uuids = UuidManager::new(104);
    let copy = inventory.grant(&db.relics[0], &mut uuids).unwrap();
    let employee = Uuid::from_u128(51);
    inventory
        .slots
        .insert(employee, RelicSlots([Some(copy), None, None]));
    inventory
        .slots
        .insert(Uuid::from_u128(52), RelicSlots([None, Some(copy), None]));
    assert!(inventory.validate(&db, |_| true).is_err());
    inventory.slots.remove(&Uuid::from_u128(52));
    assert!(inventory.validate(&db, |_| false).is_err());
    inventory.validate(&db, |id| id == employee).unwrap();
    inventory.copies.get_mut(&copy).unwrap().variant = RelicVariant::Corrupt;
    let mut changed = db;
    changed.relics[0].corruption = None;
    assert!(inventory.validate(&changed, |_| true).is_err());
}

#[test]
fn catalyst_variant_and_dust_candidate_failure_preserve_legacy_asset_and_committed_copy() {
    let db = RelicDatabase {
        relics: vec![definition("one", 1, true)],
    };
    let mut inventory = RelicInventory::default();
    let mut uuids = UuidManager::new(105);
    let copy = inventory.grant(&db.relics[0], &mut uuids).unwrap();
    let mut legacy = crate::game::skill_fragment::SkillFragmentInventory::new();
    legacy.add_fragment_dust(99).unwrap();
    let mut resources = RunCraftingResources {
        equipment_dust: u32::MAX,
        relic_corruption_catalysts: 1,
    };
    let original = bytes(&inventory);
    let materials = resources;
    let mut candidate = inventory.clone();
    let mut proposed = resources;
    candidate.corrupt(copy, &db).unwrap();
    proposed.consume_relic_catalyst().unwrap();
    assert!(proposed.grant_equipment_dust(7).is_err());
    assert_eq!(bytes(&inventory), original);
    assert_eq!(resources, materials);
    assert_eq!(legacy.fragment_dust(), 99);
    inventory.corrupt(copy, &db).unwrap();
    resources.consume_relic_catalyst().unwrap();
    let committed = bytes(&inventory);
    assert!(inventory.corrupt(copy, &db).is_err());
    assert_eq!(bytes(&inventory), committed);
    assert_eq!(resources.relic_corruption_catalysts, 0);
    assert_eq!(legacy.fragment_dust(), 99);
    inventory.unequip(copy).unwrap();
    assert_eq!(inventory.get(copy).unwrap().variant, RelicVariant::Corrupt);
}

#[test]
fn typed_authoring_rejects_unsupported_recipes_duplicate_effects_and_missing_penalties() {
    let mut db = RelicDatabase {
        relics: vec![definition("one", 1, true)],
    };
    db.validate_contract().unwrap();
    let duplicate_effect = db.relics[0].effects[0].clone();
    db.relics[0].effects.push(duplicate_effect);
    assert!(db.validate_contract().is_err());
    db.relics[0].effects.pop();
    db.relics[0].corruption.as_mut().unwrap().penalties.clear();
    assert!(db.validate_contract().is_err());
    let input = r#"(relics:[], unsupported_closure: Replace())"#;
    assert!(ron::de::from_str::<RelicDatabase>(input).is_err());
    let duplicate = r#"{"copies":{},"slots":{"00000000-0000-0000-0000-000000000001":[null,null,null],"00000000-0000-0000-0000-000000000001":[null,null,null]}}"#;
    assert!(serde_json::from_str::<RelicInventory>(duplicate).is_err());
}
