//! R07 exact-copy integration seams. Public default-RON coverage lives in authoring::trial.
use super::*;
use crate::game::data::equipment_data::EquipmentMaterialCost;
use crate::game::data::skill_fragment_data::{SkillFragmentForm, SkillFragmentRecipe};
use crate::game::managers::uuid_manager::UuidManager;
use crate::game::reward::RewardEffect;

fn setup() -> (GameCore, Uuid, SkillFragmentMetadata, Vec<Uuid>) {
    let mut meta = active_skill_fragment("r07_copy", 0x707, "r07_skill");
    if let SkillFragmentEffectDef::ActiveSkill { forms, .. } = &mut meta.effect {
        forms.insert(
            "awakened".into(),
            SkillFragmentForm {
                resonance_max: 100,
                skill_id: SkillId::from("r07_skill"),
                compatibility: None,
                bloom_recipe: Some(SkillFragmentRecipe {
                    enkephalin: 3,
                    materials: vec![],
                }),
            },
        );
    }
    let data = game_data_with_skill_fragments(vec![meta.clone()]);
    let mut core = GameCore::new(data, 707);
    let player = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player);
    let owners = core.roster().unwrap().combat_eligible_employee_ids();
    (core, player, meta, owners)
}
fn grant(core: &mut GameCore, meta: &SkillFragmentMetadata) -> Uuid {
    core.apply_grant_effects(&[RewardEffect::GrantSkillFragment {
        fragment_id: meta.id.clone(),
    }])
    .unwrap()
    .skill_fragment_diffs[0]
        .copy_uuid
}
fn maintenance(core: &mut GameCore, player: Uuid) {
    let node_id = force_first_available_node(
        core,
        MapNodeCategory::Maintenance,
        "maintenance",
        MapNodePayload::Maintenance,
    );
    core.execute(player, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
}
fn roots(core: &GameCore) -> Value {
    json!({"copies":core.state.skill_fragments,"roster":core.state.roster,"inventory":core.state.inventory,"uuids":core.state.uuid_manager,"currency":core.state.enkephalin,"resources":core.run_resources()})
}

#[test]
fn u05_selected_form_capacity_floor_and_initial_charge_use_actual_copy() {
    for (maximum, expected, bonus) in [
        (1, 0, 0),
        (4, 0, 0),
        (5, 1, 0),
        (99, 19, 0),
        (100, 20, 0),
        (100, 60, 40),
    ] {
        let authored = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r09/fragments.ron"))
            .replace("resonance_max: 100", "resonance_max: 137")
            .replace("effect: ActiveSkill(forms: {", &format!("effect: ActiveSkill(forms: {{\"selected\": (skill_id: \"r08_strike\", resonance_max: {maximum}, bloom_recipe: Some((enkephalin: 1, materials: []))),"));
        let catalog: SkillFragmentDatabase = ron::de::from_str(&authored).unwrap();
        let meta = catalog.fragments[0].clone();
        let mut core = GameCore::new(game_data_with_skill_fragments(vec![meta.clone()]), 707);
        let player = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player);
        let employee = core.roster().unwrap().combat_eligible_employee_ids()[0];
        let copy = grant(&mut core, &meta);
        core.execute(
            player,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid: employee,
                copy_uuid: copy,
            },
        )
        .unwrap();
        maintenance(&mut core, player);
        core.execute(
            player,
            PlayerBehavior::AwakenSkillFragment {
                copy_uuid: copy,
                form_id: "selected".into(),
            },
        )
        .unwrap();
        core.execute(
            player,
            PlayerBehavior::SelectSkillFragmentForm {
                copy_uuid: copy,
                form_id: "selected".into(),
            },
        )
        .unwrap();
        if bonus > 0 {
            let serum = consumable_meta(
                0x505,
                "u05_charge",
                ConsumableTier::Rare,
                ConsumableEffect::InitialSkillCharge { percent: bonus },
            );
            core.roster_mut()
                .unwrap()
                .get_mut(&employee)
                .unwrap()
                .apply_consumable_modifier(Uuid::from_u128(0x506), &serum);
        }
        let profile = core
            .state
            .roster
            .get(&employee)
            .unwrap()
            .combat_profile_for_battle(
                &core.game_data.skill_fragment_data,
                &core.state.skill_fragments,
            )
            .unwrap();
        assert_eq!(
            (
                profile.resonance.max,
                profile.resonance.start,
                profile.resonance.gain_lock_ms
            ),
            (maximum, expected, 0)
        );
        let source = profile.fragment_source.unwrap();
        assert_eq!(source.copy_uuid, copy);
        assert_eq!(source.form_id, "selected");
        assert_ne!(
            maximum, 137,
            "the base form must not supply the selected capacity"
        );
    }
    let authored = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/r09/fragments.ron"
    ));
    assert!(ron::de::from_str::<SkillFragmentDatabase>(
        &authored.replace(", resonance_max: 100", "")
    )
    .is_err());
    let zero: SkillFragmentDatabase =
        ron::de::from_str(&authored.replace("resonance_max: 100", "resonance_max: 0")).unwrap();
    assert!(zero.validate_contract().is_err());
}
#[test]
fn r07_f01_normal_grant_exact_receipts_and_failed_batch_retry_preserve_uuid() {
    let (mut core, _, meta, _) = setup();
    let before = roots(&core);
    let next = core.state.uuid_manager.peek(UuidManager::NS_OWNED_FRAGMENT);
    assert!(core
        .apply_grant_effects(&[
            RewardEffect::GrantSkillFragment {
                fragment_id: meta.id.clone()
            },
            RewardEffect::GrantEquipmentMaterial {
                material_id: "missing".into(),
                amount: 1
            }
        ])
        .is_err());
    assert_eq!(roots(&core), before);
    let a = grant(&mut core, &meta);
    let b = grant(&mut core, &meta);
    assert_eq!(a, next);
    assert_ne!(a, b);
    assert_eq!(core.state.skill_fragments.count(&meta.id), 2);
    assert_eq!(
        core.state.skill_fragments.get(&a).unwrap().selected_form,
        "base"
    );
}
#[test]
fn r07_f02_f03_copy_independence_transfer_occupied_and_missing_rollback() {
    let (mut core, player, meta, owners) = setup();
    let a = grant(&mut core, &meta);
    let b = grant(&mut core, &meta);
    core.execute(
        player,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: owners[0],
            copy_uuid: a,
        },
    )
    .unwrap();
    core.execute(
        player,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: owners[1],
            copy_uuid: b,
        },
    )
    .unwrap();
    let before = roots(&core);
    for command in [
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: owners[1],
            copy_uuid: a,
        },
        PlayerBehavior::TransferSkillFragment {
            copy_uuid: a,
            target_employee_uuid: owners[1],
        },
        PlayerBehavior::TransferSkillFragment {
            copy_uuid: Uuid::nil(),
            target_employee_uuid: owners[0],
        },
    ] {
        assert!(core.execute(player, command).is_err());
        assert_eq!(roots(&core), before);
    }
    core.execute(
        player,
        PlayerBehavior::UnequipSkillFragment {
            employee_uuid: owners[1],
            copy_uuid: b,
        },
    )
    .unwrap();
    maintenance(&mut core, player);
    core.execute(
        player,
        PlayerBehavior::AwakenSkillFragment {
            copy_uuid: a,
            form_id: "awakened".into(),
        },
    )
    .unwrap();
    assert_eq!(
        core.state.skill_fragments.get(&a).unwrap().selected_form,
        "base"
    );
    core.execute(
        player,
        PlayerBehavior::SelectSkillFragmentForm {
            copy_uuid: a,
            form_id: "awakened".into(),
        },
    )
    .unwrap();
    core.execute(
        player,
        PlayerBehavior::TransferSkillFragment {
            copy_uuid: a,
            target_employee_uuid: owners[1],
        },
    )
    .unwrap();
    let copy = core.state.skill_fragments.get(&a).unwrap();
    assert_eq!(copy.equipped_to, Some(owners[1]));
    assert_eq!(copy.selected_form, "awakened");
    assert!(copy.unlocked_forms.contains("awakened"));
    assert_eq!(
        core.state.skill_fragments.get(&b).unwrap().selected_form,
        "base"
    );
    assert_eq!(
        core.state
            .roster
            .get(&owners[0])
            .unwrap()
            .skill_fragments
            .active_copy_uuid(),
        None
    );
    core.state
        .skill_fragments
        .validate_bindings(&core.state.roster, &core.game_data.skill_fragment_data)
        .unwrap();
}
#[test]
fn r07_f04_f11_form_queries_costs_locked_insufficient_noop_are_consistent() {
    let (mut core, player, meta, _) = setup();
    let a = grant(&mut core, &meta);
    let before = roots(&core);
    assert!(core
        .execute(
            player,
            PlayerBehavior::AwakenSkillFragment {
                copy_uuid: a,
                form_id: "awakened".into()
            }
        )
        .is_err());
    assert_eq!(roots(&core), before);
    maintenance(&mut core, player);
    core.state.enkephalin.amount = 2;
    let before = roots(&core);
    let snapshot = core.get_inventory_snapshot_dto().unwrap();
    let form = snapshot.skill_fragments[0]
        .forms
        .iter()
        .find(|form| form.form_id == "awakened")
        .unwrap();
    assert_eq!(
        form.unlock_unavailable_reason.as_deref(),
        Some("insufficient_enkephalin")
    );
    assert!(!form.can_select);
    assert_eq!(roots(&core), before);
    for command in [
        PlayerBehavior::AwakenSkillFragment {
            copy_uuid: a,
            form_id: "awakened".into(),
        },
        PlayerBehavior::SelectSkillFragmentForm {
            copy_uuid: a,
            form_id: "awakened".into(),
        },
    ] {
        assert!(core.execute(player, command).is_err());
        assert_eq!(roots(&core), before);
    }
    core.state.enkephalin.amount = 3;
    assert!(matches!(
        core.execute(
            player,
            PlayerBehavior::AwakenSkillFragment {
                copy_uuid: a,
                form_id: "awakened".into()
            }
        )
        .unwrap(),
        BehaviorResult::SkillFragmentAwakened {
            changed: true,
            enkephalin_spent: 3,
            ..
        }
    ));
    let before = roots(&core);
    assert!(matches!(
        core.execute(
            player,
            PlayerBehavior::AwakenSkillFragment {
                copy_uuid: a,
                form_id: "awakened".into()
            }
        )
        .unwrap(),
        BehaviorResult::SkillFragmentAwakened {
            changed: false,
            enkephalin_spent: 0,
            ..
        }
    ));
    assert_eq!(roots(&core), before);
    assert!(matches!(
        core.execute(
            player,
            PlayerBehavior::SelectSkillFragmentForm {
                copy_uuid: a,
                form_id: "base".into()
            }
        )
        .unwrap(),
        BehaviorResult::SkillFragmentFormSelected { changed: false, .. }
    ));
    assert_eq!(roots(&core), before);
    let cost = |amount| EquipmentMaterialCost {
        material_id: "part".into(),
        amount,
    };
    assert_eq!(
        SkillFragmentRecipe {
            enkephalin: 0,
            materials: vec![cost(1), cost(3)]
        }
        .aggregated_materials()
        .unwrap()["part"],
        4
    );
    assert!(SkillFragmentRecipe {
        enkephalin: 0,
        materials: vec![cost(u32::MAX), cost(1)]
    }
    .aggregated_materials()
    .is_none());
    assert!(SkillFragmentRecipe {
        enkephalin: 0,
        materials: vec![cost(0)]
    }
    .aggregated_materials()
    .is_none());
}
#[test]
fn r07_f05_confirmed_dismantle_removes_only_selected_copy_and_failed_yield_rolls_back() {
    let (mut core, player, meta, owners) = setup();
    let a = grant(&mut core, &meta);
    let b = grant(&mut core, &meta);
    core.execute(
        player,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: owners[0],
            copy_uuid: a,
        },
    )
    .unwrap();
    core.execute(
        player,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: owners[1],
            copy_uuid: b,
        },
    )
    .unwrap();
    maintenance(&mut core, player);
    let before = roots(&core);
    assert!(core
        .execute(
            player,
            PlayerBehavior::DismantleSkillFragment {
                copy_uuid: a,
                confirmed: false
            }
        )
        .is_err());
    assert_eq!(roots(&core), before);
    core.state.enkephalin.amount = u32::MAX;
    let before = roots(&core);
    assert!(core
        .execute(
            player,
            PlayerBehavior::DismantleSkillFragment {
                copy_uuid: a,
                confirmed: true
            }
        )
        .is_err());
    assert_eq!(roots(&core), before);
    core.state.enkephalin.amount = 0;
    let other = core.state.skill_fragments.get(&b).unwrap().clone();
    core.execute(
        player,
        PlayerBehavior::DismantleSkillFragment {
            copy_uuid: a,
            confirmed: true,
        },
    )
    .unwrap();
    assert!(core.state.skill_fragments.get(&a).is_none());
    assert_eq!(core.state.skill_fragments.get(&b), Some(&other));
    assert_eq!(
        core.state
            .roster
            .get(&owners[0])
            .unwrap()
            .skill_fragments
            .active_copy_uuid(),
        None
    );
    assert_eq!(core.state.enkephalin.amount, 2);
}
#[test]
fn r07_f06_death_releases_exact_copy_preserves_forms_and_repeat_is_idempotent() {
    let (mut core, player, meta, owners) = setup();
    let a = grant(&mut core, &meta);
    core.execute(
        player,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: owners[0],
            copy_uuid: a,
        },
    )
    .unwrap();
    maintenance(&mut core, player);
    core.execute(
        player,
        PlayerBehavior::AwakenSkillFragment {
            copy_uuid: a,
            form_id: "awakened".into(),
        },
    )
    .unwrap();
    let mut roster = core.state.roster.clone();
    let mut inventory = core.state.inventory.clone();
    let mut fragments = core.state.skill_fragments.clone();
    roster
        .get_mut(&owners[0])
        .unwrap()
        .condition
        .physical_damage = MAX_PHYSICAL_DAMAGE;
    core.recover_assets_for_new_deaths(&mut roster, &mut inventory, &mut fragments)
        .unwrap();
    let once = json!({"r":roster,"i":inventory,"f":fragments});
    core.recover_assets_for_new_deaths(&mut roster, &mut inventory, &mut fragments)
        .unwrap();
    assert_eq!(json!({"r":roster,"i":inventory,"f":fragments}), once);
    let copy = fragments.get(&a).unwrap();
    assert_eq!(copy.equipped_to, None);
    assert!(copy.unlocked_forms.contains("awakened"));
}
#[test]
fn r07_f12_cardinality_duplicate_keys_and_uuid_exhaustion_fail_before_mutation() {
    let (mut core, _, meta, _) = setup();
    let a = grant(&mut core, &meta);
    core.state.skill_fragment_policy.max_copies = 1;
    let before = roots(&core);
    assert!(core
        .apply_grant_effects(&[RewardEffect::GrantSkillFragment {
            fragment_id: meta.id.clone()
        }])
        .is_err());
    assert_eq!(roots(&core), before);
    let entry = serde_json::to_string(core.state.skill_fragments.get(&a).unwrap()).unwrap();
    let duplicated =
        format!("{{\"copies\":{{\"{a}\":{entry},\"{a}\":{entry}}},\"fragment_dust\":0}}");
    assert!(
        serde_json::from_str::<crate::game::skill_fragment::SkillFragmentInventory>(&duplicated)
            .is_err()
    );
    let mut manager: Value = serde_json::to_value(&core.state.uuid_manager).unwrap();
    manager["counters"][UuidManager::NS_OWNED_FRAGMENT.to_string()] = json!(u64::MAX);
    core.state.uuid_manager = serde_json::from_value(manager).unwrap();
    core.state.skill_fragment_policy.max_copies = 1024;
    let before = roots(&core);
    assert!(core
        .apply_grant_effects(&[RewardEffect::GrantSkillFragment {
            fragment_id: meta.id.clone()
        }])
        .is_err());
    assert_eq!(roots(&core), before);
}

