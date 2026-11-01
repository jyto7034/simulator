//! Independent recovery owner tests. Explicit I fixtures below are not claims
//! that the public lifecycle driver won a battle or traversed these floors.
use super::*;
use crate::game::battle::result_stats::collect_battle_result_stats;
use crate::game::battle::types::{BattleOutcomeId, BattleVerdict, BattleWinner};
use crate::game::data::skill_fragment_data::SkillFragmentCompatibilityRequirements;
use crate::game::employee::{Employee, EmployeeRoster};
use crate::game::expedition_lifecycle::ExpeditionContinuationChoice;
use crate::game::resources::{CombatBattleState, RunFailureReason};
use crate::game::world::state::CombatRecoveryRestriction;

fn treatment() -> ConsumableMetadata {
    consumable_meta(12701, "u12_recovery_treatment", ConsumableTier::Common,
        ConsumableEffect::ReducePhysicalDamage { amount: 1 })
}

fn stimulant() -> ConsumableMetadata {
    consumable_meta(12702, "u12_recovery_stimulant", ConsumableTier::Rare,
        ConsumableEffect::OffenseBoost { attack_bonus_percent: 10 })
}

fn condition(employee: &mut Employee, physical: u8, trauma: u32, breakdown_limit: u32) {
    employee.condition = EmployeeCondition::new(physical, trauma);
    employee.reconcile_breakdown(breakdown_limit, None);
}

fn publish_fixture(core: &mut GameCore) {
    core.refresh_continue_after_operation(crate::game::world::application::ApplicationRoute::Behavior(
        ActionKind::CompleteNode)).unwrap();
}

fn resume_equal(core: &GameCore, player: Uuid) -> GameCore {
    let saved = core.export_active_run_checkpoint().unwrap().expect("current active continuation");
    let mut restored = core_with_profile_for_continue(core, 12799);
    restored.continue_from_active_run_checkpoint(player, &saved.core_payload_json,
        saved.loads_used, saved.run_peak_certified_power).expect("unmodified current checkpoint is a positive input");
    assert_eq!(core_roots(&restored), core_roots(core));
    let again = restored.export_active_run_checkpoint().unwrap().unwrap();
    assert_eq!(again.core_payload_json, saved.core_payload_json);
    assert_eq!(again.loads_used, saved.loads_used);
    assert_eq!(again.run_peak_certified_power, saved.run_peak_certified_power);
    restored
}

fn assert_unfinished_hq_rolls_back(
    core: &GameCore,
    player: Uuid,
    roster_before_entry: &Value,
    inventory_before_entry: &Value,
) {
    let saved = core.export_active_run_checkpoint().unwrap().unwrap();
    let mut restored = core_with_profile_for_continue(core, 12798);
    restored.continue_from_active_run_checkpoint(player, &saved.core_payload_json,
        saved.loads_used, saved.run_peak_certified_power).unwrap();
    assert!(matches!(restored.get_state(), GameState::ViewingMap));
    assert_eq!(restored.state.enkephalin.amount, 100, "unfinished HQ returns to its paid-entry wallet");
    assert_eq!(&serde_json::to_value(&restored.state.roster).unwrap(), roster_before_entry);
    assert_eq!(&serde_json::to_value(&restored.state.inventory).unwrap(), inventory_before_entry);
    assert_eq!(restored.state.player_profile, core.state.player_profile);
    assert_eq!(restored.state.run_checkpoint.loads_used, saved.loads_used);
}

fn reject(core: &mut GameCore, player: Uuid, behavior: PlayerBehavior) {
    let before = core_roots(core);
    assert!(core.execute(player, behavior).is_err(), "restricted payload must reject");
    assert_eq!(core_roots(core), before, "rejection preserves all roots and exact Continue bytes");
}

fn restriction(core: &GameCore, expected: &[MapNodeId]) {
    let run = core.get_run_snapshot_json().unwrap();
    let restricted = &run["combat_recovery_restriction"];
    assert_eq!(restricted["reason"], json!(RunFailureReason::NoDeployableEmployees));
    let actual: BTreeSet<MapNodeId> = serde_json::from_value(restricted["allowed_node_ids"].clone()).unwrap();
    assert_eq!(actual, expected.iter().copied().collect());
    let selectable: BTreeSet<MapNodeId> = serde_json::from_value(run["map_navigation"]["selectable_node_ids"].clone()).unwrap();
    assert_eq!(selectable, expected.iter().copied().collect());
    assert!(!core.state.roster.iter().any(Employee::is_battle_deployable));
}

fn unequip_all_weapons(core: &mut GameCore, player: Uuid) -> Vec<Uuid> {
    let equipped: Vec<_> = core.state.inventory.equipments.iter()
        .filter(|copy| copy.meta.equipment_type == EquipmentType::Weapon)
        .filter_map(|copy| copy.equipped_to.map(|employee| (copy.instance_uuid, employee)))
        .collect();
    for (item_uuid, target_unit) in &equipped {
        core.execute(player, PlayerBehavior::UnEquipItem { item_uuid: *item_uuid, target_unit: *target_unit }).unwrap();
    }
    equipped.into_iter().map(|(id, _)| id).collect()
}

fn compound_fixture() -> (GameCore, Uuid, Uuid, Uuid) {
    let player = Uuid::from_u128(12710);
    let mut core = GameCore::new(game_data_with_consumables(vec![treatment(), stimulant()]), 12710);
    start_new_game_with_default_starters(&mut core, player);
    let target = core.state.roster.iter().next().unwrap().uuid;
    let weapon = unequip_all_weapons(&mut core, player)[0];
    core.state.inventory.add_consumable(Arc::new(treatment()), 1).unwrap();
    core.state.inventory.add_consumable(Arc::new(stimulant()), 1).unwrap();
    // I: only the first employee is alive, critically injured and weaponless.
    // Immediate HQ is unaffordable; recovery requires both owned payloads.
    let limit = core.run_policy().corrosion.breakdown_collapse_cost_limit;
    for employee in core.state.roster.iter_mut() {
        condition(employee, if employee.uuid == target { 5 } else { 6 }, 0, limit);
    }
    core.state.enkephalin.amount = 0;
    core.state.run.as_mut().unwrap().combat_recovery_restriction =
        Some(CombatRecoveryRestriction { allowed_node_ids: vec![] });
    publish_fixture(&mut core);
    (core, player, target, weapon)
}

