//! Normal routed run owners, authored data, and exact native failure boundaries.
use super::*;
use crate::game::battle::runtime_control as native;
use crate::game::{
    data::{relic_data::RelicDatabase, GameDataBuilder},
    relic::{RelicOperationKind, RelicVariant},
    world::AdminCommand,
};

pub(super) fn setup() -> (GameCore, Uuid, Vec<Uuid>) {
    let base = GameCore::new(live_game_data_from_ron(), 0x808);
    let relics: RelicDatabase = ron::de::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/u08/relics.ron"
    )))
    .unwrap();
    let data = GameDataBuilder::from_validated(&base.game_data)
        .with_relics(relics)
        .build_arc();
    let mut core = GameCore::new(data, 0x808);
    let player = Uuid::from_u128(1);
    start_new_game_with_legacy_sword_fixture(&mut core, player);
    let employees = core.state.roster.combat_eligible_employee_ids();
    (core, player, employees)
}
pub(super) fn grant(core: &mut GameCore, count: u32) -> Vec<Uuid> {
    core.execute_admin_command(AdminCommand::AdminGrantRelic {
        definition_id: "u08_probe".into(),
        count,
    })
    .unwrap();
    core.state.relics.copies().map(|c| c.copy_uuid).collect()
}
fn roots(core: &GameCore) -> Value {
    json!({"world": non_battle_state_without_profile_fingerprint(core), "relics": core.state.relics, "materials": core.state.crafting_resources, "legacy": core.state.skill_fragments, "uuid": core.state.uuid_manager, "continue": core.export_active_run_checkpoint().unwrap().map(|v| v.core_payload_json)})
}

#[test]
fn canonical_checksum_valid_forged_relic_slot_graphs_refuse_continue_without_changing_any_root() {
    let (mut source, player, employees) = setup();
    let copies = grant(&mut source, 2);
    assert!(
        !source
            .game_data
            .relic_data
            .get_by_id("u08_probe")
            .unwrap()
            .allow_duplicates
    );
    source
        .execute(
            player,
            PlayerBehavior::EquipRelic {
                copy_uuid: copies[0],
                employee_uuid: employees[0],
                slot: 0,
            },
        )
        .unwrap();
    source
        .execute_admin_command(AdminCommand::AdminGrantRelicCorruptionCatalyst { amount: 3 })
        .unwrap();
    source
        .execute(
            player,
            PlayerBehavior::CorruptRelic {
                copy_uuid: copies[0],
            },
        )
        .unwrap();
    let export = source.export_active_run_checkpoint().unwrap().unwrap();
    let original: Value = serde_json::from_slice(&export.core_payload_json).unwrap();
    let source_before = roots(&source);
    let identity = source.game_data.identity().clone();
    let definitions = serde_json::to_value(&source.game_data.relic_data).unwrap();
    let source_bytes = export.core_payload_json.clone();
    let employee = employees[0].to_string();
    let other_employee = employees[1].to_string();
    let missing = Uuid::from_u128(0x808_ffff);
    assert!(source.state.relics.get(missing).is_none());
    assert!(source.state.roster.get(&missing).is_none());
    for case in [
        "copy_in_two_employees",
        "unknown_copy",
        "unknown_employee",
        "duplicate_definition",
    ] {
        let mut wire = original.clone();
        let slots = wire["continuation"]["current"]["relics"]["slots"]
            .as_object_mut()
            .unwrap();
        match case {
            "copy_in_two_employees" => {
                slots.insert(other_employee.clone(), json!([null, null, copies[0]]));
            }
            "unknown_copy" => {
                slots.insert(employee.clone(), json!([missing, null, null]));
            }
            "unknown_employee" => {
                slots.remove(&employee);
                slots.insert(missing.to_string(), json!([copies[0], null, null]));
            }
            "duplicate_definition" => {
                slots.insert(employee.clone(), json!([copies[0], copies[1], null]));
            }
            _ => unreachable!(),
        }
        // Preserve syntactic three-slot shape and recompute the real canonical
        // envelope checksum, so these must reach typed graph validation.
        refresh_active_run_checksum(&mut wire);
        assert_ne!(wire["continuation_sha256"], original["continuation_sha256"]);
        let forged = serde_json::to_vec(&wire).unwrap();
        let mut target = core_with_profile_for_continue(&source, 0x808_f0);
        let before = roots(&target);
        for _ in 0..2 {
            let error = target
                .continue_from_active_run_checkpoint(
                    player,
                    &forged,
                    export.loads_used,
                    export.run_peak_certified_power,
                )
                .unwrap_err();
            assert!(
                matches!(&error, GameError::InvalidStaticData(message) if message == "run checkpoint relic graph is inconsistent"),
                "{case}: {error:?}"
            );
            assert_eq!(roots(&target), before, "{case}");
            assert!(target.state.run.is_none());
            assert_eq!(roots(&source), source_before, "{case}");
            assert_eq!(source.game_data.identity(), &identity);
            assert_eq!(
                serde_json::to_value(&source.game_data.relic_data).unwrap(),
                definitions
            );
            assert_eq!(
                source
                    .export_active_run_checkpoint()
                    .unwrap()
                    .unwrap()
                    .core_payload_json,
                source_bytes
            );
        }
        println!("U08_FORGED_SLOT_GRAPH case={case} canonical_checksum_valid=true exact_graph_rejection=true roots_unchanged=true");
    }
    let mut valid = core_with_profile_for_continue(&source, 0x808_f1);
    valid
        .continue_from_active_run_checkpoint(
            player,
            &source_bytes,
            export.loads_used,
            export.run_peak_certified_power,
        )
        .unwrap();
    assert_eq!(valid.state.relics, source.state.relics);
    assert_eq!(
        valid.state.crafting_resources,
        source.state.crafting_resources
    );
    assert_eq!(
        valid.state.relics.get(copies[0]).unwrap().variant,
        RelicVariant::Corrupt
    );
    assert_eq!(valid.state.crafting_resources.relic_corruption_catalysts, 2);
    assert_eq!(roots(&source), source_before);
}

