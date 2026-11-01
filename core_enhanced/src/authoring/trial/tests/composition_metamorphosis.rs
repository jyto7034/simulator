#[test]
fn composition_metamorphosis_world_authored_limit_and_actual_initial_accessory_transform() {
    let (fixture, definition, source, mut document) = revival_composition_world_fixture();
    // This editor fixture tests the configurable terminal branch with one
    // recovery, then one transformation. The native World test separately
    // executes the agreed two recoveries and third trigger across three battles.
    let source = source
        .replace(
            "# First two survivals only; terminal hostile transformation is a separate follow-up.",
            "# Editor fixture: one survival, then terminal hostile transformation in the next battle.",
        )
        .replace("max_revivals: 2", "max_revivals: 1")
        .replace(
            "          invulnerable_ms: 1000",
            "          invulnerable_ms: 1000\n          transform_into: o-06-20_nothing_there",
        );
    let path = "official/data/employees/definitions.ron";
    let mut employees: crate::game::data::employee_data::EmployeeDefinitionDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    employees
        .definitions
        .iter_mut()
        .find(|e| e.employee_definition_id == "records_clerk")
        .unwrap()
        .initial_loadout
        .equipment_ids
        .push("placeholder_nothing_there_accessory".into());
    fixture.write(path, ron::ser::to_string(&employees).unwrap());
    let path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let guard = enemies
        .profiles
        .iter_mut()
        .find(|p| p.id == "corroded_guard")
        .unwrap();
    guard.attack = 10000;
    guard.basic_attack.interval_ms = 100000;
    guard.basic_attack.damage_type = crate::game::battle::damage::DamageType::True;
    for p in &mut enemies.profiles {
        if p.basic_attack_range_preset.is_some() {
            p.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(path, ron::ser::to_string(&enemies).unwrap());
    // Keep the initial actual copy equipped. The first reward still grants its
    // ordinary copy; the fixture does not replace the counter owner or inject HP.
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .remove(14 - preparation);
    fn shift(v: &mut Value) {
        match v {
            Value::Object(m) => {
                for (k, v) in m {
                    if k == "step" && v.as_u64().is_some_and(|n| n > 14) {
                        *v = json!(v.as_u64().unwrap() - 1);
                    } else {
                        shift(v);
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    shift(v);
                }
            }
            _ => (),
        }
    }
    shift(&mut document);
    // The only deployed employee in battle two exits permanently. Its first
    // clock reaches the real defeat result; further clocks are invalid after
    // terminal combat, so inspect the retained terminal observation there.
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(25 - preparation);
    // The original first-battle template only exposes its occupied objective
    // on the route. Author one real deployment cell immediately before it so
    // the wearer, rather than the objective, can intercept the lethal strike.
    let path = "official/data/map/battlefield_templates.ron";
    let mut templates: Vec<crate::game::combat_preview::BattlefieldTemplateDefinition> =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let template = templates
        .iter_mut()
        .find(|t| t.id == "surrounded_breach_ring_01")
        .unwrap();
    let mut authored = serde_json::to_value(&*template).unwrap();
    assert_eq!(authored["rows"][4], "W..PPP..R");
    authored["rows"][4] = json!("W.PPPP..R");
    *template = serde_json::from_value(authored).unwrap();
    fixture.write(path, ron::ser::to_string(&templates).unwrap());
    let cancel = CancellationToken::default();
    let probe = CompositionTrial::prepare(
        fixture.root(),
        &fixture.selection(),
        &source,
        checked(&document),
        None,
        TrialLimits::default(),
        &cancel,
    )
    .unwrap();
    let entry = probe.session().step(5).unwrap().public_result().unwrap();
    let setup = &entry["payload"]["battle_setup_snapshot"];
    let zones = setup["deployment_zones"].as_array().unwrap();
    let ground = zones.iter().position(|z| z["kind"] == "Ground").unwrap();
    let cells = zones[ground]["cells"].as_array().unwrap();
    let route = setup["routes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "defense_main")
        .unwrap();
    let occupied = entry["payload"]["battle_update"]["checkpoint"]["units"]
        .as_array()
        .unwrap();
    let intercept = route["cells"]
        .as_array()
        .unwrap()
        .iter()
        .rev()
        .filter(|cell| !occupied.iter().any(|u| u["position"] == **cell))
        .find_map(|cell| cells.iter().position(|c| c == cell))
        .unwrap_or_else(|| panic!("no interception cell: {setup}"));
    let other = cells
        .iter()
        .enumerate()
        .filter(|(i, cell)| *i != intercept && !occupied.iter().any(|u| u["position"] == **cell))
        .min_by_key(|(_, cell)| {
            let dx = cell["x"].as_i64().unwrap() - route["end"]["x"].as_i64().unwrap();
            let dy = cell["y"].as_i64().unwrap() - route["end"]["y"].as_i64().unwrap();
            (
                dx.abs() + dy.abs(),
                std::cmp::Reverse(cell["y"].as_i64().unwrap()),
            )
        })
        .unwrap()
        .0;
    for (step, cell_index) in [(9, intercept), (10, other)] {
        let cell = &cells[cell_index];
        let dx = cell["x"].as_i64().unwrap() - route["end"]["x"].as_i64().unwrap();
        let dy = cell["y"].as_i64().unwrap() - route["end"]["y"].as_i64().unwrap();
        // The interceptor faces incoming enemies; the support employee faces
        // back toward the objective, preserving the original first-battle fire.
        let (dx, dy) = if step == 10 { (-dx, -dy) } else { (dx, dy) };
        let deploy = &mut document["steps"][step - preparation];
        deploy["command"]["facing"] = json!(if dx.abs() >= dy.abs() {
            if dx > 0 {
                "right"
            } else {
                "left"
            }
        } else if dy > 0 {
            "down"
        } else {
            "up"
        });
        let binding = deploy["bindings"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|b| b["to"] == "/command/position")
            .unwrap();
        binding["from"]["pointer"] = json!(format!(
            "/payload/battle_setup_snapshot/deployment_zones/{ground}/cells/{cell_index}"
        ));
    }
    drop(probe);
    let mut expected = Vec::new();
    for form in ["o-06-20_nothing_there", "f-05-32_warm_hearted_woodsman"] {
        let current = source.replace(
            "transform_into: o-06-20_nothing_there",
            &format!("transform_into: {form}"),
        );
        let mut trial = CompositionTrial::prepare(
            fixture.root(),
            &fixture.selection(),
            &current,
            checked(&document),
            None,
            TrialLimits::default(),
            &cancel,
        )
        .unwrap();
        // Capture the initial loadout before any combat/reward. A later reward
        // grants another copy of this same definition and must not own the uses.
        let roster = trial.session().snapshot(
            crate::application::ApplicationSnapshotKind::EmployeeRoster,
            &crate::authoring::ExecutionLimits::default(), &cancel,
        ).unwrap();
        let roster: Value = serde_json::from_str(roster.document()).unwrap();
        let employees: Vec<_> = roster["read_model"]["employees"].as_array().unwrap().iter()
            .filter(|employee| employee["employee_definition_id"] == "records_clerk").collect();
        assert_eq!(employees.len(), 1);
        let copies: Vec<_> = employees[0]["equipped_items"].as_array().unwrap().iter()
            .filter(|item| item["definition_id"] == "placeholder_nothing_there_accessory")
            .map(|item| item["instance_uuid"].clone()).collect();
        assert_eq!(copies.len(), 1);
        let initial_copy = &copies[0];
        assert!(initial_copy.is_string());
        let mut healed = false;
        let mut final_witness = None;
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("metamorphosis World {pending}: {e:?}"));
            let observation = trial.session().observation().unwrap();
            let observed = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()));
            healed |= observed
                .and_then(|o| o.latest_nonzero_sourced_hp())
                .is_some_and(|h| {
                    h.equipment_source()
                        .is_some_and(|e| e.definition_uuid == definition
                            && json!(e.persistent_copy_uuid) == *initial_copy)
                        && h.change().hp_delta() > 0
                });
            if let Some(w) = observed.and_then(|o| o.latest_employee_transformation()) {
                assert!(
                    healed,
                    "the same employee must survive the first battle before transformation"
                );
                assert_eq!(w.form, form);
                assert_eq!(w.equipment.definition_uuid, definition);
                assert_eq!(json!(w.equipment.persistent_copy_uuid), *initial_copy);
                let reward = trial.session().step(13).unwrap().public_result().unwrap();
                let granted = &reward["payload"]["inventory_diff"]["added"][0]["Equipment"];
                assert_eq!(granted["definition_id"], "placeholder_nothing_there_accessory");
                assert!(granted["uuid"].is_string());
                assert_ne!(granted["uuid"], *initial_copy,
                    "the unused reward is a different copy of the same definition");
                assert_ne!(w.actor, w.successor);
                let origin = trial.latest_employee_transformation_origin().unwrap();
                assert!(origin.definition.ends_with("DefenseProgram"));
                final_witness = Some(json!({"form": form, "witness": w, "origin": origin}));
            }
        }
        expected.push(
            final_witness.unwrap_or_else(|| panic!("{form}: no actual employee transformation")),
        );
    }
    if let Some(directory) = std::env::var_os("METAMORPHOSIS_COMPOSITION_FIXTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists(), "preserve prior evidence");
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), source).unwrap();
        std::fs::write(directory.join("trial.json"), serde_json::to_vec_pretty(&json!({"root": directory.join("content"), "selection": fixture.selection(), "scenario": document})).unwrap()).unwrap();
        std::fs::write(
            directory.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
}