#[test]
fn u12_lifecycle_recovery_compound_treatment_and_weapon_succeed_in_both_orders() {
    for treatment_first in [true, false] {
        let (source, player, target, weapon) = compound_fixture();
        let mut core = resume_equal(&source, player);
        restriction(&core, &[]);
        assert!(action_capability_enabled(&core, ActionKind::UseConsumableItem));
        assert!(action_capability_enabled(&core, ActionKind::EquipItem));
        let heal = PlayerBehavior::UseConsumableItem { item_uuid: treatment().uuid, target_employee_uuid: target };
        let equip = PlayerBehavior::EquipItem { item_uuid: weapon, target_unit: target };
        let (first, second) = if treatment_first { (heal, equip) } else { (equip, heal) };
        core.execute(player, first).unwrap();
        restriction(&core, &[]);
        assert_eq!(core.state.roster.get(&target).unwrap().condition.physical_damage,
            if treatment_first { 4 } else { 5 });
        assert_eq!(core.state.roster.get(&target).unwrap().has_equipped_weapon(), !treatment_first);
        core = resume_equal(&core, player);
        core.execute(player, second).unwrap();
        let recovered = core.state.roster.get(&target).unwrap();
        assert_eq!(recovered.condition.physical_damage, 4);
        assert!(recovered.is_battle_deployable());
        assert_eq!(core.state.inventory.equipments.get_item(&weapon).unwrap().equipped_to, Some(target));
        assert!(core.state.inventory.consumables.get_item(&treatment().uuid).is_none());
        assert!(core.get_run_snapshot_json().unwrap()["combat_recovery_restriction"].is_null());
        assert_eq!(core.state.enkephalin.amount, 0);
        resume_equal(&core, player);
    }
}

#[test]
fn u12_lifecycle_recovery_active_fragment_removal_opens_the_owned_weapon_path() {
    for block_capacity_min in [None, Some(1)] {
    let mut fragment = super::super::checkpoint_validation::active_fragment_metadata();
    fragment.compatibility = SkillFragmentCompatibilityRequirements {
        allowed_weapon_archetypes: vec![WeaponArchetype::Gun], block_capacity_min, ..Default::default()
    };
    let mut gun = weapon_equipment(12721, "u12_recovery_gun", WeaponArchetype::Gun);
    gun.weapon_profile.as_mut().unwrap().block_capacity = 1;
    let data = game_data_with_equipment_and_skill_fragments(vec![gun.clone()], vec![fragment.clone()]);
    let player = Uuid::from_u128(12720);
    let mut core = GameCore::new(data, 12720);
    start_new_game_with_default_starters(&mut core, player);
    let target = core.state.roster.iter().next().unwrap().uuid;
    unequip_all_weapons(&mut core, player);
    let sword = core.state.inventory.equipments.iter().find(|copy|
        copy.meta.id == "standard_training_sword").unwrap().instance_uuid;
    let remove: Vec<_> = core.state.inventory.equipments.iter()
        .filter(|copy| copy.meta.equipment_type == EquipmentType::Weapon && copy.instance_uuid != sword)
        .map(|copy| copy.instance_uuid).collect();
    for id in remove { core.state.inventory.equipments.remove_item(id).unwrap(); }
    let gun_copy = Uuid::from_u128(12722);
    core.state.inventory.equipments.add_item(OwnedEquipment::new(gun_copy, Arc::new(gun))).unwrap();
    core.execute(player, PlayerBehavior::EquipItem { item_uuid: gun_copy, target_unit: target }).unwrap();
    let copy = core.state.skill_fragments.grant(&fragment, &mut core.state.uuid_manager,
        &core.state.skill_fragment_policy).unwrap();
    core.execute(player, PlayerBehavior::EquipSkillFragment { employee_uuid: target, copy_uuid: copy }).unwrap();
    resume_equal(&core, player);
    // A valid binding cannot excuse an incompatible equipped weapon on restore.
    let mut incompatible = resume_equal(&core, player);
    let plan = crate::game::resources::EquipmentLoadoutTransition::plan_equip(
        &incompatible.state.inventory, &incompatible.state.roster, sword, target).unwrap();
    crate::game::resources::EquipmentLoadoutTransition::new(
        &mut incompatible.state.inventory, &mut incompatible.state.roster).equip(plan).unwrap();
    publish_fixture(&mut incompatible);
    let invalid = incompatible.export_active_run_checkpoint().unwrap().unwrap();
    let mut destination = core_with_profile_for_continue(&core, 12723);
    let before = core_roots(&destination);
    let error = destination.continue_from_active_run_checkpoint(player, &invalid.core_payload_json,
        invalid.loads_used, invalid.run_peak_certified_power).unwrap_err();
    assert!(matches!(error, GameError::InvalidStaticData(message)
        if message.contains("fragment form is incompatible")));
    assert_eq!(core_roots(&destination), before);
    let valid = core.export_active_run_checkpoint().unwrap().unwrap();
    let mut bad_form: Value = serde_json::from_slice(&valid.core_payload_json).unwrap();
    bad_form["continuation"]["current"]["skill_fragments"]["copies"][copy.to_string()]["selected_form"] = json!("missing_form");
    refresh_active_run_checksum(&mut bad_form);
    let error = destination.continue_from_active_run_checkpoint(player, &serde_json::to_vec(&bad_form).unwrap(),
        valid.loads_used, valid.run_peak_certified_power).unwrap_err();
    assert!(matches!(error, GameError::InvalidStaticData(message)
        if message.contains("skill fragment graph is inconsistent")));
    assert_eq!(core_roots(&destination), before);
    core.execute(player, PlayerBehavior::UnEquipItem { item_uuid: gun_copy, target_unit: target }).unwrap();
    let roster = core.get_employee_roster_snapshot_json().unwrap();
    let employee = roster["employees"].as_array().unwrap().iter().find(|entry| entry["uuid"] == json!(target)).unwrap();
    let compatibility = employee["skill_fragments"]["compatibility"].as_array().unwrap().iter()
        .find(|entry| entry["copy_uuid"] == json!(copy)).unwrap();
    let expected_failures = if block_capacity_min.is_some() {
        json!(["weapon_required", "block_capacity_too_low"])
    } else { json!(["weapon_required"]) };
    assert_eq!(compatibility["failure_codes"], expected_failures);
    core = resume_equal(&core, player); // Ordinary public weaponless intermediate, before any I restriction.
    // I removes only the unused Gun; the valid active binding came from commands.
    core.state.inventory.equipments.remove_item(gun_copy).unwrap();
    let limit = core.run_policy().corrosion.breakdown_collapse_cost_limit;
    for employee in core.state.roster.iter_mut() {
        condition(employee, if employee.uuid == target { 0 } else { 6 }, 0, limit);
    }
    core.state.enkephalin.amount = 0;
    core.state.run.as_mut().unwrap().combat_recovery_restriction =
        Some(CombatRecoveryRestriction { allowed_node_ids: vec![] });
    publish_fixture(&mut core);
    restriction(&core, &[]);
    // A recovery state reached by public loadout commands must remain durable.
    core = resume_equal(&core, player);
    assert!(!action_capability_enabled(&core, ActionKind::EquipItem));
    assert!(action_capability_enabled(&core, ActionKind::UnequipSkillFragment));
    reject(&mut core, player, PlayerBehavior::EquipItem { item_uuid: sword, target_unit: target });
    let copies_before: BTreeSet<_> = core.state.skill_fragments.copies().map(|copy| copy.copy_uuid).collect();
    core.execute(player, PlayerBehavior::UnequipSkillFragment { employee_uuid: target, copy_uuid: copy }).unwrap();
    assert_eq!(core.state.roster.get(&target).unwrap().skill_fragments.active_copy_uuid(), None);
    assert_eq!(core.state.skill_fragments.get(&copy).unwrap().equipped_to, None);
    assert_eq!(core.state.skill_fragments.copies().map(|copy| copy.copy_uuid).collect::<BTreeSet<_>>(), copies_before);
    restriction(&core, &[]);
    core = resume_equal(&core, player);
    core.execute(player, PlayerBehavior::EquipItem { item_uuid: sword, target_unit: target }).unwrap();
    assert!(core.state.roster.get(&target).unwrap().is_battle_deployable());
    assert!(!core.has_combat_recovery_restriction());
    resume_equal(&core, player);
    }
}

