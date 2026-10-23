use super::*;
use crate::game::managers::uuid_manager::UuidManager;
use crate::game::world::state::{CombatRecoveryRestriction, RunCheckpointPayload};

const ACTIVE_FRAGMENT_ID: &str = "checkpoint_active_fragment";
const SEED_ERROR: &str = "run checkpoint seed authority is inconsistent";
const PROGRESSION_ERROR: &str = "run checkpoint progression does not match its map graph";
const ROSTER_ERROR: &str = "run checkpoint roster identity graph is inconsistent";
const EQUIPMENT_ERROR: &str = "run checkpoint equipment ownership graph is inconsistent";
const INVENTORY_ERROR: &str = "run checkpoint inventory stack graph is inconsistent";
const FRAGMENT_ERROR: &str = "run checkpoint skill fragment graph is inconsistent";
const CONTENT_ERROR: &str =
    "run checkpoint content identity does not match the retained generation";
const CANONICAL_V4_CHECKPOINT_SHA256: &str =
    "64f78ce035c38e523c95b14456296750aa2a04a1b43f3f0f00b5c101d608bcc2";

const CURRENT_CHECKPOINT_KEYS: [&str; 21] = [
    "abnormality_attempts",
    "combat_previews",
    "content_identity",
    "current_floor_power_anchor",
    "encounter_progression",
    "enkephalin",
    "event_sessions",
    "facility_run_state",
    "inventory",
    "map",
    "map_progression",
    "power_model_version",
    "record_progress",
    "recruitment",
    "roster",
    "run_progression",
    "shop_sessions",
    "skill_fragments",
    "typed_resources",
    "uuid_manager",
    "visual_novel_trigger_checkpoint_consumed",
];

const CHECKPOINT_EXCLUDED_ROOT_FIELDS: [&str; 11] = [
    "active_battle",
    "active_node_content",
    "combat_recovery_restriction",
    "encounter_compendium",
    "game_state",
    "loads_used",
    "node_session",
    "player",
    "player_profile",
    "run_peak_certified_power",
    "visual_novel",
];

#[derive(Clone, Copy)]
struct MutationCase {
    name: &'static str,
    expected_error: &'static str,
    mutate: fn(&mut Value),
}

pub(super) fn active_fragment_metadata() -> SkillFragmentMetadata {
    SkillFragmentMetadata {
        corrosion_class: crate::game::data::skill_fragment_data::FragmentCorrosionClass::Ordinary,
        id: SkillFragmentId::from(ACTIVE_FRAGMENT_ID),
        uuid: Uuid::from_u128(0xF123),
        name: "Checkpoint Active Fragment".to_string(),
        description: "checkpoint graph fixture".to_string(),
        rarity: SkillFragmentRarity::Rare,
        equip_limit: SkillFragmentEquipLimit::OwnedCopies,
        origin: None,
        sources: vec![SkillFragmentAcquisitionSource::RareReward],
        dependencies: vec![],
        compatibility: Default::default(),
        effect: SkillFragmentEffectDef::ActiveSkill {
            forms: std::collections::BTreeMap::from([(
                "base".into(),
                crate::game::data::skill_fragment_data::SkillFragmentForm {
                    resonance_max: 100,
                    skill_id: SkillId::from("checkpoint_active_skill"),
                    compatibility: None,
                    bloom_recipe: None,
                },
            )]),
            passive_guard: None,
            dismantle_recipe: None,
        },
    }
}

fn checkpoint_consumable() -> ConsumableMetadata {
    ConsumableMetadata {
        id: "checkpoint_tonic".to_string(),
        uuid: Uuid::from_u128(0xC123),
        name: "Checkpoint Tonic".to_string(),
        description: "checkpoint graph fixture".to_string(),
        tier: ConsumableTier::Common,
        rarity: crate::game::enums::RiskLevel::ZAYIN,
        price: 3,
        target_policy: ConsumableTargetPolicy::SingleEmployee,
        duration_policy: ConsumableDurationPolicy::NextCombatNode,
        effect: ConsumableEffect::ReducePhysicalDamage { amount: 1 },
        live_pool: true,
    }
}

fn checkpoint_game_data() -> Arc<GameDataBase> {
    game_data_with_skill_fragments(vec![active_fragment_metadata()])
}

fn valid_checkpoint_source() -> (Arc<GameDataBase>, GameCore) {
    let game_data = checkpoint_game_data();
    let mut core = GameCore::new(game_data.clone(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);

    let fragment = active_fragment_metadata();
    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();
    core.state.skill_fragments.add_fragment_dust(28).unwrap();
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    core.state
        .skill_fragments
        .equip_copy(
            &mut core.state.roster,
            &game_data.skill_fragment_data,
            copy_uuid,
            employee_uuid,
            false,
        )
        .unwrap();
    core.state
        .inventory
        .add_consumable(Arc::new(checkpoint_consumable()), 2)
        .unwrap();
    core.state
        .inventory
        .equipment_materials
        .add("equipment_dust", 5)
        .unwrap();

    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    core.execute(player_id, PlayerBehavior::CompleteNode)
        .unwrap();

    core.save_run_checkpoint_for_test().unwrap();
    (game_data, core)
}

fn valid_checkpoint_fixture() -> (
    Arc<GameDataBase>,
    GameCore,
    ActiveRunCheckpointExport,
    Value,
) {
    let (game_data, core) = valid_checkpoint_source();
    let export = core
        .export_active_run_checkpoint()
        .unwrap()
        .expect("valid SavePoint checkpoint must export");
    let document = serde_json::from_slice(&export.core_payload_json).unwrap();
    (game_data, core, export, document)
}

fn object_keys(value: &Value) -> Vec<String> {
    let mut keys = value
        .as_object()
        .expect("fixture JSON object")
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    keys.sort();
    keys
}

fn first_equipment_key(document: &Value) -> String {
    object_keys(&document["continuation"]["current"]["inventory"]["equipments"]["items"])
        .into_iter()
        .next()
        .expect("starter equipment")
}

fn employee_keys(document: &Value) -> Vec<String> {
    object_keys(&document["continuation"]["current"]["roster"]["employees"])
}

fn employee_weapon(document: &Value, employee_key: &str) -> Value {
    document["continuation"]["current"]["roster"]["employees"][employee_key]["loadout"]["item_slot"]
        ["layout"]["ByType"]["weapon"]
        .clone()
}

fn employee_weapon_mut<'a>(document: &'a mut Value, employee_key: &str) -> &'a mut Value {
    &mut document["continuation"]["current"]["roster"]["employees"][employee_key]["loadout"]
        ["item_slot"]["layout"]["ByType"]["weapon"]
}

fn mutate_run_progression_seed(document: &mut Value) {
    document["continuation"]["current"]["run_progression"]["run_seed"] = json!(124_u64);
}

fn mutate_uuid_manager_seed(document: &mut Value) {
    document["continuation"]["current"]["uuid_manager"]["run_seed"] = json!(124_u64);
}

fn mutate_current_node_state(document: &mut Value) {
    let current = document["continuation"]["current"]["map_progression"]["current_node_id"]
        .as_str()
        .unwrap()
        .to_string();
    let nodes = document["continuation"]["current"]["map"]["nodes"]
        .as_array_mut()
        .unwrap();
    let current_node = nodes.iter_mut().find(|node| node["id"] == current).unwrap();
    current_node["state"] = json!("Available");
}

fn mutate_duplicate_completed_node(document: &mut Value) {
    let completed = document["continuation"]["current"]["map_progression"]["completed_node_ids"]
        .as_array_mut()
        .unwrap();
    completed.push(completed[0].clone());
}

fn mutate_stale_available_node(document: &mut Value) {
    let current = document["continuation"]["current"]["map_progression"]["current_node_id"].clone();
    document["continuation"]["current"]["map_progression"]["available_node_ids"]
        .as_array_mut()
        .unwrap()
        .push(current);
}

fn mutate_roster_embedded_uuid(document: &mut Value) {
    let employee = employee_keys(document).remove(0);
    document["continuation"]["current"]["roster"]["employees"][&employee]["uuid"] =
        json!(Uuid::from_u128(0xBAD1));
}

fn mutate_equipment_embedded_uuid(document: &mut Value) {
    let equipment = first_equipment_key(document);
    document["continuation"]["current"]["inventory"]["equipments"]["items"][&equipment]
        ["instance_uuid"] = json!(Uuid::from_u128(0xBAD2));
}

fn mutate_missing_equipment(document: &mut Value) {
    let employee = employee_keys(document).remove(0);
    let instance_uuid = employee_weapon(document, &employee)["instance_uuid"]
        .as_str()
        .unwrap()
        .to_string();
    document["continuation"]["current"]["inventory"]["equipments"]["items"]
        .as_object_mut()
        .unwrap()
        .remove(&instance_uuid);
}

