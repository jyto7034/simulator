//! B1 regression: authored fixture content, public application commands, untouched
//! exported checkpoints. Amounts are test oracles, not production balance values.

use crate::application::*;
use crate::game::{
    battle::types::BattleVerdict,
    behavior::{GameStateContextDto, RunSnapshotDto},
    data::{
        pve_data::{PveEncounterClass, PveTerminalConditionData},
        reward_data::RewardMetadata,
        run_policy_data::OrdinaryCombatDifficultyBand,
        GameDataBuilder, ValidatedGameData,
    },
    enums::RewardMode,
    map::MapNodeState,
    reward::{ExperienceTargetPolicy, RewardEffect},
};
use serde_json::{json, Value};
use uuid::Uuid;

const PLAYER: Uuid = Uuid::from_u128(7001);
const REWARD: Uuid = Uuid::from_u128(0x70010001);

fn execute(
    app: &mut CoreApplication,
    document: Value,
) -> Result<ApplicationResult, ApplicationError> {
    let command = ApplicationCommand::from_document(document.into()).unwrap();
    app.execute(PLAYER, command, Default::default())
}

fn command(app: &mut CoreApplication, document: Value) {
    execute(app, document.clone()).unwrap_or_else(|error| panic!("{document}: {error:?}"));
}

fn snapshot(app: &CoreApplication, kind: ApplicationSnapshotKind) -> Value {
    app.snapshot(kind).unwrap().read_model().as_value().clone()
}

fn fixture(initial: u32, mode: RewardMode, difficulty_bonus: bool) -> ValidatedGameData {
    let base = ValidatedGameData::load_live_embedded().unwrap();
    let mut policy = (*base.run_policy).clone();
    policy.expedition_deterrence_initial = initial;
    if difficulty_bonus {
        // The seed's first combat is Standard in this profile. Keep a valid
        // authored ordering while making percentage rounding observable.
        let profile = policy
            .combat_difficulty
            .profiles
            .iter_mut()
            .find(|profile| profile.id == "facility_rooms_v1")
            .unwrap();
        profile.authored_fixed = false;
        for band in &mut profile.ordinary_bands {
            band.tuning.reward_multiplier_percent = match band.band {
                OrdinaryCombatDifficultyBand::Easy => 100,
                OrdinaryCombatDifficultyBand::Standard => 125,
                OrdinaryCombatDifficultyBand::Hard => 150,
            };
        }
        for role in &mut profile.role_tuning {
            role.tuning.reward_multiplier_percent = 150;
        }
    }
    let mut rewards = (*base.reward_data).clone();
    rewards.rewards.push(RewardMetadata {
        id: "s07_quantity_choice".into(),
        uuid: REWARD,
        name: "choice".into(),
        description: String::new(),
        icon: String::new(),
        effects: vec![
            RewardEffect::GrantEnkephalin { amount: 17 },
            RewardEffect::GrantEquipmentDust { amount: 3 },
            RewardEffect::GrantExperience {
                amount: 11,
                target: ExperienceTargetPolicy::AliveRoster,
            },
            RewardEffect::GrantEquipment {
                equipment_id: base.equipment_data.items[0].id.clone(),
            },
        ],
    });
    let mut pve = (*base.pve_data).clone();
    for encounter in &mut pve.encounters {
        if encounter.encounter_class != PveEncounterClass::Normal {
            continue;
        }
        for wave in &mut encounter.waves {
            wave.required_for_victory = false;
        }
        for outcome in &mut encounter.terminal_outcomes {
            if outcome.verdict == BattleVerdict::Victory {
                outcome.condition = PveTerminalConditionData::TimeReached { time_ms: 10 };
            }
        }
        for outcome in &mut encounter.outcome_policies {
            if encounter.terminal_outcomes.iter().any(|entry| {
                entry.id == outcome.outcome_id && entry.verdict == BattleVerdict::Victory
            }) {
                outcome.reward_mode = mode;
                outcome.reward_uuids = vec![REWARD];
                outcome.grant_node_bonus = difficulty_bonus;
            }
        }
    }
    GameDataBuilder::from_validated(&base)
        .with_run_policy(policy)
        .with_pve(pve)
        // Preserve the official pools and rebuild metadata lookup indexes.
        .with_rewards(serde_json::from_value(serde_json::to_value(rewards).unwrap()).unwrap())
        .build_arc()
        .into()
}

