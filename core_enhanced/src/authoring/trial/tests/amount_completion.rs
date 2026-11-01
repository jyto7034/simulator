use super::*;
use crate::authoring::{AmountBattle, AmountQuery, BattleKey, TraceLimits, TraceOptions};
use uuid::Uuid;

fn r06_held_public_run() -> (ExternalFixture, ValidatedGameData, TrialSession) {
    let fixture = ExternalFixture::official_with_trial_route();
    let cancel = CancellationToken::default();
    let content = crate::authoring::validate(
        fixture.root(),
        &fixture.selection(),
        &Default::default(),
        &cancel,
    )
    .unwrap();
    let token = content.token().clone();
    let document = preparation_document(&token);
    let session =
        TrialSession::prepare(content, checked(&document), TrialLimits::default(), &cancel)
            .unwrap();
    assert_eq!(session.next_step_index(), 9);
    // The public iterator includes initial and ready receipts around the nine steps.
    assert_eq!(session.receipt_documents().count(), 11);
    (fixture, token, session)
}

fn r06_held_public_state(token: &ValidatedGameData, session: &TrialSession) -> Value {
    json!({
        "generation": token.identity(),
        "source_receipt": token.source_receipt(),
        "session_generation": session.content_identity(),
        "run": session.snapshot(ApplicationSnapshotKind::Run, &Default::default(), &CancellationToken::default()).unwrap().document(),
        "work": session.accepted_work(),
        "next": session.next_step_index(),
        "receipts": session.receipt_documents().collect::<Vec<_>>()
    })
}

fn r06_public_aggregate(units: [&str; 2], reduction: &str, member: &str) -> String {
    format!(
        "Aggregate(source: AuthoredMembers([(identity: \"a\", value: (value: 1, unit: {}), owner: None, projected_tile: None), (identity: \"b\", value: (value: 2, unit: {}), owner: None, projected_tile: None)]), grouping: WholeSet, reduction: {reduction}, outer_reduction: None, member: {member})",
        units[0], units[1]
    )
}

fn r06_write_public_amount(fixture: &ExternalFixture, original: &str, root: &str) {
    let literal = "Damage(amount: 60, damage_type: Magic)";
    assert!(original.contains(literal));
    let replacement =
        format!("DamageExpr(amount: (moment: Live, root: {root}), damage_type: Magic)");
    let changed = original.replacen(literal, &replacement, 1);
    assert_ne!(changed, original);
    fixture.write("official/data/skills/base.ron", changed);
}

#[test]
fn r06_public_operator_units_reject_before_publication_and_preserve_held_run() {
    use crate::game::data::{content_pack::CatalogKind, live_ron::LiveRonPhase};
    let (fixture, token, session) = r06_held_public_run();
    let original = fixture.read("official/data/skills/base.ron");
    let authored: crate::game::data::skill_data::RawSkillDatabase =
        ron::de::from_str(&original).unwrap();
    assert_eq!(authored.skills[0].id.as_str(), "unknown_distortion_strike");
    let held = r06_held_public_state(&token, &session);
    let mut cases = vec![
        ("Add".to_owned(), "Add([Const((value: 2, unit: Magnitude)), Const((value: 3, unit: Milliseconds))])".to_owned()),
        ("Mul".to_owned(), "Mul(value: Const((value: 2, unit: Magnitude)), scalar: Const((value: 3, unit: Magnitude)))".to_owned()),
        ("Ratio".to_owned(), "Ratio(numerator: Const((value: 6, unit: Magnitude)), denominator: Const((value: 2, unit: Milliseconds)), rounding: TowardZero)".to_owned()),
        ("PointsMilliseconds".to_owned(), "Add([Const((value: 2, unit: AttackSpeedPoints)), Const((value: 3, unit: Milliseconds))])".to_owned()),
    ];
    for reduction in ["Sum", "Max"] {
        let aggregate = r06_public_aggregate(
            ["Magnitude", "Milliseconds"],
            reduction,
            "Read(AggregateMember(unit: Magnitude))",
        );
        cases.push((format!("staticMixed{reduction}"), aggregate.clone()));
        cases.push((
            format!("dynamicMixed{reduction}"),
            format!("Add([Read(Stat(subject: Source, stat: Attack)), {aggregate}])"),
        ));
    }
    assert_eq!(cases.len(), 8);
    for (name, root) in cases {
        r06_write_public_amount(&fixture, &original, &root);
        let error = crate::authoring::validate(
            fixture.root(),
            &fixture.selection(),
            &Default::default(),
            &CancellationToken::default(),
        )
        .err()
        .unwrap_or_else(|| panic!("{name} issued a validated content token"));
        let crate::authoring::AuthoringError::Content { diagnostic } = error else {
            panic!("{name}: {error:?}");
        };
        assert_eq!(
            diagnostic.code, "value_expr_unit_mismatch",
            "{name}: {diagnostic:?}"
        );
        assert_eq!(diagnostic.phase, LiveRonPhase::Compile);
        assert_eq!(diagnostic.source_file, "official/data/skills/base.ron");
        assert_eq!(diagnostic.pack_id.as_deref(), Some("official"));
        assert_eq!(diagnostic.catalog_kind, Some(CatalogKind::Skills));
        assert_eq!(
            diagnostic.object_id.as_deref(),
            Some("unknown_distortion_strike")
        );
        assert_eq!(diagnostic.action_id.as_deref(), Some("strike"));
        assert_eq!(diagnostic.effect_id.as_deref(), Some("effect_01"));
        assert_eq!(
            diagnostic.field_path.as_deref(),
            Some("skills[0].steps[0].effects[0].op")
        );
        assert!(diagnostic.expected.is_some());
        assert!(diagnostic.actual.is_some());
        assert_eq!(r06_held_public_state(&token, &session), held, "{name}");
        eprintln!(
            "R06_PUBLIC_UNIT {name} {}",
            serde_json::to_string(&diagnostic).unwrap()
        );
    }
}