fn corruption_host_fixture(cap: usize) -> (GameCore, Uuid, Uuid, Uuid) {
    let (mut prepared, player, employees) = setup();
    let copy = grant(&mut prepared, 1)[0];
    prepared
        .execute(
            player,
            PlayerBehavior::EquipRelic {
                copy_uuid: copy,
                employee_uuid: employees[0],
                slot: 2,
            },
        )
        .unwrap();
    prepared
        .execute_admin_command(AdminCommand::AdminGrantRelicCorruptionCatalyst { amount: 1 })
        .unwrap();
    prepared.save_run_checkpoint_for_test().unwrap();
    let before = roots(&prepared);
    let defaults = native::RuntimeFactLimits::default();
    let embedded = crate::runtime_safety::RuntimeSafetySettings::embedded();
    assert_eq!(defaults, embedded.native_runtime());
    assert!(embedded.development_measurement().is_none());
    assert!(cap <= defaults.max_retained_bytes);
    let raw = std::str::from_utf8(crate::runtime_safety::EMBEDDED_RUNTIME_SAFETY_RON).unwrap();
    let original = format!("max_retained_bytes: {},", defaults.max_retained_bytes);
    assert!(raw.contains(&original));
    let raw = raw.replacen(&original, &format!("max_retained_bytes: {cap},"), 1);
    let settings = Arc::new(
        crate::runtime_safety::RuntimeSafetySettings::from_ron(
            "u08 corruption host cap fixture",
            raw.as_bytes(),
        )
        .unwrap(),
    );
    assert_eq!(
        settings.native_runtime(),
        native::RuntimeFactLimits {
            max_retained_bytes: cap,
            ..Default::default()
        }
    );
    assert!(settings.development_measurement().is_none());
    let data = Arc::new(
        GameDataBuilder::from_validated(&prepared.game_data)
            .build_with_runtime_safety(settings)
            .unwrap(),
    );
    // Setup grants use default settings. The tested Core is constructed with
    // immutable host limits before receiving this synthetic prepared state.
    let mut core = GameCore::new(data, prepared.run_seed);
    assert_eq!(core.game_data.identity(), prepared.game_data.identity());
    core.state = prepared.state;
    assert_eq!(roots(&core), before);
    (core, player, employees[0], copy)
}