fn mutate_duplicate_equipment_slot(document: &mut Value) {
    let employees = employee_keys(document);
    let first_weapon = employee_weapon(document, &employees[0]);
    *employee_weapon_mut(document, &employees[1]) = first_weapon;
}

fn mutate_wrong_equipment_reverse_binding(document: &mut Value) {
    let equipment = first_equipment_key(document);
    let employees = employee_keys(document);
    let current_owner = document["continuation"]["current"]["inventory"]["equipments"]["items"]
        [&equipment]["equipped_to"]
        .as_str()
        .unwrap();
    let wrong_owner = employees
        .into_iter()
        .find(|employee| employee != current_owner)
        .unwrap();
    document["continuation"]["current"]["inventory"]["equipments"]["items"][&equipment]
        ["equipped_to"] = json!(wrong_owner);
}

fn mutate_slot_base_uuid(document: &mut Value) {
    let employee = employee_keys(document).remove(0);
    employee_weapon_mut(document, &employee)["base_uuid"] = json!(Uuid::from_u128(0xBAD3));
}

fn mutate_slot_type(document: &mut Value) {
    let employee = employee_keys(document).remove(0);
    employee_weapon_mut(document, &employee)["equipment_type"] = json!("Armor");
}

fn mutate_reserve_underflow(document: &mut Value) {
    document["continuation"]["current"]["inventory"]["reserved_slot_count"] = json!(0);
}

fn mutate_consumable_key(document: &mut Value) {
    let items = document["continuation"]["current"]["inventory"]["consumables"]["items"]
        .as_object_mut()
        .unwrap();
    let stack = items.remove("checkpoint_tonic").unwrap();
    items.insert("wrong_tonic_key".to_string(), stack);
}

fn mutate_consumable_quantity(document: &mut Value) {
    document["continuation"]["current"]["inventory"]["consumables"]["items"]["checkpoint_tonic"]
        ["quantity"] = json!(0);
}

fn mutate_material_quantity(document: &mut Value) {
    document["continuation"]["current"]["inventory"]["equipment_materials"]["stacks"]
        ["equipment_dust"] = json!(0);
}

fn fragment_key(document: &Value) -> String {
    object_keys(&document["continuation"]["current"]["skill_fragments"]["copies"])[0].clone()
}
fn mutate_fragment_stack_count(document: &mut Value) {
    let key = fragment_key(document);
    document["continuation"]["current"]["skill_fragments"]["copies"][&key]["copy_uuid"] =
        json!(Uuid::nil());
}
fn mutate_fragment_progress(document: &mut Value) {
    let key = fragment_key(document);
    document["continuation"]["current"]["skill_fragments"]["copies"][&key]["selected_form"] =
        json!("locked");
}
fn mutate_unowned_active_fragment(document: &mut Value) {
    let key = fragment_key(document);
    document["continuation"]["current"]["skill_fragments"]["copies"]
        .as_object_mut()
        .unwrap()
        .remove(&key);
}
fn mutate_active_fragment_over_owned_copies(document: &mut Value) {
    let key = fragment_key(document);
    let employees = employee_keys(document);
    let second = employees
        .iter()
        .find(|id| {
            document["continuation"]["current"]["roster"]["employees"][id.as_str()]
                ["skill_fragments"]["active_copy_uuid"]
                != key
        })
        .unwrap();
    document["continuation"]["current"]["roster"]["employees"][second]["skill_fragments"]
        ["active_copy_uuid"] = json!(key);
}

fn mutation_cases() -> [MutationCase; 19] {
    [
        MutationCase {
            name: "run progression seed",
            expected_error: SEED_ERROR,
            mutate: mutate_run_progression_seed,
        },
        MutationCase {
            name: "UUID manager seed",
            expected_error: SEED_ERROR,
            mutate: mutate_uuid_manager_seed,
        },
        MutationCase {
            name: "current node state",
            expected_error: PROGRESSION_ERROR,
            mutate: mutate_current_node_state,
        },
        MutationCase {
            name: "duplicate completed node",
            expected_error: PROGRESSION_ERROR,
            mutate: mutate_duplicate_completed_node,
        },
        MutationCase {
            name: "stale available node",
            expected_error: PROGRESSION_ERROR,
            mutate: mutate_stale_available_node,
        },
        MutationCase {
            name: "roster key/embedded UUID",
            expected_error: ROSTER_ERROR,
            mutate: mutate_roster_embedded_uuid,
        },
        MutationCase {
            name: "equipment key/embedded UUID",
            expected_error: EQUIPMENT_ERROR,
            mutate: mutate_equipment_embedded_uuid,
        },
        MutationCase {
            name: "missing equipped instance",
            expected_error: EQUIPMENT_ERROR,
            mutate: mutate_missing_equipment,
        },
        MutationCase {
            name: "duplicate equipped instance",
            expected_error: EQUIPMENT_ERROR,
            mutate: mutate_duplicate_equipment_slot,
        },
        MutationCase {
            name: "wrong equipment reverse binding",
            expected_error: EQUIPMENT_ERROR,
            mutate: mutate_wrong_equipment_reverse_binding,
        },
        MutationCase {
            name: "slot base UUID",
            expected_error: EQUIPMENT_ERROR,
            mutate: mutate_slot_base_uuid,
        },
        MutationCase {
            name: "slot equipment type",
            expected_error: EQUIPMENT_ERROR,
            mutate: mutate_slot_type,
        },
        MutationCase {
            name: "inventory reserve underflow",
            expected_error: INVENTORY_ERROR,
            mutate: mutate_reserve_underflow,
        },
        MutationCase {
            name: "consumable map key",
            expected_error: INVENTORY_ERROR,
            mutate: mutate_consumable_key,
        },
        MutationCase {
            name: "zero consumable quantity",
            expected_error: INVENTORY_ERROR,
            mutate: mutate_consumable_quantity,
        },
        MutationCase {
            name: "zero material quantity",
            expected_error: INVENTORY_ERROR,
            mutate: mutate_material_quantity,
        },
        MutationCase {
            name: "fragment key/embedded UUID",
            expected_error: "fragment copy has multiple owners",
            mutate: mutate_fragment_stack_count,
        },
        MutationCase {
            name: "locked selected form",
            expected_error: FRAGMENT_ERROR,
            mutate: mutate_fragment_progress,
        },
        MutationCase {
            name: "unowned active fragment",
            expected_error: FRAGMENT_ERROR,
            mutate: mutate_unowned_active_fragment,
        },
    ]
}

fn assert_invalid_static_data(error: GameError, expected: &str) -> Result<(), String> {
    match error {
        GameError::InvalidStaticData(actual) if actual == expected => Ok(()),
        other => Err(format!(
            "expected InvalidStaticData({expected:?}), got {other:?}"
        )),
    }
}

#[test]
fn u06_v11_implicit_damage_profile_preserves_canonical_checksum_and_continue() {
    let (game_data, source, exported, mut document) = valid_checkpoint_fixture();
    fn omit_damage(value: &mut Value) -> usize {
        match value {
            Value::Object(fields) => {
                let mut count = 0;
                for key in ["weapon_profile", "basic_attack"] {
                    if let Some(profile) = fields.get_mut(key).and_then(Value::as_object_mut) {
                        if profile.get("action") == Some(&json!("Damage")) {
                            profile.remove("action");
                        }
                        assert!(
                            !profile.contains_key("action"),
                            "legacy fixture has damage weapons"
                        );
                        count += 1;
                    }
                }
                count + fields.values_mut().map(omit_damage).sum::<usize>()
            }
            Value::Array(values) => values.iter_mut().map(omit_damage).sum(),
            _ => 0,
        }
    }
    assert!(omit_damage(&mut document) > 0);
    refresh_active_run_checksum(&mut document);
    let original_bytes = serde_json::to_vec(&document).unwrap();
    let original_hash = document["continuation_sha256"].clone();
    let mut destination = GameCore::new(game_data, 912_006);
    destination.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
    destination.continue_from_active_run_checkpoint(
        Uuid::from_u128(1), &original_bytes,
        exported.loads_used, exported.run_peak_certified_power,
    ).expect("supported V11 without the formerly implicit Damage field must retain its canonical checksum");
    let restored = destination.export_active_run_checkpoint().unwrap().unwrap();
    let restored: Value = serde_json::from_slice(&restored.core_payload_json).unwrap();
    assert_eq!(restored["continuation"], document["continuation"]);
    assert_eq!(restored["continuation_sha256"], original_hash);
    assert_eq!(serde_json::to_vec(&document).unwrap(), original_bytes);
}

#[test]
fn current_v12_checkpoint_document_has_exact_envelope_and_checksum() {
    let (_, _, _, document) = valid_checkpoint_fixture();
    assert_eq!(
        object_keys(&document),
        ["continuation", "continuation_sha256", "version"]
    );
    assert_eq!(document["version"], json!(12));
    assert_eq!(
        object_keys(&document["continuation"]["current"]),
        CURRENT_CHECKPOINT_KEYS
    );
    let mut refreshed = document.clone();
    refresh_active_run_checksum(&mut refreshed);
    assert_eq!(
        document["continuation_sha256"],
        refreshed["continuation_sha256"]
    );
    assert_eq!(document["continuation_sha256"].as_str().unwrap().len(), 64);
}

