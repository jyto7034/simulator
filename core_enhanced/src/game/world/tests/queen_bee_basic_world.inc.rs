// Actual catalog profile + reward/slot/deployment/ordinary basic-action consumer.
#[test]
fn queen_bee_basic_world_actual_grant_equip_layers_and_unique_heals() {
    use crate::game::{
        battle::event_log::BattleLogEvent,
        combat_preview::DeploymentZoneKind,
        data::{abnormality_data::BasicAttackAction, ValidatedGameData},
    };
    for injured in [0usize, 1, 2] {
        let data = ValidatedGameData::load_live_embedded().unwrap();
        let definition = data
            .equipment_data
            .items
            .iter()
            .find(|i| i.id == "placeholder_queen_bee_weapon")
            .unwrap();
        let profile = definition.weapon_profile.clone().unwrap();
        assert!(matches!(
            profile.action,
            BasicAttackAction::HealLowestHealth { max_targets: 3, .. }
        ));
        let definition_uuid = definition.uuid;
        let (mut core, player) = start_selected_with_data(
            &["field_medic", "breach_guard", "maintenance_tech"],
            GameDataBuilder::from_validated(&data).build_arc(),
        );
        let employees: Vec<_> = ["field_medic", "breach_guard", "maintenance_tech"]
            .into_iter()
            .map(|name| {
                core.state
                    .roster
                    .iter()
                    .find(|e| e.employee_definition_id == name)
                    .unwrap()
                    .uuid
            })
            .collect();
        let granted = core
            .apply_grant_effects(&[RewardEffect::GrantEquipment {
                equipment_id: "placeholder_queen_bee_weapon".into(),
            }])
            .unwrap();
        let copy = core
            .state
            .inventory
            .equipments
            .iter()
            .find(|i| i.meta.uuid == definition_uuid)
            .expect("reward must create actual persistent copy")
            .instance_uuid;
        assert!(granted.inventory_diff.added.iter().any(|item| matches!(item, crate::game::resources::InventoryItemDto::Equipment(e) if e.uuid == copy && e.definition_id == "placeholder_queen_bee_weapon")));
        core.execute(
            player,
            PlayerBehavior::EquipItem {
                item_uuid: copy,
                target_unit: employees[0],
            },
        )
        .unwrap();
        assert_eq!(
            core.state
                .inventory
                .equipments
                .iter()
                .find(|i| i.instance_uuid == copy)
                .unwrap()
                .equipped_to,
            Some(employees[0])
        );
        let node = force_first_available_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_monster",
            MapNodePayload::Encounter {
                encounter_id: Some("suppress_burrowing_heaven".into()),
            },
        );
        select_and_confirm_map_node(&mut core, player, node);
        // Explicit fixture-only TP and injuries. No weapon/profile or heal-path override.
        core.state
            .active_battle
            .as_mut()
            .unwrap()
            .tactical_resource
            .apply_delta(99);
        let active = core.state.active_battle.as_ref().unwrap();
        let cells: Vec<_> = active
            .combat_preview
            .deployment_zones
            .iter()
            .flat_map(|z| z.cells.iter().map(move |p| (z.kind, *p)))
            .filter(|(_, p)| {
                active
                    .battle
                    .validate_player_tactical_destination(*p, None)
                    .is_ok()
            })
            .collect();
        let mut placements = None;
        for (kind, origin) in &cells {
            if *kind != DeploymentZoneKind::Platform {
                continue;
            }
            let tiles = profile
                .defense_tile_range
                .affected_tiles(*origin, FacingDirection::Right)
                .unwrap();
            let ground = cells
                .iter()
                .find(|(k, p)| *k == DeploymentZoneKind::Ground && tiles.contains(p));
            let platform = cells.iter().find(|(k, p)| {
                *k == DeploymentZoneKind::Platform && p != origin && tiles.contains(p)
            });
            if let (Some((_, ground)), Some((_, platform))) = (ground, platform) {
                placements = Some([*origin, *ground, *platform]);
                break;
            }
        }
        let placements =
            placements.expect("live battlefield offers nearby platform and ground allies");
        let mut ids = Vec::new();
        for (i, employee) in employees.iter().enumerate() {
            execute_current_battle_mutation(
                &mut core,
                player,
                PlayerBehavior::DeployUnit {
                    employee_uuid: *employee,
                    position: placements[i],
                    facing: FacingDirection::Right,
                },
            )
            .unwrap();
            ids.push(
                core.state
                    .active_battle
                    .as_ref()
                    .unwrap()
                    .live_deployment
                    .as_ref()
                    .unwrap()
                    .deployed_units[employee]
                    .unit_instance_id,
            );
        }
        {
            let battle = &mut core.state.active_battle.as_mut().unwrap().battle;
            for (i, id) in ids.iter().enumerate() {
                let unit = battle.units.get_mut(id).unwrap();
                unit.stats.max_health = 100;
                unit.stats.current_health = if i > 0 && i <= injured {
                    30 + i as u32 * 10
                } else {
                    100
                };
            }
            battle.units.get_mut(&ids[0]).unwrap().stats.spell_power = 20;
            battle.enable_authoring_observation();
            assert_eq!(
                battle
                    .basic_action_snapshot(ids[0], true)
                    .unwrap()
                    .eligible_target_ids
                    .len(),
                injured
            );
        }
        // The other employee has a250ms area-heal windup; observe this200ms release first.
        core.advance_active_battle_by(240).unwrap();
        let battle = &core.state.active_battle.as_ref().unwrap().battle;
        let heals: Vec<_> = battle
            .event_log
            .entries_after_seq(0)
            .iter()
            .filter_map(|entry| match entry.event {
                BattleLogEvent::HpChanged {
                    source_instance_id: Some(source),
                    target_instance_id,
                    delta,
                    ..
                } if source == ids[0] && delta > 0 => {
                    Some((entry.time_ms, target_instance_id, delta))
                }
                _ => None,
            })
            .collect();
        assert_eq!(heals.len(), injured);
        for i in 1..=2 {
            assert_eq!(
                battle.units[&ids[i]].stats.current_health,
                if i <= injured {
                    30 + i as u32 * 10 + 14
                } else {
                    100
                }
            );
        }
        assert_eq!(battle.units[&ids[0]].stats.current_health, 100);
        assert!(!battle.event_log.entries_after_seq(0).iter().any(|e| matches!(e.event,
            BattleLogEvent::BasicAttackProjectileLaunched { attacker_instance_id, .. } if attacker_instance_id == ids[0])));
        let view = battle.runtime_fact_view();
        let mut action = None;
        for (time, target, delta) in &heals {
            assert_eq!((*time, *delta), (200, 14));
            let fact = view
                .actors()
                .find(|(id, _)| **id == *target)
                .unwrap()
                .1
                .received
                .latest_heal
                .as_ref()
                .unwrap();
            assert_eq!(fact.source.actor, Some(ids[0]));
            assert!(fact.source.cast.is_none());
            let source = fact.source.equipment.as_ref().unwrap();
            assert_eq!(source.definition_uuid, definition_uuid);
            assert_eq!(source.persistent_copy_uuid, Some(copy));
            if let Some(prior) = &action {
                assert_eq!(fact.source.action.as_ref(), Some(prior));
            } else {
                action = fact.source.action.clone();
            }
            assert!(action.is_some());
        }
        assert_eq!(
            view.actors()
                .find(|(id, _)| **id == ids[0])
                .map_or(0, |(_, r)| r.caused.damage_hp_lost),
            0
        );
        println!("H79 World actual copy={copy} injured={injured}, heals={heals:?}, same action={action:?}");
    }
}
