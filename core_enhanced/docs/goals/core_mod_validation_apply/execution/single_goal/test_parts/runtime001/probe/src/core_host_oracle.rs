//! Normal public start flow through the approved Core host facade.
//! This does not replace the separate normal server/persistence integration test.
use game_core::{application::{ApplicationCommand,ApplicationSnapshotKind,CoreApplication},
    game::behavior::{EmployeeRosterSnapshotDto,GameStateContextDto,RunSnapshotDto},
    mod_validation::{ApprovedHost,RequiredConfiguration}};
use serde_json::{json,Value};
use std::{error::Error,fs::{self,OpenOptions},io::Write,path::Path};
use uuid::Uuid;
pub const PLAYER:Uuid=Uuid::from_u128(0xc0de1100000000000000000000000701);
fn save(path:&Path,value:&Value)->Result<(),Box<dyn Error>> {
    let mut bytes=serde_json::to_vec_pretty(value)?;bytes.push(b'\n');
    OpenOptions::new().write(true).create_new(true).open(path)?.write_all(&bytes)?;Ok(())
}
pub fn snapshot(app:&CoreApplication)->Result<Value,Box<dyn Error>> {
    let mut result=serde_json::Map::new();
    for (name,kind) in [("run",ApplicationSnapshotKind::Run),("roster",ApplicationSnapshotKind::EmployeeRoster),
        ("inventory",ApplicationSnapshotKind::Inventory)] {
        let snapshot=app.snapshot(kind).map_err(|e|format!("public {name} snapshot: {e:?}"))?;
        result.insert(name.into(),snapshot.read_model().as_value().clone());
    }
    Ok(Value::Object(result))
}
pub fn start(host:&ApprovedHost,app:&mut CoreApplication,mut required:RequiredConfiguration,out:&Path)
    ->Result<RequiredConfiguration,Box<dyn Error>> {
    fs::create_dir(out)?;
    let mut next=json!({"type":"start_new_game","game_mode":"Standard","replace_active_run":false});
    for index in 0..128 {
        let command=ApplicationCommand::from_document(next.clone().into()).map_err(|e|format!("command parse: {e:?}"))?;
        let prepared=host.prepare_with_future_generation(app,&required,PLAYER,command,Default::default());
        if let Err(error)=&prepared {save(&out.join(format!("{index:03}-failed.json")),&json!({"command":next,"error":error}))?;}
        let (prepared,accepted_required)=prepared.map_err(|e|format!("normal approved preparation: {e:?}"))?;
        let result=prepared.commit();required=accepted_required;
        let actual=snapshot(app)?;
        save(&out.join(format!("{index:03}-accepted.json")),&json!({"command":next,"result":result.payload().as_value(),
            "required":required,"snapshots":actual}))?;
        assert_eq!(app.content_identity(),&required.content_identity);
        let run:RunSnapshotDto=serde_json::from_value(actual["run"].clone())?;
        if let Some(vn)=run.visual_novel {
            next=if let Some(choice)=vn.choices.iter().find(|choice|choice.selectable) {
                json!({"type":"select_visual_novel_choice","session_id":vn.session_id,
                    "current_scene_id":vn.current_scene_id,"choice_id":choice.choice_id})
            }else{json!({"type":"advance_visual_novel","session_id":vn.session_id,"current_scene_id":vn.current_scene_id})};
            continue;
        }
        if let GameStateContextDto::SelectingStarterEmployees{required_count,candidates}=run.game_state_context {
            let ids:Vec<_>=candidates.into_iter().take(required_count).map(|row|row.employee_definition_id).collect();
            assert_eq!(ids.len(),required_count);
            next=json!({"type":"select_starter_employees","employee_definition_ids":ids});continue;
        }
        let roster:EmployeeRosterSnapshotDto=serde_json::from_value(actual["roster"].clone())?;
        assert!(roster.combat_eligible_employee_ids.len()>=2,"actual normal starter roster required");
        return Ok(required);
    }
    Err("actual normal start exceeded independent finite command bound".into())
}

// Running A rejects another Start regardless of replacement intent. A separate
// A shell held from before Apply remains eligible to consume future B. This is
// a Core generation oracle; durable saved-run replacement is tested separately.
pub fn verify_generation_switch(host:&ApprovedHost,active_a:&mut CoreApplication,
    dormant_a:&mut CoreApplication,required:&RequiredConfiguration,
    expected_future:&RequiredConfiguration,out:&Path)->Result<(),Box<dyn Error>> {
    fs::create_dir(out)?;let active_before=snapshot(active_a)?;let dormant_before=snapshot(dormant_a)?;
    assert!(!active_a.is_not_started());assert!(dormant_a.is_not_started());
    assert_eq!(active_a.content_identity(),&required.content_identity);
    assert_eq!(dormant_a.content_identity(),&required.content_identity);
    let command=|replace|ApplicationCommand::from_document(json!({"type":"start_new_game",
        "game_mode":"Standard","replace_active_run":replace}).into()).map_err(|e|format!("command parse: {e:?}"));
    for replace in [false,true] {
        let denied=host.prepare_with_future_generation(active_a,required,PLAYER,command(replace)?,Default::default())
            .err().expect("running A cannot start again in its current game context");
        save(&out.join(format!("running_a_start_{replace}_denied.json")),&json!({"error":denied}))?;
        assert_eq!(active_a.content_identity(),&required.content_identity);assert_eq!(snapshot(active_a)?,active_before);
        assert_eq!(snapshot(dormant_a)?,dormant_before);
    }
    {
        let (draft,draft_required)=host.prepare_with_future_generation(dormant_a,required,PLAYER,command(false)?,Default::default())
            .map_err(|e|format!("eligible shell draft: {e:?}"))?;
        assert_eq!(&draft_required,expected_future);
        save(&out.join("uncommitted_draft.json"),&json!({"required":draft_required,"result":draft.result().payload().as_value()}))?;
        drop(draft);
    }
    assert_eq!(dormant_a.content_identity(),&required.content_identity);assert_eq!(snapshot(dormant_a)?,dormant_before);
    assert_eq!(active_a.content_identity(),&required.content_identity);assert_eq!(snapshot(active_a)?,active_before);
    let (draft,committed_required)=host.prepare_with_future_generation(dormant_a,required,PLAYER,command(false)?,Default::default())
        .map_err(|e|format!("eligible shell commit preparation: {e:?}"))?;
    let result=draft.commit();
    assert_eq!(&committed_required,expected_future);assert_eq!(dormant_a.content_identity(),&expected_future.content_identity);
    assert!(!dormant_a.is_not_started());
    assert_eq!(active_a.content_identity(),&required.content_identity);assert_eq!(snapshot(active_a)?,active_before);
    save(&out.join("committed_future_generation.json"),&json!({"required":committed_required,
        "result":result.payload().as_value(),"snapshots":snapshot(dormant_a)?,"held_running_a":snapshot(active_a)?}))?;
    Ok(())
}