#[test]
fn current_v12_checkpoint_document_has_exact_flat_semantic_field_set_and_values() {
    let (game_data, core, valid_export, document) = valid_checkpoint_fixture();
    assert_eq!(
        object_keys(&document),
        ["continuation", "continuation_sha256", "version"]
    );
    assert_eq!(document["version"], json!(12));
    assert_eq!(
        object_keys(&document["continuation"]["current"]),
        CURRENT_CHECKPOINT_KEYS
    );
    assert_eq!(
        document["continuation"]["current"]["content_identity"],
        json!(core.content_identity())
    );
    // U12 changes mode/identity/roster semantics. Compare every current field
    // to its independently held owner below; historical V7 byte integrity and
    // explicit rejection are retained in the dedicated legacy test.
    let run = core.state.run.as_ref().expect("fixture run");
    macro_rules! assert_run_field_parity {
        ($($field:ident),+ $(,)?) => {
            $(
                assert_eq!(
                    document["continuation"]["current"][stringify!($field)],
                    serde_json::to_value(&run.$field).unwrap(),
                    "checkpoint field {} must match its RunState source",
                    stringify!($field)
                );
            )+
        };
    }
    assert_run_field_parity!(
        map,
        map_progression,
        run_progression,
        facility_run_state,
        encounter_progression,
        event_sessions,
        shop_sessions,
        combat_previews,
        abnormality_attempts,
        record_progress,
        recruitment,
    );
    assert_eq!(
        document["continuation"]["current"]["power_model_version"],
        json!(core.state.run_power_authority.power_model_version)
    );
    assert_eq!(
        document["continuation"]["current"]["current_floor_power_anchor"],
        json!(core.state.run_power_authority.current_floor_power_anchor)
    );
    assert_eq!(
        document["continuation"]["current"]["visual_novel_trigger_checkpoint_consumed"],
        serde_json::to_value(&core.state.visual_novel_triggers.checkpoint_rewind_consumed).unwrap()
    );
    for (name, value) in [
        (
            "enkephalin",
            serde_json::to_value(&core.state.enkephalin).unwrap(),
        ),
        (
            "inventory",
            serde_json::to_value(&core.state.inventory).unwrap(),
        ),
        ("roster", serde_json::to_value(&core.state.roster).unwrap()),
        (
            "skill_fragments",
            serde_json::to_value(&core.state.skill_fragments).unwrap(),
        ),
        (
            "uuid_manager",
            serde_json::to_value(&core.state.uuid_manager).unwrap(),
        ),
    ] {
        assert_eq!(document["continuation"]["current"][name], value, "{name}");
    }
    for excluded in CHECKPOINT_EXCLUDED_ROOT_FIELDS {
        assert!(
            document["continuation"]["current"].get(excluded).is_none(),
            "{excluded} must remain outside the Core checkpoint"
        );
    }

    for (index, excluded) in CHECKPOINT_EXCLUDED_ROOT_FIELDS.into_iter().enumerate() {
        let mut rejected = document.clone();
        rejected["continuation"]["current"]
            .as_object_mut()
            .unwrap()
            .insert(excluded.to_string(), Value::Null);
        let mut destination = GameCore::new(game_data.clone(), 30_000 + index as u64);
    destination.install_authoritative_player_profile(core.state.player_profile.clone()).unwrap();
        let error = destination
            .continue_from_active_run_checkpoint(
                Uuid::from_u128(1),
                &serde_json::to_vec(&rejected).unwrap(),
                valid_export.loads_used,
                valid_export.run_peak_certified_power,
            )
            .expect_err("strict V11 must reject excluded root state");
        assert!(
            matches!(error, GameError::InvalidStaticData(ref message) if message.contains("unknown field")),
            "{excluded} was not rejected as an unknown strict-V11 field: {error:?}"
        );
        assert!(destination.state.run.is_none(), "{excluded}");
    }
}

#[test]
fn current_v12_strictly_rejects_missing_extra_research_and_pending_shapes() {
    let (game_data, source, valid_export, document) = valid_checkpoint_fixture();
    let mut cases = Vec::new();

    let mut missing = document.clone();
    missing["continuation"]["current"]
        .as_object_mut()
        .unwrap()
        .remove("encounter_progression");
    cases.push(("missing encounter progression", missing));

    let mut extra = document.clone();
    extra["unexpected_v6_field"] = json!(true);
    cases.push(("extra envelope field", extra));

    let mut research = document.clone();
    research["continuation"]["current"]["abnormality_research"] = json!({});
    cases.push(("removed run research", research));

    let mut pending = document.clone();
    pending["continuation"]["current"]["skill_fragments"]["pending_research_deliveries"] =
        json!({});
    cases.push(("removed pending delivery", pending));

    let mut progress = document;
    progress["continuation"]["current"]["skill_fragments"]["progress"][ACTIVE_FRAGMENT_ID]
        ["research_progress"] = json!(100);
    cases.push(("removed fragment research progress", progress));

    for (index, (name, case)) in cases.into_iter().enumerate() {
        let mut destination = GameCore::new(game_data.clone(), 50_000 + index as u64);
    destination.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
        let error = destination
            .continue_from_active_run_checkpoint(
                Uuid::from_u128(1),
                &serde_json::to_vec(&case).unwrap(),
                valid_export.loads_used,
                valid_export.run_peak_certified_power,
            )
            .expect_err(name);
        assert!(
            matches!(error, GameError::InvalidStaticData(_)),
            "{name}: {error:?}"
        );
        assert!(destination.state.run.is_none(), "{name}");
        assert!(destination.state.player.is_none(), "{name}");
    }
}

#[test]
fn current_v12_checkpoint_continue_then_v12_capture_is_semantically_identical() {
    let (game_data, mut source) = valid_checkpoint_source();
    source
        .state
        .run
        .as_mut()
        .expect("fixture run")
        .combat_recovery_restriction = Some(CombatRecoveryRestriction {
        allowed_node_ids: Vec::new(),
    });
    source.save_run_checkpoint_for_test().unwrap();
    let original = source.export_active_run_checkpoint().unwrap().unwrap();
    let original_document = serde_json::from_slice::<Value>(&original.core_payload_json).unwrap();
    assert!(
        original_document["continuation"]["current"]
            .get("combat_recovery_restriction")
            .is_none(),
        "SavePoint excludes the live recovery restriction"
    );

    let mut restored = GameCore::new(game_data, 40_000);
    restored.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
    restored
        .continue_from_active_run_checkpoint(
            Uuid::from_u128(1),
            &original.core_payload_json,
            original.loads_used,
            original.run_peak_certified_power,
        )
        .expect("canonical strict V11 must continue");
    assert!(restored.state.active_battle.is_none());
    assert!(restored.state.active_node_content.is_none());
    assert!(restored.state.node_session.is_none());
    assert!(
        restored
            .state
            .run
            .as_ref()
            .unwrap()
            .combat_recovery_restriction
            .as_ref()
            .is_some_and(|restriction| restriction.allowed_node_ids.is_empty()),
        "Continue preserves the live recovery restriction"
    );

    restored.save_run_checkpoint_for_test().unwrap();
    let recaptured = restored
        .export_active_run_checkpoint()
        .unwrap()
        .expect("restored checkpoint must recapture");
    let recaptured_document =
        serde_json::from_slice::<Value>(&recaptured.core_payload_json).unwrap();
    assert_eq!(recaptured_document["version"], json!(12));
    assert_eq!(
        recaptured_document["continuation"]["current"],
        original_document["continuation"]["current"]
    );
    assert_eq!(recaptured.loads_used, original.loads_used);
    assert_eq!(
        recaptured.run_peak_certified_power,
        original.run_peak_certified_power
    );
}