#[test]
fn r07_f12_owned_shape_budgets_cancel_retry_and_max_copy_form_limits() {
    use crate::authoring::{
        clone_shape::controlled_clone, control, CancellationToken, ExecutionLimits, ExecutionPhase,
    };
    let (mut core, _, meta, _) = setup();
    for _ in 0..crate::game::skill_fragment::MAX_OWNED_FRAGMENTS {
        core.state
            .skill_fragments
            .grant(
                &meta,
                &mut core.state.uuid_manager,
                &core.state.skill_fragment_policy,
            )
            .unwrap();
    }
    let before = roots(&core);
    assert!(core
        .state
        .skill_fragments
        .grant(
            &meta,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy
        )
        .is_err());
    assert_eq!(roots(&core), before);
    let token = CancellationToken::default();
    let (copy, usage) = control::run(
        &ExecutionLimits::default(),
        &token,
        ExecutionPhase::TrialPreparation,
        || controlled_clone(&core.state.skill_fragments),
    )
    .unwrap();
    assert_eq!(copy.unwrap().copies().count(), 1024);
    for mode in 0..2 {
        let mut exact = ExecutionLimits::default();
        match mode {
            0 => exact.max_work = usage.work,
            _ => exact.max_clone_slots = usage.clone_slots_peak,
        };
        assert!(
            control::run(&exact, &token, ExecutionPhase::TrialPreparation, || {
                controlled_clone(&core.state.skill_fragments)
            })
            .unwrap()
            .0
            .is_ok()
        );
        match mode {
            0 => exact.max_work -= 1,
            _ => exact.max_clone_slots -= 1,
        };
        assert!(
            control::run(&exact, &token, ExecutionPhase::TrialPreparation, || {
                controlled_clone(&core.state.skill_fragments)
            })
            .is_err()
        );
        assert_eq!(roots(&core), before);
    }
    let cancel = CancellationToken::default();
    cancel.cancel();
    assert!(control::run(
        &ExecutionLimits::default(),
        &cancel,
        ExecutionPhase::TrialPreparation,
        || controlled_clone(&core.state.skill_fragments)
    )
    .is_err());
    assert!(control::run(
        &ExecutionLimits::default(),
        &token,
        ExecutionPhase::TrialPreparation,
        || controlled_clone(&core.state.skill_fragments)
    )
    .unwrap()
    .0
    .is_ok());
    let mut forms = std::collections::BTreeMap::new();
    for index in 0..16 {
        forms.insert(
            if index == 0 {
                "base".into()
            } else {
                format!("form_{index}")
            },
            SkillFragmentForm {
                resonance_max: 100,
                skill_id: SkillId::from("r07_skill"),
                compatibility: None,
                bloom_recipe: None,
            },
        );
    }
    let effect = SkillFragmentEffectDef::ActiveSkill {
        forms: forms.clone(),
        passive_guard: None,
        dismantle_recipe: None,
    };
    assert!(serde_json::from_value::<SkillFragmentEffectDef>(
        serde_json::to_value(effect).unwrap()
    )
    .is_ok());
    forms.insert("overflow".into(), forms["base"].clone());
    assert!(serde_json::from_value::<SkillFragmentEffectDef>(
        serde_json::to_value(SkillFragmentEffectDef::ActiveSkill {
            forms,
            passive_guard: None,
            dismantle_recipe: None
        })
        .unwrap()
    )
    .is_err());
    let duplicate = r#"ActiveSkill(forms:{"base":(skill_id:"r07_skill", resonance_max: 100),"base":(skill_id:"r07_skill", resonance_max: 100)})"#;
    assert!(ron::de::from_str::<SkillFragmentEffectDef>(duplicate)
        .unwrap_err()
        .to_string()
        .contains("duplicate"));
}

