//! U16-4 normal authored rounds. All combat progress uses public elapsed steps.
use super::*;
use crate::game::{
    data::{live_ron::GameplayRonSourceSet, ValidatedGameData},
    map::*,
    stress_combat::*,
};

fn data(edit: impl FnOnce(&mut FacilityTemplateDatabase)) -> ValidatedGameData {
    data_with_profile(edit, false)
}
fn data_with_profile(
    edit: impl FnOnce(&mut FacilityTemplateDatabase),
    veteran: bool,
) -> ValidatedGameData {
    let mut sources = GameplayRonSourceSet::embedded();
    let mut db = FacilityTemplateDatabase::from_ron_str(sources.facility_templates).unwrap();
    let l = &mut db.layouts[0];
    l.golden_bough_plans.clear();
    for a in &mut l.activities {
        a.golden_bough_candidate = false;
    }
    l.rooms.retain(|r| r.id == "ENTRY" || r.id == "B1-01");
    l.connections
        .retain(|c| c.from == "ENTRY" && c.to == "B1-01");
    let mut activity = l.activity("early_reward").unwrap().clone();
    activity.id = "stress".into();
    activity.reward_activity = Some(RewardActivityKind::StressCombat);
    activity.first_entry_cost = Some(1);
    l.activities.push(activity);
    let r = l.rooms.iter_mut().find(|r| r.id == "B1-01").unwrap();
    r.reservations = [("2", "stress"), ("5", "save"), ("8", "stress")]
        .into_iter()
        .map(|(slot, id)| FacilityReservation {
            slot_id: MapSlotId::new(slot),
            activity_id: id.into(),
        })
        .collect();
    r.elite_slots.clear();
    r.omen_slot = None;
    for a in &mut l.allocation.sizes {
        for p in a.elite_percent_by_depth.values_mut() {
            *p = 0;
        }
    }
    edit(&mut db);
    let text = ron::ser::to_string(&db).unwrap();
    sources.facility_templates = &text;
    // A supported authored veteran profile exercises final-round success;
    // first-round extraction separately retains the unchanged starter stats.
    let definitions = sources.employee_definitions.replace(
        "attack: 10",
        if veteran { "attack: 80" } else { "attack: 10" },
    );
    sources.employee_definitions = &definitions;
    GameDataBase::load_live_sources(sources).unwrap()
}
fn start(unrestrained: bool) -> (GameCore, Uuid, MapNodeId) {
    start_with_profile(unrestrained, false)
}
fn start_with_profile(unrestrained: bool, veteran: bool) -> (GameCore, Uuid, MapNodeId) {
    let p = Uuid::from_u128(16401);
    let mut c = GameCore::new(data_with_profile(|_| {}, veteran), 16401);
    c.execute(
        p,
        PlayerBehavior::StartNewGame {
            game_mode: GameMode::Expedition,
            replace_active_run: false,
        },
    )
    .unwrap();
    for _ in 0..16 {
        let Some(vn) = c.get_run_snapshot_dto().unwrap().visual_novel else {
            break;
        };
        c.execute(
            p,
            PlayerBehavior::AdvanceVisualNovel {
                session_id: vn.session_id,
                current_scene_id: VisualNovelSceneId::new(vn.current_scene_id),
            },
        )
        .unwrap();
    }
    c.execute(
        p,
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
    if unrestrained {
        c.state.run.as_mut().unwrap().run_progression.deterrence = 0;
    }
    let id = c
        .state
        .run
        .as_ref()
        .unwrap()
        .map
        .nodes
        .iter()
        .find(|n| n.slot_id.as_str() == "2")
        .unwrap()
        .id;
    let preview = c
        .current_map_view()
        .unwrap()
        .nodes
        .into_iter()
        .find(|n| n.id == id)
        .unwrap()
        .entry_preview
        .unwrap()
        .stress_combat
        .unwrap();
    assert_eq!(
        (preview.total_rounds, preview.quantity_multiplier),
        (5, if unrestrained { 2 } else { 1 })
    );
    c.execute(p, PlayerBehavior::SelectMapNode { node_id: id })
        .unwrap();
    c.execute(p, PlayerBehavior::ConfirmEnterNode).unwrap();
    assert!(matches!(c.get_state(), GameState::InBattle { .. }));
    (c, p, id)
}
fn deploy(c: &mut GameCore, p: Uuid) {
    let employees: Vec<_> = ["scout_runner", "breach_guard"]
        .iter()
        .map(|id| {
            c.state
                .roster
                .iter()
                .find(|e| e.employee_definition_id == *id)
                .unwrap()
                .uuid
        })
        .collect();
    // Official initial TP20 pays for two deployments at10 each. Do not assume
    // that the whole three-person roster can enter before regeneration.
    for employee in employees {
        let profile =
            crate::game::combat_setup::player_spawns::effective_combat_profile_for_employee(
                c.roster().unwrap(),
                c.inventory().unwrap(),
                &c.state.skill_fragments,
                c.game_data.as_ref(),
                employee,
            )
            .unwrap();
        let active = c.state.active_battle.as_ref().unwrap();
        let route = &active.combat_preview.routes[0];
        let goal = route.end;
        let mut cells: Vec<_> = active
            .combat_preview
            .deployment_zones
            .iter()
            .filter(|z| z.kind.supports_affinity(profile.deployment_affinity))
            .flat_map(|z| z.cells.iter().copied())
            .collect();
        cells.sort_by_key(|cell| (cell.chebyshev(&goal), cell.y, cell.x));
        let mut done = false;
        for position in cells {
            // Defend the object tile where attackers stop, not the brief
            // approach segment that fast enemies have already crossed.
            let facing = if position.x < goal.x {
                FacingDirection::Right
            } else if position.x > goal.x {
                FacingDirection::Left
            } else if position.y > goal.y {
                FacingDirection::Up
            } else {
                FacingDirection::Down
            };
            if execute_current_battle_mutation(
                c,
                p,
                PlayerBehavior::DeployUnit {
                    employee_uuid: employee,
                    position,
                    facing,
                },
            )
            .is_ok()
            {
                done = true;
                break;
            }
        }
        assert!(done, "normal starter must be deployable");
    }
}
fn view(c: &GameCore) -> StressCombatView {
    c.stress_combat_view().unwrap().unwrap()
}
fn until_choice_or_result(c: &mut GameCore) {
    for _ in 0..240 {
        if c.state.active_battle.is_none() || view(c).awaiting_choice {
            return;
        }
        c.advance_active_battle_by(250).unwrap();
    }
    let a = c.state.active_battle.as_ref().unwrap();
    panic!(
        "60s pilot failed: {:?}; {:?}",
        view(c),
        a.battle
            .units
            .values()
            .map(|u| (u.owner, u.body.projected_tile(), u.stats.current_health))
            .collect::<Vec<_>>()
    );
}
fn act(c: &mut GameCore, p: Uuid, extract: bool) -> Result<BehaviorResult, GameError> {
    let v = view(c);
    execute_current_battle_mutation(
        c,
        p,
        PlayerBehavior::StressCombatAct {
            battle_uuid: v.battle_uuid,
            completed_rounds: v.completed_rounds,
            extract,
        },
    )
}
fn restore(c: &GameCore, p: Uuid) -> GameCore {
    let saved = c.export_active_run_checkpoint().unwrap().unwrap();
    let mut fresh = GameCore::new(c.game_data.clone(), 999);
    fresh
        .install_authoritative_player_profile(c.state.player_profile.clone())
        .unwrap();
    fresh
        .continue_from_active_run_checkpoint(
            p,
            &saved.core_payload_json,
            saved.loads_used,
            saved.run_peak_certified_power,
        )
        .unwrap();
    fresh
}
fn state(c: &GameCore) -> Value {
    let a = c.state.active_battle.as_ref().unwrap();
    json!({"time":a.execution.last_event_time_ms(),"kernel":a.battle.transaction_debug_snapshot(),"queue":a.battle.event_queue_debug_snapshot(),"roster":a.employee_attempt.roster,"resource":a.tactical_resource.to_dto(),"deployment":a.live_deployment_dto(a.employee_roster()),"view":view(c)})
}
fn assets(c: &GameCore) -> Value {
    json!({"inventory":c.state.inventory,"roster":c.state.roster,"fragments":c.state.skill_fragments,"relics":c.state.relics,"crafting":c.state.crafting_resources,"enkephalin":c.state.enkephalin})
}

#[test]
fn u164_pilot_first_round_freezes_and_extracts_once() {
    let (mut c, p, id) = start(false);
    let before = c.state.enkephalin.amount;
    assert_eq!(
        serde_json::to_value(view(&c).next_rewards).unwrap(),
        json!([{"GrantEnkephalin":{"amount":10}}])
    );
    deploy(&mut c, p);
    until_choice_or_result(&mut c);
    assert!(
        c.state.active_battle.is_some(),
        "pilot must protect target and clear first wave"
    );
    assert_eq!(view(&c).completed_rounds, 1);
    assert!(view(&c).awaiting_choice);
    assert_eq!(c.state.enkephalin.amount, before);
    assert_ne!(
        c.state.run.as_ref().unwrap().map.node(id).unwrap().state,
        MapNodeState::Completed
    );
    let frozen = state(&c);
    c.advance_active_battle_by(60000).unwrap();
    assert_eq!(state(&c), frozen);
    for b in [
        PlayerBehavior::PauseBattle,
        PlayerBehavior::ResumeBattle,
        PlayerBehavior::RetreatBattle,
    ] {
        assert!(execute_current_battle_mutation(&mut c, p, b).is_err());
        assert_eq!(state(&c), frozen);
    }
    let snapshot = c.get_run_snapshot_json().unwrap();
    assert_eq!(
        snapshot["game_state_context"]["stress_combat"]["completed_rounds"],
        1
    );
    let v = view(&c);
    assert!(execute_current_battle_mutation(
        &mut c,
        p,
        PlayerBehavior::StressCombatAct {
            battle_uuid: v.battle_uuid,
            completed_rounds: 0,
            extract: true
        }
    )
    .is_err());
    assert_eq!(state(&c), frozen);
    let fresh = restore(&c, p);
    assert!(fresh.state.active_battle.is_none());
    assert_eq!(fresh.state.enkephalin.amount, before);
    assert_ne!(
        fresh
            .state
            .run
            .as_ref()
            .unwrap()
            .map
            .node(id)
            .unwrap()
            .state,
        MapNodeState::Completed
    );
    act(&mut c, p, true).unwrap();
    assert!(c.state.active_battle.is_none());
    assert_eq!(c.state.enkephalin.amount, before + 10);
    assert_eq!(
        c.state.run.as_ref().unwrap().map.node(id).unwrap().state,
        MapNodeState::Completed
    );
    let after = c.state.enkephalin.amount;
    assert!(c
        .execute(
            p,
            PlayerBehavior::StressCombatAct {
                battle_uuid: v.battle_uuid,
                completed_rounds: 1,
                extract: true
            }
        )
        .is_err());
    assert_eq!(c.state.enkephalin.amount, after);
    let fresh = restore(&c, p);
    assert_eq!(fresh.state.enkephalin.amount, after);
    assert_eq!(
        fresh
            .state
            .run
            .as_ref()
            .unwrap()
            .map
            .node(id)
            .unwrap()
            .state,
        MapNodeState::Completed
    );
}

#[test]
fn u164_five_rounds_preserve_one_battle_and_double_only_success() {
    for unrestrained in [false, true] {
        let (mut c, p, id) = start_with_profile(unrestrained, true);
        let before = c.state.enkephalin.amount;
        let battle_uuid = view(&c).battle_uuid;
        deploy(&mut c, p);
        for round in 1..=5 {
            until_choice_or_result(&mut c);
            if round == 5 {
                break;
            }
            assert!(
                c.state.active_battle.is_some(),
                "round {round} must clear successfully"
            );
            let v = view(&c);
            assert_eq!(v.battle_uuid, battle_uuid);
            assert_eq!(v.completed_rounds, round);
            assert_eq!(v.quantity_multiplier, if unrestrained { 2 } else { 1 });
            let a = c.state.active_battle.as_ref().unwrap();
            let time = a.execution.last_event_time_ms();
            let deployed = a
                .live_deployment_dto(a.employee_roster())
                .unwrap()
                .deployed_units
                .iter()
                .map(|d| d.unit_instance_id)
                .collect::<Vec<_>>();
            assert_eq!(c.state.enkephalin.amount, before);
            act(&mut c, p, false).unwrap();
            let a = c.state.active_battle.as_ref().unwrap();
            assert_eq!(a.battle_uuid, battle_uuid);
            assert_eq!(a.execution.last_event_time_ms(), time);
            assert_eq!(
                a.live_deployment_dto(a.employee_roster())
                    .unwrap()
                    .deployed_units
                    .iter()
                    .map(|d| d.unit_instance_id)
                    .collect::<Vec<_>>(),
                deployed
            );
        }
        assert!(
            matches!(c.get_state(), GameState::CombatResult { .. }),
            "five rounds must auto settle"
        );
        assert_eq!(
            c.state.enkephalin.amount,
            before + 150 * if unrestrained { 2 } else { 1 }
        );
        assert_eq!(
            c.state.run.as_ref().unwrap().map.node(id).unwrap().state,
            MapNodeState::Completed
        );
    }
}

#[test]
fn u164_manual_retreat_restores_battle_assets_consumes_node_and_preserves_entry_cost() {
    let (mut c, p, id) = start(false);
    let roots = assets(&c);
    let cost = c.state.run.as_ref().unwrap().run_progression.deterrence;
    deploy(&mut c, p);
    c.advance_active_battle_by(1500).unwrap();
    execute_current_battle_mutation(&mut c, p, PlayerBehavior::RetreatBattle).unwrap();
    assert_eq!(assets(&c), roots);
    assert!(c.state.active_battle.is_none());
    let run = c.state.run.as_ref().unwrap();
    assert_eq!(run.map.node(id).unwrap().state, MapNodeState::Completed);
    assert_eq!(run.run_progression.deterrence, cost);
    assert!(run.run_progression.entry_receipts.contains(&id));
    assert!(c
        .execute(p, PlayerBehavior::SelectMapNode { node_id: id })
        .is_err());
    let fresh = restore(&c, p);
    assert_eq!(assets(&fresh), roots);
    assert_eq!(
        fresh
            .state
            .run
            .as_ref()
            .unwrap()
            .map
            .node(id)
            .unwrap()
            .state,
        MapNodeState::Completed
    );
}

#[test]
fn u164_protected_target_loss_drops_pending_and_does_not_grant_rounds() {
    let (mut c, p, id) = start(true);
    let before = c.state.enkephalin.amount;
    deploy(&mut c, p);
    until_choice_or_result(&mut c);
    assert_eq!(view(&c).completed_rounds, 1);
    let before_xp: Vec<_> = c
        .state
        .roster
        .iter()
        .map(|e| (e.uuid, e.experience))
        .collect();
    act(&mut c, p, false).unwrap();
    let employees: Vec<_> = c
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .live_deployment_dto(c.state.active_battle.as_ref().unwrap().employee_roster())
        .unwrap()
        .deployed_units
        .iter()
        .map(|d| d.employee_uuid)
        .collect();
    for employee_uuid in &employees {
        execute_current_battle_mutation(
            &mut c,
            p,
            PlayerBehavior::WithdrawUnit {
                employee_uuid: *employee_uuid,
            },
        )
        .unwrap();
    }
    // Actual enemies destroy the target after a completed but unclaimed round.
    for _ in 0..240 {
        if c.state.active_battle.is_none() {
            break;
        }
        c.advance_active_battle_by(1000).unwrap();
    }
    assert!(matches!(c.get_state(), GameState::CombatResult { .. }));
    assert_eq!(c.state.enkephalin.amount, before);
    assert_eq!(
        c.state.run.as_ref().unwrap().map.node(id).unwrap().state,
        MapNodeState::Completed
    );
    let result = c.state.confirmed_combat_result.as_ref().unwrap();
    let summary = serde_json::to_value(&result.summary).unwrap();
    assert_eq!(summary["verdict"], "defeat");
    for (id, before) in before_xp {
        assert_eq!(
            c.state.roster.get(&id).unwrap().experience - before,
            if employees.contains(&id) { 10 } else { 0 },
            "defeat receives one undoubled survival XP grant"
        );
    }
}

#[test]
fn u164_authoring_rejects_missing_wave_round_count_and_selected_xp() {
    let sources = GameplayRonSourceSet::embedded();
    let mut db = FacilityTemplateDatabase::from_ron_str(sources.facility_templates).unwrap();
    let p = db.layouts[0].stress_combat.as_mut().unwrap();
    let encounters: crate::game::data::pve_data::PveEncounterDatabase =
        ron::de::from_str(sources.pve_encounters).unwrap();
    assert!(p.validate(&encounters).is_ok());
    let good = p.clone();
    p.rounds.pop();
    assert!(p.validate(&encounters).is_err());
    *p = good.clone();
    p.rounds[1].wave_ids = vec!["unknown".into()];
    assert!(p.validate(&encounters).is_err());
    *p = good.clone();
    p.rounds[1].wave_ids = p.rounds[0].wave_ids.clone();
    assert!(p.validate(&encounters).is_err());
    *p = good;
    p.rounds[0].rewards = vec![RewardEffect::GrantExperience {
        amount: 10,
        target: crate::game::reward::ExperienceTargetPolicy::SelectedEmployee,
    }];
    assert!(p.validate(&encounters).is_err());
}

#[test]
fn u164_authoring_rejects_unrepresentable_later_round_reward() {
    let mut sources = GameplayRonSourceSet::embedded();
    let mut db = FacilityTemplateDatabase::from_ron_str(sources.facility_templates).unwrap();
    let policy = db.layouts[0].stress_combat.as_mut().unwrap();
    policy.rounds[4].rewards = vec![RewardEffect::GrantEnkephalin { amount: u32::MAX }];
    let text = ron::ser::to_string(&db).unwrap();
    sources.facility_templates = &text;
    assert!(GameDataBase::load_live_sources(sources).is_err(), "a supported pack must reject a later reward that cannot be displayed or paid at the fixed unrestrained multiplier");
}

#[test]
fn u164_authoring_rejects_future_round_actor_dependency() {
    use crate::game::data::pve_data::{PveEncounterDatabase, PveWavePrimaryTriggerData as Trigger};
    let mut sources = GameplayRonSourceSet::embedded();
    let mut encounters: PveEncounterDatabase = ron::de::from_str(sources.pve_encounters).unwrap();
    let mut binding = encounters
        .encounters
        .iter()
        .find_map(|e| e.boss_runtime.clone())
        .unwrap();
    assert_eq!(binding.actors.len(), 1);
    binding.actors[0].wave_id = "stress_wave_5".into();
    let actor_slot_id = binding.actors[0].actor_slot_id.clone();
    let encounter = encounters
        .encounters
        .iter_mut()
        .find(|e| e.id == "stress_defense_v1")
        .unwrap();
    encounter.boss_runtime = Some(binding);
    encounter.waves[4].activation.primary = Trigger::AtTimeMs { time_ms: 1000 };
    encounter.waves[0].activation.primary = Trigger::ActorHealthAtOrBelowPercent {
        actor_slot_id,
        percent: 100,
    };
    let text = ron::ser::to_string(&encounters).unwrap();
    sources.pve_encounters = &text;
    let error = GameDataBase::load_live_sources(sources).err()
        .expect("round one cannot wait on an actor gated until round five");
    assert!(
        error.to_string().contains("stress combat policy"),
        "{error}"
    );
    // The same supported actor binding in the current round is legal.
    let mut db = FacilityTemplateDatabase::from_ron_str(sources.facility_templates).unwrap();
    let rounds = &mut db.layouts[0].stress_combat.as_mut().unwrap().rounds;
    rounds[0].wave_ids = vec!["stress_wave_5".into()];
    for (round, wave) in rounds[1..].iter_mut().zip(1..=4) {
        round.wave_ids = vec![format!("stress_wave_{wave}")];
    }
    let layout = ron::ser::to_string(&db).unwrap();
    sources.facility_templates = &layout;
    GameDataBase::load_live_sources(sources).expect("earlier actor dependency is accepted");
}

fn load_cumulative_rewards(rounds: Vec<Vec<RewardEffect>>) -> Result<(), String> {
    let mut sources = GameplayRonSourceSet::embedded();
    let mut layouts = FacilityTemplateDatabase::from_ron_str(sources.facility_templates).unwrap();
    let policy = layouts.layouts[0].stress_combat.as_mut().unwrap();
    for (index, round) in policy.rounds.iter_mut().enumerate() {
        round.rewards = rounds.get(index).cloned().unwrap_or_default();
    }
    let layout_text = ron::ser::to_string(&layouts).unwrap();
    sources.facility_templates = &layout_text;
    // A second supported material verifies that distinct stack owners are not
    // incorrectly combined into a single global reward budget.
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase =
        ron::de::from_str(sources.equipment).unwrap();
    let mut material = equipment.materials.iter()
        .find(|m| m.id == "revival_catalyst").unwrap().clone();
    material.id = "u164_other_parts".into();
    material.uuid = Uuid::from_u128(16499);
    equipment.materials.push(material);
    let equipment_text = ron::ser::to_string(&equipment).unwrap();
    sources.equipment = &equipment_text;
    GameDataBase::load_live_sources(sources).map(|_| ()).map_err(|e| e.to_string())
}

#[test]
fn u164_authoring_cumulative_currency_boundary() {
    let split = [429_496_729, 429_496_729, 429_496_729, 429_496_729, 429_496_731];
    let rewards = |amounts: &[u32]| amounts.iter()
        .map(|&amount| vec![RewardEffect::GrantEnkephalin { amount }]).collect();
    // Total 2,147,483,647; doubled 4,294,967,294 is representable.
    load_cumulative_rewards(rewards(&split)).unwrap();
    let mut excessive = split;
    excessive[4] += 1;
    for amounts in [excessive, [500_000_000; 5], [1_000_000_000; 5]] {
        let error = load_cumulative_rewards(rewards(&amounts)).unwrap_err();
        assert!(error.contains("cumulative"), "{error}");
    }
    let error = load_cumulative_rewards(vec![vec![
        RewardEffect::GrantEnkephalin { amount: 1_073_741_824 },
        RewardEffect::GrantEnkephalin { amount: 1_073_741_824 },
    ]]).unwrap_err();
    assert!(error.contains("cumulative"), "same-round duplicates: {error}");
}

#[test]
fn u164_authoring_cumulative_dust_alias_and_material_boundary() {
    let material = |id: &str, amount| RewardEffect::GrantEquipmentMaterial {
        material_id: id.into(), amount,
    };
    for first in [
        RewardEffect::GrantEquipmentDust { amount: 1_000_000_000 },
        material("equipment_dust", 1_000_000_000),
        material("revival_catalyst", 1_000_000_000),
    ] {
        let id = if matches!(&first, RewardEffect::GrantEquipmentMaterial { material_id, .. }
            if material_id == "revival_catalyst") { "revival_catalyst" } else { "equipment_dust" };
        load_cumulative_rewards(vec![vec![first.clone()], vec![material(id, 1_147_483_647)]]).unwrap();
        let error = load_cumulative_rewards(vec![vec![first], vec![material(id, 1_147_483_648)]]).unwrap_err();
        assert!(error.contains("cumulative"), "{id}: {error}");
    }
}

#[test]
fn u164_authoring_cumulative_keeps_distinct_owners_and_xp_grants() {
    use crate::game::reward::ExperienceTargetPolicy;
    let half = 2_147_483_647;
    load_cumulative_rewards(vec![vec![
        RewardEffect::GrantEnkephalin { amount: half },
        RewardEffect::GrantEquipmentDust { amount: half },
        RewardEffect::GrantEquipmentMaterial { material_id: "revival_catalyst".into(), amount: half },
        RewardEffect::GrantEquipmentMaterial { material_id: "u164_other_parts".into(), amount: half },
        // XP is consumed by leveling after each grant, not retained as a sum
        // across grants. Preserve its existing per-effect multiplication limit.
        RewardEffect::GrantExperience { amount: half, target: ExperienceTargetPolicy::CombatParticipants },
        RewardEffect::GrantExperience { amount: half, target: ExperienceTargetPolicy::AliveRoster },
    ]]).unwrap();
}
