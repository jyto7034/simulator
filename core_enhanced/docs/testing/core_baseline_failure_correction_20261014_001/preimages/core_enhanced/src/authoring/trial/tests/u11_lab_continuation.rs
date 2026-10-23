//! Ephemeral Lab regression: prepared bulk loadouts survive real Retreat.
//! Duplicate Lab definitions remain invalid for ordinary persistent Continue.
use super::*;
use crate::{application::{ApplicationBattleAuthority,ApplicationCommand,ApplicationDocument,
    ApplicationExecutionContext},authoring::{LabEmployeeGroup,LabPlacement,LabPreparation,WorldLabPreparation},
    game::{battle::tile_range::FacingDirection,map::MapNodeCategory,resources::Position}};
use uuid::Uuid;

fn run(app:&CoreApplication)->Value {
    app.snapshot(ApplicationSnapshotKind::Run).unwrap().read_model().as_value().clone()
}
fn command(app:&mut CoreApplication,player:Uuid,input:Value,context:ApplicationExecutionContext)->Value {
    let parsed=ApplicationCommand::from_document(ApplicationDocument::new(input)).unwrap();
    app.execute(player,parsed,context).unwrap().payload().as_value().clone()
}
fn stable_roster(run:&Value)->Value {
    json!(run["roster"]["employees"].as_array().unwrap().iter().map(|employee|json!({
        "uuid":employee["uuid"],"definition":employee["employee_definition_id"],"level":employee["level"],
        "experience":employee["experience"],"condition":employee["condition"],"profile":employee["profile"],
        "equipped_items":employee["equipped_items"].as_array().unwrap().iter().map(|item|json!({
            "instance_uuid":item["instance_uuid"],"base_uuid":item["base_uuid"],"definition_id":item["definition_id"]})).collect::<Vec<_>>(),
        "active_fragment_copy":employee["skill_fragments"]["active_copy_uuid"],
        "baseline_fragments":employee["skill_fragments"]["baseline_ids"],
        "equipped_fragments":employee["skill_fragments"]["equipped"]
    })).collect::<Vec<_>>())
}
fn retreat(app:&mut CoreApplication,player:Uuid)->Result<Value,crate::application::ApplicationError> {
    let before=run(app);let context=&before["game_state_context"];
    let parsed=ApplicationCommand::from_document(ApplicationDocument::new(json!({"type":"retreat_battle"}))).unwrap();
    app.execute(player,parsed,ApplicationExecutionContext{
        expected_battle_authority:Some(ApplicationBattleAuthority{
            battle_uuid:serde_json::from_value(context["battle_uuid"].clone()).unwrap(),
            battle_state_revision:context["battle_state_revision"].as_u64().unwrap()}),
        ..Default::default()}).map(|result|result.payload().as_value().clone())
}
#[test]
fn world_lab_preparation_retreat_preserves_bulk_loadouts_and_normal_continue_rejects_duplicates(){
    let token=ValidatedGameData::load_live_embedded().unwrap();let player=Uuid::from_u128(25);
    let mut document=preparation_document(&token);
    document["preparation_steps"].as_array_mut().unwrap().truncate(4); // Actual normal Start/VN/starter prefix.
    document["steps"]=json!([]);
    let mut recipe=LabPreparation{schema_version:1,employees:vec![LabEmployeeGroup{
        definition_id:"records_clerk".into(),count:3,
        equipment_ids:Some(vec!["standard_training_sword".into(),"standard_armor".into()]),
        active_fragment_id:Some("training_berserker".into()),relics:vec![]}],
        encounter_id:"suppress_scarecrow".into(),category:MapNodeCategory::Combat,placements:vec![]};
    let discover=WorldLabPreparation::prepare(fixture_content(&token),checked(&document),recipe.clone(),
        TrialLimits::default(),&CancellationToken::default()).unwrap();
    let view:Value=serde_json::from_str(discover.reconnect(0,&discover.runtime_safety().authoring_execution(),
        &CancellationToken::default()).unwrap().document()).unwrap();
    let occupied:Vec<Position>=view.pointer("/payload/battle_update/checkpoint/units").unwrap().as_array().unwrap()
        .iter().map(|unit|serde_json::from_value(unit["position"].clone()).unwrap()).collect();
    let cell=view.pointer("/payload/battle_setup_snapshot/deployment_zones").unwrap().as_array().unwrap().iter()
        .filter(|zone|zone["kind"]=="Ground").flat_map(|zone|zone["cells"].as_array().unwrap()).map(|cell|
            serde_json::from_value::<Position>(cell.clone()).unwrap()).find(|cell|!occupied.contains(cell)).unwrap();
    recipe.placements.push(LabPlacement{employee_index:0,position:cell,facing:FacingDirection::Up});
    drop(discover);
    let mut trial=WorldLabPreparation::prepare(fixture_content(&token),checked(&document),recipe,
        TrialLimits::default(),&CancellationToken::default()).unwrap();
    let receipt=trial.lab_preparation().unwrap();assert_eq!(receipt.employee_ids.len(),3);assert_eq!(receipt.deployed_count,1);
    let before=run(&trial.state.application);let roster=stable_roster(&before);
    let actual=before["roster"]["employees"].as_array().unwrap();
    for id in &receipt.employee_ids {
        let employee=actual.iter().find(|row|row["uuid"]==id.to_string()).expect("actual prepared roster member");
        assert_eq!(employee["employee_definition_id"],"records_clerk");
        let equipped:Vec<_>=employee["equipped_items"].as_array().unwrap().iter().map(|row|row["definition_id"].as_str().unwrap()).collect();
        assert!(equipped.contains(&"standard_training_sword")&&equipped.contains(&"standard_armor"));
        assert!(employee["skill_fragments"]["active_copy_uuid"].is_string());
        assert!(employee["skill_fragments"]["equipped"].as_array().unwrap().iter().any(|row|row["id"]=="training_berserker"));
    }
    let initial=&before["game_state_context"]["abnormality_attempt"];
    assert_eq!(initial["attempts_started"],1);assert_eq!(initial["retreats_used"],0);
    let node=initial["node_id"].clone();assert!(node.is_string());
    let original_checkpoint=trial.state.application.export_active_run_checkpoint().unwrap().unwrap();
    // The Lab checkpoint is a deterministic comparison input, not a supported
    // normal save of its duplicated employee definitions. Prove rejection does
    // not disturb a separately prepared, live ordinary destination.
    let mut destination=TrialSession::prepare(fixture_content(&token),checked(&preparation_document(&token)),
        TrialLimits::default(),&CancellationToken::default()).unwrap();
    let destination_before=run(&destination.state.application);
    let destination_checkpoint=destination.state.application.export_active_run_checkpoint().unwrap().unwrap();
    let error=destination.state.application.continue_from_active_run_checkpoint(player,&original_checkpoint).unwrap_err();
    assert_eq!(error.code(),"invalid_action","ordinary Continue rejects an already live run before inspecting checkpoint content");
    assert_eq!(run(&destination.state.application),destination_before,"rejected Lab import preserves the live destination");
    assert_eq!(destination.state.application.export_active_run_checkpoint().unwrap().unwrap().core_payload_json(),
        destination_checkpoint.core_payload_json());
    assert_eq!(trial.state.application.export_active_run_checkpoint().unwrap().unwrap().core_payload_json(),
        original_checkpoint.core_payload_json(),"rejected import cannot alter the source Lab trial");
    // A fresh destination passes the ordinary no-existing-run admission gate,
    // so this second attempt reaches the strict imported ownership validator.
    let mut fresh=CoreApplication::new(token.clone(),0xDEAD);
    let fresh_before=run(&fresh);
    let fresh_checkpoint=fresh.export_active_run_checkpoint().unwrap().map(|checkpoint|
        (checkpoint.core_payload_json().to_vec(),checkpoint.loads_used(),checkpoint.run_peak_certified_power()));
    assert!(fresh_checkpoint.is_none(),"fresh destination has no confirmed run");
    let error=fresh.continue_from_active_run_checkpoint(player,&original_checkpoint).unwrap_err();
    assert_eq!(error.code(),"invalid_static_data");
    assert_eq!(error.details().as_value()["reason"],"recruitment queue/session/roster ownership is not exact");
    assert_eq!(run(&fresh),fresh_before,"ownership rejection preserves the fresh destination");
    assert_eq!(fresh.export_active_run_checkpoint().unwrap().map(|checkpoint|
        (checkpoint.core_payload_json().to_vec(),checkpoint.loads_used(),checkpoint.run_peak_certified_power())),fresh_checkpoint);
    assert_eq!(trial.state.application.export_active_run_checkpoint().unwrap().unwrap().core_payload_json(),
        original_checkpoint.core_payload_json(),"both rejected imports leave the source Lab unchanged");
    // The in-memory entry snapshot is valid for the isolated Lab's actual
    // Retreat. Each retreat preserves bulk copy identity and commits exactly
    // one ordinary attempt-ledger increment without persistent import.
    for expected_retreat in 1_u64..=2 {
        let entered=run(&trial.state.application);
        assert_eq!(entered["game_state_context"]["abnormality_attempt"]["attempts_started"],expected_retreat);
        assert_eq!(entered["game_state_context"]["abnormality_attempt"]["retreats_used"],expected_retreat-1);
        retreat(&mut trial.state.application,player).unwrap();
        let after=run(&trial.state.application);
        assert_eq!(after["game_state_context"]["type"],"node_confirm");
        assert_eq!(after["game_state_context"]["abnormality_attempt"]["node_id"],node);
        assert_eq!(after["game_state_context"]["abnormality_attempt"]["retreats_used"],expected_retreat);
        assert_eq!(after["game_state_context"]["abnormality_attempt"]["attempts_started"],expected_retreat);
        assert_eq!(stable_roster(&after),roster,"actual Retreat preserves every prepared equipment/fragment copy");
        command(&mut trial.state.application,player,json!({"type":"confirm_enter_node"}),Default::default());
        assert_eq!(stable_roster(&run(&trial.state.application)),roster);
    }
    let third=run(&trial.state.application);
    assert_eq!(third["game_state_context"]["abnormality_attempt"]["attempts_started"],3);
    assert_eq!(third["game_state_context"]["abnormality_attempt"]["retreats_used"],2);
    assert_eq!(third["game_state_context"]["can_retreat"],false);
    let third_checkpoint=trial.state.application.export_active_run_checkpoint().unwrap().unwrap();
    assert_eq!(retreat(&mut trial.state.application,player).unwrap_err().code(),"invalid_action");
    assert_eq!(run(&trial.state.application),third,"third retreat refusal must preserve the live bulk fixture");
    assert_eq!(trial.state.application.export_active_run_checkpoint().unwrap().unwrap().core_payload_json(),
        third_checkpoint.core_payload_json());
}
