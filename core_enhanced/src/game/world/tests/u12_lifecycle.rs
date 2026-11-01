//! U12 policy oracles authored independently of the implementation.
//! Value-model tests are structural evidence, not approved-host start evidence.
use super::*;
use crate::game::expedition_lifecycle::{
    ExpeditionInstance, ExpeditionLifecycle, ExpeditionVault, FirstReturnStory,
    VaultEquipment, VaultFragment,
};
use crate::application::{
    ApplicationCommand, ApplicationDocument, ApplicationExecutionContext,
    ApplicationSnapshotKind, CoreApplication,
};
use std::collections::{BTreeMap, BTreeSet};

fn command(value: Value) -> Result<ApplicationCommand, crate::application::ApplicationError> {
    ApplicationCommand::from_document(ApplicationDocument::new(value))
}

fn snapshot(app: &CoreApplication, kind: ApplicationSnapshotKind) -> Value {
    app.snapshot(kind).unwrap().read_model().as_value().clone()
}

fn all_snapshots(app: &CoreApplication) -> Vec<Value> {
    ApplicationSnapshotKind::ALL.iter().map(|kind| snapshot(app, *kind)).collect()
}

fn issue(app: &mut CoreApplication, player: Uuid, value: Value) {
    app.execute(player, command(value).unwrap(), ApplicationExecutionContext::default())
        .expect("accepted U12 public command");
}

fn finish_briefing(app: &mut CoreApplication, player: Uuid) -> Value {
    for _ in 0..32 {
        let run = snapshot(app, ApplicationSnapshotKind::Run);
        if run["visual_novel"].is_null() { return run; }
        let vn = &run["visual_novel"];
        issue(app, player, json!({"type":"advance_visual_novel",
            "session_id":vn["session_id"],"current_scene_id":vn["current_scene_id"]}));
    }
    panic!("finite briefing did not terminate within 32 scenes");
}

fn start_selected_public(app: &mut CoreApplication, player: Uuid) -> Value {
    issue(app, player, json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":false}));
    let selection = finish_briefing(app, player);
    assert_eq!(selection["game_state"], "selecting_starter_employees");
    assert_eq!(selection["game_state_context"]["required_count"], 3);
    let candidates = selection["game_state_context"]["candidates"].as_array().unwrap();
    assert_eq!(candidates.len(), 6);
    for role in ["Dealer", "Tank", "Healer"] {
        assert_eq!(candidates.iter().filter(|candidate| candidate["role"] == role).count(), 2);
    }
    let ids: Vec<_> = candidates.iter().take(3).map(|candidate| candidate["employee_definition_id"].clone()).collect();
    issue(app, player, json!({"type":"select_starter_employees","employee_definition_ids":ids,"loan_copy_ids":[]}));
    snapshot(app, ApplicationSnapshotKind::Run)
}

fn reject_unchanged(app: &mut CoreApplication, player: Uuid, value: Value) {
    let before = all_snapshots(app);
    let checkpoint_before = app.export_active_run_checkpoint().unwrap()
        .map(|checkpoint| checkpoint.core_payload_json().to_vec());
    let rejected = match command(value) {
        Err(_) => true,
        Ok(command) => app.execute(player, command, ApplicationExecutionContext::default()).is_err(),
    };
    assert!(rejected, "invalid input must reject, not accept a no-op");
    assert_eq!(all_snapshots(app), before, "all public roots remain unchanged");
    assert_eq!(app.export_active_run_checkpoint().unwrap()
        .map(|checkpoint| checkpoint.core_payload_json().to_vec()), checkpoint_before,
        "rejection must preserve continuation bytes");
}

fn equipment(uuid: u128, definition: &str) -> VaultEquipment {
    VaultEquipment {
            weapon_stage: Default::default(), copy_uuid: Uuid::from_u128(uuid), definition_id: definition.into(),
        equipment_type: EquipmentType::Weapon, enhancement_level: 0 }
}

fn fragment(uuid: u128, definition: &str) -> VaultFragment {
    VaultFragment { investment: Default::default(), copy_uuid: Uuid::from_u128(uuid), definition_id: definition.into(),
        unlocked_forms: BTreeSet::from(["base".into(), "awakened".into()]),
        selected_form: "awakened".into() }
}

mod start_identity;
mod codec;
mod tutorial;
mod loans_terminal;
mod recruitment;
mod return_exhaustion;
mod recovery;
mod cost;

fn core_command(core: &mut GameCore, player: Uuid, value: Value) -> Result<BehaviorResult, GameError> {
    core.execute(player, serde_json::from_value(value).expect("U12 command schema"))
}

fn core_roots(core: &GameCore) -> Value {
    json!({"runtime": non_battle_state_without_profile_fingerprint(core),
        "profile": core.state.player_profile,
        "continue": core.export_active_run_checkpoint().unwrap().map(|copy| copy.core_payload_json)})
}

pub(super) fn selecting_core(mut core: GameCore, player: Uuid) -> (GameCore, Vec<String>) {
    let start = core.execute(player, PlayerBehavior::StartNewGame {
        game_mode: GameMode::Expedition, replace_active_run: false }).unwrap();
    let BehaviorResult::RunStartFlowStarted { visual_novel: Some(vn), .. } = start else { panic!("briefing"); };
    core.execute(player, PlayerBehavior::AdvanceVisualNovel {
        session_id: vn.session_id, current_scene_id: VisualNovelSceneId::new(vn.current_scene_id) }).unwrap();
    let ids = core.state.starter_candidates.iter().take(3)
        .map(|candidate| candidate.employee_definition_id.clone()).collect();
    (core, ids)
}