#[test]
fn r07_f03_f08_form_compatibility_transfer_fingerprint_and_retained_source() {
    use crate::game::data::skill_fragment_data::SkillFragmentCompatibilityRequirements;
    let mut meta = active_skill_fragment("r07_forms", 0x708, "r07_skill");
    if let SkillFragmentEffectDef::ActiveSkill { forms, .. } = &mut meta.effect {
        forms.insert(
            "gun".into(),
            SkillFragmentForm {
                resonance_max: 100,
                skill_id: SkillId::from("r07_skill"),
                compatibility: Some(SkillFragmentCompatibilityRequirements {
                    allowed_weapon_archetypes: vec![WeaponArchetype::Gun],
                    ..Default::default()
                }),
                bloom_recipe: Some(SkillFragmentRecipe {
                    enkephalin: 1,
                    materials: vec![],
                }),
            },
        );
    }
    let sword = weapon_equipment(0x710, "r07_sword", WeaponArchetype::Sword);
    let gun = weapon_equipment(0x711, "r07_gun", WeaponArchetype::Gun);
    let data = game_data_with_equipment_and_skill_fragments(
        vec![sword.clone(), gun.clone()],
        vec![meta.clone()],
    );
    let mut core = GameCore::new(data, 707);
    let player = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player);
    let owners = core.roster().unwrap().combat_eligible_employee_ids();
    grant_and_equip_weapon(&mut core, owners[0], gun, Uuid::from_u128(0x712));
    grant_and_equip_weapon(&mut core, owners[1], sword, Uuid::from_u128(0x713));
    let copy = grant(&mut core, &meta);
    core.execute(
        player,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: owners[0],
            copy_uuid: copy,
        },
    )
    .unwrap();
    let profile = crate::game::combat_setup::player_spawns::certified_combat_profile_for_employee(
        core.state.roster.get(&owners[0]).unwrap(),
        &core.state.inventory,
        &core.state.skill_fragments,
        &core.game_data,
    )
    .unwrap();
    let retained = std::sync::Arc::new(profile.fragment_source.unwrap());
    let fingerprint = core.certified_input_fingerprint().unwrap();
    maintenance(&mut core, player);
    core.execute(
        player,
        PlayerBehavior::AwakenSkillFragment {
            copy_uuid: copy,
            form_id: "gun".into(),
        },
    )
    .unwrap();
    core.execute(
        player,
        PlayerBehavior::SelectSkillFragmentForm {
            copy_uuid: copy,
            form_id: "gun".into(),
        },
    )
    .unwrap();
    assert_ne!(fingerprint, core.certified_input_fingerprint().unwrap());
    let before = roots(&core);
    assert!(core
        .execute(
            player,
            PlayerBehavior::TransferSkillFragment {
                copy_uuid: copy,
                target_employee_uuid: owners[1]
            }
        )
        .is_err());
    assert_eq!(roots(&core), before);
    core.execute(
        player,
        PlayerBehavior::SelectSkillFragmentForm {
            copy_uuid: copy,
            form_id: "base".into(),
        },
    )
    .unwrap();
    let fingerprint = core.certified_input_fingerprint().unwrap();
    core.execute(
        player,
        PlayerBehavior::TransferSkillFragment {
            copy_uuid: copy,
            target_employee_uuid: owners[1],
        },
    )
    .unwrap();
    assert_ne!(fingerprint, core.certified_input_fingerprint().unwrap());
    assert_eq!(retained.copy_uuid, copy);
    assert_eq!(retained.form_id, "base");
    let before = roots(&core);
    assert!(core
        .execute(
            player,
            PlayerBehavior::SelectSkillFragmentForm {
                copy_uuid: copy,
                form_id: "gun".into()
            }
        )
        .is_err());
    assert_eq!(roots(&core), before);
}