#[test]
fn u12_lifecycle_recovery_rejects_unrelated_and_no_op_payloads_with_legal_alternatives() {
    for ineffective_physical in [0, 1] {
        let (mut core, player, target, weapon) = compound_fixture();
        let other = core.state.roster.iter().find(|employee| employee.uuid != target).unwrap().uuid;
        let limit = core.run_policy().corrosion.breakdown_collapse_cost_limit;
        condition(core.state.roster.get_mut(&other).unwrap(), ineffective_physical, 100, limit);
        let armor = Uuid::from_u128(12731);
        core.state.inventory.equipments.add_item(OwnedEquipment::new(armor, Arc::new(starter_equipment_meta()))).unwrap();
        publish_fixture(&mut core);
        core = resume_equal(&core, player);
        restriction(&core, &[]);
        assert!(action_capability_enabled(&core, ActionKind::UseConsumableItem));
        assert!(action_capability_enabled(&core, ActionKind::EquipItem));
        for behavior in [
            PlayerBehavior::UseConsumableItem { item_uuid: treatment().uuid, target_employee_uuid: other },
            PlayerBehavior::UseConsumableItem { item_uuid: stimulant().uuid, target_employee_uuid: target },
            PlayerBehavior::EquipItem { item_uuid: armor, target_unit: target },
            PlayerBehavior::EquipItem { item_uuid: weapon, target_unit: other },
            PlayerBehavior::UnEquipItem { item_uuid: weapon, target_unit: target },
        ] { reject(&mut core, player, behavior); }
        core.execute(player, PlayerBehavior::EquipItem { item_uuid: weapon, target_unit: target }).unwrap();
        restriction(&core, &[]);
        let alternate = core.state.inventory.equipments.iter().find(|copy|
            copy.meta.equipment_type == EquipmentType::Weapon && copy.instance_uuid != weapon).unwrap().instance_uuid;
        reject(&mut core, player, PlayerBehavior::EquipItem { item_uuid: alternate, target_unit: target });
        core.execute(player, PlayerBehavior::UseConsumableItem {
            item_uuid: treatment().uuid, target_employee_uuid: target }).unwrap();
        assert!(core.state.roster.get(&target).unwrap().is_battle_deployable());
        assert!(!core.has_combat_recovery_restriction());
    }
}

