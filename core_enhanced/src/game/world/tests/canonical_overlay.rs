use super::*;
use crate::authoring::composition::canonical_overlay_tests as canonical;
use crate::game::data::content_pack::{
    self, package_official_with_skill_files, tests::ExternalFixture, ContentSelection,
};
use crate::game::data::{
    equipment_data::EquipmentDatabase, relic_data::RelicDatabase,
    skill_fragment_data::SkillFragmentDatabase,
};
use crate::game::reward::RewardEffect;

// Keep the test overlay through shield expiry, the next hostile strike,
// and a following basic attack so both actual absorption and healing occur.
const HELLO_DURATION_MS: u64 = 12_000;

fn data(armor: bool) -> (ExternalFixture, ValidatedGameData) {
    let mut kept = None;
    let data = authored_route_with_source_consumer(false, None, false, false, |sources| {
        let fixture = ExternalFixture::official();
        for (path, text) in sources.source_inventory() {
            fixture.write(&format!("official/{path}"), text);
        }
        fixture.write("official/data/bundle.ron", sources.manifest);
        let path = "official/data/map/facility_templates.ron";
        let mut maps: FacilityTemplateDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
        let activity = maps.layouts[0]
            .activities
            .iter_mut()
            .find(|a| a.id == "u17_boss")
            .unwrap();
        activity.payload = MapNodePayload::Encounter {
            encounter_id: Some("suppress_nothing_there".into()),
        };
        let normal = maps.layouts[0].allocation.normal_activity_id.clone();
        maps.layouts[0]
            .activities
            .iter_mut()
            .find(|a| a.id == normal)
            .unwrap()
            .payload = MapNodePayload::Encounter {
            encounter_id: Some("normal_breach_response".into()),
        };
        fixture.write(path, ron::ser::to_string(&maps).unwrap());
        // Explicit observation layout exposes legal ground positions near the
        // stationary enemy columns without widening the weapon range.
        let path = "official/data/map/battlefield_templates.ron";
        let templates: Vec<crate::game::combat_preview::BattlefieldTemplateDefinition> =
            ron::de::from_str(&fixture.read(path)).unwrap();
        let mut source = serde_json::to_value(&templates).unwrap();
        let template = source
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|t| t["id"] == "obstacle_room_zigzag_01")
            .unwrap();
        template["rows"][1] = json!("PPPPPPPPP");
        template["rows"][2] = json!("PPPPPPPPP");
        let templates: Vec<crate::game::combat_preview::BattlefieldTemplateDefinition> =
            serde_json::from_value(source).unwrap();
        fixture.write(path, ron::ser::to_string(&templates).unwrap());
        // The source armor replaces the starter armor's +80 HP. Give the
        // observation employee explicit reserve HP in both cases so the whole
        // ordinary battle can finish after the required early-window checks.
        let path = "official/data/employees/definitions.ron";
        let mut employees: crate::game::data::employee_data::EmployeeDefinitionDatabase =
            ron::de::from_str(&fixture.read(path)).unwrap();
        employees
            .definitions
            .iter_mut()
            .find(|e| e.employee_definition_id == "scout_runner")
            .unwrap()
            .profile
            .base_combat_stats
            .max_hp = 300;
        fixture.write(path, ron::ser::to_string(&employees).unwrap());
        let path = "official/data/abnormalities/base.ron";
        let mut abnormalities: AbnormalityDatabase =
            ron::de::from_str(&fixture.read(path)).unwrap();
        let boss = abnormalities
            .items
            .iter_mut()
            .find(|a| a.id == "o-06-20_nothing_there")
            .unwrap();
        boss.max_health = 70;
        boss.attack = 0;
        boss.defense = 0;
        boss.magic_resist = 0;
        fixture.write(path, ron::ser::to_string(&abnormalities).unwrap());
        let path = "official/data/skill_fragments/base.ron";
        let mut fragments: SkillFragmentDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
        let mut metadata = canonical::fragment();
        if let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
            forms,
            ..
        } = &mut metadata.effect
        {
            forms.get_mut("base").unwrap().resonance_max = 5;
        }
        let previous = fragments
            .fragments
            .iter_mut()
            .find(|f| f.id == metadata.id)
            .unwrap();
        assert_eq!(previous.uuid, metadata.uuid);
        *previous = metadata;
        fixture.write(path, ron::ser::to_string(&fragments).unwrap());
        let path = "official/data/equipments/base.ron";
        let mut equipment: EquipmentDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
        for metadata in canonical::equipment() {
            let previous = equipment
                .items
                .iter_mut()
                .find(|e| e.id == metadata.id)
                .unwrap();
            assert_eq!(previous.uuid, metadata.uuid);
            *previous = metadata;
        }
        fixture.write(path, ron::ser::to_string(&equipment).unwrap());
        let path = "official/data/relics/base.ron";
        let mut relics: RelicDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
        for relic in canonical::relics().relics {
            assert!(relics.get_by_id(&relic.id).is_none());
            relics.relics.push(relic);
        }
        fixture.write(path, ron::ser::to_string(&relics).unwrap());
        // A separate follow-up encounter keeps living targets for real firing,
        // hostile damage and control; boss and revival waves stay intact.
        let path = "official/data/enemies/corroded_employees.ron";
        let mut enemies: CorrodedEmployeeProfileDatabase =
            ron::de::from_str(&fixture.read(path)).unwrap();
        let mut observer = enemies
            .profiles
            .iter()
            .find(|p| p.id == "corroded_rusher")
            .unwrap()
            .clone();
        observer.id = "nothing_there_gear_observation_enemy".into();
        observer.uuid = Uuid::from_u128(0xf122);
        observer.max_health = 1800;
        observer.defense = 0;
        observer.magic_resist = 0;
        observer.attack = if armor { 20 } else { 80 };
        observer.basic_attack.damage_type = crate::game::battle::damage::DamageType::Physical;
        observer.basic_attack.interval_ms = 2000;
        observer.basic_attack.windup_ms = 0;
        {
            // These stationary reinforcement positions cannot reach the rear
            // deployment row with the original melee preset. An explicit ranged
            // fixture attacks the armored employee through ordinary targeting.
            let mut rows = vec!["XXXXXXXXXXXXX".to_owned(); 13];
            rows[6] = "XXXXXX@XXXXXX".into();
            enemies.range_presets.push(crate::game::data::corroded_employee_data::CorrodedEmployeeBasicAttackRangePresetDef {
                id: "nothing_there_gear_observation_range".into(),
                range: crate::game::battle::tile_range::TileRangePattern {
                    include_anchor_tile: true, rows,
                },
            });
            observer.basic_attack_range_preset =
                Some("nothing_there_gear_observation_range".into());
            observer.basic_attack.range_role =
                crate::game::data::equipment_data::WeaponRangeRole::Ranged;
            observer.basic_attack.targeting_profile =
                crate::game::data::equipment_data::TargetingProfile::LowDefenseFirst;
        }
        let mut reserve = observer.clone();
        reserve.id = "nothing_there_reserve_enemy".into();
        reserve.uuid = Uuid::from_u128(0xf123);
        reserve.max_health = 800;
        reserve.attack = 0;
        enemies.profiles.push(observer);
        enemies.profiles.push(reserve);
        // The typed database resolves presets on read. Restore authored form
        // before publication so a preset and its expansion are not both supplied.
        for enemy in &mut enemies.profiles {
            if enemy.basic_attack_range_preset.is_some() {
                enemy.basic_attack.defense_tile_range = None;
            }
        }
        fixture.write(path, ron::ser::to_string(&enemies).unwrap());
        let path = "official/data/pve/encounters.ron";
        let mut encounters: PveEncounterDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
        let later = encounters
            .encounters
            .iter_mut()
            .find(|e| e.id == "normal_breach_response")
            .unwrap();
        assert_eq!(later.waves.len(), 1);
        later.waves[0].source = crate::game::data::pve_data::PveWaveSource::Manual(vec![
            crate::game::data::pve_data::PveWaveEnemyData::CorrodedEmployee {
                profile_id: "nothing_there_gear_observation_enemy".into(),
                tier: crate::game::enums::Tier::I,
                count: 1,
            },
            crate::game::data::pve_data::PveWaveEnemyData::CorrodedEmployee {
                profile_id: "nothing_there_reserve_enemy".into(),
                tier: crate::game::enums::Tier::I,
                count: 1,
            },
        ]);
        fixture.write(path, ron::ser::to_string(&encounters).unwrap());
        fixture.write("official/data/skills/nothing-there.skill.yaml", {
            let mut d = canonical::resolved("fragment");
            d["skills"][0]["admission"]["capacity"] = json!(5);
            canonical::overlay(&mut d)["duration_ms"] = json!(HELLO_DURATION_MS);
            d.to_string()
        });
        fixture.write(
            "official/data/skills/nothing-there-equipment.skill.yaml",
            canonical::original("equipment"),
        );
        let files = vec![
            "data/skills/nothing-there.skill.yaml".into(),
            "data/skills/nothing-there-equipment.skill.yaml".into(),
        ];
        let rejected = fixture.root().join("unresolved");
        let error = package_official_with_skill_files(
            &fixture.root().join("official"),
            &rejected,
            &files,
            &Default::default(),
        )
        .unwrap_err();
        assert!(format!("{error:?}").contains("Unresolved"));
        assert!(!rejected.exists());
        fixture.write(
            "official/data/skills/nothing-there-equipment.skill.yaml",
            canonical::resolved("equipment").to_string(),
        );
        let receipt = package_official_with_skill_files(
            &fixture.root().join("official"),
            &fixture.root().join("published"),
            &files,
            &Default::default(),
        )
        .unwrap();
        let loaded = content_pack::load(
            fixture.root(),
            &ContentSelection {
                packs: vec!["published".into()],
                winners: vec![],
            },
            &Default::default(),
        )
        .unwrap();
        assert_eq!(loaded.identity(), &receipt.content_identity);
        kept = Some(fixture);
        loaded
    });
    (kept.unwrap(), data)
}
fn earn(
    core: &mut GameCore,
    player: Uuid,
    slot: &str,
    stage: RevivalStage,
    equipment: &str,
) -> Uuid {
    let id = node(core, slot);
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: id })
        .unwrap();
    core.execute(
        player,
        PlayerBehavior::HatchEncounter {
            node_id: id,
            reward_source_id: "normal_boss_nothing_there".into(),
            stage,
            payment: vec![MaterialPayment {
                material_id: "revival_catalyst".into(),
                carried: 1,
                warehouse: 0,
            }],
        },
    )
    .unwrap();
    let choices = &core.revival_snapshot().unwrap().hatched_nodes[&id].equipment_choices;
    assert_eq!(choices.len(), 3);
    confirm(core, player);
    win(core, player);
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    let reward = core.state.active_node_content.as_ref().unwrap().as_reward().unwrap().rewards.iter()
        .find(|r| r.effects.iter().any(|e| matches!(e, RewardEffect::GrantEquipment { equipment_id } if equipment_id == equipment))).unwrap().uuid;
    core.execute(player, PlayerBehavior::SelectReward { reward_id: reward })
        .unwrap();
    core.execute(player, PlayerBehavior::ClaimReward).unwrap();
    let before = core.get_inventory_snapshot_json().unwrap();
    assert!(core.execute(player, PlayerBehavior::ClaimReward).is_err());
    assert_eq!(core.get_inventory_snapshot_json().unwrap(), before);
    core.execute(player, PlayerBehavior::ExitReward).unwrap();
    core.state
        .inventory
        .equipments
        .iter()
        .find(|e| e.meta.id == equipment)
        .unwrap()
        .instance_uuid
}