#[test]
fn r06_public_authored_member_scope_preserves_valid_inputs() {
    let fixture = ExternalFixture::official_with_trial_route();
    let original = fixture.read("official/data/skills/base.ron");
    let mixed = ["Magnitude", "Milliseconds"];
    let child = "Aggregate(source: AuthoredMembers([(identity: \"child\", value: (value: 2, unit: Scalar), owner: None, projected_tile: None)]), grouping: WholeSet, reduction: Sum, outer_reduction: None, member: Ratio(numerator: Const((value: 8, unit: Magnitude)), denominator: Read(AggregateMember(unit: Scalar)), rounding: TowardZero))";
    let inner = "Aggregate(source: AuthoredMembers([(identity: \"inner\", value: (value: 3, unit: Magnitude), owner: None, projected_tile: None)]), grouping: WholeSet, reduction: Sum, outer_reduction: None, member: Read(AggregateMember(unit: Magnitude)))";
    let mut cases = vec![("unchanged".to_owned(), None)];
    for reduction in ["Sum", "Max"] {
        let aggregate = r06_public_aggregate(
            ["Magnitude", "Magnitude"],
            reduction,
            "Read(AggregateMember(unit: Magnitude))",
        );
        cases.push((
            format!("homogeneous{reduction}"),
            Some(format!(
                "Add([Read(Stat(subject: Source, stat: Attack)), {aggregate}])"
            )),
        ));
    }
    cases.extend([
        ("unusedConst".to_owned(), Some(r06_public_aggregate(mixed, "Sum", "Const((value: 7, unit: Magnitude))"))),
        ("unusedExternal".to_owned(), Some(r06_public_aggregate(mixed, "Sum", "Read(Stat(subject: Source, stat: Attack))"))),
        ("scalarReadMagnitudeResult".to_owned(), Some(child.to_owned())),
        ("nestedParentUnused".to_owned(), Some(r06_public_aggregate(mixed, "Sum", child))),
        ("nestedDirectDifferentUnits".to_owned(), Some(format!("Aggregate(source: AuthoredMembers([(identity: \"outer\", value: (value: 2, unit: Scalar), owner: None, projected_tile: None)]), grouping: WholeSet, reduction: Sum, outer_reduction: None, member: Mul(value: {inner}, scalar: Read(AggregateMember(unit: Scalar))))"))),
    ]);
    assert_eq!(cases.len(), 8);
    for (name, root) in cases {
        fixture.write("official/data/skills/base.ron", &original);
        if let Some(root) = root {
            r06_write_public_amount(&fixture, &original, &root);
        }
        let accepted = crate::authoring::validate(
            fixture.root(),
            &fixture.selection(),
            &Default::default(),
            &CancellationToken::default(),
        )
        .unwrap_or_else(|error| panic!("{name}: {error:?}"));
        eprintln!(
            "R06_PUBLIC_VALID {name} {}",
            serde_json::to_string(accepted.token().identity()).unwrap()
        );
    }
}