#[test]
fn corrupt_relic_retained_refusals_preserve_copy_catalyst_and_all_roots() {
    let (mut baseline, player, employee, copy) =
        corruption_host_fixture(native::RuntimeFactLimits::default().max_retained_bytes);
    let seed = baseline.state.relic_owned_extent().unwrap();
    assert!(seed > 0);
    assert!(!native::has_operation_scope());
    let (result, admissions) = native::observe_retained_admissions(|| {
        baseline.execute(player, PlayerBehavior::CorruptRelic { copy_uuid: copy })
    });
    result.unwrap();
    assert!(!native::has_operation_scope());
    assert_eq!(
        baseline.state.relics.get(copy).unwrap().variant,
        RelicVariant::Corrupt
    );
    assert_eq!(
        baseline.state.relics.employee_slots(employee),
        [None, None, Some(copy)]
    );
    assert_eq!(
        baseline.state.crafting_resources.relic_corruption_catalysts,
        0
    );
    assert!(admissions.windows(2).all(|rows| rows[0].2 <= rows[1].1));
    println!("U08_CORRUPT_HOST_ADMISSIONS seed={seed} admissions={admissions:?}");

    // These two fixed sites allocate the response only after the candidate
    // catalyst debit and variant write. Observations locate byte boundaries.
    let mut cases = vec![("retained_operation_start", seed - 1, seed, 0)];
    for site in ["relic_query_snapshot", "relic_not_sellable_projection"] {
        let index = admissions
            .iter()
            .position(|row| row.0 == site && row.2 > row.1)
            .unwrap_or_else(|| panic!("missing post-mutation response admission {site}"));
        cases.push((site, admissions[index].2 - 1, admissions[index].2, index));
    }
    for (site, cap, requested, prefix_len) in cases {
        let (mut core, player, employee, copy) = corruption_host_fixture(cap);
        let before = roots(&core);
        for _ in 0..2 {
            assert!(!native::has_operation_scope());
            let (result, prefix) = native::observe_retained_admissions(|| {
                core.execute(player, PlayerBehavior::CorruptRelic { copy_uuid: copy })
            });
            assert!(!native::has_operation_scope());
            let GameError::BattleRuntimeFault(fault) = result.unwrap_err() else {
                panic!("expected host retained-byte refusal at {site}");
            };
            assert_eq!(fault.code, native::RuntimeFaultCode::RetainedByteLimit);
            assert_eq!(fault.site, site);
            assert_eq!(fault.limit, Some(cap as u64));
            assert_eq!(fault.requested, Some(requested as u64));
            assert_eq!(prefix, admissions[..prefix_len]);
            assert_eq!(roots(&core), before, "{site}");
            assert_eq!(
                core.state.relics.get(copy).unwrap().variant,
                RelicVariant::Normal
            );
            assert_eq!(
                core.state.relics.employee_slots(employee),
                [None, None, Some(copy)]
            );
            assert_eq!(core.state.crafting_resources.relic_corruption_catalysts, 1);
        }
    }
    let exact_cap = admissions.last().unwrap().2;
    let (mut exact, player, employee, copy) = corruption_host_fixture(exact_cap);
    exact
        .execute(player, PlayerBehavior::CorruptRelic { copy_uuid: copy })
        .unwrap();
    assert!(!native::has_operation_scope());
    assert_eq!(
        exact.state.relics.get(copy).unwrap().variant,
        RelicVariant::Corrupt
    );
    assert_eq!(
        exact.state.relics.employee_slots(employee),
        [None, None, Some(copy)]
    );
    assert_eq!(exact.state.crafting_resources.relic_corruption_catalysts, 0);
    assert_eq!(roots(&exact), roots(&baseline));
}

