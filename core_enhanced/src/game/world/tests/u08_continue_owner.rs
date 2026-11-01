//! Typed internal fixture for independent current/SavePoint owners and the two
//! unordered checkpoint collections. Normal authored grants and fresh-process
//! restore are covered separately by the external U08 trial.
use super::*;
use crate::application::{ApplicationCheckpoint, ApplicationSnapshotKind, CoreApplication};
use crate::game::battle::runtime_control as native;

fn fixture() -> (GameCore, Uuid) {
    let (base, _, _) = super::u08_relics::setup();
    let mut triggers = base.game_data.visual_novel_trigger_data.triggers.clone();
    for id in ["u08_continue_rewind_a", "u08_continue_rewind_b"] {
        let mut trigger = triggers[0].clone();
        trigger.id = VisualNovelTriggerId::new(id);
        trigger.source = VisualNovelTriggerSource::MapNodeEntered {
            category: MapNodeCategory::SavePoint,
            kind_id: MapNodeKindId::new("save_point"),
        };
        trigger.repeat = VisualNovelTriggerRepeat::OncePerRun;
        trigger.checkpoint_policy = VisualNovelTriggerCheckpointPolicy::RewindWithRunCheckpoint;
        triggers.push(trigger);
    }
    let data = GameDataBuilder::from_validated(&base.game_data)
        .with_visual_novel_triggers(VisualNovelTriggerDatabase::new(triggers))
        .build_arc();
    let mut core = GameCore::new(data, 0xa508c017);
    let player = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player);
    let copy = super::u08_relics::grant(&mut core, 1)[0];
    let employee = core.state.roster.combat_eligible_employee_ids()[0];
    core.execute(
        player,
        PlayerBehavior::EquipRelic {
            copy_uuid: copy,
            employee_uuid: employee,
            slot: 2,
        },
    )
    .unwrap();

    // Explicit typed setup: both collections contain two valid identities in
    // both owners, so empty-set normalization cannot satisfy this test.
    for id in ["u08_continue_rewind_a", "u08_continue_rewind_b"] {
        core.state
            .visual_novel_triggers
            .checkpoint_rewind_consumed
            .insert(VisualNovelTriggerConsumedKey {
                trigger_id: VisualNovelTriggerId::new(id),
                scope: VisualNovelTriggerConsumedScope::Run,
            });
    }
    let shop = core
        .game_data
        .shop_data
        .shops
        .iter()
        .find(|shop| shop.context == ShopContext::UnknownExploration)
        .unwrap()
        .clone();
    let purchased_merchandise = core
        .game_data
        .equipment_data
        .items
        .iter()
        .take(2)
        .map(|item| ShopMerchandise::Equipment {
            equipment_id: item.id.clone(),
        })
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(purchased_merchandise.len(), 2);
    let node = force_first_available_node(
        &mut core,
        MapNodeCategory::Shop,
        "u08_continue_shop",
        MapNodePayload::Shop {
            shop_id: Some(shop.id.clone()),
            shop_pool_id: None,
        },
    );
    core.state.run.as_mut().unwrap().shop_sessions.insert(
        node,
        ShopSessionState {
            id: shop.id,
            name: shop.name,
            uuid: shop.uuid,
            context: shop.context,
            node_id: node,
            offer_count: shop.offer_count,
            refresh_policy: shop.refresh_policy,
            sell_policy: shop.sell_policy,
            refresh_uses: 0,
            page_index: 0,
            offers: Vec::new(),
            purchased_merchandise,
        },
    );
    core.save_run_checkpoint_for_test().unwrap();
    core.execute_admin_command(
        crate::game::world::AdminCommand::AdminGrantRelicCorruptionCatalyst { amount: 1 },
    )
    .unwrap();
    let export = core.export_active_run_checkpoint().unwrap().unwrap();
    let document: Value = serde_json::from_slice(&export.core_payload_json).unwrap();
    assert!(
        document["continuation"]["savepoint"]["snapshot"]["checkpoint"].is_object(),
        "distinct current and SavePoint must retain independent checkpoint owners"
    );
    (core, player)
}

