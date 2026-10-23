//! Normal authored route: no state grants, synthesized victories, or save edits.
use super::*;
use crate::game::data::abnormality_data::AbnormalityDatabase;
use crate::game::{
    data::{live_ron::GameplayRonSourceSet, ValidatedGameData},
    map::*,
    material_economy::MaterialPayment,
    revival::RevivalStage,
};

fn authored_data() -> ValidatedGameData {
    authored_variant(false, 3)
}

#[test]
fn u17_completed_preview_owners_retire_before_savepoint_and_public_continue() {
    use crate::application::{ApplicationSnapshotKind, CoreApplication};
    let mut core = GameCore::new(authored_data(), 12077);
    let player = Uuid::from_u128(0xc0de1200000000000000000000001001);
    start(&mut core, player);
    for slot in ["8", "4", "5", "1"] {
        enter(&mut core, player, slot);
        win(&mut core, player);
        core.execute(player, PlayerBehavior::CompleteCombatResult).unwrap();
    }
    enter(&mut core, player, "2");
    let offer = core.state.active_node_content.as_ref().unwrap().as_shop().unwrap()
        .offers.iter().find(|offer| matches!(offer.merchandise, ShopMerchandise::RevivalCatalyst { .. }))
        .unwrap().offer_id;
    core.execute(player, PlayerBehavior::PurchaseItem { offer_id: offer }).unwrap();
    core.execute(player, PlayerBehavior::ExitShop).unwrap();
    enter(&mut core, player, "3");
    core.execute(player, PlayerBehavior::CompleteNode).unwrap();
    super::continuation_contract::assert_fresh_application_next(&core, player,
        &[serde_json::json!({"type":"load_run_checkpoint"})]);
    let hatched = hatch_stage(&mut core, player, "6", RevivalStage::Stage2);
    let data = core.game_data.clone();
    let profile = serde_json::to_value(&core.state.player_profile).unwrap();
    let source = CoreApplication::from_core(core);
    let saved = source.export_active_run_checkpoint().unwrap().unwrap();
    let expected = source.snapshot(ApplicationSnapshotKind::Inventory).unwrap();
    let mut fresh = CoreApplication::new(data, 12077);
    fresh.install_authoritative_player_profile(profile.into()).unwrap();
    let (restored, admissions) = crate::game::battle::runtime_control::observe_retained_admissions(||
        fresh.continue_from_active_run_checkpoint(player, &saved));
    restored.expect("ordinary completed route + SavePoint + hatch fits default public Continue limits");
    assert_eq!(fresh.snapshot(ApplicationSnapshotKind::Inventory).unwrap().read_model().as_value(), expected.read_model().as_value());
    let document: Value = serde_json::from_slice(saved.core_payload_json()).unwrap();
    assert!(document["continuation"]["current"].get("relics").is_none());
    assert!(admissions.iter().any(|row| row.0 == "continue_typed_decode"),
        "whole Continue decoding must be metered even without relics");
    assert!(admissions.iter().any(|row| row.0 == "relic_continue_value"));
    let current = &document["continuation"]["current"]["combat_previews"];
    assert_eq!(current.as_object().unwrap().len(), 1, "only unresolved hatch retains its preview");
    assert!(current.get(hatched.0.to_string()).is_some());
    assert!(document["continuation"]["savepoint"]["snapshot"]["checkpoint"]["combat_previews"].as_object().unwrap().is_empty());
}
fn authored_variant(unrestrained: bool, choices: usize) -> ValidatedGameData {
    authored_route(unrestrained, choices, false)
}
fn authored_route(unrestrained: bool, choices: usize, only_gate_path: bool) -> ValidatedGameData {
    let mut sources = GameplayRonSourceSet::embedded();
    let mut maps: FacilityTemplateDatabase = ron::de::from_str(sources.facility_templates).unwrap();
    let layout = &mut maps.layouts[0];
    layout.golden_bough_plans.clear();
    layout
        .rooms
        .retain(|r| ["ENTRY", "B1-01", "B1-02", "B2-01"].contains(&r.id.as_str()));
    layout.connections.retain(|c| {
        (c.from == "ENTRY" && c.to == "B1-01")
            || (c.from == "B1-01" && ["B1-02", "B2-01"].contains(&c.to.as_str()))
    });
    for room in &mut layout.rooms {
        if room.id != "ENTRY" {
            room.template_id = Some(MapTemplateId::new("facility_9A"));
            room.reservations.clear();
            room.elite_slots.clear();
            room.omen_slot = None;
            room.safe_access_slots.clear();
            room.slot_display.clear();
        }
    }
    for room in &mut layout.rooms {
        if ["B1-02", "B2-01"].contains(&room.id.as_str()) {
            room.reservations = vec![FacilityReservation {
                slot_id: MapSlotId::new("1"),
                activity_id: "save".into(),
            }];
        }
    }
    let normal_activity = layout.allocation.normal_activity_id.clone();
    for activity in &mut layout.activities {
        activity.golden_bough_candidate = false;
    }
    for (id, kind, encounter) in [
        ("u17_boss", "boss_abnormality", "suppress_big_bad_wolf"),
        ("u17_elite", "combat_elite", "suppress_censored"),
    ] {
        layout.activities.push(FacilityActivityDefinition {
            id: id.into(),
            first_entry_cost: Some(1),
            golden_bough_candidate: false,
            label: id.into(),
            kind_id: MapNodeKindId::new(kind),
            payload: MapNodePayload::Encounter {
                encounter_id: Some(encounter.into()),
            },
            support: FacilityContentSupport::Supported,
            condition: None,
            reward_activity: None,
        });
    }
    let room = layout.rooms.iter_mut().find(|r| r.id == "B1-01").unwrap();
    room.template_id = Some(MapTemplateId::new("facility_9A"));
    room.reservations = [
        ("1", "u17_boss"),
        ("2", "shop"),
        ("3", "save"),
        ("4", "u17_elite"),
        ("5", "u17_elite"),
        ("8", normal_activity.as_str()),
        ("9", "maintenance"),
    ]
    .into_iter()
    .map(|(slot, activity)| FacilityReservation {
        slot_id: MapSlotId::new(slot),
        activity_id: activity.into(),
    })
    .collect();
    room.elite_slots.clear();
    room.omen_slot = None;
    room.safe_access_slots.clear();
    room.slot_display.clear();
    for size in &mut layout.allocation.sizes {
        size.event_counts = [(0, 100)].into();
        size.reward_counts = [(0, 100)].into();
        for percent in size.elite_percent_by_depth.values_mut() {
            *percent = 0;
        }
    }
    let template = maps
        .templates
        .iter_mut()
        .find(|t| t.id.as_str() == "facility_9A")
        .unwrap();
    let categories = &mut template
        .slots
        .iter_mut()
        .find(|s| s.id.as_str() == "1")
        .unwrap()
        .allowed_categories;
    if !categories.contains(&MapNodeCategory::Boss) {
        categories.push(MapNodeCategory::Boss);
    }
    template.connections = (1..=9)
        .flat_map(|i| {
            [
                FacilityConnection {
                    from_slot_id: MapSlotId::new("S"),
                    to_slot_id: MapSlotId::new(i.to_string()),
                    direction: MapEdgeDirection::Bidirectional,
                    reveal_with_slot_id: None,
                },
                FacilityConnection {
                    from_slot_id: MapSlotId::new(i.to_string()),
                    to_slot_id: MapSlotId::new("G"),
                    direction: MapEdgeDirection::ForwardOnly,
                    reveal_with_slot_id: None,
                },
            ]
        })
        .collect();
    if only_gate_path {
        template
            .connections
            .retain(|edge| edge.to_slot_id.as_str() != "G" || edge.from_slot_id.as_str() == "6");
    }
    let maps = ron::ser::to_string(&maps).unwrap();
    sources.facility_templates = &maps;
    let mut pve: PveEncounterDatabase = ron::de::from_str(sources.pve_encounters).unwrap();
    let revival = pve
        .reward_sources
        .iter_mut()
        .find(|s| s.reward_source_id == "normal_boss_big_bad_wolf")
        .unwrap()
        .revival
        .as_mut()
        .unwrap();
    revival.catalyst_amount = 2;
    revival.stage_2.equipment_choices.truncate(choices);
    revival.stage_3.equipment_choices.truncate(choices);
    if choices == 1 {
        // Separately authored same-kind revival may disable the base encounter's
        // natural progression; the hatch owner still owns its stage resolution.
        let mut alternate = pve
            .encounters
            .iter()
            .find(|e| e.id == "suppress_big_bad_wolf")
            .unwrap()
            .clone();
        alternate.id = "u17_revival_wolf".into();
        for policy in &mut alternate.outcome_policies {
            policy.advance_encounter_progression = false;
        }
        pve.encounters.push(alternate);
        let policy = pve
            .reward_sources
            .iter_mut()
            .find(|s| s.reward_source_id == "normal_boss_big_bad_wolf")
            .unwrap()
            .revival
            .as_mut()
            .unwrap();
        policy.stage_2.encounter_id = "u17_revival_wolf".into();
        policy.stage_3.encounter_id = "u17_revival_wolf".into();
    }
    let pve = ron::ser::to_string(&pve).unwrap();
    sources.pve_encounters = &pve;
    let mut abnormalities: AbnormalityDatabase = ron::de::from_str(sources.abnormalities).unwrap();
    for a in &mut abnormalities.items {
        if ["f-02-58_big_bad_wolf", "o-03-89_censored"].contains(&a.id.as_str()) {
            a.max_health = 20;
            a.attack = 1;
            a.defense = 0;
            a.magic_resist = 0;
            a.skill_id = None;
        }
    }
    let abnormalities = ron::ser::to_string(&abnormalities).unwrap();
    sources.abnormalities = &abnormalities;
    let mut enemies: CorrodedEmployeeProfileDatabase =
        ron::de::from_str(sources.corroded_employees).unwrap();
    for e in &mut enemies.profiles {
        e.max_health = 20;
        e.attack = 1;
        e.defense = 0;
        e.magic_resist = 0;
        e.skill_id = None;
        if e.basic_attack_range_preset.is_some() {
            e.basic_attack.defense_tile_range = None;
        }
    }
    let enemies = ron::ser::to_string(&enemies).unwrap();
    sources.corroded_employees = &enemies;
    // Supported authored starter range keeps this progression fixture finite
    // across the arena's three stationary reinforcement columns.
    let equipment=sources.equipment.replace("rows: [\"..X..\", \"..X..\", \"..X..\", \"..@..\", \".....\"]", "rows: [\"XXXXX\", \"XXXXX\", \"XXXXX\", \"XXXXX\", \"XXXXX\", \"XXXXX\", \"..@..\", \".....\"]");
    sources.equipment = &equipment;
    let fragment_start = sources
        .skill_fragments
        .find("id: \"fragment_censored_trace\"")
        .unwrap();
    let fragment_end = fragment_start
        + sources.skill_fragments[fragment_start..]
            .find("        SkillFragmentMetadata(")
            .unwrap();
    let fragment=sources.skill_fragments[fragment_start..fragment_end].replace("effect: BasicAttackModifier(\n                attack_bonus: 6,\n                attack_interval_ms_reduction: 60,\n            )", "effect: ActiveSkill(forms: {\"base\": (skill_id: \"training_berserker\", resonance_max: 100)}, dismantle_recipe: Some((enkephalin: 0, materials: [(material_id: \"equipment_dust\", amount: 1)])))");
    let fragments = format!(
        "{}{}{}",
        &sources.skill_fragments[..fragment_start],
        fragment,
        &sources.skill_fragments[fragment_end..]
    );
    sources.skill_fragments = &fragments;
    let definitions = sources
        .employee_definitions
        .replace("attack: 10", "attack: 80");
    sources.employee_definitions = &definitions;
    // RON 0.5 does not round-trip internally tagged ShopMerchandise tags;
    // retain authored identifiers and change only this exact price/quantity row.
    let shops = sources.shops.replace(
        "quantity: 1, base_price: Some(150)",
        "quantity: 4, base_price: Some(50)",
    );
    sources.shops = &shops;
    let run_policy = if unrestrained {
        sources.run_policy.replace(
            "expedition_deterrence_initial: 12",
            "expedition_deterrence_initial: 1",
        )
    } else {
        sources.run_policy.to_owned()
    };
    sources.run_policy = &run_policy;
    if let (Ok(directory), false, 3) = (std::env::var("U17_EXPORT_CONTENT"), unrestrained, choices)
    {
        let root = std::path::Path::new(&directory);
        std::fs::create_dir_all(root.join("data")).unwrap();
        std::fs::write(root.join("data/bundle.ron"), sources.manifest).unwrap();
        for (path, text) in sources.source_inventory() {
            let path = root.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
    }
    GameDataBase::load_live_sources(sources).expect("supported authored U17 route")
}
fn start(core: &mut GameCore, player: Uuid) {
    core.execute(
        player,
        PlayerBehavior::StartNewGame {
            game_mode: GameMode::Expedition,
            replace_active_run: false,
        },
    )
    .unwrap();
    while let Some(vn) = core.get_run_snapshot_dto().unwrap().visual_novel {
        core.execute(
            player,
            PlayerBehavior::AdvanceVisualNovel {
                session_id: vn.session_id,
                current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId::new(
                    vn.current_scene_id,
                ),
            },
        )
        .unwrap();
    }
    core.execute(
        player,
        PlayerBehavior::SelectStarterEmployees {
            employee_definition_ids: vec![
                "scout_runner".into(),
                "breach_guard".into(),
                "field_medic".into(),
            ],
            loan_copy_ids: vec![],
        },
    )
    .unwrap();
}
fn node(c: &GameCore, slot: &str) -> MapNodeId {
    c.run_state()
        .unwrap()
        .map
        .nodes
        .iter()
        .find(|n| n.slot_id.as_str() == slot)
        .unwrap()
        .id
}
fn confirm(c: &mut GameCore, p: Uuid) {
    c.execute(p, PlayerBehavior::ConfirmEnterNode).unwrap();
    if c.run_state()
        .unwrap()
        .run_progression
        .pending_continuation
        .is_some()
    {
        c.execute(
            p,
            PlayerBehavior::ChooseExpeditionContinuation {
                choice: crate::game::expedition_lifecycle::ExpeditionContinuationChoice::Continue,
            },
        )
        .unwrap();
    }
}
fn enter(c: &mut GameCore, p: Uuid, slot: &str) {
    let id = node(c, slot);
    c.execute(p, PlayerBehavior::SelectMapNode { node_id: id })
        .unwrap();
    confirm(c, p);
}
fn deploy(c: &mut GameCore, p: Uuid) {
    for definition in ["scout_runner", "breach_guard"] {
        let employee = c
            .state
            .roster
            .iter()
            .find(|e| e.employee_definition_id == definition)
            .unwrap()
            .uuid;
        let profile =
            crate::game::combat_setup::player_spawns::effective_combat_profile_for_employee(
                c.roster().unwrap(),
                c.inventory().unwrap(),
                &c.state.skill_fragments,
                c.game_data.as_ref(),
                employee,
            )
            .unwrap();
        let preview = &c.state.active_battle.as_ref().unwrap().combat_preview;
        let mut cells: Vec<_> = preview
            .deployment_zones
            .iter()
            .filter(|z| z.kind.supports_affinity(profile.deployment_affinity))
            .flat_map(|z| z.cells.iter().copied())
            .collect();
        let center = preview
            .routes
            .first()
            .map(|r| r.end.x)
            .unwrap_or(preview.width / 2);
        let target_x = center + if definition == "scout_runner" { -1 } else { 1 };
        cells.sort_by_key(|cell| (cell.y, (cell.x - target_x).abs(), cell.x));
        let facing = if definition == "scout_runner" {
            FacingDirection::Up
        } else {
            FacingDirection::Left
        };
        assert!(
            cells
                .into_iter()
                .any(|position| execute_current_battle_mutation(
                    c,
                    p,
                    PlayerBehavior::DeployUnit {
                        employee_uuid: employee,
                        position,
                        facing
                    }
                )
                .is_ok()),
            "normal deployment {definition}"
        );
    }
}
fn win(c: &mut GameCore, p: Uuid) {
    deploy(c, p);
    for tick in 0..300 {
        if c.state.active_battle.is_none() {
            break;
        }
        if tick > 0 && tick % 40 == 0 {
            let employee = c
                .state
                .roster
                .iter()
                .find(|e| e.employee_definition_id == "scout_runner")
                .unwrap()
                .uuid;
            let active = c.state.active_battle.as_ref().unwrap();
            let deployed = &active.live_deployment.as_ref().unwrap().deployed_units[&employee];
            let position = active.battle.units[&deployed.unit_instance_id]
                .body
                .projected_tile();
            let targets: Vec<_> = active
                .battle
                .units
                .values()
                .filter(|u| {
                    u.owner == crate::game::enums::Side::Opponent && u.stats.current_health > 0
                })
                .map(|u| u.body.projected_tile())
                .collect();
            let profile =
                crate::game::combat_setup::player_spawns::effective_combat_profile_for_employee(
                    c.roster().unwrap(),
                    c.inventory().unwrap(),
                    &c.state.skill_fragments,
                    c.game_data.as_ref(),
                    employee,
                )
                .unwrap();
            let pattern = crate::game::battle::types::effective_basic_attack_tile_range(
                &profile.basic_attack,
            );
            let mut candidates = Vec::new();
            for cell in active
                .combat_preview
                .deployment_zones
                .iter()
                .filter(|z| z.kind.supports_affinity(profile.deployment_affinity))
                .flat_map(|z| z.cells.iter().copied())
            {
                for (index, facing) in [
                    FacingDirection::Up,
                    FacingDirection::Right,
                    FacingDirection::Down,
                    FacingDirection::Left,
                ]
                .into_iter()
                .enumerate()
                {
                    let hits = targets
                        .iter()
                        .filter(|target| {
                            pattern
                                .contains_target_tile(cell, facing, **target)
                                .unwrap()
                        })
                        .count();
                    if hits > 0 {
                        candidates.push((hits, cell, facing, index));
                    }
                }
            }
            if let Some((_, destination, facing, _)) =
                candidates.into_iter().max_by_key(|(hits, cell, _, index)| {
                    (
                        *hits,
                        std::cmp::Reverse(cell.chebyshev(&position)),
                        std::cmp::Reverse(*index),
                    )
                })
            {
                // Public relocation uses its actual TP/cooldown/occupancy rules.
                // A matching direction or occupied destination may reject.
                let _ = execute_current_battle_mutation(
                    c,
                    p,
                    PlayerBehavior::TacticalRelocateUnit {
                        employee_uuid: employee,
                        destination,
                        facing,
                    },
                );
            }
        }
        c.advance_active_battle_by(250).unwrap();
    }
    assert!(
        matches!(c.get_state(), GameState::CombatResult { .. }),
        "75s finite combat pilot: {:?}; units={:?}; preview={:?}",
        c.get_state(),
        c.state.active_battle.as_ref().map(|a| a
            .battle
            .units
            .values()
            .map(|u| (u.owner, u.body.projected_tile(), u.stats.current_health))
            .collect::<Vec<_>>()),
        c.state.active_battle.as_ref().map(|a| (
            &a.combat_preview.battlefield_template_id,
            &a.combat_preview.deployment_zones
        ))
    );
    assert_eq!(
        c.get_selected_event_snapshot_json().unwrap().unwrap()["verdict"],
        json!("victory")
    );
}
#[test]
fn u17_public_boss_hatch_two_and_three_reward_choices_without_debug_grants() {
    for seed in [17001, 12077] {
        let mut core = GameCore::new(authored_data(), seed);
        let player = Uuid::from_u128(0xc0de1200000000000000000000001001);
        start(&mut core, player);
        enter(&mut core, player, "1");
        win(&mut core, player);
        assert_eq!(
            core.state
                .skill_fragments
                .count(&"fragment_big_bad_wolf_trace".into()),
            1
        );
        assert_eq!(
            core.run_state()
                .unwrap()
                .revival
                .eggs
                .get("normal_boss_big_bad_wolf"),
            Some(&RevivalStage::Stage2)
        );
        core.execute(player, PlayerBehavior::CompleteCombatResult)
            .unwrap();
        enter(&mut core, player, "2");
        let offer = core
            .state
            .active_node_content
            .as_ref()
            .unwrap()
            .as_shop()
            .unwrap()
            .offers
            .iter()
            .find(|o| matches!(o.merchandise, ShopMerchandise::RevivalCatalyst { .. }))
            .unwrap()
            .offer_id;
        core.execute(player, PlayerBehavior::PurchaseItem { offer_id: offer })
            .unwrap();
        core.execute(player, PlayerBehavior::ExitShop).unwrap();
        assert_eq!(
            core.state
                .inventory
                .equipment_materials
                .amount("revival_catalyst"),
            4
        );
        for (slot, stage, parts) in [
            ("6", RevivalStage::Stage2, 2),
            ("7", RevivalStage::Stage3, 6),
        ] {
            let id = node(&core, slot);
            core.execute(player, PlayerBehavior::SelectMapNode { node_id: id })
                .unwrap();
            assert_eq!(core.hatchable_node(), Some(id));
            core.execute(
                player,
                PlayerBehavior::HatchEncounter {
                    node_id: id,
                    reward_source_id: "normal_boss_big_bad_wolf".into(),
                    stage,
                    payment: vec![MaterialPayment {
                        material_id: "revival_catalyst".into(),
                        carried: 2,
                        warehouse: 0,
                    }],
                },
            )
            .unwrap();
            let view = &core.revival_snapshot().unwrap().hatched_nodes[&id];
            assert_eq!(
                view.encounter.payment.materials["revival_catalyst"].carried,
                2
            );
            assert_eq!(view.equipment_choices.len(), 3);
            confirm(&mut core, player);
            let deterrence = core.run_state().unwrap().run_progression.deterrence;
            assert!(
                !core
                    .state
                    .active_battle
                    .as_ref()
                    .unwrap()
                    .combat_preview
                    .deterrence
                    .seal_released
            );
            win(&mut core, player);
            assert_eq!(
                core.run_state().unwrap().run_progression.deterrence,
                deterrence,
                "revival never refills deterrence"
            );
            assert_eq!(
                core.state
                    .inventory
                    .equipment_materials
                    .amount("part_big_bad_wolf"),
                parts
            );
            assert_eq!(
                core.run_state()
                    .unwrap()
                    .revival
                    .eggs
                    .get("normal_boss_big_bad_wolf")
                    .copied(),
                stage.next()
            );
            super::continuation_contract::assert_fresh_application_next(&core, player,
                &[serde_json::json!({"type":"complete_combat_result"})]);
            let saved = core.export_active_run_checkpoint().unwrap().unwrap();
            let mut next = core_with_profile_for_continue(&core, 17002);
            next.continue_from_active_run_checkpoint(
                player,
                &saved.core_payload_json,
                saved.loads_used,
                saved.run_peak_certified_power,
            )
            .unwrap();
            core = next;
            core.execute(player, PlayerBehavior::CompleteCombatResult)
                .unwrap();
            let candidates = core
                .state
                .active_node_content
                .as_ref()
                .unwrap()
                .as_reward()
                .unwrap()
                .rewards
                .clone();
            assert_eq!(candidates.len(), 3);
            super::continuation_contract::assert_fresh_application_next(&core, player,
                &[serde_json::json!({"type":"select_reward", "reward_id":candidates[0].uuid}),
                  serde_json::json!({"type":"claim_reward"}),
                  serde_json::json!({"type":"claim_reward"}),
                  serde_json::json!({"type":"exit_reward"})]);
            let selected = candidates[if stage == RevivalStage::Stage2 { 0 } else { 1 }].uuid;
            core.execute(
                player,
                PlayerBehavior::SelectReward {
                    reward_id: selected,
                },
            )
            .unwrap();
            let after = core.get_inventory_snapshot_json().unwrap();
            core.execute(
                player,
                PlayerBehavior::SelectReward {
                    reward_id: selected,
                },
            )
            .unwrap();
            assert_eq!(core.get_inventory_snapshot_json().unwrap(), after);
            let saved = core.export_active_run_checkpoint().unwrap().unwrap();
            let mut fresh = core_with_profile_for_continue(&core, 17003);
            fresh
                .continue_from_active_run_checkpoint(
                    player,
                    &saved.core_payload_json,
                    saved.loads_used,
                    saved.run_peak_certified_power,
                )
                .unwrap();
            core = fresh;
            super::continuation_contract::assert_fresh_application_next(&core, player,
                &[serde_json::json!({"type":"claim_reward"}), serde_json::json!({"type":"exit_reward"})]);
            let count = core.state.inventory.equipments.len();
            core.execute(player, PlayerBehavior::ClaimReward).unwrap();
            assert_eq!(core.state.inventory.equipments.len(), count + 1);
            let paid = core.get_inventory_snapshot_json().unwrap();
            assert!(core.execute(player, PlayerBehavior::ClaimReward).is_err());
            assert_eq!(core.get_inventory_snapshot_json().unwrap(), paid);
            let saved = core.export_active_run_checkpoint().unwrap().unwrap();
            let mut fresh = core_with_profile_for_continue(&core, 17004);
            fresh
                .continue_from_active_run_checkpoint(
                    player,
                    &saved.core_payload_json,
                    saved.loads_used,
                    saved.run_peak_certified_power,
                )
                .unwrap();
            core = fresh;
            assert_eq!(core.get_inventory_snapshot_json().unwrap(), paid);
            super::continuation_contract::assert_fresh_application_next(&core, player,
                &[serde_json::json!({"type":"claim_reward"}), serde_json::json!({"type":"exit_reward"})]);
            core.execute(player, PlayerBehavior::ExitReward).unwrap();
        }
        assert_eq!(
            core.state
                .inventory
                .equipment_materials
                .amount("revival_catalyst"),
            0
        );
        assert_eq!(
            core.state
                .skill_fragments
                .count(&"fragment_big_bad_wolf_trace".into()),
            1
        );
        assert!(core.run_state().unwrap().revival.eggs.is_empty());
    }
}

fn buy_catalysts(core: &mut GameCore, player: Uuid) {
    enter(core, player, "2");
    let offer = core
        .state
        .active_node_content
        .as_ref()
        .unwrap()
        .as_shop()
        .unwrap()
        .offers
        .iter()
        .find(|o| matches!(o.merchandise, ShopMerchandise::RevivalCatalyst { .. }))
        .unwrap()
        .offer_id;
    core.execute(player, PlayerBehavior::PurchaseItem { offer_id: offer })
        .unwrap();
    core.execute(player, PlayerBehavior::ExitShop).unwrap();
}
fn hatch_stage(core: &mut GameCore, player: Uuid, slot: &str, stage: RevivalStage) -> MapNodeId {
    let id = node(core, slot);
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: id })
        .unwrap();
    core.execute(
        player,
        PlayerBehavior::HatchEncounter {
            node_id: id,
            reward_source_id: "normal_boss_big_bad_wolf".into(),
            stage,
            payment: vec![MaterialPayment {
                material_id: "revival_catalyst".into(),
                carried: 2,
                warehouse: 0,
            }],
        },
    )
    .unwrap();
    id
}
#[test]
fn u17_public_unrestrained_zero_choice_stages_keep_fixed_parts_and_no_fallback() {
    let mut core = GameCore::new(authored_variant(true, 0), 17001);
    let player = Uuid::from_u128(17001);
    start(&mut core, player);
    enter(&mut core, player, "1");
    win(&mut core, player);
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    buy_catalysts(&mut core, player);
    let equipment = core.state.inventory.equipments.len();
    for (slot, stage, parts) in [
        ("6", RevivalStage::Stage2, 4),
        ("7", RevivalStage::Stage3, 12),
    ] {
        hatch_stage(&mut core, player, slot, stage);
        confirm(&mut core, player);
        let preview = &core.state.active_battle.as_ref().unwrap().combat_preview;
        assert!(preview.deterrence.unrestrained);
        assert_eq!(preview.deterrence.victory_quantity_multiplier, 2);
        assert!(
            preview.deterrence.enemy_health_percent >= 150
                && preview.deterrence.enemy_health_percent <= 200
        );
        assert!(
            preview.deterrence.enemy_attack_percent >= 150
                && preview.deterrence.enemy_attack_percent <= 200
        );
        win(&mut core, player);
        assert_eq!(
            core.state
                .inventory
                .equipment_materials
                .amount("part_big_bad_wolf"),
            parts
        );
        let result = core.get_selected_event_snapshot_json().unwrap().unwrap();
        assert_eq!(result["no_reward_candidates"], true);
        assert_eq!(result["rewards"], json!([]));
        let saved = core.export_active_run_checkpoint().unwrap().unwrap();
        let mut fresh = core_with_profile_for_continue(&core, 17003);
        fresh
            .continue_from_active_run_checkpoint(
                player,
                &saved.core_payload_json,
                saved.loads_used,
                saved.run_peak_certified_power,
            )
            .unwrap();
        core = fresh;
        core.execute(player, PlayerBehavior::CompleteCombatResult)
            .unwrap();
        assert_eq!(core.get_state(), GameState::ViewingMap);
        assert_eq!(
            core.state.inventory.equipments.len(),
            equipment,
            "zero choices never substitute equipment"
        );
    }
    assert!(core.run_state().unwrap().revival.eggs.is_empty());
}
#[test]
fn u17_public_normal_elite_dismantle_repeat_and_new_expedition_reset() {
    let mut core = GameCore::new(authored_data(), 17001);
    let player = Uuid::from_u128(17001);
    start(&mut core, player);
    enter(&mut core, player, "6");
    win(&mut core, player);
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    assert!(core
        .run_state()
        .unwrap()
        .completed_reward_source_ids
        .is_empty());
    enter(&mut core, player, "4");
    win(&mut core, player);
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    assert_eq!(
        core.state
            .skill_fragments
            .count(&"fragment_censored_trace".into()),
        1
    );
    let equipment = core
        .state
        .inventory
        .equipments
        .iter()
        .filter(|e| e.meta.id == "censored_plate")
        .count();
    assert_eq!(equipment, 1);
    assert!(core.run_state().unwrap().revival.eggs.is_empty());
    enter(&mut core, player, "9");
    let copy = core
        .state
        .skill_fragments
        .copies()
        .find(|c| c.definition_id.as_str() == "fragment_censored_trace")
        .unwrap()
        .copy_uuid;
    core.execute(
        player,
        PlayerBehavior::DismantleSkillFragment {
            copy_uuid: copy,
            confirmed: true,
        },
    )
    .unwrap();
    core.execute(player, PlayerBehavior::CompleteNode).unwrap();
    let dust = core.state.crafting_resources.equipment_dust;
    enter(&mut core, player, "5");
    win(&mut core, player);
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    assert_eq!(
        core.state
            .skill_fragments
            .count(&"fragment_censored_trace".into()),
        0
    );
    assert_eq!(
        core.state
            .inventory
            .equipments
            .iter()
            .filter(|e| e.meta.id == "censored_plate")
            .count(),
        equipment
    );
    // Authored salvage4 at Elite150%=6, plus source repeat5.
    assert_eq!(core.state.crafting_resources.equipment_dust, dust + 11);
    assert!(core.run_state().unwrap().revival.eggs.is_empty());
    core.execute(player, PlayerBehavior::AbandonRun).unwrap();
    start(&mut core, player);
    assert!(core
        .run_state()
        .unwrap()
        .completed_reward_source_ids
        .is_empty());
    enter(&mut core, player, "4");
    win(&mut core, player);
    assert_eq!(
        core.state
            .skill_fragments
            .count(&"fragment_censored_trace".into()),
        1
    );
}
#[test]
fn u17_public_reserved_normal_rejects_hatch_and_same_floor_revisit_keeps_it() {
    let mut core = GameCore::new(authored_data(), 17001);
    let player = Uuid::from_u128(17001);
    start(&mut core, player);
    enter(&mut core, player, "1");
    win(&mut core, player);
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    buy_catalysts(&mut core, player);
    let reserved = node(&core, "8");
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: reserved })
        .unwrap();
    assert_eq!(
        core.game_data
            .pve_data
            .get_by_id(
                match &core
                    .run_state()
                    .unwrap()
                    .map
                    .node(reserved)
                    .unwrap()
                    .payload
                {
                    MapNodePayload::Encounter {
                        encounter_id: Some(id),
                    } => id,
                    _ => panic!("reserved normal"),
                }
            )
            .unwrap()
            .encounter_class,
        crate::game::data::pve_data::PveEncounterClass::Normal
    );
    assert_eq!(core.hatchable_node(), None);
    let before = json!((
        core.get_run_snapshot_json().unwrap(),
        core.get_inventory_snapshot_json().unwrap()
    ));
    assert!(core
        .execute(
            player,
            PlayerBehavior::HatchEncounter {
                node_id: reserved,
                reward_source_id: "normal_boss_big_bad_wolf".into(),
                stage: RevivalStage::Stage2,
                payment: vec![MaterialPayment {
                    material_id: "revival_catalyst".into(),
                    carried: 2,
                    warehouse: 0
                }]
            }
        )
        .is_err());
    assert_eq!(
        json!((
            core.get_run_snapshot_json().unwrap(),
            core.get_inventory_snapshot_json().unwrap()
        )),
        before
    );
    core.execute(player, PlayerBehavior::CancelSelectedNode)
        .unwrap();
    let hatched = hatch_stage(&mut core, player, "6", RevivalStage::Stage2);
    core.execute(player, PlayerBehavior::CancelSelectedNode)
        .unwrap();
    let record = core.run_state().unwrap().revival.nodes[&hatched].clone();
    enter(&mut core, player, "G");
    if core
        .run_state()
        .unwrap()
        .run_progression
        .pending_continuation
        .is_some()
    {
        core.execute(
            player,
            PlayerBehavior::ChooseExpeditionContinuation {
                choice: crate::game::expedition_lifecycle::ExpeditionContinuationChoice::Continue,
            },
        )
        .unwrap();
    }
    core.execute(
        player,
        PlayerBehavior::MoveToFacilityRoom {
            room_id: "B1-02".into(),
        },
    )
    .unwrap();
    enter(&mut core, player, "1");
    core.execute(player, PlayerBehavior::CompleteNode).unwrap();
    enter(&mut core, player, "G");
    if core
        .run_state()
        .unwrap()
        .run_progression
        .pending_continuation
        .is_some()
    {
        core.execute(
            player,
            PlayerBehavior::ChooseExpeditionContinuation {
                choice: crate::game::expedition_lifecycle::ExpeditionContinuationChoice::Continue,
            },
        )
        .unwrap();
    }
    core.execute(
        player,
        PlayerBehavior::MoveToFacilityRoom {
            room_id: "B1-01".into(),
        },
    )
    .unwrap();
    assert_eq!(core.run_state().unwrap().revival.nodes[&hatched], record);
    assert_eq!(
        core.state
            .inventory
            .equipment_materials
            .amount("revival_catalyst"),
        2
    );
    let saved = core.export_active_run_checkpoint().unwrap().unwrap();
    let mut fresh = core_with_profile_for_continue(&core, 17004);
    fresh
        .continue_from_active_run_checkpoint(
            player,
            &saved.core_payload_json,
            saved.loads_used,
            saved.run_peak_certified_power,
        )
        .unwrap();
    core = fresh;
    assert_eq!(core.run_state().unwrap().revival.nodes[&hatched], record);
    core.execute(
        player,
        PlayerBehavior::MoveToFacilityRoom {
            room_id: "B2-01".into(),
        },
    )
    .unwrap();
    assert!(core.run_state().unwrap().revival.nodes.is_empty());
    assert!(core
        .execute(
            player,
            PlayerBehavior::MoveToFacilityRoom {
                room_id: "B1-01".into()
            }
        )
        .is_err());
}

