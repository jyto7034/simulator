use super::*;
use crate::authoring::composition::canonical_greed_tests as canonical;
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
            encounter_id: Some("suppress_king_of_greed".into()),
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
        // The original zigzag room blocked every prepared projectile. Keep
        // obstacle_stop enabled and publish an explicit clear observation room;
        // the shared native tests separately exercise obstruction and misses.
        let path = "official/data/map/battlefield_templates.ron";
        let templates: Vec<crate::game::combat_preview::BattlefieldTemplateDefinition> =
            ron::de::from_str(&fixture.read(path)).unwrap();
        let mut rows = serde_json::to_value(&templates).unwrap();
        let room = rows
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|t| t["id"] == "obstacle_room_zigzag_01")
            .unwrap();
        assert!(room["rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r.as_str().unwrap().contains('#')));
        for row in room["rows"].as_array_mut().unwrap() {
            *row = json!(row.as_str().unwrap().replace('#', "."));
        }
        let templates: Vec<crate::game::combat_preview::BattlefieldTemplateDefinition> =
            serde_json::from_value(rows).unwrap();
        fixture.write(path, ron::ser::to_string(&templates).unwrap());
        let path = "official/data/abnormalities/base.ron";
        let mut abnormalities: AbnormalityDatabase =
            ron::de::from_str(&fixture.read(path)).unwrap();
        let boss = abnormalities
            .items
            .iter_mut()
            .find(|a| a.id == "o-01-64_king_of_greed")
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
        observer.id = "king_of_greed_gear_observation_enemy".into();
        observer.uuid = Uuid::from_u128(0xf122);
        observer.max_health = 300;
        observer.defense = 0;
        observer.magic_resist = 0;
        observer.attack = 0;
        let mut reserve = observer.clone();
        reserve.id = "king_of_greed_reserve_enemy".into();
        reserve.uuid = Uuid::from_u128(0xf123);
        reserve.max_health = 900;
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
                profile_id: "king_of_greed_gear_observation_enemy".into(),
                tier: crate::game::enums::Tier::I,
                count: 2,
            },
            crate::game::data::pve_data::PveWaveEnemyData::CorrodedEmployee {
                profile_id: "king_of_greed_reserve_enemy".into(),
                tier: crate::game::enums::Tier::I,
                count: 1,
            },
        ]);
        fixture.write(path, ron::ser::to_string(&encounters).unwrap());
        fixture.write("official/data/skills/king-of-greed.skill.yaml", {
            let mut d = canonical::resolved("fragment");
            d["skills"][0]["admission"]["capacity"] = json!(5);
            let p = canonical::program(&mut d);
            p["admission"]["capacity"] = json!(5);
            p["policy"]["maximum"] = json!(20);
            p["policy"]["first_ms"] = json!(100);
            p["policy"]["last_ms"] = json!(50);
            p["projectile"]["area"] =
                json!({"include_anchor_tile":true,"rows":["XXX","X@X","XXX"]});

            d.to_string()
        });
        fixture.write(
            "official/data/skills/king-of-greed-equipment.skill.yaml",
            canonical::original("equipment"),
        );
        let files = vec![
            "data/skills/king-of-greed.skill.yaml".into(),
            "data/skills/king-of-greed-equipment.skill.yaml".into(),
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
            "official/data/skills/king-of-greed-equipment.skill.yaml",
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
            reward_source_id: "normal_boss_king_of_greed".into(),
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
        "placeholder_king_of_greed_weapon",
    );
    let item = earn(
        &mut core,
        player,
        "7",
        RevivalStage::Stage3,
        if armor {
            "placeholder_king_of_greed_armor"
        } else {
            "placeholder_king_of_greed_accessory"
        },
    );
    assert_eq!(
        core.state
            .inventory
            .equipment_materials
            .amount("part_king_of_greed"),
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
    let mut fresh = core_with_profile_for_continue(&core, 12899);
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
    let mut ready_at = None;
    let mut cast_at = None;
    let mut released = false;
    let mut removed = false;
    let (mut basic_hp, mut skill_hp, mut armor_bonus, mut area_bonus) =
        (false, false, false, false);
    let mut debt_seen = false;
    let mut fact_skill_hp = false;
    let mut flight_results = std::collections::BTreeMap::new();
    let mut modifier_fact = None;
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
                .unwrap();
            let actor = unit.instance_id;
            let observation = active.battle.authoring_observation().unwrap().unwrap();
            let facts = active.battle.runtime_fact_view();
            for (_, target) in facts.actors() {
                if let Some(hit) = &target.received.latest_nonzero_damage {
                    if hit.source.actor == Some(actor)
                        && hit
                            .source
                            .cast
                            .as_ref()
                            .is_some_and(|s| s.skill_id.as_str() == canonical::SKILL)
                    {
                        assert_eq!(hit.source.fragment.as_ref().unwrap().copy_uuid, fragment);
                        assert_eq!(
                            hit.source.authored.as_ref().unwrap().source_file,
                            "published/data/skills/king-of-greed.skill.yaml"
                        );
                        assert!(hit.actual_hp_loss() > 0);
                        fact_skill_hp = true;
                    }
                    if hit.source.actor == Some(actor) && hit.source.cast.is_none() {
                        assert_eq!(
                            hit.source.equipment.as_ref().unwrap().persistent_copy_uuid,
                            Some(weapon)
                        );
                        assert_eq!(
                            hit.source.equipment.as_ref().unwrap().definition_uuid,
                            Uuid::parse_str("e3000000-0000-0000-0000-000000000002").unwrap()
                        );
                        assert!(hit.source.authored.is_none());
                        assert!(hit.actual_hp_loss() > 0);
                        basic_hp = true;
                    }
                }
            }
            if let Some(hp) = &observation.latest_nonzero_sourced_hp {
                if hp
                    .source
                    .as_ref()
                    .is_some_and(|s| s.cast.skill_id.as_str() == canonical::SKILL)
                {
                    assert_eq!(hp.fragment.as_ref().unwrap().copy_uuid, fragment);
                    let source = hp.source.as_ref().unwrap();
                    assert_eq!(source.cast.caster_instance_id, actor);
                    assert_eq!(
                        source.authored.source_file,
                        "published/data/skills/king-of-greed.skill.yaml"
                    );
                    assert_eq!(
                        source.authored.site.action.definition_id.as_str(),
                        canonical::SKILL
                    );
                    assert_eq!(
                        source.authored.origin,
                        crate::game::ability::authoring::EffectOrigin::Original
                    );
                    skill_hp = true;
                }
            }
            if armor {
                if let Some(w) = &observation.latest_ready_wait {
                    if w.actor == actor && w.percent > 0 {
                        assert_eq!(w.contributors.len(), 1);
                        let c = &w.contributors[0];
                        assert_eq!(c.equipment.persistent_copy_uuid, Some(item));
                        assert_eq!(
                            c.source_file,
                            "published/data/skills/king-of-greed-equipment.skill.yaml"
                        );
                        assert_eq!(c.until_ms, w.time_ms + 15000);
                        assert_eq!(c.percent, ((c.waited_ms * 10 / 1000).min(50)) as u32);
                        assert_eq!(w.percent, c.percent as i32);
                        if modifier_fact.is_none() {
                            modifier_fact = facts
                                .actors()
                                .filter_map(|(_, f)| f.received.latest_nonzero_damage.as_ref())
                                .find(|h| {
                                    h.source.actor == Some(actor)
                                        && h.time_ms >= w.time_ms
                                        && h.time_ms < c.until_ms
                                })
                                .map(|h| serde_json::to_value(h).unwrap());
                        }
                        armor_bonus = true;
                    }
                }
            } else if let Some(w) = &observation.latest_area_damage {
                if w.actor == actor && w.target_count >= 2 {
                    assert_eq!(w.contributors.len(), 1);
                    assert_eq!(w.contributors[0].equipment.persistent_copy_uuid, Some(item));
                    assert_eq!(
                        w.contributors[0].source_file,
                        "published/data/skills/king-of-greed-equipment.skill.yaml"
                    );
                    assert_eq!(w.percent, ((w.target_count - 1) * 10).min(50));
                    if modifier_fact.is_none() {
                        modifier_fact = facts
                            .actors()
                            .filter_map(|(_, f)| f.received.latest_nonzero_damage.as_ref())
                            .find(|h| {
                                h.source.actor == Some(actor)
                                    && h.time_ms == w.time_ms
                                    && h.source.action_ordinal()
                                        == w.action
                                            .map(crate::game::battle::runtime_facts::ActionOrdinal)
                            })
                            .map(|h| serde_json::to_value(h).unwrap());
                    }
                    area_bonus = true;
                }
            }
            let volley = active.battle.prepared_volleys_snapshot().unwrap();
            for flight in &volley.settled {
                flight_results.insert(flight.id, (flight.end, flight.hits.len()));
            }
            let owner = volley.owners.iter().find(|v| v.copy == fragment).unwrap();
            assert_eq!(owner.employee, employee);
            assert_eq!(owner.caster, Some(actor));
            assert!(volley.flights.len() <= 1);
            if owner.balance < 0 {
                assert!(owner.borrowed);
                debt_seen = true;
            }
            if cast_at.is_none() && unit.resonance_current == 5 && ready_at.is_none() {
                ready_at = Some(now);
            }
            let cast_now = cast_at.is_none()
                && ready_at.is_some_and(|t| now >= t + 2000)
                && unit.resonance_current == 5;
            let fire = if !released && owner.prepared == 20 {
                owner.activation
            } else {
                None
            };
            if cast_now {
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
                cast_at = Some(now);
            } else if let Some(activation) = fire {
                execute_current_battle_mutation(
                    core,
                    player,
                    PlayerBehavior::ReinputSustainedSkill {
                        employee_uuid: employee,
                        skill_id: canonical::SKILL.into(),
                        activation,
                        target: None,
                    },
                )
                .unwrap();
                released = true;
            }
            core.advance_active_battle_by(250).unwrap();
        },
    );
    eprintln!("N11 armor={armor} cast={cast_at:?} released={released} basic={basic_hp} skill={skill_hp} armor_bonus={armor_bonus} area_bonus={area_bonus} debt={debt_seen}");
    eprintln!("N11 mandatory skill HP={fact_skill_hp}; flight ends={flight_results:?}");
    eprintln!("N11 armor={armor} applied modifier fact={modifier_fact:?}");
    assert!(cast_at.is_some() && released && basic_hp && skill_hp && fact_skill_hp && debt_seen);
    assert!(if armor { armor_bonus } else { area_bonus });
    let entry = core.state.run.as_ref().unwrap().credit_ledgers.entries[&fragment].clone();
    assert!(entry.borrowed && entry.unpaid > 0 && entry.unpaid <= 10);
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    let ledger = core.state.run.as_ref().unwrap().credit_ledgers.clone();
    assert!(core
        .execute(player, PlayerBehavior::CompleteCombatResult)
        .is_err());
    assert_eq!(core.state.run.as_ref().unwrap().credit_ledgers, ledger);
    let checkpoint = core.export_active_run_checkpoint().unwrap().unwrap();
    let mut fresh = core_with_profile_for_continue(&core, 12898);
    fresh
        .continue_from_active_run_checkpoint(
            player,
            &checkpoint.core_payload_json,
            checkpoint.loads_used,
            checkpoint.run_peak_certified_power,
        )
        .unwrap();
    assert_eq!(fresh.state.run.as_ref().unwrap().credit_ledgers, ledger);
    enter(&mut fresh, player, "4");
    let view = fresh
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .prepared_volleys_snapshot()
        .unwrap();
    let next = view.owners.iter().find(|v| v.copy == fragment).unwrap();
    assert_eq!(next.balance, 10 - i64::from(entry.unpaid));
    assert!(!next.can_borrow && !next.borrowed);
    assert_eq!(
        next.caster, None,
        "equipped but undeployed copy still receives settlement"
    );
    drop(fixture);
}

#[test]
fn canonical_greed_world_reward_volley_debt_continue_and_ready_armor() {
    world(true);
}
#[test]
fn canonical_greed_world_reward_volley_debt_continue_and_area_accessory() {
    world(false);
}