#[test]
fn current_v12_checkpoint_graph_rejects_one_edge_defects_before_continue_mutation() {
    let (game_data, source, valid_export, valid_document) = valid_checkpoint_fixture();
    let mut valid_restored = GameCore::new(game_data.clone(), 9_999);
    valid_restored.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
    valid_restored
        .continue_from_active_run_checkpoint(
            Uuid::from_u128(1),
            &valid_export.core_payload_json,
            valid_export.loads_used,
            valid_export.run_peak_certified_power,
        )
        .expect("canonical strict V11 checkpoint must continue");
    let restored_run = valid_restored.state.run.as_ref().unwrap();
    let restored_template = valid_restored
        .facility_template_for_map(&restored_run.map)
        .unwrap();
    assert!(restored_run
        .map_progression
        .validate_checkpoint_state(&restored_run.map, &restored_template));
    assert_eq!(
        restored_run.map_progression.available_node_ids,
        restored_run
            .map_progression
            .selectable_node_ids(&restored_run.map, &restored_template)
    );
    let valid_reexport = valid_restored
        .export_active_run_checkpoint()
        .unwrap()
        .expect("continued checkpoint must remain exportable");
    assert_eq!(
        serde_json::from_slice::<Value>(&valid_reexport.core_payload_json).unwrap()["continuation"]
            ["current"],
        valid_document["continuation"]["current"]
    );
    assert_eq!(valid_reexport.loads_used, valid_export.loads_used);
    assert_eq!(
        valid_reexport.run_peak_certified_power,
        valid_export.run_peak_certified_power
    );

    let mut failures = Vec::new();
    for (index, case) in mutation_cases().into_iter().enumerate() {
        let mut document = valid_document.clone();
        (case.mutate)(&mut document);
        refresh_active_run_checksum(&mut document);
        let bytes = serde_json::to_vec(&document).unwrap();
        let initial_seed = 20_000 + index as u64;
        let mut destination = GameCore::new(game_data.clone(), initial_seed);
    destination.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
        let next_employee_uuid = destination
            .state
            .uuid_manager
            .peek(crate::game::managers::uuid_manager::UuidManager::NS_EMPLOYEE);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            destination.continue_from_active_run_checkpoint(
                Uuid::from_u128(1),
                &bytes,
                valid_export.loads_used,
                valid_export.run_peak_certified_power,
            )
        }));
        match result {
            Err(_) => failures.push(format!("{}: panicked", case.name)),
            Ok(Ok(_)) => failures.push(format!("{}: accepted", case.name)),
            Ok(Err(error)) => match assert_invalid_static_data(error, case.expected_error) {
                Ok(()) => {
                    assert!(destination.state.run.is_none(), "{}", case.name);
                    assert!(destination.state.player.is_none(), "{}", case.name);
                    assert!(
                        destination.state.run_checkpoint.payload.is_none(),
                        "{}",
                        case.name
                    );
                    assert_eq!(
                        destination.state.run_checkpoint.loads_used, 0,
                        "{}",
                        case.name
                    );
                    assert_eq!(destination.run_seed, initial_seed, "{}", case.name);
                    assert_eq!(
                        destination
                            .state
                            .uuid_manager
                            .peek(crate::game::managers::uuid_manager::UuidManager::NS_EMPLOYEE),
                        next_employee_uuid,
                        "{}",
                        case.name
                    );
                    assert_eq!(destination.export_active_run_checkpoint().unwrap(), None);
                }
                Err(detail) => failures.push(format!("{}: {detail}", case.name)),
            },
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn checkpoint_graph_rejects_one_edge_defects_before_explicit_load_mutation() {
    let mut failures = Vec::new();
    for case in mutation_cases() {
        let (_, mut core, _, mut document) = valid_checkpoint_fixture();
        (case.mutate)(&mut document);
        let checkpoint: RunCheckpointPayload =
            serde_json::from_value(document["continuation"]["current"].clone()).unwrap();
        core.state.run_checkpoint.payload = Some(checkpoint);

        let state_before = core.get_run_snapshot_json().unwrap();
        let map_before = core.current_map_view().unwrap();
        let payload_before =
            serde_json::to_value(core.state.run_checkpoint.payload.as_ref().unwrap()).unwrap();
        let loads_before = core.state.run_checkpoint.loads_used;
        let seed_before = core.run_seed;
        let export_before = core.export_active_run_checkpoint().unwrap();
        let next_employee_uuid = core
            .state
            .uuid_manager
            .peek(crate::game::managers::uuid_manager::UuidManager::NS_EMPLOYEE);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            core.handle_load_run_checkpoint()
        }));
        match result {
            Err(_) => failures.push(format!("{}: panicked", case.name)),
            Ok(Ok(_)) => failures.push(format!("{}: accepted", case.name)),
            Ok(Err(error)) => match assert_invalid_static_data(error, case.expected_error) {
                Ok(()) => {
                    assert_eq!(
                        core.get_run_snapshot_json().unwrap(),
                        state_before,
                        "{}",
                        case.name
                    );
                    assert_eq!(
                        core.current_map_view().unwrap(),
                        map_before,
                        "{}",
                        case.name
                    );
                    assert_eq!(
                        serde_json::to_value(core.state.run_checkpoint.payload.as_ref().unwrap())
                            .unwrap(),
                        payload_before,
                        "{}",
                        case.name
                    );
                    assert_eq!(
                        core.state.run_checkpoint.loads_used, loads_before,
                        "{}",
                        case.name
                    );
                    assert_eq!(core.run_seed, seed_before, "{}", case.name);
                    assert_eq!(core.export_active_run_checkpoint().unwrap(), export_before);
                    assert_eq!(
                        core.state
                            .uuid_manager
                            .peek(crate::game::managers::uuid_manager::UuidManager::NS_EMPLOYEE),
                        next_employee_uuid,
                        "{}",
                        case.name
                    );
                }
                Err(detail) => failures.push(format!("{}: {detail}", case.name)),
            },
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn checkpoint_explicit_load_rejects_an_internally_consistent_foreign_seed() {
    let (_, mut core, _, mut document) = valid_checkpoint_fixture();
    document["continuation"]["current"]["run_progression"]["run_seed"] = json!(124_u64);
    document["continuation"]["current"]["uuid_manager"]["run_seed"] = json!(124_u64);
    core.state.run_checkpoint.payload =
        Some(serde_json::from_value(document["continuation"]["current"].clone()).unwrap());

    let state_before = core.get_run_snapshot_json().unwrap();
    let map_before = core.current_map_view().unwrap();
    let payload_before =
        serde_json::to_value(core.state.run_checkpoint.payload.as_ref().unwrap()).unwrap();
    let loads_before = core.state.run_checkpoint.loads_used;
    let seed_before = core.run_seed;
    let export_before = core.export_active_run_checkpoint().unwrap();
    let next_employee_uuid = core
        .state
        .uuid_manager
        .peek(crate::game::managers::uuid_manager::UuidManager::NS_EMPLOYEE);

    let error = core
        .handle_load_run_checkpoint()
        .expect_err("explicit load must retain the live run seed authority");
    assert_invalid_static_data(error, SEED_ERROR).unwrap();
    assert_eq!(core.get_run_snapshot_json().unwrap(), state_before);
    assert_eq!(core.current_map_view().unwrap(), map_before);
    assert_eq!(
        serde_json::to_value(core.state.run_checkpoint.payload.as_ref().unwrap()).unwrap(),
        payload_before
    );
    assert_eq!(core.state.run_checkpoint.loads_used, loads_before);
    assert_eq!(core.run_seed, seed_before);
    assert_eq!(core.export_active_run_checkpoint().unwrap(), export_before);
    assert_eq!(
        core.state
            .uuid_manager
            .peek(crate::game::managers::uuid_manager::UuidManager::NS_EMPLOYEE),
        next_employee_uuid
    );
}

#[test]
fn current_v12_checkpoint_fragment_equip_limit_is_validated_as_part_of_the_graph() {
    let (game_data, source, valid_export, mut document) = valid_checkpoint_fixture();
    mutate_active_fragment_over_owned_copies(&mut document);
    refresh_active_run_checksum(&mut document);
    let bytes = serde_json::to_vec(&document).unwrap();
    let mut destination = GameCore::new(game_data, 4_321);
    destination.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
    let error = destination
        .continue_from_active_run_checkpoint(
            Uuid::from_u128(1),
            &bytes,
            valid_export.loads_used,
            valid_export.run_peak_certified_power,
        )
        .expect_err("one owned copy cannot be active on two employees");
    assert_invalid_static_data(error, FRAGMENT_ERROR).unwrap();
    assert!(destination.state.run.is_none());
    assert_eq!(destination.state.run_checkpoint.loads_used, 0);
}

fn continuation_authority_snapshot(core: &GameCore) -> Value {
    json!({
        "player": core.state.player,
        "phase": core.get_state(),
        "active_node_content": format!("{:?}", core.state.active_node_content),
        "node_session": core.state.node_session,
        "run_snapshot": core
            .state
            .run
            .as_ref()
            .map(|_| core.get_run_snapshot_json().unwrap()),
        "visual_novel": core.state.visual_novel,
        "visual_novel_triggers": core.state.visual_novel_triggers,
        "uuid_manager": core.state.uuid_manager,
        "enkephalin": core.state.enkephalin,
        "inventory": core.state.inventory,
        "roster": core.state.roster,
        "starter_candidates": core.state.starter_candidates,
        "pending_game_mode": core.state.pending_game_mode,
        "skill_fragments": core.state.skill_fragments,
        "player_profile": core.state.player_profile,
        "run_checkpoint": core.state.run_checkpoint,
        "run_power_authority": core.state.run_power_authority,
        "run_seed": core.run_seed,
        "content_identity": core.content_identity(),
        "encoded_continuation": core.state.continue_state.encoded.as_deref(),
    })
}

#[test]
fn current_v12_checkpoint_round_trips_with_canonical_map_order_independent_checksum() {
    let (game_data, source, export, document) = valid_checkpoint_fixture();
    let repeated_export = source
        .export_active_run_checkpoint()
        .unwrap()
        .expect("same checkpoint must remain exportable");
    assert_eq!(repeated_export, export);

    let player_id = source.state.player.as_ref().unwrap().id;
    let mut direct = GameCore::new(game_data.clone(), 100_001);
    direct.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
    let direct_result = direct
        .continue_from_active_run_checkpoint(
            player_id,
            &export.core_payload_json,
            export.loads_used,
            export.run_peak_certified_power,
        )
        .expect("writer-produced V11 must continue");

    let checkpoint_json = serde_json::to_string(&document["continuation"]).unwrap();
    let checksum_json = serde_json::to_string(&document["continuation_sha256"]).unwrap();
    let version=document["version"].as_u64().unwrap();
    let reordered_bytes = format!(
        "{{\"continuation_sha256\":{checksum_json},\"continuation\":{checkpoint_json},\"version\":{version}}}"
    )
    .into_bytes();
    assert_ne!(
        reordered_bytes.as_slice(),
        export.core_payload_json.as_ref()
    );

    let mut reordered = GameCore::new(game_data, 100_002);
    reordered.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
    let reordered_result = reordered
        .continue_from_active_run_checkpoint(
            player_id,
            &reordered_bytes,
            export.loads_used,
            export.run_peak_certified_power,
        )
        .expect("semantically identical reordered V11 must continue");

    assert_eq!(
        serde_json::to_value(direct_result).unwrap(),
        serde_json::to_value(reordered_result).unwrap()
    );
    assert_eq!(
        direct.get_run_snapshot_json().unwrap(),
        reordered.get_run_snapshot_json().unwrap()
    );
    assert_eq!(
        direct.current_map_view().unwrap(),
        reordered.current_map_view().unwrap()
    );
    assert_eq!(
        serde_json::to_value(direct.state.run_checkpoint.payload.as_ref().unwrap()).unwrap(),
        serde_json::to_value(reordered.state.run_checkpoint.payload.as_ref().unwrap()).unwrap()
    );
}

#[test]
fn public_application_checkpoint_port_round_trips_into_a_fresh_application() {
    let (game_data, source) = valid_checkpoint_source();
    let source = crate::application::CoreApplication::from_core(source);
    let checkpoint = source
        .export_active_run_checkpoint()
        .unwrap()
        .expect("completed SavePoint must export through the application port");
    let document: Value = serde_json::from_slice(checkpoint.core_payload_json()).unwrap();
    assert_eq!(document["version"], 12);

    let mut destination = crate::application::CoreApplication::new(game_data, 0xCA07_0004);
    install_application_profile_for_continue(&mut destination, &source);
    let continued = destination
        .continue_from_active_run_checkpoint(Uuid::from_u128(1), &checkpoint)
        .unwrap();
    assert_eq!(
        continued.kind(),
        crate::application::ApplicationResultKind::MapState
    );
    assert_ne!(
        continued.disposition(),
        crate::application::ApplicationResultDisposition::AcceptedRecoveryPending
    );
    assert_eq!(
        destination
            .snapshot(crate::application::ApplicationSnapshotKind::Run)
            .unwrap()
            .read_model()
            .as_value()["game_state"],
        "viewing_map"
    );
}

#[test]
fn current_v12_continue_restores_rng_and_replays_the_same_next_gate() {
    let (game_data, mut source) = valid_checkpoint_source();
    let player_id = source.state.player.as_ref().unwrap().id;
    let gate = force_first_available_node(
        &mut source,
        MapNodeCategory::Gate,
        "gate_stairs",
        MapNodePayload::None,
    );
    source.save_run_checkpoint_for_test().unwrap();
    let export = source.export_active_run_checkpoint().unwrap().unwrap();
    let mut restored = GameCore::new(game_data, 110_002);
    restored.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
    restored
        .continue_from_active_run_checkpoint(
            player_id,
            &export.core_payload_json,
            export.loads_used,
            export.run_peak_certified_power,
        )
        .unwrap();
    assert_eq!(restored.run_seed, source.run_seed);
    assert_eq!(restored.content_identity(), source.content_identity());
    assert_eq!(
        json!(restored.state.uuid_manager),
        json!(source.state.uuid_manager)
    );
    assert_eq!(
        restored.current_map_view().unwrap(),
        source.current_map_view().unwrap()
    );
    assert_eq!(
        restored.state.run_power_authority,
        source.state.run_power_authority
    );
    assert_eq!(
        json!(restored.state.inventory),
        json!(source.state.inventory)
    );
    assert_eq!(json!(restored.state.roster), json!(source.state.roster));
    // Exercise the saved streams without advancing either live authority.
    let mut source_rng = source.state.uuid_manager.clone();
    let mut restored_rng = restored.state.uuid_manager.clone();
    for namespace in [
        UuidManager::NS_EMPLOYEE,
        UuidManager::NS_OWNED_EQUIPMENT,
        UuidManager::NS_BATTLE_SESSION,
        UuidManager::NS_MASTERED_ELITE_SWEEP_RESULT,
    ] {
        for _ in 0..3 {
            assert_eq!(source_rng.next(namespace), restored_rng.next(namespace));
        }
    }
    source
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id: gate })
        .unwrap();
    restored
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id: gate })
        .unwrap();
    let source_result = source
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let restored_result = restored
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert!(matches!(
        source_result,
        BehaviorResult::FloorAdvanced { .. }
    ));
    assert_eq!(json!(source_result), json!(restored_result));
    assert_eq!(
        source.current_map_view().unwrap(),
        restored.current_map_view().unwrap()
    );
    assert_eq!(
        json!(source.state.uuid_manager),
        json!(restored.state.uuid_manager)
    );
    assert_eq!(
        source.state.run_power_authority,
        restored.state.run_power_authority
    );
}