fn app_roots(app: &CoreApplication) -> Value {
    let snapshots: Vec<_> = ApplicationSnapshotKind::ALL
        .iter()
        .map(|kind| app.snapshot(*kind).unwrap().read_model().as_value().clone())
        .collect();
    let checkpoint = app.export_active_run_checkpoint().unwrap().map(|saved| {
        json!({
            "bytes": saved.core_payload_json(), "loads": saved.loads_used(),
            "peak": saved.run_peak_certified_power(),
        })
    });
    json!({"snapshots": snapshots, "checkpoint": checkpoint, "resources": app.run_resources()})
}

fn reverse_sets(value: &mut Value, counts: &mut [usize; 2]) {
    match value {
        Value::Object(fields) => {
            for (key, child) in fields {
                let index = match key.as_str() {
                    "visual_novel_trigger_checkpoint_consumed" => Some(0),
                    "purchased_merchandise" => Some(1),
                    _ => None,
                };
                if let Some(index) = index {
                    let values = child.as_array_mut().unwrap();
                    assert_eq!(values.len(), 2);
                    assert_ne!(values[0], values[1]);
                    values.reverse();
                    counts[index] += 1;
                }
                reverse_sets(child, counts);
            }
        }
        Value::Array(values) => {
            for value in values {
                reverse_sets(value, counts);
            }
        }
        _ => {}
    }
}

#[test]
fn u08_continue_nonempty_sets_preserve_wire_identity_under_array_permutation() {
    let (core, player) = fixture();
    let generation = core.game_data.clone();
    let source = CoreApplication::from_core(core);
    let saved = source.export_active_run_checkpoint().unwrap().unwrap();
    let mut document: Value = serde_json::from_slice(saved.core_payload_json()).unwrap();
    let checksum = document["continuation_sha256"].clone();
    let mut counts = [0, 0];
    reverse_sets(&mut document, &mut counts);
    assert_eq!(
        counts,
        [2, 2],
        "current and independent SavePoint each own both sets"
    );
    assert_eq!(document["continuation_sha256"], checksum);
    let reversed = serde_json::to_vec(&document).unwrap();
    assert_ne!(reversed, saved.core_payload_json());
    let checkpoint = ApplicationCheckpoint::new(
        reversed,
        saved.loads_used(),
        saved.run_peak_certified_power(),
    );
    let input = checkpoint.core_payload_json().to_vec();
    let mut restored = CoreApplication::new(generation, 0xdead);
    install_application_profile_for_continue(&mut restored, &source);
    let (result, admissions) = native::observe_retained_admissions(|| {
        restored.continue_from_active_run_checkpoint(player, &checkpoint)
    });
    result.unwrap();
    assert_eq!(checkpoint.core_payload_json(), input);
    assert_eq!(
        restored.export_active_run_checkpoint().unwrap().unwrap(),
        saved
    );
    assert_eq!(app_roots(&restored), app_roots(&source));
    let normalization: Vec<_> = admissions
        .iter()
        .filter(|row| row.0 == "relic_continue_normalization")
        .collect();
    assert!(
        normalization.len() == 4,
        "the verified wire normalizes four nonempty sets once and is reused: {admissions:?}"
    );
    assert!(normalization.iter().all(|row| row.2 > row.1));
    println!("U08_CONTINUE_SET_PERMUTATION counts={counts:?} normalization={normalization:?}");
}

fn destination(generation: crate::game::data::ValidatedGameData, source: &CoreApplication) -> CoreApplication {
    let mut app = CoreApplication::new(generation, 0xbeef);
    let mut profile: crate::game::encounter_compendium::PlayerProfileState = serde_json::from_value(
        source.snapshot(crate::application::ApplicationSnapshotKind::PlayerProfile).unwrap().read_model().as_value().clone()
    ).unwrap();
    profile.highest_floor_reached = 99;
    profile.longest_deathless_floor_streak = 17;
    app.install_authoritative_player_profile(serde_json::to_value(profile).unwrap().into())
        .unwrap();
    app
}

