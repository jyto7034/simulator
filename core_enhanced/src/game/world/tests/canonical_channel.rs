use super::*;
use crate::authoring::composition::canonical_channel_tests as canonical;
use crate::game::data::content_pack::{
    self, package_official_with_skill_files, tests::ExternalFixture, ContentSelection,
};
use crate::game::data::{
    equipment_data::EquipmentDatabase, relic_data::RelicDatabase,
    skill_fragment_data::SkillFragmentDatabase,
};
use crate::game::reward::RewardEffect;

fn data() -> (ExternalFixture, ValidatedGameData) {
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
            encounter_id: Some("suppress_burrowing_heaven".into()),
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
        // Explicit observation layout exposes legal Staff positions beside both
        // stationary columns without altering its authored attack pattern.
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
        template["rows"][3] = json!("TT.TT..TT");
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
            .find(|a| a.id == "o-04-72_burrowing_heaven")
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
        observer.id = "burrowing_heaven_gear_observation_enemy".into();
        observer.uuid = Uuid::from_u128(0xf122);
        observer.max_health = 500;
        observer.defense = 0;
        observer.magic_resist = 0;
        observer.attack = 4;
        observer.basic_attack.damage_type = crate::game::battle::damage::DamageType::True;
        observer.basic_attack.interval_ms = 1000;
        observer.basic_attack.windup_ms = 0;
        {
            // These stationary reinforcement positions cannot reach the rear
            // deployment row with the original melee preset. An explicit ranged
            // fixture attacks the armored employee through ordinary targeting.
            let mut rows = vec!["XXXXXXXXXXXXX".to_owned(); 13];
            rows[6] = "XXXXXX@XXXXXX".into();
            enemies.range_presets.push(crate::game::data::corroded_employee_data::CorrodedEmployeeBasicAttackRangePresetDef {
                id: "burrowing_heaven_gear_observation_range".into(),
                range: crate::game::battle::tile_range::TileRangePattern {
                    include_anchor_tile: true, rows,
                },
            });
            observer.basic_attack_range_preset =
                Some("burrowing_heaven_gear_observation_range".into());
            observer.basic_attack.range_role =
                crate::game::data::equipment_data::WeaponRangeRole::Ranged;
            observer.basic_attack.targeting_profile =
                crate::game::data::equipment_data::TargetingProfile::LowDefenseFirst;
        }
        let mut reserve = observer.clone();
        reserve.id = "burrowing_heaven_reserve_enemy".into();
        reserve.uuid = Uuid::from_u128(0xf123);
        reserve.max_health = 400;
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
                profile_id: "burrowing_heaven_gear_observation_enemy".into(),
                tier: crate::game::enums::Tier::I,
                count: 1,
            },
            crate::game::data::pve_data::PveWaveEnemyData::CorrodedEmployee {
                profile_id: "burrowing_heaven_reserve_enemy".into(),
                tier: crate::game::enums::Tier::I,
                count: 1,
            },
        ]);
        fixture.write(path, ron::ser::to_string(&encounters).unwrap());
        fixture.write("official/data/skills/burrowing-heaven.skill.yaml", {
            let mut d = canonical::resolved("fragment");
            d["skills"][0]["admission"]["capacity"] = json!(5);
            let tile = &mut d["values"][0]["value"]["Delivery"]["TileArea"];
            let mut rows = vec!["XXXXXXXXXXXXXXX"; 15];
            rows[7] = "XXXXXXX@XXXXXXX";
            tile["footprint"] = json!({"include_anchor_tile":true,"rows":rows});
            tile["area"]["duration_ms"] = json!(5000);
            tile["area"]["ground_status"]["statuses"][0]["kind"]["RampingSlow"]["duration_ms"] =
                json!(3000);
            tile["area"]["ground_status"]["rules"][2]["trigger"]["Dwell"]["duration_ms"] =
                json!(3000);
            d.to_string()
        });
        fixture.write(
            "official/data/skills/burrowing-heaven-equipment.skill.yaml",
            canonical::original("equipment"),
        );
        let files = vec![
            "data/skills/burrowing-heaven.skill.yaml".into(),
            "data/skills/burrowing-heaven-equipment.skill.yaml".into(),
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
            "official/data/skills/burrowing-heaven-equipment.skill.yaml",
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
            reward_source_id: "normal_boss_burrowing_heaven".into(),
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
fn canonical_channel_world_staff_and_armor_actual_opening_window() {
    world(true);
}
#[test]
fn canonical_channel_world_staff_and_accessory_manual_end_independent_root() {
    world(false);
}

fn world(armor: bool) {
    let (fixture, data) = data();
    let mut core = GameCore::new(data, 12077);
    let player = Uuid::from_u128(0xc0de1200000000000000000000001001);
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
        "placeholder_burrowing_heaven_weapon",
    );
    let item = earn(
        &mut core,
        player,
        "7",
        RevivalStage::Stage3,
        if armor {
            "placeholder_burrowing_heaven_armor"
        } else {
            "placeholder_burrowing_heaven_accessory"
        },
    );
    assert_eq!(
        core.state
            .inventory
            .equipment_materials
            .amount("part_burrowing_heaven"),
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
    let mut fresh = core_with_profile_for_continue(&core, 12399);
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
    enter(&mut core, player, "8");
    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .battle
        .enable_authoring_observation();
    let (mut start_time, mut end_time, mut area, mut opening, mut root, mut ended) =
        (None, None, None, None, None, None);
    let (mut removed, mut stunned, mut root_expired) = (false, false, false);
    let mut releases = std::collections::BTreeMap::new();
    let mut damage = std::collections::BTreeMap::new();
    let mut actor = None;
    win_with_advance_consumer(
        &mut core,
        player,
        |_| {},
        |core| {
            if !removed {
                execute_current_battle_mutation(
                    core,
                    player,
                    PlayerBehavior::WithdrawUnit {
                        employee_uuid: other,
                    },
                )
                .unwrap();
                removed = true;
            }
            let active = core.state.active_battle.as_ref().unwrap();
            let now = active.execution.last_event_time_ms();
            let unit = active
                .battle
                .units
                .values()
                .find(|u| u.source_owned_uuid == employee && u.is_active())
                .unwrap_or_else(|| panic!("actor survives observation now={now} start={start_time:?} end={end_time:?} damage={damage:?} releases={releases:?}; units={:?}", active.battle.units.values().map(|u|(u.source_owned_uuid,u.stats.current_health,u.lifecycle)).collect::<Vec<_>>()));
            let actor_id = unit.instance_id;
            actor = Some(actor_id);
            assert_eq!(unit.fragment_source.as_ref().unwrap().copy_uuid, fragment);
            let observation = active.battle.authoring_observation().unwrap().unwrap();
            if let Some(w) = &observation.latest_area {
                assert_eq!(w.skill_id.as_str(), canonical::SKILL);
                assert_eq!(w.caster, actor_id);
                assert_eq!(w.generation.as_deref(), Some(core.content_identity()));
                use crate::game::data::content_pack::{DefinitionKey, DefinitionKind};
                let origin = core
                    .game_data
                    .source_metadata()
                    .unwrap()
                    .source_for(&DefinitionKey {
                        kind: DefinitionKind::Skill,
                        id: w.skill_id.as_str().into(),
                    })
                    .unwrap();
                assert_eq!(
                    origin.source_file,
                    "published/data/skills/burrowing-heaven.skill.yaml"
                );
                assert_eq!(w.started_at_ms, start_time.unwrap());
                assert_eq!(w.expires_at_ms, w.started_at_ms + 5000);

                if serde_json::to_value(w).unwrap()["change"]["kind"] == "ended" {
                    ended = Some(serde_json::to_value(w).unwrap());
                } else {
                    area = Some(serde_json::to_value(w).unwrap());
                }
            }
            if let Some(w) = &observation.latest_channel_opening {
                assert!(armor);
                assert_eq!(w.equipment.persistent_copy_uuid, Some(item));
                assert_eq!(
                    w.source_file,
                    "published/data/skills/burrowing-heaven-equipment.skill.yaml"
                );
                assert_eq!(w.reduction_percent, 50);
                assert_eq!(w.window_end_ms - w.started_at_ms, 2000);
                opening = Some(serde_json::to_value(w).unwrap());
            }
            for entry in active.battle.event_log.entries_after_seq(0) {
                let e = serde_json::to_value(&entry).unwrap();
                if e["event"]["type"] == "BasicAttackTilesReleased"
                    && e["event"]["attacker_instance_id"] == json!(actor_id)
                {
                    releases.insert(entry.seq, e.clone());
                }
                if e["event"]["type"] == "HpChanged"
                    && e["event"]["target_instance_id"] == json!(actor_id)
                    && e["event"]["delta"].as_i64().is_some_and(|v| v < 0)
                {
                    damage.insert(entry.seq, e);
                }
            }
            if start_time.is_some() && end_time.is_none() {
                stunned |= active.battle.units.values().any(|u| {
                    u.owner == crate::game::enums::Side::Opponent
                        && u.is_active()
                        && active
                            .battle
                            .unit_control_state(u.instance_id, now)
                            .unwrap()
                            .hard_cc_active()
                });
            }
            let effects = active.battle.sustained_effect_snapshot().unwrap();
            let roots: Vec<_> = effects
                .active
                .iter()
                .filter(|e| {
                    e.hit_status.as_ref().is_some_and(|h| {
                        h.kind == crate::game::ability::sustained::HitStatusKind::Root
                    })
                })
                .collect();
            if !roots.is_empty() {
                assert!(!armor);
                assert!(end_time.is_some());
                assert!(roots.iter().all(|r| !active
                    .battle
                    .unit_control_state(r.target, now)
                    .unwrap()
                    .hard_cc_active()));
                root = Some(serde_json::to_value(&roots).unwrap());
                let w = observation.latest_sourced_sustained.as_ref().unwrap();
                assert_eq!(
                    w.equipment.as_ref().unwrap().persistent_copy_uuid,
                    Some(item)
                );
                assert_eq!(
                    w.authored.as_ref().unwrap().source_file,
                    "published/data/skills/burrowing-heaven-equipment.skill.yaml"
                );
            } else if end_time.is_some_and(|t| now >= t + 1000) {
                root_expired = true;
            }
            let cast = start_time.is_none() && unit.resonance_current == 5 && !releases.is_empty();
            let stop = start_time.is_some_and(|t| now >= t + 3500) && end_time.is_none();
            if cast || stop {
                execute_current_battle_mutation(
                    core,
                    player,
                    PlayerBehavior::ActivateSkill {
                        employee_uuid: employee,
                        skill_id: canonical::SKILL.into(),
                        target: None,
                    },
                )
                .unwrap();
                if cast {
                    start_time = Some(now + 1);
                } else {
                    end_time = Some(now);
                }
            }
            core.advance_active_battle_by(250).unwrap();
        },
    );
    eprintln!("channel world armor={armor} start={start_time:?} end={end_time:?} area={area:?} opening={opening:?} root={root:?} damage={damage:?} releases={releases:?}");
    let (start, end) = (start_time.unwrap(), end_time.unwrap());
    assert!(area.is_some() && ended.is_some() && stunned);
    assert!(releases
        .values()
        .any(|e| e["time_ms"].as_u64().unwrap() < start));
    assert!(releases
        .values()
        .any(|e| e["time_ms"].as_u64().unwrap() > end));
    assert!(!releases.values().any(|e| {
        let t = e["time_ms"].as_u64().unwrap();
        t >= start && t < end
    }));
    if armor {
        assert!(opening.is_some());
        let guarded: Vec<_> = damage
            .values()
            .filter(|e| {
                let t = e["time_ms"].as_u64().unwrap();
                t >= start && t < start + 2000
            })
            .collect();
        assert!(!guarded.is_empty());
        assert!(guarded
            .iter()
            .all(|e| e["event"]["raw_damage"] == 5 && e["event"]["delta"] == -2));
        assert!(damage.values().any(|e| e["time_ms"]
            .as_u64()
            .is_some_and(|t| t >= start + 2000 && t < end)
            && e["event"]["delta"] == -5));
    } else {
        assert!(root.is_some() && root_expired);
        assert!(root
            .as_ref()
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["expires_at_ms"] == json!(end + 1000)));
        assert!(
            releases.values().any(|e| e["time_ms"]
                .as_u64()
                .is_some_and(|t| t > end && t < end + 1000)),
            "basic attack resumes while independent Root is still active"
        );
    }
    let result = json!({"armor":armor,"fragment":fragment,"weapon":weapon,"item":item,"actor":actor,"started":start,"ended":end,"area":area,"area_end":ended,"opening":opening,"root":root,"damage":damage,"releases":releases});
    if let Some(export) = std::env::var_os("CHANNEL_WORLD_EXPORT_DIR") {
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