#[test]
fn canonical_overlay_world_reward_weapon_armor_and_manual_finish() {
    world(true);
}
#[test]
fn canonical_overlay_world_reward_accessory_revival_and_retreat() {
    world(false);
}
fn world(armor: bool) {
    let (fixture, data) = data(armor);
    let mut core = GameCore::new(data, 12577);
    let player = Uuid::from_u128(0xc0de1250000000000000000000001001);
    start(&mut core, player);
    enter(&mut core, player, "1");
    win(&mut core, player);
    let fragment = core
        .state
        .skill_fragments
        .copies()
        .find(|c| c.definition_id.as_str() == canonical::FRAGMENT)
        .unwrap()
        .copy_uuid;
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    buy_catalysts(&mut core, player);
    let weapon = earn(
        &mut core,
        player,
        "6",
        RevivalStage::Stage2,
        "placeholder_nothing_there_weapon",
    );
    let item = earn(
        &mut core,
        player,
        "7",
        RevivalStage::Stage3,
        if armor {
            "placeholder_nothing_there_armor"
        } else {
            "placeholder_nothing_there_accessory"
        },
    );
    assert_eq!(
        core.state
            .inventory
            .equipment_materials
            .amount("part_nothing_there"),
        6
    );
    let employee = core
        .state
        .roster
        .iter()
        .find(|e| e.employee_definition_id == "scout_runner")
        .unwrap()
        .uuid;
    let other = core
        .state
        .roster
        .iter()
        .find(|e| e.employee_definition_id == "breach_guard")
        .unwrap()
        .uuid;
    if let Some(previous) = core
        .state
        .roster
        .get(&employee)
        .unwrap()
        .skill_fragments
        .active_copy_uuid()
    {
        core.execute(
            player,
            PlayerBehavior::UnequipSkillFragment {
                employee_uuid: employee,
                copy_uuid: previous,
            },
        )
        .unwrap();
    }
    for id in [weapon, item] {
        core.execute(
            player,
            PlayerBehavior::EquipItem {
                item_uuid: id,
                target_unit: employee,
            },
        )
        .unwrap();
    }
    core.execute(
        player,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: employee,
            copy_uuid: fragment,
        },
    )
    .unwrap();
    let checkpoint = core.export_active_run_checkpoint().unwrap().unwrap();
    let inventory = core.get_inventory_snapshot_json().unwrap();
    let mut fresh = core_with_profile_for_continue(&core, 12599);
    fresh
        .continue_from_active_run_checkpoint(
            player,
            &checkpoint.core_payload_json,
            checkpoint.loads_used,
            checkpoint.run_peak_certified_power,
        )
        .unwrap();
    assert_eq!(fresh.get_inventory_snapshot_json().unwrap(), inventory);
    core = fresh;
    let entry_resources = core.run_resources().unwrap().clone();
    enter(&mut core, player, "8");
    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .battle
        .enable_authoring_observation();
    deploy(&mut core, player);
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: other,
        },
    )
    .unwrap();
    let (mut started, mut finished, mut overlay_seen, mut shield, mut revived) =
        (None, None, None, None, None);
    let mut heals = std::collections::BTreeMap::new();
    let mut hits = std::collections::BTreeMap::new();
    let mut finish_fact = None;
    let mut returned_to_base = false;
    let mut actor = None;
    for _ in 0..160 {
        let active = core
            .state
            .active_battle
            .as_ref()
            .expect("observation must precede battle result");
        let now = active.execution.last_event_time_ms();
        let unit = active
            .battle
            .units
            .values()
            .find(|u| u.source_owned_uuid == employee && u.is_active())
            .unwrap_or_else(|| {
                panic!(
                    "actor lost at {now}; start={started:?} finish={finished:?} revive={revived:?}; heal={:?}; observed={:?}", active.battle.runtime_fact_summary().latest_heal, active.battle.authoring_observation().unwrap().unwrap().latest_nonzero_sourced_hp
                )
            });
        let id = unit.instance_id;
        if let Some(old) = actor {
            assert_eq!(old, id, "survival preserves the runtime instance");
        }
        actor = Some(id);
        assert_eq!(unit.fragment_source.as_ref().unwrap().copy_uuid, fragment);
        let effects = active.battle.sustained_effect_snapshot().unwrap();
        if let Some(effect) = effects
            .active
            .iter()
            .find(|e| e.caster == id && e.overlay.is_some())
        {
            assert_eq!(
                effect.goodbye_skill.as_ref().unwrap().as_str(),
                canonical::FINISH
            );
            assert_eq!(
                effect
                    .overlay
                    .as_ref()
                    .unwrap()
                    .line
                    .as_ref()
                    .unwrap()
                    .power,
                40
            );
            overlay_seen = Some(serde_json::to_value(effect).unwrap());
            assert_eq!(
                unit.resonance_current, 0,
                "Hello blocks charging while it owns the use"
            );
        } else if finished.is_some() {
            returned_to_base = true;
        }
        let facts = active.battle.runtime_fact_summary();
        if let Some(h) = &facts.latest_damage {
            if h.source.actor == Some(id) && h.actual_hp_loss() > 0 {
                hits.insert(h.ordinal, serde_json::to_value(h).unwrap());
            }
        }
        let observation = active.battle.authoring_observation().unwrap().unwrap();
        // A simulation advance may finish with an unrelated zero HP update.
        // Use the retained nonzero authored application, as the public Trial
        // consumer does, while checking the real amount and acquired copy.
        if let Some(h) = &observation.latest_nonzero_sourced_hp {
            if h.target_unit == id
                && h.change.hp_delta() > 0
                && h.equipment
                    .as_ref()
                    .is_some_and(|s| s.persistent_copy_uuid == Some(weapon))
            {
                let authored = h.authored.as_ref().unwrap();
                assert_eq!(
                    authored.source_file,
                    "published/data/skills/nothing-there-equipment.skill.yaml"
                );
                assert!(authored.field_path.ends_with("program.basic_lifesteal"));
                assert!(h.change.hp_delta() <= 17);
                heals.insert(h.ordinal, serde_json::to_value(h).unwrap());
            }
        }
        if let Some(w) = &observation.latest_adaptive_shield {
            assert!(armor);
            assert_eq!(w.equipment.persistent_copy_uuid, Some(item));
            assert_eq!(
                w.source_file,
                "published/data/skills/nothing-there-equipment.skill.yaml"
            );
            assert_eq!(w.capacity, w.max_hp / 5);
            assert_eq!(w.expires_at_ms - w.time_ms, 5000);
            assert_eq!(w.ready_at_ms - w.time_ms, 15000);
            shield = Some(serde_json::to_value(w).unwrap());
        }
        if !armor {
            let count = core
                .battle_typed_resources()
                .unwrap()
                .entries()
                .find(|(k, _)| {
                    k.resource.as_str() == "revival_total"
                        && k.owner == crate::game::resource::ResourceOwner::Employee(employee)
                })
                .map_or(0, |(_, v)| v.balance);
            if count > 0 {
                assert_eq!(count, 1);
                revived.get_or_insert(now);
            }
        }
        let cast = started.is_none() && unit.resonance_current == 5;
        // First observe effective weapon healing during Hello. At 5250 ms
        // the initial line hits while HP is full; the armor then absorbs early
        // hostile hits. Finishing at a fixed 6250 ms cannot witness lifesteal.
        let stop =
            started.is_some_and(|t| now >= t + 2000) && !heals.is_empty() && finished.is_none();
        if cast || stop {
            let before_heal = facts.heal_mutations;
            let before_cost = facts.cost_mutations;
            execute_current_battle_mutation(
                &mut core,
                player,
                PlayerBehavior::ActivateSkill {
                    employee_uuid: employee,
                    skill_id: canonical::SKILL.into(),
                    target: None,
                },
            )
            .unwrap();
            if cast {
                started = Some(now + 1);
            } else {
                finished = Some(now);
                let active = core.state.active_battle.as_ref().unwrap();
                let facts = active.battle.runtime_fact_summary();
                assert_eq!(
                    facts.heal_mutations, before_heal,
                    "Goodbye never applies weapon lifesteal"
                );
                assert_eq!(
                    facts.cost_mutations, before_cost,
                    "reinput has no extra resonance cost"
                );
                let hit = facts
                    .latest_damage
                    .as_ref()
                    .expect("finish damages an enemy");
                assert_eq!(
                    hit.source.cast.as_ref().unwrap().skill_id.as_str(),
                    canonical::FINISH
                );
                assert_eq!(hit.source.fragment.as_ref().unwrap().copy_uuid, fragment);
                assert_eq!(
                    hit.source.reinput().unwrap().remaining_ms,
                    started.unwrap() + HELLO_DURATION_MS - now
                );
                assert!(hit.actual_hp_loss() > 0);
                finish_fact = Some(serde_json::to_value(hit).unwrap());
                let source = core
                    .game_data
                    .source_metadata()
                    .unwrap()
                    .source_for(&crate::game::data::content_pack::DefinitionKey {
                        kind: crate::game::data::content_pack::DefinitionKind::Skill,
                        id: canonical::FINISH.into(),
                    })
                    .unwrap();
                assert_eq!(
                    source.source_file,
                    "published/data/skills/nothing-there.skill.yaml"
                );
            }
        }
        if returned_to_base
            && !heals.is_empty()
            && if armor {
                shield.is_some()
            } else {
                revived.is_some()
            }
        {
            break;
        }
        core.advance_active_battle_by(250).unwrap();
    }
    eprintln!("overlay world armor={armor} started={started:?} finished={finished:?} overlay={overlay_seen:?} shield={shield:?} revived={revived:?} heals={heals:?} hits={hits:?} finish={finish_fact:?}");
    assert!(
        started.is_some()
            && finished.is_some()
            && overlay_seen.is_some()
            && finish_fact.is_some()
            && returned_to_base
    );
    assert!(!heals.is_empty());
    assert!(
        hits.values().any(|h| h["time_ms"]
            .as_u64()
            .is_some_and(|t| t >= started.unwrap() && t < finished.unwrap())
            && h["source"]["cast"].is_null()),
        "direct basic damage continues during Hello"
    );
    if armor {
        assert!(shield.is_some());
    } else {
        assert!(revived.is_some());
    }
    let result = json!({"armor":armor,"employee":employee,"actor":actor,"fragment":fragment,"weapon":weapon,"item":item,"started":started,"finished":finished,"overlay":overlay_seen,"shield":shield,"revived":revived,"heals":heals,"hits":hits,"finish":finish_fact});
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::RetreatBattle).unwrap();
    assert_eq!(core.run_resources().unwrap(), &entry_resources);
    assert!(core.state.roster.get(&employee).unwrap().is_alive());
    assert!(core
        .state
        .inventory
        .equipments
        .iter()
        .any(|e| e.instance_uuid == item && e.equipped_to == Some(employee)));
    if let Some(export) = std::env::var_os("OVERLAY_WORLD_EXPORT_DIR") {
        let root = std::path::PathBuf::from(export).join(if armor { "armor" } else { "accessory" });
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("witness.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
        fn copy(from: &std::path::Path, to: &std::path::Path) {
            std::fs::create_dir_all(to).unwrap();
            for e in std::fs::read_dir(from).unwrap() {
                let e = e.unwrap();
                if e.file_type().unwrap().is_dir() {
                    copy(&e.path(), &to.join(e.file_name()));
                } else {
                    std::fs::copy(e.path(), to.join(e.file_name())).unwrap();
                }
            }
        }
        copy(fixture.root(), &root.join("content"));
    }
}