#[test]
fn u17_public_hatch_on_only_route_to_gate_preserves_graph_and_requires_manual_entry() {
    let mut core = GameCore::new(authored_route(false, 3, true), 17001);
    let player = Uuid::from_u128(17001);
    start(&mut core, player);
    enter(&mut core, player, "1");
    win(&mut core, player);
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    buy_catalysts(&mut core, player);
    let gate = node(&core, "G");
    let target = node(&core, "6");
    let topology = |core: &GameCore| {
        let map = &core.run_state().unwrap().map;
        json!((
            &map.map_template_id,
            map.facility_template_version,
            map.nodes
                .iter()
                .map(|n| (n.id, &n.slot_id))
                .collect::<Vec<_>>(),
            &core.facility_template_for_map(map).unwrap().connections
        ))
    };
    let edges = topology(&core);
    assert!(core
        .execute(player, PlayerBehavior::SelectMapNode { node_id: gate })
        .is_err());
    let id = hatch_stage(&mut core, player, "6", RevivalStage::Stage2);
    assert_eq!(id, target);
    assert_eq!(topology(&core), edges);
    assert!(core.state.active_battle.is_none());
    assert!(matches!(core.get_state(),GameState::NodeConfirm {node_id,..} if node_id==id));
    core.execute(player, PlayerBehavior::CancelSelectedNode)
        .unwrap();
    assert!(core
        .execute(player, PlayerBehavior::SelectMapNode { node_id: gate })
        .is_err());
}