#[test]
fn corrupt_relic_requires_its_catalyst_and_supported_normal_variant() {
    let (mut core, player, employees) = setup();
    let copy = grant(&mut core, 1)[0];
    core.execute(
        player,
        PlayerBehavior::EquipRelic {
            copy_uuid: copy,
            employee_uuid: employees[0],
            slot: 1,
        },
    )
    .unwrap();
    for command in [
        AdminCommand::AdminSetEnkephalin { amount: 1000 },
        AdminCommand::AdminGrantEquipmentDust { amount: 1000 },
        AdminCommand::AdminGrantFragmentDust { amount: 1000 },
    ] {
        core.execute_admin_command(command).unwrap();
    }
    assert_eq!(core.state.crafting_resources.relic_corruption_catalysts, 0);
    let before = roots(&core);
    assert!(matches!(
        core.execute(player, PlayerBehavior::CorruptRelic { copy_uuid: copy }),
        Err(GameError::InvalidAction)
    ));
    assert_eq!(roots(&core), before);
    core.execute_admin_command(AdminCommand::AdminGrantRelicCorruptionCatalyst { amount: 1 })
        .unwrap();
    let other_assets = json!([
        core.state.enkephalin,
        core.state.crafting_resources.equipment_dust,
        core.state.skill_fragments.fragment_dust()
    ]);
    core.execute(player, PlayerBehavior::CorruptRelic { copy_uuid: copy })
        .unwrap();
    assert_eq!(
        core.state.relics.get(copy).unwrap().variant,
        RelicVariant::Corrupt
    );
    assert_eq!(
        core.state.relics.employee_slots(employees[0]),
        [None, Some(copy), None]
    );
    assert_eq!(core.state.crafting_resources.relic_corruption_catalysts, 0);
    assert_eq!(
        json!([
            core.state.enkephalin,
            core.state.crafting_resources.equipment_dust,
            core.state.skill_fragments.fragment_dust()
        ]),
        other_assets
    );
    core.execute_admin_command(AdminCommand::AdminGrantRelicCorruptionCatalyst { amount: 1 })
        .unwrap();
    let missing = Uuid::from_u128(0xBAD_808);
    assert!(core.state.relics.get(missing).is_none());
    for invalid_copy in [copy, missing] {
        for behavior in [
            PlayerBehavior::PreviewRelicCorruption {
                copy_uuid: invalid_copy,
            },
            PlayerBehavior::CorruptRelic {
                copy_uuid: invalid_copy,
            },
        ] {
            let expected_route =
                if matches!(behavior, PlayerBehavior::PreviewRelicCorruption { .. }) {
                    crate::game::behavior::ActionRoute::CoreCommon
                } else {
                    crate::game::behavior::ActionRoute::CoreDomain
                };
            assert_eq!(behavior.kind().route(), expected_route);
            // The query's common gate accepts this loadout context and
            // nonempty inventory. Both domain paths reject the target.
            assert!(core.is_loadout_action_context() && !core.state.relics.is_empty());
            let before = roots(&core);
            assert!(matches!(
                core.execute(player, behavior),
                Err(GameError::InvalidAction)
            ));
            assert_eq!(roots(&core), before);
        }
    }

    // The current catalog has no revival catalyst; do not invent a second
    // asset to claim U17's future cross-catalyst compatibility boundary.
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/u08/relics.ron"
    ));
    let mut relics: RelicDatabase = ron::de::from_str(source).unwrap();
    relics.relics[0].corruption = None;
    let data = GameDataBuilder::from_validated(&core.game_data)
        .with_relics(relics)
        .build_arc();
    let mut unsupported = GameCore::new(data, 0x808);
    start_new_game_with_default_starters(&mut unsupported, player);
    let copy = grant(&mut unsupported, 1)[0];
    unsupported
        .execute_admin_command(AdminCommand::AdminGrantRelicCorruptionCatalyst { amount: 1 })
        .unwrap();
    for behavior in [
        PlayerBehavior::PreviewRelicCorruption { copy_uuid: copy },
        PlayerBehavior::CorruptRelic { copy_uuid: copy },
    ] {
        let before = roots(&unsupported);
        assert!(matches!(
            unsupported.execute(player, behavior),
            Err(GameError::InvalidAction)
        ));
        assert_eq!(roots(&unsupported), before);
    }
    assert_eq!(
        unsupported.state.relics.get(copy).unwrap().variant,
        RelicVariant::Normal
    );
    assert_eq!(
        unsupported
            .state
            .crafting_resources
            .relic_corruption_catalysts,
        1
    );
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
fn typed_refusal(error: GameError, cap: usize) {
    match error {
        GameError::BattleRuntimeFault(fault) => {
            assert_eq!(fault.code, native::RuntimeFaultCode::RetainedByteLimit);
            assert_eq!(fault.limit, Some(cap as u64));
            assert!(fault.requested.unwrap() > cap as u64);
        }
        other => panic!("expected native retained refusal, got {other:?}"),
    }
}

#[test]
fn normal_grant_and_three_slot_transfer_use_current_variant_duplicate_policy() {
    let (mut core, player, employees) = setup();
    let copies = grant(&mut core, 2);
    core.execute(
        player,
        PlayerBehavior::EquipRelic {
            copy_uuid: copies[0],
            employee_uuid: employees[0],
            slot: 0,
        },
    )
    .unwrap();
    core.execute_admin_command(AdminCommand::AdminGrantRelicCorruptionCatalyst { amount: 1 })
        .unwrap();
    core.execute(
        player,
        PlayerBehavior::CorruptRelic {
            copy_uuid: copies[1],
        },
    )
    .unwrap();
    core.execute(
        player,
        PlayerBehavior::EquipRelic {
            copy_uuid: copies[1],
            employee_uuid: employees[1],
            slot: 2,
        },
    )
    .unwrap();
    let before = roots(&core);
    assert!(core
        .execute(
            player,
            PlayerBehavior::EquipRelic {
                copy_uuid: copies[1],
                employee_uuid: employees[0],
                slot: 1
            }
        )
        .is_err());
    assert_eq!(roots(&core), before);
    assert_eq!(
        core.state.relics.employee_slots(employees[1]),
        [None, None, Some(copies[1])]
    );
    assert_eq!(
        core.state.relics.get(copies[1]).unwrap().variant,
        RelicVariant::Corrupt
    );
    assert_eq!(core.state.crafting_resources.relic_corruption_catalysts, 0);
    assert!(matches!(
        core.execute(player, PlayerBehavior::RequestRelicState)
            .unwrap(),
        BehaviorResult::RelicState { .. }
    ));
}