#[test]
fn u12_lifecycle_recovery_rejects_unrelated_active_fragment_removal_and_transfer() {
    let fragment = super::super::checkpoint_validation::active_fragment_metadata();
    let base = GameCore::new(game_data_with_skill_fragments(vec![fragment.clone()]), 12732);
    let data = GameDataBuilder::from_validated(&base.game_data).with_consumables(vec![treatment()]).build_arc();
    let player = Uuid::from_u128(12732);
    let mut core = GameCore::new(data, 12732);
    start_new_game_with_default_starters(&mut core, player);
    let ids: Vec<_> = core.state.roster.iter().map(|employee| employee.uuid).collect();
    let copy = core.state.skill_fragments.grant(&fragment, &mut core.state.uuid_manager,
        &core.state.skill_fragment_policy).unwrap();
    core.execute(player, PlayerBehavior::EquipSkillFragment { employee_uuid: ids[0], copy_uuid: copy }).unwrap();
    core.state.inventory.add_consumable(Arc::new(treatment()), 1).unwrap();
    let limit = core.run_policy().corrosion.breakdown_collapse_cost_limit;
    for employee in core.state.roster.iter_mut() { condition(employee, 5, 0, limit); }
    core.state.enkephalin.amount = 0;
    core.state.run.as_mut().unwrap().combat_recovery_restriction =
        Some(CombatRecoveryRestriction { allowed_node_ids: vec![] });
    publish_fixture(&mut core);
    core = resume_equal(&core, player);
    restriction(&core, &[]);
    assert!(action_capability_enabled(&core, ActionKind::UseConsumableItem));
    for action in [ActionKind::UnequipSkillFragment, ActionKind::EquipSkillFragment,
        ActionKind::TransferSkillFragment, ActionKind::DismantleSkillFragment,
        ActionKind::EquipRelic, ActionKind::CorruptRelic] {
        assert!(!action_capability_enabled(&core, action), "unrelated action {action:?}");
    }
    for behavior in [
        PlayerBehavior::UnequipSkillFragment { employee_uuid: ids[0], copy_uuid: copy },
        PlayerBehavior::TransferSkillFragment { copy_uuid: copy, target_employee_uuid: ids[1] },
        PlayerBehavior::DismantleSkillFragment { copy_uuid: copy, confirmed: true },
    ] { reject(&mut core, player, behavior); }
    core.execute(player, PlayerBehavior::UseConsumableItem {
        item_uuid: treatment().uuid, target_employee_uuid: ids[0] }).unwrap();
    assert!(core.state.roster.get(&ids[0]).unwrap().is_battle_deployable());
    assert_eq!(core.state.roster.get(&ids[0]).unwrap().skill_fragments.active_copy_uuid(), Some(copy));
    assert!(!core.has_combat_recovery_restriction());
}

// I: the authored branching template is retained. Completed transit rooms make
// HQ immediately reachable after this combat only when `immediate_hq` is true.
// The HQ itself remains uncompleted; a future or visited HQ is not a witness.
fn combat_map(core: &mut GameCore, immediate_hq: bool) -> (MapNodeId, MapNodeId, MapNodeId) {
    let id = |n| MapNodeId::new(Uuid::from_u128(127500 + n));
    let transit = if immediate_hq { MapNodeState::Completed } else { MapNodeState::Unavailable };
    let authored = vec![
        ("entry", "start", MapNodeCategory::Start, MapNodeState::Completed, MapNodePayload::None),
        ("dispatch_hall", "combat_defense", MapNodeCategory::Combat, MapNodeState::Available,
            MapNodePayload::Encounter { encounter_id: Some("defense_encounter".into()) }),
        ("hq_terminal", "headquarters_contact", MapNodeCategory::HeadquartersContact, MapNodeState::Unavailable,
            MapNodePayload::HeadquartersContact { shop_pool_id: None, candidate_count: 6 }),
        ("save_archive", "save_point_a", MapNodeCategory::SavePoint, MapNodeState::Unavailable, MapNodePayload::SavePoint),
        ("maintenance_bay", "maintenance", MapNodeCategory::Maintenance, transit, MapNodePayload::Maintenance),
        ("supply_depot", "shop", MapNodeCategory::Shop, transit,
            MapNodePayload::Shop { shop_id: None, shop_pool_id: None }),
        ("crossroads_archive", "event_story", MapNodeCategory::Event, transit, MapNodePayload::Event { event_id: None }),
        ("containment_guard", "combat_elite", MapNodeCategory::Combat, MapNodeState::Unavailable,
            MapNodePayload::Encounter { encounter_id: None }),
        ("floor_gate", "gate_stairs", MapNodeCategory::Gate, MapNodeState::Unavailable, MapNodePayload::None),
    ];
    let power_model_version = core.run_policy().combat_difficulty.power_model_version;
    let floor_power_anchor = core.state.run_power_authority.current_floor_power_anchor;
    let mut nodes: Vec<MapNode> = authored.into_iter().enumerate().map(|(index, (slot, kind, category, state, payload))| MapNode {
        id: id(index as u128), slot_id: MapSlotId::new(slot), kind_id: MapNodeKindId::new(kind), category, state,
        visibility: MapNodeVisibility::Revealed, payload, omen: None,
        combat_difficulty: (category == MapNodeCategory::Combat).then(|| crate::game::map::MapNodeCombatDifficulty {
            power_model_version, difficulty_profile_id: "default_facility".into(),
            class: if kind == "combat_elite" {
                crate::game::map::MapNodeCombatDifficultyClass::Role(
                    crate::game::data::run_policy_data::CombatNodeDifficultyRole::Elite)
            } else { crate::game::map::MapNodeCombatDifficultyClass::Ordinary(
                crate::game::data::run_policy_data::OrdinaryCombatDifficultyBand::Standard) },
            revival: None, floor_power_anchor, target_power: floor_power_anchor,
        }),
    }).collect();
    // I uses each declared class's actual tuning: Elite is not Standard's
    // 100% target. Checkpoint validation must accept this map before recovery.
    for node in &mut nodes {
        if let Some(difficulty) = node.combat_difficulty.as_mut() {
            let tuning = core.run_policy().resolved_tuning_for_manifested_node(difficulty)
                .expect("declared fixture class has current policy tuning");
            difficulty.target_power = u64::try_from(
                (u128::from(floor_power_anchor) * u128::from(tuning.target_power_percent) + 99) / 100)
                .expect("finite fixture target power");
        }
    }
    let template = core.game_data.map_catalogs.facility_templates
        .template(&MapTemplateId::new("standard_branching_a")).unwrap();
    // Start is a decorative completed room; navigation begins at its authored
    // outgoing slot. Derive the real entry IDs rather than equating these roles.
    let start_node_ids = template.navigation_start_slot_ids().iter().map(|slot|
        nodes.iter().find(|node| &node.slot_id == slot).expect("authored navigation slot").id).collect();
    let map = RunMap { map_template_id: template.id.clone(),
        facility_template_version: FACILITY_TEMPLATE_VERSION, nodes, start_node_ids,
        terminals: vec![MapTerminalNode { node_id: id(8), resolution: FacilityTerminalResolution::FloorGate }] };
    let navigation = crate::game::map::MapProgression { current_node_id: Some(id(0)), available_node_ids: vec![id(1)],
        completed_node_ids: if immediate_hq { vec![id(0), id(4), id(5), id(6)] } else { vec![id(0)] } };
    map.validate_facility_template_contract(template).expect("valid complete recovery fixture graph");
    assert!(navigation.validate_checkpoint_state(&map, template), "valid recovery fixture navigation");
    let previous = core.state.run.as_ref().unwrap();
    let mut progression = previous.run_progression.clone();
    progression.entry_receipts.clear();
    progression.pending_continuation = None;
    core.state.run = Some(RunState::new(map, navigation, progression).with_recruitment(previous.recruitment.clone()));
    (id(1), id(2), id(3))
}