#[test]
fn r07_f12_source_payload_native_retained_exact_short_and_retry() {
    use crate::game::battle::{
        runtime_control::{self, RuntimeFactLimits},
        runtime_facts::RuntimeFactSource,
    };
    use crate::game::skill_fragment::FragmentSourceSnapshot;
    let source = FragmentSourceSnapshot {
        copy_uuid: Uuid::from_u128(7),
        definition_id: SkillFragmentId::from("fragment"),
        form_id: "form".into(),
        skill_id: SkillId::from("skill"),
    };
    let extent = source.retained_extent().unwrap() + 64;
    let exact = RuntimeFactLimits {
        max_retained_bytes: extent,
        ..Default::default()
    };
    let short = RuntimeFactLimits {
        max_retained_bytes: extent - 1,
        ..Default::default()
    };
    assert!(runtime_control::run(short, || source.clone().into_shared()).is_err());
    let shared = runtime_control::run(exact, || source.clone().into_shared()).unwrap();
    let fact = RuntimeFactSource {
        fragment: Some(shared),
        ..Default::default()
    };
    let extent = fact.retained_extent().unwrap();
    assert!(extent > source.retained_extent().unwrap());
    assert!(runtime_control::run(
        RuntimeFactLimits {
            max_retained_bytes: extent - 1,
            ..Default::default()
        },
        || fact.admit_owner("r07_source_boundary")
    )
    .is_err());
    runtime_control::run(
        RuntimeFactLimits {
            max_retained_bytes: extent,
            ..Default::default()
        },
        || fact.admit_owner("r07_source_boundary"),
    )
    .unwrap();
}