#[test]
fn current_v12_corrupt_strict_and_unknown_inputs_reject_before_mutation_on_retry() {
    let (game_data, source, export, document) = valid_checkpoint_fixture();
    let mut cases: Vec<(&str, Vec<u8>)> = vec![("malformed JSON", b"{".to_vec())];

    let mut missing_version = document.clone();
    missing_version.as_object_mut().unwrap().remove("version");
    cases.push((
        "missing version",
        serde_json::to_vec(&missing_version).unwrap(),
    ));

    let mut unknown_version = document.clone();
    unknown_version["version"] = json!(99);
    cases.push((
        "unknown version",
        serde_json::to_vec(&unknown_version).unwrap(),
    ));

    for old in [9, 10, 11, 13] {
        let mut old_version = document.clone();
        old_version["version"] = json!(old);
        cases.push((
            "unsupported previous or checkpoint-only version",
            serde_json::to_vec(&old_version).unwrap(),
        ));
    }
    let mut removed_field = document.clone();
    for employee in removed_field["continuation"]["current"]["roster"]["employees"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        employee["combat_profile"]["battle_profile"]["resonance"] =
            json!({"start":20,"max":100,"gain_lock_ms":0});
    }
    refresh_active_run_checksum(&mut removed_field);
    cases.push((
        "removed employee resonance field with valid checksum",
        serde_json::to_vec(&removed_field).unwrap(),
    ));

    let mut missing_checksum = document.clone();
    missing_checksum
        .as_object_mut()
        .unwrap()
        .remove("continuation_sha256");
    cases.push((
        "missing checksum",
        serde_json::to_vec(&missing_checksum).unwrap(),
    ));

    let mut missing_identity = document.clone();
    missing_identity["continuation"]["current"]
        .as_object_mut()
        .unwrap()
        .remove("content_identity");
    cases.push((
        "missing required content identity",
        serde_json::to_vec(&missing_identity).unwrap(),
    ));
    for field in [
        "pack_format",
        "catalog_schema",
        "authoring",
        "execution",
        "fingerprint",
    ] {
        let mut missing = document.clone();
        missing["continuation"]["current"]["content_identity"]["contract"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        cases.push((field, serde_json::to_vec(&missing).unwrap()));
    }
    let mut missing_fingerprint = document.clone();
    missing_fingerprint["continuation"]["current"]["content_identity"]
        .as_object_mut()
        .unwrap()
        .remove("semantic_sha256");
    cases.push((
        "missing fingerprint",
        serde_json::to_vec(&missing_fingerprint).unwrap(),
    ));

    let mut extra_envelope = document.clone();
    extra_envelope["legacy_payload"] = json!(null);
    cases.push((
        "extra envelope field",
        serde_json::to_vec(&extra_envelope).unwrap(),
    ));

    let mut missing_checkpoint = document.clone();
    missing_checkpoint
        .as_object_mut()
        .unwrap()
        .remove("continuation");
    cases.push((
        "missing checkpoint",
        serde_json::to_vec(&missing_checkpoint).unwrap(),
    ));

    let mut extra_checkpoint = document.clone();
    extra_checkpoint["continuation"]["current"]["legacy_alias"] = json!(true);
    cases.push((
        "extra checkpoint field",
        serde_json::to_vec(&extra_checkpoint).unwrap(),
    ));

    let mut corrupted_checkpoint = document.clone();
    corrupted_checkpoint["continuation"]["current"]["run_progression"]["run_seed"] = json!(124_u64);
    cases.push((
        "semantic corruption with stale checksum",
        serde_json::to_vec(&corrupted_checkpoint).unwrap(),
    ));

    let mut graph_invalid_with_valid_checksum = document.clone();
    mutate_current_node_state(&mut graph_invalid_with_valid_checksum);
    refresh_active_run_checksum(&mut graph_invalid_with_valid_checksum);
    cases.push((
        "graph-invalid checkpoint with valid checksum",
        serde_json::to_vec(&graph_invalid_with_valid_checksum).unwrap(),
    ));

    let mut uppercase_checksum = document;
    uppercase_checksum["continuation_sha256"] = json!(uppercase_checksum["continuation_sha256"]
        .as_str()
        .unwrap()
        .to_ascii_uppercase());
    cases.push((
        "non-canonical checksum spelling",
        serde_json::to_vec(&uppercase_checksum).unwrap(),
    ));

    for (index, (name, bytes)) in cases.into_iter().enumerate() {
        let mut destination = GameCore::new(game_data.clone(), 120_000 + index as u64);
    destination.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
        let before = continuation_authority_snapshot(&destination);
        let first = destination
            .continue_from_active_run_checkpoint(
                Uuid::from_u128(1),
                &bytes,
                export.loads_used,
                export.run_peak_certified_power,
            )
            .expect_err(name);
        assert!(matches!(first, GameError::InvalidStaticData(_)), "{name}");
        assert_eq!(
            continuation_authority_snapshot(&destination),
            before,
            "{name}"
        );

        let retry = destination
            .continue_from_active_run_checkpoint(
                Uuid::from_u128(1),
                &bytes,
                export.loads_used,
                export.run_peak_certified_power,
            )
            .expect_err(name);
        assert_eq!(format!("{first:?}"), format!("{retry:?}"), "{name}");
        assert_eq!(
            continuation_authority_snapshot(&destination),
            before,
            "{name}"
        );
    }
}

#[test]
fn u12_removed_psychology_rejects_current_and_nested_savepoint_before_mutation() {
    let (game_data, source, export, mut document) = valid_checkpoint_fixture();
    // Encode the genuine SavePoint explicitly so its reader is exercised
    // independently of the compact reference to the current checkpoint.
    assert!(document["continuation"]["savepoint"].is_object());
    document["continuation"]["savepoint"] = json!({
        "snapshot": {"checkpoint": document["continuation"]["current"].clone()}
    });
    refresh_active_run_checksum(&mut document);
    let mut positive = GameCore::new(game_data.clone(), 0xC120);
    positive.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
    positive.continue_from_active_run_checkpoint(
        Uuid::from_u128(1), &serde_json::to_vec(&document).unwrap(),
        export.loads_used, export.run_peak_certified_power,
    ).expect("explicit nested SavePoint is valid before the removed field is injected");
    assert_eq!(serde_json::to_value(&positive.state.roster).unwrap(), serde_json::to_value(&source.state.roster).unwrap());

    for nested in [false, true] {
        for removed in [Value::Null, json!({}), json!({"memory": [], "trust": {}})] {
            let mut invalid = document.clone();
            let checkpoint = if nested {
                &mut invalid["continuation"]["savepoint"]["snapshot"]["checkpoint"]
            } else {
                &mut invalid["continuation"]["current"]
            };
            let employee = checkpoint["roster"]["employees"].as_object_mut().unwrap().values_mut().next().unwrap();
            assert!(employee.get("psychology").is_none());
            employee["psychology"] = removed;
            refresh_active_run_checksum(&mut invalid);
            let bytes = serde_json::to_vec(&invalid).unwrap();
            let mut destination = GameCore::new(game_data.clone(), 0xC121);
            destination.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
            let before = continuation_authority_snapshot(&destination);
            for _ in 0..2 {
                let error = destination.continue_from_active_run_checkpoint(
                    Uuid::from_u128(1), &bytes, export.loads_used, export.run_peak_certified_power,
                ).expect_err("removed psychology must be rejected, including null and empty objects");
                assert!(matches!(error, GameError::InvalidStaticData(_)), "{error:?}");
                assert!(format!("{error:?}").contains("psychology"), "must reach the removed-field reader: {error:?}");
                assert_eq!(continuation_authority_snapshot(&destination), before);
            }
        }
    }
}

fn append_boundary_consumable(document: &mut Value, index: usize) {
    let mut stack = document["continuation"]["current"]["inventory"]["consumables"]["items"]
        ["checkpoint_tonic"]
        .clone();
    let definition_id = format!("checkpoint_boundary_consumable_{index:02}");
    stack["meta"]["id"] = json!(definition_id);
    stack["meta"]["uuid"] = json!(Uuid::from_u128(0xCB00 + index as u128));
    document["continuation"]["current"]["inventory"]["consumables"]["items"]
        .as_object_mut()
        .unwrap()
        .insert(definition_id, stack);
}

#[test]
fn current_v12_checkpoint_round_trips_exact_inventory_reservation_boundary() {
    let (game_data, source, export, mut document) = valid_checkpoint_fixture();
    let equipment_count = document["continuation"]["current"]["inventory"]["equipments"]["items"]
        .as_object()
        .unwrap()
        .len();
    let initial_consumable_count = document["continuation"]["current"]["inventory"]["consumables"]
        ["items"]
        .as_object()
        .unwrap()
        .len();
    for index in initial_consumable_count
        ..crate::game::resources::inventory::INITIAL_RESERVED_SLOT_COUNT - equipment_count
    {
        append_boundary_consumable(&mut document, index);
    }
    assert_eq!(
        document["continuation"]["current"]["inventory"]["equipments"]["items"]
            .as_object()
            .unwrap()
            .len()
            + document["continuation"]["current"]["inventory"]["consumables"]["items"]
                .as_object()
                .unwrap()
                .len(),
        crate::game::resources::inventory::INITIAL_RESERVED_SLOT_COUNT
    );
    refresh_active_run_checksum(&mut document);

    let mut exact = GameCore::new(game_data.clone(), 125_000);
    exact.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
    exact
        .continue_from_active_run_checkpoint(
            Uuid::from_u128(1),
            &serde_json::to_vec(&document).unwrap(),
            export.loads_used,
            export.run_peak_certified_power,
        )
        .expect("exact reserved inventory boundary must continue");
    assert_eq!(
        exact.state.inventory.occupied_slot_count(),
        crate::game::resources::inventory::INITIAL_RESERVED_SLOT_COUNT
    );
    assert_eq!(
        exact.state.inventory.reserved_slot_count(),
        crate::game::resources::inventory::INITIAL_RESERVED_SLOT_COUNT
    );
    exact.save_run_checkpoint_for_test().unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(
            &exact
                .export_active_run_checkpoint()
                .unwrap()
                .expect("boundary checkpoint must recapture")
                .core_payload_json
        )
        .unwrap()["version"],
        json!(12)
    );

    let mut overflow = document;
    append_boundary_consumable(
        &mut overflow,
        crate::game::resources::inventory::INITIAL_RESERVED_SLOT_COUNT,
    );
    refresh_active_run_checksum(&mut overflow);
    let mut rejected = GameCore::new(game_data, 125_001);
    rejected.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
    let before = continuation_authority_snapshot(&rejected);
    let error = rejected
        .continue_from_active_run_checkpoint(
            Uuid::from_u128(1),
            &serde_json::to_vec(&overflow).unwrap(),
            export.loads_used,
            export.run_peak_certified_power,
        )
        .expect_err("occupied inventory above its persisted reservation must reject");
    assert_invalid_static_data(error, INVENTORY_ERROR).unwrap();
    assert_eq!(continuation_authority_snapshot(&rejected), before);
}