#[test]
fn r06_public_zero_points_and_weapon_interval_refuse_and_preserve_held_run() {
    use crate::game::data::{
        content_pack::CatalogKind, employee_data::EmployeeDefinitionDatabase,
        equipment_data::EquipmentDatabase, live_ron::LiveRonPhase,
    };
    let (fixture, token, session) = r06_held_public_run();
    let held = r06_held_public_state(&token, &session);
    let employee_path = "official/data/employees/definitions.ron";
    let equipment_path = "official/data/equipments/base.ron";
    let original_employee = fixture.read(employee_path);
    let original_equipment = fixture.read(equipment_path);
    let mut employees: EmployeeDefinitionDatabase = ron::de::from_str(&original_employee).unwrap();
    let mut equipment: EquipmentDatabase = ron::de::from_str(&original_equipment).unwrap();
    assert_eq!(
        employees.definitions[0].employee_definition_id,
        "field_medic"
    );
    assert_eq!(
        employees.definitions[0].profile.basic_attack_speed_percent,
        100
    );
    assert_eq!(equipment.items[0].id, "justitia");
    assert_eq!(
        equipment.items[0]
            .weapon_profile
            .as_ref()
            .unwrap()
            .interval_ms,
        1500
    );
    let employee_before = serde_json::to_value(&employees).unwrap();
    let equipment_before = serde_json::to_value(&equipment).unwrap();
    employees.definitions[0].profile.basic_attack_speed_percent = 0;
    equipment.items[0]
        .weapon_profile
        .as_mut()
        .unwrap()
        .interval_ms = 0;
    let mut restored_employee = employees.clone();
    restored_employee.definitions[0]
        .profile
        .basic_attack_speed_percent = 100;
    assert_eq!(
        serde_json::to_value(restored_employee).unwrap(),
        employee_before
    );
    let mut restored_equipment = equipment.clone();
    restored_equipment.items[0]
        .weapon_profile
        .as_mut()
        .unwrap()
        .interval_ms = 1500;
    assert_eq!(
        serde_json::to_value(restored_equipment).unwrap(),
        equipment_before
    );
    let cases = [
        (
            employee_path,
            ron::ser::to_string(&employees).unwrap(),
            CatalogKind::Employees,
            "field_medic",
            "definitions[0].profile.basic_attack_speed_percent",
            "basic_attack_speed_percent must be > 0",
        ),
        (
            equipment_path,
            ron::ser::to_string(&equipment).unwrap(),
            CatalogKind::Equipment,
            "justitia",
            "items[0].weapon_profile.interval_ms",
            "weapon 'justitia' weapon_profile interval_ms must be > 0",
        ),
    ];
    for (path, input, catalog, object, field, cause) in cases {
        fixture.write(employee_path, &original_employee);
        fixture.write(equipment_path, &original_equipment);
        fixture.write(path, input);
        let error = crate::authoring::validate(
            fixture.root(),
            &fixture.selection(),
            &Default::default(),
            &CancellationToken::default(),
        )
        .err()
        .expect("zero cadence input published");
        let crate::authoring::AuthoringError::Content { diagnostic } = error else {
            panic!("{error:?}");
        };
        assert_eq!(diagnostic.code, "content_validation", "{diagnostic:?}");
        assert_eq!(diagnostic.phase, LiveRonPhase::FinalValidation);
        assert_eq!(diagnostic.source_file, path);
        assert_eq!(diagnostic.catalog_kind, Some(catalog));
        assert_eq!(diagnostic.pack_id.as_deref(), Some("official"));
        assert_eq!(diagnostic.object_id.as_deref(), Some(object));
        assert_eq!(diagnostic.field_path.as_deref(), Some(field));
        assert_eq!(diagnostic.cause, cause);
        assert_eq!(r06_held_public_state(&token, &session), held);
        eprintln!(
            "R06_PUBLIC_ZERO {}",
            serde_json::to_string(&diagnostic).unwrap()
        );
    }
}