// This explicit post-battle I enters the production settlement owner. It does
// not replace the separate public driver, which must win its own live battles.
fn settle_conditions(core: &mut GameCore, node: MapNodeId, physical: u8, trauma: u32, reward: Option<u32>) {
    let battle_uuid = core.state.active_battle.as_ref().expect("actual combat entry").battle_uuid;
    let winner = if reward.is_some() { BattleWinner::Player } else { BattleWinner::Opponent };
    let verdict = if reward.is_some() { BattleVerdict::Victory } else { BattleVerdict::Defeat };
    let mut employees: EmployeeRoster = core.state.roster.clone();
    let limit = core.run_policy().corrosion.breakdown_collapse_cost_limit;
    for employee in employees.iter_mut() {
        employee.condition = EmployeeCondition::new(physical, trauma);
        employee.reconcile_breakdown(limit, Some(node));
    }
    let event_log = BattleEventLog::default();
    let result_stats = collect_battle_result_stats(winner, &event_log, &[], &core.run_policy().battle_result_stats);
    let battle = CombatBattleState {
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None, runtime_facts: None, primary_abnormality_id: Some("defense_risk_abno".into()),
        encounter_id: "defense_encounter".into(), node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense, abnormality_uuid: node.0,
        winner, outcome_id: BattleOutcomeId::new(if reward.is_some() { "victory" } else { "defeat" }), verdict,
        grant_node_bonus: false, advance_encounter_progression: false, result_stats, event_log,
        reward_mode: RewardMode::ClaimAll,
        rewards: reward.filter(|amount| *amount > 0).map(|amount| vec![RewardOption {
            id: "u12_recovery_currency".into(), uuid: Uuid::from_u128(12751), name: "Recovery currency".into(),
            description: "Explicit settlement fixture".into(), icon: "test".into(),
            effects: vec![RewardEffect::GrantEnkephalin { amount }],
        }]).unwrap_or_default(),
        participant_results: vec![], employee_conditions: employees, compendium_candidates: vec![],
    };
    core.execute_application(crate::game::world::application::ApplicationRoute::BattleElapsedAdvance,
        move |draft| draft.transition_finished_live_battle_to_result(battle_uuid, battle)).unwrap();
    assert!(matches!(core.get_state(), GameState::CombatResult { .. }));
}

#[test]
fn u12_lifecycle_recovery_savepoint_uses_staged_conditions_and_real_completion() {
    for (physical, trauma, recoverable) in [(0, 100, true), (5, 0, false)] {
        let player = Uuid::from_u128(12760);
        let mut core = GameCore::new(game_data_with_pve_encounters(), 12760);
        start_new_game_with_default_starters(&mut core, player);
        core.state.enkephalin.amount = 0;
        let (combat, hq, save) = combat_map(&mut core, false);
        assert!(!core.selectable_map_node_ids().contains(&save));
        select_and_confirm_map_node(&mut core, player, combat);
        settle_conditions(&mut core, combat, physical, trauma, None);
        core.execute(player, PlayerBehavior::CompleteCombatResult).unwrap();
        if !recoverable {
            assert!(matches!(core.get_state(), GameState::RunFailed { reason: RunFailureReason::NoDeployableEmployees }));
            assert!(core.export_active_run_checkpoint().unwrap().is_none());
            continue;
        }
        restriction(&core, &[save]);
        assert!(!core.selectable_map_node_ids().contains(&hq), "future HQ is not immediate recovery");
        core = resume_equal(&core, player);
        select_and_confirm_map_node(&mut core, player, save);
        assert!(core.state.roster.iter().all(|employee| employee.condition.trauma == 100));
        assert!(core.state.run_checkpoint.payload.is_none());
        core.execute(player, PlayerBehavior::CompleteNode).unwrap();
        assert!(core.state.roster.iter().all(|employee| employee.condition.trauma == 85 && employee.is_battle_deployable()));
        assert!(!core.has_combat_recovery_restriction());
        assert!(core.state.run_checkpoint.payload.is_some());
        core = resume_equal(&core, player);
        core.execute(player, PlayerBehavior::LoadRunCheckpoint).unwrap();
        assert!(core.state.roster.iter().all(|employee| employee.condition.trauma == 85));
        assert_eq!(core.state.run_checkpoint.loads_used, 1);
    }
}