#[test]
fn corruption_preview_matches_current_dismantle_variant_and_cancel_preserves_every_root() {
    let (mut core, player, employees) = setup();
    let copy = grant(&mut core, 1)[0];
    core.execute(
        player,
        PlayerBehavior::EquipRelic {
            copy_uuid: copy,
            employee_uuid: employees[0],
            slot: 2,
        },
    )
    .unwrap();
    core.execute_admin_command(AdminCommand::AdminGrantRelicCorruptionCatalyst { amount: 1 })
        .unwrap();
    let preview = match core
        .execute(
            player,
            PlayerBehavior::PreviewRelicCorruption { copy_uuid: copy },
        )
        .unwrap()
    {
        BehaviorResult::RelicPreview { result } => result,
        _ => panic!(),
    };
    assert_eq!(preview.catalyst_cost, 1);
    assert_eq!(preview.effects[0].flat, 40);
    core.execute(player, PlayerBehavior::CorruptRelic { copy_uuid: copy })
        .unwrap();
    let before_repeat = roots(&core);
    assert!(core
        .execute(player, PlayerBehavior::CorruptRelic { copy_uuid: copy })
        .is_err());
    assert_eq!(roots(&core), before_repeat);
    maintenance(&mut core, player);
    let dismantle = match core
        .execute(
            player,
            PlayerBehavior::PreviewRelicDismantle { copy_uuid: copy },
        )
        .unwrap()
    {
        BehaviorResult::RelicPreview { result } => result,
        _ => panic!(),
    };
    assert!(matches!(dismantle.operation, RelicOperationKind::Dismantle));
    assert_eq!(dismantle.effects, preview.effects);
    assert_eq!(dismantle.penalties, preview.penalties);
    let before = roots(&core);
    core.execute(
        player,
        PlayerBehavior::DismantleRelic {
            copy_uuid: copy,
            confirmed: false,
        },
    )
    .unwrap();
    assert_eq!(roots(&core), before);
    core.execute(
        player,
        PlayerBehavior::DismantleRelic {
            copy_uuid: copy,
            confirmed: true,
        },
    )
    .unwrap();
    assert!(core.state.relics.get(copy).is_none());
    assert_eq!(core.state.relics.employee_slots(employees[0]), [None; 3]);
    assert_eq!(core.state.crafting_resources.equipment_dust, 7);
    assert_eq!(core.state.skill_fragments.fragment_dust(), 0);
}

#[test]
fn overflowing_authored_dust_reward_refuses_without_removing_equipped_copy() {
    let (mut core, player, employees) = setup();
    let copy = grant(&mut core, 1)[0];
    core.execute(
        player,
        PlayerBehavior::EquipRelic {
            copy_uuid: copy,
            employee_uuid: employees[0],
            slot: 1,
        },
    )
    .unwrap();
    core.execute_admin_command(AdminCommand::AdminGrantEquipmentDust { amount: u32::MAX })
        .unwrap();
    maintenance(&mut core, player);
    let before = roots(&core);
    assert!(core
        .execute(
            player,
            PlayerBehavior::DismantleRelic {
                copy_uuid: copy,
                confirmed: true
            }
        )
        .is_err());
    assert_eq!(roots(&core), before);
}

