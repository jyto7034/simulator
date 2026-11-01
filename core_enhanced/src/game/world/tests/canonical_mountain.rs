use super::*;
use crate::authoring::composition::canonical_mountain_tests as canonical;
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
            encounter_id: Some("suppress_mountain".into()),
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
        {
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
            .find(|a| a.id == "t-01-75_mountain")
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
        observer.id = "mountain_smiling_bodies_gear_observation_enemy".into();
        observer.uuid = Uuid::from_u128(0xf122);
        observer.max_health = 300;
        observer.defense = 0;
        observer.magic_resist = 0;
        observer.attack = 0;
        let mut reserve = observer.clone();
        reserve.id = "mountain_smiling_bodies_reserve_enemy".into();
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
                profile_id: "mountain_smiling_bodies_gear_observation_enemy".into(),
                tier: crate::game::enums::Tier::I,
                count: 2,
            },
            crate::game::data::pve_data::PveWaveEnemyData::CorrodedEmployee {
                profile_id: "mountain_smiling_bodies_reserve_enemy".into(),
                tier: crate::game::enums::Tier::I,
                count: 1,
            },
        ]);
        fixture.write(path, ron::ser::to_string(&encounters).unwrap());
        fixture.write(
            "official/data/skills/mountain-of-smiling-bodies.skill.yaml",
            {
                let mut d = canonical::resolved("fragment");
                d["skills"][0]["admission"]["capacity"] = json!(5);
                d.to_string()
            },
        );
        fixture.write(
            "official/data/skills/mountain-of-smiling-bodies-equipment.skill.yaml",
            canonical::original("equipment"),
        );
        let files = vec![
            "data/skills/mountain-of-smiling-bodies.skill.yaml".into(),
            "data/skills/mountain-of-smiling-bodies-equipment.skill.yaml".into(),
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
            "official/data/skills/mountain-of-smiling-bodies-equipment.skill.yaml",
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
            reward_source_id: "normal_boss_mountain".into(),
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
        "placeholder_mountain_smiling_bodies_weapon",
    );
    let item = earn(
        &mut core,
        player,
        "7",
        RevivalStage::Stage3,
        if armor {
            "placeholder_mountain_smiling_bodies_armor"
        } else {
            "placeholder_mountain_smiling_bodies_accessory"
        },
    );
    assert_eq!(
        core.state
            .inventory
            .equipment_materials
            .amount("part_mountain_smiling_bodies"),
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
    let mut fresh = core_with_profile_for_continue(&core, 12799);
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
    let (mut taunt, mut hp, mut defense, mut primary) = (false, false, false, false);
    let mut growth_source = false;
    let mut bonus_source = false;
    let mut cast_at = None;
    let mut expired = false;
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
            }
            let active = core.state.active_battle.as_ref().unwrap();
            let now = active.execution.last_event_time_ms();
            let actor = active
                .battle
                .units
                .values()
                .find(|u| u.source_owned_uuid == employee && u.is_active())
                .expect("N10 actor survives observation battle");
            let actor_id = actor.instance_id;
            let observation = active.battle.authoring_observation().unwrap().unwrap();
            let facts = active.battle.runtime_fact_view();
            if let Some(change) = &facts.summary().latest_resource_change {
                if change.actual_delta() > 0
                    && change
                        .source
                        .equipment
                        .as_ref()
                        .is_some_and(|e| e.persistent_copy_uuid == Some(weapon))
                {
                    assert_eq!(change.source.actor, Some(actor_id));
                    assert_eq!(
                        change.actual_delta(),
                        2,
                        "normal primary kill grant from explicit fixture"
                    );
                    assert_eq!(
                        change.source.authored.as_ref().unwrap().source_file,
                        "published/data/skills/mountain-of-smiling-bodies-equipment.skill.yaml"
                    );
                    assert!(change.source.cast.is_none());
                    growth_source = true;
                }
            }
            for (_, summary) in facts.actors() {
                if let Some(hit) = &summary.received.latest_nonzero_damage {
                    if hit.source.actor == Some(actor_id) {
                        let source = hit
                            .source
                            .equipment
                            .as_ref()
                            .expect("ordinary attack carries weapon copy");
                        assert_eq!(source.persistent_copy_uuid, Some(weapon));
                        assert_eq!(
                            source.definition_uuid,
                            Uuid::parse_str("e2000000-0000-0000-0000-000000000013").unwrap()
                        );
                        assert!(
                            hit.source.cast.is_none(),
                            "ordinary attack must not invent a fragment cast"
                        );
                        assert!(hit.hp_before > hit.hp_after);
                        assert!(active.battle.event_log.entries_after_seq(0).iter().any(|entry| {
                            matches!(&entry.event, crate::game::battle::event_log::BattleLogEvent::HpChanged {
                                source_instance_id: Some(source), target_instance_id, hp_before, hp_after, ..
                            } if *source == actor_id && *target_instance_id == hit.target && *hp_before == hit.hp_before && *hp_after == hit.hp_after)
                        }), "the sourced damage must correspond to actual HP change");
                        hp = true;
                        assert!(hit.source.authored.is_none(), "ordinary hit keeps its own source");
                        for contributor in &hit.modifier_sources {
                            if contributor.authored.as_ref().is_some_and(|a| a.site.effect_id == "bonus.amount") {
                                assert_eq!(contributor.actor, Some(actor_id));
                                let copy = contributor.equipment.as_ref().unwrap();
                                assert_eq!(copy.persistent_copy_uuid, Some(weapon));
                                assert_eq!(copy.definition_uuid, Uuid::parse_str("e2000000-0000-0000-0000-000000000013").unwrap());
                                let authored = contributor.authored.as_ref().unwrap();
                                assert_eq!(authored.source_file, "published/data/skills/mountain-of-smiling-bodies-equipment.skill.yaml");
                                assert_eq!(authored.field_path, "equipment_attacks[0].program.bonus.amount");
                                assert_eq!(authored.site.action.definition_id.as_str(), "equipment:placeholder_mountain_smiling_bodies_weapon");
                                assert_eq!(authored.origin_definition_id.as_str(), "equipment:placeholder_mountain_smiling_bodies_weapon");
                                assert_eq!(authored.origin, crate::game::ability::authoring::EffectOrigin::Original);
                                assert_eq!(contributor.action_ordinal(), hit.source.action_ordinal());
                                assert!(contributor.cast.is_none());
                                bonus_source = true;
                            }
                        }
                    }
                }
            }
            if armor {
                if let Some(w) = &observation.latest_nearby_defense {
                    if w.actor == actor_id && w.bonus > 0 && w.time_ms == now {
                        assert_eq!(w.contributors.len(), 1);
                        let c = &w.contributors[0];
                        assert_eq!(c.equipment.persistent_copy_uuid, Some(item));
                        assert_eq!(
                            c.source_file,
                            "published/data/skills/mountain-of-smiling-bodies-equipment.skill.yaml"
                        );
                        assert_eq!((c.per_enemy, c.max_bonus), (10, 25));
                        let nearby = active
                            .battle
                            .units
                            .values()
                            .filter(|u| {
                                u.owner == crate::game::enums::Side::Opponent
                                    && u.is_active()
                                    && u.stats.current_health > 0
                                    && u.body
                                        .projected_tile()
                                        .chebyshev(&actor.body.projected_tile())
                                        <= 1
                            })
                            .count() as u32;
                        assert_eq!(
                            c.count, nearby,
                            "independent symmetric 3x3 center-tile count"
                        );
                        assert_eq!(w.bonus as u32, (c.count * 10).min(25));
                        assert_eq!(
                            w.actual_defense,
                            active.battle.units[&actor_id].stats.defense
                        );
                        defense = true;
                    }
                }
            } else if let Some(w) = &observation.latest_primary_damage {
                if w.actor == actor_id && w.percent > 0 {
                    assert_eq!(w.contributors.len(), 1);
                    assert_eq!(w.contributors[0].equipment.persistent_copy_uuid, Some(item));
                    assert_eq!(
                        w.contributors[0].source_file,
                        "published/data/skills/mountain-of-smiling-bodies-equipment.skill.yaml"
                    );
                    let expected = ((u64::from(w.target_max_hp - w.target_current_hp) * 100)
                        / u64::from(w.target_max_hp))
                    .min(50);
                    assert_eq!(u64::from(w.percent), expected);
                    assert_eq!(w.contributors[0].percent, w.percent);
                    let damage = facts
                        .actors()
                        .find(|(id, _)| **id == w.target)
                        .unwrap()
                        .1
                        .received
                        .latest_nonzero_damage
                        .as_ref()
                        .unwrap();
                    if damage.time_ms == w.time_ms
                        && damage.source.action.as_ref().unwrap().ordinal().0 == w.action
                    {
                        assert_eq!(damage.hp_before, w.target_current_hp);
                        assert_eq!(
                            active.battle.units[&w.target].stats.max_health,
                            w.target_max_hp
                        );
                        assert_eq!(
                            damage
                                .source
                                .equipment
                                .as_ref()
                                .unwrap()
                                .persistent_copy_uuid,
                            Some(weapon)
                        );
                        primary = true;
                    }
                }
            }
            let snapshot = active.battle.sustained_effect_snapshot().unwrap();
            let controls: Vec<_> = snapshot
                .active
                .iter()
                .filter(|e| e.control.is_some())
                .collect();
            if !controls.is_empty() {
                assert!(cast_at.is_some());
                for e in controls {
                    assert_eq!(e.caster, actor_id);
                    assert_eq!(e.fragment_copy_uuid, Some(fragment));
                    assert_eq!(e.skill_id.as_ref().unwrap().as_str(), canonical::SKILL);
                    assert!(e.authored_site.is_some() && e.cast_sequence.is_some());
                    assert!(!e.is_channel);
                }
                let w = observation.latest_sourced_sustained.as_ref().unwrap();
                assert_eq!(w.fragment.as_ref().unwrap().copy_uuid, fragment);
                let source = w.source.as_ref().unwrap();
                assert_eq!(source.cast.caster_instance_id, actor_id);
                assert_eq!(source.cast.skill_id.as_str(), canonical::SKILL);
                assert_eq!(
                    source.authored.source_file,
                    "published/data/skills/mountain-of-smiling-bodies.skill.yaml"
                );
                taunt = true;
            } else if taunt && cast_at.is_some_and(|t| now > t + 2002) {
                expired = true;
            }
            let ready = cast_at.is_none()
                && actor.resonance_current == 5
                && active.battle.units.values().any(|u| {
                    u.owner == crate::game::enums::Side::Opponent
                        && u.is_active()
                        && u.stats.current_health > 0
                        && u.body
                            .projected_tile()
                            .chebyshev(&actor.body.projected_tile())
                            <= 1
                });
            if ready {
                let hp_before: std::collections::BTreeMap<_, _> = active
                    .battle
                    .units
                    .iter()
                    .map(|(id, u)| (*id, u.stats.current_health))
                    .collect();
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
                // Command admission does not manufacture direct skill damage.
                assert_eq!(
                    core.state
                        .active_battle
                        .as_ref()
                        .unwrap()
                        .battle
                        .units
                        .iter()
                        .map(|(id, u)| (*id, u.stats.current_health))
                        .collect::<std::collections::BTreeMap<_, _>>(),
                    hp_before
                );
            }
            core.advance_active_battle_by(250).unwrap();
        },
    );
    eprintln!("N10 armor={armor} cast={cast_at:?} taunt={taunt} expired={expired} hp={hp} defense={defense} primary={primary} resources={:?}",core.run_resources().unwrap());
    assert!(
        cast_at.is_some() && taunt && expired,
        "actual reward fragment must apply and end its one-shot taunt"
    );
    assert!(hp, "actual acquired weapon deals HP damage");
    assert!(bonus_source, "mass bonus contributes its precise authored expression to real damage");
    assert!(
        growth_source,
        "positive primary kill grant retains authored weapon origin"
    );
    assert!(
        if armor { defense } else { primary },
        "actual second equipment contributes its authored behavior"
    );
    use crate::game::resource::ResourceOwner;
    let run = core.run_resources().unwrap();
    let mass = run
        .entries
        .iter()
        .find(|e| {
            e.resource.as_str() == "weapon_mass" && e.owner == ResourceOwner::EquipmentCopy(weapon)
        })
        .expect("primary kills persist on the acquired weapon copy");
    assert!(mass.balance > 0 && mass.balance % 2 == 0);
    assert!(run
        .entries
        .iter()
        .filter(|e| e.resource.as_str() == "weapon_mass")
        .all(|e| e.owner == ResourceOwner::EquipmentCopy(weapon) || e.balance == 0));
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    let checkpoint = core.export_active_run_checkpoint().unwrap().unwrap();
    let run_before = core.run_resources().unwrap().clone();
    let mut fresh = core_with_profile_for_continue(&core, 12701);
    fresh
        .continue_from_active_run_checkpoint(
            player,
            &checkpoint.core_payload_json,
            checkpoint.loads_used,
            checkpoint.run_peak_certified_power,
        )
        .unwrap();
    assert_eq!(fresh.run_resources().unwrap(), &run_before);
    drop(fixture);
}

#[test]
fn canonical_mountain_world_reward_taunt_growth_and_nearby_armor() {
    world(true);
}
#[test]
fn canonical_mountain_world_reward_taunt_growth_and_primary_accessory() {
    world(false);
}