#[test]
fn r07_f08_run_owner_follows_copy_through_transfer_and_atomic_dismantle() {
    use crate::game::resource::store::ResourceOwner;
    use crate::game::resource::{
        ResourceCatalog, ResourceDefinition, ResourceId, ResourceLifetime, ResourceOwnerKind,
        RunResourceEntry,
    };
    let meta = active_skill_fragment("r07_owner", 0x709, "r07_owner_skill");
    let base = game_data_with_skill_fragments(vec![meta.clone()]);
    let id = ResourceId::new("copy_run".into()).unwrap();
    let data = test_game_data_builder()
        .with_skill_data(base.skill_data.clone())
        .with_skill_fragment_data(base.skill_fragment_data.clone())
        .with_resources(ResourceCatalog {
            resources: vec![ResourceDefinition {
                id: id.clone(),
                owner: ResourceOwnerKind::FragmentCopy,
                lifetime: ResourceLifetime::Run,
                initial: 7,
                cap: 10,
            }],
        })
        .build_arc();
    let mut core = GameCore::new(data, 707);
    let player = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player);
    let owners = core.roster().unwrap().combat_eligible_employee_ids();
    let a = grant(&mut core, &meta);
    let b = grant(&mut core, &meta);
    let run = &mut core.state.run.as_mut().unwrap().typed_resources;
    run.revision = 1;
    run.entries = vec![
        RunResourceEntry {
            resource: id.clone(),
            owner: ResourceOwner::FragmentCopy(a),
            balance: 5,
            revision: 1,
        },
        RunResourceEntry {
            resource: id,
            owner: ResourceOwner::FragmentCopy(b),
            balance: 4,
            revision: 1,
        },
    ];
    run.entries.sort_by_key(|e| e.owner);
    core.execute(
        player,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: owners[0],
            copy_uuid: a,
        },
    )
    .unwrap();
    let before = core.run_resources().unwrap().clone();
    core.execute(
        player,
        PlayerBehavior::TransferSkillFragment {
            copy_uuid: a,
            target_employee_uuid: owners[1],
        },
    )
    .unwrap();
    assert_eq!(core.run_resources(), Some(&before));
    maintenance(&mut core, player);
    core.state.enkephalin.amount = u32::MAX;
    let roots_before = roots(&core);
    assert!(core
        .execute(
            player,
            PlayerBehavior::DismantleSkillFragment {
                copy_uuid: a,
                confirmed: true
            }
        )
        .is_err());
    assert_eq!(roots(&core), roots_before);
    core.state.enkephalin.amount = 0;
    core.execute(
        player,
        PlayerBehavior::DismantleSkillFragment {
            copy_uuid: a,
            confirmed: true,
        },
    )
    .unwrap();
    let entries = &core.run_resources().unwrap().entries;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].owner, ResourceOwner::FragmentCopy(b));
    assert_eq!(entries[0].balance, 4);
}