fn enter_first_battle(app: &mut CoreApplication) {
    command(
        app,
        json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":false}),
    );
    for _ in 0..32 {
        let raw = snapshot(app, ApplicationSnapshotKind::Run);
        let view: RunSnapshotDto = serde_json::from_value(raw.clone()).unwrap();
        if let Some(vn) = view.visual_novel {
            let action = if let Some(choice) = vn.choices.iter().find(|choice| choice.selectable) {
                json!({"type":"select_visual_novel_choice","session_id":vn.session_id,"current_scene_id":vn.current_scene_id,"choice_id":choice.choice_id})
            } else {
                json!({"type":"advance_visual_novel","session_id":vn.session_id,"current_scene_id":vn.current_scene_id})
            };
            command(app, action);
            continue;
        }
        if let GameStateContextDto::SelectingStarterEmployees {
            required_count,
            candidates,
        } = view.game_state_context
        {
            let ids: Vec<_> = candidates
                .into_iter()
                .take(required_count)
                .map(|entry| entry.employee_definition_id)
                .collect();
            command(
                app,
                json!({"type":"select_starter_employees","employee_definition_ids":ids,"loan_copy_ids":[]}),
            );
            continue;
        }
        let enabled = |name: &str| {
            raw["action_capabilities"]
                .as_array()
                .unwrap()
                .iter()
                .any(|action| action["action"] == name && action["enabled"] == true)
        };
        // The fixed route has one initial reward activity before combat.
        if enabled("ExitReward") {
            command(app, json!({"type":"exit_reward"}));
        } else if enabled("ClaimReward") {
            command(app, json!({"type":"claim_reward"}));
        } else if enabled("SelectReward") {
            let selected = snapshot(app, ApplicationSnapshotKind::SelectedEvent);
            let id = selected["rewards"][0]["uuid"].clone();
            assert!(
                !id.is_null(),
                "initial reward activity must have a candidate"
            );
            command(app, json!({"type":"select_reward","reward_id":id}));
        } else {
            let node = view
                .map
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|node| node.state == MapNodeState::Available)
                .unwrap_or_else(|| {
                    panic!(
                        "unexpected preparation state: {}",
                        raw["game_state_context"]
                    )
                });
            command(app, json!({"type":"select_map_node","node_id":node.id}));
            command(app, json!({"type":"confirm_enter_node"}));
            let after = snapshot(app, ApplicationSnapshotKind::Run);
            if after["action_capabilities"]
                .as_array()
                .unwrap()
                .iter()
                .any(|action| {
                    action["action"] == "ChooseExpeditionContinuation" && action["enabled"] == true
                })
            {
                command(
                    app,
                    json!({"type":"choose_expedition_continuation","choice":"Continue"}),
                );
            }
            if app.has_active_battle() {
                return;
            }
        }
    }
    panic!("the authored route did not reach its first combat");
}

fn current(app: &CoreApplication) -> Value {
    let checkpoint = app.export_active_run_checkpoint().unwrap().unwrap();
    let saved: Value = serde_json::from_slice(checkpoint.core_payload_json()).unwrap();
    saved["continuation"]["current"].clone()
}

fn restart(app: &CoreApplication, data: &ValidatedGameData) -> CoreApplication {
    let checkpoint = app.export_active_run_checkpoint().unwrap().unwrap();
    let mut restored = CoreApplication::new(data.clone(), 9999);
    restored
        .install_authoritative_player_profile(
            app.snapshot(ApplicationSnapshotKind::PlayerProfile)
                .unwrap()
                .read_model()
                .clone(),
        )
        .unwrap();
    restored
        .continue_from_active_run_checkpoint(PLAYER, &checkpoint)
        .unwrap();
    assert_eq!(current(&restored), current(app));
    assert_eq!(
        snapshot(&restored, ApplicationSnapshotKind::SelectedEvent),
        snapshot(app, ApplicationSnapshotKind::SelectedEvent)
    );
    restored
}

fn equipment_count(state: &Value) -> usize {
    state["inventory"]["equipments"]["items"]
        .as_object()
        .unwrap()
        .len()
}

