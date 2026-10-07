use super::*;
use crate::game::data::equipment_data::{
    EquipmentMaterialCost, EquipmentMaterialMetadata, EquipmentMaterialType,
};
use std::sync::Arc;
fn material() -> EquipmentMaterialMetadata {
    EquipmentMaterialMetadata {
        id: "dust".into(),
        uuid: Uuid::from_u128(0x608),
        name: "Dust".into(),
        description: String::new(),
        material_type: EquipmentMaterialType::Generic,
        rarity: RiskLevel::ZAYIN,
        equipment_type: None,
    }
}
fn cost() -> Vec<EquipmentMaterialCost> {
    vec![EquipmentMaterialCost {
        material_id: "dust".into(),
        amount: 1,
    }]
}
use crate::game::battle::types::{BattleEquipmentBinding, BattleEquipmentEnhancement};
use crate::game::data::equipment_data::{EquipmentDatabase, EquipmentEnhancementRecipeMetadata};

fn numeric_fixture(weapon_attack: u32, growth: i32) -> (GameDataBase, BattleUnitDraft) {
    let uuid = Uuid::from_u128(0x601);
    let copy = Uuid::from_u128(0x602);
    let modifier = StatModifier {
        stat: StatId::Attack,
        kind: StatModifierKind::Flat,
        value: 5,
    };
    let mut weapon = weapon_with_permanent_modifier(uuid, modifier);
    weapon.weapon_profile.as_mut().unwrap().attack = weapon_attack;
    weapon.weapon_profile.as_mut().unwrap().spell_power = 20;
    let data = GameDataBuilder::empty()
        .with_equipment_data(Arc::new(EquipmentDatabase::with_all(
            vec![weapon],
            vec![material()],
            vec![],
            vec![EquipmentEnhancementRecipeMetadata {
                equipment_id: "test_weapon".into(),
                max_level: 3,
                costs_per_level: cost(),
                modifiers_per_level: vec![modifier],
            }],
        )))
        .build();
    let mut stacks = GrowthStack::new();
    stacks.add(GrowthId::KillStack, growth);
    let draft = BattleUnitDraft {
        owned_uuid: Uuid::from_u128(0x603),
        source: BattleUnitSource::Employee(UnitCombatProfile::for_employee(100, 40, 0, 0, 100)),
        threat_class: crate::game::battle::types::BattleUnitThreatClass::Normal,
        level: Tier::I,
        stat_scale: Default::default(),
        growth_stacks: stacks,
        equipment_layout: None,
        equipped_items: vec![BattleEquipmentBinding {
            base_uuid: uuid,
            persistent_copy_uuid: Some(copy),
            slot: None,
        }],
        equipped_item_enhancements: vec![BattleEquipmentEnhancement {
            base_uuid: uuid,
            persistent_copy_uuid: Some(copy),
            enhancement_level: 1,
        }],
        employee_start_condition: None,
    };
    (data, draft)
}

#[test]
fn r06_each_employee_and_modifier_contribution_is_removed_once() {
    // Independent authored arithmetic: employee40 + weapon40 + growth10 + permanent5 + enhancement5.
    for (removed, expected) in [("employee", 60), ("permanent", 95), ("enhancement", 95)] {
        let (mut data, mut draft) = numeric_fixture(40, 10);
        assert_eq!(
            persistent_stats_for_draft(&draft, &data).unwrap().attack,
            100
        );
        assert_eq!(
            data.equipment_data.items[0]
                .weapon_profile
                .as_ref()
                .unwrap()
                .attack,
            40
        );
        match removed {
            "employee" => {
                let BattleUnitSource::Employee(profile) = &mut draft.source else {
                    panic!("employee fixture");
                };
                assert_eq!(profile.stats.attack, 40);
                profile.stats.attack = 0;
            }
            "permanent" => {
                let mut equipment = (*data.equipment_data).clone();
                assert_eq!(equipment.items[0].triggered_effects.len(), 1);
                equipment.items[0]
                    .triggered_effects
                    .remove(&TriggerType::Permanent)
                    .unwrap();
                data.equipment_data = Arc::new(EquipmentDatabase::with_all(
                    equipment.items,
                    vec![material()],
                    vec![],
                    vec![EquipmentEnhancementRecipeMetadata {
                        equipment_id: "test_weapon".into(),
                        max_level: 3,
                        costs_per_level: cost(),
                        modifiers_per_level: vec![StatModifier {
                            stat: StatId::Attack,
                            kind: StatModifierKind::Flat,
                            value: 5,
                        }],
                    }],
                ));
            }
            "enhancement" => {
                assert_eq!(draft.equipped_item_enhancements[0].enhancement_level, 1);
                draft.equipped_item_enhancements[0].enhancement_level = 0;
            }
            _ => unreachable!(),
        }
        for _ in 0..2 {
            let persistent = persistent_stats_for_draft(&draft, &data).unwrap();
            let effective = effective_stats_for_draft(&draft, &data).unwrap();
            assert_eq!(persistent.attack, expected, "{removed}");
            assert_eq!(effective.attack, expected, "{removed}");
            assert_eq!(persistent.spell_power, 20);
            assert_eq!(effective.spell_power, 20);
        }
        eprintln!("R06_CONTRIBUTION removed={removed} attack={expected} baseline=100 weapon=40");
    }
}