#[test]
fn empty_first_grant_is_cumulative_and_exact_or_one_short_retained_refusal_is_atomic() {
    let (mut probe, _, _) = setup();
    let (_, admissions) = native::observe_retained_admissions(|| grant(&mut probe, 2));
    assert!(
        admissions
            .iter()
            .filter(|r| r.0 == "relic_copy_grant")
            .count()
            >= 2
    );
    let cap = admissions.last().unwrap().2;
    println!("first grant cumulative cap={cap}; admissions={admissions:?}");
    for (limit, succeeds) in [(cap, true), (cap - 1, false)] {
        let (mut core, _, _) = setup();
        let before = roots(&core);
        let result = native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: limit,
                ..Default::default()
            },
            || {
                core.execute_admin_command(AdminCommand::AdminGrantRelic {
                    definition_id: "u08_probe".into(),
                    count: 2,
                })
            },
        );
        if succeeds {
            result.unwrap();
            assert_eq!(core.state.relics.copies().count(), 2);
        } else {
            typed_refusal(result.unwrap_err(), limit);
            assert_eq!(roots(&core), before);
        }
    }
}

#[test]
fn following_operation_seeds_owned_current_checkpoint_and_continue_before_new_allocation() {
    let (mut core, _, _) = setup();
    grant(&mut core, 1);
    let extent = core.state.relic_owned_extent().unwrap();
    assert!(extent > 0);
    let (result, admissions) = native::observe_retained_admissions(|| {
        core.execute_admin_command(AdminCommand::AdminGrantRelicCorruptionCatalyst { amount: 1 })
    });
    result.unwrap();
    assert!(!admissions.is_empty());
    assert_eq!(admissions[0].1, extent);
    println!(
        "next operation owned seed={extent}; first={:?}",
        admissions[0]
    );
}

#[test]
fn authored_zero_dismantle_reward_is_rejected_before_catalog_publication() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/u08/relics.ron"
    ))
    .replace("dismantle_equipment_dust: 7", "dismantle_equipment_dust: 0");
    let database: RelicDatabase = ron::de::from_str(&source).unwrap();
    let error = database.validate_contract().unwrap_err();
    assert!(error.contains("relics[0].dismantle_equipment_dust"));
}

#[test]
fn durable_continue_and_savepoint_keep_independent_variant_cost_and_slot_owners() {
    let (mut core, player, employees) = setup();
    let copy = grant(&mut core, 1)[0];
    core.execute(
        player,
        PlayerBehavior::EquipRelic {
            copy_uuid: copy,
            employee_uuid: employees[0],
            slot: 2,
        },
    )
    .unwrap();
    core.execute_admin_command(AdminCommand::AdminGrantRelicCorruptionCatalyst { amount: 1 })
        .unwrap();
    core.save_run_checkpoint_for_test().unwrap();
    core.execute(player, PlayerBehavior::CorruptRelic { copy_uuid: copy })
        .unwrap();
    let export = core.export_active_run_checkpoint().unwrap().unwrap();
    let wire: Value = serde_json::from_slice(&export.core_payload_json).unwrap();
    assert_eq!(wire["version"], 12);
    assert!(wire["continuation"]["current"]["relics"].is_object());
    assert!(wire["continuation"]["savepoint"]["snapshot"]["checkpoint"]["relics"].is_object());
    let mut restarted = core_with_profile_for_continue(&core, 0xDEAD);
    let (result, admissions) = native::observe_retained_admissions(|| {
        restarted.continue_from_active_run_checkpoint(
            player,
            &export.core_payload_json,
            export.loads_used,
            export.run_peak_certified_power,
        )
    });
    result.unwrap();
    assert!(admissions.iter().any(|row| row.0 == "relic_continue_value"));
    assert!(admissions
        .iter()
        .any(|row| row.0 == "relic_continue_decode_savepoint_arc"));
    assert_eq!(
        restarted.state.relics.get(copy).unwrap().variant,
        RelicVariant::Corrupt
    );
    assert_eq!(
        restarted.state.relics.employee_slots(employees[0]),
        [None, None, Some(copy)]
    );
    assert_eq!(
        restarted
            .state
            .crafting_resources
            .relic_corruption_catalysts,
        0
    );
    restarted
        .execute(player, PlayerBehavior::LoadRunCheckpoint)
        .unwrap();
    assert_eq!(
        restarted.state.relics.get(copy).unwrap().variant,
        RelicVariant::Normal
    );
    assert_eq!(
        restarted
            .state
            .crafting_resources
            .relic_corruption_catalysts,
        1
    );
    assert_eq!(
        restarted.state.relics.employee_slots(employees[0]),
        [None, None, Some(copy)]
    );
    println!("Continue admissions={admissions:?}");
}