fn check_grant(before: &Value, after: &Value, amount: u32, xp: u32) {
    assert_eq!(
        after["enkephalin"]["amount"].as_u64().unwrap(),
        before["enkephalin"]["amount"].as_u64().unwrap() + u64::from(amount)
    );
    assert_eq!(
        equipment_count(after),
        equipment_count(before) + 1,
        "singular grant is not multiplied"
    );
    let employees = before["roster"]["employees"].as_object().unwrap();
    assert!(!employees.is_empty());
    for (id, employee) in employees {
        assert_eq!(
            after["roster"]["employees"][id]["experience"]
                .as_u64()
                .unwrap(),
            employee["experience"].as_u64().unwrap() + u64::from(xp)
        );
    }
}

fn exercise(mode: RewardMode) {
    for (initial, multiplier) in [(100, 1), (1, 2)] {
        for difficulty_bonus in [false, true] {
            let data = fixture(initial, mode, difficulty_bonus);
            let mut app = CoreApplication::new(data.clone(), 7001);
            enter_first_battle(&mut app);
            // No employee is deployed: only the explicit AliveRoster reward
            // contributes XP here. Existing U14 tests cover survival/bonus XP.
            let before = current(&app);
            app.advance_active_battle_by(50, 0).unwrap();
            let summary = snapshot(&app, ApplicationSnapshotKind::SelectedEvent);
            // 125% rounds 17/3/11 to 21/4/14 before the quantity multiplier.
            let (amount, dust, xp) = if difficulty_bonus {
                (21, 4, 14)
            } else {
                (17, 3, 11)
            };
            let (amount, dust, xp) = (amount * multiplier, dust * multiplier, xp * multiplier);
            let expected = json!([
                {"GrantEnkephalin":{"amount":amount}},
                {"GrantEquipmentDust":{"amount":dust}},
                {"GrantExperience":{"amount":xp,"target":"AliveRoster"}},
                {"GrantEquipment":{"equipment_id":data.equipment_data.items[0].id}},
            ]);
            assert_eq!(summary["rewards"].as_array().unwrap().len(), 1);
            assert_eq!(summary["rewards"][0]["effects"], expected);
            app = restart(&app, &data);
            let at_result = current(&app);
            command(&mut app, json!({"type":"complete_combat_result"}));
            assert_eq!(current(&app), at_result, "ACK must not grant again");
            assert!(execute(&mut app, json!({"type":"complete_combat_result"})).is_err());
            app = restart(&app, &data);
            if mode == RewardMode::ChooseOne {
                assert_eq!(current(&app)["enkephalin"], before["enkephalin"]);
                assert_eq!(equipment_count(&current(&app)), equipment_count(&before));
                let rewards = snapshot(&app, ApplicationSnapshotKind::SelectedEvent);
                assert_eq!(rewards["rewards"][0]["effects"], expected);
                assert!(execute(&mut app, json!({"type":"claim_reward"})).is_err());
                assert!(execute(&mut app, json!({"type":"exit_reward"})).is_err());
                command(&mut app, json!({"type":"select_reward","reward_id":REWARD}));
                app = restart(&app, &data);
                command(&mut app, json!({"type":"claim_reward"}));
            }
            check_grant(&before, &current(&app), amount, xp);
            let inventory = snapshot(&app, ApplicationSnapshotKind::Inventory);
            assert!(inventory["equipment_materials"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["material_id"] == "equipment_dust" && entry["amount"] == dust));
            app = restart(&app, &data);
            let granted = current(&app);
            assert!(execute(&mut app, json!({"type":"claim_reward"})).is_err());
            assert_eq!(current(&app), granted);
            if mode == RewardMode::ChooseOne {
                command(&mut app, json!({"type":"exit_reward"}));
                app = restart(&app, &data);
                check_grant(&before, &current(&app), amount, xp);
            }
        }
    }
}

#[test]
fn s07_choose_one_quantities_match_display_grant_and_continue() {
    exercise(RewardMode::ChooseOne);
}

#[test]
fn s07_claim_all_quantities_match_display_grant_and_continue() {
    exercise(RewardMode::ClaimAll);
}
