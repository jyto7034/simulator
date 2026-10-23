use super::*;

fn seeded_vault() -> ExpeditionVault {
    ExpeditionVault { materials: Default::default(),
        equipment: BTreeMap::from([
            (Uuid::from_u128(12301), equipment(12301, "standard_training_sword")),
            (Uuid::from_u128(12302), equipment(12302, "standard_training_sword")),
            (Uuid::from_u128(12303), equipment(12303, "dormant_missing_weapon")),
        ]),
        fragments: BTreeMap::new(),
    }
}

#[test]
fn u12_lifecycle_zero_one_two_loans_move_exact_copies_and_abandon_loses_only_active_assets() {
    for count in 0..=2 {
        let player = Uuid::from_u128(12310 + count as u128);
        let mut core = GameCore::new(empty_game_data(), 12310);
        core.state.player_profile.lifecycle.vault = seeded_vault();
        let (mut core, ids) = selecting_core(core, player);
        let loans: Vec<_> = [Uuid::from_u128(12301), Uuid::from_u128(12302)]
            .into_iter().take(count).collect();
        core_command(&mut core, player, json!({"type":"select_starter_employees",
            "employee_definition_ids":ids,"loan_copy_ids":loans})).unwrap();
        assert_eq!(core.state.player_profile.lifecycle.vault.equipment.len(), 3 - count);
        for loan in &loans {
            assert!(!core.state.player_profile.lifecycle.vault.equipment.contains_key(loan));
            let active = core.state.inventory.equipments.get_item(loan).expect("loan in active inventory");
            assert_eq!(active.instance_uuid, *loan);
            assert_eq!(active.meta.id, "standard_training_sword");
        }
        core.state.player_profile.lifecycle.vault.validate_active_ownership(
            &core.state.inventory, &core.state.skill_fragments).unwrap();
        let dormant = core.state.player_profile.lifecycle.vault.clone();
        core_command(&mut core, player, json!({"type":"abandon_run"})).unwrap();
        assert_eq!(core.state.player_profile.lifecycle.vault, dormant);
        assert!(core.state.player_profile.lifecycle.active_expedition.is_none());
        assert_eq!(core.state.player_profile.lifecycle.first_return_story, FirstReturnStory::Never);
        assert!(core.export_active_run_checkpoint().unwrap().is_none());
        assert_eq!(core.state.roster.len(), 0, "all expedition employees terminate");
    }
}

#[test]
fn u12_lifecycle_invalid_loan_input_preserves_uuid_wallet_queue_profile_and_continue() {
    let player = Uuid::from_u128(12321);
    let mut core = GameCore::new(empty_game_data(), 12321);
    core.state.player_profile.lifecycle.vault = seeded_vault();
    let (mut core, ids) = selecting_core(core, player);
    for loans in [
        json!([Uuid::from_u128(12301),Uuid::from_u128(12301)]),
        json!([Uuid::from_u128(12301),Uuid::from_u128(12302),Uuid::from_u128(12303)]),
        json!([Uuid::from_u128(12303)]),
        json!([Uuid::from_u128(12399)]),
    ] {
        let before = core_roots(&core);
        assert!(core_command(&mut core, player, json!({"type":"select_starter_employees",
            "employee_definition_ids":ids,"loan_copy_ids":loans})).is_err());
        assert_eq!(core_roots(&core), before, "one rejected loan cannot consume earlier valid copies or UUIDs");
    }
    core_command(&mut core, player, json!({"type":"select_starter_employees",
        "employee_definition_ids":ids,"loan_copy_ids":[Uuid::from_u128(12301),Uuid::from_u128(12302)]})).unwrap();
    assert_eq!(core.state.player_profile.lifecycle.vault.equipment.len(), 1);
}

#[test]
fn u12_lifecycle_vault_active_union_rejects_same_uuid_even_across_copy_kinds() {
    let mut core = GameCore::new(empty_game_data(), 12331);
    start_new_game_with_default_starters(&mut core, Uuid::from_u128(12331));
    let active = core.state.inventory.equipments.iter().next().unwrap().instance_uuid;
    let mut vault = ExpeditionVault::default();
    vault.validate_active_ownership(&core.state.inventory, &core.state.skill_fragments).unwrap();
    vault.equipment.insert(active, equipment(active.as_u128(), "dormant_weapon"));
    assert!(vault.validate_active_ownership(&core.state.inventory, &core.state.skill_fragments).is_err());
    vault.equipment.clear();
    vault.fragments.insert(active, fragment(active.as_u128(), "dormant_fragment"));
    assert!(vault.validate_active_ownership(&core.state.inventory, &core.state.skill_fragments).is_err());
}