#[test]
fn durable_empty_fields_preserve_old_codec_and_present_malformed_forgery_or_generation_fail_atomically(
) {
    let (mut core, player, _) = setup();
    let old = core.export_active_run_checkpoint().unwrap().unwrap();
    let empty: Value = serde_json::from_slice(&old.core_payload_json).unwrap();
    assert!(empty["continuation"]["current"].get("relics").is_none());
    assert!(empty["continuation"]["current"]
        .get("crafting_resources")
        .is_none());
    let mut restored = core_with_profile_for_continue(&core, 0xDEAD);
    restored
        .continue_from_active_run_checkpoint(
            player,
            &old.core_payload_json,
            0,
            old.run_peak_certified_power,
        )
        .unwrap();
    assert_eq!(restored.get_state(), GameState::ViewingMap);
    grant(&mut core, 1);
    let new = core.export_active_run_checkpoint().unwrap().unwrap();
    let original: Value = serde_json::from_slice(&new.core_payload_json).unwrap();
    for case in ["malformed", "forgery", "generation"] {
        let mut wire = original.clone();
        match case {
            "malformed" => wire["continuation"]["current"]["relics"] = json!(false),
            "forgery" => wire["continuation_sha256"] = json!("0".repeat(64)),
            "generation" => {
                wire["continuation"]["current"]["content_identity"]["semantic_sha256"] =
                    json!("0".repeat(64));
                refresh_active_run_checksum(&mut wire);
            }
            _ => unreachable!(),
        }
        let mut fresh = GameCore::new(core.game_data.clone(), 0xDEAD);
        let before = roots(&fresh);
        let error = fresh
            .continue_from_active_run_checkpoint(
                player,
                &serde_json::to_vec(&wire).unwrap(),
                0,
                new.run_peak_certified_power,
            )
            .unwrap_err();
        assert!(
            matches!(error, GameError::InvalidStaticData(_)),
            "{case}: {error:?}"
        );
        assert_eq!(roots(&fresh), before, "{case}");
        assert!(fresh.state.run.is_none());
        assert!(fresh.state.player.is_none());
    }
}

#[test]
fn uuid_stream_remains_an_owned_root_after_last_copy_is_dismantled() {
    let (mut core, player, _) = setup();
    let copy = grant(&mut core, 1)[0];
    maintenance(&mut core, player);
    core.execute(
        player,
        PlayerBehavior::DismantleRelic {
            copy_uuid: copy,
            confirmed: true,
        },
    )
    .unwrap();
    assert!(core.state.relics.is_empty());
    assert!(core.state.uuid_manager.has_relic_stream());
    assert!(core.state.has_relic_owner());
    assert_eq!(core.state.uuid_manager.relic_stream_extent().unwrap(), 400);
    let unfinished = core.export_active_run_checkpoint().unwrap().unwrap();
    let mut pre_entry = core_with_profile_for_continue(&core, 0xDEAD);
    pre_entry
        .continue_from_active_run_checkpoint(
            player,
            &unfinished.core_payload_json,
            unfinished.loads_used,
            unfinished.run_peak_certified_power,
        )
        .unwrap();
    assert_eq!(pre_entry.state.crafting_resources.equipment_dust, 0);
    assert!(pre_entry.state.relics.get(copy).is_some());
    core.execute(player, PlayerBehavior::CompleteNode).unwrap();
    let export = core.export_active_run_checkpoint().unwrap().unwrap();
    let mut restored = core_with_profile_for_continue(&core, 0xDEAD);
    restored
        .continue_from_active_run_checkpoint(
            player,
            &export.core_payload_json,
            export.loads_used,
            export.run_peak_certified_power,
        )
        .unwrap();
    assert!(restored.state.uuid_manager.has_relic_stream());
    assert!(restored.state.has_relic_owner());
    assert_eq!(restored.state.crafting_resources.equipment_dust, 7);
}

#[test]
fn nonbattle_relic_owner_registers_initial_battle_once_before_startup_growth() {
    let (mut core, player, _) = setup();
    grant(&mut core, 1);
    let node = force_first_available_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        MapNodePayload::Encounter {
            encounter_id: Some("suppress_burrowing_heaven".into()),
        },
    );
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    assert!(core.state.active_battle.is_none());
    let (result, admissions) = native::observe_retained_admissions(|| {
        core.execute(player, PlayerBehavior::ConfirmEnterNode)
    });
    result.unwrap();
    let registrations = admissions
        .iter()
        .enumerate()
        .filter(|(_, r)| r.0 == "relic_battle_initial_owner")
        .collect::<Vec<_>>();
    assert_eq!(registrations.len(), 1);
    let (index, registration) = registrations[0];
    assert!(registration.1 > 0);
    assert!(registration.2 > registration.1);
    assert!(admissions[index + 1..]
        .iter()
        .any(|r| r.0 != "relic_battle_initial_owner" && r.2 > r.1));
    assert!(core.state.active_battle.is_some());
    println!(
        "nonbattle→battle initial registration={registration:?}; following growth={:?}",
        &admissions[index + 1..]
    );
}