#[test]
fn current_v12_load_count_and_active_lifecycle_boundaries_reject_without_mutation() {
    let (game_data, source, export, _) = valid_checkpoint_fixture();
    let player_id = source.state.player.as_ref().unwrap().id;
    let empty = GameCore::new(checkpoint_game_data(), 130_000);
    assert_eq!(empty.export_active_run_checkpoint().unwrap(), None);

    let mut at_max = GameCore::new(game_data, 130_001);
    at_max.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
    at_max
        .continue_from_active_run_checkpoint(
            player_id,
            &export.core_payload_json,
            crate::game::world::state::RUN_CHECKPOINT_MAX_LOADS,
            export.run_peak_certified_power,
        )
        .expect("exact maximum load count remains a valid continue payload");
    assert_eq!(
        at_max.state.run_checkpoint.loads_used,
        crate::game::world::state::RUN_CHECKPOINT_MAX_LOADS
    );

    let mut over_max = GameCore::new(checkpoint_game_data(), 130_002);
    let over_max_before = continuation_authority_snapshot(&over_max);
    assert!(matches!(
        over_max.continue_from_active_run_checkpoint(
            player_id,
            &export.core_payload_json,
            crate::game::world::state::RUN_CHECKPOINT_MAX_LOADS + 1,
            export.run_peak_certified_power,
        ),
        Err(GameError::InvalidAction)
    ));
    assert_eq!(continuation_authority_snapshot(&over_max), over_max_before);

    let mut active = source;
    let active_before = continuation_authority_snapshot(&active);
    assert!(matches!(
        active.continue_from_active_run_checkpoint(
            player_id,
            &export.core_payload_json,
            export.loads_used,
            export.run_peak_certified_power,
        ),
        Err(GameError::InvalidAction)
    ));
    assert_eq!(continuation_authority_snapshot(&active), active_before);
}