#[test]
fn r06_public_dynamic_query_limits_identity_lifecycle_and_trace_independence() {
    let (_fixture, token, document) = public_amount_trial_fixture(|f| {
        let path = "official/data/skills/base.ron";
        let text=f.read(path).replace("Damage(amount: 42, damage_type: Magic)",
            "DamageExpr(amount: (moment: Live, root: Add([Add([Mul(value: Const((value: 1125899906842624, unit: Magnitude)), scalar: Mul(value:Const((value:1125899906842624,unit:Scalar)),scalar:Ratio(numerator:Read(Stat(subject:Source,stat:Attack)),denominator:Read(Stat(subject:Source,stat:Attack)),rounding:TowardZero))), Const((value: 1, unit: Magnitude))]), Mul(value: Const((value: 1125899906842624, unit: Magnitude)), scalar: Const((value: -1125899906842624, unit: Scalar))), Const((value: 41, unit: Magnitude))])), damage_type: Magic)");
        f.write(path, text);
    });
    if let Some(output) = std::env::var_os("R06_PUBLIC_CLI_EXPORT") {
        let output = std::path::PathBuf::from(output);
        assert!(!output.exists());
        fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
            std::fs::create_dir_all(to).unwrap();
            for entry in std::fs::read_dir(from).unwrap() {
                let entry = entry.unwrap();
                let target = to.join(entry.file_name());
                if entry.file_type().unwrap().is_dir() {
                    copy_tree(&entry.path(), &target)
                } else {
                    std::fs::copy(entry.path(), target).unwrap();
                }
            }
        }
        copy_tree(_fixture.root(), &output.join("content"));
        std::fs::write(
            output.join("selection.json"),
            serde_json::to_vec(&_fixture.selection()).unwrap(),
        )
        .unwrap();
        std::fs::write(
            output.join("scenario.json"),
            serde_json::to_vec(&document).unwrap(),
        )
        .unwrap();
    }
    let mut baseline = None;
    for trace in [
        TraceOptions::default(),
        TraceOptions {
            limits: Some(TraceLimits::default()),
        },
        TraceOptions {
            limits: Some(TraceLimits {
                max_rows: 1,
                max_bytes: 1,
                max_row_bytes: 1,
                max_attempt_work: 1,
            }),
        },
    ] {
        let cancel = CancellationToken::default();
        let mut session = TrialSession::prepare(
            fixture_content(&token),
            checked(&document),
            TrialLimits {
                amount_observation: true,
                trace,
                ..Default::default()
            },
            &cancel,
        )
        .unwrap();
        let mut first_terminal = None;
        let mut witnessed_second_active = false;
        while !session.is_complete() {
            session.advance(&cancel).unwrap();
            let observation = session.observation().unwrap();
            if let Some(terminal) = observation.last_terminal() {
                first_terminal.get_or_insert(terminal.key());
            }
            if let (Some(first), Some(active)) = (first_terminal, observation.active()) {
                if active.key().unwrap() != first {
                    witnessed_second_active = true;
                }
            }
        }
        assert!(witnessed_second_active);
        let key = session
            .observation()
            .unwrap()
            .last_terminal()
            .unwrap()
            .key();
        assert_ne!(key, first_terminal.unwrap());
        let query = AmountQuery::Latest {
            battle: AmountBattle::LastTerminal,
        };
        let limits = ExecutionLimits::default();
        let preview = session.preview_amount(query, &limits, &cancel).unwrap();
        let explanation = session.explain_amount(query, &limits, &cancel).unwrap();
        assert!(preview
            .document()
            .contains("1267650600228229401496703205377"));
        assert!(explanation
            .document()
            .contains("1267650600228229401496703205377"));
        if let Some((p, e)) = &baseline {
            assert_eq!(preview.document(), p);
            assert_eq!(explanation.document(), e);
        } else {
            baseline = Some((
                preview.document().to_string(),
                explanation.document().to_string(),
            ));
        }
        let before = (
            session
                .state
                .application
                .snapshot(ApplicationSnapshotKind::Run)
                .unwrap(),
            session.accepted_work(),
            session.next_step_index(),
            session
                .receipt_documents()
                .map(str::to_owned)
                .collect::<Vec<_>>(),
        );
        for key in [
            first_terminal.unwrap(),
            BattleKey {
                battle_uuid: Uuid::nil(),
                ..key
            },
            BattleKey {
                entry_ordinal: key.entry_ordinal + 1,
                ..key
            },
        ] {
            let doc = session
                .preview_amount(
                    AmountQuery::Exact {
                        battle_key: key,
                        evaluation_ordinal: 1,
                    },
                    &limits,
                    &cancel,
                )
                .unwrap();
            assert!(doc.document().contains("not_retained"));
        }
        assert!(session
            .preview_amount(
                AmountQuery::Exact {
                    battle_key: key,
                    evaluation_ordinal: 0
                },
                &limits,
                &cancel
            )
            .is_err());
        for mode in 0..3 {
            let mut exact = limits.clone();
            match mode {
                0 => exact.max_work = preview.usage().work,
                1 => exact.max_metadata_bytes = preview.usage().metadata_bytes_peak,
                _ => exact.max_result_bytes = preview.usage().result_bytes_peak,
            };
            assert_eq!(
                preview.document(),
                session
                    .preview_amount(query, &exact, &cancel)
                    .unwrap()
                    .document()
            );
            match mode {
                0 => exact.max_work -= 1,
                1 => exact.max_metadata_bytes -= 1,
                _ => exact.max_result_bytes -= 1,
            };
            assert!(session.preview_amount(query, &exact, &cancel).is_err());
            assert_eq!(
                preview.document(),
                session
                    .preview_amount(query, &limits, &cancel)
                    .unwrap()
                    .document()
            );
        }
        for site in ["value_expr_trace", "mandatory_result_encode"] {
            let mid = CancellationToken::default();
            let result = control::with_test_cancellation(site, 3, || {
                session.explain_amount(query, &limits, &mid)
            });
            assert!(mid.is_cancelled(), "hook not reached: {site}");
            assert!(result.is_err());
            assert_eq!(
                explanation.document(),
                session
                    .explain_amount(query, &limits, &cancel)
                    .unwrap()
                    .document()
            );
        }
        let stopped = CancellationToken::default();
        stopped.cancel();
        assert!(session.explain_amount(query, &limits, &stopped).is_err());
        assert_eq!(
            before,
            (
                session
                    .state
                    .application
                    .snapshot(ApplicationSnapshotKind::Run)
                    .unwrap(),
                session.accepted_work(),
                session.next_step_index(),
                session
                    .receipt_documents()
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            )
        );
    }
}