#[test]
fn u12_lifecycle_recovery_hq_uses_reward_wallet_99_to_100_before_actual_paid_hire() {
    for reward in [0, 1] {
        let player = Uuid::from_u128(12770);
        let mut core = GameCore::new(game_data_with_pve_encounters(), 12770);
        start_new_game_with_default_starters(&mut core, player);
        core.state.enkephalin.amount = 99;
        let (combat, hq, _) = combat_map(&mut core, true);
        assert!(!core.selectable_map_node_ids().contains(&hq));
        select_and_confirm_map_node(&mut core, player, combat);
        settle_conditions(&mut core, combat, 5, 0, Some(reward));
        assert_eq!(core.state.enkephalin.amount, 99 + reward);
        core.execute(player, PlayerBehavior::CompleteCombatResult).unwrap();
        if reward == 0 {
            assert!(matches!(core.get_state(), GameState::RunFailed { reason: RunFailureReason::NoDeployableEmployees }));
            assert!(core.state.player_profile.lifecycle.active_expedition.is_none());
            continue;
        }
        restriction(&core, &[hq]);
        let profile = core.state.player_profile.clone();
        let old_savepoint = serde_json::to_value(&core.state.run_checkpoint.payload).unwrap();
        core = resume_equal(&core, player);
        let roster_before_entry = serde_json::to_value(&core.state.roster).unwrap();
        let inventory_before_entry = serde_json::to_value(&core.state.inventory).unwrap();
        let entered = select_and_confirm_map_node(&mut core, player, hq);
        let BehaviorResult::HeadquartersContactState { recruitment_candidates, .. } = entered else { panic!("real HQ handler"); };
        let selected = recruitment_candidates.first().expect("priced current candidate").employee_definition_id.clone();
        let ids: BTreeSet<_> = core.state.roster.iter().map(|employee| employee.uuid).collect();
        core.execute(player, PlayerBehavior::RecruitEmployee { employee_definition_id: selected.clone() }).unwrap();
        assert_eq!(core.state.enkephalin.amount, 0);
        let new: Vec<_> = core.state.roster.iter().filter(|employee| !ids.contains(&employee.uuid)).collect();
        assert_eq!(new.len(), 1);
        let hired_uuid = new[0].uuid;
        assert_eq!(new[0].employee_definition_id, selected);
        assert!(new[0].is_battle_deployable());
        assert!(!core.has_combat_recovery_restriction());
        assert_eq!(core.state.player_profile, profile);
        assert_unfinished_hq_rolls_back(&core, player, &roster_before_entry, &inventory_before_entry);
        let emergency_enkephalin = core.run_policy().headquarters.emergency_enkephalin;
        let granted = core.execute(player, PlayerBehavior::RequestEmergencySupplies).unwrap();
        let BehaviorResult::EmergencySuppliesGranted { enkephalin, completion, .. } = granted else {
            panic!("actual HQ emergency supplies completion");
        };
        assert_eq!(enkephalin, emergency_enkephalin);
        assert!(matches!(*completion, BehaviorResult::NodeCompleted { .. }));
        assert!(matches!(core.get_state(), GameState::ViewingMap));
        assert_eq!(core.state.enkephalin.amount, emergency_enkephalin);
        assert_eq!(core.state.roster.len(), ids.len() + 1);
        let hired = core.state.roster.iter().find(|employee| employee.uuid == hired_uuid).unwrap();
        assert_eq!(hired.employee_definition_id, selected);
        assert!(hired.is_battle_deployable());
        assert_eq!(serde_json::to_value(&core.state.run_checkpoint.payload).unwrap(), old_savepoint);
        assert_eq!(core.state.player_profile, profile);
        resume_equal(&core, player);
    }
}

#[test]
fn u12_lifecycle_recovery_all_dead_and_no_deployable_obey_checkpoint_remaining_and_exhausted() {
    for (physical, reason) in [(6, RunFailureReason::NoLivingEmployees), (5, RunFailureReason::NoDeployableEmployees)] {
        for used in [None, Some(2), Some(3)] {
            let player = Uuid::from_u128(12780);
            let mut core = GameCore::new(game_data_with_pve_encounters(), 12780);
            core.state.player_profile.lifecycle.vault.equipment.insert(Uuid::from_u128(12781),
                equipment(12781, "dormant_unborrowed_weapon"));
            start_new_game_with_default_starters(&mut core, player);
            let active = core.state.player_profile.lifecycle.active_expedition;
            let vault = core.state.player_profile.lifecycle.vault.clone();
            if used.is_some() {
                let save = force_first_available_node(&mut core, MapNodeCategory::SavePoint,
                    "save_point", MapNodePayload::SavePoint);
                select_and_confirm_map_node(&mut core, player, save);
                core.execute(player, PlayerBehavior::CompleteNode).unwrap();
                assert!(core.state.run_checkpoint.payload.is_some());
                core.state.run_checkpoint.loads_used = used.unwrap();
            }
            let healthy = serde_json::to_value(&core.state.roster).unwrap();
            core.state.enkephalin.amount = 0;
            let (combat, _, _) = combat_map(&mut core, false);
            select_and_confirm_map_node(&mut core, player, combat);
            let held: BTreeSet<_> = core.state.inventory.equipments.iter().map(|copy| copy.instance_uuid).collect();
            settle_conditions(&mut core, combat, physical, 0, None);
            let result = core.execute(player, PlayerBehavior::CompleteCombatResult).unwrap();
            assert_eq!(core.state.player_profile.lifecycle.vault, vault);
            if used == Some(2) {
                assert!(matches!(result, BehaviorResult::RunFailureDecisionRequired { reason: actual, .. } if actual == reason));
                assert!(matches!(core.get_state(), GameState::AwaitingRunFailureDecision { reason: actual, .. } if actual == reason));
                assert_eq!(core.state.player_profile.lifecycle.active_expedition, active);
                assert_eq!(core.state.roster.len(), 3, "failure candidate has not terminated employees");
                assert_eq!(core.state.inventory.equipments.iter().map(|copy| copy.instance_uuid).collect::<BTreeSet<_>>(), held);
                assert!(action_capability_enabled(&core, ActionKind::LoadRunCheckpoint));
                assert!(action_capability_enabled(&core, ActionKind::ConfirmRunFailure));
                let mut recover = resume_equal(&core, player);
                recover.execute(player, PlayerBehavior::LoadRunCheckpoint).unwrap();
                assert_eq!(recover.state.run_checkpoint.loads_used, 3);
                assert_eq!(serde_json::to_value(&recover.state.roster).unwrap(), healthy);
                assert_eq!(recover.state.player_profile.lifecycle.active_expedition, active);
                assert_eq!(recover.state.player_profile.lifecycle.vault, vault);
                assert!(!action_capability_enabled(&recover, ActionKind::LoadRunCheckpoint));
                recover = resume_equal(&recover, player);
                reject(&mut recover, player, PlayerBehavior::LoadRunCheckpoint);
                let mut confirm = resume_equal(&core, player);
                confirm.execute(player, PlayerBehavior::ConfirmRunFailure).unwrap();
                assert!(matches!(confirm.get_state(), GameState::RunFailed { reason: actual } if actual == reason));
                assert!(confirm.state.player_profile.lifecycle.active_expedition.is_none());
                assert_eq!(confirm.state.roster.len(), 0);
                assert!(confirm.state.inventory.equipments.iter().next().is_none());
                assert_eq!(confirm.state.player_profile.lifecycle.vault, vault);
                assert!(confirm.export_active_run_checkpoint().unwrap().is_none());
                reject(&mut confirm, player, PlayerBehavior::ConfirmRunFailure);
            } else {
                assert!(matches!(result, BehaviorResult::RunFailed { reason: actual, .. } if actual == reason));
                assert!(matches!(core.get_state(), GameState::RunFailed { reason: actual } if actual == reason));
                assert!(core.state.player_profile.lifecycle.active_expedition.is_none());
                assert_eq!(core.state.roster.len(), 0);
                assert!(core.state.inventory.equipments.iter().next().is_none());
                assert!(core.export_active_run_checkpoint().unwrap().is_none());
                reject(&mut core, player, PlayerBehavior::LoadRunCheckpoint);
                reject(&mut core, player, PlayerBehavior::ConfirmRunFailure);
            }
        }
    }
}