#[test]
fn r09_completed_node_collapse_recovers_assets_and_bypasses_death_prevent_once() {
    let (mut core, player, meta, owners) = setup();
    let copy = grant(&mut core, &meta);
    core.execute(
        player,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: owners[0],
            copy_uuid: copy,
        },
    )
    .unwrap();
    let prevent = consumable_meta(
        0x9099,
        "r09_prevent",
        ConsumableTier::Critical,
        ConsumableEffect::DeathPrevent,
    );
    let employee = core.state.roster.get_mut(&owners[0]).unwrap();
    employee.apply_consumable_modifier(Uuid::from_u128(0x90999), &prevent);
    employee.condition.trauma = 100;
    employee.breakdown = crate::game::employee::BreakdownCounter::Active {
        remaining_nodes: 1,
        entered_node: None,
    };
    assert!(!employee.loadout.item_slot.is_empty());
    maintenance(&mut core, player);
    core.execute(player, PlayerBehavior::CompleteNode).unwrap();
    let employee = core.state.roster.get(&owners[0]).unwrap();
    assert!(!employee.is_alive());
    assert!(employee.active_consumable_modifier.is_none());
    assert!(employee.skill_fragments.active_copy_uuid().is_none());
    assert!(employee.loadout.item_slot.is_empty());
    assert_eq!(
        core.state.skill_fragments.get(&copy).unwrap().equipped_to,
        None
    );
    let after = roots(&core);
    assert!(core.execute(player, PlayerBehavior::CompleteNode).is_err());
    assert_eq!(roots(&core), after);
}

#[test]
fn r09_savepoint_recovery_precedes_counter_expiry() {
    let (mut core, player, _, owners) = setup();
    let employee = core.state.roster.get_mut(&owners[0]).unwrap();
    employee.condition.trauma = 100;
    employee.breakdown = crate::game::employee::BreakdownCounter::Active {
        remaining_nodes: 1,
        entered_node: None,
    };
    let node = force_first_available_node(
        &mut core,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
    );
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    core.execute(player, PlayerBehavior::CompleteNode).unwrap();
    let employee = core.state.roster.get(&owners[0]).unwrap();
    assert!(employee.is_alive());
    assert!(employee.condition.trauma < 100);
    assert_eq!(
        employee.breakdown,
        crate::game::employee::BreakdownCounter::Inactive
    );
    assert!(core.export_active_run_checkpoint().unwrap().is_some());
}