#[test]
fn u17_public_single_choice_alternate_encounter_keeps_stage_owner() {
    let mut core = GameCore::new(authored_variant(false, 1), 17001);
    let player = Uuid::from_u128(17001);
    start(&mut core, player);
    enter(&mut core, player, "1");
    win(&mut core, player);
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    buy_catalysts(&mut core, player);
    hatch_stage(&mut core, player, "6", RevivalStage::Stage2);
    confirm(&mut core, player);
    win(&mut core, player);
    assert_eq!(
        core.state
            .inventory
            .equipment_materials
            .amount("part_big_bad_wolf"),
        2
    );
    assert_eq!(
        core.run_state().unwrap().revival.eggs["normal_boss_big_bad_wolf"],
        RevivalStage::Stage3
    );
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    let candidates = &core
        .state
        .active_node_content
        .as_ref()
        .unwrap()
        .as_reward()
        .unwrap()
        .rewards;
    assert_eq!(candidates.len(), 1);
    let selected = candidates[0].uuid;
    core.execute(
        player,
        PlayerBehavior::SelectReward {
            reward_id: selected,
        },
    )
    .unwrap();
    let before = core.state.inventory.equipments.len();
    core.execute(player, PlayerBehavior::ClaimReward).unwrap();
    assert_eq!(core.state.inventory.equipments.len(), before + 1);
}
