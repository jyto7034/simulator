use super::*;
use crate::game::data::{
    live_ron::{GameplayRonSourceSet, LiveRonPhase},
    skill_data::RawSkillDatabase,
};
use std::{fs, path::PathBuf};

pub(crate) struct ExternalFixture(PathBuf);
impl ExternalFixture {
    pub(crate) fn official() -> Self {
        let root = std::env::temp_dir().join(format!("capg-external-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        let fixture = Self(root);
        let sources = GameplayRonSourceSet::embedded();
        for (path, input) in sources.source_inventory() {
            fixture.write(&format!("official/{path}"), input);
        }
        fixture.manifest(
            "official",
            PackManifest {
                pack_id: "official".into(),
                pack_version: "1".into(),
                namespace: PackNamespace::Global,
                contract: ContentContract::current(),
                required_capabilities: PackManifest::SUPPORTED_CAPABILITIES
                    .iter()
                    .map(|s| (*s).into())
                    .collect(),
                dependencies: vec![],
                files: CatalogKind::ALL
                    .iter()
                    .map(|kind| CatalogFile {
                        kind: *kind,
                        path: kind.official_path().into(),
                    })
                    .collect(),
                operations: vec![],
            },
        );
        fixture
    }
    /// Current role-specific starter sets and a supported authored facility route.
    pub(crate) fn official_with_trial_route() -> Self {
        let fixture = Self::official();
        fixture.author_trial_facility();
        fixture
    }
    /// Author a small current Expedition facility for public Trial scenarios.
    /// The old slot names are stable test labels, not a legacy mode or restored run.
    /// Call after scenario-specific catalog edits so the emitted RON is the tested input.
    pub(crate) fn author_trial_facility(&self) {
        use crate::game::map::*;
        use crate::game::data::{pve_data::PveEncounterDatabase, reward_data::RewardDatabase};
        use std::collections::BTreeMap;
        let path = "official/data/map/facility_templates.ron";
        let mut catalog: FacilityTemplateDatabase = ron::de::from_str(&self.read(path)).unwrap();
        let assignments: NodeAssignmentProfileDatabase = ron::de::from_str(
            &self.read("official/data/map/node_assignment_profiles.ron")).unwrap();
        let profile = assignments.profiles.iter().find(|p| p.id == "standard_floor_1").unwrap();
        let template = catalog.templates.iter_mut().find(|t| t.id.as_str() == "standard_intro_a").unwrap();
        // Current allocation requires two ordinary, unreserved encounters.
        // Keep them optional and distinct from the explicitly tested encounters.
        for id in ["trial_optional_a", "trial_optional_b"] {
            if !template.slots.iter().any(|s| s.id.as_str() == id) {
                template.slots.push(ron::de::from_str(&format!(r#"(id:"{id}",role:Ordinary,allowed_categories:[Combat,Event,Reward,Maintenance,Shop,HeadquartersContact])"#)).unwrap());
                template.connections.push(ron::de::from_str(&format!(r#"(from_slot_id:"entry",to_slot_id:"{id}",direction:Bidirectional)"#)).unwrap());
            }
        }
        // Fixed encounters must not block any other authored activity or the Gate.
        for slot in &template.slots {
            if slot.id.as_str() != "entry" && !slot.allowed_categories.contains(&MapNodeCategory::Combat)
                && !template.connections.iter().any(|e| e.from_slot_id.as_str() == "entry" && e.to_slot_id == slot.id) {
                template.connections.push(ron::de::from_str(&format!(r#"(from_slot_id:"entry",to_slot_id:"{}",direction:Bidirectional)"#,slot.id.as_str())).unwrap());
            }
        }
        let size = template.slots.iter().filter(|s| s.role == FacilitySlotRole::Ordinary).count() as u32;
        let template_id = template.id.clone();
        let slots = template.slots.clone();
        let mut layout = catalog.layouts[0].clone();
        let entrance = layout.room(&layout.entrance_room_id).unwrap().clone();
        let mut room = layout.room(&layout.initial_room_id).unwrap().clone();
        room.id = "trial_room".into(); room.template_id = Some(template_id);
        room.name = "Authored public trial route".into(); room.reservations.clear();
        room.omen_slot = None; room.elite_slots.clear(); room.safe_access_slots.clear(); room.slot_display.clear();
        layout.id = "authoring_trial_facility_v1".into(); layout.initial_room_id = room.id.clone();
        layout.golden_bough_plans.clear();
        for activity in &mut layout.activities { activity.golden_bough_candidate = false; }
        for fixed in &profile.fixed_slot_assignments {
            if !slots.iter().any(|s| s.id == fixed.slot_id && s.role == FacilitySlotRole::Ordinary) { continue; }
            let id = format!("trial_{}",fixed.slot_id.as_str());
            layout.activities.retain(|a| a.id != id);
            layout.activities.push(FacilityActivityDefinition { id:id.clone(),first_entry_cost:Some(0),golden_bough_candidate:false,
                label:id.clone(),kind_id:fixed.kind_id.clone(),payload:fixed.payload.clone(),
                support:FacilityContentSupport::Supported,condition:None,reward_activity:None });
            room.reservations.push(FacilityReservation {slot_id:fixed.slot_id.clone(),activity_id:id});
        }
        layout.connections = vec![FacilityRoomConnection {from:entrance.id.clone(),to:room.id.clone(),direction:MapEdgeDirection::ForwardOnly,condition:None}];
        layout.rooms = vec![entrance,room];
        let support = layout.reserved_support_count(&layout.rooms[1]);
        layout.allocation.support_counts = BTreeMap::from([(0,100)]);
        layout.allocation.one_reserved_support_counts = BTreeMap::from([(1,100)]);
        assert!(support <= 2);
        layout.allocation.sizes = vec![FacilitySizeAllocation {size,event_counts:BTreeMap::from([(0,100)]),reward_counts:BTreeMap::from([(0,100)]),elite_percent_by_depth:BTreeMap::from([(1,0)])}];
        // Current facility difficulty and role rules remain enabled.
        catalog.layouts = vec![layout];
        self.write(path,ron::ser::to_string(&catalog).unwrap());
        // Authored rewards now belong to the actual encounter producer.
        let rewards: RewardDatabase = ron::de::from_str(&self.read("official/data/events/rewards/base.ron")).unwrap();
        let encounters_path = "official/data/pve/encounters.ron";
        let mut encounters: PveEncounterDatabase = ron::de::from_str(&self.read(encounters_path)).unwrap();
        for fixed in &profile.fixed_slot_assignments {
            let (MapNodePayload::Encounter {encounter_id:Some(id)},Some(authored)) = (&fixed.payload,&fixed.authored_combat) else {continue;};
            let encounter = encounters.encounters.iter_mut().find(|e| &e.id == id).unwrap();
            for outcome in &authored.outcome_policies {
                let policy = encounter.outcome_policies.iter_mut().find(|p| p.outcome_id == outcome.outcome_id).unwrap();
                policy.reward_uuids = outcome.reward_ids.iter().map(|id|rewards.rewards.iter().find(|r| &r.id == id).unwrap().uuid).collect();
                policy.grant_node_bonus = outcome.grant_node_bonus;
                policy.advance_encounter_progression = outcome.advance_encounter_progression;
            }
        }
        self.write(encounters_path,ron::ser::to_string(&encounters).unwrap());
    }


    pub(crate) fn write(&self, path: &str, input: impl AsRef<[u8]>) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, input).unwrap();
    }
    pub(crate) fn read(&self, path: &str) -> String {
        fs::read_to_string(self.0.join(path)).unwrap()
    }
    pub(crate) fn manifest(&self, directory: &str, manifest: PackManifest) {
        self.write(
            &format!("{directory}/pack.ron"),
            ron::ser::to_string(&manifest).unwrap(),
        );
    }
    pub(crate) fn selection(&self) -> ContentSelection {
        ContentSelection {
            packs: vec!["official".into()],
            winners: vec![],
        }
    }
    pub(crate) fn load(&self) -> ValidatedGameData {
        load(&self.0, &self.selection(), &InputLimits::default()).unwrap()
    }
    pub(crate) fn root(&self) -> &std::path::Path {
        &self.0
    }
    pub(super) fn replace_first_skill(&self, id: &str, missing_preset: bool) -> DefinitionKey {
        let official =
            RawSkillDatabase::from_ron("fixture", GameplayRonSourceSet::embedded().skills).unwrap();
        let skill = official.skills.into_iter().next().unwrap();
        let target = DefinitionKey {
            kind: DefinitionKind::Skill,
            id: skill.id.to_string(),
        };
        let input = ron::ser::to_string(&RawSkillDatabase {
            range_presets: vec![],
            skills: vec![skill],
        })
        .unwrap();
        let input = if missing_preset {
            let changed = input.replace("front_3x2", "discarded_missing_preset");
            assert_ne!(input, changed);
            changed
        } else {
            input
        };
        self.write(&format!("{id}/skills.ron"), input);
        self.manifest(
            id,
            PackManifest {
                pack_id: id.into(),
                pack_version: "1".into(),
                namespace: PackNamespace::Global,
                contract: ContentContract::current(),
                required_capabilities: vec![],
                dependencies: vec!["official".into()],
                files: vec![CatalogFile {
                    kind: CatalogKind::Skills,
                    path: "skills.ron".into(),
                }],
                operations: vec![DefinitionOperation {
                    target: target.clone(),
                    operation: OperationKind::Replace,
                    after: vec!["official".into()],
                }],
            },
        );
        target
    }
}
impl Drop for ExternalFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn external_resource_catalog_is_resolved_and_changes_semantic_identity() {
    let fixture = ExternalFixture::official();
    let path = "official/data/resources/base.ron";
    let input = r#"(resources:[(id:"charge",owner:EquipmentCopy,lifetime:Run,initial:3,cap:5)])"#;
    fixture.write(path, input);
    let a = fixture.load();
    assert_eq!(a.resource_data.resources[0].initial, 3);
    let source = a
        .source_metadata()
        .unwrap()
        .definitions()
        .iter()
        .find(|(key, _)| key.kind == DefinitionKind::Resource)
        .unwrap();
    assert_eq!(source.0.id, "charge");
    assert_eq!(source.1.source_file, path);
    assert_eq!(source.1.field_path, "resources[0]");
    fixture.write(path, input.replace("initial:3", "initial:4"));
    let b = fixture.load();
    assert_eq!(a.resource_data.resources[0].initial, 3);
    assert_eq!(b.resource_data.resources[0].initial, 4);
    assert_ne!(a.identity(), b.identity());
    fixture.write(path, input.replace("cap:5", "cap:6"));
    let c = fixture.load();
    assert_ne!(a.identity(), c.identity());
    assert_ne!(b.identity(), c.identity());
}

#[test]
fn external_resource_errors_keep_source_location_and_previous_generation() {
    let fixture = ExternalFixture::official();
    let path = "official/data/resources/base.ron";
    let valid = r#"(resources:[(id:"charge",owner:EquipmentCopy,lifetime:Run,initial:3,cap:5)])"#;
    fixture.write(path, valid);
    let accepted = fixture.load();
    for invalid in [
        valid.replace("initial:3,", ""),
        valid.replace(",cap:5", ""),
        valid.replace("cap:5", "cap:2"),
        valid.replace("lifetime:Run", "lifetime:Forever"),
        valid.replace("owner:EquipmentCopy", "owner:Definition"),
        valid.replace("initial:3", "initial:-1"),
        valid.replace("cap:5", "cap:5,unknown:0"),
        valid.replace("id:\"charge\"", "id:\"\""),
    ] {
        fixture.write(path, &invalid);
        let error = load(
            fixture.root(),
            &fixture.selection(),
            &InputLimits::default(),
        )
        .err()
        .expect("must reject resource input");
        assert_eq!(error.source_file, path, "{invalid}: {error}");
        assert_eq!(error.pack_id.as_deref(), Some("official"));
        assert_eq!(error.catalog_kind, Some(CatalogKind::Resources));
        assert_eq!(accepted.resource_data.resources[0].initial, 3);
        assert_eq!(fixture.read(path), invalid);
    }
    fixture.write(path, valid.replace("cap:5", "cap:2"));
    let error = load(
        fixture.root(),
        &fixture.selection(),
        &InputLimits::default(),
    )
    .err()
    .unwrap();
    assert_eq!(error.definition_kind, Some(DefinitionKind::Resource));
    assert_eq!(error.object_id.as_deref(), Some("charge"));
    assert_eq!(error.field_path.as_deref(), Some("resources[0]"));
    let definition = r#"(id:"charge",owner:EquipmentCopy,lifetime:Run,initial:3,cap:5)"#;
    fixture.write(path, format!("(resources:[{definition},{definition}])"));
    assert!(load(
        fixture.root(),
        &fixture.selection(),
        &InputLimits::default()
    )
    .is_err());
}

#[test]
fn public_resource_definition_explanation_describes_actual_authored_fields() {
    use crate::authoring::{
        self, AuthoringLimits, CancellationToken, ExecutionLimits, ExplainQuery,
    };
    let fixture = ExternalFixture::official();
    fixture.write(
        "official/data/resources/base.ron",
        r#"(resources:[(id:"charge",owner:EquipmentCopy,lifetime:Run,initial:3,cap:5)])"#,
    );
    let content = authoring::validate(
        fixture.root(),
        &fixture.selection(),
        &AuthoringLimits::default(),
        &CancellationToken::default(),
    )
    .unwrap();
    let explained = authoring::explain(
        &content,
        &ExplainQuery::Definition {
            key: DefinitionKey {
                kind: DefinitionKind::Resource,
                id: "charge".into(),
            },
        },
        &ExecutionLimits::default(),
        &CancellationToken::default(),
    )
    .unwrap();
    let document: serde_json::Value = serde_json::from_str(explained.document()).unwrap();
    assert_eq!(document["value"]["initial"], 3);
    assert_eq!(document["value"]["cap"], 5);
    assert_eq!(document["value"]["owner"], "EquipmentCopy");
    assert_eq!(document["value"]["lifetime"], "Run");
    assert_eq!(
        document["source"]["source_file"],
        "official/data/resources/base.ron"
    );
    let described = authoring::describe(
        &content,
        &ExecutionLimits::default(),
        &CancellationToken::default(),
    )
    .unwrap();
    let document: serde_json::Value = serde_json::from_str(described.document()).unwrap();
    assert!(document["catalog_kinds"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("Resources")));
    // A prior catalog contract is rejected; it is not implicitly upgraded.
    let original = fixture.read("official/pack.ron");
    let mut old: PackManifest = ron::de::from_str(&original).unwrap();
    old.contract.catalog_schema = 2;
    fixture.manifest("official", old);
    let error = load(
        fixture.root(),
        &fixture.selection(),
        &InputLimits::default(),
    )
    .err()
    .unwrap();
    assert_eq!(error.code, "content_contract_unsupported");
    assert_eq!(content.token().resource_data.resources[0].initial, 3);
}

#[test]
fn same_binary_external_generations_retain_exact_content_and_sources_after_files_change() {
    let fixture = ExternalFixture::official();
    let a = fixture.load();
    let item = &a.equipment_data.items[0];
    let old_price = item.price;
    let path = "official/data/equipments/base.ron";
    let original = fixture.read(path);
    let changed = original.replacen(
        &format!("price: {old_price}"),
        &format!("price: {}", old_price + 1),
        1,
    );
    assert_ne!(changed, original);
    fixture.write(path, changed);
    let b = fixture.load();
    assert_eq!(a.equipment_data.items[0].id, b.equipment_data.items[0].id);
    assert_eq!(a.equipment_data.items[0].price, old_price);
    assert_eq!(b.equipment_data.items[0].price, old_price + 1);
    assert_ne!(a.identity(), b.identity());
    assert_eq!(a.source_receipt().files.len(), 27);
    let relic_source = |data: &crate::game::data::ValidatedGameData| {
        data.source_receipt()
            .files
            .iter()
            .find(|file| file.path == "official/data/relics/base.ron")
            .unwrap()
            .clone()
    };
    assert_eq!(relic_source(&a), relic_source(&b));
    let key = DefinitionKey {
        kind: DefinitionKind::Skill,
        id: a.skill_data.skills[0].id.to_string(),
    };
    let source_before = a
        .source_metadata()
        .unwrap()
        .source_for(&key)
        .unwrap()
        .clone();
    assert_eq!(source_before.pack_id, "official");
    assert_eq!(source_before.source_file, "official/data/skills/base.ron");
    assert_eq!(
        a.skill_data.skills[0].steps[0].effects[0]
            .source
            .as_ref()
            .unwrap()
            .source_file,
        "official/data/skills/base.ron"
    );
    fixture.write("official/data/skills/base.ron", "invalid syntax");
    assert!(load(&fixture.0, &fixture.selection(), &InputLimits::default()).is_err());
    assert_eq!(a.equipment_data.items[0].price, old_price);
    assert_eq!(b.equipment_data.items[0].price, old_price + 1);
    assert_eq!(
        a.source_metadata().unwrap().source_for(&key),
        Some(&source_before)
    );
}

#[test]
fn comments_locations_and_inventory_enumeration_do_not_change_semantic_identity() {
    let a = ExternalFixture::official();
    let b = ExternalFixture::official();
    let first = a.load();
    assert_eq!(first.identity(), b.load().identity());
    let mut manifest: PackManifest = ron::de::from_str(&a.read("official/pack.ron")).unwrap();
    manifest.files.reverse();
    manifest.required_capabilities.reverse();
    a.manifest("official", manifest);
    a.write(
        "official/data/skills/base.ron",
        format!(
            "// comment-only edit\n{}",
            a.read("official/data/skills/base.ron")
        ),
    );
    let second = a.load();
    assert_eq!(first.identity(), second.identity());
    assert_ne!(first.source_receipt(), second.source_receipt());
}

fn replace_once(source: &str, before: &str, after: &str) -> String {
    let changed = source.replacen(before, after, 1);
    assert_ne!(
        changed, source,
        "fixture replacement must change exactly one intended input"
    );
    changed
}

fn constant_damage_expr(value: i64, padding: usize) -> String {
    format!(
        "DamageExpr(/*{}*/ amount: (moment: Snapshot, root: Const((value: {value}, unit: Magnitude))), damage_type: Magic)",
        "x".repeat(padding)
    )
}

#[test]
fn external_expression_publishes_supported_and_reports_unsupported_source() {
    let fixture = ExternalFixture::official();
    let accepted = fixture.load();
    assert!(!accepted
        .value_expr_catalog()
        .contains_authored_expressions());
    let accepted_identity = accepted.identity().clone();
    let path = "official/data/skills/base.ron";
    let original = fixture.read(path);
    let expression = constant_damage_expr(60, 0);
    fixture.write(
        path,
        replace_once(
            &original,
            "Damage(amount: 60, damage_type: Magic)",
            &expression,
        ),
    );
    let published = fixture.load();
    assert!(published
        .value_expr_catalog()
        .contains_authored_expressions());
    assert_ne!(published.identity(), &accepted_identity);
    let unsupported="DamageExpr(amount:(moment:Live,root:Aggregate(source:ResolvedTargets,grouping:ByOwner,reduction:Sum,outer_reduction:Some(Sum),member:Read(Stat(subject:AggregateMember,stat:Attack)))),damage_type:Magic)";
    fixture.write(
        path,
        replace_once(
            &original,
            "Damage(amount: 60, damage_type: Magic)",
            unsupported,
        ),
    );
    let error = load(
        fixture.root(),
        &fixture.selection(),
        &InputLimits::default(),
    )
    .err()
    .unwrap();
    assert_eq!(error.code, "content_capability_unsupported");
    assert_eq!(error.phase, LiveRonPhase::Compile);
    assert_eq!(error.pack_id.as_deref(), Some("official"));
    assert_eq!(error.source_file, path);
    assert_eq!(
        error.object_id.as_deref(),
        Some("unknown_distortion_strike")
    );
    assert_eq!(error.action_id.as_deref(), Some("strike"));
    assert_eq!(error.effect_id.as_deref(), Some("effect_01"));

    assert_eq!(accepted.identity(), &accepted_identity);
    assert!(!accepted
        .value_expr_catalog()
        .contains_authored_expressions());
}

#[test]
fn external_expression_contract_errors_precede_the_support_boundary() {
    let fixture = ExternalFixture::official();
    let path = "official/data/skills/base.ron";
    let original = fixture.read(path);
    let first_literal = "Damage(amount: 60, damage_type: Magic)";
    let cases = [
        (
            "value_expr_unit_mismatch",
            "DamageExpr(amount: (moment: Snapshot, root: Add([Const((value: 2, unit: Magnitude)), Const((value: 3, unit: Milliseconds))])), damage_type: Magic)",
        ),
        (
            "value_expr_missing_context",
            "DamageExpr(amount: (moment: Live, root: Read(Resource(resource: \"not_defined\", owner: SourceEmployee, field: Balance))), damage_type: Magic)",
        ),
        (
            "value_expr_missing_rounding",
            "DamageExpr(amount: (moment: Snapshot, root: Ratio(numerator: Const((value: 6, unit: Magnitude)), denominator: Const((value: 2, unit: Scalar)))), damage_type: Magic)",
        ),
        (
            "ron_parse",
            "DamageExpr(amount: (moment: Snapshot, root: UnsupportedNode((value: 1, unit: Magnitude))), damage_type: Magic)",
        ),
        (
            "value_expr_overflow",
            "DamageExpr(amount: (moment: Snapshot, root: Const((value: -1, unit: Magnitude))), damage_type: Magic)",
        ),
        (
            "value_expr_overflow",
            "DamageExpr(amount: (moment: Snapshot, root: Const((value: 4294967296, unit: Magnitude))), damage_type: Magic)",
        ),
        (
            "value_expr_overflow",
            "DamageExpr(amount: (moment: Snapshot, root: Aggregate(source: AuthoredMembers([(identity: \"a\", value: (value: -1, unit: Magnitude), owner: None, projected_tile: None)]), grouping: WholeSet, reduction: Sum, outer_reduction: None, member: Read(AggregateMember(unit: Magnitude)))), damage_type: Magic)",
        ),
        (
            "ron_parse",
            "DamageExpr(amount: (moment: Snapshot, root: Mul(value: Const((value: 1, unit: Magnitude)), scalar: Const((value: 2, unit: Scalar)), typo_parameter: 123)), damage_type: Magic)",
        ),
    ];
    for (expected_code, op) in cases {
        fixture.write(path, replace_once(&original, first_literal, op));
        let error = load(
            fixture.root(),
            &fixture.selection(),
            &InputLimits::default(),
        )
        .err()
        .expect("invalid expression must reject");
        assert_eq!(error.code, expected_code, "{error:?}");
        assert_ne!(error.code, "content_capability_unsupported");
        assert_eq!(error.pack_id.as_deref(), Some("official"));
        assert_eq!(error.catalog_kind, Some(CatalogKind::Skills));
        assert_eq!(error.source_file, path);
        assert_eq!(
            error.object_id.as_deref(),
            Some("unknown_distortion_strike")
        );
        assert_eq!(error.action_id.as_deref(), Some("strike"));
        assert_eq!(error.effect_id.as_deref(), Some("effect_01"));
    }

    let area_literal = "Damage(amount: 32, damage_type: Magic)";
    let unsupported_snapshot = "DamageExpr(amount: (moment: Snapshot, root: Read(HitPoints(subject: Target, field: Current))), damage_type: Magic)";
    fixture.write(
        path,
        replace_once(&original, area_literal, unsupported_snapshot),
    );
    let error = load(
        fixture.root(),
        &fixture.selection(),
        &InputLimits::default(),
    )
    .err()
    .expect("a TileArea target is not available at Snapshot");
    assert_eq!(error.code, "value_expr_unsupported_moment", "{error:?}");
    assert_eq!(
        error.object_id.as_deref(),
        Some("queen_of_hatred_magical_beam")
    );
    assert_eq!(error.action_id.as_deref(), Some("beam_trace"));
    assert_eq!(error.effect_id.as_deref(), Some("effect_01"));
}

#[test]
fn external_expression_span_limit_is_exact_and_runs_before_snapshot_clone() {
    let fixture = ExternalFixture::official();
    let accepted = fixture.load();
    let path = "official/data/skills/base.ron";
    let original = fixture.read(path);
    let literal = "Damage(amount: 60, damage_type: Magic)";
    let expression = constant_damage_expr(60, 32);
    fixture.write(path, replace_once(&original, literal, &expression));
    let exact = InputLimits {
        value_expr: crate::game::value_expr::ExprParseLimits {
            max_input_bytes: expression.len(),
            ..Default::default()
        },
        ..InputLimits::default()
    };
    assert!(load(fixture.root(), &fixture.selection(), &exact)
        .unwrap()
        .value_expr_catalog()
        .contains_authored_expressions());

    let over = InputLimits {
        value_expr: crate::game::value_expr::ExprParseLimits {
            max_input_bytes: expression.len() - 1,
            ..Default::default()
        },
        ..InputLimits::default()
    };
    let error = load(fixture.root(), &fixture.selection(), &over)
        .err()
        .expect("one byte over the span limit must reject during acquisition");
    assert_eq!(error.code, "value_expr_input_limit");
    assert_eq!(error.phase, LiveRonPhase::Structure);
    assert_eq!(error.pack_id.as_deref(), Some("official"));
    assert_eq!(error.catalog_kind, Some(CatalogKind::Skills));
    assert_eq!(error.source_file, path);
    assert!(error.field_path.as_deref().unwrap().starts_with("byte["));
    assert!(!accepted
        .value_expr_catalog()
        .contains_authored_expressions());
}

#[test]
fn only_final_winning_expression_is_compiled_and_published() {
    let fixture = ExternalFixture::official();
    let target = fixture.replace_first_skill("loser", false);
    fixture.replace_first_skill("winner", false);
    let loser_path = "loser/skills.ron";
    let source = fixture.read(loser_path);
    fixture.write(
        loser_path,
        replace_once(
            &source,
            "Damage(amount:60,damage_type:Magic,)",
            &constant_damage_expr(60, 0),
        ),
    );
    let selection = ContentSelection {
        packs: vec!["official".into(), "loser".into(), "winner".into()],
        winners: vec![WriterSelection {
            target: target.clone(),
            pack_id: "winner".into(),
        }],
    };
    let literal_generation = load(fixture.root(), &selection, &InputLimits::default()).unwrap();
    assert!(!literal_generation
        .value_expr_catalog()
        .contains_authored_expressions());

    let expression_wins = ContentSelection {
        winners: vec![WriterSelection {
            target,
            pack_id: "loser".into(),
        }],
        ..selection
    };
    let published = load(fixture.root(), &expression_wins, &InputLimits::default()).unwrap();
    let entry = published
        .value_expr_catalog()
        .entries()
        .find(|(_, entry)| entry.is_authored_expression())
        .unwrap()
        .1;
    assert_eq!(entry.plan().source().source_file, loser_path);
    assert!(!literal_generation
        .value_expr_catalog()
        .contains_authored_expressions());
}

#[test]
fn winning_raw_reference_only_is_lowered_and_published_from_actual_external_packs() {
    let fixture = ExternalFixture::official();
    let target = fixture.replace_first_skill("loser", true);
    fixture.replace_first_skill("winner", false);
    let selection = ContentSelection {
        packs: vec!["winner".into(), "official".into(), "loser".into()],
        winners: vec![WriterSelection {
            target: target.clone(),
            pack_id: "winner".into(),
        }],
    };
    let data = load(&fixture.0, &selection, &InputLimits::default()).unwrap();
    let metadata = data.source_metadata().unwrap();
    assert_eq!(metadata.source_for(&target).unwrap().pack_id, "winner");
    let writers: Vec<_> = metadata
        .writers()
        .iter()
        .filter(|writer| writer.target == target)
        .collect();
    assert_eq!(writers.len(), 3);
    assert_eq!(writers.iter().filter(|writer| writer.selected).count(), 1);
    let winner = writers.iter().find(|writer| writer.selected).unwrap();
    assert_eq!(winner.pack_id, "winner");
    assert_eq!(winner.operation, DefinitionWriteKind::Replace);
    assert_eq!(
        winner.after,
        std::collections::BTreeSet::from(["official".into()])
    );
    assert_eq!(
        data.skill_data.get_by_id(&target.id).unwrap().steps[0].effects[0]
            .source
            .as_ref()
            .unwrap()
            .source_file,
        "winner/skills.ron"
    );
    let bad = ContentSelection {
        winners: vec![WriterSelection {
            target,
            pack_id: "loser".into(),
        }],
        ..selection
    };
    let error = load(&fixture.0, &bad, &InputLimits::default())
        .err()
        .unwrap();
    assert_eq!(error.source_file, "loser/skills.ron");
    assert_eq!(
        error.field_path.as_deref(),
        Some("skills[0].cast_targeting")
    );
    assert!(error.cause.contains("discarded_missing_preset"));
}

#[test]
fn unsupported_unused_external_opcode_cannot_publish_even_in_a_losing_writer() {
    let fixture = ExternalFixture::official();
    let target = fixture.replace_first_skill("loser", false);
    fixture.replace_first_skill("winner", false);
    let source = fixture.read("loser/skills.ron");
    let invalid = source.replacen("Damage(", "UnsupportedOpcode(", 1);
    assert_ne!(invalid, source);
    fixture.write("loser/skills.ron", invalid);
    let selection = ContentSelection {
        packs: vec!["official".into(), "winner".into(), "loser".into()],
        winners: vec![WriterSelection {
            target: target.clone(),
            pack_id: "winner".into(),
        }],
    };
    let error = load(&fixture.0, &selection, &InputLimits::default())
        .err()
        .unwrap();
    assert_eq!(error.source_file, "loser/skills.ron", "{error:?}");
    assert_eq!(
        error.object_id.as_deref(),
        Some(target.id.as_str()),
        "{error:?}"
    );
    assert!(error.action_id.is_some(), "{error:?}");
    assert!(error.effect_id.is_some(), "{error:?}");
}

#[test]
fn generations_are_reclaimed_after_the_last_token_and_core_reference() {
    let fixture = ExternalFixture::official();
    let token = fixture.load();
    let weak = Arc::downgrade(&token.0);
    let core = crate::game::world::GameCore::new(token.clone(), 7);
    drop(token);
    assert!(weak.upgrade().is_some());
    drop(core);
    assert!(weak.upgrade().is_none());
    // There is no global publication registry retaining successful generations.
    for _ in 0..3 {
        let token = fixture.load();
        let weak = Arc::downgrade(&token.0);
        drop(token);
        assert!(weak.upgrade().is_none());
    }
}

#[test]
fn fixture_identity_tracks_actual_content_without_arc_or_source_addresses() {
    let first = ValidatedGameData::load_live_embedded().unwrap();
    let second = ValidatedGameData::load_live_embedded().unwrap();
    assert_eq!(first.identity(), second.identity());
    let mut equipment = (*first.equipment_data).clone();
    equipment.items[0].price += 1;
    let builder = crate::game::data::GameDataBuilder::from_validated(&first)
        .with_equipment_data(Arc::new(equipment));
    let changed = ValidatedGameData::from(builder.build_arc());
    assert_ne!(first.identity(), changed.identity());
}

fn start_command(
    mode: crate::game::map::GameMode,
    replace: bool,
) -> crate::application::ApplicationCommand {
    crate::application::ApplicationCommand::try_new(
        crate::application::ApplicationCommandKind::StartNewGame,
        crate::application::ApplicationDocument::new(
            serde_json::json!({"game_mode": mode, "replace_active_run": replace}),
        ),
    )
    .unwrap()
}

fn changed_briefing(fixture: &ExternalFixture) -> ValidatedGameData {
    let path = "official/data/visual_novels/base.ron";
    let source = fixture.read(path);
    let changed = source.replace(
        "script_id: \"hq_briefing_endless_intro\"",
        "script_id: \"hq_briefing_from_pack_b\"",
    );
    assert_ne!(changed, source);
    fixture.write(path, changed);
    fixture.load()
}

#[test]
fn active_briefing_pins_a_and_next_eligible_start_commits_b_before_briefing_and_starters() {
    use crate::application::{
        ApplicationCommand, ApplicationCommandKind as Kind, ApplicationDocument as Doc,
        ApplicationExecutionContext as Context, CoreApplication,
    };
    use crate::game::map::GameMode;
    let fixture = ExternalFixture::official();
    let a = fixture.load();
    let mut app = CoreApplication::new(a.clone(), 123);
    let player = uuid::Uuid::from_u128(1);
    let briefing_a = app
        .execute(
            player,
            start_command(GameMode::Expedition, false),
            Context::default(),
        )
        .unwrap();
    assert_eq!(
        briefing_a
            .payload()
            .as_value()
            .pointer("/visual_novel/script_id")
            .unwrap(),
        "hq_briefing_endless_intro"
    );
    let b = changed_briefing(&fixture);
    assert_ne!(a.identity(), b.identity());
    assert_eq!(app.content_identity(), a.identity());
    let before = app
        .snapshot(crate::application::ApplicationSnapshotKind::Run)
        .unwrap();
    assert!(app
        .execute_with_generation(
            player,
            start_command(GameMode::Expedition, true),
            Context::default(),
            b.clone()
        )
        .is_err());
    assert_eq!(app.content_identity(), a.identity());
    assert_eq!(
        app.snapshot(crate::application::ApplicationSnapshotKind::Run)
            .unwrap(),
        before
    );
    let vn_a = &briefing_a.payload().as_value()["visual_novel"];
    app.execute(player, ApplicationCommand::try_new(Kind::AdvanceVisualNovel, Doc::new(serde_json::json!({"session_id": vn_a["session_id"], "current_scene_id": vn_a["current_scene_id"]}))).unwrap(), Context::default()).unwrap();
    let starters_a: Vec<_> = a
        .employee_definition_data
        .definitions
        .iter()
        .filter(|d| d.starter_eligible)
        .take(a.run_policy.setup.starter_employee_count)
        .map(|d| d.employee_definition_id.clone())
        .collect();
    app.execute(
        player,
        ApplicationCommand::try_new(
            Kind::SelectStarterEmployees,
            Doc::new(serde_json::json!({"employee_definition_ids": starters_a, "loan_copy_ids": []})),
        )
        .unwrap(),
        Context::default(),
    )
    .unwrap();
    assert_eq!(app.content_identity(), a.identity());
    app.execute(
        player,
        ApplicationCommand::try_new(Kind::AbandonRun, Doc::new(serde_json::json!({}))).unwrap(),
        Context::default(),
    )
    .unwrap();
    let briefing_b = app
        .execute_with_generation(
            player,
            start_command(GameMode::Expedition, false),
            Context::default(),
            b.clone(),
        )
        .unwrap();
    assert_eq!(app.content_identity(), b.identity());
    let vn = &briefing_b.payload().as_value()["visual_novel"];
    assert_eq!(vn["script_id"], "hq_briefing_from_pack_b");
    app.execute(player, ApplicationCommand::try_new(Kind::AdvanceVisualNovel, Doc::new(serde_json::json!({"session_id": vn["session_id"], "current_scene_id": vn["current_scene_id"]}))).unwrap(), Context::default()).unwrap();
    let starters: Vec<_> = b
        .employee_definition_data
        .definitions
        .iter()
        .filter(|d| d.starter_eligible)
        .take(b.run_policy.setup.starter_employee_count)
        .map(|d| d.employee_definition_id.clone())
        .collect();
    app.execute(
        player,
        ApplicationCommand::try_new(
            Kind::SelectStarterEmployees,
            Doc::new(serde_json::json!({"employee_definition_ids": starters, "loan_copy_ids": []})),
        )
        .unwrap(),
        Context::default(),
    )
    .unwrap();
    assert_eq!(app.content_identity(), b.identity());
}

#[test]
fn rejected_nonstart_generation_preserves_a_and_expedition_start_adopts_b_without_tutorial_gate() {
    use crate::application::{
        ApplicationCommand, ApplicationCommandKind as Kind, ApplicationDocument as Doc,
        ApplicationExecutionContext as Context, ApplicationSnapshotKind as Snapshot,
        CoreApplication,
    };
    use crate::game::map::GameMode;
    let fixture = ExternalFixture::official();
    let a = fixture.load();
    let b = changed_briefing(&fixture);
    let mut app = CoreApplication::new(a.clone(), 321);
    let player = uuid::Uuid::from_u128(2);
    let kinds = [
        Snapshot::Run,
        Snapshot::Inventory,
        Snapshot::PlayerProfile,
        Snapshot::EncounterCompendium,
        Snapshot::EmployeeRoster,
    ];
    let before: Vec<_> = kinds
        .iter()
        .map(|kind| app.snapshot(*kind).unwrap())
        .collect();
    let request =
        ApplicationCommand::try_new(Kind::RequestMapData, Doc::new(serde_json::json!({}))).unwrap();
    assert!(app
        .execute_with_generation(player, request, Context::default(), b.clone())
        .is_err());
    assert_eq!(app.content_identity(), a.identity());
    let after_rejection: Vec<_> = kinds.iter().map(|kind| app.snapshot(*kind).unwrap()).collect();
    assert_eq!(after_rejection, before);
    let started = app
        .execute_with_generation(
            player,
            start_command(GameMode::Expedition, false),
            Context::default(),
            b.clone(),
        )
        .unwrap();
    assert_eq!(app.content_identity(), b.identity());
    assert_eq!(started.payload().as_value()["visual_novel"]["script_id"], "hq_briefing_from_pack_b");
}

#[test]
fn namespaced_boss_and_summon_catalog_keys_compile_while_local_ids_remain_local() {
    use crate::game::data::{boss_runtime_data::*, summoned_unit_data::*};
    let (catalog, summons) = crate::game::data::boss_runtime_data::tests::summon_catalog_fixture();
    let mut boss_text = ron::ser::to_string(&catalog).unwrap();
    let mut summon_text = ron::ser::to_string(&summons).unwrap();
    let ids = catalog
        .patterns
        .iter()
        .map(|d| d.id.as_str())
        .chain(catalog.actor_behaviors.iter().map(|d| d.id.as_str()))
        .chain(catalog.encounters.iter().map(|d| d.id.as_str()))
        .chain(summons.definitions.iter().map(|d| d.id.as_str()));
    for id in ids {
        let from = format!("\"{id}\"");
        let to = format!("\"custom::{id}\"");
        boss_text = boss_text.replace(&from, &to);
        summon_text = summon_text.replace(&from, &to);
    }
    let mut namespaced: BossRuntimeCatalogData = ron::de::from_str(&boss_text).unwrap();
    let namespaced_summons: SummonedUnitCatalogData = ron::de::from_str(&summon_text).unwrap();
    namespaced.signals.push(BossSignalDefinitionData {
        id: BossSignalId::new("custom::signal"),
    });
    let compiled = namespaced
        .compile_with_summoned_units(&namespaced_summons)
        .unwrap();
    assert!(compiled
        .get_encounter(&BossEncounterRuntimeId::new("custom::tracking_encounter"))
        .is_some());
    namespaced.patterns[0].states[0].id = BossPatternStateId::new("custom::acquire");
    assert!(namespaced
        .compile_with_summoned_units(&namespaced_summons)
        .unwrap_err()
        .cause
        .contains("lower_snake_case"));
}

#[test]
fn external_skill_and_corroded_definitions_resolve_presets_in_another_pack() {
    use crate::game::data::corroded_employee_data::RawCorrodedEmployeeProfileDatabase;
    let fixture = ExternalFixture::official();
    let original = fixture.load();
    let mut skills =
        RawSkillDatabase::from_ron("fixture", &fixture.read("official/data/skills/base.ron"))
            .unwrap();
    let mut profiles: RawCorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read("official/data/enemies/corroded_employees.ron")).unwrap();
    let skill_presets = RawSkillDatabase {
        range_presets: std::mem::take(&mut skills.range_presets),
        skills: vec![],
    };
    let profile_presets = RawCorrodedEmployeeProfileDatabase {
        range_presets: std::mem::take(&mut profiles.range_presets),
        profiles: vec![],
    };
    assert!(!skill_presets.range_presets.is_empty());
    assert!(!profile_presets.range_presets.is_empty());
    fixture.write(
        "official/data/skills/base.ron",
        ron::ser::to_string(&skills).unwrap(),
    );
    fixture.write(
        "official/data/enemies/corroded_employees.ron",
        ron::ser::to_string(&profiles).unwrap(),
    );
    fixture.write(
        "presets/skills.ron",
        ron::ser::to_string(&skill_presets).unwrap(),
    );
    fixture.write(
        "presets/profiles.ron",
        ron::ser::to_string(&profile_presets).unwrap(),
    );
    let mut official: PackManifest = ron::de::from_str(&fixture.read("official/pack.ron")).unwrap();
    official.dependencies = vec!["presets".into()];
    fixture.manifest("official", official);
    fixture.manifest(
        "presets",
        PackManifest {
            pack_id: "presets".into(),
            pack_version: "1".into(),
            namespace: PackNamespace::Global,
            contract: ContentContract::current(),
            required_capabilities: vec![],
            dependencies: vec![],
            files: vec![
                CatalogFile {
                    kind: CatalogKind::Skills,
                    path: "skills.ron".into(),
                },
                CatalogFile {
                    kind: CatalogKind::CorrodedEmployees,
                    path: "profiles.ron".into(),
                },
            ],
            operations: vec![],
        },
    );
    let mut selection = fixture.selection();
    selection.packs.push("presets".into());
    let split = load(fixture.root(), &selection, &InputLimits::default()).unwrap();
    assert_eq!(
        serde_json::to_value(&original.skill_data).unwrap(),
        serde_json::to_value(&split.skill_data).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&original.corroded_employee_data).unwrap(),
        serde_json::to_value(&split.corroded_employee_data).unwrap()
    );
    selection.packs.reverse();
    let reversed = load(fixture.root(), &selection, &InputLimits::default()).unwrap();
    assert_eq!(split.identity(), reversed.identity());
    let source = split.skill_data.skills[0].steps[0].effects[0]
        .source
        .as_ref()
        .unwrap();
    assert_eq!(source.source_file, "official/data/skills/base.ron");
}

#[test]
fn external_final_buff_reward_and_registration_references_reject_without_publication() {
    use crate::game::data::{pve_data::PveEncounterDatabase, reward_data::RewardDatabase};
    let fixture = ExternalFixture::official();
    let a = fixture.load();
    let a_identity = a.identity().clone();
    let a_skills = serde_json::to_value(&a.skill_data).unwrap();
    let skill_path = "official/data/skills/base.ron";
    let skills = fixture.read(skill_path);
    let bad_skill = skills.replacen(
        "Damage(amount: 60, damage_type: Magic)",
        "ApplyBuff(buff_id: \"missing_authoring_buff\", duration_ms: 10)",
        1,
    );
    assert_ne!(skills, bad_skill);
    let reward_path = "official/data/events/rewards/base.ron";
    let mut rewards: RewardDatabase = ron::de::from_str(&fixture.read(reward_path)).unwrap();
    rewards.pools[0].reward_ids[0] = "missing_final_reward".into();
    let pve_path = "official/data/pve/encounters.ron";
    let mut pve: PveEncounterDatabase = ron::de::from_str(&fixture.read(pve_path)).unwrap();
    let mut missing_encounter = pve.clone();
    missing_encounter.reward_sources[0].encounter_id = "missing_registered_encounter".into();
    pve.reward_sources[0].fragment_id = Some("missing_registered_fragment".into());
    for (path, bad_input, cause_fragment) in [
        (skill_path, bad_skill, "missing_authoring_buff"),
        (
            reward_path,
            ron::ser::to_string(&rewards).unwrap(),
            "missing_final_reward",
        ),
        (
            pve_path,
            ron::ser::to_string(&pve).unwrap(),
            "must equal primary abnormality",
        ),
        (
            pve_path,
            ron::ser::to_string(&missing_encounter).unwrap(),
            "references missing PVE encounter 'missing_registered_encounter'",
        ),
    ] {
        let before = fixture.read(path);
        fixture.write(path, bad_input);
        let mut previous = None;
        for _ in 0..2 {
            let error = match load(
                fixture.root(),
                &fixture.selection(),
                &InputLimits::default(),
            ) {
                Ok(_) => panic!("dangling final reference published: {path}"),
                Err(error) => error,
            };
            assert_eq!(error.source_file, path);
            assert_eq!(error.phase, LiveRonPhase::FinalValidation, "{error}");
            assert_eq!(error.code, "content_validation");
            assert_eq!(error.pack_id.as_deref(), Some("official"));
            assert!(error.object_id.is_some(), "{error}");
            assert!(error.field_path.is_some(), "{error}");
            assert!(error.cause.contains(cause_fragment), "{error}");
            if let Some(previous) = previous {
                assert_eq!(error, previous);
            }
            previous = Some(error);
            assert_eq!(a.identity(), &a_identity);
            assert_eq!(serde_json::to_value(&a.skill_data).unwrap(), a_skills);
        }
        fixture.write(path, before);
        assert_eq!(fixture.load().identity(), &a_identity);
    }
}

#[test]
fn external_late_generated_setup_failure_cannot_publish_or_replace_a() {
    use crate::game::data::{
        pve_data::{PveWaveActivationData, PveWaveData, PveWaveEnemyData, PveWaveSource},
        run_policy_data::RunPolicyData,
    };
    let fixture = ExternalFixture::official();
    let a = fixture.load();
    let a_identity = a.identity().clone();
    let path = "official/data/run/policy.ron";
    let original = fixture.read(path);
    let mut policy: RunPolicyData = ron::de::from_str(&original).unwrap();
    policy
        .combat_difficulty
        .profiles
        .iter_mut()
        .find(|p| p.id == "default_facility")
        .unwrap()
        .ordinary_bands[0]
        .tuning
        .extra_waves
        .push(PveWaveData {
            id: "invalid_manifested_spawn_zone".into(),
            activation: PveWaveActivationData::at_time("invalid_manifested_spawn_zone", 5_000),
            spawn_zone_ids: vec!["missing_manifested_spawn_zone".into()],
            route_id: Some("defense_main".into()),
            required_for_victory: true,
            source: PveWaveSource::Manual(vec![PveWaveEnemyData::CorrodedEmployee {
                profile_id: "corroded_guard".into(),
                tier: crate::game::enums::Tier::I,
                count: 1,
            }]),
        });
    fixture.write(path, ron::ser::to_string(&policy).unwrap());
    let error = match load(
        fixture.root(),
        &fixture.selection(),
        &InputLimits::default(),
    ) {
        Ok(_) => panic!("invalid late generated setup published"),
        Err(error) => error,
    };
    assert_eq!(error.domain, "generated_combat_preview");
    assert_eq!(error.source_file, path);
    assert_eq!(error.field_path.as_deref(), Some(
        "combat_difficulty.profiles[1].ordinary_bands[0].tuning.extra_waves[0].spawn_zone_ids[0]"
    ));
    assert!(error.related_sources.iter().any(|source| source.source_file
        == "official/data/pve/encounters.ron"
        && source.object_id.as_deref() == Some("normal_corroded_patrol")));
    assert!(error.cause.contains("manifested difficulty"), "{error}");
    assert!(
        error
            .cause
            .contains("spawn wave references missing spawn zone"),
        "{error}"
    );
    assert_eq!(a.identity(), &a_identity);
    assert!(a
        .run_policy
        .combat_difficulty
        .profiles
        .iter()
        .find(|p| p.id == "default_facility")
        .unwrap()
        .ordinary_bands[0]
        .tuning
        .extra_waves
        .iter()
        .all(|wave| wave.id != "invalid_manifested_spawn_zone"));
    fixture.write(path, original);
    assert_eq!(fixture.load().identity(), &a_identity);
}

#[test]
fn custom_policy_generated_failures_keep_winning_wave_fields_and_contributors_on_retry() {
    use crate::game::data::{
        pve_data::{PveWaveActivationData, PveWaveData, PveWaveEnemyData, PveWaveSource},
        run_policy_data::{CombatNodeDifficultyRole, RunPolicyData},
    };
    let fixture = ExternalFixture::official();
    let a = fixture.load();
    let a_identity = a.identity().clone();
    let a_policy = serde_json::to_value(&a.run_policy).unwrap();
    let original = fixture.read("official/data/run/policy.ron");
    fixture.write("custom/policy.ron", &original);
    fixture.manifest(
        "custom",
        PackManifest {
            pack_id: "custom".into(),
            pack_version: "1".into(),
            namespace: PackNamespace::Global,
            contract: ContentContract::current(),
            required_capabilities: vec![],
            dependencies: vec!["official".into()],
            files: vec![CatalogFile {
                kind: CatalogKind::RunPolicy,
                path: "policy.ron".into(),
            }],
            operations: vec![DefinitionOperation {
                target: DefinitionKey {
                    kind: DefinitionKind::RunPolicy,
                    id: "run_policy".into(),
                },
                operation: OperationKind::Replace,
                after: vec!["official".into()],
            }],
        },
    );
    let selection = ContentSelection {
        packs: vec!["custom".into(), "official".into()],
        winners: vec![],
    };
    let valid_b = load(fixture.root(), &selection, &InputLimits::default()).unwrap();
    for role in [false, true] {
        for missing_zone in [true, false] {
            let mut policy: RunPolicyData = ron::de::from_str(&original).unwrap();
            let (profile_index, profile) = policy
                .combat_difficulty
                .profiles
                .iter_mut()
                .enumerate()
                .find(|(_, p)| p.id == "default_facility")
                .unwrap();
            let (tuning, tuning_path) = if role {
                let (index, entry) = profile
                    .role_tuning
                    .iter_mut()
                    .enumerate()
                    .find(|(_, entry)| {
                        entry.role == CombatNodeDifficultyRole::Revival
                    })
                    .unwrap();
                (
                    &mut entry.tuning,
                    format!(
                        "combat_difficulty.profiles[{profile_index}].role_tuning[{index}].tuning"
                    ),
                )
            } else {
                (
                    &mut profile.ordinary_bands[0].tuning,
                    format!("combat_difficulty.profiles[{profile_index}].ordinary_bands[0].tuning"),
                )
            };
            let wave_index = tuning.extra_waves.len();
            tuning.extra_waves.push(PveWaveData {
                id: "custom_late_wave".into(),
                activation: PveWaveActivationData::at_time("custom_late_wave", 5_000),
                spawn_zone_ids: if missing_zone {
                    vec!["missing_zone".into()]
                } else {
                    vec![]
                },
                route_id: Some(
                    if missing_zone {
                        "defense_main"
                    } else {
                        "missing_route"
                    }
                    .into(),
                ),
                required_for_victory: true,
                source: PveWaveSource::Manual(vec![PveWaveEnemyData::CorrodedEmployee {
                    profile_id: "corroded_guard".into(),
                    tier: crate::game::enums::Tier::I,
                    count: 1,
                }]),
            });
            let invalid = ron::ser::to_string(&policy).unwrap();
            fixture.write("custom/policy.ron", &invalid);
            let field = if missing_zone {
                "spawn_zone_ids[0]"
            } else {
                "route_id"
            };
            let expected = format!("{tuning_path}.extra_waves[{wave_index}].{field}");
            let mut previous = None;
            for _ in 0..2 {
                let error = match load(fixture.root(), &selection, &InputLimits::default()) {
                    Ok(_) => panic!("invalid generated custom wave published"),
                    Err(error) => error,
                };
                assert_eq!(error.domain, "generated_combat_preview", "{error}");
                assert_eq!(error.source_file, "custom/policy.ron", "{error}");
                assert_eq!(error.phase, LiveRonPhase::GeneratedSetup);
                assert_eq!(error.code, "generated_setup");
                assert_eq!(error.pack_id.as_deref(), Some("custom"));
                assert_eq!(error.definition_kind, Some(DefinitionKind::RunPolicy));
                assert_eq!(error.object_id.as_deref(), Some("run_policy"));
                assert_eq!(
                    error.field_path.as_deref(),
                    Some(expected.as_str()),
                    "{error}"
                );
                assert!(
                    error.related_sources.iter().any(|source| source.source_file
                        == "official/data/pve/encounters.ron"
                        && source.field_path.as_deref() == Some(if role { "encounters[9]" } else { "encounters[0]" })),
                    "{error}"
                );
                let template_source = error
                    .related_sources
                    .iter()
                    .find(|source| {
                        source.source_file == "official/data/map/battlefield_templates.ron"
                    })
                    .unwrap();
                let template_index = a
                    .map_catalogs
                    .battlefield_templates
                    .templates
                    .iter()
                    .position(|template| {
                        Some(template.id.as_str()) == template_source.object_id.as_deref()
                    })
                    .unwrap();
                assert_eq!(
                    template_source.field_path.as_deref(),
                    Some(format!("[{template_index}]").as_str()),
                    "{error}"
                );
                if role {
                    assert!(error
                        .related_sources
                        .iter()
                        .any(|source| source.source_file == "official/data/pve/encounters.ron"
                            && source.field_path.as_deref() == Some("encounters[9].reward_sources.revival")));
                }
                if let Some(previous) = previous {
                    assert_eq!(error, previous);
                }
                previous = Some(error);
                assert_eq!(a.identity(), &a_identity);
                assert_eq!(serde_json::to_value(&a.run_policy).unwrap(), a_policy);
                assert_eq!(fixture.read("custom/policy.ron"), invalid);
                assert_eq!(fixture.read("official/data/run/policy.ron"), original);
            }
            fixture.write("custom/policy.ron", &original);
            let restored = load(fixture.root(), &selection, &InputLimits::default()).unwrap();
            assert_eq!(restored.identity(), valid_b.identity());
            assert_eq!(restored.source_receipt(), valid_b.source_receipt());
        }
    }
}

#[test]
fn custom_encounter_generated_failure_keeps_original_field_and_policy_template_context() {
    use crate::game::data::pve_data::PveEncounterDatabase;
    let fixture = ExternalFixture::official();
    let a = fixture.load();
    let identity = a.identity().clone();
    let original = fixture.read("official/data/pve/encounters.ron");
    let mut catalog: PveEncounterDatabase = ron::de::from_str(&original).unwrap();
    let mut encounter = catalog.encounters.remove(0);
    let id = encounter.id.clone();
    let before = serde_json::to_value(&a.pve_data).unwrap();
    encounter.waves[0].spawn_zone_ids = vec!["missing_custom_encounter_zone".into()];
    let custom = ron::ser::to_string(&PveEncounterDatabase::new(vec![encounter])).unwrap();
    fixture.write("custom/encounter.ron", &custom);
    fixture.manifest(
        "custom",
        PackManifest {
            pack_id: "custom".into(),
            pack_version: "1".into(),
            namespace: PackNamespace::Global,
            contract: ContentContract::current(),
            required_capabilities: vec![],
            dependencies: vec!["official".into()],
            files: vec![CatalogFile {
                kind: CatalogKind::PveEncounters,
                path: "encounter.ron".into(),
            }],
            operations: vec![DefinitionOperation {
                target: DefinitionKey {
                    kind: DefinitionKind::PveEncounter,
                    id: id.clone(),
                },
                operation: OperationKind::Replace,
                after: vec!["official".into()],
            }],
        },
    );
    let selection = ContentSelection {
        packs: vec!["official".into(), "custom".into()],
        winners: vec![],
    };
    let mut previous = None;
    for _ in 0..2 {
        let error = match load(fixture.root(), &selection, &InputLimits::default()) {
            Ok(_) => panic!("invalid custom encounter published"),
            Err(error) => error,
        };
        assert_eq!(error.source_file, "custom/encounter.ron", "{error}");
        assert_eq!(error.object_id.as_deref(), Some(id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some("encounters[0].waves[0].spawn_zone_ids[0]")
        );
        assert!(error
            .related_sources
            .iter()
            .any(|source| source.source_file == "official/data/run/policy.ron"));
        assert!(error
            .related_sources
            .iter()
            .any(|source| source.source_file == "official/data/map/battlefield_templates.ron"));
        if let Some(previous) = previous {
            assert_eq!(error, previous);
        }
        previous = Some(error);
        assert_eq!(a.identity(), &identity);
        assert_eq!(serde_json::to_value(&a.pve_data).unwrap(), before);
        assert_eq!(fixture.read("official/data/pve/encounters.ron"), original);
        assert_eq!(fixture.read("custom/encounter.ron"), custom);
    }
}

#[test]
fn custom_boss_pattern_generated_setup_failure_keeps_used_pattern_and_behavior_sources() {
    use crate::game::data::boss_runtime_data::{
        BossRuntimeCatalogData, BossSummonPlacementData, BossTaskData,
    };
    let fixture = ExternalFixture::official();
    let a = fixture.load();
    let identity = a.identity().clone();
    let original = fixture.read("official/data/boss_runtime/base.ron");
    let catalog: BossRuntimeCatalogData = ron::de::from_str(&original).unwrap();
    let pattern = catalog
        .patterns
        .into_iter()
        .find(|pattern| pattern.id.as_str() == "cwbs_reference_summon_cycle")
        .unwrap();
    let mut custom = BossRuntimeCatalogData {
        patterns: vec![pattern],
        ..Default::default()
    };
    let valid = ron::ser::to_string(&custom).unwrap();
    fixture.write("custom/pattern.ron", &valid);
    fixture.manifest(
        "custom",
        PackManifest {
            pack_id: "custom".into(),
            pack_version: "1".into(),
            namespace: PackNamespace::Global,
            contract: ContentContract::current(),
            required_capabilities: vec![],
            dependencies: vec!["official".into()],
            files: vec![CatalogFile {
                kind: CatalogKind::BossRuntime,
                path: "pattern.ron".into(),
            }],
            operations: vec![DefinitionOperation {
                target: DefinitionKey {
                    kind: DefinitionKind::BossPattern,
                    id: "cwbs_reference_summon_cycle".into(),
                },
                operation: OperationKind::Replace,
                after: vec!["official".into()],
            }],
        },
    );
    let selection = ContentSelection {
        packs: vec!["official".into(), "custom".into()],
        winners: vec![],
    };
    let b = load(fixture.root(), &selection, &InputLimits::default()).unwrap();
    for missing_zone in [true, false] {
        custom = ron::de::from_str(&valid).unwrap();
        let BossTaskData::Summon {
            placement,
            route_id,
            ..
        } = &mut custom.patterns[0].states[0].tasks[0]
        else {
            panic!("reference Summon fixture");
        };
        if missing_zone {
            *placement = BossSummonPlacementData::SpawnZone {
                zone_id: "missing_custom_summon_zone".into(),
            };
        } else {
            *route_id = Some("missing_custom_summon_route".into());
        }
        let invalid = ron::ser::to_string(&custom).unwrap();
        fixture.write("custom/pattern.ron", &invalid);
        let mut previous = None;
        for _ in 0..2 {
            let error = match load(fixture.root(), &selection, &InputLimits::default()) {
                Ok(_) => panic!("invalid custom summon setup published"),
                Err(error) => error,
            };
            assert_eq!(error.domain, "generated_combat_preview", "{error}");
            assert!(error.cause.contains("resolved battle setup"), "{error}");
            assert_eq!(error.phase, LiveRonPhase::GeneratedSetup);
            assert_eq!(error.code, "generated_setup");
            let source = error
                .related_sources
                .iter()
                .find(|source| source.source_file == "custom/pattern.ron")
                .unwrap();
            assert_eq!(source.pack_id.as_deref(), Some("custom"));
            assert_eq!(source.catalog_kind, Some(CatalogKind::BossRuntime));
            assert_eq!(source.definition_kind, Some(DefinitionKind::BossPattern));
            assert!(
                error.cause.contains(if missing_zone {
                    "missing summon spawn zone"
                } else {
                    "missing summon route"
                }),
                "{error}"
            );
            assert!(
                error
                    .related_sources
                    .iter()
                    .any(|source| source.source_file == "custom/pattern.ron"
                        && source.object_id.as_deref() == Some("cwbs_reference_summon_cycle")
                        && source.field_path.as_deref() == Some("patterns[0]")),
                "{error}"
            );
            assert!(
                error.related_sources.iter().any(|source| source.source_file
                    == "official/data/boss_runtime/base.ron"
                    && source.object_id.as_deref() == Some("cwbs_reference_boss_behavior")
                    && source.field_path.as_deref() == Some("actor_behaviors[1]")),
                "{error}"
            );
            if let Some(previous) = previous {
                assert_eq!(error, previous);
            }
            previous = Some(error);
            assert_eq!(a.identity(), &identity);
            assert_eq!(
                fixture.read("official/data/boss_runtime/base.ron"),
                original
            );
            assert_eq!(fixture.read("custom/pattern.ron"), invalid);
        }
        fixture.write("custom/pattern.ron", &valid);
        let restored = load(fixture.root(), &selection, &InputLimits::default()).unwrap();
        assert_eq!(restored.identity(), b.identity());
        assert_eq!(restored.source_receipt(), b.source_receipt());
    }
}

#[cfg(test)]
mod authoring_generation_lifetime {
    use super::*;
    use crate::{
        application::{
            ApplicationCommand, ApplicationDocument, ApplicationExecutionContext,
            ApplicationSnapshotKind, CoreApplication,
        },
        authoring::{
            self, AuthoringContent, CancellationToken, TrialLimits, TrialScenario, TrialSession,
        },
    };
    use serde_json::{json, Value};
    use uuid::Uuid;
    fn lifetime_command(app: &mut CoreApplication, player: Uuid, input: Value) -> Value {
        serde_json::to_value(
            app.execute(
                player,
                ApplicationCommand::from_document(ApplicationDocument::new(input)).unwrap(),
                ApplicationExecutionContext::default(),
            )
            .unwrap(),
        )
        .unwrap()
    }
    fn lifetime_scenario(content: &AuthoringContent) -> Value {
        let player = Uuid::from_u128(0xCA03_0001);
        let mut app = CoreApplication::new(content.token().clone(), 77);
        let start = lifetime_command(
            &mut app,
            player,
            json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":false}),
        );
        lifetime_command(
            &mut app,
            player,
            json!({"type":"advance_visual_novel","session_id":start["payload"]["visual_novel"]["session_id"],
        "current_scene_id":start["payload"]["visual_novel"]["current_scene_id"]}),
        );
        let snapshot =
            serde_json::to_value(app.snapshot(ApplicationSnapshotKind::Run).unwrap()).unwrap();
        let context = &snapshot["read_model"]["game_state_context"];
        let count = context["required_count"].as_u64().unwrap() as usize;
        let ids: Vec<_> = context["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .take(count)
            .map(|entry| entry["employee_definition_id"].clone())
            .collect();
        assert_eq!(ids.len(), count);
        let selected = lifetime_command(
            &mut app,
            player,
            json!({"type":"select_starter_employees","employee_definition_ids":ids,"loan_copy_ids":[]}),
        );
        let node = selected["payload"]["map"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .position(|node| node["state"] == "Available" && node["category"] == "Combat")
            .unwrap();
        let bindings: Vec<_> = (0..count).map(|index| json!({"from":{"step":2,
        "pointer":format!("/read_model/game_state_context/candidates/{index}/employee_definition_id")},
        "to":format!("/command/employee_definition_ids/{index}"),"value_type":"string"})).collect();
        json!({"initial":{"player_id":player,"run_seed":77,"profile":"default_memory"},"preparation_steps":[
        {"kind":"command","command":{"type":"start_new_game","game_mode":"Expedition","replace_active_run":false}},
        {"kind":"command","command":{"type":"advance_visual_novel","session_id":null,"current_scene_id":null},"bindings":[
            {"from":{"step":0,"pointer":"/payload/visual_novel/session_id"},"to":"/command/session_id","value_type":"string"},
            {"from":{"step":0,"pointer":"/payload/visual_novel/current_scene_id"},"to":"/command/current_scene_id","value_type":"string"}]},
        {"kind":"snapshot","snapshot":"run"},
        {"kind":"command","command":{"type":"select_starter_employees","employee_definition_ids":vec![Value::Null;count],"loan_copy_ids":[]},"bindings":bindings},
        {"kind":"command","command":{"type":"select_map_node","node_id":null},"bindings":[
            {"from":{"step":3,"pointer":format!("/payload/map/nodes/{node}/id")},"to":"/command/node_id","value_type":"string"}]},
        {"kind":"command","command":{"type":"confirm_enter_node"}},
        {"kind":"command","command":{"type":"pause_battle"},"context":{"expected_battle_authority":{"battle_uuid":null,"battle_state_revision":null}},"bindings":[
            {"from":{"step":5,"pointer":"/payload/battle_uuid"},"to":"/context/expected_battle_authority/battle_uuid","value_type":"string"},
            {"from":{"step":5,"pointer":"/payload/battle_update/battle_state_revision"},"to":"/context/expected_battle_authority/battle_state_revision","value_type":"number"}]},
        {"kind":"clock","clock":{"mode":"host_tick","delta_ms":1,"after_seq":0}},
        {"kind":"assert","from":{"step":5,"pointer":"/kind"},"assertion":{"test":"type","value_type":"string"}}
    ],"steps":[{"kind":"snapshot","snapshot":"run"},{"kind":"clock","clock":{"mode":"next_event_bucket"}}]})
    }

    #[test]
    fn real_trial_replacement_failures_and_last_pin_reclaim_generations() {
        let fixture = ExternalFixture::official_with_trial_route();
        let cancel = CancellationToken::default();
        let load = || {
            authoring::validate(
                fixture.root(),
                &fixture.selection(),
                &Default::default(),
                &cancel,
            )
            .unwrap()
        };
        let a = load();
        let weak_a = Arc::downgrade(&a.token().0);
        let pin_a = a.token().clone();
        let raw = serde_json::to_vec(&lifetime_scenario(&a)).unwrap();
        let parse = || TrialScenario::from_json_slice(&raw, &Default::default()).unwrap();
        let mut session = TrialSession::prepare(a, parse(), Default::default(), &cancel).unwrap();
        session.advance(&cancel).unwrap();
        let before = session
            .receipt_documents()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let before_work = session.accepted_work();
        let before_index = session.next_step_index();
        let before_snapshot = session
            .snapshot(ApplicationSnapshotKind::Run, &Default::default(), &cancel)
            .unwrap()
            .document()
            .to_owned();
        let reference_content = load();
        let weak_reference = Arc::downgrade(&reference_content.token().0);
        let reference =
            TrialSession::prepare(reference_content, parse(), Default::default(), &cancel).unwrap();
        let receipt_cap = reference.receipt_bytes() - 1;
        drop(reference);
        assert!(weak_reference.upgrade().is_none());
        let mut errors = Vec::new();
        for mode in ["cancelled", "late_receipt_cap", "fuel"] {
            let b = load();
            let weak_b = Arc::downgrade(&b.token().0);
            let token = CancellationToken::default();
            let mut limits = TrialLimits::default();
            match mode {
                "cancelled" => token.cancel(),
                "late_receipt_cap" => limits.max_receipt_bytes = receipt_cap,
                "fuel" => limits.execution.max_work = 1,
                _ => unreachable!(),
            }
            let error = session.replace(b, parse(), limits, &token).unwrap_err();
            let rendered =
                authoring::render_trial_failure(&error, &Default::default(), &cancel).unwrap();
            if mode == "late_receipt_cap" {
                assert!(
                    rendered.document().contains("authoring_receipt_limit"),
                    "{}",
                    rendered.document()
                );
            }
            errors.push(error);
            assert!(
                weak_b.upgrade().is_none(),
                "failed {mode} retained generation"
            );
            assert!(weak_a.upgrade().is_some());
            assert_eq!(session.receipt_documents().collect::<Vec<_>>(), before);
            assert_eq!(session.accepted_work(), before_work);
            assert_eq!(session.next_step_index(), before_index);
            assert_eq!(
                session
                    .snapshot(ApplicationSnapshotKind::Run, &Default::default(), &cancel)
                    .unwrap()
                    .document(),
                before_snapshot
            );
        }
        let b = load();
        let weak_b = Arc::downgrade(&b.token().0);
        session
            .replace(b, parse(), Default::default(), &cancel)
            .unwrap();
        assert!(weak_a.upgrade().is_some());
        drop(pin_a);
        assert!(weak_a.upgrade().is_none());
        assert!(weak_b.upgrade().is_some());
        session.run_to_end(&cancel).unwrap();
        drop(session);
        assert!(weak_b.upgrade().is_none());
        assert_eq!(errors.len(), 3);
    }
}

#[path = "../../../../docs/goals/core_mod_validation_apply/execution/single_goal/test_parts/runtime001/shared/issuer_generation_lifetime.rs"]
mod u11_issuer_lifetime;

#[test]
fn compiled_graph_artifact_loads_in_external_pack_and_rejects_tampering() {
    let fixture = ExternalFixture::official();
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../content_assembler/docs/examples/compiler-preflight/G01.node-graph.json"));
    let artifact = crate::authoring::node_graph::compile(source, "p7_compiled_use".into()).unwrap().to_ron().unwrap();
    let path = "official/data/skills/p7.skill-graph.ron";
    fixture.write(path, &artifact);
    let mut manifest: PackManifest = ron::de::from_str(&fixture.read("official/pack.ron")).unwrap();
    manifest.files.push(CatalogFile { kind: CatalogKind::Skills, path: "data/skills/p7.skill-graph.ron".into() });
    fixture.manifest("official", manifest);
    let accepted = fixture.load();
    assert_eq!(accepted.skill_data.get_by_id("p7_compiled_use").unwrap().resonance_admission.unwrap().capacity, 100);
    let provenance = &accepted.source_metadata().unwrap().definitions().iter().find(|(key, _)| key.id == "p7_compiled_use").unwrap().1;
    assert_eq!(provenance.source_file, path);
    for bad in [artifact.replace("version: 1,", "version: 999,"), artifact.replace("native-manual-employee-v1", "untrusted-target"), artifact.replace("capacity: 100", "capacity: 0")] {
        assert_ne!(bad, artifact);
        fixture.write(path, bad);
        let error = load(fixture.root(), &fixture.selection(), &InputLimits::default()).err().unwrap();
        assert_eq!(error.source_file, path);
        assert_eq!(error.code, "graph_artifact_invalid");
        assert!(accepted.skill_data.get_by_id("p7_compiled_use").is_some());
    }
}

#[test]
fn composition_yaml_artifact_publishes_through_real_pack_loader_and_rejects_bad_envelopes() {
    let fixture = ExternalFixture::official();
    let yaml = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/woodsman_monk.skill.yaml"));
    let artifact = crate::authoring::composition::compile_yaml(yaml).unwrap().to_ron().unwrap();
    let path = "official/data/skills/authored.skill-composition.ron";
    fixture.write(path, &artifact);
    let mut manifest: PackManifest = ron::de::from_str(&fixture.read("official/pack.ron")).unwrap();
    manifest.files.push(CatalogFile { kind: CatalogKind::Skills,
        path: "data/skills/authored.skill-composition.ron".into() });
    fixture.manifest("official", manifest);
    let accepted = fixture.load();
    for id in ["woodsman-authored", "monk-authored"] {
        let skill = accepted.skill_data.get_by_id(id).unwrap();
        assert!(!skill.steps.is_empty());
        let provenance = accepted.source_metadata().unwrap().definitions().iter()
            .find(|(key, _)| key.id == id).unwrap();
        assert_eq!(provenance.1.source_file, path);
        for effect in skill.steps.iter().flat_map(|step| &step.effects) {
            assert_eq!(effect.source.as_ref().unwrap().source_file, path);
        }
    }
    for invalid in [artifact.replacen("version: 1,", "version: 999,", 1),
        artifact.replace("skill-composition-native", "unsupported-composition")] {
        assert_ne!(invalid, artifact);
        fixture.write(path, invalid);
        let error = load(fixture.root(), &fixture.selection(), &InputLimits::default()).err().unwrap();
        assert_eq!(error.code, "composition_artifact_invalid");
        assert_eq!(error.source_file, path);
        assert!(accepted.skill_data.get_by_id("monk-authored").is_some());
    }
}

#[test]
fn composition_preview_uses_captured_base_and_enforces_overlay_inventory_limits() {
    let fixture = ExternalFixture::official();
    let settings = crate::runtime_safety::RuntimeSafetySettings::embedded_shared();
    let limits = InputLimits::default();
    let artifact = crate::authoring::composition::compile_yaml(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/woodsman_monk.skill.yaml"))).unwrap();
    let base = capture_composition_base(fixture.root(), &fixture.selection(), &limits, settings.clone()).unwrap();
    let constrained = capture_composition_base(fixture.root(), &fixture.selection(), &limits, settings.clone()).unwrap();
    fixture.write("official/data/skills/base.ron", "invalid content after capture");
    let published = load_composition_preview(base, &fixture.selection(), &artifact, &limits, settings.clone()).unwrap();
    assert!(published.token.skill_data.get_by_id("monk-authored").is_some());
    assert!(published.token.source_receipt().files.iter().any(|f| f.path == published.artifact_path && f.sha256 == published.artifact_digest));
    let mut low = limits;
    low.max_packs = 1;
    let error = load_composition_preview(constrained, &fixture.selection(), &artifact, &low, settings).err().unwrap();
    assert_eq!(error.code, "composition_preview_invalid");
    assert!(error.cause.contains("input limits"));
    assert!(published.token.skill_data.get_by_id("monk-authored").is_some());
    assert_eq!(fixture.read("official/data/skills/base.ron"), "invalid content after capture");
}

#[test]
fn composition_resource_publication_preserves_unrelated_definitions_and_requires_explicit_replacement() {
    let fixture = ExternalFixture::official();
    let base_resources = "(resources:[(id:\"employee_meter\",owner:Employee,lifetime:Run,initial:1,cap:4),(id:\"unrelated\",owner:Employee,lifetime:Run,initial:2,cap:9)])";
    fixture.write("official/data/resources/base.ron", base_resources);
    let base = fixture.load();
    let yaml = include_str!("../../../../tests/fixtures/skill_composition/resources.skill.yaml");
    let artifact = crate::authoring::composition::compile_yaml(yaml).unwrap();
    let path = "authored/resources.skill-composition.ron";
    fixture.write(path, artifact.to_ron().unwrap());
    let key = DefinitionKey { kind: DefinitionKind::Resource, id: "employee_meter".into() };
    let mut manifest = PackManifest {
        pack_id: "authored".into(), pack_version: "1".into(), namespace: PackNamespace::Global,
        contract: ContentContract::current(),
        required_capabilities: vec!["typed_resources_v1".into(), "structured_skill_flow_v1".into()],
        dependencies: vec!["official".into()],
        files: vec![CatalogFile { kind: CatalogKind::Skills, path: "resources.skill-composition.ron".into() }],
        operations: vec![],
    };
    fixture.manifest("authored", manifest.clone());
    let selection = ContentSelection { packs: vec!["official".into(), "authored".into()], winners: vec![] };
    let error = load(fixture.root(), &selection, &InputLimits::default()).err().unwrap();
    assert!(error.to_string().contains("employee_meter"), "{error:?}");
    assert_eq!(base.resource_data.resources[0].cap, 4);
    manifest.operations.push(DefinitionOperation { target: key.clone(), operation: OperationKind::Replace, after: vec!["official".into()] });
    fixture.manifest("authored", manifest);
    let selection = ContentSelection { winners: vec![WriterSelection { target: key, pack_id: "authored".into() }], ..selection };
    let accepted = load(fixture.root(), &selection, &InputLimits::default()).unwrap();
    assert_eq!(accepted.resource_data.resources.len(), 4);
    assert_eq!(accepted.resource_data.resources.iter().find(|r| r.id.as_str() == "unrelated").unwrap().initial, 2);
    assert_eq!(accepted.resource_data.resources.iter().find(|r| r.id.as_str() == "employee_meter").unwrap().cap, 15);
    assert_ne!(base.identity(), accepted.identity());
    let modified = crate::authoring::composition::compile_yaml(&yaml.replacen("initial: 7, cap: 15", "initial: 7, cap: 14", 1)).unwrap();
    fixture.write(path, modified.to_ron().unwrap());
    let changed = load(fixture.root(), &selection, &InputLimits::default()).unwrap();
    assert_ne!(accepted.identity(), changed.identity());
    assert_eq!(accepted.resource_data.resources.iter().find(|r| r.id.as_str() == "employee_meter").unwrap().cap, 15);

    // An authoring trial replaces the requested resource in its isolated
    // candidate through the same resolver, preserving captured base files.
    let settings = crate::runtime_safety::RuntimeSafetySettings::embedded_shared();
    let limits = InputLimits::default();
    let captured = capture_composition_base(fixture.root(), &fixture.selection(), &limits, settings.clone()).unwrap();
    let preview = load_composition_preview(captured, &fixture.selection(), &artifact, &limits, settings).unwrap();
    assert_eq!(preview.token.resource_data.resources.len(), 4);
    assert_eq!(fixture.read("official/data/resources/base.ron"), base_resources);
    assert_eq!(preview.token.resource_data.resources.iter().find(|r| r.id.as_str() == "employee_meter").unwrap().cap, 15);
}

#[test]
fn optional_fragment_pack_publication_preserves_none_and_rejects_mismatch_atomically() {
    let fixture = ExternalFixture::official();
    let accepted = fixture.load();
    let source = accepted.pve_data.get_reward_source_by_encounter_id("suppress_big_bad_wolf").unwrap();
    assert!(source.fragment_id.is_none());
    let plan = crate::game::encounter_progression::EncounterProgressionState::default()
        .plan_encounter_reward(&accepted.pve_data, "suppress_big_bad_wolf").unwrap();
    assert!(plan.effects.is_empty());
    let path = "official/data/pve/encounters.ron";
    let mut pve: crate::game::data::pve_data::PveEncounterDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
    pve.reward_sources.iter_mut().find(|s| s.reward_source_id == "normal_boss_big_bad_wolf").unwrap().fragment_id = Some("fragment_big_bad_wolf_trace".into());
    fixture.write(path, ron::ser::to_string(&pve).unwrap());
    let error = load(fixture.root(), &fixture.selection(), &InputLimits::default()).err().unwrap();
    assert_eq!(error.source_file, path);
    assert!(error.field_path.as_deref().is_some_and(|p| p.ends_with("fragment_id")));
    assert!(accepted.pve_data.get_reward_source_by_encounter_id("suppress_big_bad_wolf").unwrap().fragment_id.is_none());
}