fn gate_fixture(depth: u32, physical: u8, trauma: u32, wallet: u32) -> (GameCore, Uuid, MapNodeId) {
    let player = Uuid::from_u128(12790);
    let mut core = GameCore::new(legacy_pool_game_data_from_ron(), 12790);
    start_new_game_with_default_starters(&mut core, player);
    // A real completed SavePoint is retained across the subsequent explicit
    // location/condition fixture and actual Gate transition.
    let save = force_first_available_node(&mut core, MapNodeCategory::SavePoint, "save_point", MapNodePayload::SavePoint);
    select_and_confirm_map_node(&mut core, player, save);
    core.execute(player, PlayerBehavior::CompleteNode).unwrap();
    let gate = force_first_available_node(&mut core, MapNodeCategory::Gate, "gate_stairs", MapNodePayload::None);
    let limit = core.run_policy().corrosion.breakdown_collapse_cost_limit;
    for employee in core.state.roster.iter_mut() { condition(employee, physical, trauma, limit); }
    core.state.enkephalin.amount = wallet;
    let run = core.state.run.as_mut().unwrap();
    run.run_progression.mode_state = crate::game::map::RunProgressionModeState::Expedition { floor_index: depth - 1 };
    run.run_progression.deterrence = 12;
    run.combat_recovery_restriction = Some(CombatRecoveryRestriction { allowed_node_ids: vec![gate] });
    publish_fixture(&mut core);
    (core, player, gate)
}

#[test]
fn u12_lifecycle_recovery_depth_two_gate_heals_without_return_and_preserves_prior_savepoint() {
    let (source, player, gate) = gate_fixture(2, 0, 100, 0);
    let mut core = resume_equal(&source, player);
    restriction(&core, &[gate]);
    let old_savepoint = serde_json::to_value(&core.state.run_checkpoint.payload).unwrap();
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: gate }).unwrap();
    reject(&mut core, player, PlayerBehavior::ChooseExpeditionContinuation { choice: ExpeditionContinuationChoice::Return });
    let result = core.execute(player, PlayerBehavior::ConfirmEnterNode).unwrap();
    assert!(matches!(result, BehaviorResult::FloorAdvanced { floor_index: 2, .. }));
    assert_eq!(core.state.run.as_ref().unwrap().run_progression.floor_index(), 2);
    assert!(core.state.run.as_ref().unwrap().run_progression.pending_continuation.is_none());
    assert!(core.state.roster.iter().all(|employee| employee.condition.trauma == 90 && employee.is_battle_deployable()));
    assert!(!core.has_combat_recovery_restriction());
    assert_eq!(serde_json::to_value(&core.state.run_checkpoint.payload).unwrap(), old_savepoint);
    assert_eq!(core.state.run_checkpoint.loads_used, 0);
    reject(&mut core, player, PlayerBehavior::ChooseExpeditionContinuation { choice: ExpeditionContinuationChoice::Return });
    resume_equal(&core, player);
}