#[test]
fn r06_public_literal_expression_delivery_matrix_and_actual_damage_prefix() {
    for delivery in ["Instant", "Projectile(speed_units_per_ms:7000)", "TileArea(footprint:(include_anchor_tile:true,rows:[\"@\"]),area:(anchor:CastTarget,tile_origin:Anchor,hit_targets:Enemies))"] {
        for amount in ["Damage(amount:4,damage_type:True)","DamageExpr(amount:(moment:Snapshot,root:Const((value:4,unit:Magnitude))),damage_type:True)","DamageExpr(amount:(moment:Live,root:Const((value:4,unit:Magnitude))),damage_type:True)"] {
            let (_fixture,token,document)=public_amount_trial_fixture(|f| {
                let path="official/data/skills/base.ron";
                let text=f.read(path);
                let begin=text.find("id: \"fragment_freischutz_black_round\"").unwrap();
                let end=text[begin..].find("id: \"fragment_funeral_butterfly_eulogy\"").unwrap()+begin;
                let skill=text[begin..end].replace("delivery: Projectile(speed_units_per_ms: 7000)",&format!("delivery: {delivery}"))
                    .replace("(id: \"effect_01\", op: Damage(amount: 42, damage_type: Magic))",&format!("(id:\"damage\",op:{amount}),(id:\"heal\",op:HealExpr(amount:(moment:Live,root:Read(Result(selector:EarlierEffect(\"damage\"),field:DamageHpLost)))))"));
                f.write(path,format!("{}{}{}",&text[..begin],skill,&text[end..]));
            });
            let cancel=CancellationToken::default();
            let session=simulate(fixture_content(&token),checked(&document),TrialLimits{amount_observation:true,..Default::default()},&cancel).unwrap();
            let window=if session.observation().unwrap().active().is_some(){AmountBattle::Active}else{AmountBattle::LastTerminal};
            let query=AmountQuery::Latest{battle:window};
            let p=session.preview_amount(query,&ExecutionLimits::default(),&cancel).unwrap();
            let value:Value=serde_json::from_str(p.document()).unwrap();
            assert_eq!(value["status"],"evaluated","{delivery}: {}",p.document());
            assert_eq!(value["site"]["effect_id"],"heal");
            assert_eq!(value["typed_base_amount"]["value"],4);
            assert_eq!(value["identity"]["evaluation_ordinal"],2);
            assert!(!value["source"]["action_ordinal"].is_null());
            assert_eq!(session.explain_amount(query,&ExecutionLimits::default(),&cancel).unwrap().document().contains("recorded_evaluation"),true);
        }
    }
}

#[test]
fn r10_public_flow_amount_generation_query_trace_and_cli_export() {
    for moment in ["Snapshot", "Live"] {
        let (fixture, token, mut document) = public_amount_trial_fixture(|fixture| {
            let path = "official/data/skills/base.ron";
            let mut source = fixture.read(path).replace("id: \"fragment_freischutz_black_round\",", "id: \"fragment_freischutz_black_round\", flow: Some(Parallel(id: \"r10_root\", branches: [Action(id: \"round\", step_id: \"fragment_black_round\", wait: EffectsSettled), Action(id: \"smoke\", step_id: \"fragment_smoke_focus\", wait: EffectsSettled)])),");
            source = source.replace("Damage(amount: 42, damage_type: Magic)", &format!("DamageExpr(amount: (moment: {moment}, root: Const((value: 42, unit: Magnitude))), damage_type: Magic)"));
            fixture.write(path, source);
        });
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"command", "command":{"type":"complete_combat_result"}}));
        let plan = token
            .skill_data
            .compiled_flow("fragment_freischutz_black_round")
            .unwrap();
        assert!(!plan.legacy);
        assert_eq!(plan.nodes[1].id, "round");
        let mut observed = None;
        for trace in [
            TraceOptions::default(),
            TraceOptions {
                limits: Some(TraceLimits::default()),
            },
        ] {
            let cancel = CancellationToken::default();
            let session = simulate(
                fixture_content(&token),
                checked(&document),
                TrialLimits {
                    amount_observation: true,
                    trace,
                    ..Default::default()
                },
                &cancel,
            )
            .unwrap();
            let query = AmountQuery::Latest {
                battle: AmountBattle::LastTerminal,
            };
            let preview = session
                .preview_amount(query, &ExecutionLimits::default(), &cancel)
                .unwrap();
            let value: Value = serde_json::from_str(preview.document()).unwrap();
            assert_eq!(value["status"], "evaluated");
            assert_eq!(value["site"]["action"]["action_id"], "fragment_black_round");
            assert_eq!(value["source"]["flow"]["node_index"], 1);
            assert_eq!(value["source"]["flow"]["occurrence"], 0);
            assert_eq!(value["moment"], moment);
            if let Some(prior) = &observed {
                assert_eq!(prior, &value);
            } else {
                observed = Some(value);
            }
        }
        if let Some(root) = std::env::var_os("R10_PUBLIC_CLI_EXPORT") {
            let output = std::path::PathBuf::from(root).join(moment.to_ascii_lowercase());
            assert!(!output.exists());
            fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
                std::fs::create_dir_all(to).unwrap();
                for entry in std::fs::read_dir(from).unwrap() {
                    let entry = entry.unwrap();
                    let destination = to.join(entry.file_name());
                    if entry.file_type().unwrap().is_dir() {
                        copy_tree(&entry.path(), &destination);
                    } else {
                        std::fs::copy(entry.path(), destination).unwrap();
                    }
                }
            }
            copy_tree(fixture.root(), &output.join("content"));
            std::fs::write(
                output.join("selection.json"),
                serde_json::to_vec(&fixture.selection()).unwrap(),
            )
            .unwrap();
            std::fs::write(
                output.join("scenario.json"),
                serde_json::to_vec(&document).unwrap(),
            )
            .unwrap();
        }
    }
}

