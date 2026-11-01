use super::*;
#[test]
fn composition_headshot_resonance_world_real_copy_platform_cast_and_reward_edit() {
    headshot_world(false);
}

#[test]
fn canonical_scarecrow_published_original_drives_actual_fragment_copy_and_reward() {
    headshot_world(true);
}

fn headshot_world(published: bool) {
    let (fixture, _, mut document) = state_world_fixture();
    let equipment_id = "scarecrow_fixture_gun";
    let path = "official/data/equipments/base.ron";
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut accessory = equipment
        .items
        .iter()
        .find(|i| i.equipment_type == crate::game::data::equipment_data::EquipmentType::Weapon)
        .unwrap()
        .clone();
    accessory.id = equipment_id.into();
    accessory.uuid = uuid::Uuid::from_u128(0xe1000);
    accessory.name = "수확 1발 탄창과 파편 보상 검증값".into();
    accessory.triggered_effects.clear();
    accessory.ability_activations.clear();
    accessory.weapon_profile = Some(
        ron::de::from_str(include_str!(
            "../../../../tests/fixtures/skill_composition/scarecrow.weapon.ron"
        ))
        .unwrap(),
    );
    equipment.items.push(accessory);
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/events/rewards/base.ron";
    fixture.write(
        path,
        fixture.read(path).replacen(
            "GrantEnkephalin(amount: 100),",
            &format!(
                "GrantEnkephalin(amount: 100), GrantEquipment(equipment_id: \"{equipment_id}\"),"
            ),
            1,
        ),
    );
    fixture.author_trial_facility();
    // Preserve the opening victory and actual copy grant. Insert equip before
    // the next encounter; remap every later authority reference together.
    let preparation = document["preparation_steps"].as_array().unwrap().len();
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(25 - preparation);
    fn shift(value: &mut Value) {
        match value {
            Value::Object(map) => {
                for (key, value) in map {
                    if key == "step" && value.as_u64().is_some_and(|n| n >= 14) {
                        *value = json!(value.as_u64().unwrap() + 1);
                    } else {
                        shift(value);
                    }
                }
            }
            Value::Array(values) => {
                for value in values {
                    shift(value);
                }
            }
            _ => (),
        }
    }
    shift(&mut document);
    document["steps"].as_array_mut().unwrap().insert(5, json!({
        "kind":"command", "command":{"type":"equip_item","item_uuid":null,"target_unit":null},
        "bindings":[
            {"from":{"step":13,"pointer":"/payload/inventory_diff/added/0/Equipment/uuid"},"to":"/command/item_uuid","value_type":"string"},
            {"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/target_unit","value_type":"string"}
        ]
    }));
    let mut source =
        include_str!("../../../../tests/fixtures/skill_composition/scarecrow_headshot.skill.yaml")
            .replace(
                "equipment: r11_weapon",
                &format!("equipment: {equipment_id}"),
            )
            .replace("authored-prepared-state", "fragment_freischutz_black_round")
            .replace("capacity: 100", "capacity: 5")
            .replace("numerator: 5", "numerator: 100")
            .replace("value: 20, unit: Magnitude", "value: 2, unit: Magnitude");
    let path = "official/data/skill_fragments/base.ron";
    let mut fragments: crate::game::data::skill_fragment_data::SkillFragmentDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let mut updated = 0;
    for fragment in &mut fragments.fragments {
        if let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
            forms,
            ..
        } = &mut fragment.effect
        {
            for form in forms.values_mut() {
                if form.skill_id.as_str() == "fragment_freischutz_black_round" {
                    form.resonance_max = 5;
                    updated += 1;
                }
            }
        }
    }
    assert!(updated > 0);
    fixture.write(path, ron::ser::to_string(&fragments).unwrap());
    let path = "official/data/enemies/corroded_employees.ron";
    let mut enemies: crate::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let enemy = enemies
        .profiles
        .iter_mut()
        .find(|p| p.id == "corroded_rusher")
        .unwrap();
    enemy.max_health = 2000;
    enemy.attack = 0;
    enemy.defense = 0;
    enemy.magic_resist = 0;
    for p in &mut enemies.profiles {
        if p.basic_attack_range_preset.is_some() {
            p.basic_attack.defense_tile_range = None;
        }
    }
    fixture.write(path, ron::ser::to_string(&enemies).unwrap());
    let skill_id = if published {
        "fragment_scarecrow_wisdom_harvest"
    } else {
        "fragment_freischutz_black_round"
    };
    if published {
        source = canonical_scarecrow::resolved_source(2);
        let path = "official/data/skill_fragments/base.ron";
        let mut fragments: crate::game::data::skill_fragment_data::SkillFragmentDatabase =
            ron::de::from_str(&fixture.read(path)).unwrap();
        let target = fragments
            .fragments
            .iter_mut()
            .find(|f| f.id.as_str() == "fragment_scarecrow_trace")
            .unwrap();
        let replacement = canonical_scarecrow::resolved_catalog();
        assert_eq!(target.id, replacement.id);
        assert_eq!(target.uuid, replacement.uuid);
        *target = replacement;
        fixture.write(path, ron::ser::to_string(&fragments).unwrap());
        let path = "official/data/events/rewards/base.ron";
        let rewards = fixture.read(path);
        assert!(rewards.contains("authoring_preview_fragment"));
        // Encounter-origin fragments cannot be granted by an ordinary reward.
        // Remove only the generic preview grant and let the REAL encounter
        // progression produce the canonical fragment and its existing first gear.
        let mut rewards: crate::game::data::reward_data::RewardDatabase =
            ron::de::from_str(&rewards).unwrap();
        for reward in &mut rewards.rewards {
            reward.effects.retain(|effect| {
                !matches!(effect,
                crate::game::reward::RewardEffect::GrantSkillFragment { fragment_id }
                    if fragment_id.as_str() == "authoring_preview_fragment")
            });
        }
        fixture.write(path, ron::ser::to_string(&rewards).unwrap());
        let path = "official/data/map/facility_templates.ron";
        let mut maps: crate::game::map::FacilityTemplateDatabase =
            ron::de::from_str(&fixture.read(path)).unwrap();
        let activity = maps.layouts[0]
            .activities
            .iter_mut()
            .find(|a| a.id == "trial_intake_corridor")
            .unwrap();
        activity.kind_id = crate::game::map::MapNodeKindId::new("combat_elite");
        activity.payload = crate::game::map::MapNodePayload::Encounter {
            encounter_id: Some("suppress_scarecrow".into()),
        };
        fixture.write(path, ron::ser::to_string(&maps).unwrap());
        // Real authored encounter, source identity, waves and rewards; only the
        // enemy's combat stats are reduced for this bounded acceptance scenario.
        let path = "official/data/abnormalities/base.ron";
        let mut abnormalities: crate::game::data::abnormality_data::AbnormalityDatabase =
            ron::de::from_str(&fixture.read(path)).unwrap();
        let enemy = abnormalities
            .items
            .iter_mut()
            .find(|a| a.id == "f-01-18_scarecrow")
            .unwrap();
        enemy.max_health = 70;
        enemy.attack = 0;
        enemy.defense = 0;
        enemy.magic_resist = 0;
        fixture.write(path, ron::ser::to_string(&abnormalities).unwrap());
        // The old scarecrow_wisdom_reap skill belongs to the enemy encounter.
        // Only the fragment metadata changes; retain every existing skill.
        let skills: crate::game::data::skill_data::RawSkillDatabase =
            ron::de::from_str(&fixture.read("official/data/skills/base.ron")).unwrap();
        assert!(skills
            .skills
            .iter()
            .any(|s| s.id.as_str() == "scarecrow_wisdom_reap"));
        assert!(!skills.skills.iter().any(|s| s.id.as_str() == skill_id));
        fixture.write(
            "official/data/bundle.ron",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../game_resources/data/bundle.ron"
            )),
        );
        // Weapon magazine is a separate TEST source; the canonical fragment
        // never installs/rewrites a weapon, ammunition or firing cadence.
        let mut weapon = crate::authoring::composition::parse_yaml(include_str!(
            "../../../../tests/fixtures/skill_composition/scarecrow_headshot.skill.yaml"
        ))
        .unwrap();
        weapon["id"] = json!("scarecrow-test-weapon");
        weapon["values"].as_array_mut().unwrap().remove(0);
        weapon["equipment_attacks"][0]["equipment"] = json!(equipment_id);
        weapon["definitions"] = json!([]);
        weapon["skills"] = json!([]);
        fixture.write(
            "official/data/skills/test-weapon.skill.yaml",
            serde_json::to_string_pretty(&weapon).unwrap(),
        );
        fixture.write(
            "original-fragment.skill.yaml",
            canonical_scarecrow::ORIGINAL,
        );
        fixture.write(
            "original-fragment.catalog.json",
            canonical_scarecrow::CATALOG,
        );
        fixture.write(
            "official/data/skills/scarecrow.skill.yaml",
            canonical_scarecrow::ORIGINAL,
        );
        let rejected = fixture.root().join("unresolved-original");
        let failure = crate::game::data::content_pack::package_official_with_skill_files(
            &fixture.root().join("official"),
            &rejected,
            &[
                "data/skills/scarecrow.skill.yaml".into(),
                "data/skills/test-weapon.skill.yaml".into(),
            ],
            &Default::default(),
        )
        .unwrap_err();
        assert!(format!("{failure:?}").contains("Unresolved"));
        assert!(!rejected.exists());
    }
    let cancel = CancellationToken::default();
    let mut pre = document.clone();
    pre["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(24 - preparation);
    let mut probe = HeadshotRun::prepare(
        &fixture,
        &source,
        &pre,
        published,
        "published-probe",
        &cancel,
    );
    probe.run_to_end(&cancel).unwrap();
    if published {
        let grant = probe.session().step(13).unwrap().public_result().unwrap();
        let index = grant["payload"]["inventory_diff"]["added"]
            .as_array()
            .unwrap()
            .iter()
            .position(|item| item["Equipment"]["definition_id"] == equipment_id)
            .unwrap();
        let equip = document["steps"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|step| step["command"]["type"] == "equip_item")
            .unwrap();
        let binding = equip["bindings"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|binding| binding["to"] == "/command/item_uuid")
            .unwrap();
        binding["from"]["pointer"] = json!(format!(
            "/payload/inventory_diff/added/{index}/Equipment/uuid"
        ));
    }
    let entry = probe.session().step(21).unwrap().public_result().unwrap();
    let zones = entry["payload"]["battle_setup_snapshot"]["deployment_zones"]
        .as_array()
        .unwrap();
    let platform = zones.iter().position(|z| z["kind"] == "Platform").unwrap();
    assert!(!zones[platform]["cells"].as_array().unwrap().is_empty());
    for b in document["steps"][24 - preparation]["bindings"]
        .as_array_mut()
        .unwrap()
    {
        if b["to"] == "/command/position" {
            b["from"]["pointer"] = json!(format!(
                "/payload/battle_setup_snapshot/deployment_zones/{platform}/cells/0"
            ));
        }
    }
    drop(probe);
    document["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(25 - preparation);
    // The fixture route settles at its harmless objective before 10s. The
    // earlier moving-target shot legitimately missed at 7551ms; assert the
    // stationary target before casting for this positive kill-reward case.
    document["steps"]
        .as_array_mut()
        .unwrap()
        .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":10000,"after_seq":0}}));
    let mut activate = authority_step("activate_skill", 25);
    activate["command"]["employee_uuid"] = Value::Null;
    activate["command"]["skill_id"] = json!(skill_id);
    activate["bindings"].as_array_mut().unwrap().push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},"to":"/command/employee_uuid","value_type":"string"}));
    document["steps"].as_array_mut().unwrap().extend([
        activate,
        json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":1,"after_seq":0}}),
    ]);
    for _ in 0..100 {
        document["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":100,"after_seq":0}}));
    }
    let mut expected = Vec::new();
    for reward in [2, 4] {
        let current = if published {
            canonical_scarecrow::resolved_source(reward)
        } else {
            source.replace(
                "value: 2, unit: Magnitude",
                &format!("value: {reward}, unit: Magnitude"),
            )
        };
        let mut trial = HeadshotRun::prepare(
            &fixture,
            &current,
            &document,
            published,
            &format!("published-{reward}"),
            &cancel,
        );
        while !trial.session().is_complete() {
            let pending = trial.session().next_step_index();
            let index = trial
                .advance(&cancel)
                .unwrap_or_else(|e| panic!("headshot World step {pending}: {e:?}"))
                .index();
            if index == 25 {
                let result = trial
                    .session()
                    .step(index)
                    .unwrap()
                    .public_result()
                    .unwrap();
                let units = result["payload"]["battle_update"]["checkpoint"]["units"]
                    .as_array()
                    .unwrap();
                let target = units.iter().find(|u| u["owner"] == "Opponent").unwrap();
                assert_eq!(target["position"], json!({"x":3,"y":6}));
                assert!(target["stats"]["current_health"].as_u64().unwrap() > 0);
            }
            let observation = trial.session().observation().unwrap();
            let latest = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
                .and_then(|o| o.latest_sourced_resonance());
            if index >= 28
                && latest.is_some_and(|r| {
                    r.authored_source()
                        .is_some_and(|s| s.field_path().contains("handlers[0].body[0]"))
                })
            {
                if reward == 2 {
                    document["steps"]
                        .as_array_mut()
                        .unwrap()
                        .truncate(index + 1 - preparation);
                }
                break;
            }
        }
        let grant = trial.session().step(13).unwrap().public_result().unwrap();
        let item = &grant["payload"]["inventory_diff"]["added"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["Equipment"]["definition_id"] == equipment_id)
            .unwrap()["Equipment"];
        assert_eq!(item["definition_id"], equipment_id);
        let equip = trial.session().step(14).unwrap().public_result().unwrap();
        assert_eq!(equip["disposition"], "accepted");
        assert_eq!(equip["payload"]["requested_item_uuid"], item["uuid"]);
        let deploy = trial.session().step(24).unwrap().public_result().unwrap();
        assert_eq!(deploy["disposition"], "accepted");
        let installed = &trial.session().step(27).unwrap().public_result().unwrap()["payload"]
            ["battle_update"]["checkpoint"]["sustained_effects"]["prepared_basics"];
        assert_eq!(installed.as_array().unwrap().len(), 1, "{installed}");
        assert_eq!(installed[0]["state"]["definition"], "wisdom-harvest");
        assert!(installed[0]["fragment_copy_uuid"].is_string());
        if published {
            let fragment = &grant["payload"]["skill_fragment_diffs"][0];
            assert_eq!(
                fragment["fragment_id"], "fragment_scarecrow_trace",
                "{fragment}"
            );
            assert_eq!(installed[0]["fragment_copy_uuid"], fragment["copy_uuid"]);
            assert_eq!(
                fixture.read("original-fragment.skill.yaml"),
                canonical_scarecrow::ORIGINAL
            );
            assert_eq!(
                fixture.read("original-fragment.catalog.json"),
                canonical_scarecrow::CATALOG
            );
        }
        let observation = trial.session().observation().unwrap();
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))
            .unwrap();
        let latest = observed.latest_sourced_resonance().unwrap_or_else(|| {
            let last = trial.session().next_step_index() - 1;
            let diagnostic = json!({"installed":installed,"last":trial.session().step(last).unwrap().public_result().unwrap(),"magazine":observed.latest_magazine(),"steps":document});
            if let Some(directory) = std::env::var_os(if published { "CANONICAL_SCARECROW_FIXTURE_DIR" } else { "HEADSHOT_COMPOSITION_FIXTURE_DIR" }) {
                std::fs::write(std::path::PathBuf::from(directory).with_extension("diagnostic.json"), serde_json::to_vec_pretty(&diagnostic).unwrap()).unwrap();
            }
            panic!("prepared direct kill grants resonance; last step {last}, magazine {:?}", observed.latest_magazine());
        });
        let change = serde_json::to_value(latest.change()).unwrap();
        assert_eq!(
            change["after"].as_u64().unwrap() - change["before"].as_u64().unwrap(),
            reward
        );
        let authored = latest.authored_source().unwrap();
        assert!(authored.field_path().contains("handlers[0].body[0]"));
        let mut mixed = authored.source().clone();
        mixed.site.action.definition_id = "fragment_woodsman_fierce_fury".into();
        assert!(trial.origin_for_source(&mixed).is_none());
        let mut invalid_path = authored.source().clone();
        invalid_path.field_path.push_str(".unknown_native_field");
        assert!(trial.origin_for_source(&invalid_path).is_none());
        let origin = trial.origin_for_observed(authored).unwrap();
        let magazine = observed.latest_magazine().unwrap();
        assert_eq!((magazine.capacity, magazine.remaining), (1, 0));
        assert_eq!(json!(magazine.equipment.persistent_copy_uuid), item["uuid"]);
        assert!(!magazine.enhanced);
        expected.push(json!({"reward":reward,"latest":latest,"origin":origin,"magazine":magazine,"installed":installed,"copy":item["uuid"]}));
    }
    if let Some(directory) = std::env::var_os(if published {
        "CANONICAL_SCARECROW_FIXTURE_DIR"
    } else {
        "HEADSHOT_COMPOSITION_FIXTURE_DIR"
    }) {
        let directory = std::path::PathBuf::from(directory);
        assert!(!directory.exists());
        copy_fixture(fixture.root(), &directory.join("content"));
        std::fs::write(directory.join("source.skill.yaml"), source).unwrap();
        std::fs::write(directory.join("trial.json"), serde_json::to_vec_pretty(&json!({"root":directory.join("content"),"selection":fixture.selection(),"scenario":document})).unwrap()).unwrap();
        std::fs::write(
            directory.join("expected.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
    }
}

// Test harness shares the established real combat scenario while keeping the
// legacy preview regression and normal published-content acceptance separate.
enum HeadshotRun {
    Preview(CompositionTrial),
    Published {
        session: TrialSession,
        artifact: crate::authoring::composition::Artifact,
        source_file: String,
    },
}
impl HeadshotRun {
    fn prepare(
        fixture: &ExternalFixture,
        source: &str,
        document: &Value,
        published: bool,
        name: &str,
        cancel: &CancellationToken,
    ) -> Self {
        if !published {
            return Self::Preview(
                CompositionTrial::prepare(
                    fixture.root(),
                    &fixture.selection(),
                    source,
                    checked(document),
                    None,
                    TrialLimits::default(),
                    cancel,
                )
                .unwrap(),
            );
        }
        use crate::game::data::content_pack::{
            package_official_with_skill_files, ContentSelection,
        };
        let path = "data/skills/scarecrow.skill.yaml";
        fixture.write(&format!("official/{path}"), source);
        let receipt = package_official_with_skill_files(
            &fixture.root().join("official"),
            &fixture.root().join(name),
            &[path.into(), "data/skills/test-weapon.skill.yaml".into()],
            &Default::default(),
        )
        .unwrap();
        assert_eq!(
            std::fs::read_to_string(fixture.root().join(name).join(path)).unwrap(),
            source
        );
        let content = crate::authoring::validate(
            fixture.root(),
            &ContentSelection {
                packs: vec![name.into()],
                winners: vec![],
            },
            &Default::default(),
            cancel,
        )
        .unwrap();
        assert_eq!(content.token().identity(), &receipt.content_identity);
        let session =
            TrialSession::prepare(content, checked(document), TrialLimits::default(), cancel)
                .unwrap();
        Self::Published {
            session,
            artifact: crate::authoring::composition::compile_yaml(source).unwrap(),
            source_file: format!("{name}/{path}"),
        }
    }
    fn session(&self) -> &TrialSession {
        match self {
            Self::Preview(p) => p.session(),
            Self::Published { session, .. } => session,
        }
    }
    fn advance(
        &mut self,
        cancel: &CancellationToken,
    ) -> Result<crate::authoring::StepView<'_>, crate::authoring::TrialFailure> {
        match self {
            Self::Preview(p) => p.advance(cancel),
            Self::Published { session, .. } => session.advance(cancel),
        }
    }
    fn run_to_end(
        &mut self,
        cancel: &CancellationToken,
    ) -> Result<(), crate::authoring::TrialFailure> {
        match self {
            Self::Preview(p) => p.run_to_end(cancel),
            Self::Published { session, .. } => session.run_to_end(cancel),
        }
    }
    fn origin_for_observed(
        &self,
        source: crate::authoring::AuthoredEffectView<'_>,
    ) -> Option<&crate::authoring::composition::SourceTrace> {
        self.origin_for_source(source.source())
    }
    fn origin_for_source(
        &self,
        source: &crate::game::ability::authoring::EffectSource,
    ) -> Option<&crate::authoring::composition::SourceTrace> {
        match self {
            Self::Preview(p) => p.origin_for(source),
            Self::Published { session, artifact, source_file } => {
                if &source.source_file != source_file { return None; }
                session.composition_origin(artifact, source)
            }
        }
    }
}
