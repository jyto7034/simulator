use super::*;
use crate::authoring::composition::canonical_presence_tests as canonical;
use crate::game::data::content_pack::{
    self, package_official_with_skill_files, tests::ExternalFixture, ContentSelection,
};
use crate::game::data::{
    equipment_data::EquipmentDatabase, relic_data::RelicDatabase,
    skill_fragment_data::SkillFragmentDatabase,
};
use crate::game::reward::RewardEffect;

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
            encounter_id: Some("suppress_melting_love".into()),
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
        if !armor {
            // This fixture's normal wave stays near its spawn rows. Expose legal
            // ground positions beside it so the real melee Shield can reach it;
            // do not widen the canonical weapon's attack range.
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
        }
        let path = "official/data/abnormalities/base.ron";
        let mut abnormalities: AbnormalityDatabase =
            ron::de::from_str(&fixture.read(path)).unwrap();
        let boss = abnormalities
            .items
            .iter_mut()
            .find(|a| a.id == "d-03-109_melting_love")
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
        // hostile damage and cohort kills; boss and revival waves stay intact.
        let path = "official/data/enemies/corroded_employees.ron";
        let mut enemies: CorrodedEmployeeProfileDatabase =
            ron::de::from_str(&fixture.read(path)).unwrap();
        let mut observer = enemies
            .profiles
            .iter()
            .find(|p| p.id == "corroded_rusher")
            .unwrap()
            .clone();
        observer.id = "melting_love_gear_observation_enemy".into();
        observer.uuid = Uuid::from_u128(0xf122);
        observer.max_health = 400;
        observer.defense = 0;
        observer.magic_resist = 0;
        observer.attack = if armor { 5 } else { 1 };
        if armor {
            // These stationary reinforcement positions cannot reach the rear
            // deployment row with the original melee preset. An explicit ranged
            // fixture attacks the armored employee through ordinary targeting.
            let mut rows = vec!["XXXXXXXXXXXXX".to_owned(); 13];
            rows[6] = "XXXXXX@XXXXXX".into();
            enemies.range_presets.push(crate::game::data::corroded_employee_data::CorrodedEmployeeBasicAttackRangePresetDef {
                id: "melting_love_gear_observation_range".into(),
                range: crate::game::battle::tile_range::TileRangePattern {
                    include_anchor_tile: true, rows,
                },
            });
            observer.basic_attack_range_preset = Some("melting_love_gear_observation_range".into());
            observer.basic_attack.range_role =
                crate::game::data::equipment_data::WeaponRangeRole::Ranged;
            observer.basic_attack.targeting_profile =
                crate::game::data::equipment_data::TargetingProfile::LowDefenseFirst;
        }
        let mut reserve = observer.clone();
        reserve.id = "melting_love_reserve_enemy".into();
        reserve.uuid = Uuid::from_u128(0xf123);
        reserve.max_health = 2000;
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
                profile_id: "melting_love_gear_observation_enemy".into(),
                tier: crate::game::enums::Tier::I,
                count: 2,
            },
            crate::game::data::pve_data::PveWaveEnemyData::CorrodedEmployee {
                profile_id: "melting_love_reserve_enemy".into(),
                tier: crate::game::enums::Tier::I,
                count: 1,
            },
        ]);
        fixture.write(path, ron::ser::to_string(&encounters).unwrap());
        fixture.write("official/data/skills/melting-love.skill.yaml", {
            let mut d = canonical::resolved("fragment");
            d["skills"][0]["admission"]["capacity"] = json!(5);
            d.to_string()
        });
        fixture.write(
            "official/data/skills/melting-love-equipment.skill.yaml",
            canonical::original("equipment"),
        );
        let files = vec![
            "data/skills/melting-love.skill.yaml".into(),
            "data/skills/melting-love-equipment.skill.yaml".into(),
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
            "official/data/skills/melting-love-equipment.skill.yaml",
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
            reward_source_id: "normal_boss_melting_love".into(),
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
fn canonical_presence_world_shield_intrinsic_burst_and_captured_kill_reward() {
    world(false);
}
#[test]
fn canonical_presence_world_armor_live_dot_recovery_and_captured_kill_reward() {
    world(true);
}
fn world(armor: bool) {
    let (fixture, data) = data(armor);
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
    let selected = if armor {
        "placeholder_melting_love_armor"
    } else {
        "placeholder_melting_love_weapon"
    };
    let item = earn(&mut core, player, "6", RevivalStage::Stage2, selected);
    let accessory = earn(
        &mut core,
        player,
        "7",
        RevivalStage::Stage3,
        "placeholder_melting_love_accessory",
    );
    assert_eq!(
        core.state
            .inventory
            .equipment_materials
            .amount("part_melting_love"),
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
    for id in [item, accessory] {
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
    let mut fresh = core_with_profile_for_continue(&core, 12299);
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
    let (mut network, mut periodic, mut heal, mut burst, mut reward) =
        (None, None, None, None, None);
    let mut casts = 0;
    let mut removed = false;
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
                // Empty intrinsic network must reject before cost, events or state change.
                let before = core
                    .state
                    .active_battle
                    .as_ref()
                    .unwrap()
                    .battle
                    .transaction_debug_snapshot();
                assert!(execute_current_battle_mutation(
                    core,
                    player,
                    PlayerBehavior::ActivateSkill {
                        employee_uuid: employee,
                        skill_id: canonical::SKILL.into(),
                        target: None
                    }
                )
                .is_err());
                assert_eq!(
                    core.state
                        .active_battle
                        .as_ref()
                        .unwrap()
                        .battle
                        .transaction_debug_snapshot(),
                    before
                );
            }
            let active = core.state.active_battle.as_ref().unwrap();
            let observation = active.battle.authoring_observation().unwrap().unwrap();
            if let Some(w) = &observation.latest_nonzero_sourced_hp {
                if w.equipment
                    .as_ref()
                    .is_some_and(|e| e.persistent_copy_uuid == Some(item))
                    && w.change.hp_delta() > 0
                {
                    assert_eq!(
                        w.authored.as_ref().unwrap().source_file,
                        "published/data/skills/melting-love-equipment.skill.yaml"
                    );
                    heal = Some(serde_json::to_value(w).unwrap());
                }
                if w.fragment.as_ref().is_some_and(|f| f.copy_uuid == fragment)
                    && w.change.hp_delta() < 0
                {
                    if w.source.is_some() {
                        burst = Some(serde_json::to_value(w).unwrap());
                    } else if w.authored.is_some() {
                        periodic = Some(serde_json::to_value(w).unwrap());
                    }
                }
            }
            if let Some(w) = &observation.latest_sourced_resonance {
                if w.equipment
                    .as_ref()
                    .is_some_and(|e| e.persistent_copy_uuid == Some(accessory))
                {
                    let value = serde_json::to_value(w).unwrap();
                    if value["change"]["after"].as_u64().unwrap()
                        > value["change"]["before"].as_u64().unwrap()
                    {
                        assert_eq!(
                            w.authored.as_ref().unwrap().source_file,
                            "published/data/skills/melting-love-equipment.skill.yaml"
                        );
                        reward = Some(value);
                    }
                }
            }
            let unit = active
                .battle
                .units
                .values()
                .find(|u| u.source_owned_uuid == employee && u.is_active())
                .expect("equipped actor survives fixture combat");
            let now = active.execution.last_event_time_ms();
            let snapshot = active.battle.presence_marks_snapshot(now).unwrap();
            let current = snapshot
                .networks
                .iter()
                .find(|n| n.fragment.copy_uuid == fragment && !n.hosts.is_empty());
            let mut should_cast = false;
            if let Some(n) = current {
                assert_eq!(n.fragment.definition_id.as_str(), canonical::FRAGMENT);
                assert_eq!(n.fragment.skill_id.as_str(), canonical::SKILL);
                assert_eq!(n.definition, "melting-love-spores");
                network = Some(serde_json::to_value(n).unwrap());
                let power = n.power_units.parse::<u128>().unwrap() / u128::from(n.power_scale);
                should_cast = casts == 0
                    && unit.resonance_current == 5
                    && power >= 20
                    && n.hosts.iter().any(|h| {
                        active.battle.units.get(&h.unit).is_some_and(|u| {
                            u.stats.current_health > 0
                                && u.stats.current_health as u128 <= power * 3
                        })
                    });
            }
            if should_cast {
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
                casts += 1;
                let a = core.state.active_battle.as_ref().unwrap();
                let s = a
                    .battle
                    .presence_marks_snapshot(a.execution.last_event_time_ms())
                    .unwrap();
                assert!(s
                    .networks
                    .iter()
                    .filter(|n| n.fragment.copy_uuid == fragment)
                    .all(|n| n.hosts.is_empty()));
                assert!(s
                    .pending_bursts
                    .iter()
                    .any(|p| p.fragment_copy_uuid == fragment));
                // Read the instant release before a later periodic heal can
                // replace the bounded latest-HP witness in the next 250ms step.
                core.advance_active_battle_by(1).unwrap();
                if let Some(active) = &core.state.active_battle {
                    let o = active.battle.authoring_observation().unwrap().unwrap();
                    eprintln!("presence release armor={armor} observation={o:?}");
                    if let Some(w) = &o.latest_nonzero_sourced_hp {
                        if w.source.is_some()
                            && w.change.hp_delta() < 0
                            && w.fragment.as_ref().is_some_and(|f| f.copy_uuid == fragment)
                        {
                            burst = Some(serde_json::to_value(w).unwrap());
                        }
                    }
                }
            }
            core.advance_active_battle_by(250).unwrap();
        },
    );
    eprintln!("presence final armor={armor} casts={casts} network={network:?} periodic={periodic:?} burst={burst:?} heal={heal:?} reward={reward:?}");
    assert!(
        network.is_some(),
        "ordinary basic attacks must infect with actual equipped fragment"
    );
    assert!(
        periodic.is_some(),
        "actual authored periodic HP loss with fragment source"
    );
    assert!(burst.is_some(), "actual authored detonation HP loss");
    assert_eq!(casts, 1);
    assert!(
        reward.is_some(),
        "actual accessory resonance after own DOT-qualified kill"
    );
    if armor {
        assert!(
            heal.is_some(),
            "actual armor recovery from living own DOT targets"
        );
    }
    let result = json!({"armor":armor,"fragment":fragment,"item":item,"accessory":accessory,"network":network,"periodic":periodic,"heal":heal,"burst":burst,"reward":reward});
    eprintln!("canonical presence World {result}");
    if let Some(root) = std::env::var_os("PRESENCE_WORLD_EXPORT_DIR") {
        let root = std::path::PathBuf::from(root).join(if armor { "armor" } else { "weapon" });
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("witness.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
        fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
            std::fs::create_dir_all(to).unwrap();
            for entry in std::fs::read_dir(from).unwrap() {
                let e = entry.unwrap();
                if e.file_type().unwrap().is_dir() {
                    copy_tree(&e.path(), &to.join(e.file_name()));
                } else {
                    std::fs::copy(e.path(), to.join(e.file_name())).unwrap();
                }
            }
        }
        copy_tree(fixture.root(), &root.join("content"));
    }
}