#[test]
fn r06_numeric_weapon_growth_permanent_copy_compose_once_and_check_overflow() {
    for (weapon, growth, expected) in [(40, 10, 100), (0, 10, 60), (40, 0, 90)] {
        let (data, draft) = numeric_fixture(weapon, growth);
        for _ in 0..2 {
            let stats = persistent_stats_for_draft(&draft, &data).unwrap();
            assert_eq!(stats.attack, expected);
            assert_eq!(stats.spell_power, 20);
        }
        let effective = effective_stats_for_draft(&draft, &data).unwrap();
        assert_eq!(effective.attack, expected);
    }
    let (data, draft) = numeric_fixture(u32::MAX, 0);
    assert!(matches!(
        persistent_stats_for_draft(&draft, &data),
        Err(GameError::InvalidStaticData(_))
    ));
}

#[test]
fn r06_same_definition_copy_levels_are_isolated_and_mismatches_rejected() {
    let (mut data, mut draft) = numeric_fixture(0, 0);
    let mut db = (*data.equipment_data).clone();
    // Use a second non-weapon definition to exercise two legal same-definition copies.
    let mut item = db
        .get_by_uuid(&draft.equipped_items[0].base_uuid)
        .unwrap()
        .clone();
    item.equipment_type = EquipmentType::Accessory;
    item.weapon_profile = None;
    item.triggered_effects.clear();
    item.id = "copies".into();
    item.uuid = Uuid::from_u128(0x604);
    db = EquipmentDatabase::with_all(
        vec![
            db.get_by_uuid(&draft.equipped_items[0].base_uuid)
                .unwrap()
                .clone(),
            item.clone(),
        ],
        vec![material()],
        vec![],
        vec![EquipmentEnhancementRecipeMetadata {
            equipment_id: item.id.clone(),
            max_level: 3,
            costs_per_level: cost(),
            modifiers_per_level: vec![StatModifier {
                stat: StatId::Attack,
                kind: StatModifierKind::Flat,
                value: 5,
            }],
        }],
    );
    data.equipment_data = Arc::new(db);
    draft.equipped_item_enhancements.clear();
    for (copy, level) in [(0x605, 1), (0x606, 2)] {
        draft.equipped_items.push(BattleEquipmentBinding {
            base_uuid: item.uuid,
            persistent_copy_uuid: Some(Uuid::from_u128(copy)),
            slot: None,
        });
        draft
            .equipped_item_enhancements
            .push(BattleEquipmentEnhancement {
                base_uuid: item.uuid,
                persistent_copy_uuid: Some(Uuid::from_u128(copy)),
                enhancement_level: level,
            });
    }
    assert_eq!(
        persistent_stats_for_draft(&draft, &data).unwrap().attack,
        60
    ); // 40+Permanent5+copies15
    for mode in 0..4 {
        let mut invalid = draft.clone();
        match mode {
            0 => invalid
                .equipped_item_enhancements
                .push(invalid.equipped_item_enhancements[0]),
            1 => invalid.equipped_item_enhancements[0].base_uuid = Uuid::nil(),
            2 => invalid.equipped_item_enhancements[0].persistent_copy_uuid = None,
            _ => invalid.equipped_item_enhancements[0].persistent_copy_uuid = Some(Uuid::nil()),
        }
        assert!(persistent_stats_for_draft(&invalid, &data).is_err());
    }
}

#[test]
fn r06_checked_cadence_preserves_other_weapon_timers_and_rejects_invalid() {
    use crate::game::battle::types::checked_scale_attack_interval_ms as scale;
    for (base, points, expected) in [(1500, 200, 750), (1001, 150, 668), (1, 200, 1)] {
        assert_eq!(scale(base, points).unwrap(), expected);
    }
    for (base, points) in [(0, 100), (100, 0), (u64::MAX, 1)] {
        assert!(scale(base, points).is_err());
    }
    let mut profile = UnitCombatProfile::for_employee(100, 10, 0, 0, 200);
    let weapon = crate::game::data::equipment_data::WeaponCombatProfile::default();
    profile.apply_weapon_profile(&weapon).unwrap();
    assert_eq!(profile.stats.attack_interval_ms, 750);
    assert_eq!(profile.basic_attack.windup_ms, weapon.windup_ms);
    assert_eq!(
        profile.basic_attack.ranged_reposition_ms,
        weapon.ranged_reposition_ms
    );
    assert_eq!(
        serde_json::to_string(&profile.basic_attack.delivery).unwrap(),
        serde_json::to_string(&weapon.delivery).unwrap()
    );
    profile.basic_attack_speed_percent = 0;
    let before = serde_json::to_string(&profile).unwrap();
    assert!(profile.apply_weapon_profile(&weapon).is_err());
    assert_eq!(before, serde_json::to_string(&profile).unwrap());
}