#[test]
fn u12_lifecycle_recovery_gate_continue_uses_actual_next_floor_hq_and_rolls_back_unrecoverable_depth_five() {
    for depth in [3, 4] {
        let (source, player, gate) = gate_fixture(depth, 5, 0, 100);
        let mut core = resume_equal(&source, player);
        let old_nodes: BTreeSet<_> = core.state.run.as_ref().unwrap().map.nodes.iter().map(|node| node.id).collect();
        let old_savepoint = serde_json::to_value(&core.state.run_checkpoint.payload).unwrap();
        select_and_confirm_map_node(&mut core, player, gate);
        let pending = core.state.run.as_ref().unwrap().run_progression.pending_continuation.unwrap();
        assert_eq!(pending.depth, depth);
        assert_eq!(pending.node_id, gate);
        core = resume_equal(&core, player);
        let behavior = PlayerBehavior::ChooseExpeditionContinuation { choice: ExpeditionContinuationChoice::Continue };
        if depth == 4 {
            // GameCore::execute uses the same execute_application draft route
            // as public commands. Rejection must discard generated map/UUIDs,
            // recovery, profile records and the consumed pending choice.
            reject(&mut core, player, behavior);
            assert_eq!(core.state.run.as_ref().unwrap().run_progression.floor_index(), 3);
            assert_eq!(core.state.run.as_ref().unwrap().run_progression.pending_continuation, Some(pending));
            core.execute(player, PlayerBehavior::ChooseExpeditionContinuation { choice: ExpeditionContinuationChoice::Return }).unwrap();
            assert!(matches!(core.get_state(), GameState::RunComplete));
            continue;
        }
        core.execute(player, behavior).unwrap();
        assert_eq!(core.state.run.as_ref().unwrap().run_progression.floor_index(), 3);
        assert_eq!(core.state.enkephalin.amount, 100);
        assert_eq!(serde_json::to_value(&core.state.run_checkpoint.payload).unwrap(), old_savepoint);
        let selectable = core.selectable_map_node_ids();
        let hq = selectable.iter().copied().find(|id|
            core.state.run.as_ref().unwrap().map.node(*id).unwrap().category == MapNodeCategory::HeadquartersContact)
            .expect("actual generated depth-four map must expose immediate selectable HQ");
        assert!(!old_nodes.contains(&hq));
        restriction(&core, &[hq]);
        assert!(core.state.run.as_ref().unwrap().run_progression.entry_receipts.is_empty());
        core = resume_equal(&core, player);
        let roster_before_entry = serde_json::to_value(&core.state.roster).unwrap();
        let inventory_before_entry = serde_json::to_value(&core.state.inventory).unwrap();
        let entered = select_and_confirm_map_node(&mut core, player, hq);
        let BehaviorResult::HeadquartersContactState { recruitment_candidates, .. } = entered else { panic!("actual next-floor HQ"); };
        let profile = core.state.player_profile.clone();
        let selected = recruitment_candidates.first().expect("depth-four paid candidate").employee_definition_id.clone();
        let hired = core.execute(player, PlayerBehavior::RecruitEmployee { employee_definition_id: selected.clone() }).unwrap();
        let BehaviorResult::EmployeeRecruited { employee_uuid: hired_uuid, .. } = hired else {
            panic!("actual depth-four paid hire");
        };
        assert_eq!(core.state.enkephalin.amount, 0);
        assert!(core.state.roster.iter().any(Employee::is_battle_deployable));
        assert!(!core.has_combat_recovery_restriction());
        assert_unfinished_hq_rolls_back(&core, player, &roster_before_entry, &inventory_before_entry);
        let emergency_enkephalin = core.run_policy().headquarters.emergency_enkephalin;
        let granted = core.execute(player, PlayerBehavior::RequestEmergencySupplies).unwrap();
        let BehaviorResult::EmergencySuppliesGranted { enkephalin, completion, .. } = granted else {
            panic!("actual next-floor HQ emergency supplies completion");
        };
        assert_eq!(enkephalin, emergency_enkephalin);
        assert!(matches!(*completion, BehaviorResult::NodeCompleted { .. }));
        assert!(matches!(core.get_state(), GameState::ViewingMap));
        assert_eq!(core.state.enkephalin.amount, emergency_enkephalin);
        let hired = core.state.roster.iter().find(|employee| employee.uuid == hired_uuid).unwrap();
        assert_eq!(hired.employee_definition_id, selected);
        assert!(hired.is_battle_deployable());
        assert_eq!(core.state.player_profile, profile);
        assert_eq!(serde_json::to_value(&core.state.run_checkpoint.payload).unwrap(), old_savepoint);
        resume_equal(&core, player);
    }
}

#[test]
fn u12_lifecycle_recovery_gate_keeps_bag_compound_restriction_and_exhaustion_rejects_false_continue() {
    let (mut core, player, target, weapon) = compound_fixture();
    let gate = force_first_available_node(&mut core, MapNodeCategory::Gate, "gate_stairs", MapNodePayload::None);
    let run = core.state.run.as_mut().unwrap();
    run.run_progression.mode_state = crate::game::map::RunProgressionModeState::Expedition { floor_index: 2 };
    run.combat_recovery_restriction = Some(CombatRecoveryRestriction { allowed_node_ids: vec![gate] });
    publish_fixture(&mut core);
    select_and_confirm_map_node(&mut core, player, gate);
    core.execute(player, PlayerBehavior::ChooseExpeditionContinuation { choice: ExpeditionContinuationChoice::Continue }).unwrap();
    assert_eq!(core.state.run.as_ref().unwrap().run_progression.floor_index(), 3);
    restriction(&core, &[]);
    core = resume_equal(&core, player);
    core.execute(player, PlayerBehavior::UseConsumableItem { item_uuid: treatment().uuid, target_employee_uuid: target }).unwrap();
    restriction(&core, &[]);
    core.execute(player, PlayerBehavior::EquipItem { item_uuid: weapon, target_unit: target }).unwrap();
    assert!(!core.has_combat_recovery_restriction());

    let (mut source, player, _) = gate_fixture(3, 5, 0, 0);
    let unrelated = force_first_available_node(&mut source, MapNodeCategory::Reward, "reward_treasure",
        MapNodePayload::Reward { reward_pool_id: Some("default_treasures".into()) });
    let run = source.state.run.as_mut().unwrap();
    run.run_progression.deterrence = 1;
    run.combat_recovery_restriction = Some(CombatRecoveryRestriction { allowed_node_ids: vec![unrelated] });
    publish_fixture(&mut source);
    select_and_confirm_map_node(&mut source, player, unrelated);
    assert_eq!(source.state.run.as_ref().unwrap().run_progression.deterrence, 0);
    assert!(source.state.run.as_ref().unwrap().run_progression.entry_receipts.contains(&unrelated));
    // U14 unfinished-node Continue rewinds the entry and its exhaustion choice.
    let saved=source.export_active_run_checkpoint().unwrap().unwrap();
    let mut restored=core_with_profile_for_continue(&source,12799);
    restored.continue_from_active_run_checkpoint(player,&saved.core_payload_json,saved.loads_used,saved.run_peak_certified_power).unwrap();
    assert_eq!(restored.state.run.as_ref().unwrap().run_progression.deterrence,1);
    assert!(restored.state.run.as_ref().unwrap().run_progression.pending_continuation.is_none());
    select_and_confirm_map_node(&mut restored,player,unrelated);
    reject(&mut restored, player, PlayerBehavior::ChooseExpeditionContinuation { choice: ExpeditionContinuationChoice::Continue });
    restored.execute(player, PlayerBehavior::ChooseExpeditionContinuation { choice: ExpeditionContinuationChoice::Return }).unwrap();
    assert!(matches!(restored.get_state(), GameState::RunComplete));
    assert!(restored.export_active_run_checkpoint().unwrap().is_none());
}