#[test]
fn current_v12_rejects_v4_and_v5_before_mutation_on_retry() {
    let (game_data, source, export, document) = valid_checkpoint_fixture();
    for version in [4, 5, 6] {
        let mut legacy = document.clone();
        legacy["version"] = json!(version);
        legacy["continuation"]["current"]
            .as_object_mut()
            .unwrap()
            .remove("content_identity");
        if version == 4 {
            legacy
                .as_object_mut()
                .unwrap()
                .remove("continuation_sha256");
        } else {
            legacy["continuation_sha256"] = json!(CANONICAL_V4_CHECKPOINT_SHA256);
        }
        let mut destination = GameCore::new(game_data.clone(), 140_000 + version);
    destination.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
        let before = continuation_authority_snapshot(&destination);
        for _ in 0..2 {
            let error = destination
                .continue_from_active_run_checkpoint(
                    Uuid::from_u128(1),
                    &serde_json::to_vec(&legacy).unwrap(),
                    export.loads_used,
                    export.run_peak_certified_power,
                )
                .unwrap_err();
            assert_invalid_static_data(
                error,
                &format!(
                    "active-run continuation unsupported active-run core payload version {version}"
                ),
            )
            .unwrap();
            assert_eq!(continuation_authority_snapshot(&destination), before);
        }
    }
}