#[cfg(feature = "runtime-observation")]
#[test]
fn relic_observation_reports_committed_owner_and_discards_candidate_without_mutating_session() {
    use crate::application::{
        ApplicationAdminCommand, ApplicationCommand, ApplicationExecutionContext, CoreApplication,
    };
    use crate::runtime_observation::{self as observation, NativeOwnerExtent, Outcome};
    let (core, player, employees) = setup();
    let copy = core
        .state
        .uuid_manager
        .peek(crate::game::managers::uuid_manager::UuidManager::NS_OWNED_RELIC);
    let mut app = CoreApplication::from_core(core);
    let (sender, receiver) = observation::channel();
    let (observer, reader) = sender.session(808);
    app.set_runtime_observer(Some(observer));
    app.execute_admin(
        ApplicationAdminCommand::from_document(
            json!({"type":"admin_grant_relic","definition_id":"u08_probe","count":1}).into(),
        )
        .unwrap(),
    )
    .unwrap();
    let accepted = receiver.try_recv().unwrap();
    assert_eq!(accepted.outcome, Outcome::Accepted);
    let current = accepted.committed.unwrap();
    assert!(matches!(current.owner_extent, NativeOwnerExtent::Relics { bytes } if bytes > 0));
    assert!(accepted.retained.unwrap().accepted_total_bytes > 0);
    assert!(accepted.max_relic_owner_extent_bytes.unwrap() > 0);
    assert_eq!(
        current.usage,
        observation::Usage::default(),
        "Usage stays battle-scoped"
    );
    let before = reader.snapshot(false).unwrap();
    let command = || {
        ApplicationCommand::from_document(
            json!({"type":"equip_relic","copy_uuid":copy,"employee_uuid":employees[0],"slot":2})
                .into(),
        )
        .unwrap()
    };
    let prepared = app
        .prepare_command(player, command(), ApplicationExecutionContext::default())
        .unwrap();
    drop(prepared);
    let discarded = receiver.try_recv().unwrap();
    assert_eq!(discarded.outcome, Outcome::Discarded);
    assert!(
        matches!(discarded.prepared.unwrap().owner_extent, NativeOwnerExtent::Relics { bytes } if bytes > 0)
    );
    assert!(discarded.committed.is_none());
    let after = reader.snapshot(false).unwrap();
    assert_eq!(after.current, before.current);
    assert_eq!(after.commits, before.commits);
    let failed = app.execute(
        player,
        ApplicationCommand::from_document(json!({"type":"corrupt_relic","copy_uuid":copy}).into())
            .unwrap(),
        ApplicationExecutionContext::default(),
    );
    assert!(failed.is_err());
    let rejected = receiver.try_recv().unwrap();
    assert_eq!(rejected.outcome, Outcome::Rejected);
    assert!(rejected.committed.is_none());
    assert_eq!(reader.snapshot(false).unwrap().current, before.current);
    app.prepare_command(player, command(), ApplicationExecutionContext::default())
        .unwrap()
        .commit();
    let committed = receiver.try_recv().unwrap();
    assert_eq!(committed.outcome, Outcome::Accepted);
    assert!(
        matches!(committed.committed.unwrap().owner_extent, NativeOwnerExtent::Relics { bytes } if bytes > 0)
    );
    assert_eq!(reader.snapshot(false).unwrap().commits, before.commits + 1);
    println!(
        "relic observed accepted_total={} max_extent={} owner={:?}",
        accepted.retained.unwrap().accepted_total_bytes,
        accepted.max_relic_owner_extent_bytes.unwrap(),
        current.owner_extent
    );
}

#[test]
fn empty_relic_roots_add_no_clone_meter_slots_to_existing_world_captures() {
    use crate::authoring::{clone_shape::CloneShape, control::CloneMeter};
    let mut meter = CloneMeter::default();
    crate::game::relic::RelicInventory::default()
        .visit_clone_shape(&mut meter)
        .unwrap();
    crate::game::relic::RunCraftingResources::default()
        .visit_clone_shape(&mut meter)
        .unwrap();
    assert_eq!(meter.native_scan_slots(), 0);
    assert_eq!(meter.native_copy_slots(), Some(0));
}