#[test]
fn u12_lifecycle_fragment_pair_and_mixed_loans_keep_exact_forms_through_return() {
    let mut metadata = super::super::checkpoint_validation::active_fragment_metadata();
    if let SkillFragmentEffectDef::ActiveSkill { forms, .. } = &mut metadata.effect {
        forms.insert("awakened".into(), forms["base"].clone());
    } else { panic!("active fixture"); }
    for mixed in [false, true] {
        let player = Uuid::from_u128(12341 + u128::from(mixed));
        let mut core = GameCore::new(game_data_with_skill_fragments(vec![metadata.clone()]), 12341);
        let first = fragment(12342, metadata.id.as_str());
        let second = fragment(12343, metadata.id.as_str());
        core.state.player_profile.lifecycle.vault.fragments.insert(first.copy_uuid, first.clone());
        core.state.player_profile.lifecycle.vault.fragments.insert(second.copy_uuid, second.clone());
        core.state.player_profile.lifecycle.vault.equipment.insert(Uuid::from_u128(12344), equipment(12344, "standard_training_sword"));
        let loans = if mixed { vec![first.copy_uuid, Uuid::from_u128(12344)] }
            else { vec![first.copy_uuid, second.copy_uuid] };
        let (mut core, ids) = selecting_core(core, player);
        core_command(&mut core, player, json!({"type":"select_starter_employees",
            "employee_definition_ids":ids,"loan_copy_ids":loans})).unwrap();
        let carried = core.state.skill_fragments.get(&first.copy_uuid).unwrap();
        assert_eq!(carried.unlocked_forms, first.unlocked_forms);
        assert_eq!(carried.selected_form, "awakened");
        assert_eq!(carried.equipped_to, None);
        assert_eq!(core.state.player_profile.lifecycle.vault.fragments.contains_key(&second.copy_uuid), mixed);
        core.state.run.as_mut().unwrap().run_progression.mode_state =
            crate::game::map::RunProgressionModeState::Expedition { floor_index: 2 };
        let gate = force_first_available_node(&mut core, MapNodeCategory::Gate, "gate_stairs", MapNodePayload::None);
        select_and_confirm_map_node(&mut core, player, gate);
        core_command(&mut core, player, json!({"type":"choose_expedition_continuation","choice":"Return"})).unwrap();
        assert_eq!(core.state.player_profile.lifecycle.vault.fragments.get(&first.copy_uuid), Some(&first));
        assert_eq!(core.state.player_profile.lifecycle.vault.fragments.get(&second.copy_uuid), Some(&second));
        assert_eq!(core.state.skill_fragments.copies().count(), 0);
    }
}

#[test]
fn u12_lifecycle_reward_fragment_grant_cannot_duplicate_a_dormant_copy_owner() {
    let metadata = super::super::checkpoint_validation::active_fragment_metadata();
    let fragments = game_data_with_skill_fragments(vec![metadata.clone()]);
    let reward_id = Uuid::from_u128(12361);
    let data = test_game_data_builder()
        .with_skill_data(fragments.skill_data.clone())
        .with_skill_fragment_data(fragments.skill_fragment_data.clone())
        .with_rewards(RewardDatabase::new_with_pools(vec![RewardMetadata {
            id: "u12_fragment_collision_reward".into(), uuid: reward_id,
            name: "Fragment reward".into(), description: "Independent collision fixture".into(), icon: "test".into(),
            effects: vec![RewardEffect::GrantEnkephalin { amount: 7 },
                RewardEffect::GrantSkillFragment { fragment_id: metadata.id.clone() }],
        }], vec![RewardPoolMetadata { id: "u12_fragment_collision_pool".into(),
            reward_ids: vec!["u12_fragment_collision_reward".into()] }]))
        .build_arc();
    for cross_kind in [false, true] {
        let player = Uuid::from_u128(12362);
        let mut core = GameCore::new(data.clone(), 12362);
        start_new_game_with_default_starters(&mut core, player);
        let node = force_first_available_node(&mut core, MapNodeCategory::Reward, "reward_treasure",
            MapNodePayload::Reward { reward_pool_id: Some("u12_fragment_collision_pool".into()) });
        let entered = select_and_confirm_map_node(&mut core, player, node);
        assert!(matches!(entered, BehaviorResult::RewardState { rewards, .. } if rewards.len() == 1 && rewards[0].uuid == reward_id));
        let collision = core.state.uuid_manager.clone().next_owned_fragment().unwrap();
        if cross_kind {
            core.state.player_profile.lifecycle.vault.equipment.insert(collision,
                equipment(collision.as_u128(), "dormant_reward_weapon"));
        } else {
            core.state.player_profile.lifecycle.vault.fragments.insert(collision,
                fragment(collision.as_u128(), "dormant_reward_fragment"));
        }
        core.state.player_profile.validate_persisted().unwrap();
        core.state.player_profile.lifecycle.vault.validate_active_ownership(
            &core.state.inventory, &core.state.skill_fragments).unwrap();
        let before = core_roots(&core);
        assert!(core_command(&mut core, player, json!({"type":"claim_reward"})).is_err());
        assert_eq!(core_roots(&core), before, "earlier currency grant, UUID and staged reward remain unchanged");
        core.state.player_profile.lifecycle.vault.equipment.remove(&collision);
        core.state.player_profile.lifecycle.vault.fragments.remove(&collision);
        let wallet = core.state.enkephalin.amount;
        core_command(&mut core, player, json!({"type":"claim_reward"})).unwrap();
        assert_eq!(core.state.enkephalin.amount, wallet + 7);
        let granted = core.state.skill_fragments.get(&collision).expect("the original unconsumed fragment UUID is granted on retry");
        assert_eq!(granted.definition_id, metadata.id);
    }
}
