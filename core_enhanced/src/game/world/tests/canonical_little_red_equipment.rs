use super::*;
use crate::authoring::composition::{
    canonical_input_plan_tests as fragment, canonical_little_red_equipment_tests as gear,
};
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
            encounter_id: Some("suppress_little_red".into()),
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
        let path = "official/data/abnormalities/base.ron";
        let mut abnormalities: AbnormalityDatabase =
            ron::de::from_str(&fixture.read(path)).unwrap();
        let boss = abnormalities
            .items
            .iter_mut()
            .find(|a| a.id == "f-01-57_little_red")
            .unwrap();
        boss.max_health = 70;
        boss.attack = 0;
        boss.defense = 0;
        boss.magic_resist = 0;
        fixture.write(path, ron::ser::to_string(&abnormalities).unwrap());
        let path = "official/data/skill_fragments/base.ron";
        let mut fragments: SkillFragmentDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
        let metadata = fragment::catalog();
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
        for metadata in gear::catalog() {
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
        for relic in fragment::relics().relics {
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
        observer.id = "little_red_gear_observation_enemy".into();
        observer.uuid = Uuid::from_u128(0xf117);
        observer.max_health = 10000;
        observer.attack = 30;
        if armor {
            // These stationary reinforcement positions cannot reach the rear
            // deployment row with the original melee preset. An explicit ranged
            // fixture attacks the armored employee through ordinary targeting.
            let mut rows = vec!["XXXXXXXXXXXXX".to_owned(); 13];
            rows[6] = "XXXXXX@XXXXXX".into();
            enemies.range_presets.push(crate::game::data::corroded_employee_data::CorrodedEmployeeBasicAttackRangePresetDef {
                id: "little_red_gear_observation_range".into(),
                range: crate::game::battle::tile_range::TileRangePattern {
                    include_anchor_tile: true, rows,
                },
            });
            observer.basic_attack_range_preset = Some("little_red_gear_observation_range".into());
            observer.basic_attack.range_role =
                crate::game::data::equipment_data::WeaponRangeRole::Ranged;
            observer.basic_attack.targeting_profile =
                crate::game::data::equipment_data::TargetingProfile::LowDefenseFirst;
        }
        enemies.profiles.push(observer);
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
                profile_id: "little_red_gear_observation_enemy".into(),
                tier: crate::game::enums::Tier::I,
                count: 2,
            },
        ]);
        fixture.write(path, ron::ser::to_string(&encounters).unwrap());
        fixture.write(
            "official/data/skills/little-red.skill.yaml",
            fragment::resolved().to_string(),
        );
        fixture.write(
            "official/data/skills/little-red-equipment.skill.yaml",
            gear::ORIGINAL,
        );
        let files = vec![
            "data/skills/little-red.skill.yaml".into(),
            "data/skills/little-red-equipment.skill.yaml".into(),
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
            "official/data/skills/little-red-equipment.skill.yaml",
            gear::resolved().to_string(),
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
            reward_source_id: "normal_boss_little_red".into(),
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
fn canonical_little_red_world_revived_weapon_accessory_and_shared_fragment() {
    world(false);
}
#[test]
fn canonical_little_red_world_revived_armor_accessory_and_deferred_damage() {
    world(true);
}
fn world(armor: bool) {
    let (fixture, data) = data(armor);
    let mut core = GameCore::new(data, 12077);
    let player = Uuid::from_u128(0xc0de1200000000000000000000001001);
    start(&mut core, player);
    enter(&mut core, player, "1");
    win(&mut core, player);
    assert_eq!(
        core.state
            .skill_fragments
            .count(&"fragment_little_red_trace".into()),
        1
    );
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    buy_catalysts(&mut core, player);
    let selected = if armor {
        "placeholder_little_red_armor"
    } else {
        "placeholder_little_red_weapon"
    };
    let item = earn(&mut core, player, "6", RevivalStage::Stage2, selected);
    let accessory = earn(
        &mut core,
        player,
        "7",
        RevivalStage::Stage3,
        "placeholder_little_red_accessory",
    );
    assert_eq!(
        core.state
            .inventory
            .equipment_materials
            .amount("part_little_red"),
        6
    );
    let employee = core
        .state
        .roster
        .iter()
        .find(|e| {
            e.employee_definition_id
                == if armor {
                    "breach_guard"
                } else {
                    "scout_runner"
                }
        })
        .unwrap()
        .uuid;
    for copy in [item, accessory] {
        core.execute(
            player,
            PlayerBehavior::EquipItem {
                item_uuid: copy,
                target_unit: employee,
            },
        )
        .unwrap();
    }
    let fragment = core
        .state
        .skill_fragments
        .copies()
        .find(|c| c.definition_id.as_str() == "fragment_little_red_trace")
        .unwrap()
        .copy_uuid;
    if let Some(previous) = core
        .state
        .roster
        .get(&employee)
        .unwrap()
        .skill_fragments
        .active_copy_uuid()
    {
        assert_ne!(previous, fragment);
        core.execute(
            player,
            PlayerBehavior::UnequipSkillFragment {
                employee_uuid: employee,
                copy_uuid: previous,
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
    enter(&mut core, player, "8");
    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .battle
        .enable_authoring_observation();
    let shared = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .shared_cohorts_snapshot()
        .unwrap();
    assert_eq!(shared.owners.len(), 1);
    assert_eq!(shared.owners[0].providers.len(), 2);
    deploy(&mut core, player);
    if armor {
        let other = core
            .state
            .roster
            .iter()
            .find(|e| e.employee_definition_id == "scout_runner")
            .unwrap()
            .uuid;
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::WithdrawUnit {
                employee_uuid: other,
            },
        )
        .unwrap();
    }
    let mut witness = None;
    for _ in 0..300 {
        assert!(
            core.state.active_battle.is_some(),
            "required gear effect must precede terminal"
        );
        core.advance_active_battle_by(100).unwrap();
        let Some(active) = &core.state.active_battle else {
            break;
        };
        let observation = active.battle.authoring_observation().unwrap().unwrap();
        let source = if armor {
            observation
                .latest_deferred_damage
                .as_ref()
                .filter(|d| {
                    d.actual_hp_loss > 0
                        && matches!(
                            d.operation,
                            crate::game::battle::core::equipment_defense::DeferredOperation::Repay
                        )
                })
                .map(|d| serde_json::to_value(d).unwrap())
        } else {
            observation
                .latest_magazine
                .as_ref()
                .filter(|m| {
                    m.enhanced
                        && matches!(
                            m.operation,
                            crate::game::battle::core::equipment_magazine::MagazineOperation::Fired
                        )
                })
                .map(|m| serde_json::to_value(m).unwrap())
        };
        if let Some(source) = source {
            witness = Some(source);
            break;
        }
    }
    if witness.is_none() {
        let active = core.state.active_battle.as_ref().unwrap();
        eprintln!(
            "missing gear witness armor={armor}; observation={:?}; units={:?}",
            active.battle.authoring_observation().unwrap(),
            active
                .battle
                .units
                .values()
                .map(|u| (
                    u.instance_id,
                    u.source_owned_uuid,
                    u.owner,
                    u.body.projected_tile(),
                    u.stats.current_health,
                    u.stats.attack,
                    u.stats.defense
                ))
                .collect::<Vec<_>>()
        );
    }
    let witness =
        witness.expect("real received damage or cohort kill must activate selected equipment");
    assert_eq!(witness["equipment"]["persistent_copy_uuid"], json!(item));
    assert_eq!(
        witness["source_file"],
        "published/data/skills/little-red-equipment.skill.yaml"
    );
    eprintln!(
        "canonical little-red gear World {}",
        json!({"armor":armor,"equipment":item,"accessory":accessory,"fragment":fragment,"shared":shared,"witness":witness})
    );
    if let Some(root) = std::env::var_os("LITTLE_RED_GEAR_WORLD_EXPORT_DIR") {
        let root = std::path::PathBuf::from(root).join(if armor { "armor" } else { "weapon" });
        assert!(!root.exists());
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
        copy_tree(fixture.root(), &root.join("content"));
        std::fs::write(
            root.join("witness.json"),
            serde_json::to_vec_pretty(&witness).unwrap(),
        )
        .unwrap();
    }
}