#[test]
fn r12_public_sustained_protection_amount_trace_and_cli_export() {
    for (name, authored) in [
        (
            "hello",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/r11/hello_world_result.ron"
            )),
        ),
        (
            "protection",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/r12/world_protection.ron"
            )),
        ),
    ] {
        let (fixture, token, mut document) = public_amount_trial_fixture(|fixture| {
            let path = "official/data/skills/base.ron";
            let source = fixture.read(path).replacen(
                "id: \"fragment_freischutz_black_round\"",
                "id: \"r12_original_black_round\"",
                1,
            );
            let authored = authored.replace("r08_strike", "fragment_freischutz_black_round");
            let entries = &authored[authored.find('[').unwrap() + 1..authored.rfind(']').unwrap()];
            fixture.write(
                path,
                source.replacen("skills: [", &format!("skills: [{entries},"), 1),
            );
        });
        let steps = document["steps"].as_array_mut().unwrap();
        steps.truncate(steps.len() - 3);
        let mut activate = authority_step("activate_skill", 24);
        activate["command"]["employee_uuid"] = Value::Null;
        activate["command"]["skill_id"] = json!("fragment_freischutz_black_round");
        activate["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}));
        steps.push(activate);
        steps.push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":400,"after_seq":0}}));
        if name == "hello" {
            let mut reinput = authority_step("reinput_sustained_skill", 26);
            reinput["command"]["activation"] = Value::Null;
            reinput["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":26,"pointer":"/payload/battle_update/checkpoint/sustained_effects/active/0/activation"},"to":"/command/activation","value_type":"number"}));
            reinput["command"]["employee_uuid"] = Value::Null;
            reinput["command"]["skill_id"] = json!("fragment_freischutz_black_round");
            reinput["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}));
            steps.push(reinput);
        }
        steps
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":7000,"after_seq":0}}));
        steps.push(json!({"kind":"command","command":{"type":"complete_combat_result"}}));
        let mut baseline = None;
        let mut amount_baseline = None;
        for (observe, trace) in [
            (true, TraceOptions::default()),
            (
                true,
                TraceOptions {
                    limits: Some(TraceLimits::default()),
                },
            ),
            (false, TraceOptions::default()),
        ] {
            let cancel = CancellationToken::default();
            let session = simulate(
                fixture_content(&token),
                checked(&document),
                TrialLimits {
                    amount_observation: observe,
                    trace,
                    ..Default::default()
                },
                &cancel,
            )
            .unwrap();
            let query = AmountQuery::Latest {
                battle: AmountBattle::LastTerminal,
            };
            let before = session
                .receipt_documents()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            let work = session.accepted_work();
            let mut observed = None;
            for _ in 0..if observe { 3 } else { 0 } {
                let preview = session
                    .preview_amount(query, &ExecutionLimits::default(), &cancel)
                    .unwrap();
                let explanation = session
                    .explain_amount(query, &ExecutionLimits::default(), &cancel)
                    .unwrap();
                let value: Value = serde_json::from_str(preview.document()).unwrap();
                let explain: Value = serde_json::from_str(explanation.document()).unwrap();
                assert_eq!(value["status"], "evaluated", "{name}: {value}");
                assert_eq!(
                    value["site"]["action"]["action_id"],
                    if name == "hello" { "remaining" } else { "counter" }
                );
                if name == "protection" {
                    // Facing the actual arrival tile permits the authored parry;
                    // its 100 True counter is now the terminal amount observation.
                    assert_eq!(value["typed_base_amount"]["value"],100);
                }
                assert!(!value["source"]["fragment"].is_null());
                assert_eq!(value["typed_base_amount"], explain["typed_base_amount"]);
                assert_eq!(value["trace"], explain["trace"]);
                assert_eq!(explain["static_plan"]["evaluation"]["performed"], false);
                if let Some(prior) = &observed {
                    assert_eq!(prior, &value);
                } else {
                    observed = Some(value);
                }
            }
            assert_eq!(
                session
                    .receipt_documents()
                    .map(str::to_owned)
                    .collect::<Vec<_>>(),
                before
            );
            assert_eq!(session.accepted_work(), work);
            let public = (0..=document["preparation_steps"].as_array().map_or(0, Vec::len)
                + document["steps"].as_array().unwrap().len())
                .filter_map(|i| session.step(i).and_then(|s| s.public_result()).cloned())
                .collect::<Vec<_>>();
            if observe {
                if let Some(prior) = &amount_baseline {
                    assert_eq!(prior, &observed);
                } else {
                    amount_baseline = Some(observed);
                }
            }
            let current = public;
            if let Some(prior) = &baseline {
                assert_eq!(prior, &current);
            } else {
                baseline = Some(current);
            }
        }
        if let Some(root) = std::env::var_os("R12_PUBLIC_CLI_EXPORT") {
            let output = std::path::PathBuf::from(root).join(name);
            assert!(!output.exists());
            fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
                std::fs::create_dir_all(to).unwrap();
                for entry in std::fs::read_dir(from).unwrap() {
                    let entry = entry.unwrap();
                    let destination = to.join(entry.file_name());
                    if entry.file_type().unwrap().is_dir() {
                        copy_tree(&entry.path(), &destination);
                    } else {
                        std::fs::copy(entry.path(), destination).unwrap();
                    }
                }
            }
            copy_tree(fixture.root(), &output.join("content"));
            std::fs::write(
                output.join("selection.json"),
                serde_json::to_vec(&fixture.selection()).unwrap(),
            )
            .unwrap();
            std::fs::write(
                output.join("scenario.json"),
                serde_json::to_vec(&document).unwrap(),
            )
            .unwrap();
        }
    }
}