#[test]
fn u08_continue_stages_are_cumulative_and_one_short_refusal_is_atomic() {
    let (core, player) = fixture();
    let generation = core.game_data.clone();
    let source = CoreApplication::from_core(core);
    let saved = source.export_active_run_checkpoint().unwrap().unwrap();
    let input = saved.core_payload_json().to_vec();
    let mut probe = destination(generation.clone(), &source);
    // An outer test allowance includes the complete public call. As with the
    // existing query-owner probes, its typed sticky fault is the outer result.
    let (result, admissions) = native::observe_retained_admissions(|| {
        native::run(Default::default(), || {
            Ok(probe.continue_from_active_run_checkpoint(player, &saved))
        })
    });
    result.unwrap().unwrap();
    let expected = app_roots(&probe);
    assert!(
        admissions.windows(2).all(|rows| rows[1].1 == rows[0].2),
        "independent producer charges must accumulate without refunds: {admissions:?}"
    );
    let values: Vec<_> = admissions
        .iter()
        .enumerate()
        .filter(|(_, row)| matches!(row.0, "continue_typed_decode" | "relic_continue_value"))
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        values.len(),
        2,
        "typed decode and one normalized Value reused as the confirmed cache: {admissions:?}"
    );
    let decode_arc = admissions
        .iter()
        .position(|row| row.0 == "relic_continue_decode_savepoint_arc")
        .unwrap();
    assert!(values[0] < decode_arc && decode_arc < values[1]);
    let normalization = admissions
        .iter()
        .position(|row| row.0 == "relic_continue_normalization")
        .unwrap();
    assert!(values[1] < normalization);
    let encoded = admissions.iter().position(|row| row.0 == "relic_continue_bytes").unwrap();
    assert!(normalization < encoded);
    let cap = admissions.last().unwrap().2;
    assert!(cap <= native::RuntimeFactLimits::default().max_retained_bytes);
    println!("U08_CONTINUE_CUMULATIVE cap={cap} values={values:?} admissions={admissions:?}");

    // Each selected refusal is one byte below an observed producer boundary;
    // this tests enforcement/order, independently of the allocation formula.
    let mut boundaries = vec![
        values[0],
        decode_arc,
        values[1],
        normalization,
        encoded,
        admissions.len() - 1,
    ];
    boundaries.sort_unstable();
    boundaries.dedup();
    for index in boundaries {
        let (site, _, after) = admissions[index];
        let limit = after - 1;
        let mut app = destination(generation.clone(), &source);
        let before = app_roots(&app);
        let (result, refused_admissions) = native::observe_retained_admissions(|| {
            native::run(
                native::RuntimeFactLimits {
                    max_retained_bytes: limit,
                    ..Default::default()
                },
                || Ok(app.continue_from_active_run_checkpoint(player, &saved)),
            )
        });
        let GameError::BattleRuntimeFault(fault) = result.unwrap_err() else {
            panic!("expected typed native retained refusal at {site}")
        };
        assert_eq!(fault.code, native::RuntimeFaultCode::RetainedByteLimit);
        assert_eq!(fault.site, site);
        assert_eq!(fault.limit, Some(limit as u64));
        assert_eq!(fault.requested, Some(after as u64));
        assert_eq!(refused_admissions, admissions[..index]);
        assert_eq!(
            app_roots(&app),
            before,
            "{site} must not publish partial gameplay roots"
        );
        assert_eq!(saved.core_payload_json(), input);
    }
    let mut exact = destination(generation, &source);
    native::run(
        native::RuntimeFactLimits {
            max_retained_bytes: cap,
            ..Default::default()
        },
        || Ok(exact.continue_from_active_run_checkpoint(player, &saved)),
    )
    .unwrap()
    .unwrap();
    assert_eq!(app_roots(&exact), expected);
    assert_eq!(saved.core_payload_json(), input);
}