#[test]
fn current_v12_rejects_each_content_authority_mismatch_even_with_valid_checksum() {
    let (game_data, source, export, document) = valid_checkpoint_fixture();
    for field in [
        "semantic_sha256",
        "pack_format",
        "catalog_schema",
        "authoring",
        "execution",
        "fingerprint",
    ] {
        let mut mismatched = document.clone();
        let identity = &mut mismatched["continuation"]["current"]["content_identity"];
        if field == "semantic_sha256" {
            identity[field] = json!("0".repeat(64));
        } else {
            identity["contract"][field] = json!(identity["contract"][field].as_u64().unwrap() + 1);
        }
        refresh_active_run_checksum(&mut mismatched);
        let mut destination = GameCore::new(game_data.clone(), 150_000);
    destination.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
        let before = continuation_authority_snapshot(&destination);
        for _ in 0..2 {
            let error = destination
                .continue_from_active_run_checkpoint(
                    Uuid::from_u128(1),
                    &serde_json::to_vec(&mismatched).unwrap(),
                    export.loads_used,
                    export.run_peak_certified_power,
                )
                .unwrap_err();
            assert_invalid_static_data(error, CONTENT_ERROR).unwrap();
            assert_eq!(
                continuation_authority_snapshot(&destination),
                before,
                "{field}"
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn external_generations_bind_save_continue_explicit_load_and_cached_export() {
    use crate::game::data::content_pack::tests::ExternalFixture;
    let files = ExternalFixture::official();
    let a = files.load();
    let a_identity = a.identity().clone();
    let player_id = Uuid::from_u128(0xFA00);
    let mut source = GameCore::new(a.clone(), 123);
    start_new_game_with_default_starters(&mut source, player_id);
    let savepoint = force_first_available_node(
        &mut source,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
    );
    source
        .execute(
            player_id,
            PlayerBehavior::SelectMapNode { node_id: savepoint },
        )
        .unwrap();
    source
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    source
        .execute(player_id, PlayerBehavior::CompleteNode)
        .unwrap();
    let export = source.export_active_run_checkpoint().unwrap().unwrap();
    let original = files.read("official/data/equipments/base.ron");
    let price = a.equipment_data.items[0].price;
    let changed = original.replacen(
        &format!("price: {price}"),
        &format!("price: {}", price + 1),
        1,
    );
    assert_ne!(original, changed);
    files.write("official/data/equipments/base.ron", changed);
    let b = files.load();
    assert_ne!(a_identity, *b.identity());
    assert_eq!(source.content_identity(), &a_identity);
    assert_eq!(
        source.export_active_run_checkpoint().unwrap().unwrap(),
        export
    );
    let document: Value = serde_json::from_slice(&export.core_payload_json).unwrap();
    assert_eq!(document["version"], 12);
    assert_eq!(
        document["continuation"]["current"]["content_identity"],
        json!(a_identity)
    );

    let mut wrong = GameCore::new(b.clone(), 160_000);
    wrong.state.player_profile.highest_floor_reached = 12;
    let before = continuation_authority_snapshot(&wrong);
    for _ in 0..2 {
        let error = wrong
            .continue_from_active_run_checkpoint(
                player_id,
                &export.core_payload_json,
                export.loads_used,
                export.run_peak_certified_power,
            )
            .unwrap_err();
        assert_invalid_static_data(error, CONTENT_ERROR).unwrap();
        assert_eq!(continuation_authority_snapshot(&wrong), before);
    }
    start_new_game_with_default_starters(&mut wrong, player_id);
    wrong.state.run_checkpoint = source.state.run_checkpoint.clone();
    assert!(wrong.state.run_checkpoint.payload.is_some());
    assert!(!wrong.checkpoint_payload_is_loadable());
    let before = continuation_authority_snapshot(&wrong);
    let export_before = wrong.export_active_run_checkpoint().unwrap();
    assert!(
        export_before.is_some(),
        "immutable current continuation is independent of a replaced SavePoint"
    );
    assert_invalid_static_data(
        wrong.handle_load_run_checkpoint().unwrap_err(),
        CONTENT_ERROR,
    )
    .unwrap();
    assert_eq!(continuation_authority_snapshot(&wrong), before);
    assert_eq!(wrong.export_active_run_checkpoint().unwrap(), export_before);

    files.write("official/data/equipments/base.ron", original);
    let equivalent_a = files.load();
    assert_eq!(equivalent_a.identity(), &a_identity);
    let mut restored = GameCore::new(equivalent_a, 160_001);
    restored.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
    restored.state.player_profile.highest_floor_reached = 15;
    restored
        .continue_from_active_run_checkpoint(
            player_id,
            &export.core_payload_json,
            export.loads_used,
            export.run_peak_certified_power,
        )
        .unwrap();
    assert_eq!(restored.state.player_profile.highest_floor_reached, 15);
    assert_eq!(
        restored.current_map_view().unwrap(),
        source.current_map_view().unwrap()
    );
    assert_eq!(
        json!(restored.state.uuid_manager),
        json!(source.state.uuid_manager)
    );
    restored
        .execute(player_id, PlayerBehavior::LoadRunCheckpoint)
        .unwrap();
    assert_eq!(restored.content_identity(), &a_identity);
    assert_eq!(
        restored.state.run_checkpoint.loads_used,
        export.loads_used + 1
    );
    assert_eq!(restored.state.player_profile.highest_floor_reached, 15);
    assert_eq!(
        restored.current_map_view().unwrap(),
        source.current_map_view().unwrap()
    );
    assert_eq!(
        json!(restored.state.uuid_manager),
        json!(source.state.uuid_manager)
    );

    // The same-process rollback restores the strong generation handle with state
    // and seed, while keeping newer profile knowledge and the monotonic run peak.
    let rollback = source
        .capture_active_run_transaction_rollback()
        .unwrap()
        .unwrap();
    wrong.state.run_power_authority.run_peak_certified_power = 999_999;
    wrong.restore_active_run_transaction_rollback(rollback);
    assert_eq!(wrong.content_identity(), &a_identity);
    assert_eq!(wrong.run_seed, source.run_seed);
    assert_eq!(wrong.state.player_profile.highest_floor_reached, 12);
    assert_eq!(
        wrong.state.run_power_authority.run_peak_certified_power,
        999_999
    );
    assert_eq!(
        wrong.current_map_view().unwrap(),
        source.current_map_view().unwrap()
    );
    assert_eq!(
        wrong
            .export_active_run_checkpoint()
            .unwrap()
            .unwrap()
            .core_payload_json,
        export.core_payload_json
    );
}

#[test]
fn u12_original_preimplementation_v7_bytes_are_unsupported_and_preserved() {
    use sha2::{Digest, Sha256};
    let raw = include_bytes!("../../../../docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json");
    let original_digest = "47af09db635ab1691dc61d7094d6b2414237562c27cd6e1d1b6ce9a4e49024c9";
    assert_eq!(format!("{:x}", Sha256::digest(raw)), original_digest);
    let value: Value = serde_json::from_slice(raw).unwrap();
    let expected = serde_json::from_value(value["checkpoint"]["content_identity"].clone()).unwrap();
    let mut core = GameCore::new(checkpoint_game_data(), 900);
    let before = continuation_authority_snapshot(&core);
    for _ in 0..2 {
        assert!(core.continue_from_active_run_checkpoint(Uuid::from_u128(0xD000), raw, 0, 1).is_err());
        assert!(core.convert_active_run_v7(raw, &expected).is_err());
        assert_eq!(continuation_authority_snapshot(&core), before);
        assert_eq!(format!("{:x}", Sha256::digest(raw)), original_digest);
    }
}

#[test]
fn r06_nonzero_checkpoint_fresh_process_preserves_effective_inputs_and_rejects_tampering() {
    const CHILD: &str = "R06_NONZERO_CHECKPOINT_CHILD";
    if let Some(path) = std::env::var_os(CHILD) {
        let wrapper: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let raw = serde_json::to_vec(&wrapper["save"]).unwrap();
        let mut core = GameCore::new(checkpoint_game_data(), 987654);
        core.install_authoritative_player_profile(serde_json::from_value(wrapper["profile"].clone()).unwrap()).unwrap();
        core.continue_from_active_run_checkpoint(Uuid::from_u128(1), &raw, 0, 1)
            .unwrap();
        assert_eq!(
            json!(core.certified_input_fingerprint().unwrap()),
            wrapper["fingerprint"]
        );
        let profiles=core.state.roster.iter().map(|e|(e.uuid,crate::game::combat_setup::player_spawns::certified_combat_profile_for_employee(e,&core.state.inventory,&core.state.skill_fragments,&core.game_data).unwrap())).collect::<std::collections::BTreeMap<_,_>>();
        assert_eq!(json!(profiles), wrapper["profiles"]);
        assert!(core.state.active_battle.is_none());
        let before = continuation_authority_snapshot(&core);
        let mut corrupted = wrapper["save"].clone();
        let employee = employee_keys(&corrupted)[0].clone();
        corrupted["continuation"]["current"]["roster"]["employees"][&employee]["combat_profile"]
            ["battle_profile"]["stats"]["spell_power"] = json!(36);
        assert!(core
            .continue_from_active_run_checkpoint(
                Uuid::from_u128(1),
                &serde_json::to_vec(&corrupted).unwrap(),
                0,
                1
            )
            .is_err());
        assert_eq!(before, continuation_authority_snapshot(&core));
        return;
    }
    let (_, mut core) = valid_checkpoint_source();
    let e = core.state.roster.iter_mut().next().unwrap();
    e.combat_profile.battle_profile.stats.spell_power = 35;
    e.combat_profile
        .growth_stacks
        .add(crate::game::growth::GrowthId::KillStack, 10);
    let copy = e.loadout.item_slot.iter().next().unwrap().instance_uuid;
    core.state
        .inventory
        .equipments
        .get_item_mut(&copy)
        .unwrap()
        .enhancement_level = 1;
    core.save_run_checkpoint_for_test().unwrap();
    let export = core.export_active_run_checkpoint().unwrap().unwrap();
    let save: Value = serde_json::from_slice(&export.core_payload_json).unwrap();
    let profiles = core
        .state
        .roster
        .iter()
        .map(|e| {
            (
                e.uuid,
                crate::game::combat_setup::player_spawns::certified_combat_profile_for_employee(
                    e,
                    &core.state.inventory,
                    &core.state.skill_fragments,
                    &core.game_data,
                )
                .unwrap(),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let wrapper = json!({"profile":core.state.player_profile,"save":save,"fingerprint":core.certified_input_fingerprint().unwrap(),"profiles":profiles});
    let path = std::env::temp_dir().join(format!(
        "r06-nonzero-checkpoint-{}.json",
        std::process::id()
    ));
    std::fs::write(&path, serde_json::to_vec(&wrapper).unwrap()).unwrap();
    let output=std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact","game::world::tests::checkpoint_validation::r06_nonzero_checkpoint_fresh_process_preserves_effective_inputs_and_rejects_tampering","--nocapture"])
        .env(CHILD,&path).output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn u12_v7_converter_rejects_awakened_zero_count_and_duplicate_raw_legacy_shapes() {
    let raw = include_bytes!("../../../../docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json");
    let original = raw.to_vec();
    let mut value: Value = serde_json::from_slice(raw).unwrap();
    let expected = serde_json::from_value(value["checkpoint"]["content_identity"].clone()).unwrap();
    let core = GameCore::new(checkpoint_game_data(), 900);
    let before = continuation_authority_snapshot(&core);
    for count in [0, 2] {
        value["checkpoint"]["skill_fragments"]["stacks"][ACTIVE_FRAGMENT_ID]["count"] = json!(count);
        value["checkpoint"]["skill_fragments"]["progress"][ACTIVE_FRAGMENT_ID]["awakened"] = json!(true);
        refresh_active_run_checksum(&mut value);
        let input = serde_json::to_vec(&value).unwrap();
        assert!(core.convert_active_run_v7(&input, &expected).is_err());
        assert_eq!(continuation_authority_snapshot(&core), before);
    }
    let duplicate = String::from_utf8(original.clone()).unwrap().replacen(
        "\"version\":7", "\"version\":7,\"version\":7", 1);
    assert!(core.convert_active_run_v7(duplicate.as_bytes(), &expected).is_err());
    assert_eq!(raw.as_slice(), original.as_slice());
}

#[test]
fn u12_v8_converter_and_continue_reject_even_checksum_valid_legacy_shape() {
    use sha2::{Digest, Sha256};
    let (data, source, _, current) = valid_checkpoint_fixture();
    let mut legacy = json!({"version":8,"checkpoint":current["continuation"]["current"],"checkpoint_sha256":""});
    refresh_active_run_checksum(&mut legacy);
    let original = serde_json::to_vec(&legacy).unwrap();
    let digest = Sha256::digest(&original);
    let mut destination = GameCore::new(data, 9);
    let before = continuation_authority_snapshot(&destination);
    for _ in 0..2 {
        assert!(destination.continue_from_active_run_checkpoint(Uuid::from_u128(9), &original, 0, 1).is_err());
        assert!(destination.convert_active_run_v8(&original, source.content_identity()).is_err());
        assert_eq!(continuation_authority_snapshot(&destination), before);
        assert_eq!(Sha256::digest(&original), digest);
    }
}

#[test]
fn r09_counter_checkpoint_fresh_process_preserves_remaining_cost() {
    const CHILD: &str = "R09_COUNTER_CHECKPOINT_CHILD";
    if let Some(path) = std::env::var_os(CHILD) {
        let wrapper: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let raw = serde_json::to_vec(&wrapper["save"]).unwrap();
        let mut core = GameCore::new(checkpoint_game_data(), 919);
        core.install_authoritative_player_profile(serde_json::from_value(wrapper["profile"].clone()).unwrap()).unwrap();
        core.continue_from_active_run_checkpoint(Uuid::from_u128(1), &raw, 0, 1)
            .unwrap();
        let employee = core
            .state
            .roster
            .get(&serde_json::from_value(wrapper["employee"].clone()).unwrap())
            .unwrap();
        assert_eq!(employee.condition.trauma, 100);
        assert_eq!(employee.breakdown.remaining_cost(), Some(1));
        assert_eq!(
            serde_json::to_value(employee.breakdown).unwrap(),
            wrapper["counter"]
        );
        return;
    }
    let (_, mut core, _, _) = valid_checkpoint_fixture();
    let id = core.state.roster.iter().next().unwrap().uuid;
    let employee = core.state.roster.get_mut(&id).unwrap();
    employee.condition.trauma = 100;
    employee.breakdown = crate::game::employee::BreakdownCounter::Active {
        remaining_cost: 1,
        entered_node: None,
    };
    let counter = employee.breakdown;
    core.save_run_checkpoint_for_test().unwrap();
    let export = core.export_active_run_checkpoint().unwrap().unwrap();
    let wrapper = json!({"profile":core.state.player_profile,"save":serde_json::from_slice::<Value>(&export.core_payload_json).unwrap(),"employee":id,"counter":counter});
    let path = std::env::temp_dir().join(format!(
        "r09-counter-checkpoint-{}.json",
        std::process::id()
    ));
    std::fs::write(&path, serde_json::to_vec(&wrapper).unwrap()).unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact", "game::world::tests::checkpoint_validation::r09_counter_checkpoint_fresh_process_preserves_remaining_cost", "--nocapture"]).env(CHILD, &path).output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    std::fs::remove_file(path).unwrap();
}