#[test]
fn r12_public_boss_flow_projectile_area_milestones_and_cli_export() {
    use crate::game::data::{boss_runtime_data::*, pve_data::*};
    for milestone in ["Released", "EffectsSettled"] {
        let fixture = ExternalFixture::official_with_trial_route();
        {
            let path = "official/data/boss_runtime/base.ron";
            let mut catalog: BossRuntimeCatalogData =
                ron::de::from_str(&fixture.read(path)).unwrap();
            let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r08/boss.ron"))
                .replace("milestone: EffectsSettled", &format!("milestone: {milestone}"))
                .replace("timeout_ms: 1000", "timeout_ms: 20000")
                .replace("State(id: \"completed\", tasks: []", "State(id: \"completed\", tasks: [StartAction(id: \"marker\", skill_id: \"r12_boss_marker\", target: SelfUnit, facing: left, milestone: EffectsSettled, timeout_ms: 20000, on_timeout: CancelUnreleased)]");
            let mut addition: BossRuntimeCatalogData = ron::de::from_str(&source).unwrap();
            let BossControlNodeData::State { transitions, .. } =
                &mut addition.actor_behaviors[0].nodes[1]
            else {
                unreachable!()
            };
            for (i, result) in [
                BossTaskResultData::Completed,
                BossTaskResultData::RequestRejected,
                BossTaskResultData::TargetInvalidated,
                BossTaskResultData::TimedOut,
                BossTaskResultData::Canceled,
            ]
            .into_iter()
            .enumerate()
            {
                transitions.push(BossControlTransitionData {
                    id: BossTransitionId::new(format!("marker_{i}")),
                    priority: i as i32,
                    condition: BossConditionData::TaskResult {
                        task_id: BossTaskId::new("marker"),
                        result,
                    },
                    target_node_id: BossControlNodeId::new("marker_done"),
                });
            }
            addition.actor_behaviors[0]
                .nodes
                .push(BossControlNodeData::State {
                    id: BossControlNodeId::new("marker_done"),
                    tasks: vec![],
                    transitions: vec![],
                });
            catalog.patterns.append(&mut addition.patterns);
            catalog
                .actor_behaviors
                .append(&mut addition.actor_behaviors);
            catalog.encounters.append(&mut addition.encounters);
            fixture.write(path, ron::ser::to_string(&catalog).unwrap());
            let path = "official/data/pve/encounters.ron";
            let mut encounters: PveEncounterDatabase =
                ron::de::from_str(&fixture.read(path)).unwrap();
            for encounter in &mut encounters.encounters {
                if encounter.encounter_class == PveEncounterClass::Normal
                    && encounter.boss_runtime.is_none()
                {
                    encounter.boss_runtime = Some(PveBossRuntimeBindingData {
                        encounter_id: BossEncounterRuntimeId::new("r08_encounter"),
                        actors: vec![PveBossActorBindingData {
                            actor_slot_id: BossActorSlotId::new("primary"),
                            wave_id: encounter.waves[0].id.clone(),
                            spawn_index: 0,
                        }],
                    });
                }
            }
            fixture.write(path, ron::ser::to_string(&encounters).unwrap());
            let path = "official/data/skills/base.ron";
            let authored = r#"
SkillDef(id: "r08_strike", name: "External boss finite flow", kind: Targeted, focus_time_ms: 100,
 use_lifetime: Action,
 cast_targeting: Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: global, air_capable: true),
 flow: Some(Sequence(id: "root", children: [Action(id: "projectile", step_id: "projectile", wait: EffectsSettled), Wait(id: "pause", duration_ms: 100), Action(id: "area", step_id: "area", wait: EffectsSettled)])),
 steps: [
 SkillStepDef(id: "projectile", delay_ms: 200, target: CastTarget, delivery: Projectile(speed_units_per_ms: 10000), effects: [(id: "hit", op: DamageExpr(amount: (moment: Snapshot, root: Const((value: 1, unit: Magnitude))), damage_type: True))]),
 SkillStepDef(id: "area", delay_ms: 0, target: CastTarget, delivery: TileArea(footprint: (include_anchor_tile: true, rows: ["XXX", "X@X", "XXX"]), area: (anchor: CastTarget, tile_origin: Anchor)), effects: [(id: "hit", op: DamageExpr(amount: (moment: Live, root: Const((value: 1, unit: Magnitude))), damage_type: True))])
 ]),
SkillDef(id: "r12_boss_marker", name: "Public milestone marker", kind: Targeted, focus_time_ms: 0,
 use_lifetime: Action,
 cast_targeting: Explicit(selection: self, target: SelfUnit, reach: pattern, air_capable: true),
 steps: [SkillStepDef(id: "marker", delay_ms: 0, target: SelfUnit, delivery: Instant, effects: [(id: "heal", op: Heal(amount: 1))])]),
"#;
            fixture.write(
                path,
                fixture
                    .read(path)
                    .replacen("skills: [", &format!("skills: [{authored}"), 1),
            );
        }
        let token = fixture.load();
        let mut document = preparation_document(&token);
        let cancel = CancellationToken::default();
        let probe = TrialSession::prepare(
            fixture_content(&token),
            checked(&document),
            TrialLimits::default(),
            &cancel,
        )
        .unwrap();
        let ground = probe.step(5).unwrap().public_result().unwrap()["payload"]["battle_setup_snapshot"]
            ["deployment_zones"]
            .as_array()
            .unwrap()
            .iter()
            .position(|z| z["kind"] == "Ground")
            .unwrap();
        let mut steps = Vec::new();
        for (i, authority) in [(0, 6), (1, 9)] {
            let mut deploy = authority_step("deploy_unit", authority);
            deploy["command"]["employee_uuid"] = Value::Null;
            deploy["command"]["position"] = Value::Null;
            deploy["command"]["facing"] = json!("right");
            deploy["bindings"].as_array_mut().unwrap().extend([
                json!({"from":{"step":6,"pointer":format!("/payload/battle_update/checkpoint/deployment/unit_deploy_costs/{i}/employee_uuid")},"to":"/command/employee_uuid","value_type":"string"}),
                json!({"from":{"step":5,"pointer":format!("/payload/battle_setup_snapshot/deployment_zones/{ground}/cells/{}",3-i)},"to":"/command/position","value_type":"object"}),
            ]);
            steps.push(deploy);
        }
        // Preserve the default per-operation hard limit by using ordinary short
        // host ticks. This fixture observes the live boss before terminal.
        for _ in 0..16 {
            steps.push(
                json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":500,"after_seq":0}}),
            );
        }
        document["steps"] = Value::Array(steps);
        let cancel = CancellationToken::default();
        let session = simulate(
            fixture_content(&token),
            checked(&document),
            TrialLimits {
                amount_observation: true,
                trace: TraceOptions {
                    limits: Some(TraceLimits::default()),
                },
                ..Default::default()
            },
            &cancel,
        )
        .unwrap();
        // The public presentation log is the oracle: a second action can only
        // start after the authored Boss task transitions to its completed node.
        fn events(value: &Value, output: &mut Vec<Value>) {
            match value {
                Value::Object(map) => {
                    if map.contains_key("time_ms") && map.contains_key("event") {
                        output.push(value.clone());
                    }
                    for value in map.values() {
                        events(value, output);
                    }
                }
                Value::Array(values) => {
                    for value in values {
                        events(value, output);
                    }
                }
                _ => {}
            }
        }
        let mut entries = Vec::new();
        for i in 0..session.next_step_index() {
            if let Some(value) = session.step(i).and_then(|s| s.public_result()) {
                events(value, &mut entries);
            }
        }
        let step_time = |skill: &str, step: &str| {
            entries
                .iter()
                .filter(|e| e["event"]["skill_id"] == skill && e["event"]["step_id"] == step)
                .filter_map(|e| e["time_ms"].as_u64())
                .min()
                .unwrap_or_else(|| panic!("missing {skill}/{step}: {entries:?}"))
        };
        let projectile = step_time("r08_strike", "projectile");
        let area = step_time("r08_strike", "area");
        let marker = step_time("r12_boss_marker", "marker");
        assert!(area > projectile);
        if milestone == "Released" {
            assert!(marker < projectile);
        } else {
            assert!(marker >= area);
        }
        assert!(
            entries
                .iter()
                .any(|e| e["event"]["damage_source"] == "Ability"
                    && e["event"]["raw_damage"] == 1
                    && e["event"]["delta"].as_i64().is_some_and(|d| d < 0)),
            "no real boss HP damage: {entries:?}"
        );
        if let Some(root) = std::env::var_os("R12_PUBLIC_CLI_EXPORT") {
            let output = std::path::PathBuf::from(root).join(milestone.to_ascii_lowercase());
            assert!(!output.exists());
            fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
                std::fs::create_dir_all(to).unwrap();
                for entry in std::fs::read_dir(from).unwrap() {
                    let entry = entry.unwrap();
                    let dest = to.join(entry.file_name());
                    if entry.file_type().unwrap().is_dir() {
                        copy_tree(&entry.path(), &dest);
                    } else {
                        std::fs::copy(entry.path(), dest).unwrap();
                    }
                }
            }
            copy_tree(fixture.root(), &output.join("content"));
            std::fs::write(
                output.join("selection.json"),
                serde_json::to_vec(&fixture.selection()).unwrap(),
            )
            .unwrap();
            std::fs::write(
                output.join("scenario.json"),
                serde_json::to_vec(&document).unwrap(),
            )
            .unwrap();
        }
    }
}
